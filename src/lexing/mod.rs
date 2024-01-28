//!
//! The process of lexing involves converting [char]s
//! from source code into lexical tokens according to
//! some [lexical grammar](https://en.wikipedia.org/wiki/Lexical_grammar).
//!

pub mod utils;
pub mod tokens;

pub use tokens::{CharPattern, Verbatim};
pub use utils::stream::{SourceStream, CharacterRange};