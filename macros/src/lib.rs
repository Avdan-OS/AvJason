//!
//! Macros for the main crate.
//!

#![feature(proc_macro_diagnostic)]

mod spanned;
mod type_traversal;
mod utils;

use proc_macro::{Diagnostic, Level, Span, TokenStream as Tokens};
use quote::ToTokens;
use spanned::{derive_spanned_for_enum, derive_spanned_for_struct};
use syn::parse_macro_input;
use utils::{get_item_attrs, ECMARef, JSON5Ref, ToRustdoc};

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
///
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
