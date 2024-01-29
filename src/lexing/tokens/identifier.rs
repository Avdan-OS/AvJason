//!
//! ## Identifiers
//!

use std::iter::once;

use finl_unicode::categories::{CharacterCategories, MinorCategory};

use crate::{
    common::{Source, Spanned},
    lexing::{Lex, LexError, LexResult, LexT, Many, SourceStream},
    unicode as u, verbatim as v, ECMARef, Spanned, SpecRef,
};

use super::{
    escapes::UnicodeEscapeSequence,
    string::{collect_cv_into_utf16, CharacterValue, StringValue},
};

#[SpecRef("JSON5Identifier")]
#[derive(Debug, Spanned)]
pub struct Identifier(IdentifierName);

///
/// > Identifier Names are tokens that are interpreted
/// > according to the grammar given in the “Identifiers” section
/// > of chapter 5 of the Unicode standard, with some small modifications.
///
#[ECMARef("IdentifierName", "https://262.ecma-international.org/5.1/#sec-7.6")]
#[derive(Debug, Spanned)]
pub struct IdentifierName(IdentifierStart, Many<IdentifierPart>);

///
/// The first character in an identifier.
/// 
#[ECMARef("IdentifierStart", "https://262.ecma-international.org/5.1/#sec-7.6")]
#[derive(Debug, Spanned, Clone)]
pub enum IdentifierStart {
    Letter(UnicodeLetter),
    Dollar(v!('$')),
    Underscore(v!('_')),
    Escape(v!('\\'), UnicodeEscapeSequence),
}

///
/// Any part of an identifier folowing the starting part.
/// 
#[ECMARef("IdentifierPart", "https://262.ecma-international.org/5.1/#sec-7.6")]
#[derive(Debug, Spanned, Clone)]
pub enum IdentifierPart {
    ///
    /// This is not part of the ECMAScript spec,
    /// but is necessary in order to get the context
    /// correctly in the escaped character's validity checks.
    ///
    Escape(v!('\\'), UnicodeEscapeSequence),
    Start(IdentifierStart),
    CombiningMark(UnicodeCombiningMark),
    Digit(UnicodeDigit),
    ConnectorPunctuation(UnicodeConnectorPunctuation),

    ///
    /// Zero width non-joiner
    ///
    ZWNJ(v!('\u{200C}')),

    ///
    /// Zero width joiner
    ///
    ZWJ(v!('\u{200D}')),
}

///
/// > any character in the Unicode categories “Uppercase letter (Lu)”,
/// > “Lowercase letter (Ll)”, “Titlecase letter (Lt)”, “Modifier letter (Lm)”,
/// > “Other letter (Lo)”, or “Letter number (Nl)”
///
#[ECMARef("UnicodeLetter", "https://262.ecma-international.org/5.1/#sec-7.6")]
pub type UnicodeLetter = u!(Lu | Ll | Lt | Lm | Lo | Nl);

///
/// > any character in the Unicode categories “Non-spacing mark (Mn)”
/// > or “Combining spacing mark (Mc)”
///
#[ECMARef(
    "UnicodeCombiningMark",
    "https://262.ecma-international.org/5.1/#sec-7.6"
)]
pub type UnicodeCombiningMark = u!(Mn | Mc);

///
/// > any character in the Unicode category “Decimal number (Nd)”
///
#[ECMARef("UnicodeDigit", "https://262.ecma-international.org/5.1/#sec-7.6")]
pub type UnicodeDigit = u!(Nd);

///
/// any character in the Unicode category “Connector punctuation (Pc)”
///
#[ECMARef(
    "UnicodeConnectorPunctuation",
    "https://262.ecma-international.org/5.1/#sec-7.6"
)]
pub type UnicodeConnectorPunctuation = u!(Pc);

// ---

///
/// What characters does this identifier part accept?
///
pub trait CharacterAcceptor {
    fn accepts(ch: &char) -> bool;
}

