//!
//! String literals.
//!

use avjason_macros::{Lex, Spanned};

use crate::{
    lex::tokens::{LineTerminator, LineTerminatorSeq},
    utils::{SourceIter, Span, TryIntoSpan},
};

use super::{escape::EscapeSequence, tokens::Lex, IntoLexResult, LexError, LexResult};

#[derive(Debug, Spanned)]
#[Lex]
pub enum LString {
    Single(SingleString),
    Double(DoubleString),
}

fn eat_inner_chars(
    input: &mut SourceIter,
    delimit: char,
) -> Result<Option<Vec<StrFrag>>, LexError> {
    let mut contents = vec![];

    while let Some(ch) = input.peek() {
        if ch == &delimit {
            break;
        }

        if LineTerminator::peek(input) {
            return input.error().unexpected(Some(0..1), "<LINE BREAK>");
        }

        if ch == &'\\' {
            // Escape sequence.
            let mut fork = input.fork();
            fork.offset(1);

            if EscapeSequence::peek(&fork) {
                contents.push(StrFrag::EscSeq(EscapeSequence::lex(&mut fork).into_lex_result()?.unwrap()));

                input.advance_to(fork);
                continue;
            }

            if LineTerminatorSeq::peek(&fork) {
                contents.push(StrFrag::LineEsc(LineTerminatorSeq::lex(&mut fork).into_lex_result()?.unwrap()));

                input.advance_to(fork);
                continue;
            }

            return input
                .error()
                .expected(Some(0..1), "Escaped Newline, or escape sequence");
        }

        let (_, c) = input.next().unwrap();
        contents.push(StrFrag::Char(c));
    }

    Ok(Some(contents))
}

#[derive(Debug)]
pub enum StrFrag {
    Char(char),
    EscSeq(EscapeSequence),
    LineEsc(LineTerminatorSeq),
}

#[derive(Debug, Spanned)]
pub struct SingleString(Span, Vec<StrFrag>);

impl Lex for SingleString {
    fn lex(input: &mut SourceIter) -> LexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let start = input.next().unwrap().0;

        let contents = eat_inner_chars(input, '\'')?.unwrap();

        if input.peek() != Some(&'\'') {
            return input.error().expected(Some(0..1), "\'");
        }

        let end = input.next().unwrap().0;

        Ok(Some(Self(
            TryIntoSpan::try_into_span(start..=end).unwrap(),
            contents,
        )))
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek() == Some(&'\'')
    }
}

#[derive(Debug, Spanned)]
pub struct DoubleString(Span, Vec<StrFrag>);

impl Lex for DoubleString {
    fn lex(input: &mut SourceIter) -> LexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let start = input.next().unwrap().0;

        let contents = eat_inner_chars(input, '\"')?.unwrap();

        if input.peek() != Some(&'\"') {
            return input.error().expected(Some(0..1), "\"");
        }

        let end = input.next().unwrap().0;

        Ok(Some(Self(
            TryIntoSpan::try_into_span(start..=end).unwrap(),
            contents,
        )))
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek() == Some(&'"')
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lex::{strings::DoubleString, tokens::Lex, IntoLexResult, LexResult},
        utils::SourceFile,
    };

    fn test_lex<T: Lex>(s: impl ToString, src: &str) -> LexResult<T> {
        let src = SourceFile::dummy_file(format!("test.{}", s.to_string()), src);
        let iter = &mut src.iter();
        T::lex(iter).into_lex_result()
    }

    #[test]
    fn unicode_escape() {
        let twice_valid = test_lex::<DoubleString>(0, r#""\u1522\u2431""#);
        assert!(matches!(twice_valid, Ok(Some(_))));
        let once_valid_once_invalid = test_lex::<DoubleString>(1, r#""\u1522\u241""#);
        assert!(once_valid_once_invalid.is_err());
        let once_invalid = test_lex::<DoubleString>(3, r#""\u1S2Y""#);
        assert!(once_invalid.is_err());
    }

    #[test]
    fn hex_escape() {
        let twice_valid = test_lex::<DoubleString>(0, r#""\x0F\xFF""#);
        assert!(matches!(twice_valid, Ok(Some(_))));
        let once_valid_once_invalid = test_lex::<DoubleString>(0, r#""\x0F\xSF""#);
        assert!(once_valid_once_invalid.is_err());
        let once_invalid = test_lex::<DoubleString>(0, r#""\xSF""#);
        assert!(once_invalid.is_err());
    }

    #[test]
    fn single_char() {
        let escaped = test_lex::<DoubleString>(0, r#""\t\r\v\n\"\\""#);
        assert!(matches!(escaped, Ok(Some(_))));
        let normal = test_lex::<DoubleString>(0, r#""\!\?\:\@\~\#\}\{\(\)\&\$""#);
        assert!(matches!(normal, Ok(Some(_))));
    }

    #[test]
    fn null_escape() {
        let valid = test_lex::<DoubleString>(0, r#""\0\0\0\0\0\0\0\0""#);
        assert!(matches!(valid, Ok(Some(_))));
        let invalid = test_lex::<DoubleString>(0, r#""\00\01\04\06"#);
        assert!(invalid.is_err());
    }

    #[test]
    fn mixed_escapes() {
        let test0 = test_lex::<DoubleString>(0, r#""\v\!\%\x00""#);
        assert!(matches!(test0, Ok(Some(_))));
        let test1 = test_lex::<DoubleString>(1, r#""\v\!\% abhbdasjdas^da'''gadudgasi a@@@~ {} dauasdhi\x00""#);
        assert!(matches!(test1, Ok(Some(_))));
    }
}
