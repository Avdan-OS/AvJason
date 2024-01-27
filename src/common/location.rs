//!
//! Things that help trace errors and tokens: [Span] and [Loc].
//! 

use std::ops::{Add, Bound, Range, RangeBounds};

///
/// Represents the index of a character in source code.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc(pub(crate) usize);

impl From<usize> for Loc {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Loc> for usize {
    fn from(value: Loc) -> Self {
        value.0
    }
}

impl<A> Add<A> for Loc
where
    usize: Add<A, Output = usize>,
{
    type Output = Loc;

    fn add(self, rhs: A) -> Self::Output {
        Self(self.0 + rhs)
    }
}

///
/// Represents the location of a token in source code.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    ///
    /// Start index: inclusive lower bound.
    ///
    pub(crate) start: Loc,

    ///
    /// End index: exclusive upper bound.
    ///
    pub(crate) end: Loc,
}

impl Span {
    ///
    /// Returns Some(subspan), given the relative indexes from the start of this span,
    /// returning None if the end index is out of this span's bounds.
    ///
    pub fn subspan(&self, indexes: impl RangeBounds<usize>) -> Option<Span> {
        let start = match indexes.start_bound() {
            Bound::Included(included) => self.start + included,
            Bound::Excluded(excluded) => self.start + (excluded + 1),
            Bound::Unbounded => self.start,
        };

        let end = match indexes.end_bound() {
            Bound::Included(included) => self.start + (included + 1),
            Bound::Excluded(excluded) => self.start + excluded,
            Bound::Unbounded => self.end,
        };

        if end > self.end {
            // Not a subspan, since we overflow the end.
            return None;
        }

        Some(Self { start, end })
    }

    ///
    /// Use this [Span] as a start, taking the range between this span's start,
    /// and the end oi the last of the passed in iterator (including itself).
    ///
    pub fn combine(self, others: impl IntoIterator<Item = Span>) -> Span {
        let Self { start, end } = self;

        // Take the end bound of the last Span,
        // if others is not empty, and use that instead.
        let last = others.into_iter().last();
        if let Some(Self { end, .. }) = last {
            return Self { start, end };
        }

        Self { start, end }
    }

    ///
    /// Return the start and end bounds as a Rust [Range]
    ///
    pub fn as_range(&self) -> Range<usize> {
        self.start.0..self.end.0
    }
}

///
/// Utility trait for handling multiple spans.
///
pub trait SpanIter: Sized + IntoIterator<Item = Span> {
    ///
    /// Combine all of this iterator's spans,
    /// resulting in a [Span] encompassing all
    /// passed in [Span]s (assuming this iter is in ascending order).
    ///
    fn combine(self) -> Option<Span>;
}

impl<Iter: IntoIterator<Item = Span>> SpanIter for Iter {
    fn combine(self) -> Option<Span> {
        let mut iter = self.into_iter();
        iter.next().map(|s| s.combine(iter))
    }
}

impl RangeBounds<usize> for Span {
    fn start_bound(&self) -> Bound<&usize> {
        Bound::Included(&self.start.0)
    }

    fn end_bound(&self) -> Bound<&usize> {
        Bound::Excluded(&self.end.0)
    }
}

impl RangeBounds<Loc> for Span {
    fn start_bound(&self) -> Bound<&Loc> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&Loc> {
        Bound::Excluded(&self.end)
    }
}

///
/// Returns the span attached to this
/// object.
/// 
pub trait Spanned {
    ///
    /// Returns the span attached to this
    /// object.
    /// 
    fn span(&self) -> Span;
}

#[cfg(test)]
mod tests {
    use crate::common::source::{DummySource, Source, ToSpan};

    #[test]
    fn subspan() {
        let source = DummySource::new("testthing.");
        let span = (0..9).to_span(&source);

        // Valid
        assert_eq!(span.subspan(1..).map(|s| s.as_range()), Some(1..9));
        assert_eq!(span.subspan(1..2).map(|s| s.as_range()), Some(1..2));
        assert_eq!(span.subspan(..5).map(|s| s.as_range()), Some(0..5));
        assert_eq!(span.subspan(..).map(|s| s.as_range()), Some(0..9));

        // Invalid
        assert_eq!(span.subspan(..17).map(|s| s.as_range()), None);
        assert_eq!(span.subspan(144..1343).map(|s| s.as_range()), None);
    }

    #[test]
    fn source_at() {
        let source = DummySource::new("testthing.");
        let span = (0..9).to_span(&source);

        assert_eq!(
            span.subspan(..4).and_then(|s| source.source_at(s)),
            Some("test".to_string())
        );

        assert_eq!(
            span.subspan(4..).and_then(|s| source.source_at(s)),
            Some("thing".to_string())
        );

        assert_eq!(
            span.subspan(..4).and_then(|s| source.source_at(s)),
            Some("test".to_string())
        );

        assert_eq!(span.subspan(49..).and_then(|s| source.source_at(s)), None);
    }
}
