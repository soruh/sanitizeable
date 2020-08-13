#![feature(untagged_unions)]
use sanitizeable::{sanitizeable, Sanitizeable};

#[sanitizeable]
#[derive(Debug)]
#[public_attr::derive(serde::Serialize)] // This only derives `serde::Serialize` for the public variant
struct User {
    name: String,
    // This attrribute is only applied to the field on the private type
    #[private_attr::doc = "This is the user's email, make sure to send them a lot of spam"]
    email: String,
    #[private]
    pin: u16,
}

// using `UserPrivate` here would not compile since it does not implement `serde::Serialize`
fn send_user_information<W: std::io::Write>(writer: &mut W, user_data: &UserPublic) {
    eprintln!("sending user {}", user_data.name);
    serde_json::to_writer(writer, user_data).unwrap();

    // dbg!(user_data.pin); // This would not compile
}

fn change_pin(user: &mut UserPrivate, new_pin: u16) {
    user.pin = new_pin;
}

fn main() {
    let mut user_buffer = Vec::new();

    let mut user = User::from_private(UserPrivate {
        name: "A user".into(),
        email: "some@email.com".into(),
        pin: 1337,
    });

    send_user_information(&mut user_buffer, user.public());

    change_pin(user.private_mut(), 42);

    dbg!(user.public());
}
