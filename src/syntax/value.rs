//!
//! JSON5 Values.
//!

use avjason_macros::Spanned;

use crate::{
    lex::{
        number::Number,
        strings::LString,
        tokens::{False, LIdentifier, Null, Token, True},
    },
    Token, utils::{Spanned, Span, Loc},
};

use super::{Parse, ParseBuffer, ParserResult};

#[derive(Debug, Clone, Spanned)]
pub enum Boolean {
    True(Token![true]),
    False(Token![false]),
}

impl Boolean {
    fn peek(input: &ParseBuffer) -> bool {
        input
            .upcoming()
            .map(|token| True(token) || False(token))
            .unwrap_or_default()
    }
}

impl Parse for Boolean {
    fn parse(input: &mut super::ParseBuffer) -> super::ParserResult<Self> {
        if input.upcoming().map(True).unwrap_or_default() {
            return Ok(Self::True(Parse::parse(input)?));
        }

        if input.upcoming().map(False).unwrap_or_default() {
            return Ok(Self::False(Parse::parse(input)?));
        }

        input
            .error()
            .expected("boolean literal `true`, or `false`.")
    }
}

#[derive(Debug, Clone, Spanned)]
pub enum Value {
    Null(Token![null]),
    Boolean(Boolean),
    String(LString),
    Number(Number),
    Object(Object),
    Array(Array),
}

impl Parse for Value {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        let Some(token) = input.upcoming() else {
            return input.error().expected("Expected Value here!");
        };

        if Null(token) {
            return Ok(Self::Null(input.parse()?));
        }

        if Boolean::peek(input) {
            return Ok(Self::Boolean(input.parse()?));
        }

        if LString::peek_token(token) {
            return Ok(Self::String(input.parse()?));
        }

        if Number::peek_token(token) {
            return Ok(Self::Number(input.parse()?));
        }

        if Object::peek(input) {
            return Ok(Self::Object(input.parse()?));
        }

        if Array::peek(input) {
            return Ok(Self::Array(input.parse()?));
        }

        input
            .error()
            .expected("JSON value (`null`, number, string, boolean, object, or array")
    }
}

#[derive(Debug, Clone)]
pub struct Punctuated<El, Punct> {
    inner: Vec<El>,
    trailing: Option<Punct>,
}

impl<El, Punct> Spanned for Punctuated<El, Punct>
    where
        El: Spanned,
        Punct: Spanned
{
    fn span(&self) -> crate::utils::Span {
        if self.inner.is_empty() {
            return Span::single_char(Loc {index: 0});
        }

        let s = self.inner[0].span();
        let e = if let Some(ref t) = self.trailing {
            t.span()
        } else if self.inner.len() > 1 {
            self.inner.last().unwrap().span()
        } else {
            s
        };

        s.combine([e])
    }
}

impl<El, Punct> Punctuated<El, Punct>
where
    El: Parse,
    Punct: Parse,
{
    fn parse_until(
        input: &mut ParseBuffer,
        pred: impl Fn(&ParseBuffer) -> bool,
    ) -> ParserResult<Self> {
        let mut inner: Vec<El> = vec![];
        let mut trailing: Option<Punct> = None;

        loop {
            if pred(input) {
                break;
            }

            inner.push(El::parse(input)?);
            trailing = None;

            if pred(input) {
                break;
            }

            trailing = Some(Punct::parse(input)?);
        }

        Ok(Self { inner, trailing })
    }
}

#[derive(Debug, Clone)]
pub struct Object {
    open: Token!['{'],
    members: Punctuated<Member, Token![,]>,
    close: Token!['}'],
}

impl Spanned for Object {
    fn span(&self) -> Span {
        let s = self.open.span();
        let e = self.close.span();

        s.combine([e])
    }
}

impl Object {
    pub(crate) fn peek(input: &ParseBuffer) -> bool {
        input.peek(Token!['{'])
    }
}

impl Parse for Object {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        let open = Parse::parse(input)?;
        let members = Punctuated::parse_until(input, |input| input.peek(Token!['}']))?;
        let close = Parse::parse(input)?;
        Ok(Self {
            open,
            members,
            close,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    name: MemberName,
    colon: Token![:],
    value: Value,
}

impl Spanned for Member {
    fn span(&self) -> Span {
        let s = self.name.span();
        let e = self.value.span();

        s.combine([e])
    }
}

impl Parse for Member {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        Ok(Self {
            name: input.parse()?,
            colon: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[derive(Debug, Clone, Spanned)]
pub enum MemberName {
    Identifier(LIdentifier),
    String(LString),
}

impl Parse for LString {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        match input {
            i if i.upcoming().map(LString::peek_token).unwrap_or_default() => {
                match i.next().unwrap() {
                    Token::String(l) => Ok(l),
                    _ => unreachable!(),
                }
            }
            _ => input.error().expected("string literal"),
        }
    }
}

impl Parse for Number {
    fn parse(input: &mut crate::syntax::ParseBuffer) -> crate::syntax::ParserResult<Self> {
        let Some(Token::Number(token)) = input.next() else {
            return input.error().expected("number literal");
        };

        Ok(token)
    }
}

impl Parse for MemberName {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        if input
            .upcoming()
            .map(LIdentifier::peek_token)
            .unwrap_or_default()
        {
            return Ok(Self::Identifier(Parse::parse(input)?));
        }

        if input
            .upcoming()
            .map(LString::peek_token)
            .unwrap_or_default()
        {
            return Ok(Self::String(Parse::parse(input)?));
        }

        input
            .error()
            .expected("either string literal, or identifier")
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    open: Token!['['],
    elements: Punctuated<Value, Token![,]>,
    close: Token![']'],
}

impl Spanned for Array {
    fn span(&self) -> Span {
        let s = self.open.span();
        let e = self.close.span();

        s.combine([e])
    }
}

impl Array {
    pub(crate) fn peek(input: &ParseBuffer) -> bool {
        input.peek(Token!['['])
    }
}

impl Parse for Array {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self> {
        let open = input.parse()?;
        let elements = Punctuated::parse_until(input, |input| input.peek(Token![']']))?;
        let close = input.parse()?;

        Ok(Self {
            open,
            elements,
            close,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::SourceFile;

    #[test]
    fn parse_value() {
        let src = SourceFile::dummy_file("test.0", r#"{"fruits": [{name: "apple", qty: 2}], }"#);
        let v = src.parse();
        println!("{v:#?}");
    }
}