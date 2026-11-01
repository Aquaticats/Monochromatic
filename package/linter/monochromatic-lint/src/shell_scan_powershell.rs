//! What:
//!  The scanner for PowerShell run bodies.
//! Why:
//!  PowerShell quotes with doubled quote characters,
//!  escapes with a backtick,
//!  runs `$(...)` even inside double quotes,
//!  and uses a leading `&` as the call operator,
//!  all unlike the POSIX family.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scan(text: string): Hit[];
//! ```

/// Import the construct names.
use crate::shell_dialect::Construct;
/// Import the hit record and the GitHub expression skipper.
use crate::shell_scan::{Hit, expression_end};

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
    /// The run body as text.
    text: &'a str,
    /// The same bytes,
    ///  for indexing.
    bytes: &'a [u8],
    /// The next byte to read.
    index: usize,
    /// Nesting of double quotes and `$(`.
    frames: Vec<Frame>,
    /// Everything found so far.
    hits: Vec<Hit>,
}

/// Movement and recording for the scan steps.
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
        return self.bytes.get(self.index + offset).copied().unwrap_or(0);
    }

    /// The last byte before the cursor that is not a space,
    ///  tab or carriage return,
    ///  or zero when there is none.
    fn previous_significant(&self) -> u8 {
        let mut cursor: usize = self.index;
        while cursor > 0 {
            cursor -= 1;
            if !b" \t\r".contains(&self.bytes[cursor]) {
                return self.bytes[cursor];
            }
        }
        return 0;
    }

    /// Move past the next `quote` that is not doubled,
    ///  starting at `from`.
    fn skip_quoted(&mut self, from: usize, quote: u8) {
        let mut cursor: usize = from;
        while cursor < self.bytes.len() {
            if self.bytes[cursor] == quote {
                if self.bytes.get(cursor + 1) == Some(&quote) {
                    cursor += 2;
                    continue;
                }
                self.index = cursor + 1;
                return;
            }
            cursor += 1;
        }
        self.index = self.bytes.len();
    }

    /// Move past a here-string body that ends at a line starting with `closer`,
    ///  recording `$(` inside it when `live` is set.
    fn skip_here_string(&mut self, closer: &str, live: bool) {
        let body_start: usize = self.index + 2;
        let end: usize = match self.text[body_start..].find(closer) {
            Some(found) => body_start + found,
            None => self.bytes.len(),
        };
        if live {
            let mut cursor: usize = body_start;
            while let Some(found) = self.text[cursor..end].find("$(") {
                self.hits.push(Hit {
                    construct: Construct::DollarParen,
                    start: cursor + found,
                    end: cursor + found + 2,
                });
                cursor += found + 2;
            }
        }
        self.index = (end + closer.len()).min(self.bytes.len());
    }

    /// Whether a `#` at the cursor starts a comment.
    fn comment_here(&self) -> bool {
        return self.index == 0 || b" \t\r\n;|({".contains(&self.bytes[self.index - 1]);
    }

    /// Move past a `<# ... #>` block comment.
    fn skip_block_comment(&mut self) {
        match self.text[self.index + 2..].find("#>") {
            Some(found) => self.index = self.index + 2 + found + 2,
            None => self.index = self.bytes.len(),
        }
    }

    /// Whether a here-string opener `@'` or `@"` ends its line.
    fn here_string_opener(&self) -> bool {
        let quote: u8 = self.ahead(1);
        if self.ahead(0) != b'@' || (quote != b'\'' && quote != b'"') {
            return false;
        }
        let mut cursor: usize = self.index + 2;
        while matches!(
            self.bytes.get(cursor),
            Some(b' ') | Some(b'\t') | Some(b'\r')
        ) {
            cursor += 1;
        }
        return self.bytes.get(cursor) == Some(&b'\n');
    }
}

/// The steps for one byte inside double quotes,
///  and one byte of code.
impl Scan<'_> {
    /// Handle one byte inside double quotes,
    ///  where only `$(` still runs commands.
    fn in_double(&mut self) {
        let byte: u8 = self.bytes[self.index];
        // A backtick escapes the next byte, and a doubled quote is one literal quote.
        if byte == b'`' || (byte == b'"' && self.ahead(1) == b'"') {
            self.index += 2;
        } else if byte == b'"' {
            self.frames.pop();
            self.index += 1;
        } else if byte == b'$' && self.ahead(1) == b'(' {
            self.record(Construct::DollarParen, 2);
            self.frames.push(Frame::Sub { depth: 1 });
            self.index += 2;
        } else {
            self.index += 1;
        }
    }

    /// Handle `&&`,
    ///  `&` as the call operator at the start of a statement,
    ///  the redirection `>&`,
    ///  and a background `&`.
    fn ampersand(&mut self) {
        let before: u8 = self.previous_significant();
        if self.ahead(1) == b'&' {
            self.record(Construct::AndChain, 2);
            self.index += 2;
        } else if (self.index > 0 && self.bytes[self.index - 1] == b'>')
            || before == 0
            || b"\n;|({".contains(&before)
        {
            // A redirection like `2>&1`, or the call operator at the start of a statement.
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
            b'<' if self.ahead(1) == b'#' => self.skip_block_comment(),
            b'#' if self.comment_here() => {
                while self.index < self.bytes.len() && self.bytes[self.index] != b'\n' {
                    self.index += 1;
                }
            }
            b'`' => self.index += 2,
            b'@' if self.here_string_opener() => {
                let double: bool = self.ahead(1) == b'"';
                let closer: &str = if double { "\n\"@" } else { "\n'@" };
                self.skip_here_string(closer, double);
            }
            b'\'' => self.skip_quoted(self.index + 1, b'\''),
            b'"' => {
                self.frames.push(Frame::Double);
                self.index += 1;
            }
            b'$' if self.ahead(1) == b'(' => {
                self.record(Construct::DollarParen, 2);
                self.frames.push(Frame::Sub { depth: 1 });
                self.index += 2;
            }
            b'|' if self.ahead(1) == b'|' => {
                self.record(Construct::OrChain, 2);
                self.index += 2;
            }
            b'|' => {
                self.record(Construct::Pipe, 1);
                self.index += 1;
            }
            b'&' => self.ampersand(),
            b';' => {
                self.record(Construct::Semicolon, 1);
                while self.ahead(0) == b';' {
                    self.index += 1;
                }
            }
            b'(' => {
                if let Some(Frame::Sub { depth }) = self.frames.last_mut() {
                    *depth += 1;
                }
                self.index += 1;
            }
            b')' => {
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
            _ => self.index += 1,
        }
    }
}

/// What:
///  Scan one PowerShell run body.
/// Why:
///  One linear pass over the bytes,
///  with a stack that holds only open double quotes and subexpressions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(text: string): Hit[];
/// ```
pub(crate) fn scan(text: &str) -> Vec<Hit> {
    let mut state: Scan<'_> = Scan {
        text,
        bytes: text.as_bytes(),
        index: 0,
        frames: Vec::new(),
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

/// PowerShell scan controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_scan_powershell_tests.rs"]
mod tests;
