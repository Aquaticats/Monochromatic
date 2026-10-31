//! What:
//!  Layout probes over the original source:
//! line ending,
//! indent unit,
//! a line's indentation,
//! and whether
//!  a span owns its line.
//! Why:
//!  New elements should look like their neighbours,
//! so edits copy the document's own conventions instead of
//!  imposing fixed ones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function detectEol(source: string): string; function lineIndent(source: string, offset: number): string;
//! ```

/// What:
/// Import the XML whitespace test.
/// Why:
/// Blank-tail detection must use XML's narrow whitespace set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { isXmlSpace } from './chars';
/// ```
use crate::chars::is_xml_space;

/// What:
///  Detect the document's line ending from its first line break.
/// Why:
///  A CRLF document must stay CRLF when new lines are inserted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function detectEol(source: string): '\n' | '\r\n' | '\r';
/// ```
pub fn detect_eol(source: &str) -> &'static str {
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            return "\n";
        }
        if byte == b'\r' {
            if source.as_bytes().get(index + 1) == Some(&b'\n') {
                return "\r\n";
            }
            return "\r";
        }
    }
    return "\n";
}

/// What:
///  Return the start offset of the line containing `offset`.
/// Why:
///  Indentation is measured from the line start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineStart(source: string, offset: number): number;
/// ```
pub fn line_start(source: &str, offset: usize) -> usize {
    let bytes: &[u8] = source.as_bytes();
    let mut index: usize = offset;
    while index > 0 && bytes[index - 1] != b'\n' && bytes[index - 1] != b'\r' {
        index -= 1;
    }
    return index;
}

/// What:
///  Return the run of spaces and tabs that starts the line containing `offset`.
/// Why:
///  A replacement element's later lines should line up under its first line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineIndent(source: string, offset: number): string;
/// ```
pub fn line_indent(source: &str, offset: usize) -> &str {
    let start: usize = line_start(source, offset);
    let bytes: &[u8] = source.as_bytes();
    let mut end: usize = start;
    while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
        end += 1;
    }
    return &source[start..end];
}

/// What:
///  True when only spaces and tabs sit between the line start and `offset`.
/// Why:
///  Removing or aligning an element is line-oriented only when the element begins its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startsOwnLine(source: string, offset: number): boolean;
/// ```
pub fn starts_own_line(source: &str, offset: usize) -> bool {
    let start: usize = line_start(source, offset);
    return line_indent(source, offset).len() == offset - start;
}

/// What:
///  Detect one indentation step from the first indented line that starts with a tag.
/// Why:
///  A new nested element should be indented one step deeper using the document's own width (tabs or spaces).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function detectIndentUnit(source: string): string;
/// ```
pub fn detect_indent_unit(source: &str) -> String {
    let bytes: &[u8] = source.as_bytes();
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index] == b'\n' || bytes[index] == b'\r' {
            let start: usize = index + 1;
            let mut end: usize = start;
            while end < bytes.len() && (bytes[end] == b' ' || bytes[end] == b'\t') {
                end += 1;
            }
            if end > start && end < bytes.len() && bytes[end] == b'<' {
                return source[start..end].to_string();
            }
        }
        index += 1;
    }
    return "  ".to_string();
}

/// What:
///  Return the offset just past the line break that ends the line holding `offset`'s trailing blank run,
///  or `None` when other text follows on that line.
/// Why:
///  Removing a whole-line element must also take its line break,
/// but only when nothing else shares the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function blankTailEnd(source: string, offset: number): number | undefined;
/// ```
pub fn blank_tail_end(source: &str, offset: usize) -> Option<usize> {
    let bytes: &[u8] = source.as_bytes();
    let mut index: usize = offset;
    while index < bytes.len() && is_xml_space(bytes[index]) && bytes[index] != b'\n' && bytes[index] != b'\r' {
        index += 1;
    }
    if index >= bytes.len() {
        return Some(index);
    }
    if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') {
        return Some(index + 2);
    }
    if bytes[index] == b'\n' || bytes[index] == b'\r' {
        return Some(index + 1);
    }
    return None;
}
