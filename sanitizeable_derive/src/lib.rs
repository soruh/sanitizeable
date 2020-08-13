#![feature(proc_macro_diagnostic)]
#![deny(clippy::pedantic)]

mod datatypes;
mod state_machine;
mod states;
mod util;

use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sanitizeable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    state_machine::run(parse_macro_input!(args), parse_macro_input!(input)).into()
}
