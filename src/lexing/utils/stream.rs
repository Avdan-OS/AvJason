use crate::common::{Loc, Source, Span, Spanned, ToSpan};

use super::{Lex, LexResult};

#[derive(Debug, Clone)]
pub struct SourceStream<'a, S: Source> {
    index: usize,
    source: &'a S,
}

impl<'a, S: Source> SourceStream<'a, S> {
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
}

impl<'a, S: Source> Spanned for SourceStream<'a, S> {
    fn span(&self) -> Span {
        (self.index..=self.index).to_span(self.source)
    }
}
