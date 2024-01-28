use crate::common::{Loc, Source, Span, Spanned, ToSpan};

use super::{Lex, LexResult};

#[derive(Debug, Clone)]
pub struct SourceStream<'a, S: Source> {
    index: usize,
    source: &'a S,
}

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
        input.source.characters()[input.index..(input.index + chars.len())] == chars
    }
}

impl<'a, S: Source> SourceStream<'a, S> {
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

    pub fn upcoming<L: Lookahead + ?Sized>(&self, lookahead: &L) -> bool {
        lookahead.upcoming(self)
    }
}

impl<'a, S: Source> Spanned for SourceStream<'a, S> {
    fn span(&self) -> Span {
        (self.index..=self.index).to_span(self.source)
    }
}