impl CharacterAcceptor for IdentifierStart {
    fn accepts(ch: &char) -> bool {
        use MinorCategory::*;
        match ch {
            c if matches!(c.get_minor_category(), Lu | Ll | Lt | Lm | Lo | Nl) => true,
            '$' => true,
            '_' => true,
            _ => false,
        }
    }
}

impl CharacterAcceptor for IdentifierPart {
    fn accepts(ch: &char) -> bool {
        use MinorCategory::*;
        match ch {
            c if IdentifierStart::accepts(c) => true,
            c if matches!(c.get_minor_category(), Mn | Mc | Nd | Pc) => true,
            '\u{200C}' => true,
            '\u{200D}' => true,
            _ => false,
        }
    }
}

///
/// Check to see if the unicode escape code's value
/// is still valid in the context of an identifier part.
///
/// > A UnicodeEscapeSequence cannot be used to put a
/// > character into an IdentifierName that would otherwise be illegal.
///
/// &mdash; [see more](https://262.ecma-international.org/5.1/#sec-7.6).
///
pub fn check_unicode_escape<T: CharacterAcceptor>(
    backslash: v!('\\'),
    escape: UnicodeEscapeSequence,
    map: fn(v!('\\'), UnicodeEscapeSequence) -> T,
) -> LexResult<T> {
    let ch = escape.try_as_char();
    if !ch.map(|ch: char| T::accepts(&ch)).unwrap_or(false) {
        return LexResult::Errant(LexError::new(
            &backslash.span().combine([escape.span()]),
            format!(
                "Invalid escaped character in identifier: `{}` is not valid here.",
                ch.unwrap()
            ),
        ));
    }

    LexResult::Lexed(map(backslash, escape))
}

// ---

impl LexT for Identifier {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <IdentifierName as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, crate::lexing::LexError> {
        Ok(Self(<IdentifierName as LexT>::lex(input)?))
    }
}

impl LexT for IdentifierName {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <IdentifierStart as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, crate::lexing::LexError> {
        let start = LexT::lex(input)?;
        let after = Lex::lex(input).unwrap_as_result()?;
        Ok(Self(start, after))
    }
}

impl LexT for IdentifierStart {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <UnicodeLetter as LexT>::peek(input)
            || <v!('$') as LexT>::peek(input)
            || <v!('_') as LexT>::peek(input)
            || <v!('\\') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, crate::lexing::LexError> {
        // .unwrap_as_reult() ok since Self::peek() -> one variant exists.
        Lex::lex(input)
            .map(Self::Letter)
            .or(|| input.lex().map(Self::Dollar))
            .or(|| input.lex().map(Self::Underscore))
            .or(|| {
                input.lex().and(|backslash: v!('\\')| {
                    input
                        .lex()
                        .expected_msg(input, "Expected a unicode escape sequence `\\uXXXX` here.")
                        .and(|escape: UnicodeEscapeSequence| {
                            check_unicode_escape(backslash, escape, Self::Escape)
                        })
                })
            })
            .unwrap_as_result()
    }
}

impl LexT for IdentifierPart {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <IdentifierStart as LexT>::peek(input)
            || <UnicodeCombiningMark as LexT>::peek(input)
            || <UnicodeDigit as LexT>::peek(input)
            || <UnicodeConnectorPunctuation as LexT>::peek(input)
            || <v!('\u{200C}') as LexT>::peek(input)
            || <v!('\u{200D}') as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, crate::lexing::LexError> {
        // .unwrap_as_result() ok since Self::peek() -> exists one of the variants.
        Lex::lex(input)
            .and(|backslash: v!('\\')| {
                input
                    .lex()
                    .expected_msg(input, "Expected a unicode escape sequence `\\uXXXX` here.")
                    .and(|escape: UnicodeEscapeSequence| {
                        check_unicode_escape(backslash, escape, Self::Escape)
                    })
            })
            .or(|| input.lex().map(Self::Start))
            .or(|| input.lex().map(Self::CombiningMark))
            .or(|| input.lex().map(Self::Digit))
            .or(|| input.lex().map(Self::ConnectorPunctuation))
            .or(|| input.lex().map(Self::ZWNJ))
            .or(|| input.lex().map(Self::ZWJ))
            .unwrap_as_result()
    }
}

