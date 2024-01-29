//!
//! Allows for capturing different unicode groups.
//!
//! This is a stupid hack because at the moment,
//! ConstParamTy is not auto-implemented,
//! so [finl_unicode::categories::MinorCategory] doesn't implement it;
//! meaning we must do it nastily.

use std::marker::ConstParamTy;

use avjason_macros::Spanned;
use finl_unicode::categories::CharacterCategories;

use crate::{
    common::{Source, Span},
    lexing::tokens::string::CharacterValue,
};

use super::{LexError, LexT, SourceStream};

///
/// Looks for a character in any of the
/// unicode major categories supplied as a const parameter.
///
/// ***
///
/// **Do not use me directly, use [crate::unicode] instead!**
///
#[derive(Debug, Spanned, Clone)]
pub struct MatchMajorCategory<const C: &'static [MajorCategory]> {
    span: Span,
    raw: char,
}

///
/// Looks for a character in any of the
/// unicode minor categories supplied as a const parameter.
///
/// ***
///
/// **Do not use me directly, use [crate::unicode] instead!**
///
#[derive(Debug, Spanned, Clone)]
pub struct MatchMinorCategory<const C: &'static [MinorCategory]> {
    span: Span,
    raw: char,
}

// ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MajorCategory {
    /// Letter
    L,
    /// Mark
    M,
    /// Number
    N,
    /// Punctuation
    P,
    /// Symbol
    S,
    /// Separator
    Z,
    /// Other character
    C,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinorCategory {
    /// Uppercase letter
    Lu,
    /// Lowercase letter
    Ll,
    /// Titlecase letter
    Lt,
    /// Modifier letter
    Lm,
    /// Other letter
    Lo,
    /// Non-spacing mark
    Mn,
    /// Spacing mark
    Mc,
    /// Enclosing mark
    Me,
    /// Decimal number
    Nd,
    /// Letterlike number
    Nl,
    /// Other number
    No,
    /// Connector punctuation
    Pc,
    /// Dash punctuation
    Pd,
    /// Opening punctuation
    Ps,
    /// Closing punctuation
    Pe,
    /// Initial punctuation
    Pi,
    /// Final punctuation
    Pf,
    /// Other punctuation
    Po,
    /// Math symbol
    Sm,
    /// Modifier symbol
    Sk,
    /// Currency symbol
    Sc,
    /// Other symbol
    So,
    /// Space separator
    Zs,
    /// Line separator
    Zl,
    /// Paragraph separator
    Zp,
    /// Control character
    Cc,
    /// Format character
    Cf,
    /// Private use character
    Co,
    /// Unassigned character
    Cn,
}

// ---

impl<const C: &'static [MajorCategory]> LexT for MatchMajorCategory<C> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input
            .peek()
            .map(|ch| {
                let cat = ch.get_major_category();
                C.iter().any(|major| &cat == major)
            })
            .unwrap_or(false)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .unwrap() ok since Self::peek() -> next character exists.
        let (loc, raw) = input.take().unwrap();
        Ok(Self {
            span: Span::from(loc),
            raw,
        })
    }
}

impl<const C: &'static [MinorCategory]> LexT for MatchMinorCategory<C> {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input
            .peek()
            .map(|ch| {
                let cat = ch.get_minor_category();
                C.iter().any(|major| &cat == major)
            })
            .unwrap_or(false)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .unwrap() ok since Self::peek() -> next character exists.
        let (loc, raw) = input.take().unwrap();
        Ok(Self {
            span: Span::from(loc),
            raw,
        })
    }
}

// ---

impl<const C: &'static [MajorCategory]> CharacterValue for MatchMajorCategory<C> {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        self.raw.encode_utf16(buf)
    }
}

impl<const C: &'static [MinorCategory]> CharacterValue for MatchMinorCategory<C> {
    fn cv<'a, 'b: 'a>(&'a self, buf: &'b mut [u16; 2]) -> &'b [u16] {
        self.raw.encode_utf16(buf)
    }
}

// ---

impl ConstParamTy for MajorCategory {}
impl ConstParamTy for MinorCategory {}

impl From<MajorCategory> for finl_unicode::categories::MajorCategory {
    fn from(value: MajorCategory) -> Self {
        match value {
            MajorCategory::L => Self::L,
            MajorCategory::M => Self::M,
            MajorCategory::N => Self::N,
            MajorCategory::P => Self::P,
            MajorCategory::S => Self::S,
            MajorCategory::Z => Self::Z,
            MajorCategory::C => Self::C,
        }
    }
}

impl From<MinorCategory> for finl_unicode::categories::MinorCategory {
    fn from(value: MinorCategory) -> Self {
        match value {
            MinorCategory::Lu => Self::Lu,
            MinorCategory::Ll => Self::Ll,
            MinorCategory::Lt => Self::Lt,
            MinorCategory::Lm => Self::Lm,
            MinorCategory::Lo => Self::Lo,
            MinorCategory::Mn => Self::Mn,
            MinorCategory::Mc => Self::Mc,
            MinorCategory::Me => Self::Me,
            MinorCategory::Nd => Self::Nd,
            MinorCategory::Nl => Self::Nl,
            MinorCategory::No => Self::No,
            MinorCategory::Pc => Self::Pc,
            MinorCategory::Pd => Self::Pd,
            MinorCategory::Ps => Self::Ps,
            MinorCategory::Pe => Self::Pe,
            MinorCategory::Pi => Self::Pi,
            MinorCategory::Pf => Self::Pf,
            MinorCategory::Po => Self::Po,
            MinorCategory::Sm => Self::Sm,
            MinorCategory::Sk => Self::Sk,
            MinorCategory::Sc => Self::Sc,
            MinorCategory::So => Self::So,
            MinorCategory::Zs => Self::Zs,
            MinorCategory::Zl => Self::Zl,
            MinorCategory::Zp => Self::Zp,
            MinorCategory::Cc => Self::Cc,
            MinorCategory::Cf => Self::Cf,
            MinorCategory::Co => Self::Co,
            MinorCategory::Cn => Self::Cn,
        }
    }
}

impl PartialEq<MajorCategory> for finl_unicode::categories::MajorCategory {
    fn eq(&self, other: &MajorCategory) -> bool {
        Self::from(*other).eq(self)
    }
}

impl PartialEq<MinorCategory> for finl_unicode::categories::MinorCategory {
    fn eq(&self, other: &MinorCategory) -> bool {
        Self::from(*other).eq(self)
    }
}

#[cfg(test)]
mod tests {
    use avjason_macros::unicode;

    use crate::{
        common::{file::SourceFile, Source},
        lexing::Many,
    };

    type Letter = unicode!(Lu | Ll);

    #[test]
    fn test_lex() {
        let source = SourceFile::dummy_file("Apples");
        let input = &mut source.stream();
        let _: Many<Letter> = input.lex().expect("Valid parse");
    }
}
