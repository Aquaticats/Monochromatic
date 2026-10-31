//! What:
//!  Tests for the `JSON.stringify(x, null, 2)` rendering.
//! Why:
//!  The expected texts are what Node prints for the same arrays, including the empty array and escapes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('renderResolvedJson', () => { /* empty, one, many, escapes */ });
//! ```

use crate::render::render_resolved_json;

/// Build owned strings from literals.
fn owned(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| return (*item).to_string()).collect();
}

/// An empty list is `[]` plus a newline, like `JSON.stringify([], null, 2)`.
#[test]
fn empty_list() {
    assert_eq!(render_resolved_json(&[]), "[]\n");
}

/// One and several entries use two-space indentation, commas after all but the last, and a final newline.
#[test]
fn one_and_many_entries() {
    assert_eq!(render_resolved_json(&owned(&["chrome 156"])), "[\n  \"chrome 156\"\n]\n");
    assert_eq!(
        render_resolved_json(&owned(&["and_chr 156", "node 26.13.0"])),
        "[\n  \"and_chr 156\",\n  \"node 26.13.0\"\n]\n"
    );
}

/// Quotes, backslashes and control characters escape the way `JSON.stringify` escapes them.
#[test]
fn escapes_match_json_stringify() {
    assert_eq!(render_resolved_json(&owned(&["a\"b\\c\nd\u{1}e"])), "[\n  \"a\\\"b\\\\c\\nd\\u0001e\"\n]\n");
}
