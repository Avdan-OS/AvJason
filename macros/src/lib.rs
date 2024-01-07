//!
//! Utility macros for the main crate.
//!

#![feature(proc_macro_diagnostic, iter_intersperse)]
mod utils;

use proc_macro::{Diagnostic, Level, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, ItemStruct, Token};
use utils::{get_struct_member_where_type, path_contains, NonEmptyStr};

///
/// Parses the item, and arguments for the Ref* macros.
///
/// Also provides useful errors.
///
fn reference_macro<Args>(arg: TokenStream, target: TokenStream) -> Option<(syn::Item, Args)>
where
    Args: syn::parse::Parse,
{
    // Only apply to items (Rust syntax).
    let target: syn::Item = match syn::parse(target) {
        Ok(item) => item,
        Err(err) => {
            Diagnostic::spanned(
                err.span().unwrap(),
                Level::Error,
                "Cannot apply this to a non-item (e.g. struct, enum, type).",
            )
            .emit();

            return None;
        }
    };

    let arg: Args = match syn::parse(arg) {
        Ok(arg) => arg,
        Err(err) => {
            Diagnostic::spanned(
                err.span().unwrap(),
                Level::Error,
                "Expected literal &str here.",
            )
            .emit();

            return None;
        }
    };

    Some((target, arg))
}

///
/// Reference to a part of the JSON5 spec.
///
/// Adds a link to the part of the original spec.
///
/// ### Example
/// ```ignore
/// use avjason_macros::SpecRef;
///
/// ///
/// /// Whitespace characters that do not influence syntax.
/// ///
/// #[SpecRef("WhiteSpace")]
/// pub struct WhiteSpace {
///     /* Blah, blah, blah. */
/// }
/// ```
///
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn SpecRef(arg: TokenStream, target: TokenStream) -> TokenStream {
    let Some((target, arg)): Option<(_, NonEmptyStr)> = reference_macro(arg, target) else {
        return TokenStream::default();
    };

    let link = format!(
        "See the original spec: [**{0}**](https://spec.json5.org/#prod-{0}).",
        arg.value()
    );

    quote! {
        #[doc = ""]
        #[doc = "---"]
        #[doc = #link]
        #[doc = ""]
        #target
    }
    .into()
}

///
/// Format for the ECMAScript spec reference, since their urls
/// make no sense.
///
struct EcmaRef {
    ///
    /// Display text for the link.
    ///
    text: NonEmptyStr,

    ///
    /// Comma seperator.
    ///
    _comma: Token![,],

    ///
    /// Href for the link.
    ///
    href: NonEmptyStr,
}

impl syn::parse::Parse for EcmaRef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            text: input.parse()?,
            _comma: input.parse()?,
            href: input.parse()?,
        })
    }
}

/// Adds a link to the part of the original ECMAScript spec.
///
/// ### Example
/// ```ignore
/// use avjason_macros::ECMARef;
///
/// #[ECMARef("LineTermintor", "https://262.ecma-international.org/5.1/#sec-7.3")]
/// pub struct LineTerminator {
///     /* Blah, blah, blah */
/// }
/// ```
///
///
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn ECMARef(arg: TokenStream, target: TokenStream) -> TokenStream {
    let Some((target, arg)): Option<(_, EcmaRef)> = reference_macro(arg, target) else {
        return TokenStream::default();
    };

    let link = format!(
        "See the original ECMAScript spec: [**{}**]({}).",
        arg.text.value(),
        arg.href.value(),
    );

    quote! {
        #[doc = ""]
        #[doc = "---"]
        #[doc = #link]
        #[doc = ""]
        #target
    }
    .into()
}

#[proc_macro_derive(Spanned)]
pub fn spanned(item: TokenStream) -> TokenStream {
    let (st, en): (syn::Result<ItemStruct>, syn::Result<syn::ItemEnum>) =
        (syn::parse(item.clone()), syn::parse(item));

    if let Ok(ref st) = st {
        // Find first field with the `Span` type.
        let Some(m) = get_struct_member_where_type(st, |ty| match ty {
            syn::Type::Path(syn::TypePath { path, .. }) => path_contains(path, "Span"),
            _ => false,
        }) else {
            Diagnostic::spanned(
                st.span().unwrap(),
                Level::Error,
                "Need a field with type `Span` in it.",
            )
            .emit();

            return Default::default();
        };

        let ident = &st.ident;
        return quote! {
            impl crate::utils::Spanned for #ident {
                fn span(&self) -> Span {
                    #m
                }
            }
        }
        .into();
    }

    if let Ok(en) = en {
        let vars: Vec<_> = en
            .variants
            .iter()
            .map(|var| match &var.fields {
                syn::Fields::Named(_) => None,
                syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
                    (unnamed.len() == 1).then_some(&var.ident)
                }
                syn::Fields::Unit => None,
            })
            .collect();

        if vars.iter().any(Option::is_none) {
            Diagnostic::spanned(
                en.span().unwrap(),
                Level::Error,
                "Enum variants can only be a single-element tuple.",
            )
            .emit();

            return Default::default();
        }
        let ident = &en.ident;
        // SAFETY: We've already checked above if any are none.
        let vars = vars
            .into_iter()
            .map(|a| unsafe { a.unwrap_unchecked() })
            .map(|ident| {
                quote! {
                    Self::#ident(inner) => inner.span()
                }
            });

        return quote! {
            impl crate::utils::Spanned for #ident {
                fn span(&self) -> crate::utils::Span {
                    match self {
                        #(#vars),*,
                        _ => unreachable!()
                    }
                }
            }
        }
        .into();
    }

    // SAFETY: We check before if st is Ok and early-return,
    // so this is safe.
    // This is done since syn::ItemStruct doesn't impl `Debug` :(
    let err = unsafe { st.unwrap_err_unchecked() };

    Diagnostic::spanned(
        err.span().unwrap(),
        Level::Error,
        "Expected either struct, or enum definition here.",
    )
    .emit();

    Default::default()
}
