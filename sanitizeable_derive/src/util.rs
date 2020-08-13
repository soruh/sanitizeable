use crate::datatypes::{Attrs, FieldTokenStreams, Fields, Names};
use proc_macro::Diagnostic;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, Field, FieldsNamed, FieldsUnnamed, Ident, Lit, Meta, MetaNameValue, NestedMeta,
};

pub fn check_privacy(field: &mut Field) -> bool {
    // dbg!(quote! {#field}.to_string());

    let len_public = field.attrs.len();

    field.attrs = field
        .attrs
        .iter()
        .filter(|attr| attr.path.segments.first().unwrap().ident != "private")
        .cloned()
        .collect();

    field.attrs.len() < len_public
}

pub fn build_remaining_attr(
    segments: syn::punctuated::Iter<syn::PathSegment>,
) -> syn::punctuated::Punctuated<syn::PathSegment, syn::Token!(::)> {
    let last_segment = &segments.clone().last().expect("Empty attribute").ident;
    if last_segment.to_string().as_str() == "cfg" {
        Diagnostic::spanned(
            last_segment.span().unwrap(),
            proc_macro::Level::Error,
            "You may not use #[cfg(...)] in an attribute that is only applied to some variants",
        )
        .emit();
    }

    segments.cloned().collect()
}

pub fn split_attrs(attrs: &[Attribute]) -> Attrs {
    let mut private_attrs = Vec::new();
    let mut public_attrs = Vec::new();
    let mut normal_attrs = Vec::new();
    let mut phantom_attrs = Vec::new();

    for mut attr in attrs.iter().cloned() {
        let mut segments = attr.path.segments.iter();
        if let Some(first) = segments.next() {
            if first.ident == "private_attr" {
                attr.path.segments = build_remaining_attr(segments);
                private_attrs.push(attr);
                continue;
            }

            if first.ident == "public_attr" {
                attr.path.segments = build_remaining_attr(segments);
                public_attrs.push(attr);
                continue;
            }

            if first.ident == "phantom_attr" {
                attr.path.segments = build_remaining_attr(segments);
                phantom_attrs.push(attr);
                continue;
            }
        }

        core::mem::drop(segments);

        normal_attrs.push(attr);
    }

    Attrs {
        private_attrs,
        public_attrs,
        normal_attrs,
        phantom_attrs: Some(phantom_attrs),
    }
}

macro_rules! name_attr {
    ($attrs: ident, $key: literal) => {{
        let expected_ident = Ident::new($key, Span::call_site());
        // Check if the attr path is an assigment that has only $key as it's path
        // and return its string value (if it has one)
        $attrs.iter().find_map(|attr| match attr.clone() {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path:
                    syn::Path {
                        leading_colon: None,
                        segments,
                    },
                lit,
                ..
            })) if segments
                .iter()
                .map(|segment| &segment.ident)
                .collect::<Vec<_>>()
                == vec![&expected_ident] =>
            {
                match lit {
                    Lit::Str(ref string) => Some(Ident::new(&string.value(), lit.span())),
                    _ => None,
                }
            }
            _ => None,
        })
    }};
}

pub fn derive_names(input: Ident, attrs: &[NestedMeta]) -> Names {
    let span = input.span();
    let private_name = name_attr!(attrs, "private_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Private", input), span));
    let public_name = name_attr!(attrs, "public_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Public", input), span));
    let union_name = name_attr!(attrs, "union_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Union", input), span));
    let container_name = name_attr!(attrs, "container_name").unwrap_or_else(|| input);

    Names {
        private_name,
        public_name,
        union_name,
        container_name,
    }
}

pub fn field_with_attrs(mut field: Field, mut attrs: Vec<Vec<Attribute>>) -> Field {
    field.attrs = attrs.pop().unwrap_or_else(Vec::new);
    for attr_args in attrs {
        field.attrs.extend(attr_args);
    }
    field
}

pub fn split_fields_by_privacy(fields: &syn::Fields) -> Fields {
    let mut private_fields = Vec::new();
    let mut public_fields = Vec::new();

    for mut field in fields.clone() {
        let is_private = check_privacy(&mut field);
        if is_private {
            private_fields.push(field);
        } else {
            public_fields.push(field);
        }
    }

    Fields {
        private_fields,
        public_fields,
        phantom_fields: vec![],
    }
}

pub fn distribute_attributes(fields: Fields) -> Fields {
    let mut private_fields: Vec<Field> = Vec::new();
    let mut public_fields: Vec<Field> = Vec::new();
    let mut phantom_fields: Vec<Field> = Vec::new();

    for field in fields.public_fields {
        let attrs = split_attrs(&field.attrs);

        public_fields.push(field_with_attrs(
            field.clone(),
            vec![attrs.public_attrs, attrs.normal_attrs.clone()],
        ));
        private_fields.push(field_with_attrs(
            field,
            vec![attrs.private_attrs.clone(), attrs.normal_attrs.clone()],
        ));
    }

    for field in fields.private_fields {
        let attrs = split_attrs(&field.attrs);

        phantom_fields.push(field_with_attrs(
            field.clone(),
            vec![attrs.phantom_attrs.unwrap(), attrs.normal_attrs.clone()],
        ));
        private_fields.push(field_with_attrs(
            field,
            vec![attrs.private_attrs, attrs.normal_attrs],
        ));
    }

    if phantom_fields.is_empty() {
        Diagnostic::new(proc_macro::Level::Warning, "struct has no private fields").emit();
    }

    Fields {
        public_fields,
        private_fields,
        phantom_fields,
    }
}

pub fn build_phantom_fields(phantom_fields: Vec<Field>) -> proc_macro2::TokenStream {
    if phantom_fields.is_empty() {
        proc_macro2::TokenStream::new()
    } else {
        let mut names = Vec::new();
        let mut types = Vec::new();
        for field in phantom_fields {
            names.push(if let Some(ident) = field.ident {
                let ident = Ident::new(&format!("_{}", ident), ident.span());
                quote! {#ident: }
            } else {
                proc_macro2::TokenStream::new()
            });
            types.push(field.ty);
        }
        quote! {
            #(#names core::marker::PhantomData<#types>,)*
        }
    }
}

pub fn wrap_fields_in_parens(
    fields: FieldTokenStreams,
    input_fields: &syn::Fields,
) -> FieldTokenStreams {
    let FieldTokenStreams {
        private_fields,
        public_fields,
    } = fields;

    let mut out = FieldTokenStreams {
        private_fields: proc_macro2::TokenStream::new(),
        public_fields: proc_macro2::TokenStream::new(),
    };

    match &input_fields {
        syn::Fields::Named(FieldsNamed { brace_token, .. }) => {
            brace_token.surround(&mut out.private_fields, |f| *f = private_fields);
            brace_token.surround(&mut out.public_fields, |f| *f = public_fields);
        }
        syn::Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
            paren_token.surround(&mut out.private_fields, |f| *f = private_fields);
            paren_token.surround(&mut out.public_fields, |f| *f = public_fields);
        }
        syn::Fields::Unit => {
            assert!(private_fields.is_empty());
            assert!(public_fields.is_empty());
        }
    }

    out
}
