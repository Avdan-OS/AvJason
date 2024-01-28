//!
//! A source file.
//!

use std::{fmt::Formatter, ops::Range, path::Path};

use super::{Loc, Source, Span, Spanned};

///
/// Line and column information for
/// a particular location in source code.
///
#[derive(Debug)]
pub struct LineColumn<'a> {
    file: &'a str,
    line: usize,
    column: usize,
}

///
/// Converting to 1-based only for display.
///
impl<'a> std::fmt::Display for LineColumn<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line + 1, self.column + 1)
    }
}

///
/// Finds the starting character index of all
/// lines, using any [ECMAScript LineTerminatorSequence](https://262.ecma-international.org/5.1/#sec-7.3)
/// to delimit lines.
///
fn line_starts(st: &[char]) -> Vec<usize> {
    let mut v = vec![0];
    let mut i = 0;

    while i < st.len() {
        let ch = st[i];

        match ch {
            '\u{000A}' => v.push(i + 1), // <LF>
            '\u{2028}' => v.push(i + 1), // <LS>
            '\u{2029}' => v.push(i + 1), // <PS>
            '\u{000D}' => {
                if matches!(st.get(i + 1), Some('\u{000A}')) {
                    v.push(i + 2); // <CR><LF>
                    i += 1;
                } else {
                    v.push(i + 1); // <CR>
                }
            }
            _ => (),
        }

        i += 1;
    }

    if matches!(v.last(), Some(i) if *i >= st.len()) {
        let _ = v.pop();
    }

    v
}

///
/// A real source file.
///
/// Here, line-column information can be provided.
///
#[derive(Debug, Clone)]
pub struct SourceFile {
    path: String,
    contents: String,
    chars: Vec<char>,
    line_starts: Vec<usize>,
}

impl SourceFile {
    ///
    /// TESTING ONLY
    /// ***
    /// Create a dumy file with a fake path.
    ///
    #[cfg(test)]
    pub fn dummy_file(contents: &'static str) -> Self {
        let path = "DUMMY.FILE".to_string();
        let contents = contents.to_string();

        let chars = contents.chars().collect::<Vec<_>>();
        let line_starts = line_starts(&chars);

        Self {
            path,
            contents,
            chars,
            line_starts,
        }
    }

    ///
    /// Attempts to read source code from a given file path.
    ///
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = path.as_ref().to_owned();
        let contents = std::fs::read_to_string(&path)?;

        let path = path.to_str().expect("Valid path as string").to_string();
        let chars = contents.chars().collect::<Vec<_>>();
        let line_starts = line_starts(&chars);

        Ok(Self {
            path,
            contents,
            chars,
            line_starts,
        })
    }

    ///
    /// Return the (0-based) line and column information at a [Loc] in this file.
    ///
    fn line_col(&self, loc: Loc) -> Option<(usize, usize)> {
        // Essentially, pair the start of the a line with the end of the next (or EOF),
        // check if loc is in its range. If so, get the corresponding line and calculate the
        // corresponding column.
        self.line_starts
            .iter()
            .copied()
            .zip(
                self.line_starts
                    .iter()
                    .copied()
                    .skip(1)
                    .chain([self.contents.len()]),
            )
            .enumerate()
            .filter(|&(_, (start_col, end_col))| (start_col <= loc.0 && loc.0 < end_col))
            .map(|(ln, (start_col, _))| (ln, loc.0 - start_col))
            .next()
    }
}

impl Source for SourceFile {
    type Location<'a> = LineColumn<'a>
    where Self: 'a;

    fn locate(&self, span: Span) -> Option<Self::Location<'_>> {
        if self.in_bounds(&span) {
            let (line, column) = self.line_col(span.start)?;
            return Some(LineColumn {
                file: &self.path,
                line,
                column,
            });
        }

        None
    }

    fn bounds(&self) -> Range<Loc> {
        Loc(0)..Loc(self.chars.len())
    }

    fn source_at(&self, span: impl Spanned) -> Option<String> {
        let span = span.span();
        if self.in_bounds(&span) {
            return Some(self.chars[span.as_range()].iter().collect());
        }

        None
    }

    fn characters(&self) -> &[char] {
        &self.chars
    }
}

#[cfg(test)]
mod tests {
    use crate::common::{file::LineColumn, Source};

    use super::{super::ToSpan, line_starts, SourceFile};

    #[test]
    fn lines() {
        assert!(matches!(
            &line_starts(&"ba\nb\nc".chars().collect::<Vec<_>>())[..],
            &[0, 3, 5]
        ));

        assert!(matches!(
            &line_starts(
                &"babs\r\nbaaa\r__\u{2028}asagsgas\u{2029}a\nc\n"
                    .chars()
                    .collect::<Vec<_>>()
            )[..],
            &[0, 6, 11, 14, 23, 25,]
        ))
    }

    #[test]
    fn line_col() {
        let f = SourceFile::dummy_file("PEN\nPINEAPPLE\nAPPLE\nPEN");
        let ananas = (4..13).to_span(&f);
        assert_eq!(f.source_at(ananas), Some("PINEAPPLE".to_string()));
        assert!(matches!(
            f.locate(ananas),
            Some(LineColumn {
                line: 1,
                column: 0,
                ..
            })
        ));
    }
}
