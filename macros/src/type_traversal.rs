//!
//! Utilities that allow use to traverse `struct`s and `enum`s.
//!

use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;

///
/// Checks to see if an identifier is in a path.
///
pub fn in_path<'a>(path: &'a syn::Path, ident: &str) -> Option<&'a syn::PathSegment> {
    path.segments
        .iter()
        .find(|syn::PathSegment { ident: id, .. }| id == ident)
}

///
/// Checks if a type has the ident inside its name.
///
pub fn is_named_type<'a>(ty: &'a syn::Type, ident: &str) -> Option<&'a syn::PathSegment> {
    match ty {
        syn::Type::Path(syn::TypePath { path, .. }) => in_path(path, ident),
        _ => None,
    }
}

pub fn self_keyword() -> syn::Expr {
    syn::Expr::Path(syn::ExprPath {
        attrs: Default::default(),
        qself: Default::default(),
        path: syn::Ident::new("self", Span::call_site()).into(),
    })
}

pub trait ToMember {
    fn to_member(self) -> syn::Member;
}

impl ToMember for syn::Index {
    fn to_member(self) -> syn::Member {
        syn::Member::Unnamed(self)
    }
}

impl ToMember for syn::Ident {
    fn to_member(self) -> syn::Member {
        syn::Member::Named(self)
    }
}

pub fn index(index: u32) -> syn::Index {
    syn::Index {
        index,
        span: Span::call_site(),
    }
}

pub fn field_access(m: impl ToMember) -> syn::Expr {
    syn::Expr::Field(syn::ExprField {
        attrs: Default::default(),
        base: Box::new(self_keyword()),
        dot_token: Default::default(),
        member: m.to_member(),
    })
}

pub trait Generic {
    fn ident(&self) -> &syn::Ident;

    fn generics(&self) -> &syn::Generics;

    fn generic_letters(&self) -> proc_macro2::TokenStream {
        let generics = self.generics();
        let letters = generics.params.iter().map(|param| match param {
            syn::GenericParam::Lifetime(l) => l.lifetime.to_token_stream(),
            syn::GenericParam::Type(ty) => ty.ident.to_token_stream(),
            syn::GenericParam::Const(cons) => cons.ident.to_token_stream(),
        });

        quote! {
            <#(#letters),*>
        }
    }
}

impl Generic for syn::ItemStruct {
    fn generics(&self) -> &syn::Generics {
        &self.generics
    }

    fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}

impl Generic for syn::ItemEnum {
    fn generics(&self) -> &syn::Generics {
        &self.generics
    }

    fn ident(&self) -> &syn::Ident {
        &self.ident
    }
}

pub fn variant_path(var: &syn::Ident) -> syn::Path {
    syn::Path {
        leading_colon: Default::default(),
        segments: Punctuated::from_iter(
            [syn::Ident::new("Self", Span::call_site()), var.clone()]
                .into_iter()
                .map(|ident| syn::PathSegment {
                    ident,
                    arguments: syn::PathArguments::None,
                }),
        ),
    }
}
