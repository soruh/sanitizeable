#![feature(untagged_unions)]

use sanitizeable::{sanitizeable, Sanitizeable};

// Test generics

#[sanitizeable]
#[derive(Debug)]
struct GenericStruct<'name, T>
where
    T: Sized + Debug,
{
    pub name: &'name str,
    #[private]
    pub value: T,
}

// Test unnamed fields

#[sanitizeable]
#[derive(Debug)]
struct Unnamed(u64, #[private] f64);


fn main() {
    let generic = GenericStruct::from_private(GenericStructPrivate::<u8> {
        name: "Some name",
        value: 5
    });

    dbg!(generic.private());
    dbg!(generic.public());

    let u = Unnamed::from_private(UnnamedPrivate(124131, 12.5));

    dbg!(u.private());
    dbg!(u.public());
}