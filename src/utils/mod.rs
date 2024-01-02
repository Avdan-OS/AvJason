//!
//! Utilities.
//!

pub mod span;
use std::{
    fmt::Debug,
    fs, io,
    ops::RangeBounds,
    path::{Path, PathBuf},
};

pub use span::*;

use crate::lex::{LexError, LexResult};

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    contents: Vec<char>,
    line_starts: Vec<usize>,
}

impl SourceFile {
    ///
    /// Splits lines by ECMA-abiding line endings.
    ///
    fn split_lines(src: &str) -> impl Iterator<Item = usize> + '_ {
        src.chars()
            .enumerate()
            .map_windows(|[(a_i, a), (b_i, b)]| {
                // Implementing https://262.ecma-international.org/5.1/#sec-7.3
                Some(match (*a, *b) {
                    ('\n', _) => a_i + 1,
                    ('\r', '\n') => b_i + 1,
                    ('\r', _) => a_i + 1,
                    ('\u{2028}', _) => a_i + 1,
                    ('\u{2029}', _) => a_i + 1,
                    _ => return None,
                })
            })
            .flatten()
            .chain(std::iter::once(src.len()))
    }

    ///
    /// Returns a string representing a [Loc] in ${FILE}:${LINE}:${COLUMN} format.
    ///
    pub fn file_line_column(&self, loc: &Loc) -> Option<String> {
        let Some((ln, col)) = self
            .line_starts
            .iter()
            .enumerate()
            .find(|(_, i)| loc.index < **i)
            .map(|(ln, len)| (ln, len - loc.index))
        else {
            return None;
        };

        Some(format!("{}:{ln}:{col}", &self.path.to_str()?))
    }

    ///
    /// Returns the original source code at a particular [Span].
    ///
    pub fn source_at<S: TryIntoSpan>(&self, span: impl RangeBounds<S>) -> Option<String> {
        let span = S::try_into_span(span)?;
        if span.end.index > self.contents.len() {
            return None;
        }

        if span.start.index >= span.end.index {
            return None;
        }

        Some(
            self.contents[span.start.index..span.end.index]
                .iter()
                .collect(),
        )
    }

    ///
    /// Returns the original source code at a particular [Span].
    ///
    pub fn source_at_span(&self, span: Span) -> Option<String> {
        if span.end.index > self.contents.len() {
            return None;
        }

        Some(
            self.contents[span.start.index..span.end.index]
                .iter()
                .collect(),
        )
    }

    #[cfg(test)]
    pub(crate) fn dummy_file(path: impl AsRef<Path>, contents: impl ToString) -> Self {
        let contents = contents.to_string();
        let line_lengths = Self::split_lines(&contents).collect();
        Self {
            path: path.as_ref().to_owned(),
            contents: contents.chars().collect(),
            line_starts: line_lengths,
        }
    }

    ///
    /// Attempts to read a [SourceFile] from a file.
    ///
    pub fn load_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let line_starts = Self::split_lines(&contents).collect();

        Ok(Self {
            path: path.to_owned(),
            contents: contents.chars().collect(),
            line_starts,
        })
    }

    pub(crate) fn iter(&self) -> SourceIter {
        SourceIter::new(self)
    }
}

#[derive(Clone)]
pub struct SourceIter<'a> {
    file: &'a SourceFile,
    inner: &'a Vec<char>,
    index: usize,
}

