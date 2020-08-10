#![feature(proc_macro_diagnostic)]

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, Field, FieldsNamed, FieldsUnnamed, Ident,
    ItemStruct, Lit, Meta, MetaNameValue, NestedMeta,
};

fn check_privacy(field: &mut Field) -> bool {
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

fn build_remaining_attr(
    segments: syn::punctuated::Iter<syn::PathSegment>,
) -> syn::punctuated::Punctuated<syn::PathSegment, syn::Token!(::)> {
    let last_segment = &segments.clone().last().expect("Empty attribute").ident;
    if last_segment.to_string().as_str() == "cfg" {
        proc_macro::Diagnostic::spanned(
            last_segment.span().unwrap(),
            proc_macro::Level::Error,
            "You may not use #[cfg(...)] in an attribute that is only applied to some variants",
        )
        .emit();
    }

    segments.cloned().collect()
}

fn split_attrs(
    attrs: Vec<Attribute>,
) -> (
    Vec<Attribute>,
    Vec<Attribute>,
    Vec<Attribute>,
    Vec<Attribute>,
) {
    let mut private_attrs = Vec::new();
    let mut public_attrs = Vec::new();
    let mut normal_attrs = Vec::new();
    let mut phantom_attrs = Vec::new();

    for mut attr in attrs {
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

    (private_attrs, public_attrs, normal_attrs, phantom_attrs)
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

fn derive_names(input: Ident, attrs: AttributeArgs) -> (Ident, Ident, Ident, Ident) {
    let span = input.span();
    let private_name = name_attr!(attrs, "private_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Private", input), span));
    let public_name = name_attr!(attrs, "public_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Public", input), span));
    let union_name = name_attr!(attrs, "union_name")
        .unwrap_or_else(|| Ident::new(&format!("{}Union", input), span));
    let container_name = name_attr!(attrs, "container_name").unwrap_or_else(|| input);

    (private_name, public_name, union_name, container_name)
}

fn field_with_attrs(mut field: Field, mut attrs: Vec<Vec<Attribute>>) -> Field {
    field.attrs = attrs.pop().unwrap_or_else(Vec::new);
    for attr_args in attrs {
        field.attrs.extend(attr_args);
    }
    field
}

#[proc_macro_attribute]
pub fn sanitizeable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let semi_token = input.semi_token;

    let visibility = input.vis;
    let generics = input.generics;

    let (private_attrs, public_attrs, normal_attrs, _phantom_attrs) = split_attrs(input.attrs);

    let (private_name, public_name, union_name, container_name) =
        derive_names(input.ident.clone(), args);

    let mut phantom_fields = Vec::new();

    if input.fields.is_empty() {
        proc_macro::Diagnostic::spanned(
            vec![
                input.struct_token.span.unwrap(),
                input.ident.span().unwrap(),
            ],
            proc_macro::Level::Error,
            "struct has no fields",
        )
        .emit();
    }

    let mut private = Vec::new();
    let mut public = Vec::new();

    for mut field in input.fields.clone() {
        let is_private = check_privacy(&mut field);

        if is_private {
            private.push(field);
        } else {
            public.push(field);
        }
    }

    let mut private_fields: Vec<Field> = Vec::new();
    let mut public_fields: Vec<Field> = Vec::new();

    for field in public {
        let (private_attrs, public_attrs, normal_attrs, _) = split_attrs(field.attrs.clone());

        public_fields.push(field_with_attrs(
            field.clone(),
            vec![public_attrs, normal_attrs.clone()],
        ));
        private_fields.push(field_with_attrs(
            field,
            vec![private_attrs.clone(), normal_attrs.clone()],
        ));
    }

    for field in private {
        let (private_attrs, _, normal_attrs, phantom_attrs) = split_attrs(field.attrs.clone());

        phantom_fields.push(field_with_attrs(
            field.clone(),
            vec![phantom_attrs, normal_attrs.clone()],
        ));
        private_fields.push(field_with_attrs(field, vec![private_attrs, normal_attrs]));
    }

    if phantom_fields.is_empty() {
        proc_macro::Diagnostic::spanned(
            vec![
                input.struct_token.span.unwrap(),
                input.ident.span().unwrap(),
            ],
            proc_macro::Level::Warning,
            "struct has no private fields",
        )
        .emit();
    }

    let phantom = if phantom_fields.is_empty() {
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
    };

    let mut private_fields = quote! {
        #(#private_fields,)*
    };

    let mut public_fields = quote! {
        #(#public_fields,)*
        #phantom
    };

    // dbg!(private_fields.to_string());
    // dbg!(public_fields.to_string());

    match &input.fields {
        syn::Fields::Named(FieldsNamed { brace_token, .. }) => {
            let mut new_private_fields = proc_macro2::TokenStream::new();
            brace_token.surround(&mut new_private_fields, |f| *f = private_fields);
            private_fields = new_private_fields;

            let mut new_public_fields = proc_macro2::TokenStream::new();
            brace_token.surround(&mut new_public_fields, |f| *f = public_fields);
            public_fields = new_public_fields;
        }
        syn::Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => {
            let mut new_private_fields = proc_macro2::TokenStream::new();
            paren_token.surround(&mut new_private_fields, |f| *f = private_fields);
            private_fields = new_private_fields;

            let mut new_public_fields = proc_macro2::TokenStream::new();
            paren_token.surround(&mut new_public_fields, |f| *f = public_fields);
            public_fields = new_public_fields;
        }
        syn::Fields::Unit => {
            assert!(private_fields.is_empty());
            assert!(public_fields.is_empty());
        }
    };

    // println!("private_fields: {}", private_fields);
    // println!("public_fields: {}", public_fields);

    let expanded = quote! {
        #(#private_attrs)*
        #(#normal_attrs)*
        #[repr(C)]
        #visibility struct #private_name #generics #private_fields #semi_token


        #(#public_attrs)*
        #(#normal_attrs)*
        #[repr(C)]
        #visibility struct #public_name #generics #public_fields  #semi_token

        union #union_name #generics {
            private: core::mem::ManuallyDrop<#private_name #generics>,
            public: core::mem::ManuallyDrop<#public_name #generics>,
        }


        #[repr(transparent)]
        #visibility struct #container_name #generics (#union_name #generics);


        impl #generics core::ops::Drop for #container_name #generics {
            fn drop(&mut self) {
                unsafe { core::mem::ManuallyDrop::drop(&mut self.0.private); }
            }
        }

        impl #generics ::sanitizeable::Sanitizeable for #container_name #generics {
            type Public = #public_name #generics;
            type Private = #private_name #generics;

            fn from_private(private: Self::Private) -> Self {
                Self(#union_name {
                    private: core::mem::ManuallyDrop::new(private),
                })
            }
            fn public(&self) -> &Self::Public {
                unsafe { &*self.0.public }
            }
            fn public_mut(&mut self) -> &mut Self::Public {
                unsafe { &mut *self.0.public }
            }
            fn private(&self) -> &Self::Private {
                unsafe { &*self.0.private }
            }
            fn private_mut(&mut self) -> &mut Self::Private {
                unsafe { &mut *self.0.private }
            }
            fn into_private(self) -> Self::Private {
                let inner = unsafe {
                    let ptr = &self
                        as *const #container_name #generics
                        as *const #union_name     #generics;

                    let value = ptr.read();
                    core::mem::forget(self);
                    value
                };
                core::mem::ManuallyDrop::into_inner(unsafe {inner.private})
            }
        }
    };

    // println!("expanded: {}", expanded);

    proc_macro::TokenStream::from(expanded)
}
