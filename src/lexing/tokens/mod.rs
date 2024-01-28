//!
//! Lexical tokens.
//!

use avjason_macros::Spanned;

use crate::common::{Loc, Source, Span, SpanIter, ToSpan};

use super::utils::{stream::CharacterRange, LexError, LexT, SourceStream};

///
/// Do not use me directly, use [crate::verbatim] instead!
///
#[derive(Debug, Spanned)]
pub struct Verbatim<const A: &'static str> {
    span: Span,
}

impl<const A: &'static str> LexT for Verbatim<A> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming(A)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let mut locs = vec![];

        for _ in 0..A.len() {
            let (Loc(loc), _) = input.take().unwrap();
            locs.push((loc..(loc + 1)).to_span(input.source()));
        }

        Ok(Self {
            // If A == "", then an empty Span is returned.
            span: locs.into_iter().combine(),
        })
    }
}

///
/// Matches a character with a given range.
///
#[derive(Debug, Spanned)]
pub struct CharPattern<const R: CharacterRange> {
    raw: char,
    span: Span,
}

impl<const R: CharacterRange> CharPattern<R> {
    pub fn raw(&self) -> &char {
        &self.raw
    }
}

impl<const R: CharacterRange> LexT for CharPattern<R> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming(&R)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let (loc, raw) = input.take().unwrap();
        Ok(Self {
            raw,
            span: Span::from(loc),
        })
    }
}

#[cfg(test)]
mod tests {
    use avjason_macros::verbatim as v;

    use crate::{
        common::{file::SourceFile, Source},
        lexing::{
            tokens::CharPattern,
            utils::{stream::CharacterRange, Many},
        },
    };

    use super::Verbatim;

    #[test]
    fn verbatim() {
        let source = SourceFile::dummy_file(",.");
        let input = &mut source.stream();
        let comma: Verbatim<","> = input.lex().expect("Valid parse");
        println!("{comma:?}")
    }

    #[test]
    fn ranged() {
        const DIGIT: CharacterRange = CharacterRange {
            start: '0',
            end: ':',
        };

        let source = SourceFile::dummy_file("126439012363421890");
        let input = &mut source.stream();
        let digit: Many<CharPattern<DIGIT>> = input.lex().expect("Valid parse");
        println!("{digit:?}")
    }

    #[test]
    fn macro_test() {
        type Comma = v!(',');
        type DoubleColon = v!("::");
        type Digit = v!('0'..='9');

        {
            let source = SourceFile::dummy_file(",");
            let input = &mut source.stream();
            let _: Comma = input.lex().expect("Valid parse");
        }
        
        {
            let source = SourceFile::dummy_file("::");
            let input = &mut source.stream();
            let _: DoubleColon = input.lex().expect("Valid parse");
        }

        {
            let source = SourceFile::dummy_file("126439012363421890");
            let input = &mut source.stream();
            let _: Many<Digit> = input.lex().expect("Valid parse");
        }
    }
}
