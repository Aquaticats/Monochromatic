//! What:
//!  Splits `.browserslistrc` text into query lines, accepting only `#` comments and blank lines.
//! Why:
//!  npm browserslist's `parseConfig` also handles commas, sections and CR line endings;
//!  the user's decision accepts only comments and blank lines and refuses the rest with "not implemented".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseConfig(text: string): { number: number; text: string }[]; // throws NotImplemented
//! ```

use crate::error::{Outcome, not_implemented};

/// What:
///  One query line with its 1-based line number in the config text.
/// Why:
///  Diagnostics name the line, and the line number survives comment and blank-line removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigLine = { number: number; text: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLine {
    /// 1-based line number in the original text.
    pub number: usize,
    /// The line with its comment removed and spaces and tabs trimmed from both ends.
    pub text: String,
}

/// What:
///  Whether a trimmed line has the shape of a browserslist section header, `[name]`.
/// Why:
///  browserslist's `IS_SECTION` is `^\s*\[(.+)]\s*$`: an opening bracket, at least one character, a closing bracket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSection = (line: string) => /^\[(.+)\]$/.test(line);
/// ```
fn is_section_header(trimmed: &str) -> bool {
    return trimmed.len() >= 3 && trimmed.starts_with('[') && trimmed.ends_with(']');
}

/// What:
///  Turn config text into query lines.
///  A `#` starts a comment that runs to the end of its line, wherever the `#` is;
///  blank lines are dropped;
///  a carriage return, a comma outside a comment, or a section header is refused as not implemented.
/// Why:
///  This mirrors the comment rule of `parseConfig` and rejects every other `parseConfig` feature.
/// Gotcha:
///  Only spaces and tabs are trimmed, not the full JavaScript whitespace set, so an exotic space makes the
///  line fail to match any supported query instead of being silently trimmed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfig(text: string): ConfigLine[];
/// ```
pub fn parse_config(text: &str) -> Outcome<Vec<ConfigLine>> {
    if let Some(position) = text.find('\r') {
        let line_number = text[..position].matches('\n').count() + 1;
        return Err(not_implemented(&format!(
            "CR line endings in the browserslist config (carriage return on line {line_number}); use LF line endings"
        )));
    }
    let mut lines: Vec<ConfigLine> = Vec::new();
    for (index, raw_line) in text.split('\n').enumerate() {
        let number = index + 1;
        let without_comment = raw_line.split('#').next().unwrap_or("");
        if without_comment.contains(',') {
            return Err(not_implemented(&format!(
                "commas as query separators in the browserslist config (line {number}); put one query on each line"
            )));
        }
        let trimmed = without_comment.trim_matches([' ', '\t']);
        if trimmed.is_empty() {
            continue;
        }
        if is_section_header(trimmed) {
            return Err(not_implemented(&format!(
                "config sections in the browserslist config ({trimmed:?} on line {number})"
            )));
        }
        lines.push(ConfigLine { number, text: trimmed.to_string() });
    }
    return Ok(lines);
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
