//!
//! Utilities for lexing.
//!

use std::fmt::Debug;

use thiserror::Error;

use crate::utils::{SourceIter, Span, TryIntoSpan, Spanned};

///
/// Errors that can occur during lexing.
///
#[derive(Debug, Clone, Error, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) enum LexError {
    ///
    /// Expected a specific character/sub-token at a location.
    ///
    #[error("Expected `{token}` {extra}\n\tat {position}")]
    Expected {
        token: String,
        position: String,
        extra: String,
    },

    ///
    /// Invalid character at acertain position.
    ///
    #[error("Unexpected `{token}` {extra}\n\tat {position}")]
    Unexpected {
        token: String,
        position: String,
        extra: String,
    },
}

///
/// Convenience type for lexer result types.
///
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) enum LexResult<T> {
    ///
    /// No lexical grammars were violated, but the
    /// characters cannot be lexed into this token,
    /// so skip trying to lex this token.
    ///
    /// It's important to note that the character iterator
    /// *has not* advanced.
    ///
    Stop,

    ///
    /// Successful lex result according to
    /// our lexical grammar.
    ///
    /// Character iterator has been advanced.
    ///
    Ok(T),

    ///
    /// Some grammatical rules have violated whilst lexing
    /// -- stop lexing immeadiately.
    ///
    Err(LexError),
}

///
/// Implementing [std::ops::Try], though experimental, allows the leverage of
/// the `?` operator for our convinence.
///
/// Now, we can stop halfway through if:
/// * we get an error ([LexResult::Err]), or
/// * we know that we don't need to continue the rest of the computation,
///     but don't have an error ([LexResult::Stop]).
///
impl<T> std::ops::Try for LexResult<T> {
    type Output = T;

    type Residual = LexResult<std::convert::Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            LexResult::Stop => std::ops::ControlFlow::Break(LexResult::Stop),
            LexResult::Ok(o) => std::ops::ControlFlow::Continue(o),
            LexResult::Err(err) => std::ops::ControlFlow::Break(LexResult::Err(err)),
        }
    }
}

impl<T> std::ops::FromResidual for LexResult<T> {
    fn from_residual(residual: <Self as std::ops::Try>::Residual) -> Self {
        match residual {
            LexResult::Stop => Self::Stop,
            LexResult::Err(err) => Self::Err(err),
            LexResult::Ok(_) => unreachable!(),
        }
    }
}

impl<T> From<Option<T>> for LexResult<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(s) => Self::Ok(s),
            None => Self::Stop,
        }
    }
}

///
/// ## Lexing
/// 
/// Attempts to lex characters into a proto-token.
/// 
/// Uses [LexResult] as control flow.
/// 
pub(crate) trait Lex: Sized + Debug  {
    fn lex(input: &mut SourceIter) -> LexResult<Self>;
    fn peek(input: &SourceIter) -> bool;
}

///
/// Utility implementation for
/// optional lexing.
///
impl<T> Lex for Option<T>
where
    T: Lex,
{
    fn lex(input: &mut SourceIter) -> LexResult<Self> {
        if !T::peek(input) {
            return LexResult::Ok(None);
        }

        match T::lex(input) {
            LexResult::Ok(t) => LexResult::Ok(Some(t)),

            // Here, we allow `Stop`-s since they indicate that T is not present.
            // So, it's a valid parse -- we have nothing.
            LexResult::Stop => LexResult::Ok(None),

            // Error variant indicates that something looking like T
            // was present, but it doesn't conform to T's grammar.
            LexResult::Err(err) => LexResult::Err(err),
        }
    }

    fn peek(_: &SourceIter) -> bool {
        // Checking to see if an optional thing is present is
        // virtual insanity -- it's *always true*.
        unimplemented!("Tautologically useless!")
    }
}

///
/// Keeps collecting characters (into a span)
/// while a condition is true.
///
pub(crate) fn capture_while(
    input: &mut SourceIter,
    pred: impl Fn(&char) -> bool,
) -> LexResult<Span> {
    let pred = &pred;
    if !input.peek().map(pred).unwrap_or(false) {
        return LexResult::Stop;
    }
    // Unwrap ok as we know (a) char exists, and (b) is whitespace.
    let start = input.next().unwrap().0;
    let mut end = start + 1;
    while let Some(ch) = input.peek() {
        if pred(ch) {
            // Unwrap ok for same reason as above.
            end = input.next().unwrap().0;
        } else {
            break;
        }
    }

    LexResult::Ok(TryIntoSpan::try_into_span(start..=end).unwrap())
}

#[cfg(test)]
mod tests {
    use crate::lex::utils::LexResult;

    #[test]
    fn lex_result() {
        fn dummy_lexer(input: usize) -> LexResult<usize> {
            match input % 3 {
                0 => LexResult::Ok(input),
                1 => LexResult::Stop,
                2 => LexResult::Err(crate::lex::utils::LexError::Expected {
                    token: "number that is {0, 1} mod 3".to_string(),
                    position: "nope".to_string(),
                    extra: "".to_string(),
                }),
                _ => unreachable!(),
            }
        }

        fn dummy(input: [usize; 3]) -> LexResult<[usize; 3]> {
            let first = dummy_lexer(input[0])?;
            let second = dummy_lexer(input[1])?;
            let third = dummy_lexer(input[2])?;

            LexResult::Ok([first, second, third])
        }

        assert_eq!(dummy([1, 2, 3]), LexResult::Stop); // Stops on 3n + 1
        assert!(matches!(dummy([0, 2, 3]), LexResult::Err(_))); // Error on 3n + 2
    }
}
