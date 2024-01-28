//!
//! Macros for the main crate.
//!

#![feature(proc_macro_diagnostic, char_min)]

mod spanned;
mod type_traversal;
mod utils;
mod verbatim;

use proc_macro::{Diagnostic, Level, Span, TokenStream as Tokens};
use quote::ToTokens;
use spanned::{derive_spanned_for_enum, derive_spanned_for_struct};
use syn::parse_macro_input;
use utils::{get_item_attrs, ECMARef, JSON5Ref, ToRustdoc};
use verbatim::VerbatimPat;

///
/// ## SpecRef
///
/// Allows easy reference of the **JSON5** specification.
///
/// This macro will add an additional section at the top of the Rustdoc
/// for the item attached, linking to the relevant section in the specification.
///
/// ### Example
///
/// ```ignore
/// use crate::SpecRef;
///    
/// // With custom title.
/// #[SpecRef("Number", "JSON5Number")]
/// struct Number;
///
/// // Without custom title.
/// #[SpecRef("JSON5String")]
/// struct LitString;
/// ```
///
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn SpecRef(params: Tokens, target: Tokens) -> Tokens {
    let mut target: syn::Item = parse_macro_input!(target);
    let params: JSON5Ref = parse_macro_input!(params);
    let attrs = params.to_rustdoc();

    let Some(original_attrs) = get_item_attrs(&mut target) else {
        return syn::Error::new_spanned(target, "Cannot add spec ref to this item.")
            .into_compile_error()
            .into();
    };

    // Prepend our new documentation to the start of
    // the attribute macros.
    *original_attrs = attrs
        .into_iter()
        .chain(original_attrs.iter().cloned())
        .collect();

    target.into_token_stream().into()
}

///
/// ## ECMARef
///
/// Allows easy reference of the **ECMAScript** specification.
///
/// This macro will add an additional section at the top of the Rustdoc
/// for the item attached, linking to the relevant section in the specification.
///
/// ### Example
///
/// ```ignore
/// use crate::ECMARef;
///
/// // You must always include an acompanying URL.
/// #[ECMARef("NullLiteral", "https://262.ecma-international.org/5.1/#sec-7.8.1")]
/// struct LitNull;
/// ```
///
#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn ECMARef(params: Tokens, target: Tokens) -> Tokens {
    let mut target: syn::Item = parse_macro_input!(target);
    let params: ECMARef = parse_macro_input!(params);
    let attrs = params.to_rustdoc();

    let Some(original_attrs) = get_item_attrs(&mut target) else {
        return syn::Error::new_spanned(target, "Cannot add spec ref to this item.")
            .into_compile_error()
            .into();
    };

    // Prepend our new documentation to the start of
    // the attribute macros.
    *original_attrs = attrs
        .into_iter()
        .chain(original_attrs.iter().cloned())
        .collect();

    target.into_token_stream().into()
}

///
/// ## derive(Spanned)
///
/// Derives the Spanned trait for both structs and enums.
///
/// ### Terminal Tokens
/// ```ignore
/// ///
/// /// (1) Named span field.
/// ///
/// /// ASCII digit '0'..='9'.
/// ///
/// #[derive(Spanned)]
/// struct Digit {
///     letter: char,
///     span: Span,
/// }
///    
/// ///
/// /// (2) Tuple struct.
/// ///
/// /// Literally `.`
/// ///
/// #[derive(Spanned)]
/// struct Dot(Span);
/// ```
/// These are not composed of any smaller tokens. These *must* either:
/// 1. have a name `span: Span` field, or
/// 2. be a tuple struct with *only* a single Span field.
///
/// ***
///
/// ### Non-terminal Tokens
/// ```ignore
/// ///
/// /// (1.1) Named Struct
/// ///
/// /// A base-10 decimal number,
/// /// with optional integral part.
/// ///
/// #[derive(Spanned)]
/// struct Decimal {
///     integral: Many<Digit>,
///     point: Dot,
///     mantissa: AtLeast<1, Digit>
/// }
///
/// ///
/// /// (1.2) Tuple struct
/// ///
/// /// A base-10 integer.
/// ///
/// #[derive(Spanned)]
/// struct Integer(AtLeast<1, Digit>);
///
/// ///
/// /// (2.1) Enum (union of tokens).
/// ///
/// /// A number: either an integer, or floating-point.
/// ///
/// #[derive(Spanned)]
/// enum Number {
///     Decimal(Decimal),
///     Integer(Integer),
/// }
///
/// ///
/// /// (2.2) More complex enum.
/// ///
/// /// Either a base-10 integer, or hex integer.
/// ///
/// #[derive(Spanned)]
/// enum NumberOrHex {
///     Base10(AtLeast<1, Digit>),
///     Base16(v!(0x), AtLeast<1, HexDigit>),
/// }
/// ```
///
/// These tokens derive their span from all of their child tokens.
/// They can be expressed either as:
///
/// 1. Structs, either:
///     1. Named, or
///     2. Tuple.
/// 2. Enums:
///     1. Union types, and even
///     2. More complicated structures.
///
#[proc_macro_derive(Spanned)]
pub fn spanned(target: Tokens) -> Tokens {
    if let Ok(st) = syn::parse::<syn::ItemStruct>(target.clone()) {
        return derive_spanned_for_struct(&st);
    }

    if let Ok(en) = syn::parse::<syn::ItemEnum>(target.clone()) {
        return derive_spanned_for_enum(&en);
    }

    Diagnostic::spanned(
        Span::call_site(),
        Level::Error,
        "Expected a struct or enum here.",
    )
    .emit();

    Default::default()
}

///
/// ## verbatim!
///
/// Often shortend to `v!`, use *this* macro instead
/// of its struct helper friends `Verbatim<...>`, `CharPattern<...>`.
///
/// ### Examples
/// ```ignore
/// use avjason_macros::verbatim as v;
///
/// // (1) Single char match -> Verbatim<{char as &str}>
/// type Comma = v!(',');
///
/// // (2) String match -> Verbatim<{&str}>
/// type NaN = v!("NaN");
///
/// // (3) Char range match -> CharPattern<{CharacterRange {
/// //      start: start,
/// //      end: end, // (modified to make the end exclusive)
/// //  }}>
/// type Digit = v!('0'..='9');
/// type NonZero = v!('1'..='9');
/// ```
///
#[proc_macro]
pub fn verbatim(params: Tokens) -> Tokens {
    let params: VerbatimPat = syn::parse_macro_input!(params);
    let ty = params.into_type();
    ty.into_token_stream().into()
}
