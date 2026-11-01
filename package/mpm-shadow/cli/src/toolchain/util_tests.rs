//! Unit tests of the shared named helpers.

use std::path::Path;

use super::fixtures::sample_lock;
use super::util::{all_ascii_digits, contains_any, ends_with_any, file_name_os, file_name_text, find_tool, number_or_zero, starts_with_ascii_digit};

#[test]
fn digit_predicates_accept_only_ascii_digits() {
    assert!(all_ascii_digits("0123"));
    assert!(all_ascii_digits(""));
    assert!(!all_ascii_digits("12a"));
    assert!(!all_ascii_digits("\u{0663}"));
    assert!(starts_with_ascii_digit("4.1"));
    assert!(!starts_with_ascii_digit("v4"));
    assert!(!starts_with_ascii_digit(""));
}

#[test]
fn any_of_predicates_match_substrings_and_endings() {
    assert!(contains_any("tool-linux-x64", &["darwin", "linux"]));
    assert!(!contains_any("tool-linux-x64", &["darwin", "windows"]));
    assert!(!contains_any("tool", &[]));
    assert!(ends_with_any("a.tar.gz", &[".zip", ".gz"]));
    assert!(!ends_with_any("a.tar.gz", &[".zip", ".xz"]));
}

#[test]
fn a_tool_is_found_by_its_id() {
    let lock = sample_lock();
    assert_eq!(find_tool(&lock.tools, "node").map(|tool| return tool.version.as_str()), Some("1.2.3"));
    assert!(find_tool(&lock.tools, "absent").is_none());
}

#[test]
fn file_names_convert_and_missing_ones_are_empty() {
    assert_eq!(file_name_text(Path::new("/a/b/c.txt")), "c.txt");
    assert_eq!(file_name_text(Path::new("/")), "");
    assert_eq!(file_name_os(Path::new("/a/b")).to_string_lossy(), "b");
    assert_eq!(number_or_zero("17"), 17);
    assert_eq!(number_or_zero("rc1"), 0);
}
