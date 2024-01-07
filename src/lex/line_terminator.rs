use avjason_macros::{ECMARef, Spanned};

use crate::utils::{SourceIter, Span, TryIntoSpan};

use super::{utils::capture_while, Lex, LexResult};

///
/// ## LineTerminator
///
/// Defined in [Section 7.3 Table 3](https://262.ecma-international.org/5.1/#sec-7.3).
/// Characters that end lines (single characters only, no `\r\n` here.)
///
/// See [LineTerminatorSequence] for the version including `\r\n`.
///
#[ECMARef("LineTerminator", "https://262.ecma-international.org/5.1/#sec-7.3")]
#[derive(Debug, Spanned)]
pub struct LineTerminator {
    span: Span,
}

impl LineTerminator {
    fn is_line_terminator(ch: &char) -> bool {
        matches!(ch, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
    }
}

impl Lex for LineTerminator {
    fn lex(input: &mut SourceIter) -> LexResult<Self> {
        if !Self::peek(input) {
            return LexResult::Stop;
        }

        LexResult::Ok(Self {
            span: capture_while(input, Self::is_line_terminator)?,
        })
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek().map(Self::is_line_terminator).unwrap_or(false)
    }
}

///
/// ## LineTerminatorSequence
///
/// All accepted line endings, including `\r\n`.
///
#[ECMARef(
    "LineTerminatorSequence",
    "https://262.ecma-international.org/5.1/#sec-7.3"
)]
#[derive(Debug, Spanned)]
pub enum LineTerminatorSequence {
    LF(Span),
    CR(Span),
    LS(Span),
    PS(Span),
    CRLF(Span),
}

impl Lex for LineTerminatorSequence {
    fn lex(input: &mut SourceIter) -> LexResult<Self> {
        if !Self::peek(input) {
            return LexResult::Stop;
        }

        if let Some(ch) = input.peek() {
            return match ch {
                '\u{000A}' => LexResult::Ok(Self::CR(Span::single_char(input.next().unwrap().0))),
                '\u{000D}' => {
                    if input.peek2().map(|n| n == &'\u{000A}').unwrap_or(false) {
                        let start = input.next().unwrap().0;
                        let end = input.next().unwrap().0;
                        LexResult::Ok(Self::CRLF(TryIntoSpan::try_into_span(start..=end).unwrap()))
                    } else {
                        LexResult::Ok(Self::CR(Span::single_char(input.next().unwrap().0)))
                    }
                }
                '\u{2028}' => LexResult::Ok(Self::CR(Span::single_char(input.next().unwrap().0))),
                '\u{2029}' => LexResult::Ok(Self::CR(Span::single_char(input.next().unwrap().0))),
                _ => LexResult::Stop,
            }
        }

        LexResult::Stop
    }

    fn peek(input: &SourceIter) -> bool {
        input
            .peek()
            .map(LineTerminator::is_line_terminator)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {}
