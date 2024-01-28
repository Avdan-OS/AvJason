//!
//! Utility implementations for [Lex].
//!

use std::{any::type_name, ops::{Deref, DerefMut}};

use crate::common::{Source, Span, SpanIter, Spanned};

use super::{LexError, LexT, SourceStream};

///
/// Many (possibly one or zero) of a lexical token.
///
pub type Many<L> = Vec<L>;

impl<L: LexT> LexT for Many<L> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        L::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let mut v = vec![];

        while L::peek(input) {
            v.push(L::lex(input)?);
        }

        Ok(v)
    }
}

impl<S: Spanned> Spanned for Many<S> {
    fn span(&self) -> Span {
        SpanIter::combine(self.iter().map(S::span))
    }
}

///
/// At least N lots of `L`-tokens.
///
#[derive(Debug)]
pub struct AtLeast<const N: usize, L>(Vec<L>);

impl<const N: usize, L: LexT> LexT for AtLeast<N, L> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        L::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let many: Many<L> = LexT::lex(input)?;

        if many.len() < N {
            return Err(input.error(format!(
                "Expected at least {N} {} tokens: got {}.",
                std::any::type_name::<L>(),
                many.len(),
            )));
        }

        Ok(Self(many))
    }
}

impl<const N: usize, S: Spanned> Spanned for AtLeast<N, S> {
    fn span(&self) -> Span {
        SpanIter::combine(self.iter().map(S::span))
    }
}

impl<const N: usize, L> Deref for AtLeast<N, L> {
    type Target = Vec<L>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, L> DerefMut for AtLeast<N, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

///
/// Exactly N lots of `L`-tokens: no more, no less.
///
#[derive(Debug)]
pub struct Exactly<const N: usize, L>([L; N])
where
    [(); N]: Sized;

impl<const N: usize, L: LexT> LexT for Exactly<N, L>
where
    [(); N]: Sized,
{
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        L::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let many: Many<L> = LexT::lex(input)?;

        if many.len() != N {
            return Err(input.error(format!(
                "Expected {N} {} tokens: got {}.",
                std::any::type_name::<L>(),
                many.len()
            )));
        }

        // SAFETY: Just checked the length, so unwrap okay.
        let many: [L; N] = unsafe { many.try_into().unwrap_unchecked() };

        Ok(Self(many))
    }
}

impl<const N: usize, S: Spanned> Spanned for Exactly<N, S> {
    fn span(&self) -> Span {
        SpanIter::combine(self.iter().map(S::span))
    }
}

impl<const N: usize, L> Deref for Exactly<N, L> {
    type Target = [L; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, L> DerefMut for Exactly<N, L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
