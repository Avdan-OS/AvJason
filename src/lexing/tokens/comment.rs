//!
//! Comments.
//!

use avjason_macros::{verbatim as v, ECMARef, Spanned};

use crate::{
    common::{Source, Span, Spanned},
    lexing::{Lex, LexError, LexT, SourceStream},
};

use super::line_terminator::LineTerminator;

///
/// ```js
/// // Comments
/// /* of either type. */
/// ```
///
#[ECMARef("Comment", "https://262.ecma-international.org/5.1/#sec-7.4")]
#[derive(Debug, Spanned)]
pub enum Comment {
    Single(SingleLineComment),
    Multi(MultiLineComment),
}

///
/// ```js
/// // Single-line comment.
/// ```
///
#[ECMARef("SingleLineComment", "https://262.ecma-international.org/5.1/#sec-7.4")]
#[derive(Debug, Spanned)]
pub struct SingleLineComment {
    span: Span,

    ///
    /// Span of the contents of this comment
    ///
    inner: Span,
}

///
/// ```js
/// /* Multi-line comment. */
/// ```
///
#[ECMARef("MultiLineComment", "https://262.ecma-international.org/5.1/#sec-7.4")]
#[derive(Debug, Spanned)]
pub struct MultiLineComment {
    span: Span,

    ///
    /// Span of the contents of this comment
    ///
    inner: Span,
}

impl Comment {
    pub fn inner(&self) -> Span {
        match self {
            Comment::Single(single) => single.inner,
            Comment::Multi(multi) => multi.inner,
        }
    }
}

impl LexT for Comment {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        <SingleLineComment as LexT>::peek(input) || <MultiLineComment as LexT>::peek(input)
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        // .into_result() ok since Self::peek() -> exists either variant.
        Lex::lex(input)
            .map(Self::Single)
            .or(|| Lex::lex(input).map(Self::Multi))
            .into_result()
    }
}

impl LexT for SingleLineComment {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming("//")
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let double_slash = <v!("//") as LexT>::lex(input)?;
        let contents = input
            .take_until(<LineTerminator as LexT>::peek)
            .map(|(span, _)| span)
            .unwrap_or(Span::empty());

        Ok(Self {
            span: double_slash.span().combine([contents]),
            inner: contents,
        })
    }
}

impl LexT for MultiLineComment {
    fn peek<S: Source>(input: &SourceStream<S>) -> bool {
        input.upcoming("/*")
    }

    fn lex<S: Source>(input: &mut SourceStream<S>) -> Result<Self, LexError> {
        let opening = <v!("/*") as LexT>::lex(input)?;
        let contents = input
            .take_until(<v!("*/") as LexT>::peek)
            .map(|(span, _)| span)
            .unwrap_or(Span::empty());

        Ok(Self {
            span: opening.span().combine([contents]),
            inner: contents,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        common::{file::SourceFile, Source},
        lexing::tokens::comment::Comment,
    };

    use super::{MultiLineComment, SingleLineComment};

    #[test]
    fn single_line_comment() {
        {
            let source = SourceFile::dummy_file("// An apple a day...");
            let input = &mut source.stream();
            let comment: SingleLineComment = input.lex().expect("Valid parse");

            assert_eq!(
                source.source_at(comment.inner),
                Some(" An apple a day...".to_string())
            );
        }
    }

    #[test]
    fn multi_line_comment() {
        {
            let source =
                SourceFile::dummy_file("/* An apple a day\n\r\u{2029}Keeps the doctor away! */");
            let input = &mut source.stream();
            let comment: MultiLineComment = input.lex().expect("Valid parse");

            assert_eq!(
                source.source_at(comment.inner),
                Some(" An apple a day\n\r\u{2029}Keeps the doctor away! ".to_string())
            );
        }
    }

    #[test]
    fn comments() {
        {
            let source =
                SourceFile::dummy_file("/* An apple a day\n\r\u{2029}Keeps the doctor away! */");
            let input = &mut source.stream();
            let comment: Comment = input.lex().expect("Valid parse");

            assert_eq!(
                source.source_at(comment.inner()),
                Some(" An apple a day\n\r\u{2029}Keeps the doctor away! ".to_string())
            );
        }
    }
}
