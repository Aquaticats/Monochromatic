//! What:
//!  Line reading for the workflow walker:
//!  indentation,
//!  dashes,
//!  keys and values.
//! Why:
//!  The walker needs a key's column and its value's bytes,
//!  and nothing else of YAML;
//!  a form this reader cannot place is reported, not guessed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Line = { start: number; end: number; indent: number; text: string };
//! function parseEntry(line: Line): Entry;
//! ```

/// Import the physical line splitter and the refusal record.
use crate::processors_lines::physical_lines;
use crate::workflow_model::WalkError;

/// One physical line with its indentation removed.
#[derive(Debug)]
pub(crate) struct Line<'a> {
    /// First byte of the line.
    pub start: usize,
    /// One past the newline,
    ///  or the end of the text.
    pub end: usize,
    /// Leading spaces.
    pub indent: usize,
    /// The text after the indentation,
    ///  without the newline.
    pub text: &'a str,
}

/// Line-level questions the walker asks.
impl Line<'_> {
    /// Whether the line has no content.
    pub fn is_blank(&self) -> bool {
        return self.text.trim().is_empty();
    }

    /// Whether the line is blank or only a comment.
    pub fn is_skippable(&self) -> bool {
        return self.is_blank() || self.text.starts_with('#');
    }

    /// First byte of the text after the indentation.
    pub fn text_start(&self) -> usize {
        return self.start + self.indent;
    }
}

/// What:
///  Split the workflow into lines and refuse tab indentation.
/// Why:
///  YAML forbids tabs in indentation,
///  so a tab means the structure cannot be read reliably.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLines(text: string): Line[];
/// ```
pub(crate) fn read_lines(text: &str) -> Result<Vec<Line<'_>>, WalkError> {
    let mut lines: Vec<Line<'_>> = Vec::new();
    for physical in physical_lines(text) {
        let raw: &str = &text[physical.start..physical.content_end];
        let stripped: &str = raw.trim_start_matches(' ');
        if stripped.starts_with('\t') {
            return Err(WalkError {
                offset: physical.start + (raw.len() - stripped.len()),
                message: String::from(
                    "tab indentation; YAML indents with spaces, so the structure cannot be read",
                ),
            });
        }
        lines.push(Line {
            start: physical.start,
            end: physical.end,
            indent: raw.len() - stripped.len(),
            text: stripped,
        });
    }
    return Ok(lines);
}

/// What one line says after its dashes.
pub(crate) enum Content<'a> {
    /// Nothing follows the dashes:
    ///  the item's mapping starts on later lines.
    Nothing,
    /// A plain `key: value` pair.
    Key {
        /// The key text.
        name: &'a str,
        /// Column of the key.
        column: usize,
        /// The value text with surrounding space removed,
        ///  still carrying any trailing comment.
        value: &'a str,
        /// Offset of the value text from the start of the line's text.
        value_offset: usize,
    },
    /// A key written in a form the walker does not read:
    ///  quoted,
    ///  an anchor,
    ///  a tag,
    ///  an alias,
    ///  a merge key or a complex key.
    OddKey,
    /// A scalar,
    ///  a flow collection or other text with no key.
    Scalar,
}

/// One line split into dashes and content.
pub(crate) struct Entry<'a> {
    /// Columns of the sequence dashes at the start of the line.
    pub dashes: Vec<usize>,
    /// What follows the dashes.
    pub content: Content<'a>,
}

/// Characters that start a key form the walker does not read.
const ODD_KEY_START: &[u8] = b"\"'?*&!%@[{|>`";

/// What:
///  Find the colon that ends a plain key,
///  if the text is a `key: value` pair.
/// Why:
///  A colon only ends a key when a space or the end of the line follows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyColon(text: string): number | undefined;
/// ```
fn key_colon(text: &str) -> Option<usize> {
    let bytes: &[u8] = text.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b':' && (index + 1 == bytes.len() || bytes[index + 1] == b' ') {
            return Some(index);
        }
    }
    return None;
}

/// What:
///  Split a line into its dashes and its key or scalar.
/// Why:
///  `- run: x` is a dash and a key whose column is after the dash,
///  and the column decides which mapping the key belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseEntry(line: Line): Entry;
/// ```
pub(crate) fn parse_entry<'a>(line: &Line<'a>) -> Entry<'a> {
    let mut dashes: Vec<usize> = Vec::new();
    let mut column: usize = line.indent;
    let mut rest: &'a str = line.text;
    // Bytes from the start of the line's text to the start of `rest`.
    let mut consumed: usize = 0;
    while rest == "-" || rest.starts_with("- ") {
        dashes.push(column);
        let after: &str = &rest[1..];
        let trimmed: &'a str = after.trim_start_matches(' ');
        let step: usize = 1 + (after.len() - trimmed.len());
        column += step;
        consumed += step;
        rest = trimmed;
    }
    if rest.is_empty() || rest.starts_with('#') {
        return Entry {
            dashes,
            content: Content::Nothing,
        };
    }
    let Some(colon) = key_colon(rest) else {
        return Entry {
            dashes,
            content: Content::Scalar,
        };
    };
    let name: &str = rest[..colon].trim_end();
    let first: u8 = name.as_bytes().first().copied().unwrap_or(b' ');
    if ODD_KEY_START.contains(&first) || name == "<<" {
        return Entry {
            dashes,
            content: Content::OddKey,
        };
    }
    let after: &str = &rest[colon + 1..];
    let value: &'a str = after.trim_start_matches(' ');
    let value_offset: usize = consumed + colon + 1 + (after.len() - value.len());
    return Entry {
        dashes,
        content: Content::Key {
            name,
            column,
            value: value.trim_end(),
            value_offset,
        },
    };
}

/// What:
///  Remove a trailing ` # comment` from a value,
///  leaving quoted text alone.
/// Why:
///  A `#` starts a comment only after whitespace and outside quotes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withoutComment(value: string): string;
/// ```
pub(crate) fn without_comment(value: &str) -> &str {
    let bytes: &[u8] = value.as_bytes();
    let mut quote: u8 = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if quote != 0 {
            if *byte == quote {
                quote = 0;
            }
        } else if *byte == b'"' || *byte == b'\'' {
            quote = *byte;
        } else if *byte == b'#' && (index == 0 || bytes[index - 1] == b' ') {
            return value[..index].trim_end();
        }
    }
    return value.trim_end();
}

/// Line reading controls stay outside release artifacts.
#[cfg(test)]
#[path = "workflow_lines_tests.rs"]
mod tests;