// ---

impl StringValue for Identifier {
    fn sv(&self) -> Vec<u16> {
        self.0.sv()
    }
}

impl StringValue for IdentifierName {
    fn sv(&self) -> Vec<u16> {
        let binding = IdentifierPart::Start(self.0.clone());
        let tmp: Vec<_> = once(&binding).chain(self.1.iter()).collect();
        collect_cv_into_utf16(tmp)
    }
}

// ---

impl CharacterValue for IdentifierStart {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        match self {
            IdentifierStart::Letter(letter) => letter.cv(buf),
            IdentifierStart::Dollar(_) => '$'.encode_utf16(buf),
            IdentifierStart::Underscore(_) => '_'.encode_utf16(buf),
            IdentifierStart::Escape(_, esc) => esc.cv(buf),
        }
    }
}

impl CharacterValue for IdentifierPart {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        match self {
            IdentifierPart::Escape(_, escape) => escape.cv(buf),
            IdentifierPart::Start(start) => start.cv(buf),
            IdentifierPart::CombiningMark(cm) => cm.cv(buf),
            IdentifierPart::Digit(digit) => digit.cv(buf),
            IdentifierPart::ConnectorPunctuation(cp) => cp.cv(buf),
            IdentifierPart::ZWNJ(_) => '\u{200C}'.encode_utf16(buf),
            IdentifierPart::ZWJ(_) => '\u{200D}'.encode_utf16(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{file::SourceFile, Source},
        lexing::LexResult,
    };

    use super::{Identifier, IdentifierPart, IdentifierStart};

    fn test_identifier(st: &'static str) -> LexResult<Identifier> {
        let source = SourceFile::dummy_file(st);
        let input = &mut source.stream();
        input.lex()
    }

    fn test_start(st: &'static str) -> LexResult<IdentifierStart> {
        let source = SourceFile::dummy_file(st);
        let input = &mut source.stream();
        input.lex()
    }

    fn test_middle(st: &'static str) -> LexResult<IdentifierPart> {
        let source = SourceFile::dummy_file(st);
        let input = &mut source.stream();
        input.lex()
    }

