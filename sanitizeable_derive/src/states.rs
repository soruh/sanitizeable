use crate::datatypes::{Attrs, FieldTokenStreams, Fields, Names};
use syn::{AttributeArgs, ItemStruct};

pub trait Intermediate {
    type Output;
    fn next(self) -> Self::Output;
}

pub trait Absorbing {
    fn finish(self) -> proc_macro2::TokenStream;
}

impl<S: Intermediate> Absorbing for S
where
    S::Output: Absorbing,
{
    fn finish(self) -> proc_macro2::TokenStream {
        self.next().finish()
    }
}

pub struct Init {
    pub args: AttributeArgs,
    pub input: ItemStruct,
}

pub struct CalculateNames {
    pub input: ItemStruct,
    pub names: Names,
}

pub struct SplitStructAttributes {
    pub input: ItemStruct,
    pub names: Names,
    pub struct_attrs: Attrs,
}

pub struct SplitFieldsByPrivacy {
    pub input: ItemStruct,
    pub names: Names,
    pub struct_attrs: Attrs,
    pub fields: Fields,
}

pub struct QuoteFields {
    pub input: ItemStruct,
    pub names: Names,
    pub struct_attrs: Attrs,
    pub fields: FieldTokenStreams,
}
