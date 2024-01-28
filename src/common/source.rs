//!
//! Sources of source code.
//!

use std::ops::{Bound, Range, RangeBounds};

use super::{Loc, Span, Spanned};
use crate::lexing::utils::SourceStream;

#[cfg(test)]
pub use testing_only::DummySource;

///
/// Generic idea of source code: could be a file,
/// or a simple string.
///
/// This trait aims to abstract the gathering of the source
/// text and focus on the Source -> Lexing -> Syntax -> AST
/// pipeline.
///
pub trait Source {
    ///
    /// A friendly appropriate format to point
    /// to a location of a token.
    ///
    /// This could be line-column information, or simply an index.
    ///
    type Location<'a>
    where
        Self: 'a;

    ///
    /// Find the location of this span,
    /// and put it into a friendly appropriate format.
    ///
    fn locate(&self, span: Span) -> Option<Self::Location<'_>>;

    ///
    /// Returns the start and (exclusive) end index of this source.
    ///
    fn bounds(&self) -> Range<Loc>;

    ///
    /// Checks if a given [Span] is within bounds.
    ///
    fn in_bounds(&self, span: &Span) -> bool {
        self.bounds().end >= span.end
    }

    ///
    /// Returns the source code at a given [Span], if within bounds.
    ///
    fn source_at(&self, span: impl Spanned) -> Option<String>;

    ///
    /// Get the characters in this [Source].
    ///
    fn characters(&self) -> &[char];

    ///
    /// Crate a stream from this source.
    ///
    fn stream(&self) -> SourceStream<Self>
    where
        Self: Sized,
    {
        SourceStream::new(self)
    }
}

///
/// Utility conversion into a [Span], given
/// boundary information from the origin [Source].
///
pub trait ToSpan {
    fn to_span(self, source: &impl Source) -> Span;
}

impl<R: RangeBounds<usize>> ToSpan for R {
    fn to_span(self, source: &impl Source) -> Span {
        let Range {
            start: start_bound,
            end: end_bound,
        } = source.bounds();

        let start = match self.start_bound() {
            Bound::Included(included) => Loc(*included),
            Bound::Excluded(excluded) => Loc(*excluded + 1),
            Bound::Unbounded => start_bound,
        };

        let end = match self.end_bound() {
            Bound::Included(included) => Loc(included + 1),
            Bound::Excluded(excluded) => Loc(*excluded),
            Bound::Unbounded => end_bound,
        };

        Span { start, end }
    }
}

#[cfg(test)]
mod testing_only {
    use std::ops::Range;

    use crate::common::{Loc, Span, Spanned};

    use super::Source;

    ///
    /// [Source] implementation for testing purposes only!
    ///
    pub struct DummySource {
        text: String,
    }

    impl DummySource {
        pub fn new(text: impl ToString) -> Self {
            let text = text.to_string();
            Self { text }
        }
    }

    impl Source for DummySource {
        type Location<'a> = Range<usize>
            where Self: 'a;

        fn locate(&self, span: Span) -> Option<Self::Location<'_>> {
            if self.in_bounds(&span) {
                return Some(span.start.0..span.end.0);
            }

            None
        }

        fn bounds(&self) -> Range<Loc> {
            Loc(0)..Loc(self.text.len())
        }

        fn source_at(&self, span: impl Spanned) -> Option<String> {
            let span = span.span();
            if self.in_bounds(&span) {
                self.text.get(span.as_range()).map(ToString::to_string)
            } else {
                None
            }
        }

        fn characters(&self) -> &[char] {
            unimplemented!()
        }
    }
}
