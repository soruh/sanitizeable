pub use sanitizeable_derive::*;

pub trait Sanitizeable: Sized {
    type Public;
    type Private;

    fn from_private(private: Self::Private) -> Self;

    fn public(&self) -> &Self::Public;
    fn public_mut(&mut self) -> &mut Self::Public;

    fn private(&self) -> &Self::Private;
    fn private_mut(&mut self) -> &mut Self::Private;

    fn into_private(self) -> Self::Private;
}
