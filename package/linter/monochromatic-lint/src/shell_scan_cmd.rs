//! What:
//!  The scanner for `cmd.exe` run bodies.
//! Why:
//!  `cmd` escapes with a caret,
//!  quotes with plain double quotes,
//!  and comments with `rem` or `::` at the start of a line,
//!  and it has only four composing operators.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function scan(text: string): Hit[];
//! ```

/// Import the hit record,
///  the GitHub expression skipper and the construct names.
use crate::shell_dialect::Construct;
use crate::shell_scan::{Hit, expression_end};

/// What:
///  Whether the line that holds `index` is a comment up to that point:
///  only spaces and tabs precede it,
///  and `::` or the word `rem` follows.
/// Why:
///  A comment line may contain `&` and `|` that are not operators.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commentLineAt(text: string, index: number): boolean;
/// ```
fn comment_line_at(text: &str, index: usize) -> bool {
    let bytes: &[u8] = text.as_bytes();
    let mut cursor: usize = index;
    while cursor > 0 && b" \t".contains(&bytes[cursor - 1]) {
        cursor -= 1;
    }
    if cursor > 0 && bytes[cursor - 1] != b'\n' {
        return false;
    }
    let rest: &str = &text[index..];
    if rest.starts_with("::") {
        return true;
    }
    if rest.len() >= 3 && rest.as_bytes()[..3].eq_ignore_ascii_case(b"rem") {
        let after: u8 = rest.as_bytes().get(3).copied().unwrap_or(b' ');
        return b" \t\r\n".contains(&after);
    }
    return false;
}

/// What:
///  Scan one `cmd` run body.
/// Why:
///  One linear pass:
///  a caret skips the next byte,
///  a double quote skips to its pair,
///  and only `|`,
///  `||`,
///  `&&` and `&` are recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(text: string): Hit[];
/// ```
pub(crate) fn scan(text: &str) -> Vec<Hit> {
    let bytes: &[u8] = text.as_bytes();
    let mut hits: Vec<Hit> = Vec::new();
    let mut index: usize = 0;
    while index < bytes.len() {
        if let Some(end) = expression_end(bytes, index) {
            index = end;
            continue;
        }
        let byte: u8 = bytes[index];
        let next: u8 = bytes.get(index + 1).copied().unwrap_or(0);
        if byte == b'^' {
            index += 2;
        } else if byte == b'"' {
            // A double quote runs to its pair; cmd has no escape inside it.
            index += 1;
            while index < bytes.len() && bytes[index] != b'"' {
                index += 1;
            }
            index += 1;
        } else if (byte == b':' || byte == b'r' || byte == b'R') && comment_line_at(text, index) {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if byte == b'|' && next == b'|' {
            hits.push(Hit {
                construct: Construct::OrChain,
                start: index,
                end: index + 2,
            });
            index += 2;
        } else if byte == b'|' {
            hits.push(Hit {
                construct: Construct::Pipe,
                start: index,
                end: index + 1,
            });
            index += 1;
        } else if byte == b'&' && next == b'&' {
            hits.push(Hit {
                construct: Construct::AndChain,
                start: index,
                end: index + 2,
            });
            index += 2;
        } else if byte == b'&' && index > 0 && bytes[index - 1] == b'>' {
            // `2>&1` redirects a handle; it does not chain commands.
            index += 1;
        } else if byte == b'&' {
            hits.push(Hit {
                construct: Construct::Ampersand,
                start: index,
                end: index + 1,
            });
            index += 1;
        } else {
            index += 1;
        }
    }
    return hits;
}

/// `cmd` scan controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_scan_cmd_tests.rs"]
mod tests;
