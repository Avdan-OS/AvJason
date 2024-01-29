//!
//! Utilities for the [crate::unicode] macro.
//!

use std::iter::once;

use proc_macro::{Diagnostic, Level};
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
};

///
/// The input into the [crate::unicode] macro.
///
pub struct UnicodePatInput {
    categories: Vec<UnicodeCategory>,
}

fn ident(st: &str) -> syn::Ident {
    syn::Ident::new(st, Span::call_site())
}

pub enum UnicodeCategory {
    Major(syn::Ident),
    Minor(syn::Ident),
}

impl UnicodePatInput {
    ///
    /// Converts this collection of unicode catgeories
    /// into its appropriate matcher type (determined by the first category).
    ///
    pub fn into_type(self) -> Option<syn::Type> {
        let mut iter = self.categories.into_iter();
        let Some(first) = iter.next() else {
            Diagnostic::new(Level::Error, "Expected unicode major/minor categories!").emit();
            return None;
        };
        Some(first.into_matcher(iter))
    }
}

impl UnicodeCategory {
    ///
    /// Attempt to parse a unicode major/minor category
    /// from an identifier (only checks length).
    ///
    fn parse(ident: syn::Ident) -> Result<Self, ()> {
        let st = ident.to_string();

        match st.len() {
            0 => unreachable!(),
            1 => Ok(Self::Major(ident)),
            2 => Ok(Self::Minor(ident)),
            _ => {
                Diagnostic::spanned(ident.span().unwrap(), Level::Error, "Expected either a one-letter unicode major catgeory, or a two-letter unicode minor category.")
                    .emit();

                Err(())
            }
        }
    }

    ///
    /// Gets this category as an expression.
    ///
    fn into_expr(self) -> syn::Expr {
        let (ty, cat) = match self {
            Self::Major(ident) => ("MajorCategory", ident),
            Self::Minor(ident) => ("MinorCategory", ident),
        };

        syn::Expr::Path(syn::ExprPath {
            attrs: Default::default(),
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: Punctuated::from_iter(
                    ["crate", "lexing", "utils", "unicode", ty, &cat.to_string()]
                        .iter()
                        .map(|s| syn::Ident::new(s, cat.span()))
                        .map(syn::PathSegment::from),
                ),
            },
        })
    }

    ///
    /// Returns the type of this category type's matcher, along with supplying
    /// itself and the other categories as const params.
    ///
    pub fn into_matcher(self, others: impl IntoIterator<Item = Self>) -> syn::Type {
        let ty = match self {
            Self::Major(_) => "MatchMajorCategory",
            Self::Minor(_) => "MatchMinorCategory",
        };

        let array = syn::Expr::Array(syn::ExprArray {
            attrs: Default::default(),
            bracket_token: Default::default(),
            elems: Punctuated::from_iter(once(self).chain(others).map(Self::into_expr)),
        });

        let static_ref = syn::Expr::Reference(syn::ExprReference {
            attrs: Default::default(),
            and_token: Default::default(),
            mutability: None,
            expr: Box::new(array),
        });

        let braced = syn::Expr::Block(syn::ExprBlock {
            attrs: Default::default(),
            label: None,
            block: syn::Block {
                brace_token: Default::default(),
                stmts: vec![syn::Stmt::Expr(static_ref, None)],
            },
        });

        let generic_arg = syn::GenericArgument::Const(braced);
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: Punctuated::from_iter(
                    ["crate", "lexing", "utils", "unicode"]
                        .into_iter()
                        .map(ident)
                        .map(syn::PathSegment::from)
                        .chain(once(syn::PathSegment {
                            ident: ident(ty),
                            arguments: syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Default::default(),
                                    args: Punctuated::from_iter(once(generic_arg)),
                                    gt_token: Default::default(),
                                },
                            ),
                        })),
                ),
            },
        })
    }
}

impl Parse for UnicodePatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = syn::Pat::parse_multi(input)?;
        let cases: Vec<_> = match pat {
            syn::Pat::Or(or) => or.cases.into_iter().collect(),
            ident @ syn::Pat::Ident(_) => vec![ident],
            pat => {
                return Err(syn::Error::new_spanned(
                    pat,
                    "Expected either one or many (with |) unicode major/minor categories here.",
                ))
            }
        };

        let idents: Vec<_> = cases
            .iter()
            .map(|pat| match pat {
                syn::Pat::Ident(syn::PatIdent { ident, .. }) => Some(ident),
                pat => {
                    Diagnostic::spanned(
                        pat.span().unwrap(),
                        Level::Error,
                        "Expected either a unicode major or minor category here.",
                    )
                    .emit();
                    None
                }
            })
            .collect();

        if idents.iter().any(Option::is_none) {
            return Err(syn::Error::new(
                Span::call_site(),
                "An error occurred whilst parsing syntax.",
            ));
        }

        // ::unwrap() okay since !any(is_none) -> all(is_some)
        let idents = idents.into_iter().map(Option::unwrap);

        let categories: Vec<_> = idents.cloned().map(UnicodeCategory::parse).collect();

        if categories.iter().any(Result::is_err) {
            return Err(syn::Error::new(
                Span::call_site(),
                "Invalid unicode major or minor category.",
            ));
        }

        // ::unwrap() okay since !any(is_err) -> all(is_ok)
        Ok(Self {
            categories: categories.into_iter().map(Result::unwrap).collect(),
        })
    }
}
