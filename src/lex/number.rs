//!
//! Number.
//!

use std::iter::once;
use std::ops::RangeBounds;

use avjason_macros::{Lex, Spanned};

use super::tokens::{Dot, LIdentifier, Lex, Minus, Plus};
use super::{IntoLexResult, LexResult};

use crate::lex::escape::is_hex_digit;
use crate::utils::{SourceIter, Span, Spanned, TryIntoSpan};
use crate::Token;

///
/// **JSON5Number**.
///
/// ---
/// See [the JSON5 specification](https://spec.json5.org/#prod-JSON5Number).
///
#[derive(Debug)]
pub struct Number(Option<Sign>, Numeric);

impl Spanned for Number {
    fn span(&self) -> Span {
        if let Some(ref sign) = self.0 {
            sign.span().combine([self.1.span()])
        } else {
            self.1.span()
        }
    }
}

impl Lex for Number {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let sign = if Sign::peek(input) {
            Sign::lex(input).into_lex_result().unwrap()
        } else {
            None
        };

        let Ok(Some(numeric)) = Numeric::lex(input).into_lex_result() else {
            return input.error().expected(Some(-1..0), "<NUMERIC LITERAL>");
        };

        Ok(Some(Self(sign, numeric)))
    }

    fn peek(input: &SourceIter) -> bool {
        Sign::peek(input) || Numeric::peek(input)
    }
}

#[derive(Debug, Spanned)]
#[Lex]
pub enum Sign {
    Positive(Plus),
    Negative(Minus),
}

trait Keyword: Sized {
    const TOKEN: &'static str;

    fn new<S: TryIntoSpan>(sp: impl RangeBounds<S>) -> Self;
}

impl<K: Keyword> Lex for K {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let start = input.next().unwrap().0;
        let end = start + Self::TOKEN.len();
        input.offset(Self::TOKEN.len() + 1);

        Ok(Some(Self::new(start..end)))
    }

    fn peek(input: &SourceIter) -> bool {
        input
            .ahead(..Self::TOKEN.len())
            .map(|ref s| s == Self::TOKEN)
            .unwrap_or(false)
    }
}

#[derive(Debug, Spanned)]
pub struct Infinity(Span);

impl Keyword for Infinity {
    const TOKEN: &'static str = "Infinity";

    fn new<S: TryIntoSpan>(sp: impl RangeBounds<S>) -> Self {
        Self(TryIntoSpan::try_into_span(sp).unwrap())
    }
}

#[derive(Debug, Spanned)]
pub struct NaN(Span);

impl Keyword for NaN {
    const TOKEN: &'static str = "NaN";

    fn new<S: TryIntoSpan>(sp: impl RangeBounds<S>) -> Self {
        Self(TryIntoSpan::try_into_span(sp).unwrap())
    }
}

///
/// **JSON5NumericLiteral**
///
/// ---
///
/// See [the JSON5 specification](https://spec.json5.org/#prod-JSON5NumericLiteral).
///
#[derive(Debug, Spanned)]
#[Lex]
pub enum Numeric {
    Infinity(Infinity),
    NaN(NaN),
    Lit(NumericLiteral),
}

///
/// ECMAScript **NumericLiteral**
///
/// ---
///
/// See the [ECMAScript specification](https://262.ecma-international.org/5.1/#sec-7.8.3).
///
#[derive(Debug, Spanned)]
pub enum NumericLiteral {
    Decimal(DecimalLiteral),
    Hex(HexIntegerLiteral),
}

impl NumericLiteral {
    ///
    /// From [ECMAScript standard](https://262.ecma-international.org/5.1/#sec-7.8.3)
    /// > NOTE: The source character immediately following a [NumericLiteral] must not be an *IdentifierStart* or *DecimalDigit*.
    ///
    fn after_check(input: &SourceIter) -> bool {
        !(LIdentifier::is_identifier_start(input)
            || input.peek().map(char::is_ascii_digit).unwrap_or(false))
    }
}

impl Lex for NumericLiteral {
    fn lex(mut input: &mut SourceIter) -> impl IntoLexResult<Self> {
        let res: LexResult<Self> = match input {
            ref mut input if HexIntegerLiteral::peek(input) => Ok(Some(Self::Hex(
                HexIntegerLiteral::lex(input)
                    .into_lex_result()
                    .unwrap()
                    .unwrap(),
            ))),
            ref mut input if DecimalLiteral::peek(input) => Ok(Some(Self::Decimal(
                DecimalLiteral::lex(input)
                    .into_lex_result()
                    .unwrap()
                    .unwrap(),
            ))),
            _ => Ok(None),
        };

        if !Self::after_check(input) {
            return input
                .error()
                .unexpected(Some(-1..0), "<DECIMAL DIGIT or IDENTIFIER START>");
        }

        res
    }

