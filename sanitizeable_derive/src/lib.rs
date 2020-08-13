#![feature(proc_macro_diagnostic)]
#![deny(clippy::pedantic)]

mod datatypes;
mod state_machine;
mod states;
mod util;

use syn::parse_macro_input;

/// Derive the Serializeable trait and create all required structs
///
/// Use this to create two different `struct`s, one with all fields
/// and one which only has the fields that are not marked as `#[private]`.
///
/// This macro additionally creates a container `struct` which is used to access the data
/// and a `union` which you should not use directly.
/// The default names for these types are derived from the name of your `struct`
///
///
/// A `struct` called `Test` will result in the following `struct`s being created:
/// - public: `TestPublic`
/// - private: `TestPrivate`
/// - container: `Test`
/// - union: `TestUnion`
///
///
/// You can change the names of all of these by using the following attributes:
/// - `#[public_name = "..."]`
/// - `#[private_name = "..."]`
/// - `#[container_name = "..."]`
/// - `#[union_name = "..."]`
///
///
/// You can also apply attributes to only one of the variants by using
/// - `#[public_attr::your_attribute]`
/// - `#[private_attr::your_attribute]`
///
///
/// Note that this works both on the whole struct as well as on specific fields
/// You are however **not** able use the `cfg` attribute, since that would break internal layout guarantees.
///
/// To use the resulting types you need to import the `Sanitizeable` trait.
/// You can then call the `public`, `public_mut`, `private`, `private_mut` and `into_private` methods on
/// the container type.
///
/// Contructing the container type can be done by using the `from_private` method defined on it.
///
/// There is not currenty a `into_public` method, since that is pretty difficult to do due to how `Drop` works.
/// This functionality might be added in the future.
#[proc_macro_attribute]
pub fn sanitizeable(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    state_machine::run(parse_macro_input!(args), parse_macro_input!(input)).into()
}
