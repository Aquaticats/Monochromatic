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
//!  The cursor,
//!  the open quotes and the here-documents a scan keeps are in
//!  `shell_scan_posix_state`,
//!  together with the movement each step here uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scan(text: string, fish: boolean): Hit[];
//! ```

/// Import the construct names.
use crate::shell_dialect::Construct;
/// Import the hit record and the GitHub expression skipper.
use crate::shell_scan::{
    Hit,
    expression_end,
};
/// Import the state a scan keeps and the movement its steps share.
use crate::shell_scan_posix_state::{
    Frame,
    Scan,
};

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
            self.frames.push(Frame::Sub {
                depth: 1,
            });
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
            self.frames.push(Frame::Sub {
                depth: 1,
            });
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
        if let Some(Frame::Sub {
            depth,
        }) = self.frames.last_mut()
        {
            *depth += 1;
        }
        self.index += 1;
    }

    /// Count a `)` inside `$( ... )` and leave the substitution when it balances.
    fn close_paren(&mut self) {
        let mut finished: bool = false;
        if let Some(Frame::Sub {
            depth,
        }) = self.frames.last_mut()
        {
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
pub(crate) fn scan(
    text: &str,
    fish: bool,
) -> Vec<Hit> {
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
