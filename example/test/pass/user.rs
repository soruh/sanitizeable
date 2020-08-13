#![feature(untagged_unions)]

use sanitizeable::{sanitizeable, Sanitizeable};

// One way to use this
#[sanitizeable]
#[derive(Debug)]
#[private_attr::derive(PartialEq)] // This could be serde::Serialize
#[public_attr::derive(Clone)]
struct User {
    pub name: String,
    pub address: String,
    pub username: String,

    #[private]
    pub pin: Option<u64>,
    #[private]
    pub social_security_number: String,

    pub id: u32,
    pub score: f64,
    pub birthday: (u16, u8, u8),
}

impl std::fmt::Display for UserPublic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " public: User [{}] {} ({}) lives at \"{}\" was born on {}.{}.{} and has a score of {}.",
            self.id,
            self.name,
            self.username,
            self.address,
            self.birthday.2,
            self.birthday.1,
            self.birthday.0,
            self.score
        )
    }
}

impl std::fmt::Display for UserPrivate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "private: User [{}] {} ({}) lives at \"{}\" was born on {}.{}.{} and has a score of {}. pin: {}, ssn: {}",
            self.id,
            self.name,
            self.username,
            self.address,
            self.birthday.2,
            self.birthday.1,
            self.birthday.0,
            self.score,
            self.pin.map(|x| x.to_string()).unwrap_or_else(|| "N/A".to_string()),
            self.social_security_number,
        )
    }
}

fn change_birthday(user: &mut <User as Sanitizeable>::Public, new_birthday: (u16, u8, u8)) {
    user.birthday = new_birthday;
}

impl UserPrivate {
    fn reset_pin(&mut self) -> &mut Self {
        self.pin = None;
        self
    }
    fn add_to_score(&mut self, additional_score: f64) -> &mut Self {
        self.score += additional_score;
        self
    }
}

fn main() {
    let mut user = User::from_private(UserPrivate {
        name: "Max Musterman".into(),
        address: "Example Street, 64d".into(),
        username: "max_1123".into(),
        birthday: (1970, 4, 19),
        id: 32710,
        score: 420.69,

        pin: Some(1234),
        social_security_number: "001-01-0001".into(),
    });

    println!("{}", user.public());
    println!("{}", user.private());

    change_birthday(user.public_mut(), (2000, 4, 20));
    user.private_mut().reset_pin().add_to_score(16.5);

    let public_copy: UserPublic = user.public().clone(); // Note: we can't have into_public due to how `Drop` works

    println!();

    println!("{}", user.into_private()); // Note: we `can` have into_private since I did some very ugly things to make it work (idk. if it actually does)
    println!("{}", public_copy);

    // `user` has been moved
}
