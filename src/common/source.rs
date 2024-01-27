//!
//! Sources of source code.
//!

use std::ops::{Bound, Range, RangeBounds};

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
    type Location;

    ///
    /// Find the location of this span,
    /// and put it into a friendly appropriate format.
    ///
    fn locate(&self, span: Span) -> Option<Self::Location>;

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
    fn source_at(&self, span: Span) -> Option<String>;
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
pub use testing_only::DummySource;

use super::{Loc, Span};

#[cfg(test)]
mod testing_only {
    use std::ops::Range;

    use crate::common::{Loc, Span};

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
        type Location = Range<usize>;

        fn locate(&self, span: Span) -> Option<Self::Location> {
            if self.in_bounds(&span) {
                Some(span.start.0..span.end.0)
            } else {
                None
            }
        }

        fn bounds(&self) -> Range<Loc> {
            Loc(0)..Loc(self.text.len())
        }

        fn source_at(&self, span: Span) -> Option<String> {
            if self.in_bounds(&span) {
                self.text.get(span.as_range()).map(ToString::to_string)
            } else {
                None
            }
        }
    }
}
