//!
//! ## Punctuators
//!
//! Pieces of punctuation: `{}[]:,`.
//!

use avjason_macros::{verbatim as v, SpecRef};

use crate::{
    common::Source,
    lexing::{LexError, LexT, SourceStream},
};

///
/// `{`
///
pub type OpenBrace = v!('{');

///
/// `}`
///
pub type CloseBrace = v!('}');

///
/// `[`
///
pub type OpenBracket = v!('[');

///
/// `]`
///
pub type CloseBracket = v!(']');

///
/// `:`
///
pub type Colon = v!(':');

///
/// `,`
///
pub type Comma = v!(',');

///
/// `{ } [ ] : ,`
///
#[SpecRef("JSON5Punctuator")]
pub enum Punctuator {
    OpenBrace(OpenBrace),
    CloseBrace(CloseBrace),
    OpenBracket(OpenBracket),
    CloseBracket(CloseBracket),
    Colon(Colon),
    Comma(Comma),
}

impl LexT for Punctuator {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <OpenBrace as LexT>::peek(input)
            || <CloseBrace as LexT>::peek(input)
            || <OpenBracket as LexT>::peek(input)
            || <CloseBracket as LexT>::peek(input)
            || <Colon as LexT>::peek(input)
            || <Comma as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .unwrap_as_result() ok since Self::peek() -> one variant present.
        input
            .lex()
            .map(Self::OpenBrace)
            .or(|| input.lex().map(Self::CloseBrace))
            .or(|| input.lex().map(Self::OpenBracket))
            .or(|| input.lex().map(Self::CloseBracket))
            .or(|| input.lex().map(Self::Colon))
            .or(|| input.lex().map(Self::Comma))
            .unwrap_as_result()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{file::SourceFile, Source},
        lexing::Exactly,
    };

    use super::Punctuator;

    #[test]
    fn mixed_test() {
        let source = SourceFile::dummy_file("{}[]:,");
        let input = &mut source.stream();
        let puncts: Exactly<6, Punctuator> = input.lex().expect("Valid parse");

        assert!(matches!(
            &*puncts,
            &[
                Punctuator::OpenBrace(_),
                Punctuator::CloseBrace(_),
                Punctuator::OpenBracket(_),
                Punctuator::CloseBracket(_),
                Punctuator::Colon(_),
                Punctuator::Comma(_)
            ]
        ))
    }
}