    fn peek(input: &SourceIter) -> bool {
        DecimalLiteral::peek(input) || HexIntegerLiteral::peek(input)
    }
}

#[derive(Debug, Spanned)]
#[Lex]
pub enum DecimalLiteral {
    IntegralDecimalMantissa(IntegralDecimalMantissa),
    DecimalMantissa(DecimalMantissa),
    Integer(Integer),
}

#[derive(Debug)]
pub struct IntegralDecimalMantissa(
    DecimalIntegerLiteral,
    Token![.],
    Option<DecimalDigits>,
    Option<ExponentPart>,
);

impl Spanned for IntegralDecimalMantissa {
    fn span(&self) -> Span {
        self.0.span().combine(
            self.2
                .as_ref()
                .map(|s| s.span())
                .into_iter()
                .chain(self.3.as_ref().map(|s| s.span())),
        )
    }
}

impl Lex for IntegralDecimalMantissa {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let i = DecimalIntegerLiteral::lex(input)
            .into_lex_result()
            .unwrap()
            .unwrap();

        let Ok(Some(d)) = Dot::lex(input).into_lex_result() else {
            return input.error().expected(Some(-1..1), ".");
        };

        let m = if DecimalDigits::peek(input) {
            DecimalDigits::lex(input).into_lex_result().unwrap()
        } else {
            None
        };

        let exp = if ExponentPart::peek(input) {
            ExponentPart::lex(input).into_lex_result().unwrap()
        } else {
            None
        };

        Ok(Some(Self(i, d, m, exp)))
    }

    fn peek(input: &SourceIter) -> bool {
        if DecimalIntegerLiteral::peek(input) {
            let mut fork = input.fork();
            let _ = DecimalIntegerLiteral::lex(&mut fork)
                .into_lex_result()
                .unwrap()
                .unwrap();

            return Dot::peek(&fork);
        }

        false
    }
}

#[derive(Debug)]
pub struct DecimalMantissa(Token![.], DecimalDigits, Option<ExponentPart>);

impl Spanned for DecimalMantissa {
    fn span(&self) -> Span {
        let s = self.0.span();

        if let Some(ref exp) = self.2 {
            s.combine([exp.span()])
        } else {
            s.combine([self.1.span()])
        }
    }
}

impl Lex for DecimalMantissa {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let d = Dot::lex(input).into_lex_result().unwrap().unwrap();

        let Ok(Some(ds)) = DecimalDigits::lex(input).into_lex_result() else {
            return input
                .error()
                .expected(Some(-1..0), "<DECIMAL DIGITS [0-9]>");
        };

        let exp = if ExponentPart::peek(input) {
            ExponentPart::lex(input).into_lex_result().unwrap()
        } else {
            None
        };

        Ok(Some(Self(d, ds, exp)))
    }

    fn peek(input: &SourceIter) -> bool {
        Dot::peek(input)
    }
}

#[derive(Debug)]
pub struct Integer(DecimalIntegerLiteral, Option<ExponentPart>);

impl Spanned for Integer {
    fn span(&self) -> Span {
        self.0.span().combine(self.1.as_ref().map(Spanned::span))
    }
}

impl Lex for Integer {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        let int = DecimalIntegerLiteral::lex(input)
            .into_lex_result()
            .unwrap()?;

        let exp = if ExponentPart::peek(input) {
            ExponentPart::lex(input).into_lex_result().unwrap()
        } else {
            None
        };

        Some(Self(int, exp))
    }

    fn peek(input: &SourceIter) -> bool {
        DecimalIntegerLiteral::peek(input)
    }
}

#[derive(Debug)]
pub enum DecimalIntegerLiteral {
    Zero(Zero),
    NonZero(NonZero, Option<DecimalDigits>),
}

impl Spanned for DecimalIntegerLiteral {
    fn span(&self) -> Span {
        match self {
            DecimalIntegerLiteral::Zero(z) => z.span(),
            DecimalIntegerLiteral::NonZero(a, b) => a.span().combine(b.as_ref().map(Spanned::span)),
        }
    }
}

