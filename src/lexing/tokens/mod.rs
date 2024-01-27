//!
//! Lexical tokens.
//! 

use crate::common::{Loc, Source, Span, SpanIter, ToSpan};

use super::utils::{LexError, LexT, SourceStream};

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
            locs.push((loc..(loc+1)).to_span(input.source()));
        }
        
        Ok(Self {
            span: locs.into_iter().combine()
                .expect("DO PUT EMPTY STRINGS IN VERBATIM!"),
        })
        
    }
}