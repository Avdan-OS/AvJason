//!
//! Utility implementations for [Lex].
//!

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::common::{Source, Span, SpanIter, Spanned};

use super::{Lex, LexResult, LexT, Peek, SourceStream};

///
/// Many (possibly one or zero) of a lexical token.
///
pub type Many<L> = Vec<L>;

impl<L: LexT> Lex for Many<L> {
    fn peek<S: Source>(_: &SourceStream<S>) -> Peek<Self> {
        Peek::Possible(PhantomData::<Self>)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> LexResult<Self> {
        let mut v = vec![];

        loop {
            match <L as Lex>::lex(input) {
                LexResult::Lexed(lexed) => v.push(lexed),
                LexResult::Errant(errant) => return LexResult::Errant(errant),
                LexResult::Nothing => break,
            }
        }

        LexResult::Lexed(v)
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

impl<const N: usize, L: LexT> Lex for AtLeast<N, L> {
    fn peek<S: Source>(input: &SourceStream<S>) -> Peek<Self> {
        if N == 0 {
            return Peek::Possible(PhantomData::<Self>);
        }

        <L as Lex>::peek(input).map()
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> LexResult<Self> {
        let many: Many<L> = Lex::lex(input)?;

        if many.len() < N {
            return LexResult::Errant(input.error(format!(
                "Expected at least {N} {} tokens: got {}.",
                std::any::type_name::<L>(),
                many.len(),
            )));
        }

        LexResult::Lexed(Self(many))
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

impl<const N: usize, L: LexT> Lex for Exactly<N, L>
where
    [(); N]: Sized,
{
    fn peek<S: Source>(input: &SourceStream<S>) -> Peek<Self> {
        if N == 0 {
            return Peek::Possible(PhantomData::<Self>);
        }

        <L as Lex>::peek(input).map()
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> LexResult<Self> {
        let many: Many<L> = Lex::lex(input)?;

        if many.len() != N {
            return LexResult::Errant(input.error(format!(
                "Expected {N} {} tokens: got {}.",
                std::any::type_name::<L>(),
                many.len()
            )));
        }

        // SAFETY: Just checked the length, so unwrap okay.
        let many: [L; N] = unsafe { many.try_into().unwrap_unchecked() };

        LexResult::Lexed(Self(many))
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