impl Lex for DecimalIntegerLiteral {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if Zero::peek(input) {
            return Some(Self::Zero(Zero::lex(input).into_lex_result().unwrap()?));
        }
        if NonZero::peek(input) {
            let s = NonZero::lex(input).into_lex_result().unwrap()?;
            let after = if DecimalDigits::peek(input) {
                DecimalDigits::lex(input).into_lex_result().unwrap()
            } else {
                None
            };

            return Some(Self::NonZero(s, after));
        }

        None
    }

    fn peek(input: &SourceIter) -> bool {
        Zero::peek(input) || NonZero::peek(input)
    }
}

#[derive(Debug, Spanned)]
#[Lex('0')]
pub struct Zero(Span);

#[derive(Debug, Spanned)]
pub struct NonZero(Span);

impl Lex for NonZero {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        Some(Self(Span::single_char(input.next()?.0)))
    }

    fn peek(input: &SourceIter) -> bool {
        input
            .peek()
            .map(|d| matches!(d, '1'..='9'))
            .unwrap_or(false)
    }
}

#[derive(Debug, Spanned)]
pub struct DecimalDigits(Span);

impl Lex for DecimalDigits {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        let start = input.next()?.0;
        let mut end = start;

        loop {
            if !Self::peek(input) {
                break;
            }

            end = input.next().unwrap().0;
        }

        Some(Self(TryIntoSpan::try_into_span(start..=end).unwrap()))
    }

    fn peek(input: &SourceIter) -> bool {
        input.peek().map(|d| d.is_ascii_digit()).unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct ExponentPart(ExponentIdicator, SignedInteger);

impl Spanned for ExponentPart {
    fn span(&self) -> Span {
        self.0.span().combine([self.1.span()])
    }
}

impl Lex for ExponentPart {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return Ok(None);
        }

        let e_token = ExponentIdicator::lex(input)
            .into_lex_result()
            .unwrap()
            .unwrap();

        let Ok(Some(int)) = SignedInteger::lex(input).into_lex_result() else {
            return input
                .error()
                .expected(Some(-2..0), "Signed integer (e.g. +1, -2, 4)");
        };

        Ok(Some(Self(e_token, int)))
    }

    fn peek(input: &SourceIter) -> bool {
        ExponentIdicator::peek(input)
    }
}

#[derive(Debug, Spanned)]
#[Lex]
pub enum ExponentIdicator {
    Uppercase(E),
    Lowercase(e),
}

#[derive(Debug, Spanned)]
#[Lex('E')]
pub struct E(Span);

#[derive(Debug, Spanned)]
#[Lex('e')]
#[allow(non_camel_case_types)]
pub struct e(Span);

#[derive(Debug)]
pub enum SignedInteger {
    None(DecimalDigits),
    Positive(Token![+], DecimalDigits),
    Negative(Token![-], DecimalDigits),
}

impl Spanned for SignedInteger {
    fn span(&self) -> Span {
        match self {
            SignedInteger::None(d) => d.span(),
            SignedInteger::Positive(s, d) => s.span().combine([d.span()]),
            SignedInteger::Negative(s, d) => s.span().combine([d.span()]),
        }
    }
}

impl Lex for SignedInteger {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if Plus::peek(input) {
            return Some(Self::Positive(
                Plus::lex(input).into_lex_result().unwrap()?,
                DecimalDigits::lex(input).into_lex_result().unwrap()?,
            ));
        }

        if Minus::peek(input) {
            return Some(Self::Negative(
                Minus::lex(input).into_lex_result().unwrap()?,
                DecimalDigits::lex(input).into_lex_result().unwrap()?,
            ));
        }

        if DecimalDigits::peek(input) {
            return Some(Self::None(
                DecimalDigits::lex(input).into_lex_result().unwrap()?,
            ));
        }

        None
    }

    fn peek(input: &SourceIter) -> bool {
        <DecimalDigits as Lex>::peek(input)
            || <Token![+] as Lex>::peek(input)
            || <Token![-] as Lex>::peek(input)
    }
}

#[derive(Debug)]
pub struct HexIntegerLiteral(HexPrefix, HexDigit, Vec<HexDigit>);

#[derive(Debug, Spanned)]
#[Lex]
pub enum HexPrefix {
    Lowercase(LowercaseHexPrefix),
    Uppercase(UppercaseHexPrefix),
}

impl Spanned for HexIntegerLiteral {
    fn span(&self) -> Span {
        self.0
            .span()
            .combine(once(self.1.span()).chain(self.2.iter().map(Spanned::span)))
    }
}

