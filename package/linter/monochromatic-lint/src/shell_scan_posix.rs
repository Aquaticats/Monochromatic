//! What:
//!  The scanner for POSIX-family shells and fish.
//! Why:
//!  bash,
//!  sh,
//!  zsh and fish share quoting,
//!  comments,
//!  redirections and `$(...)`,
//!  and differ in backticks,
//!  heredocs and arithmetic,
//!  which a flag selects.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scan(text: string, fish: boolean): Hit[];
//! ```

/// Import the construct names.
use crate::shell_dialect::Construct;
/// Import the hit record and the GitHub expression skipper.
use crate::shell_scan::{Hit, expression_end};

/// A here-document whose body starts on the line after the command line.
struct Heredoc {
    /// The line that ends the body.
    word: String,
    /// `<<-` strips leading tabs before comparing a line with the word.
    strip_tabs: bool,
}

/// Where the scanner is nested.
enum Frame {
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
struct Scan<'a> {
    /// The run body as text,
    ///  for slicing delimiter words.
    text: &'a str,
    /// The same bytes,
    ///  for indexing.
    bytes: &'a [u8],
    /// Whether fish rules apply:
    ///  no backtick substitution,
    ///  no heredoc,
    ///  no arithmetic expansion,
    ///  and backslash escapes inside single quotes.
    fish: bool,
    /// The next byte to read.
    index: usize,
    /// Nesting of double quotes and `$(`.
    frames: Vec<Frame>,
    /// Here-documents started on the current line.
    heredocs: Vec<Heredoc>,
    /// Everything found so far.
    hits: Vec<Hit>,
}

