use avjason_macros::{ECMARef, Spanned};
use finl_unicode::categories::{MinorCategory, CharacterCategories};

use crate::{utils::Span, lex::utils::capture_while};

use super::{Lex, LexResult};

///
/// ## WhiteSpace
///
/// Whitespace characters (e.g. spaces, tabs, etc.).
///
#[ECMARef(
    "WhiteSpace",
    "https://www.ecma-international.org/ecma-262/5.1/#sec-7.2"
)]
#[derive(Debug, Spanned)]
pub struct WhiteSpace {
    span: Span,
}

impl WhiteSpace {
    ///
    /// Implementation matching Table 2 in [Section 7.2](https://262.ecma-international.org/5.1/#sec-7.2).
    ///
    fn is_whitespace(ch: &char) -> bool {
        use MinorCategory::*;

        match ch {
            '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' => true,
            c if c.get_minor_category() == Zs => true,
            _ => false,
        }
    }
}

impl Lex for WhiteSpace {
    fn lex(input: &mut crate::utils::SourceIter) -> LexResult<Self> {
        LexResult::Ok(Self {
            span: capture_while(input, Self::is_whitespace)?,
        })
    }

    fn peek(input: &crate::utils::SourceIter) -> bool {
        input.peek().map(Self::is_whitespace).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {

}