#![feature(untagged_unions)]

use sanitizeable::{sanitizeable, Sanitizeable};

// This file should not compile

#[sanitizeable]
#[derive(Debug)]
struct UndefinedBehaviour {
    #[private]
    field_a: String,

    #[public_attr::cfg(all(target_os = "windows", target_os = "linux"))]
    // This would break internal layout guarantees
    field_b: String,
    field_c: u64,
}

unsafe fn null_pointer_dereference() {
    let mut undef = UndefinedBehaviour::from_private(UndefinedBehaviourPrivate {
        field_a: "Safe".into(),
        field_b: "Broken".into(),
        field_c: 0,
    });

    dbg!(undef.private());
    dbg!(undef.public());

    undef.public_mut().field_c = 0;

    dbg!(undef.public());
    dbg!(undef.private()); // Segfault
}

fn main() {
    unsafe { null_pointer_dereference() }
}
