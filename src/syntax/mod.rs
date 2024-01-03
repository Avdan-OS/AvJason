//!
//! Syntax Grammar.
//!

pub mod utils;
pub mod value;

use crate::{
    lex::tokens::Token,
    utils::{Loc, SourceFile, Span},
};

use self::utils::Peek;

#[derive(Debug)]
pub struct ParseError {
    near: String,
    message: String,
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error occured during parsing:\t{}\n\tAt {}",
            self.message, self.near
        )
    }
}

pub type ParserResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseBuffer<'a> {
    file: &'a SourceFile,
    inner: Vec<Token>,
    index: usize,
}

impl<'a> ParseBuffer<'a> {
    pub(crate) fn new(file: &'a SourceFile, inner: Vec<Token>) -> Self {
        Self {
            file,
            inner,
            index: 0,
        }
    }

    pub(crate) fn fork(&self) -> Self {
        self.clone()
    }

    pub(crate) fn source_text(&self, span: Span) -> String {
        self.file.source_at_span(span).unwrap()
    }

    pub(crate) fn upcoming(&self) -> Option<&Token> {
        self.inner.get(self.index)
    }

    pub(crate) fn peek<P>(&self, p: P) -> bool
    where
        P: Peek,
    {
        self.upcoming().map(|t| p.peek(t)).unwrap_or(false)
    }

    pub(crate) fn error(&'a self) -> ParseErrorHelper<'a> {
        ParseErrorHelper(self)
    }

    pub(crate) fn index_display(&self, loc: impl IntoLoc) -> String {
        self.file.file_line_column(&loc.into_loc()).unwrap()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.index
    }

    pub(crate) fn parse<P: Parse>(&mut self) -> ParserResult<P> {
        P::parse(self)
    }

    pub(crate) fn advance_to(&mut self, other: Self) {
        self.index = other.index;
    }
}

pub(crate) trait IntoLoc {
    fn into_loc(self) -> Loc;
}

impl IntoLoc for Loc {
    fn into_loc(self) -> Loc {
        self
    }
}

impl<I: Into<usize>> IntoLoc for I {
    fn into_loc(self) -> Loc {
        Loc { index: self.into() }
    }
}

pub struct ParseErrorHelper<'a>(&'a ParseBuffer<'a>);

impl<'a> ParseErrorHelper<'a> {
    pub(crate) fn unexpected<T>(self, message: impl ToString) -> ParserResult<T> {
        Err(ParseError {
            near: self.0.index_display(self.0.cursor() - 1),
            message: format!("Unexpected {}", message.to_string()),
        })
    }

    pub(crate) fn expected<T>(self, message: impl ToString) -> ParserResult<T> {
        Err(ParseError {
            near: self.0.index_display(self.0.cursor() - 1),
            message: format!("Expected {}", message.to_string()),
        })
    }
}

impl<'a> Iterator for ParseBuffer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.get(self.index);
        self.index += 1;
        item.cloned()
    }
}

pub trait Parse: Sized {
    fn parse(input: &mut ParseBuffer) -> ParserResult<Self>;
}