impl<'a> std::fmt::Debug for SourceIter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceIter")
            .field("left", &String::from_iter(&self.inner[self.index..]))
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> SourceIter<'a> {
    pub(crate) fn new(file: &'a SourceFile) -> Self {
        Self {
            file,
            inner: &file.contents,
            index: 0,
        }
    }

    pub(crate) fn peek(&self) -> Option<&char> {
        self.inner.get(self.index)
    }

    pub(crate) fn peek2(&self) -> Option<&char> {
        self.inner.get(self.index + 1)
    }

    pub(crate) fn fork(&self) -> Self {
        self.clone()
    }

    pub(crate) fn ahead(&self, range: impl RangeBounds<usize>) -> Option<String> {
        let abs_start = self.index
            + match range.start_bound() {
                std::ops::Bound::Included(d) => *d,
                std::ops::Bound::Excluded(d) => (*d) + 1,
                std::ops::Bound::Unbounded => 0,
            };

        let abs_end = self.index
            + match range.end_bound() {
                std::ops::Bound::Included(d) => *d + 1,
                std::ops::Bound::Excluded(d) => *d,
                std::ops::Bound::Unbounded => self.inner.len(),
            };

        if !(abs_start < self.inner.len() && abs_end <= self.inner.len()) {
            return None;
        }

        Some(self.inner[abs_start..abs_end].iter().collect())
    }

    pub(crate) fn relative_match(
        &self,
        range: impl RangeBounds<usize>,
        pred: impl Fn(&char) -> bool,
    ) -> bool {
        let abs_start = self.index
            + match range.start_bound() {
                std::ops::Bound::Included(d) => *d,
                std::ops::Bound::Excluded(d) => (*d) + 1,
                std::ops::Bound::Unbounded => 0,
            };
        let abs_end = self.index
            + match range.end_bound() {
                std::ops::Bound::Included(d) => *d + 1,
                std::ops::Bound::Excluded(d) => *d,
                std::ops::Bound::Unbounded => self.inner.len(),
            };

        if !(abs_start < self.inner.len() && abs_end <= self.inner.len()) {
            return false;
        }

        let s = &self.inner[abs_start..abs_end];
        s.iter().all(pred)
    }

    pub(crate) fn offset(&mut self, offset: usize) {
        self.index += offset;
    }

    pub(crate) fn advance_to(&mut self, other: Self) {
        self.index = other.index;
    }

    pub(crate) fn error(&self) -> SourceErrorHelper {
        SourceErrorHelper { iter: self }
    }
}

pub(crate) struct SourceErrorHelper<'a> {
    iter: &'a SourceIter<'a>,
}

impl<'a> SourceErrorHelper<'a> {
    pub(crate) fn unexpected<T, A>(
        self,
        range: Option<impl RangeBounds<A>>,
        token: impl ToString,
    ) -> LexResult<T>
    where
        isize: TryFrom<A>,
        A: Copy + Debug,
        <isize as std::convert::TryFrom<A>>::Error: Debug,
    {
        let token = token.to_string();

        let mut text = None;
        let mut span = 0..self.iter.inner.len();
        if let Some(range) = range {
            let i = self.iter.index as isize;
            let start = i + match range.start_bound() {
                std::ops::Bound::Included(r) => isize::try_from(*r).unwrap(),
                std::ops::Bound::Excluded(r) => isize::try_from(*r).unwrap() + 1,
                std::ops::Bound::Unbounded => 0isize,
            };

            let end = i + match range.start_bound() {
                std::ops::Bound::Included(r) => isize::try_from(*r).unwrap() + 1,
                std::ops::Bound::Excluded(r) => isize::try_from(*r).unwrap(),
                std::ops::Bound::Unbounded => self.iter.inner.len() as isize,
            };

            let start = start as usize;
            let end = end as usize;

            text = self.iter.file.source_at(start..end);
            span = start..end;
        }

        Err(LexError::new(
            span,
            format!("Unexpected token `{token}`"),
            text,
        ))
    }

    pub(crate) fn expected<T, A>(
        self,
        rel_range: Option<impl RangeBounds<A>>,
        token: impl ToString,
    ) -> LexResult<T>
    where
        isize: TryFrom<A>,
        A: Copy + Debug,
        <isize as std::convert::TryFrom<A>>::Error: Debug,
    {
        let token = token.to_string();

        let mut text = None;
        let mut span = 0..self.iter.inner.len();
        if let Some(range) = rel_range {
            let i = self.iter.index as isize;
            let start = i + match range.start_bound() {
                std::ops::Bound::Included(r) => isize::try_from(*r).unwrap(),
                std::ops::Bound::Excluded(r) => isize::try_from(*r).unwrap() + 1,
                std::ops::Bound::Unbounded => 0isize,
            };

            let end = i + match range.start_bound() {
                std::ops::Bound::Included(r) => isize::try_from(*r).unwrap() + 1,
                std::ops::Bound::Excluded(r) => isize::try_from(*r).unwrap(),
                std::ops::Bound::Unbounded => self.iter.inner.len() as isize,
            };

            let start = start as usize;
            let end = end as usize;

            text = self.iter.file.source_at(start..end);
            span = start..end;
        }

        Err(LexError::new(
            span,
            format!("Expected token `{token}` here"),
            text,
        ))
    }
}

impl<'a> Iterator for SourceIter<'a> {
    type Item = (Loc, char);

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.inner.get(self.index)?;
        let l = Loc { index: self.index };

        self.index += 1;

        Some((l, *ch))
    }
}

#[cfg(test)]
mod tests {
    use super::SourceFile;

    #[test]
    fn source_file() {
        let src = SourceFile::dummy_file("example.txt", "I am a\ngood file!\n\nGimme a pet!");
        println!("{src:?}");

        println!("{:?}", src.source_at(7..11))
    }
}
