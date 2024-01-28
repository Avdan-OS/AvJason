//!
//! ## Line Terminators
//!
//! These signify the end of lines (although techincally [LineTerminatorSequence]s do!)
//!

use avjason_macros::{verbatim as v, ECMARef, Spanned};

use crate::{
    common::Source,
    lexing::{Lex, LexError, LexT, SourceStream},
};

#[ECMARef("LineTerminator", "https://262.ecma-international.org/5.1/#sec-7.3")]
#[derive(Debug, Spanned)]
pub enum LineTerminator {
    LF(v!('\n')),
    CR(v!('\r')),
    LS(v!('\u{2028}')),
    PS(v!('\u{2029}')),
}

#[ECMARef(
    "LineTerminatorSequence",
    "https://262.ecma-international.org/5.1/#sec-7.3"
)]
#[derive(Debug, Spanned)]
pub enum LineTerminatorSequence {
    CRLF(v!("\r\n")),
    LF(v!('\n')),
    CR(v!('\r')),
    LS(v!('\u{2028}')),
    PS(v!('\u{2029}')),
}

impl LexT for LineTerminator {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <v!('\n') as LexT>::peek(input)
            || <v!('\r') as LexT>::peek(input)
            || <v!('\u{2028}') as LexT>::peek(input)
            || <v!('\u{2029}') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .into_result() ok since we know there's at least one upcoming variant.
        Lex::lex(input)
            .map(Self::LF)
            .or(|| Lex::lex(input).map(Self::CR))
            .or(|| Lex::lex(input).map(Self::LS))
            .or(|| Lex::lex(input).map(Self::PS))
            .into_result()
    }
}

impl LexT for LineTerminatorSequence {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <v!('\n') as LexT>::peek(input)
            || <v!('\r') as LexT>::peek(input)
            || <v!('\u{2028}') as LexT>::peek(input)
            || <v!('\u{2029}') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .into_result() ok since we know there's at least one upcoming variant.
        Lex::lex(input)
            .map(Self::CRLF)
            .or(|| Lex::lex(input).map(Self::LF))
            .or(|| Lex::lex(input).map(Self::CR))
            .or(|| Lex::lex(input).map(Self::LS))
            .or(|| Lex::lex(input).map(Self::PS))
            .into_result()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{file::SourceFile, Source},
        lexing::{tokens::line_terminator::LineTerminatorSequence, Exactly},
    };

    use super::LineTerminator;

    #[test]
    fn line_terminators() {
        let source = SourceFile::dummy_file("\r\n\u{2028}\u{2029}");
        let input = &mut source.stream();
        let new_lines: Exactly<4, LineTerminator> = input.lex().expect("Valid parse");
        assert!(matches!(
            &*new_lines,
            &[
                LineTerminator::CR(_),
                LineTerminator::LF(_),
                LineTerminator::LS(_),
                LineTerminator::PS(_)
            ]
        ));
    }

    #[test]
    fn line_terminator_sequences() {
        let source = SourceFile::dummy_file("\r\r\n\n\u{2028}\u{2029}");
        let input = &mut source.stream();
        let new_lines: Exactly<5, LineTerminatorSequence> = input.lex().expect("Valid parse");
        assert!(matches!(
            &*new_lines,
            &[
                LineTerminatorSequence::CR(_),
                LineTerminatorSequence::CRLF(_),
                LineTerminatorSequence::LF(_),
                LineTerminatorSequence::LS(_),
                LineTerminatorSequence::PS(_)
            ]
        ));
    }
}
