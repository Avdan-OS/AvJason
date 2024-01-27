//!
//! ## AvJason
//! > A child of the [AvdanOS](https://github.com/Avdan-OS) project.
//!
//! A parser for [JSON5](https://json5.org/).
//!
//! ## Why?
//! This crate provides a very important function: traceability.
//! ### Tracability
//! This allows for line-column data to be preserved so that further
//! processing can benefit from spanned errors, which tell the end
//! user *where* the error happened.
//!

// This will have to be removed to solve #5
#![allow(incomplete_features)]
#![feature(adt_const_params)]

pub mod common;
pub mod lexing;

mod macro_test {
    use std::marker::PhantomData;

    use avjason_macros::{ECMARef, Spanned, SpecRef};

    use crate::common::Span;

    #[SpecRef("Identifier", "JSON5Identifier")]
    #[allow(unused)]
    struct Identifier;

    #[SpecRef("JSON5Null")]
    #[allow(unused)]
    struct Null;

    #[ECMARef("BooleanLiteral", "https://262.ecma-international.org/5.1/#sec-7.8.2")]
    #[allow(unused)]
    struct LitBool;
}