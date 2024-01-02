//!
//! Escape sequences.
//!

use avjason_macros::{Lex, Spanned};

use crate::utils::{SourceIter, Span, TryIntoSpan};
use crate::lex::IntoLexResult;

use super::tokens::{Lex, LineTerminator};

#[inline]
pub fn is_hex_digit(ch: &char) -> bool {
    ch.is_ascii_hexdigit()
}

#[derive(Debug, Spanned)]
#[Lex]
pub enum EscapeSequence {
    Unicode(UnicodeEscapeSequence),
    Hex(HexEscapeSequence),
    Null(NullEscapeSequence),
    Character(CharacterEscapeSequence),
}

#[derive(Debug, Spanned)]
#[Lex]
pub enum CharacterEscapeSequence {
    Single(SingleEscapeCharacter),
    NonEscape(NonEscapeCharacter),
}

#[derive(Debug, Spanned)]
pub struct SingleEscapeCharacter(Span);

impl Lex for SingleEscapeCharacter {
    fn lex(input: &mut SourceIter) -> Option<Self> {
        if !Self::peek(input) {
            return None;
        }

        let loc = input.next()?.0;
        Some(Self(Span::single_char(loc)))
    }

    fn peek(input: &SourceIter) -> bool {
        matches!(
            input.peek(),
            Some(&'\'' | &'"' | &'\\' | &'b' | &'f' | &'n' | &'r' | &'t' | &'v')
        )
    }
}

#[derive(Debug, Spanned)]
pub struct NonEscapeCharacter(Span);

struct EscapeCharacter;

impl Lex for EscapeCharacter {
    fn lex(_: &mut SourceIter) -> Option<Self> {
        unimplemented!()
    }

    fn peek(input: &SourceIter) -> bool {
        let Some(ch) = input.peek() else {
            return false;
        };

        SingleEscapeCharacter::peek(input)
            || ch.is_ascii_digit() // DecimalDigit
            || ch == &'x'
            || ch == &'u'
    }
}

impl Lex for NonEscapeCharacter {
    fn lex(input: &mut SourceIter) -> Option<Self> {
        if !Self::peek(input) {
            return None;
        }

        let loc = input.next()?.0;
        Some(Self(Span::single_char(loc)))
    }

    fn peek(input: &SourceIter) -> bool {
        !(EscapeCharacter::peek(input) || LineTerminator::peek(input))
    }
}

#[derive(Debug, Spanned)]
pub struct NullEscapeSequence(Span);

impl Lex for NullEscapeSequence {
    fn lex(input: &mut SourceIter) -> Option<Self> {
        if !Self::peek(input) {
            return None;
        }

        let loc = input.next()?.0;
        Some(Self(Span::single_char(loc)))
    }

    fn peek(input: &SourceIter) -> bool {
        // with lookahead: not DecimalDigit.
        input.peek() == Some(&'0') && !input.peek2().map(char::is_ascii_digit).unwrap_or(false)
    }
}

#[derive(Debug, Spanned)]
pub struct HexEscapeSequence(Span);

impl Lex for HexEscapeSequence {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let start = input.next().unwrap().0;

        let mut end = start;

        for _ in 0..2 {
            if input.peek().map(is_hex_digit).unwrap_or(false) {
                end = input.next().unwrap().0;
            } else {
                return input.error()
                    .expected(Some(-1..1), "<HEX DIGIT>");
            }
        }

        Ok(Some(Self(TryIntoSpan::try_into_span(start..=end).unwrap())))
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek() == Some(&'x') && input.relative_match(1..=2, is_hex_digit)
    }
}

#[derive(Debug, Spanned)]
pub struct UnicodeEscapeSequence(Span);

impl Lex for UnicodeEscapeSequence {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let start = input.next().unwrap().0;

        let mut end = start;

        for _ in 0..4 {
            if is_hex_digit(input.peek().unwrap()) {
                end = input.next().unwrap().0;
            } else {
                return input.error()
                    .expected(Some(-1..), "<HEX DIGIT>")
            }
        }

        Ok(Some(Self(TryIntoSpan::try_into_span(start..=end).unwrap())))
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek() == Some(&'u') && input.relative_match(1..=4, is_hex_digit)
    }
}
