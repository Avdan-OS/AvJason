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

pub mod common;

mod macro_test {
    use avjason_macros::{ECMARef, SpecRef};

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