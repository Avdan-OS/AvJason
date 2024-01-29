//!
//! The process of lexing involves converting [char]s
//! from source code into lexical tokens according to
//! some [lexical grammar](https://en.wikipedia.org/wiki/Lexical_grammar).
//!

pub mod tokens;
pub mod utils;

pub use utils::{
    stream::CharacterRange,
    verbatim::{CharPattern, Verbatim},
    AtLeast, Exactly, Lex, LexError, LexResult, LexT, Many, Peek, SourceStream,
};