impl Lex for HexIntegerLiteral {
    fn lex(mut input: &mut SourceIter) -> impl IntoLexResult<Self> {
        let p = match input {
            ref mut i if HexPrefix::peek(i) => {
                HexPrefix::lex(i).into_lex_result().unwrap().unwrap()
            }
            _ => return Ok(None),
        };

        let Ok(Some(d)) = HexDigit::lex(input).into_lex_result() else {
            return input.error().expected(Some(-1..0), "<HEX DIGIT>");
        };

        let mut ds = vec![];

        while let Some(ch) = input.peek() {
            if is_hex_digit(ch) {
                ds.push(HexDigit::lex(input).into_lex_result().unwrap().unwrap());
            } else {
                break;
            }
        }

        Ok(Some(Self(p, d, ds)))
    }

    fn peek(input: &SourceIter) -> bool {
        LowercaseHexPrefix::peek(input) || UppercaseHexPrefix::peek(input)
    }
}

#[derive(Debug, Spanned)]
pub struct LowercaseHexPrefix(Span);

impl Lex for LowercaseHexPrefix {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        let start = input.next().unwrap().0;
        input.offset(1);

        Some(Self(
            TryIntoSpan::try_into_span(start..=(start + 1)).unwrap(),
        ))
    }

    fn peek(input: &SourceIter) -> bool {
        input.ahead(0..2).map(|s| s == "0x").unwrap_or(false)
    }
}

#[derive(Debug, Spanned)]
pub struct UppercaseHexPrefix(Span);

impl Lex for UppercaseHexPrefix {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        let start = input.next().unwrap().0;
        input.offset(1);

        Some(Self(
            TryIntoSpan::try_into_span(start..=(start + 1)).unwrap(),
        ))
    }

    fn peek(input: &SourceIter) -> bool {
        input.ahead(0..2).map(|s| s == "0X").unwrap_or(false)
    }
}

#[derive(Debug, Spanned)]
pub struct HexDigit(Span);

impl Lex for HexDigit {
    fn lex(input: &mut SourceIter) -> impl IntoLexResult<Self> {
        if !Self::peek(input) {
            return None;
        }

        Some(Self(Span::single_char(input.next().unwrap().0)))
    }

    fn peek(input: &SourceIter) -> bool {
        matches!(input.peek(), Some(a) if is_hex_digit(a))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        lex::{
            number::{
                DecimalLiteral, DecimalMantissa, HexIntegerLiteral, Integer,
                IntegralDecimalMantissa, Number, Numeric, NumericLiteral,
            },
            tokens::Lex,
            IntoLexResult, LexResult,
        },
        utils::SourceFile,
    };

    use super::{ExponentIdicator, ExponentPart, HexPrefix, Sign, SignedInteger};

    fn test_lex<T: Lex>(s: impl ToString, src: &str) -> LexResult<T> {
        let src = SourceFile::dummy_file(format!("test.{}", s.to_string()), src);
        let iter = &mut src.iter();
        T::lex(iter).into_lex_result()
    }

    macro_rules! dot_man_exp {
        ($m: pat, $e: pat) => {
            Ok(Some(Number(
                None,
                Numeric::Lit(NumericLiteral::Decimal(DecimalLiteral::DecimalMantissa(
                    DecimalMantissa(_, $m, $e),
                ))),
            )))
        };
        ($s: pat, $m: pat, $e: pat) => {
            Ok(Some(Number(
                $s,
                Numeric::Lit(NumericLiteral::Decimal(DecimalLiteral::DecimalMantissa(
                    DecimalMantissa(_, $m, $e),
                ))),
            )))
        };
    }

    macro_rules! int_exp {
        ($m: pat, $e: pat) => {
            Ok(Some(Number(
                None,
                Numeric::Lit(NumericLiteral::Decimal(DecimalLiteral::Integer(Integer(
                    $m, $e,
                )))),
            )))
        };
        ($s: pat, $m: pat, $e: pat) => {
            Ok(Some(Number(
                $s,
                Numeric::Lit(NumericLiteral::Decimal(DecimalLiteral::Integer(Integer(
                    $m, $e,
                )))),
            )))
        };
    }

