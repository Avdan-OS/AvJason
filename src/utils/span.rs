//!
//! Helpers for finding the locations of things.
//!

use std::ops::{Add, RangeBounds};

///
/// Represents a character's location in source code.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loc {
    pub(crate) index: usize,
}

impl<Rhs> Add<Rhs> for Loc
where
    Rhs: Copy,
    usize: Add<Rhs, Output = usize>,
{
    type Output = Loc;

    fn add(self, rhs: Rhs) -> Self::Output {
        Self {
            index: self.index + rhs,
        }
    }
}

///
/// Represents a token's position in the code.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    ///
    /// Lower bound.
    ///
    pub(crate) start: Loc,

    ///
    /// Exclusive upper bound.
    ///
    pub(crate) end: Loc,
}

impl Span {
    ///
    /// Returns the length of this span in characters.
    ///
    pub fn len(&self) -> usize {
        self.end.index - self.start.index
    }

    ///
    /// Returns whether this [Span] contains nothing.
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    ///
    /// Allows you to find a smaller [Span] within this span.
    ///
    /// ```ignore
    /// // Find location of the word "pumpkin"
    /// let pumpkin: Span = find_word("pumpkin");
    ///
    /// // Gets the Span corresponding to the "pump" substring of "pumpkin".
    /// let pump = pumpkin.subspan(..4);
    /// ```
    ///
    pub fn subspan<R>(&self, bounds: R) -> Option<Span>
    where
        R: RangeBounds<usize>,
    {
        let start = match bounds.start_bound() {
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(_) => unimplemented!("Excluded lower bounds: impossible."),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match bounds.end_bound() {
            std::ops::Bound::Included(i) => *i + 1,
            std::ops::Bound::Excluded(i) => *i,
            std::ops::Bound::Unbounded => self.len(),
        };

        if start > 0 || start > end {
            return None;
        }

        if end > self.len() {
            return None;
        }

        Some(Span {
            start: Loc { index: start },
            end: Loc { index: end },
        })
    }

    pub fn single_char(loc: Loc) -> Span {
        Self {
            start: loc,
            end: loc + 1,
        }
    }

    pub fn combine(self, iter: impl IntoIterator<Item = Self>) -> Self {
        let start = self;
        let last = iter.into_iter().last();

        if let Some(end) = last {
            Self {
                start: start.start,
                end: end.end,
            }
        } else {
            start
        }
    }
}

///
/// Convenience converter trait.
///
/// ### Examples
/// ```
/// use avjason::utils::TryIntoSpan;
///
/// fn test<S: TryIntoSpan>(span: S) {
///     let span = span.into_span();
///     // TODO: Do stuff with `s`...
/// }
/// ```
///
pub trait TryIntoSpan {
    fn try_into_span(range: impl RangeBounds<Self>) -> Option<Span>;
}

impl TryIntoSpan for Loc {
    fn try_into_span(range: impl RangeBounds<Self>) -> Option<Span> {
        let start = range.start_bound();
        let end = range.end_bound();

        let start = match start {
            std::ops::Bound::Included(Loc { index: i }) => *i,
            std::ops::Bound::Excluded(_) => unimplemented!("Not possible: excluded lower bound."),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match end {
            std::ops::Bound::Included(Loc { index: i }) => *i + 1,
            std::ops::Bound::Excluded(Loc { index: i }) => *i,
            std::ops::Bound::Unbounded => return None,
        };

        Some(Span {
            start: Loc { index: start },
            end: Loc { index: end },
        })
    }
}

impl TryIntoSpan for usize {
    fn try_into_span(range: impl RangeBounds<Self>) -> Option<Span> {
        let start = range.start_bound();
        let end = range.end_bound();

        let start = match start {
            std::ops::Bound::Included(i) => *i,
            std::ops::Bound::Excluded(_) => unimplemented!("Not possible: excluded lower bound."),
            std::ops::Bound::Unbounded => 0,
        };

        let end = match end {
            std::ops::Bound::Included(i) => *i + 1,
            std::ops::Bound::Excluded(i) => *i,
            std::ops::Bound::Unbounded => return None,
        };

        Some(Span {
            start: Loc { index: start },
            end: Loc { index: end },
        })
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
}
