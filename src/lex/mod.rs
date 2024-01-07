//!
//! ## Step 1: Lexing
//!
//! This process involves converting source code
//! into what's known as a [lexical grammar](https://en.wikipedia.org/wiki/Lexical_grammar).
//!
//! This step will eventually yield tokens, which can then be
//! checked against syntax.
//!
//! This lexical grammar is defined in the [JSON5 specification](https://spec.json5.org/#lexical-grammar).
//!

pub mod utils;
pub mod whitespace;
pub mod line_terminator;

use avjason_macros::{SpecRef, Spanned};

use self::{whitespace::WhiteSpace, line_terminator::LineTerminator};

pub(crate) use utils::{LexError, Lex, LexResult};

///
/// ## JSON5InputElement
///
/// All possible acceptable things our lexer accepts.
/// * A superset of valid tokens: Valid Tokens + { Comments, Whitespace, LineTerminator, }.
///
#[SpecRef("JSON5InputElement")]
#[derive(Debug, Spanned)]
pub(crate) enum InputElement {
    WhiteSpace(WhiteSpace),
    LineTerminator(LineTerminator),
    // Comment(Comment),
    // Token(Token),
}
