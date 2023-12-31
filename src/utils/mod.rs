//!
//! Utilities.
//!

pub mod span;
use std::{
    fs, io,
    ops::RangeBounds,
    path::{Path, PathBuf},
};

pub use span::*;

#[derive(Debug)]
pub struct SourceFile {
    path: PathBuf,
    contents: String,
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
    pub fn source_at<S: TryIntoSpan>(&self, span: impl RangeBounds<S>) -> Option<&str> {
        let span = S::try_into_span(span)?;
        if span.end.index > self.contents.len() {
            return None;
        }

        Some(&self.contents[span.start.index..span.end.index])
    }

    #[cfg(test)]
    pub(crate) fn dummy_file(path: impl AsRef<Path>, contents: impl ToString) -> Self {
        let contents = contents.to_string();
        let line_lengths = Self::split_lines(&contents).collect();
        Self {
            path: path.as_ref().to_owned(),
            contents,
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
            contents,
            line_starts,
        })
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
