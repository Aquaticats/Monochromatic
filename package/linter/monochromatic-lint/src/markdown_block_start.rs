//! What:
//!  Conservative block-start detection before inserting prose line breaks.
//! Why:
//!  A newline that replaces a space can still turn inline text into a heading,
//!  list,
//!  quote or code block.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Decline a break when the following source would acquire block syntax at line start.
//! ```

/// Treat EOF and ordinary Markdown spacing as a marker boundary.
fn boundary(byte: Option<u8>) -> bool {
    // None is EOF; byte values otherwise describe ASCII syntax, not Unicode text characters.
    let Some(value) = byte else {
        return true;
    };
    return value == b' ' || value == b'\t' || value == b'\n' || value == b'\r';
}

/// What:
///  Check only as far as a candidate marker requires,
///  without rescanning every remaining line suffix.
/// Why:
///  Long prose lines with many punctuation characters must not force repeated full-line scans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startsBlockConstruct(source, at): boolean;
/// ```
pub(crate) fn starts_block_construct(source: &str, at: usize) -> bool {
    // The text scanner supplies a UTF-8 boundary inside the source, so this slice cannot split a character.
    // The standard trim drops the horizontal spacing before a candidate marker; no index is stepped by hand.
    // Bytes (u8, not char) suffice afterwards because every Markdown marker is ASCII.
    let line: &[u8] = source[at..].trim_start_matches([' ', '\t']).as_bytes();
    // A missing/empty next line cannot introduce a block.
    let Some(first): Option<u8> = line.first().copied() else {
        return false;
    };
    // Newline bytes match no marker or digit branch and reach the ordinary false result.
    if first == b'>' || first == b'<' || first == b'|' {
        return true;
    }
    // Count the first written marker's run; non-ASCII leading bytes never match the ASCII marker catalog.
    // Walking the slice ends at its last byte whatever happens to the counter, unlike a hand-stepped index.
    let mut width: usize = 0;
    for byte in line {
        if *byte != first {
            break;
        }
        width += 1;
    }
    let after: Option<u8> = line.get(width).copied();
    if first == b'#' && width <= 6 && boundary(after) {
        return true;
    }
    if (first == b'`' || first == b'~') && width >= 3 {
        return true;
    }
    if (first == b'-' || first == b'+' || first == b'*') && width == 1 && boundary(after) {
        return true;
    }
    if first == b'-' || first == b'=' || first == b'_' || first == b'*' {
        // Uniform rule lines contain no further prose break points, so this scan cannot repeat per punctuation.
        for current in &line[width..] {
            if *current == b'\n' || *current == b'\r' {
                return true;
            }
            if *current != first && *current != b' ' && *current != b'\t' {
                return false;
            }
        }
        return true;
    }
    // An ordered marker needs digits, a delimiter and a separating boundary.
    let mut digits: usize = 0;
    for byte in line {
        if !byte.is_ascii_digit() {
            break;
        }
        digits += 1;
    }
    // Without digits the first byte would be read as the delimiter, so a leading '.' or ')' is not a marker.
    if digits == 0 {
        return false;
    }
    let Some(delimiter): Option<u8> = line.get(digits).copied() else {
        return false;
    };
    return (delimiter == b'.' || delimiter == b')') && boundary(line.get(digits + 1).copied());
}

/// Verify both block starters and lookalike prose without requiring a complete Markdown parse.
#[cfg(test)]
#[path = "markdown_block_start_tests.rs"]
mod tests;