    #[test]
    fn start() {
        // Ll
        test_identifier("a").expect("Valid parse!");
        test_identifier("ʘ").expect("Valid parse!");
        test_identifier("ξ").expect("Valid parse!");
        test_identifier("я").expect("Valid parse!");
        test_identifier("ᴓ").expect("Valid parse!");
        test_identifier("ⱅ").expect("Valid parse!");
        test_identifier("ꮇ").expect("Valid parse!");
        test_identifier("ｖ").expect("Valid parse!");
        test_identifier("𐳭").expect("Valid parse!");
        test_identifier("𝐨").expect("Valid parse!");
        test_identifier("𝕘").expect("Valid parse!");
        test_identifier("𝛝").expect("Valid parse!");
        test_identifier("𞥃").expect("Valid parse!");

        // Lm
        test_identifier("ˑ").expect("Valid parse!");
        test_identifier("ˬ").expect("Valid parse!");
        test_identifier("ᶾ").expect("Valid parse!");
        test_identifier("〲").expect("Valid parse!");
        test_identifier("ꫝ").expect("Valid parse!");
        test_identifier("𖿡").expect("Valid parse!");

        // Lo
        test_identifier("ڧ").expect("Valid parse!");
        test_identifier("ݦ").expect("Valid parse!");
        test_identifier("ࠊ").expect("Valid parse!");
        test_identifier("ओ").expect("Valid parse!");
        test_identifier("ੴ").expect("Valid parse!");
        test_identifier("ࣅ").expect("Valid parse!");
        test_identifier("ഐ").expect("Valid parse!");
        test_identifier("ᆿ").expect("Valid parse!");
        test_identifier("ሥ").expect("Valid parse!");
        test_identifier("ᐚ").expect("Valid parse!");
        test_identifier("ᑺ").expect("Valid parse!");
        test_identifier("ᔐ").expect("Valid parse!");
        test_identifier("ᖲ").expect("Valid parse!");
        test_identifier("ᚙ").expect("Valid parse!");
        test_identifier("ᛦ").expect("Valid parse!");
        test_identifier("ᠩ").expect("Valid parse!");
        test_identifier("ᩐ").expect("Valid parse!");
        test_identifier("ᮯ").expect("Valid parse!");
        test_identifier("ⶦ").expect("Valid parse!");
        test_identifier("ツ").expect("Valid parse!");
        test_identifier("ㆈ").expect("Valid parse!");
        test_identifier("㐯").expect("Valid parse!");
        test_identifier("㔇").expect("Valid parse!");
        test_identifier("㠓").expect("Valid parse!");
        test_identifier("㨝").expect("Valid parse!");

        // Lt
        test_identifier("ᾫ").expect("Valid parse!");
        test_identifier("ᾝ").expect("Valid parse!");
        test_identifier("ǅ").expect("Valid parse!");

        // Lu
        test_identifier("A").expect("Valid parse!");
        test_identifier("Ǡ").expect("Valid parse!");
        test_identifier("Έ").expect("Valid parse!");
        test_identifier("Щ").expect("Valid parse!");
        test_identifier("Ꮿ").expect("Valid parse!");
        test_identifier("Å").expect("Valid parse!");
        test_identifier("ℜ").expect("Valid parse!");
        test_identifier("Ᵽ").expect("Valid parse!");
        test_identifier("Ｔ").expect("Valid parse!");
        test_identifier("𐲱").expect("Valid parse!");
        test_identifier("𝓨").expect("Valid parse!");
        test_identifier("𝗨").expect("Valid parse!");
        test_identifier("𝝫").expect("Valid parse!");
        test_identifier("𞤞").expect("Valid parse!");

        // Nl
        test_identifier("Ⅲ").expect("Valid parse!");
        test_identifier("ↈ").expect("Valid parse!");
        test_identifier("𐅰").expect("Valid parse!");
        test_identifier("𒐒").expect("Valid parse!");
        test_identifier("𒐪").expect("Valid parse!");
        test_identifier("𒑚").expect("Valid parse!");
        test_identifier("𒑮").expect("Valid parse!");

        test_identifier("_").expect("Valid parse!");
        test_identifier("$").expect("Valid parse!");
        test_identifier(r"\u0041").expect("Valid parse"); // `A`

        // Invalid Starting unicode escape code `@`
        test_identifier(r"\u0040").unwrap_err();

        // Middle-only characters
        // Mn
        assert!(!test_start("◌̣").is_lexed());
        assert!(!test_start("◌ַ").is_lexed());
        assert!(!test_start("◌ܶ").is_lexed());
        assert!(!test_start("◌ࣟ").is_lexed());
        assert!(!test_start("◌ై").is_lexed());
        assert!(!test_start("◌ླྀ").is_lexed());
        assert!(!test_start("◌ᬼ").is_lexed());
        assert!(!test_start("◌ⷻ").is_lexed());
        assert!(!test_start("◌ꦸ").is_lexed());
        assert!(!test_start("◌𝨰").is_lexed());
        assert!(!test_start("◌𝪩").is_lexed());
        assert!(!test_start("◌󠇬").is_lexed());

        // Mc
        assert!(!test_start("ா").is_lexed());
        assert!(!test_start("ௌ").is_lexed());
        assert!(!test_start("ෛ").is_lexed());
        assert!(!test_start("ြ").is_lexed());
        assert!(!test_start("ᬽ").is_lexed());
        assert!(!test_start("ꦾ").is_lexed());
        assert!(!test_start("𑍣").is_lexed());
        assert!(!test_start("𑲩").is_lexed());
        assert!(!test_start("𝅲").is_lexed());
        assert!(!test_start("𝅦").is_lexed());

        // Nd
        assert!(!test_start("1").is_lexed());
        assert!(!test_start("9").is_lexed());
        assert!(!test_start("٢").is_lexed());
        assert!(!test_start("٤").is_lexed());
        assert!(!test_start("৩").is_lexed());
        assert!(!test_start("੦").is_lexed());
        assert!(!test_start("௫").is_lexed());
        assert!(!test_start("൫").is_lexed());
        assert!(!test_start("໙").is_lexed());
        assert!(!test_start("႒").is_lexed());
        assert!(!test_start("᭑").is_lexed());
        assert!(!test_start("꧓").is_lexed());
        assert!(!test_start("꩘").is_lexed());
        assert!(!test_start("𝟯").is_lexed());
        assert!(!test_start("🯷").is_lexed());

        // Pc
        assert!(!test_start("‿").is_lexed());
        assert!(!test_start("⁀").is_lexed());
        assert!(!test_start("⁔").is_lexed());
        assert!(!test_start("︳").is_lexed());
        assert!(!test_start("︴").is_lexed());
        assert!(!test_start("﹍").is_lexed());
        assert!(!test_start("﹎").is_lexed());
        assert!(!test_start("﹏").is_lexed());
        assert!(!test_start("＿").is_lexed());
    }

