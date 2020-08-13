#![feature(proc_macro_diagnostic)]
#![warn(clippy::pedantic)]

mod state;
mod util;

use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sanitizeable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    state::run(parse_macro_input!(args), parse_macro_input!(input)).into()
}
