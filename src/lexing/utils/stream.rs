use std::marker::ConstParamTy;

use crate::common::{Loc, Source, Span, Spanned, ToSpan};

use super::{Lex, LexResult};

///
/// Things that [SourceStream] can
/// check are coming up.
///
pub trait Lookahead {
    fn upcoming<S: Source>(&self, input: &SourceStream<S>) -> bool;
}

impl Lookahead for str {
    fn upcoming<S: Source>(&self, input: &SourceStream<S>) -> bool {
        let chars = self.chars().collect::<Vec<_>>();
        input
            .source
            .characters()
            .get(input.index..(input.index + chars.len()))
            .map(|st| st == chars)
            .unwrap_or(false)
    }
}

///
/// A const-friendly implementation of [std::ops::Range]<char>.
/// 
/// This works with the [crate::verbatim] macro to support
/// the range syntax: `v!('0'..='9')`.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CharacterRange {
    ///
    /// Inclusive start.
    ///
    pub start: char,

    ///
    /// Exclusive end.
    ///
    pub end: char,
}

impl ConstParamTy for CharacterRange {}

impl Lookahead for CharacterRange {
    fn upcoming<S: Source>(&self, input: &SourceStream<S>) -> bool {
        input
            .source
            .characters()
            .get(input.index)
            .map(|ch| (self.start..self.end).contains(ch))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct SourceStream<'a, S: Source> {
    index: usize,
    source: &'a S,
}

impl<'a, S: Source> SourceStream<'a, S> {
    ///
    /// Create a new stream from a source.
    ///
    pub fn new(source: &'a S) -> Self {
        Self { index: 0, source }
    }

    ///
    /// Returns the source where this [SourceStream]
    /// came from.
    ///
    pub fn source(&self) -> &S {
        self.source
    }

    ///
    /// Take the next character in this [SourceStream].
    ///
    pub fn take(&mut self) -> Option<(Loc, char)> {
        let index = self.index;

        if let Some(ch) = self.source.characters().get(index) {
            self.index += 1;
            return Some((Loc(index), *ch));
        }

        None
    }

    ///
    /// Attempt to lex for token `L`.
    ///
    pub fn lex<L: Lex>(&mut self) -> LexResult<L> {
        Lex::lex(self)
    }

    ///
    /// Checks if a lookahead pattern is next in the stream.
    /// 
    pub fn upcoming<L: Lookahead + ?Sized>(&self, lookahead: &L) -> bool {
        lookahead.upcoming(self)
    }
}

impl<'a, S: Source> Spanned for SourceStream<'a, S> {
    fn span(&self) -> Span {
        (self.index..=self.index).to_span(self.source)
    }
}