    #[test]
    fn middle() {
        // Ll
        test_identifier("_a").expect("Valid parse!");
        test_identifier("_ʘ").expect("Valid parse!");
        test_identifier("_ξ").expect("Valid parse!");
        test_identifier("_я").expect("Valid parse!");
        test_identifier("_ᴓ").expect("Valid parse!");
        test_identifier("_ⱅ").expect("Valid parse!");
        test_identifier("_ꮇ").expect("Valid parse!");
        test_identifier("_ｖ").expect("Valid parse!");
        test_identifier("_𐳭").expect("Valid parse!");
        test_identifier("_𝐨").expect("Valid parse!");
        test_identifier("_𝕘").expect("Valid parse!");
        test_identifier("_𝛝").expect("Valid parse!");
        test_identifier("_𞥃").expect("Valid parse!");

        // Lm
        test_identifier("_ˑ").expect("Valid parse!");
        test_identifier("_ˬ").expect("Valid parse!");
        test_identifier("_ᶾ").expect("Valid parse!");
        test_identifier("_〲").expect("Valid parse!");
        test_identifier("_ꫝ").expect("Valid parse!");
        test_identifier("_𖿡").expect("Valid parse!");

        // Lo
        test_identifier("_ڧ").expect("Valid parse!");
        test_identifier("_ݦ").expect("Valid parse!");
        test_identifier("_ࠊ").expect("Valid parse!");
        test_identifier("_ओ").expect("Valid parse!");
        test_identifier("_ੴ").expect("Valid parse!");
        test_identifier("_ࣅ").expect("Valid parse!");
        test_identifier("_ഐ").expect("Valid parse!");
        test_identifier("_ᆿ").expect("Valid parse!");
        test_identifier("_ሥ").expect("Valid parse!");
        test_identifier("_ᐚ").expect("Valid parse!");
        test_identifier("_ᑺ").expect("Valid parse!");
        test_identifier("_ᔐ").expect("Valid parse!");
        test_identifier("_ᖲ").expect("Valid parse!");
        test_identifier("_ᚙ").expect("Valid parse!");
        test_identifier("_ᛦ").expect("Valid parse!");
        test_identifier("_ᠩ").expect("Valid parse!");
        test_identifier("_ᩐ").expect("Valid parse!");
        test_identifier("_ᮯ").expect("Valid parse!");
        test_identifier("_ⶦ").expect("Valid parse!");
        test_identifier("_ツ").expect("Valid parse!");
        test_identifier("_ㆈ").expect("Valid parse!");
        test_identifier("_㐯").expect("Valid parse!");
        test_identifier("_㔇").expect("Valid parse!");
        test_identifier("_㠓").expect("Valid parse!");
        test_identifier("_㨝").expect("Valid parse!");

        // Lt
        test_identifier("_ᾫ").expect("Valid parse!");
        test_identifier("_ᾝ").expect("Valid parse!");
        test_identifier("_ǅ").expect("Valid parse!");

        // Lu
        test_identifier("_A").expect("Valid parse!");
        test_identifier("_Ǡ").expect("Valid parse!");
        test_identifier("_Έ").expect("Valid parse!");
        test_identifier("_Щ").expect("Valid parse!");
        test_identifier("_Ꮿ").expect("Valid parse!");
        test_identifier("_Å").expect("Valid parse!");
        test_identifier("_ℜ").expect("Valid parse!");
        test_identifier("_Ᵽ").expect("Valid parse!");
        test_identifier("_Ｔ").expect("Valid parse!");
        test_identifier("_𐲱").expect("Valid parse!");
        test_identifier("_𝓨").expect("Valid parse!");
        test_identifier("_𝗨").expect("Valid parse!");
        test_identifier("_𝝫").expect("Valid parse!");
        test_identifier("_𞤞").expect("Valid parse!");

        // Nl
        test_identifier("_Ⅲ").expect("Valid parse!");
        test_identifier("_ↈ").expect("Valid parse!");
        test_identifier("_𐅰").expect("Valid parse!");
        test_identifier("_𒐒").expect("Valid parse!");
        test_identifier("_𒐪").expect("Valid parse!");
        test_identifier("_𒑚").expect("Valid parse!");
        test_identifier("_𒑮").expect("Valid parse!");

        // Mn
        test_identifier("_◌̣").expect("Valid parse!");
        test_identifier("_◌ַ").expect("Valid parse!");
        test_identifier("_◌ܶ").expect("Valid parse!");
        test_identifier("_◌ࣟ").expect("Valid parse!");
        test_identifier("_◌ై").expect("Valid parse!");
        test_identifier("_◌ླྀ").expect("Valid parse!");
        test_identifier("_◌ᬼ").expect("Valid parse!");
        test_identifier("_◌ⷻ").expect("Valid parse!");
        test_identifier("_◌ꦸ").expect("Valid parse!");
        test_identifier("_◌𝨰").expect("Valid parse!");
        test_identifier("_◌𝪩").expect("Valid parse!");
        test_identifier("_◌󠇬").expect("Valid parse!");

        // Mc
        test_identifier("_ா").expect("Valid parse!");
        test_identifier("_ௌ").expect("Valid parse!");
        test_identifier("_ෛ").expect("Valid parse!");
        test_identifier("_ြ").expect("Valid parse!");
        test_identifier("_ᬽ").expect("Valid parse!");
        test_identifier("_ꦾ").expect("Valid parse!");
        test_identifier("_𑍣").expect("Valid parse!");
        test_identifier("_𑲩").expect("Valid parse!");
        test_identifier("_𝅲").expect("Valid parse!");
        test_identifier("_𝅦").expect("Valid parse!");

        // Nd
        test_identifier("_1").expect("Valid parse!");
        test_identifier("_9").expect("Valid parse!");
        test_identifier("_٢").expect("Valid parse!");
        test_identifier("_٤").expect("Valid parse!");
        test_identifier("_৩").expect("Valid parse!");
        test_identifier("_੦").expect("Valid parse!");
        test_identifier("_௫").expect("Valid parse!");
        test_identifier("_൫").expect("Valid parse!");
        test_identifier("_໙").expect("Valid parse!");
        test_identifier("_႒").expect("Valid parse!");
        test_identifier("_᭑").expect("Valid parse!");
        test_identifier("_꧓").expect("Valid parse!");
        test_identifier("_꩘").expect("Valid parse!");
        test_identifier("_𝟯").expect("Valid parse!");
        test_identifier("_🯷").expect("Valid parse!");

        // Pc
        test_identifier("_‿").expect("Valid parse!");
        test_identifier("_⁀").expect("Valid parse!");
        test_identifier("_⁔").expect("Valid parse!");
        test_identifier("_︳").expect("Valid parse!");
        test_identifier("_︴").expect("Valid parse!");
        test_identifier("_﹍").expect("Valid parse!");
        test_identifier("_﹎").expect("Valid parse!");
        test_identifier("_﹏").expect("Valid parse!");
        test_identifier("_＿").expect("Valid parse!");

        test_identifier("__").expect("Valid parse!");
        test_identifier("_$").expect("Valid parse!");
        test_identifier(r"_\u0041").expect("Valid parse"); // `A`

        test_identifier(r"_\u0040").unwrap_err();
    }

