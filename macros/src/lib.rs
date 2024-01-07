#![feature(proc_macro_diagnostic, iter_intersperse)]
use proc_macro::{Diagnostic, Level, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

