#![feature(drain_filter)]

use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, AttributeArgs, Field, Ident, ItemStruct, Lit, Meta,
    MetaNameValue, NestedMeta,
};

fn check_privacy(field: &mut Field) -> bool {
    let private_fields: Vec<_> = field
        .attrs
        .drain_filter(|attr| attr.path.segments.first().unwrap().ident == "private")
        .collect();

    !private_fields.is_empty()
}

fn split_attrs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>, Vec<Attribute>) {
    let mut private_attrs = Vec::new();
    let mut public_attrs = Vec::new();
    let mut normal_attrs = Vec::new();

    for mut attr in attrs {
        let mut segments = attr.path.segments.iter();
        if let Some(first) = segments.next() {
            if first.ident == "private_attr" {
                attr.path.segments = segments.cloned().collect();
                private_attrs.push(attr);
                continue;
            }

            if first.ident == "public_attr" {
                attr.path.segments = segments.cloned().collect();
                public_attrs.push(attr);
                continue;
            }
        }

        core::mem::drop(segments);

        normal_attrs.push(attr);
    }

    (private_attrs, public_attrs, normal_attrs)
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

#[proc_macro_attribute]
pub fn sanitizeable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input = parse_macro_input!(input as ItemStruct);

    let visibility = input.vis;
    let generics = input.generics; // TODO: add PhantomData if needed?

    let (private_attrs, public_attrs, normal_attrs) = split_attrs(input.attrs);

    let (private_name, public_name, union_name, container_name) = derive_names(input.ident, args);

    let mut private_fields = Vec::new();
    let mut public_fields = Vec::new();

    if input.fields.is_empty() {
        panic!("struct has no fields");
    }

    for mut field in input.fields {
        let is_private = check_privacy(&mut field);

        let (private_attrs, public_attrs, normal_attrs) = split_attrs(field.attrs.clone());

        if !is_private {
            let mut public_field = field.clone();
            public_field.attrs = public_attrs;
            public_field.attrs.extend(normal_attrs.clone());
            public_fields.push(public_field);
        }

        let mut private_field = field;
        private_field.attrs = private_attrs;
        private_field.attrs.extend(normal_attrs);
        private_fields.push(private_field);
    }

    if private_fields.is_empty() {
        println!("struct has no private fields");
    }

    let expanded = quote! {
        #(#private_attrs)*
        #(#normal_attrs)*
        #[repr(C)]
        #visibility struct #private_name #generics {
            #(#private_fields,)*
        }


        #(#public_attrs)*
        #(#normal_attrs)*
        #[repr(C)]
        #visibility struct #public_name #generics {
            #(#public_fields,)*
        }

        union #union_name {
            private: core::mem::ManuallyDrop<#private_name>,
            public: core::mem::ManuallyDrop<#public_name>,
        }


        #[repr(transparent)]
        #visibility struct #container_name(#union_name);


        impl core::ops::Drop for #container_name {
            fn drop(&mut self) {
                unsafe {
                    core::mem::ManuallyDrop::drop(&mut self.0.private);
                }
            }
        }

        impl ::sanitizeable::Sanitizeable for #container_name { // TODO
            type Public = #public_name;
            type Private = #private_name;

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
                core::mem::ManuallyDrop::into_inner(unsafe { std::mem::transmute::<_, #union_name>(self).private })
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