    #[test]
    fn invalid() {
        // Sm
        assert!(!test_start(r"÷").is_lexed());
        assert!(!test_start(r"⅀").is_lexed());
        assert!(!test_start(r"∃").is_lexed());
        assert!(!test_start(r"∉").is_lexed());
        assert!(!test_start(r"∏").is_lexed());
        assert!(!test_start(r"∜").is_lexed());
        assert!(!test_start(r"⌠").is_lexed());
        assert!(!test_start(r"⌡").is_lexed());
        assert!(!test_start(r"⟜").is_lexed());
        assert!(!test_start(r"⨜").is_lexed());
        assert!(!test_start(r"⨷").is_lexed());
        assert!(!test_start(r"⪔").is_lexed());
        assert!(!test_start(r"𞻱").is_lexed());

        assert!(!test_middle(r"÷").is_lexed());
        assert!(!test_middle(r"⅀").is_lexed());
        assert!(!test_middle(r"∃").is_lexed());
        assert!(!test_middle(r"∉").is_lexed());
        assert!(!test_middle(r"∏").is_lexed());
        assert!(!test_middle(r"∜").is_lexed());
        assert!(!test_middle(r"⌠").is_lexed());
        assert!(!test_middle(r"⌡").is_lexed());
        assert!(!test_middle(r"⟜").is_lexed());
        assert!(!test_middle(r"⨜").is_lexed());
        assert!(!test_middle(r"⨷").is_lexed());
        assert!(!test_middle(r"⪔").is_lexed());
        assert!(!test_middle(r"𞻱").is_lexed());
    }

