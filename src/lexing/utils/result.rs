use std::{
    any::type_name,
    convert::Infallible,
    fmt::Debug,
    ops::{ControlFlow, FromResidual, Try},
};

use avjason_macros::Spanned;

use crate::common::{Source, Span, Spanned};

use super::SourceStream;

#[derive(Debug, Spanned)]
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
#[derive(Debug)]
pub enum LexResult<L> {
    ///
    /// Valid `L` token.
    ///
    Lexed(L),

    ///
    /// An attempt was made to parse an `L` token,
    /// but the input did not fully abide by `L`'s lexical grammar.
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
            LexResult::Errant(errant) => {
                panic!("called `LexResult::unwrap()` on an `Errant` value: {errant:?}")
            }
            LexResult::Nothing => panic!("called `LexResult::unwrap()` on a `Nothing` value"),
        }
    }

    ///
    /// Allegory of [Result::unwrap_err]
    ///
    pub fn unwrap_err(self) -> LexError
    where
        L: Debug,
    {
        match self {
            LexResult::Lexed(lexed) => {
                panic!("called `LexResult::unwrap()` on an `Lexed` value: {lexed:?}")
            }
            LexResult::Errant(errant) => errant,
            LexResult::Nothing => panic!("called `LexResult::unwrap_err()` on a `Nothing` value"),
        }
    }

    ///
    /// Turn this into a normal Rust [Result],
    /// panicking if this is a [LexResult::Nothing].
    ///
    pub fn unwrap_as_result(self) -> Result<L, LexError> {
        match self {
            LexResult::Lexed(lexed) => Ok(lexed),
            LexResult::Errant(errant) => Err(errant),
            LexResult::Nothing => panic!("Called `LexResult::into_result()` on a Nothing value."),
        }
    }

    ///
    /// Is this [LexResult::Errant]?
    ///
    pub fn is_errant(&self) -> bool {
        matches!(self, Self::Errant(_))
    }

    ///
    /// Is this [LexResult::Lexed]?
    ///
    pub fn is_lexed(&self) -> bool {
        matches!(self, Self::Lexed(_))
    }

    ///
    /// Is this [LexResult::Nothing]?
    ///
    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }

    ///
    /// Allegory of [Result::map].
    ///
    /// If this is [LexResult::Lexed], the mapper function will be called,
    /// and then its return type will be re-wrapped.
    ///
    pub fn map<T, F: FnOnce(L) -> T>(self, mapper: F) -> LexResult<T> {
        match self {
            LexResult::Lexed(lexed) => LexResult::Lexed(mapper(lexed)),
            LexResult::Errant(errant) => LexResult::Errant(errant),
            LexResult::Nothing => LexResult::Nothing,
        }
    }

    ///
    /// If this is [LexResult::Nothing], execute the `or` function instead,
    /// and return its result.
    ///
    /// This allows for chaining of results, which may be useful
    /// in lexing enums with different variants.
    ///
    pub fn or<F: FnOnce() -> Self>(self, or: F) -> Self {
        match self {
            s @ LexResult::Lexed(_) => s,
            s @ LexResult::Errant(_) => s,
            LexResult::Nothing => or(),
        }
    }

    ///
    /// Allegory of [Result::and_then].
    ///
    /// If this is [LexResult::Lexed], the mapper function will be called,
    /// and its return value is returned.
    ///
    pub fn and<T, F: FnOnce(L) -> LexResult<T>>(self, mapper: F) -> LexResult<T> {
        match self {
            LexResult::Lexed(lexed) => mapper(lexed),
            LexResult::Errant(errant) => LexResult::Errant(errant),
            LexResult::Nothing => LexResult::Nothing,
        }
    }

    ///
    /// Require this potential token to be present, not [LexResult::Nothing] or [LexResult::Errant].
    ///
    /// If this is [LexResult::Nothing], make this into a [LexResult::Errant]
    /// with the message "expected a {$`L`} token".
    ///
    pub fn expected<S: Source>(self, input: &SourceStream<S>) -> Self {
        match self {
            s @ LexResult::Lexed(_) => s,
            s @ LexResult::Errant(_) => s,
            LexResult::Nothing => LexResult::Errant(LexError {
                span: input.span(),
                message: format!("Expected a {} token here.", type_name::<L>()),
            }),
        }
    }

    ///
    /// Require this potential token to be present, not [LexResult::Nothing] or [LexResult::Errant].
    ///
    /// If this is [LexResult::Nothing], display the custom message.
    ///
    pub fn expected_msg<S: Source>(self, input: &SourceStream<S>, msg: impl ToString) -> Self {
        match self {
            s @ LexResult::Lexed(_) => s,
            s @ LexResult::Errant(_) => s,
            LexResult::Nothing => LexResult::Errant(LexError {
                span: input.span(),
                message: msg.to_string(),
            }),
        }
    }
}

impl<L> Try for LexResult<L> {
    type Output = L;

    type Residual = LexResult<Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Lexed(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            LexResult::Lexed(lexed) => ControlFlow::Continue(lexed),
            LexResult::Errant(errant) => ControlFlow::Break(LexResult::Errant(errant)),
            LexResult::Nothing => ControlFlow::Break(LexResult::Nothing),
        }
    }
}

impl<L> FromResidual for LexResult<L> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            LexResult::Lexed(_) => unreachable!(),
            LexResult::Errant(errant) => LexResult::Errant(errant),
            LexResult::Nothing => LexResult::Nothing,
        }
    }
}
