//!
//! Utilities for lexing.
//!

pub mod lex_impls;
pub mod peek;
pub mod result;
pub mod stream;
pub mod verbatim;

use std::marker::PhantomData;

use crate::common::Source;

pub use self::{
    lex_impls::{AtLeast, Exactly, Many},
    peek::Peek,
    result::{LexError, LexResult},
    stream::SourceStream,
};

///
/// Private trait, only for internal use.
///
#[doc(hidden)]
pub trait LexT: Sized {
    ///
    /// Checks to see if this token is possibly upcoming.
    ///
    fn peek<S: Source>(input: &SourceStream<S>) -> bool;

    ///
    /// Given that the token is potentially present,
    /// start lexing.
    ///
    /// This function has guaranteed side-effects on the input [SourceStream] (advancing it).
    ///
    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError>;
}

///
/// Oprations on lexical tokens:
/// * Lexing,
/// * Peeking
///
pub trait Lex: Sized {
    ///
    /// Checks is this token is potentially present,
    /// which can then be further further lexed.
    ///
    fn peek<S: Source>(input: &SourceStream<S>) -> Peek<Self>;

    ///
    /// Returns a [LexResult] with either:
    /// * a valid token [LexResult::Lexed],
    /// * [LexResult::Nothing] (token not present),
    /// * or [LexResult::Errant] (spanned error).
    ///
    fn lex<S: Source>(input: &mut SourceStream<S>) -> LexResult<Self>;
}

///
/// The public-facing implementation.
///
impl<L: LexT> Lex for L {
    #[inline]
    fn peek<S: Source>(input: &SourceStream<S>) -> Peek<Self> {
        // Forward to internal impl, then make proper [Peek]
        // enum variant.
        match <Self as LexT>::peek(input) {
            true => Peek::Possible(PhantomData::<Self>),
            false => Peek::Absent,
        }
    }

    ///
    /// Returns a [LexResult] with either:
    /// * a valid token [LexResult::Lexed],
    /// * [LexResult::Nothing] (token not present),
    /// * or [LexResult::Errant] (spanned error).
    ///
    fn lex<S: Source>(input: &mut SourceStream<S>) -> LexResult<Self> {
        <Self as Lex>::peek(input).then_lex(input)
    }
}
