//!
//! ## Number
//!
//! Number tokens like integers, hex integers, and decimals,
//!

use std::ops::Add;

use avjason_macros::{verbatim as v, ECMARef, Spanned};

use crate::{
    common::{Source, Span},
    lexing::{AtLeast, Exactly, LexError, LexT, SourceStream},
};

///
/// The numerical value of a literal.
///
/// See the [ECMAScript spec](https://262.ecma-international.org/5.1/#sec-7.8.3).
///
pub trait MathematicalValue {
    type Value: Copy + Add<Self::Value, Output = Self::Value>;
    const BASE: usize;

    fn mv(&self) -> Self::Value;
}

#[ECMARef("DecimalDigit", "https://262.ecma-international.org/5.1/#sec-7.8.3")]
pub type DecimalDigit = v!('0'..='9');

#[ECMARef("HexDigit", "https://262.ecma-international.org/5.1/#sec-7.8.3")]
#[derive(Debug, Spanned)]
pub struct HexDigit {
    span: Span,
    raw: char,
}

// ---

impl LexT for HexDigit {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming(char::is_ascii_hexdigit)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .unwrap() ok since Self::peek() -> character exists.
        let (loc, raw) = input.take().unwrap();
        Ok(Self {
            span: Span::from(loc),
            raw,
        })
    }
}

// ---

impl MathematicalValue for DecimalDigit {
    type Value = u8;
    const BASE: usize = 10;

    fn mv(&self) -> Self::Value {
        match self.raw() {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            _ => unreachable!(),
        }
    }
}

impl MathematicalValue for HexDigit {
    type Value = u8;
    const BASE: usize = 16;

    fn mv(&self) -> Self::Value {
        match self.raw {
            '0' => 0x0,
            '1' => 0x1,
            '2' => 0x2,
            '3' => 0x3,
            '4' => 0x4,
            '5' => 0x5,
            '6' => 0x6,
            '7' => 0x7,
            '8' => 0x8,
            '9' => 0x9,
            'A' => 0xA,
            'B' => 0xB,
            'C' => 0xC,
            'D' => 0xD,
            'E' => 0xE,
            'F' => 0xF,
            'a' => 0xA,
            'b' => 0xB,
            'c' => 0xC,
            'd' => 0xD,
            'e' => 0xE,
            'f' => 0xF,
            _ => unreachable!(),
        }
    }
}

impl MathematicalValue for Exactly<2, HexDigit> {
    type Value = u8;
    const BASE: usize = 16;

    fn mv(&self) -> Self::Value {
        self[0].mv() * Self::BASE as u8 + self[1].mv()
    }
}

impl MathematicalValue for Exactly<4, HexDigit> {
    type Value = u16;
    const BASE: usize = 16;

    fn mv(&self) -> Self::Value {
        (self[0].mv() as u16) * (Self::BASE.pow(3) as u16)
            + (self[1].mv() as u16) * (Self::BASE.pow(2) as u16)
            + (self[2].mv() as u16) * (Self::BASE.pow(1) as u16)
            + self[3].mv() as u16
    }
}

impl<const N: usize> MathematicalValue for AtLeast<N, HexDigit> {
    type Value = u64;
    const BASE: usize = 16;

    fn mv(&self) -> Self::Value {
        self.iter()
            .map(MathematicalValue::mv)
            .map(|mv| mv as u64)
            .enumerate()
            .map(|(i, v)| v * (Self::BASE.pow(i as u32) as u64))
            .sum()
    }
}
