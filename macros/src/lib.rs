#![feature(proc_macro_diagnostic, iter_intersperse)]
use proc_macro::{Diagnostic, Level, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(Spanned)]
pub fn derive(input: TokenStream) -> TokenStream {
    if let Ok(en) = syn::parse::<syn::ItemEnum>(input.clone()) {
        let ident = en.ident.clone();
        let passed = en
            .variants
            .iter()
            .map(|var| {
                let ident = var.ident.clone();
                let syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed: _, .. }) = var.fields else {
                    return Err(var.span());
                };

                Ok(ident)
            })
            .collect::<Vec<_>>();

        if passed.iter().any(Result::is_err) {
            let errors = passed.into_iter().filter_map(Result::err);

            errors.for_each(|s| {
                Diagnostic::spanned(s.unwrap(), Level::Error, "Need tuple-like struct here.").emit()
            });

            return syn::Error::new(
                Span::call_site().into(),
                "Expected enum with tuple variants.",
            )
            .into_compile_error()
            .into();
        }

        let vars = passed.into_iter().filter_map(Result::ok).map(|var| {
            quote! {
                #ident::#var(ref s) => crate::utils::Spanned::span(s)
            }
        });

        return quote! {
            impl crate::utils::Spanned for #ident {
                fn span(&self) -> crate::utils::Span {
                    match self {
                        #(#vars),*
                    }
                }
            }
        }
        .into();
    };

    if let Ok(st) = syn::parse::<syn::ItemStruct>(input) {
        let ident = st.ident.clone();
        match st.fields {
            syn::Fields::Named(syn::FieldsNamed { named: f, .. }) => {
                let pass = f.iter().any(|syn::Field { ident, .. }| {
                    ident.as_ref().map(|ident| ident == "span").unwrap_or(false)
                });

                if !pass {
                    return syn::Error::new(
                        f.span(),
                        "Cannot derive Spanned for named struct without `span` field.",
                    )
                    .into_compile_error()
                    .into();
                }

                return quote! {
                    impl crate::utils::Spanned for #ident {
                        fn span(&self) -> Span {
                            self.span
                        }
                    }
                }
                .into();
            }
            syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed: f, .. }) => {
                if f.is_empty() {
                    return syn::Error::new(
                        f.span(),
                        "Cannot derive Spanned for empty tuple struct.",
                    )
                    .into_compile_error()
                    .into();
                }

                return quote! {
                    impl crate::utils::Spanned for #ident {
                        fn span(&self) -> Span {
                            self.0
                        }
                    }
                }
                .into();
            }
            syn::Fields::Unit => {
                return syn::Error::new(st.span(), "Cannot derive Spanned for unit struct.")
                    .into_compile_error()
                    .into();
            }
        }
    }

    syn::Error::new(Span::call_site().into(), "Expected either enum or struct.")
        .into_compile_error()
        .into()
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Lex(args: TokenStream, input: TokenStream) -> TokenStream {
    let st = syn::parse::<syn::ItemStruct>(input.clone());
    let en = syn::parse::<syn::ItemEnum>(input);

    match (st, en) {
        (Ok(st), Err(_)) => {
            let ident = &st.ident;
            let ch: syn::LitChar = match syn::parse(args) {
                Ok(ch) => ch,
                Err(err) => {
                    return err.into_compile_error().into();
                }
            };
            quote! {
                #st

                impl Lex for #ident {
                    fn lex(input: &mut crate::utils::SourceIter) -> Option<Self> {
                        if input.peek() == Some(&#ch) {
                            // Unwrap okay, because otherwise .peek returns None.
                            let (l, _) = input.next().unwrap();
                            return Some(Self(crate::utils::Span::single_char(l)));
                        }

                        None
                    }

                    fn peek(input: &crate::utils::SourceIter) -> bool {
                        input.peek() == Some(&#ch)
                    }
                }
            }
            .into()
        }
        (Err(_), Ok(en)) => {
            let ident = &en.ident;

            let vars = en
                .variants
                .iter()
                .map(|syn::Variant { ident, fields, .. }| match fields {
                    syn::Fields::Named(_) => None,
                    syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed: f, .. }) => {
                        if f.is_empty() {
                            return None;
                        }
                        let f = f.iter().next().unwrap();
                        Some((ident.clone(), f.ty.clone()))
                    }
                    syn::Fields::Unit => None,
                })
                .collect::<Vec<_>>();

            if vars.iter().any(Option::is_none) {
                return syn::Error::new_spanned(
                    en,
                    "Cannot auto-impl Lex on enum that is not only single-tuple variants.",
                )
                .into_compile_error()
                .into();
            }

            let (vars, peeks): (Vec<_>, Vec<_>) = vars
                .into_iter()
                .flatten()
                .map(|(v, ty)| {
                    (
                        quote! {
                            if let Some(s) = #ty::lex(input).into_lex_result()? {
                                return Ok(Some(Self::#v(s)));
                            }
                        },
                        quote! {
                            #ty::peek(input)
                        },
                    )
                })
                .unzip();

            let peeks = peeks.into_iter().intersperse(quote! {||});

            quote! {
                #en

                impl Lex for #ident {
                    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
                        #(#vars)*

                        Ok(None)
                    }

                    fn peek(input: &SourceIter) -> bool {
                        #(#peeks)*
                    }
                }

            }
            .into()
        }
        _ => unimplemented!("Mutually exlusive parsing."),
    }
}
