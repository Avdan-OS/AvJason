//!
//! ## WhiteSpace
//! Empty space that doesn't contribute syntactically.
//!

use avjason_macros::{ECMARef, Spanned};
use finl_unicode::categories::{CharacterCategories, MinorCategory};

use crate::{
    common::{Source, Span},
    lexing::{LexError, LexT, SourceStream},
};

///
/// Whitespace characters.
///
#[derive(Debug, Spanned)]
#[ECMARef("WhiteSpace", "https://262.ecma-international.org/5.1/#sec-7.2")]
pub struct WhiteSpace {
    span: Span,
}

///
/// Is this character whitespace?
///
/// Compliant with [Table 2, Section 7.2](https://262.ecma-international.org/5.1/#sec-7.2) of the ECMAScript specification.
///
fn is_whitespace(ch: &char) -> bool {
    use MinorCategory::Zs;

    match ch {
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' => true,
        c if matches!(c.get_minor_category(), Zs) => true,
        _ => false,
    }
}

impl LexT for WhiteSpace {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming(is_whitespace)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // Since Self::peek() -> there's at least one character.
        let (span, _) = input.take_while(is_whitespace).unwrap();
        Ok(Self { span })
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{file::SourceFile, Source};

    use super::WhiteSpace;

    #[test]
    fn lex_whitespace() {
        let ws = "\t\t \t\t\u{000B}\u{000C}";
        let source = SourceFile::dummy_file(ws);
        let input = &mut source.stream();
        let whitespace: WhiteSpace = input.lex().expect("Valid parse");
        assert_eq!(source.source_at(whitespace), Some(ws.to_string()))
    }
}