//!
//! Utilities for the `verbatim!` macro.
//!

use std::ops::Deref;

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};

use self::paths::generic_path;

///
/// Accepted patterns for `verbatim!`.
///
pub enum VerbatimPat {
    LitStr(syn::LitStr),
    LitChar(syn::LitChar),
    CharRange(char, char),
}

mod paths {
    use proc_macro2::Span;
    use syn::punctuated::Punctuated;

    use crate::type_traversal::ToMember;

    ///
    /// Makes an ident from a string,
    /// with the Span resolving to Span::call_site()
    ///
    fn ident(st: &str) -> syn::Ident {
        syn::Ident::new(st, Span::call_site())
    }

    ///
    /// Generates a path with the last segment
    /// having generic parameters.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// $path<$arg>
    /// ```
    ///
    pub fn generic_path<const N: usize>(path: [&str; N], arg: syn::GenericArgument) -> syn::Type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: Punctuated::from_iter(
                    path[..N - 1]
                        .iter()
                        .copied()
                        .map(ident)
                        .map(syn::PathSegment::from)
                        .chain([syn::PathSegment {
                            ident: ident(path[N - 1]),
                            arguments: syn::PathArguments::AngleBracketed(
                                syn::AngleBracketedGenericArguments {
                                    colon2_token: Default::default(),
                                    lt_token: Default::default(),
                                    args: Punctuated::from_iter([arg]),
                                    gt_token: Default::default(),
                                },
                            ),
                        }]),
                ),
            },
        })
    }

    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// crate::lexing::CharacterRange {
    ///     start: $start,
    ///     end: $end,
    /// }
    /// ```
    ///
    pub fn character_range(start: syn::Expr, end: syn::Expr) -> syn::Expr {
        let path = syn::Path {
            leading_colon: None,
            segments: Punctuated::from_iter(
                ["crate", "lexing", "CharacterRange"]
                    .map(ident)
                    .map(syn::PathSegment::from),
            ),
        };

        syn::Expr::Struct(syn::ExprStruct {
            attrs: Default::default(),
            qself: Default::default(),
            path,
            brace_token: Default::default(),
            fields: Punctuated::from_iter([("start", start), ("end", end)].map(|(f, expr)| {
                syn::FieldValue {
                    attrs: Default::default(),
                    member: ident(f).to_member(),
                    colon_token: Some(Default::default()),
                    expr,
                }
            })),
            dot2_token: None,
            rest: None,
        })
    }
}

impl VerbatimPat {
    ///
    /// Build the AST for this pattern,
    /// using helper structs in the main crate.
    ///
    pub fn into_type(self) -> syn::Type {
        match self {
            VerbatimPat::LitStr(st) => paths::generic_path(
                ["crate", "lexing", "Verbatim"],
                syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                    attrs: Default::default(),
                    lit: syn::Lit::Str(st),
                })),
            ),
            VerbatimPat::LitChar(ch) => paths::generic_path(
                ["crate", "lexing", "Verbatim"],
                syn::GenericArgument::Const(syn::Expr::Lit(syn::ExprLit {
                    attrs: Default::default(),
                    lit: syn::Lit::Str(syn::LitStr::new(
                        &ch.value().to_string(),
                        Span::call_site(),
                    )),
                })),
            ),
            VerbatimPat::CharRange(start, end) => {
                let bounds = [start, end].map(|c| syn::LitChar::new(c, Span::call_site()));
                let [start, end] = bounds.map(|ch| {
                    syn::Expr::Lit(syn::ExprLit {
                        attrs: Default::default(),
                        lit: syn::Lit::Char(ch),
                    })
                });

                let constructed = paths::character_range(start, end);
                let braced = syn::Expr::Block(syn::ExprBlock {
                    attrs: Default::default(),
                    label: Default::default(),
                    block: syn::Block {
                        brace_token: Default::default(),
                        stmts: vec![syn::Stmt::Expr(constructed, None)],
                    },
                });

                let const_param = syn::GenericArgument::Const(braced);

                generic_path(["crate", "lexing", "CharPattern"], const_param)
            }
        }
    }
}

///
/// Is this expression a char literal?
///
fn is_lit_char(expr: &impl Deref<Target = syn::Expr>) -> bool {
    let expr = expr.deref();
    matches!(
        expr,
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(_),
            ..
        })
    )
}

///
/// Gets the character value if this expression is a char literal.
///
fn get_char(expr: &impl Deref<Target = syn::Expr>) -> Option<char> {
    match expr.deref() {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Char(litchar),
            ..
        }) => Some(litchar.value()),
        _ => None,
    }
}

impl Parse for VerbatimPat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = syn::Pat::parse_single(input)?;

        // Nasty pattern matching, but that's the downside of nested enums.
        match pat {
            syn::Pat::Lit(syn::ExprLit {
                lit: lit @ (syn::Lit::Char(_) | syn::Lit::Str(_)),
                ..
            }) => match lit {
                syn::Lit::Char(ch) => Ok(Self::LitChar(ch)),
                syn::Lit::Str(st) => Ok(Self::LitStr(st)),
                _ => unreachable!(),
            },
            syn::Pat::Range(syn::PatRange {
                start, end, limits, ..
            }) if start.as_ref().map(is_lit_char).unwrap_or(true)
                && end.as_ref().map(is_lit_char).unwrap_or(true) =>
            {
                let c_start = start.as_ref().and_then(get_char).unwrap_or(char::MIN);
                let c_end = end.as_ref().and_then(get_char).unwrap_or(char::MAX);

                let (c_start, c_end) = match limits {
                    syn::RangeLimits::HalfOpen(_) => (c_start, Some(c_end)),
                    syn::RangeLimits::Closed(_) => (c_start, char::from_u32(c_end as u32 + 1)),
                };

                if c_end.is_none() {
                    return Err(syn::Error::new_spanned(
                        end,
                        "This char literal cannot be used as an inclusive end.",
                    ));
                }

                let (start, end) = (c_start, c_end.unwrap());
                Ok(Self::CharRange(start, end))
            }
            _ => Err(syn::Error::new_spanned(
                pat,
                "Only string and char literals, and char ranges are accepted here",
            )),
        }
    }
}
