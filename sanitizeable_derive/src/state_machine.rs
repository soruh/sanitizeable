use crate::datatypes::{Attrs, FieldTokenStreams, Fields, Names};
use crate::states::{
    Absorbing, CalculateNames, Init, Intermediate, QuoteFields, SplitFieldsByPrivacy,
    SplitStructAttributes,
};
use proc_macro::Diagnostic;
use quote::quote;
use syn::{AttributeArgs, ItemStruct};

use crate::util::{
    build_phantom_fields, derive_names, distribute_attributes, split_attrs,
    split_fields_by_privacy, wrap_fields_in_parens,
};

pub fn run(args: AttributeArgs, input: ItemStruct) -> proc_macro2::TokenStream {
    if input.fields.is_empty() {
        Diagnostic::new(proc_macro::Level::Error, "struct has no fields").emit();
    }

    Init { args, input }.finish()
}

impl Intermediate for Init {
    type Output = CalculateNames;
    fn next(self) -> Self::Output {
        CalculateNames {
            names: derive_names(self.input.ident.clone(), &self.args),
            input: self.input,
        }
    }
}
impl Intermediate for CalculateNames {
    type Output = SplitStructAttributes;
    fn next(self) -> Self::Output {
        SplitStructAttributes {
            struct_attrs: split_attrs(&self.input.attrs),
            input: self.input,
            names: self.names,
        }
    }
}
impl Intermediate for SplitStructAttributes {
    type Output = SplitFieldsByPrivacy;
    fn next(self) -> Self::Output {
        SplitFieldsByPrivacy {
            fields: distribute_attributes(split_fields_by_privacy(&self.input.fields)),
            input: self.input,
            names: self.names,
            struct_attrs: self.struct_attrs,
        }
    }
}

impl Intermediate for SplitFieldsByPrivacy {
    type Output = QuoteFields;
    fn next(self) -> Self::Output {
        let Fields {
            public_fields,
            private_fields,
            phantom_fields,
        } = self.fields;

        let phantom = build_phantom_fields(phantom_fields);

        let fields = FieldTokenStreams {
            private_fields: quote! { #(#private_fields,)* },
            public_fields: quote! { #(#public_fields,)* #phantom },
        };

        let fields = wrap_fields_in_parens(fields, &self.input.fields);

        QuoteFields {
            input: self.input,
            names: self.names,
            struct_attrs: self.struct_attrs,
            fields,
        }
    }
}
impl Absorbing for QuoteFields {
    fn finish(self) -> proc_macro2::TokenStream {
        let QuoteFields {
            input:
                ItemStruct {
                    vis,
                    generics,
                    semi_token,
                    ..
                },
            names:
                Names {
                    private_name,
                    public_name,
                    union_name,
                    container_name,
                },
            struct_attrs:
                Attrs {
                    private_attrs,
                    public_attrs,
                    normal_attrs,
                    ..
                },
            fields:
                FieldTokenStreams {
                    private_fields,
                    public_fields,
                },
        } = self;

        quote! {
            #(#private_attrs)*
            #(#normal_attrs)*
            #[repr(C)]
            #vis struct #private_name #generics #private_fields #semi_token


            #(#public_attrs)*
            #(#normal_attrs)*
            #[repr(C)]
            #vis struct #public_name #generics #public_fields  #semi_token

            union #union_name #generics {
                private: core::mem::ManuallyDrop<#private_name #generics>,
                public: core::mem::ManuallyDrop<#public_name #generics>,
            }


            #[repr(transparent)]
            #vis struct #container_name #generics (#union_name #generics);


            impl #generics core::ops::Drop for #container_name #generics {
                fn drop(&mut self) {
                    /// Safety:
                    /// - Since `private` always contains all fields we can drop the whole structure by dropping `private`
                    /// - We ensure that `Drop` is only run if dropping `self.private` is still our responsibility (see `into_private`)
                    ///
                    /// We can run `core::mem::ManuallyDrop::drop` safely, since `self` can not be accessed after `drop`
                    /// and has not yet been dropped (see above). We can thus ensure that `core::mem::ManuallyDrop::drop` is only
                    /// called once
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
                    /// Safety:
                    /// - We ensure that `std::mem::ManuallyDrop` has not yet been dropped (see `into_private` and `impl Drop`)
                    /// - The fields of `public` are a strict subset of `private` and are in the same order.
                    /// It is thus safe to access and modify `public` without invalidating `private`
                    unsafe { &*self.0.public }
                }
                fn public_mut(&mut self) -> &mut Self::Public {
                    /// Safety:
                    /// see `public`
                    unsafe { &mut *self.0.public }
                }
                fn private(&self) -> &Self::Private {
                    /// Safety:
                    /// - We ensure that `std::mem::ManuallyDrop` has not yet been dropped (see `into_private` and `impl Drop`)
                    /// - The fields of `public` are a strict subset of `private` and are in the same order.
                    /// It is thus safe to access and modify `private` without invalidating `public`
                    unsafe { &*self.0.private }
                }
                fn private_mut(&mut self) -> &mut Self::Private {
                    /// Safety:
                    /// see `private`
                    unsafe { &mut *self.0.private }
                }
                fn into_private(self) -> Self::Private {
                    /// Safety:
                    /// - `std::mem::ManuallyDrop::drop` has not yet been called, since self still exists
                    ///     -> We can call `std::mem::ManuallyDrop::into_inner`
                    ///     - we `core::mem::forget(self);` to make sure that `Drop` does not run and drop `private` twice
                    /// - `Self` is `#[repr(transparent)]` which makes it safe to cast to it's inner value
                    let inner = unsafe {
                        let ptr = &self
                            as *const #container_name #generics
                            as *const #union_name     #generics;


                        // Read the inner value ("cast" `self` to `#union_name`)
                        let value = ptr.read();

                        // `core::mem::forget(self)` to skip running it's `Drop` implementation
                        // This is done after the `ptr.read()` to ensure that the data pointed to by `ptr`
                        // is valid during the read
                        core::mem::forget(self);

                        value
                    };
                    core::mem::ManuallyDrop::into_inner(unsafe {inner.private})
                }
            }
        }
    }
}
