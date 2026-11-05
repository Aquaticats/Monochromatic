//! What:
//!  Controls of the top doc reader.
//! Why:
//!  The file rule trusts this reader to find the list and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('reads entries after the marker', () => expect(readTopDoc(text).entries.map(e => e.name)).toEqual([...]));
//! ```

use super::{LIST_MARKER, read_top_doc};

/// Entries after the marker are read in order with their lines,
/// and free text before the marker and after the list is ignored.
#[test]
fn entries_after_the_marker_are_read_in_order() {
    let text: String = format!(
        "//! What: this folder saves files.\n//!\n//! {LIST_MARKER}\n//!\n//! - `first.rs`: writes a file.\n//!   More words about it.\n//! - `second.rs`: saves two.\n//!\n//! Closing words that end the list.\n//! - `ignored.rs`: after the list.\n\nmod first;\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(3));
    let seen: Vec<(&str, usize)> = doc
        .entries
        .iter()
        .map(|entry| return (entry.name.as_str(), entry.line))
        .collect::<Vec<(&str, usize)>>();
    assert_eq!(seen, [("first.rs", 5), ("second.rs", 7)]);
    assert_eq!(doc.first_line, 1);
}

/// A doc without the marker has no list even when it has bullet lines.
#[test]
fn bullets_without_the_marker_are_no_list() {
    let doc = read_top_doc("//! Words.\n//! - `first.rs`: something.\n");
    assert_eq!(doc.marker_line, None);
    assert!(doc.entries.is_empty());
    assert_eq!(doc.first_line, 1);
}

/// Only the leading doc lines count:
/// comments and inner attributes do not end the doc,
/// code does.
#[test]
fn only_the_leading_doc_block_counts() {
    let text: String = format!(
        "// plain\n#![allow(dead_code)]\n//! {LIST_MARKER}\n//! - `a.rs`: x\n\nfn code() {{}}\n//! {LIST_MARKER}\n//! - `late.rs`: y\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(3));
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.first_line, 3);
}

/// A file with no doc reports the first line,
/// and an entry line without backticks ends the list.
#[test]
fn no_doc_and_unquoted_entries() {
    assert_eq!(read_top_doc("fn code() {}\n").first_line, 1);
    let text: String = format!(
        "//! {LIST_MARKER}\n//! - `a.rs`: x\n//! - b.rs: unquoted\n//! - `c.rs`: not read\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.entries.len(), 1);
}
