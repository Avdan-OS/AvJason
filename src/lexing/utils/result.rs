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

impl<L> LexResult<L> {
    ///
    /// Allegory of [Result::expect]
    ///
    pub fn expect(self, msg: impl ToString) -> L {
        match self {
            LexResult::Lexed(lexed) => lexed,
            LexResult::Errant(errant) => panic!("{}: {errant:?}", msg.to_string()),
            LexResult::Nothing => panic!("{}: on LexResult::Nothing", msg.to_string()),
        }
    }

    ///
    /// Allegory of [Result::unwrap]
    ///
    pub fn unwrap(self) -> L {
        match self {
            LexResult::Lexed(lexed) => lexed,
            LexResult::Errant(errant) => panic!("called `LexResult::unwrap()` on an `Errant` value: {errant:?}"),
            LexResult::Nothing => panic!("called `LexResult::unwrap()` on a `Nothing` value"),
        }
    }

    pub fn is_errant(&self) -> bool {
        matches!(self, Self::Errant(_))
    }

    pub fn is_lexed(&self) -> bool {
        matches!(self, Self::Lexed(_))
    }

    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }
}
