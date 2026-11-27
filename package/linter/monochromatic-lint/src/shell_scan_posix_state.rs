//! What:
//!  What one POSIX-family or fish scan keeps while it reads a script,
//!  and the movement every step of the scan shares:
//!  where the cursor stands,
//!  how it steps past a quoted run,
//!  where a hit is recorded,
//!  and how the bodies of the here-documents a line opened are skipped.
//! Why:
//!  Every byte of the script is read once,
//!  so the cursor,
//!  the open quotes and substitutions,
//!  the pending here-documents
//!  and the hits found so far sit in one record,
//!  and the steps that decide what a byte means move that record only through these methods.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Heredoc = { word: string; stripTabs: boolean };
//! type Frame = { kind: 'double' } | { kind: 'sub'; depth: number };
//! type Scan = {
//!   text: string; bytes: Uint8Array; fish: boolean; index: number;
//!   frames: Frame[]; heredocs: Heredoc[]; hits: Hit[];
//! };
//! // record, ahead, behind, skipPast, commentHere, heredocOpener and skipHeredocBodies
//! ```

/// Import the construct names a hit records.
use crate::shell_dialect::Construct;
/// Import the hit record a step reports.
use crate::shell_scan::Hit;

/// A here-document whose body starts on the line after the command line.
pub(crate) struct Heredoc {
    /// The line that ends the body.
    word: String,
    /// `<<-` strips leading tabs before comparing a line with the word.
    strip_tabs: bool,
}

/// Where the scanner is nested.
pub(crate) enum Frame {
    /// Inside double quotes.
    Double,
    /// Inside `$( ... )`,
    ///  counting parentheses to find the end.
    Sub {
        /// Open parentheses not yet closed.
        depth: usize,
    },
}

/// The state of one scan.
pub(crate) struct Scan<'a> {
    /// The run body as text,
    ///  for slicing delimiter words.
    pub(crate) text: &'a str,
    /// The same bytes,
    ///  for indexing.
    pub(crate) bytes: &'a [u8],
    /// Whether fish rules apply:
    ///  no backtick substitution,
    ///  no heredoc,
    ///  no arithmetic expansion,
    ///  and backslash escapes inside single quotes.
    pub(crate) fish: bool,
    /// The next byte to read.
    pub(crate) index: usize,
    /// Nesting of double quotes and `$(`.
    pub(crate) frames: Vec<Frame>,
    /// Here-documents started on the current line.
    pub(crate) heredocs: Vec<Heredoc>,
    /// Everything found so far.
    pub(crate) hits: Vec<Hit>,
}

/// Movement and recording shared by the scan steps.
impl Scan<'_> {
    /// Record a construct covering `length` bytes from the cursor.
    pub(crate) fn record(
        &mut self,
        construct: Construct,
        length: usize,
    ) {
        self.hits.push(Hit {
            construct,
            start: self.index,
            end: self.index + length,
        });
    }

    /// The byte `offset` places ahead of the cursor,
    ///  or zero past the end.
    pub(crate) fn ahead(
        &self,
        offset: usize,
    ) -> u8 {
        // `get` returns None past the end instead of panicking; `copied` turns `Option<&u8>` into `Option<u8>`.
        return self.bytes.get(self.index + offset).copied().unwrap_or(0);
    }

    /// The byte before the cursor,
    ///  or zero at the start.
    pub(crate) fn behind(&self) -> u8 {
        if self.index == 0 {
            return 0;
        }
        return self.bytes[self.index - 1];
    }

    /// Move the cursor to the byte after the next `needle` at or after `from`,
    ///  skipping backslash-escaped bytes when `escapes` is set,
    ///  or to the end when there is none.
    pub(crate) fn skip_past(
        &mut self,
        from: usize,
        needle: u8,
        escapes: bool,
    ) {
        let mut cursor: usize = from;
        while cursor < self.bytes.len() {
            if escapes && self.bytes[cursor] == b'\\' {
                cursor += 2;
                continue;
            }
            if self.bytes[cursor] == needle {
                self.index = cursor + 1;
                return;
            }
            cursor += 1;
        }
        self.index = self.bytes.len();
    }

    /// Whether a `#` at the cursor starts a comment:
    ///  only at the start of a word.
    pub(crate) fn comment_here(&self) -> bool {
        return self.index == 0 || b" \t\n;&|()".contains(&self.behind());
    }

    /// Move past a here-document opener like `<<-'EOF'` and remember its end word.
    ///  Returns false when the `<<` is not an opener.
    pub(crate) fn heredoc_opener(&mut self) -> bool {
        // A third `<` before or after makes this a here-string `<<<`, not a here-document.
        if self.fish
            || self.ahead(1) != b'<'
            || self.ahead(2) == b'<'
            || self.ahead(2) == b'('
            || self.behind() == b'<'
        {
            return false;
        }
        let mut cursor: usize = self.index + 2;
        let strip_tabs: bool = self.bytes.get(cursor) == Some(&b'-');
        if strip_tabs {
            cursor += 1;
        }
        while matches!(self.bytes.get(cursor), Some(b' ') | Some(b'\t')) {
            cursor += 1;
        }
        let start: usize = cursor;
        let quote: u8 = self.bytes.get(cursor).copied().unwrap_or(0);
        if quote == b'\'' || quote == b'"' {
            cursor += 1;
            let word_start: usize = cursor;
            while cursor < self.bytes.len() && self.bytes[cursor] != quote {
                cursor += 1;
            }
            let word: String = String::from(&self.text[word_start..cursor]);
            self.heredocs.push(Heredoc {
                word,
                strip_tabs,
            });
            self.index = (cursor + 1).min(self.bytes.len());
            return true;
        }
        if self.bytes.get(cursor) == Some(&b'\\') {
            cursor += 1;
        }
        let word_start: usize = cursor;
        while cursor < self.bytes.len() && !b" \t\r\n;&|()<>".contains(&self.bytes[cursor]) {
            cursor += 1;
        }
        if cursor == word_start || start == cursor {
            return false;
        }
        let word: String = String::from(&self.text[word_start..cursor]);
        self.heredocs.push(Heredoc {
            word,
            strip_tabs,
        });
        self.index = cursor;
        return true;
    }

    /// Move past the bodies of the here-documents the line just ended opened.
    pub(crate) fn skip_heredoc_bodies(
        &mut self,
        mut position: usize,
    ) {
        let pending: Vec<Heredoc> = std::mem::take(&mut self.heredocs);
        for heredoc in pending {
            loop {
                if position >= self.bytes.len() {
                    break;
                }
                let line_end: usize = match self.text[position..].find('\n') {
                    Some(found) => position + found,
                    None => self.bytes.len(),
                };
                let line: &str = self.text[position..line_end].trim_end_matches('\r');
                let compared: &str = if heredoc.strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    line
                };
                position = (line_end + 1).min(self.bytes.len());
                if compared == heredoc.word {
                    break;
                }
            }
        }
        self.index = position;
    }
}
