//!
//! Peeking for lexical tokens.
//!

use std::marker::PhantomData;

use crate::common::Source;

use super::{LexResult, LexT, SourceStream};

///
/// Result of a peek, either:
/// * Possibly present,
/// * or not.
///
pub enum Peek<T> {
    Possible(PhantomData<T>),
    Absent,
}

impl<L: LexT> Peek<L> {
    pub fn then_lex<S: Source>(self, input: &mut SourceStream<S>) -> LexResult<L> {
        match self {
            Peek::Possible(_) => match LexT::lex(input) {
                Ok(lexed) => LexResult::Lexed(lexed),
                Err(errant) => LexResult::Errant(errant),
            },
            Peek::Absent => LexResult::Nothing,
        }
    }
}