    #[test]
    fn escape_codes() {
        // Valid Start tests
        test_start(r"\u0061").expect("Valid parse!");
        test_start(r"\u0298").expect("Valid parse!");
        test_start(r"\u03be").expect("Valid parse!");
        test_start(r"\u044f").expect("Valid parse!");
        test_start(r"\u1d13").expect("Valid parse!");
        test_start(r"\u2c45").expect("Valid parse!");
        test_start(r"\uab87").expect("Valid parse!");
        test_start(r"\uff56").expect("Valid parse!");

        test_start(r"\u02d1").expect("Valid parse!");
        test_start(r"\u02ec").expect("Valid parse!");
        test_start(r"\u1dbe").expect("Valid parse!");
        test_start(r"\u3032").expect("Valid parse!");
        test_start(r"\uaadd").expect("Valid parse!");
        test_start(r"\u06a7").expect("Valid parse!");
        test_start(r"\u0766").expect("Valid parse!");
        test_start(r"\u080a").expect("Valid parse!");
        test_start(r"\u0913").expect("Valid parse!");
        test_start(r"\u0a74").expect("Valid parse!");
        test_start(r"\u08c5").expect("Valid parse!");
        test_start(r"\u0d10").expect("Valid parse!");
        test_start(r"\u11bf").expect("Valid parse!");
        test_start(r"\u1225").expect("Valid parse!");
        test_start(r"\u141a").expect("Valid parse!");
        test_start(r"\u147a").expect("Valid parse!");
        test_start(r"\u1510").expect("Valid parse!");
        test_start(r"\u15b2").expect("Valid parse!");
        test_start(r"\u1699").expect("Valid parse!");
        test_start(r"\u16e6").expect("Valid parse!");
        test_start(r"\u1829").expect("Valid parse!");
        test_start(r"\u1a50").expect("Valid parse!");
        test_start(r"\u1baf").expect("Valid parse!");
        test_start(r"\u2da6").expect("Valid parse!");
        test_start(r"\u30c4").expect("Valid parse!");
        test_start(r"\u3188").expect("Valid parse!");
        test_start(r"\u342f").expect("Valid parse!");
        test_start(r"\u3507").expect("Valid parse!");
        test_start(r"\u3813").expect("Valid parse!");
        test_start(r"\u3a1d").expect("Valid parse!");
        test_start(r"\u1fab").expect("Valid parse!");
        test_start(r"\u1f9d").expect("Valid parse!");
        test_start(r"\u01c5").expect("Valid parse!");
        test_start(r"\u0041").expect("Valid parse!");
        test_start(r"\u01e0").expect("Valid parse!");
        test_start(r"\u0388").expect("Valid parse!");
        test_start(r"\u0429").expect("Valid parse!");
        test_start(r"\u13ef").expect("Valid parse!");
        test_start(r"\u212b").expect("Valid parse!");
        test_start(r"\u211c").expect("Valid parse!");
        test_start(r"\u2c63").expect("Valid parse!");
        test_start(r"\uff34").expect("Valid parse!");
        test_start(r"\u2162").expect("Valid parse!");
        test_start(r"\u2188").expect("Valid parse!");
        test_start(r"\u005f").expect("Valid parse!");
        test_start(r"\u0024").expect("Valid parse!");

        // Invalid start character tests
        assert!(!test_start(r"\u0031").is_lexed());
        assert!(!test_start(r"\u0039").is_lexed());
        assert!(!test_start(r"\u0662").is_lexed());
        assert!(!test_start(r"\u0664").is_lexed());
        assert!(!test_start(r"\u09e9").is_lexed());
        assert!(!test_start(r"\u0a66").is_lexed());
        assert!(!test_start(r"\u0beb").is_lexed());
        assert!(!test_start(r"\u0d6b").is_lexed());
        assert!(!test_start(r"\u0ed9").is_lexed());
        assert!(!test_start(r"\u1092").is_lexed());
        assert!(!test_start(r"\u1b51").is_lexed());
        assert!(!test_start(r"\ua9d3").is_lexed());
        assert!(!test_start(r"\uaa58").is_lexed());
        assert!(!test_start(r"\u203f").is_lexed());
        assert!(!test_start(r"\u2040").is_lexed());
        assert!(!test_start(r"\u2054").is_lexed());
        assert!(!test_start(r"\ufe33").is_lexed());
        assert!(!test_start(r"\ufe34").is_lexed());
        assert!(!test_start(r"\ufe4d").is_lexed());
        assert!(!test_start(r"\ufe4e").is_lexed());
        assert!(!test_start(r"\ufe4f").is_lexed());
        assert!(!test_start(r"\uff3f").is_lexed());

        // Valid middle character tests
        assert!(test_middle(r"\u0031").is_lexed());
        assert!(test_middle(r"\u0039").is_lexed());
        assert!(test_middle(r"\u0662").is_lexed());
        assert!(test_middle(r"\u0664").is_lexed());
        assert!(test_middle(r"\u09e9").is_lexed());
        assert!(test_middle(r"\u0a66").is_lexed());
        assert!(test_middle(r"\u0beb").is_lexed());
        assert!(test_middle(r"\u0d6b").is_lexed());
        assert!(test_middle(r"\u0ed9").is_lexed());
        assert!(test_middle(r"\u1092").is_lexed());
        assert!(test_middle(r"\u1b51").is_lexed());
        assert!(test_middle(r"\ua9d3").is_lexed());
        assert!(test_middle(r"\uaa58").is_lexed());
        assert!(test_middle(r"\u203f").is_lexed());
        assert!(test_middle(r"\u2040").is_lexed());
        assert!(test_middle(r"\u2054").is_lexed());
        assert!(test_middle(r"\ufe33").is_lexed());
        assert!(test_middle(r"\ufe34").is_lexed());
        assert!(test_middle(r"\ufe4d").is_lexed());
        assert!(test_middle(r"\ufe4e").is_lexed());
        assert!(test_middle(r"\ufe4f").is_lexed());
        assert!(test_middle(r"\uff3f").is_lexed());
        assert!(test_middle(r"\u005f").is_lexed());
        assert!(test_middle(r"\u0024").is_lexed());
    }
}
