//! Escaping of client-chosen title and app-ID text on the one-line answer.

/// Brings the escaping rule into scope.
use super::*;

/// Plain titles, including spaces and non-ASCII text, pass through unchanged.
#[test]
fn plain_text_is_unchanged() {
    assert_eq!(escape_text("Untitled - Editor"), "Untitled - Editor");
    assert_eq!(escape_text("日記"), "日記");
    assert_eq!(escape_text(""), "");
}

/// Line breaks become two-character escapes, so the answer stays on one line.
#[test]
fn line_breaks_are_escaped() {
    let escaped = escape_text("first\nsecond\r\nthird");
    assert_eq!(escaped, "first\\nsecond\\r\\nthird");
    assert!(!escaped.contains('\n'));
    assert!(!escaped.contains('\r'));
}

/// A literal backslash followed by `n` is distinguishable from a line feed.
#[test]
fn backslashes_are_escaped_so_the_mapping_is_reversible() {
    assert_eq!(escape_text("C:\\new"), "C:\\\\new");
    assert_eq!(escape_text("\\\n"), "\\\\\\n");
    assert_ne!(escape_text("\\n"), escape_text("\n"));
}
