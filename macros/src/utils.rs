//!
//! Utilities for the utility macros.
//!

use std::ops::Deref;

use proc_macro::{Diagnostic, Level, Span};
use quote::quote;
use syn::punctuated::Punctuated;

///
/// A lit str, but a warning is displayed
/// if it is empty.
///
pub struct NonEmptyStr(syn::LitStr);

impl syn::parse::Parse for NonEmptyStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: syn::LitStr = input.parse()?;

        if lit.value().is_empty() {
            Diagnostic::spanned(
                lit.span().unwrap(),
                Level::Warning,
                "This should not be empty.",
            )
            .emit();
        }

        Ok(Self(lit))
    }
}

impl Deref for NonEmptyStr {
    type Target = syn::LitStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn path_contains(path: &syn::Path, st: &str) -> bool {
    path.segments.iter().any(|seg| seg.ident == st)
}

pub fn get_struct_member_where_type(
    st: &syn::ItemStruct,
    pred: impl Fn(&syn::Type) -> bool,
) -> Option<syn::Expr> {
    let ident = match &st.fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => named
            .iter()
            .find_map(|f| pred(&f.ty).then(|| f.ident.as_ref().unwrap().clone())),
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            unnamed.iter().enumerate().find_map(|(i, f)| {
                pred(&f.ty).then(|| syn::Ident::new(&i.to_string(), Span::call_site().into()))
            })
        }
        syn::Fields::Unit => None,
    }?;

    // Unwrap as we should have valid syntax here.
    Some(syn::parse2(quote! {
        self.#ident
    }).unwrap())
}