    macro_rules! hex_int {
        ($c: pat, $d: pat, $ds: pat) => {
            Ok(Some(Number(
                None,
                Numeric::Lit(NumericLiteral::Hex(HexIntegerLiteral($c, $d, $ds))),
            )))
        };
        ($s: pat, $c: pat, $d: pat, $ds: pat) => {
            Ok(Some(Number(
                $s,
                Numeric::Lit(NumericLiteral::Hex(HexIntegerLiteral($c, $d, $ds))),
            )))
        };
    }

    macro_rules! int_dot_man_exp {
        ($m: pat, $n: pat) => {
            Ok(Some(Number(
                None,
                Numeric::Lit(NumericLiteral::Decimal(
                    DecimalLiteral::IntegralDecimalMantissa(IntegralDecimalMantissa(_, _, $m, $n)),
                )),
            )))
        };
        ($s: pat, $m: pat, $n: pat) => {
            Ok(Some(Number(
                $s,
                Numeric::Lit(NumericLiteral::Decimal(
                    DecimalLiteral::IntegralDecimalMantissa(IntegralDecimalMantissa(_, _, $m, $n)),
                )),
            )))
        };
    }

    macro_rules! test_lex {
        ($s: expr, $p: pat) => {{
            let tmp = test_lex::<Number>(0, $s);
            if !matches!(tmp, $p) {
                panic!("{tmp:?}");
            }
        }};
    }

