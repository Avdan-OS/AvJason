//!
//! Lexxing utilities.
//!

use std::ops::RangeBounds;

use crate::utils::{Span, TryIntoSpan};

pub mod escape;
pub mod strings;
pub mod tokens;
pub mod number;

#[derive(Debug)]
pub struct LexError {
    span: Span,
    message: String,
    text: Option<String>,
}

impl LexError {
    pub(crate) fn new<S: TryIntoSpan, B: RangeBounds<S>>(
        span: B,
        message: impl ToString,
        text: impl Into<Option<String>>,
    ) -> Self {
        let span = TryIntoSpan::try_into_span(span).unwrap();
        let message = message.to_string();
        let text = text.into();

        Self {
            span,
            message,
            text,
        }
    }
}

///
/// Utility for Lexer erorrs,
///
pub type LexResult<T> = Result<Option<T>, LexError>;

pub trait IntoLexResult<T>: Sized {
    fn into_lex_result(self) -> LexResult<T>;
}

default impl<T> IntoLexResult<T> for T {
    fn into_lex_result(self) -> LexResult<T> {
        Ok(Some(self))
    }
}

impl<T> IntoLexResult<T> for Option<T> {
    fn into_lex_result(self) -> LexResult<T> {
        Ok(self)
    }
}

impl<T> IntoLexResult<T> for LexResult<T> {
    fn into_lex_result(self) -> LexResult<T> {
        self
    }
}

impl<T> IntoLexResult<T> for Result<T, LexError> {
    fn into_lex_result(self) -> LexResult<T> {
        self.map(Option::Some)
    }
}
