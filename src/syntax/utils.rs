//!
//! Utilities for parsing tokens.
//!

use crate::lex::tokens::Token;

#[allow(private_bounds)]
pub trait Peek: Sealed {}

#[doc(hidden)]
pub(crate) trait Sealed {
    type T;

    fn peek(&self, token: &Token) -> bool;
    fn try_from(&self, token: Token) -> Option<Self::T>;
}

impl<S> Peek for S
    where S: Sealed
{}

pub type Peeker<T> = (fn(&Token) -> Option<&T>, fn(Token) -> Option<T>);

impl<F, T1> Sealed for F
where
    F: Fn() -> Peeker<T1>,
{
    type T = T1;

    default fn peek(&self, token: &Token) -> bool {
        self().0(token).is_some()
    }

    default fn try_from(&self, token: Token) -> Option<Self::T> {
        self().1(token)
    }
}

pub enum Unparseable {}

default impl Sealed for fn(&Token) -> bool {
    type T = Unparseable;

    fn peek(&self, token: &Token) -> bool {
        self(token)
    }

    fn try_from(&self, token: Token) -> Option<Self::T> {
        unimplemented!()
    }
}