/// Movement and recording shared by the scan steps.
impl Scan<'_> {
    /// Record a construct covering `length` bytes from the cursor.
    fn record(&mut self, construct: Construct, length: usize) {
        self.hits.push(Hit {
            construct,
            start: self.index,
            end: self.index + length,
        });
    }

    /// The byte `offset` places ahead of the cursor,
    ///  or zero past the end.
    fn ahead(&self, offset: usize) -> u8 {
        // `get` returns None past the end instead of panicking; `copied` turns `Option<&u8>` into `Option<u8>`.
        return self.bytes.get(self.index + offset).copied().unwrap_or(0);
    }

    /// The byte before the cursor,
    ///  or zero at the start.
    fn behind(&self) -> u8 {
        if self.index == 0 {
            return 0;
        }
        return self.bytes[self.index - 1];
    }

    /// Move the cursor to the byte after the next `needle` at or after `from`,
    ///  skipping backslash-escaped bytes when `escapes` is set,
    ///  or to the end when there is none.
    fn skip_past(&mut self, from: usize, needle: u8, escapes: bool) {
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
    fn comment_here(&self) -> bool {
        return self.index == 0 || b" \t\n;&|()".contains(&self.behind());
    }

    /// Move past a here-document opener like `<<-'EOF'` and remember its end word.
    ///  Returns false when the `<<` is not an opener.
    fn heredoc_opener(&mut self) -> bool {
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
            self.heredocs.push(Heredoc { word, strip_tabs });
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
        self.heredocs.push(Heredoc { word, strip_tabs });
        self.index = cursor;
        return true;
    }

    /// Move past the bodies of the here-documents the line just ended opened.
    fn skip_heredoc_bodies(&mut self, mut position: usize) {
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

/// The steps for one byte inside double quotes,
///  and one byte of code.
impl Scan<'_> {
    /// Handle one byte inside double quotes,
    ///  where only `$(` and backticks still run commands.
    fn in_double(&mut self) {
        let byte: u8 = self.bytes[self.index];
        if byte == b'\\' {
            self.index += 2;
        } else if byte == b'"' {
            self.frames.pop();
            self.index += 1;
        } else if byte == b'$' && self.ahead(1) == b'(' && self.ahead(2) != b'(' {
            self.record(Construct::DollarParen, 2);
            self.frames.push(Frame::Sub { depth: 1 });
            self.index += 2;
        } else if byte == b'`' && !self.fish {
            self.record(Construct::Backtick, 1);
            self.skip_past(self.index + 1, b'`', true);
        } else {
            self.index += 1;
        }
    }

    /// Handle a `$` in code:
    ///  `$((` arithmetic,
    ///  `$(` substitution,
    ///  or `$'` quoting.
    fn dollar(&mut self) {
        if self.ahead(1) == b'(' && self.ahead(2) == b'(' && !self.fish {
            // Arithmetic expansion is not a command; skip to the closing `))`.
            let from: usize = self.index + 3;
            match self.text[from..].find("))") {
                Some(found) => self.index = from + found + 2,
                None => self.index = self.bytes.len(),
            }
        } else if self.ahead(1) == b'(' {
            self.record(Construct::DollarParen, 2);
            self.frames.push(Frame::Sub { depth: 1 });
            self.index += 2;
        } else if self.ahead(1) == b'\'' && !self.fish {
            self.skip_past(self.index + 2, b'\'', true);
        } else {
            self.index += 1;
        }
    }

    /// Handle `|`,
    ///  `||`,
    ///  `|&` and the `>|` redirection.
    fn pipe(&mut self) {
        if self.behind() == b'>' {
            self.index += 1;
        } else if self.ahead(1) == b'|' {
            self.record(Construct::OrChain, 2);
            self.index += 2;
        } else if self.ahead(1) == b'&' {
            self.record(Construct::PipeStderr, 2);
            self.index += 2;
        } else {
            self.record(Construct::Pipe, 1);
            self.index += 1;
        }
    }

    /// Handle `&&`,
    ///  a lone `&`,
    ///  and the redirections `>&`,
    ///  `<&` and `&>`.
    fn ampersand(&mut self) {
        if self.ahead(1) == b'&' {
            self.record(Construct::AndChain, 2);
            self.index += 2;
        } else if self.behind() == b'>' || self.behind() == b'<' || self.ahead(1) == b'>' {
            self.index += 1;
        } else {
            self.record(Construct::Ampersand, 1);
            self.index += 1;
        }
    }

    /// Handle one byte of code.
    fn in_code(&mut self) {
        let byte: u8 = self.bytes[self.index];
        match byte {
            b'#' if self.comment_here() => {
                while self.index < self.bytes.len() && self.bytes[self.index] != b'\n' {
                    self.index += 1;
                }
            }
            b'\\' => self.index += 2,
            b'\'' => self.skip_past(self.index + 1, b'\'', self.fish),
            b'"' => {
                self.frames.push(Frame::Double);
                self.index += 1;
            }
            b'`' if !self.fish => {
                self.record(Construct::Backtick, 1);
                self.skip_past(self.index + 1, b'`', true);
            }
            b'$' => self.dollar(),
            b'|' => self.pipe(),
            b'&' => self.ampersand(),
            b';' => {
                self.record(Construct::Semicolon, 1);
                while self.ahead(0) == b';' {
                    self.index += 1;
                }
            }
            b'<' => {
                if !self.heredoc_opener() {
                    self.index += 1;
                }
            }
            b'\n' if !self.heredocs.is_empty() => self.skip_heredoc_bodies(self.index + 1),
            b'(' => self.open_paren(),
            b')' => self.close_paren(),
            _ => self.index += 1,
        }
    }

    /// Count a `(` inside `$( ... )`.
    fn open_paren(&mut self) {
        if let Some(Frame::Sub { depth }) = self.frames.last_mut() {
            *depth += 1;
        }
        self.index += 1;
    }

    /// Count a `)` inside `$( ... )` and leave the substitution when it balances.
    fn close_paren(&mut self) {
        let mut finished: bool = false;
        if let Some(Frame::Sub { depth }) = self.frames.last_mut() {
            *depth -= 1;
            finished = *depth == 0;
        }
        if finished {
            self.frames.pop();
        }
        self.index += 1;
    }
}

/// What:
///  Scan one POSIX-family or fish run body.
/// Why:
///  One linear pass:
///  each byte is read once,
///  and the nesting stack holds only open quotes and substitutions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(text: string, fish: boolean): Hit[];
/// ```
pub(crate) fn scan(text: &str, fish: bool) -> Vec<Hit> {
    let mut state: Scan<'_> = Scan {
        text,
        bytes: text.as_bytes(),
        fish,
        index: 0,
        frames: Vec::new(),
        heredocs: Vec::new(),
        hits: Vec::new(),
    };
    while state.index < state.bytes.len() {
        if let Some(end) = expression_end(state.bytes, state.index) {
            state.index = end;
            continue;
        }
        if matches!(state.frames.last(), Some(Frame::Double)) {
            state.in_double();
        } else {
            state.in_code();
        }
    }
    return state.hits;
}

/// POSIX and fish scan controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_scan_posix_tests.rs"]
mod tests;
