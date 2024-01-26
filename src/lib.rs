//!
//! ## AvJason
//! > A child of the [AvdanOS](https://github.com/Avdan-OS) project.
//! 
//! A parser for [JSON5](https://json5.org/).
//! 

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