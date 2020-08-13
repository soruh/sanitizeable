#![feature(untagged_unions)]

use sanitizeable::{sanitizeable, Sanitizeable};

#[sanitizeable(
    private_name = "Theft",
    public_name = "Product",
    container_name = "ProductContainer",
    union_name = "_ProductUnion", // You probably only want to use this if the automatic name causes conflicts
)]
#[derive(Clone)]
#[public_attr::derive(Copy)]
struct ThisNameIsPrettyIrrelevantDueToOurAttributes {
    name: &'static str,
    #[private_attr::doc = "This is the extremly inflated price"]
    // This attribute only shows up on the private variant
    price: f64,
    #[private]
    worth: f64,
    #[private]
    manager_message: String,
}

impl ProductContainer {
    fn new(name: &'static str, price: f64, worth: f64, manager_message: &str) -> Self {
        // TODO: <Self as Sanitizeable>::Private
        Self::from_private(Theft::new(name, price, worth, manager_message))
    }
}

impl Theft {
    fn new(name: &'static str, price: f64, worth: f64, manager_message: &str) -> Self {
        Self {
            name,
            price,
            worth,
            manager_message: manager_message.to_string(),
        }
    }
    fn markup(&self) -> f64 {
        self.price - self.worth
    }
}

impl core::fmt::Display for Theft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The product \"{}\" costs ${:.2} and is worth ${:.2}. We gain ${:.2} ({}% of worth)",
            self.name,
            self.price,
            self.worth,
            self.markup(),
            self.markup() * 100. / self.worth,
        )
    }
}

impl core::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The product \"{}\" costs ${:.2}", self.name, self.price,)
    }
}

fn main() {
    let product = ProductContainer::new("Printer Ink Cartrige", 24.50, 0.50, "Work harder!");

    println!("{}", product.public());
    println!("{}", product.private());
}
