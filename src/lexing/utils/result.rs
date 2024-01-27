use crate::common::{Source, Span, Spanned};

use super::SourceStream;

#[derive(Debug)]
pub struct LexError {
    span: Span,
    message: String,
}

impl LexError {
    pub fn new(span: &impl Spanned, message: impl ToString) -> Self {
        Self {
            span: span.span(),
            message: message.to_string(),
        }
    }
}

impl<'a, S: Source> SourceStream<'a, S> {
    ///
    /// Make a new error at the stream's current location.
    ///
    pub fn error(&self, msg: impl ToString) -> LexError {
        LexError::new(self, msg)
    }
}

///
/// The rust of attempting parse token `L`
/// from a [SourceStream].
///
pub enum LexResult<L> {
    ///
    /// Valid token.
    ///
    Lexed(L),

    ///
    /// An attempt was made to parse a token,
    /// but it did not fully abide by the lexical grammar.
    ///
    Errant(LexError),

    ///
    /// The token `L` was not found,
    /// so the parsing was skipped.
    ///
    Nothing,
}
