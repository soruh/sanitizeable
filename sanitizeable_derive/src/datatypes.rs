use syn::{Attribute, Ident};

pub struct Attrs {
    pub private_attrs: Vec<Attribute>,
    pub public_attrs: Vec<Attribute>,
    pub normal_attrs: Vec<Attribute>,
    pub phantom_attrs: Option<Vec<Attribute>>,
}

pub struct Names {
    pub private_name: Ident,
    pub public_name: Ident,
    pub union_name: Ident,
    pub container_name: Ident,
}

pub struct Fields {
    pub private_fields: Vec<syn::Field>,
    pub public_fields: Vec<syn::Field>,
    pub phantom_fields: Vec<syn::Field>,
}

pub struct FieldTokenStreams {
    pub private_fields: proc_macro2::TokenStream,
    pub public_fields: proc_macro2::TokenStream,
}