    #[test]
    fn no_sign() {
        assert!(!matches!(test_lex::<Number>(0, "02."), Ok(Some(_))));

        test_lex!("1.", int_dot_man_exp!(None, None));
        test_lex!("123.", int_dot_man_exp!(None, None));
        test_lex!("1.2", int_dot_man_exp!(Some(_), None));
        test_lex!("13.2", int_dot_man_exp!(Some(_), None));
        test_lex!("1.e-5", int_dot_man_exp!(None, Some(_)));
        test_lex!("134.2e-5", int_dot_man_exp!(Some(_), Some(_)));

        test_lex!(".1234", dot_man_exp!(_, None));
        test_lex!(".1234e-5", dot_man_exp!(_, Some(_)));

        test_lex!("1234", int_exp!(_, None));

        test_lex!(
            "467832674328438e2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "467832674328438E2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "467832674328438e+2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "467832674328438E+2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "467832674328438e-2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );
        test_lex!(
            "467832674328438E-2",
            int_exp!(
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );

        test_lex!("0x6432ABA3", hex_int!(HexPrefix::Lowercase(_), _, _));
        test_lex!("0x6432aba3", hex_int!(HexPrefix::Lowercase(_), _, _));
        test_lex!("0X6432ABA3", hex_int!(HexPrefix::Uppercase(_), _, _));
        test_lex!("0X6432ABA3", hex_int!(HexPrefix::Uppercase(_), _, _));
    }

    #[test]
    fn positive() {
        test_lex!("+1.", int_dot_man_exp!(Some(Sign::Positive(_)), None, None));
        test_lex!(
            "+123.",
            int_dot_man_exp!(Some(Sign::Positive(_)), None, None)
        );
        test_lex!(
            "+1.2",
            int_dot_man_exp!(Some(Sign::Positive(_)), Some(_), None)
        );
        test_lex!(
            "+13.2",
            int_dot_man_exp!(Some(Sign::Positive(_)), Some(_), None)
        );
        test_lex!(
            "+1.e-5",
            int_dot_man_exp!(Some(Sign::Positive(_)), None, Some(_))
        );
        test_lex!(
            "+134.2e-5",
            int_dot_man_exp!(Some(Sign::Positive(_)), Some(_), Some(_))
        );

        test_lex!("+.1234", dot_man_exp!(Some(Sign::Positive(_)), _, None));
        test_lex!(
            "+.1234e-5",
            dot_man_exp!(Some(Sign::Positive(_)), _, Some(_))
        );

        test_lex!("+1234", int_exp!(Some(Sign::Positive(_)), _, None));

        test_lex!(
            "+467832674328438e2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "+467832674328438E2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "+467832674328438e+2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "+467832674328438E+2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "+467832674328438e-2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );
        test_lex!(
            "+467832674328438E-2",
            int_exp!(
                Some(Sign::Positive(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );

        test_lex!(
            "+0x6432ABA3",
            hex_int!(Some(Sign::Positive(_)), HexPrefix::Lowercase(_), _, _)
        );
        test_lex!(
            "+0x6432aba3",
            hex_int!(Some(Sign::Positive(_)), HexPrefix::Lowercase(_), _, _)
        );
        test_lex!(
            "+0X6432ABA3",
            hex_int!(Some(Sign::Positive(_)), HexPrefix::Uppercase(_), _, _)
        );
        test_lex!(
            "+0X6432ABA3",
            hex_int!(Some(Sign::Positive(_)), HexPrefix::Uppercase(_), _, _)
        );
    }

    #[test]
    fn negative() {
        test_lex!("-1.", int_dot_man_exp!(Some(Sign::Negative(_)), None, None));
        test_lex!(
            "-123.",
            int_dot_man_exp!(Some(Sign::Negative(_)), None, None)
        );
        test_lex!(
            "-1.2",
            int_dot_man_exp!(Some(Sign::Negative(_)), Some(_), None)
        );
        test_lex!(
            "-13.2",
            int_dot_man_exp!(Some(Sign::Negative(_)), Some(_), None)
        );
        test_lex!(
            "-1.e-5",
            int_dot_man_exp!(Some(Sign::Negative(_)), None, Some(_))
        );
        test_lex!(
            "-134.2e-5",
            int_dot_man_exp!(Some(Sign::Negative(_)), Some(_), Some(_))
        );

        test_lex!("-.1234", dot_man_exp!(Some(Sign::Negative(_)), _, None));
        test_lex!(
            "-.1234e-5",
            dot_man_exp!(Some(Sign::Negative(_)), _, Some(_))
        );

        test_lex!("-1234", int_exp!(Some(Sign::Negative(_)), _, None));

        test_lex!(
            "-467832674328438e2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "-467832674328438E2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::None(_)
                ))
            )
        );
        test_lex!(
            "-467832674328438e+2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "-467832674328438E+2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Positive(_, _)
                ))
            )
        );
        test_lex!(
            "-467832674328438e-2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Lowercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );
        test_lex!(
            "-467832674328438E-2",
            int_exp!(
                Some(Sign::Negative(_)),
                _,
                Some(ExponentPart(
                    ExponentIdicator::Uppercase(_),
                    SignedInteger::Negative(_, _)
                ))
            )
        );

        test_lex!(
            "-0x6432ABA3",
            hex_int!(Some(Sign::Negative(_)), HexPrefix::Lowercase(_), _, _)
        );
        test_lex!(
            "-0x6432aba3",
            hex_int!(Some(Sign::Negative(_)), HexPrefix::Lowercase(_), _, _)
        );
        test_lex!(
            "-0X6432ABA3",
            hex_int!(Some(Sign::Negative(_)), HexPrefix::Uppercase(_), _, _)
        );
        test_lex!(
            "-0X6432ABA3",
            hex_int!(Some(Sign::Negative(_)), HexPrefix::Uppercase(_), _, _)
        );
    }

    #[test]
    fn idents() {
        assert!(matches!(
            test_lex::<Number>(0, "Infinity"),
            Ok(Some(Number(None, Numeric::Infinity(_))))
        ));
        assert!(matches!(
            test_lex::<Number>(0, "+Infinity"),
            Ok(Some(Number(Some(Sign::Positive(_)), Numeric::Infinity(_))))
        ));
        assert!(matches!(
            test_lex::<Number>(0, "-Infinity"),
            Ok(Some(Number(Some(Sign::Negative(_)), Numeric::Infinity(_))))
        ));

        assert!(test_lex::<Number>(0, "-Ifty").is_err());
        assert!(test_lex::<Number>(0, "+Inf").is_err());
        assert!(matches!(test_lex::<Number>(0, "Infinty"), Ok(None)));
        assert!(matches!(
            test_lex::<Number>(0, "Idfhfdsbhjfdsvbaysj"),
            Ok(None)
        ));

        assert!(matches!(
            test_lex::<Number>(0, "NaN"),
            Ok(Some(Number(None, Numeric::NaN(_))))
        ));
        assert!(matches!(
            test_lex::<Number>(0, "+NaN"),
            Ok(Some(Number(Some(Sign::Positive(_)), Numeric::NaN(_))))
        ));
        assert!(matches!(
            test_lex::<Number>(0, "-NaN"),
            Ok(Some(Number(Some(Sign::Negative(_)), Numeric::NaN(_))))
        ));

        assert!(test_lex::<Number>(0, "-NAN").is_err());
        assert!(matches!(test_lex::<Number>(0, "nAN"), Ok(None)));
        assert!(test_lex::<Number>(0, "+nAn").is_err());
        assert!(test_lex::<Number>(0, "-NAn").is_err());
    }
}
