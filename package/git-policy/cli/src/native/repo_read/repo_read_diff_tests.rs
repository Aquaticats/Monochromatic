//! Controls for the list difference,
//! the pathspec forms and the output spellings.

use super::*;

/// A flat entry for a regular file.
fn file(path: &str, id: u8) -> FlatEntry {
    return FlatEntry { path: path.as_bytes().to_vec(), mode: 0o100_644, id: vec![id; 20] };
}

/// Added,
/// deleted,
/// modified and type-changed paths are told apart and come out in path order.
#[test]
fn statuses_follow_the_lists() {
    let old = vec![file("a", 1), file("b", 2), file("c", 3)];
    let mut link = file("c", 4);
    link.mode = 0o120_000;
    let new = vec![file("a", 9), file("b", 2), link, file("d", 5)];
    let changes = diff_sorted(old.as_slice(), new.as_slice()).expect("plain lists");
    let summary: Vec<(String, char)> = changes.iter().map(|change| { return (String::from_utf8(change.path.clone()).expect("UTF-8"), char::from(change.status)); }).collect();
    assert_eq!(summary, vec![(String::from("a"), 'M'), (String::from("c"), 'T'), (String::from("d"), 'A')]);
}

/// A path that is a file on one side and a directory on the other is declined,
/// in both directions.
#[test]
fn a_file_that_became_a_directory_is_declined() {
    let with_file = vec![file("a", 1)];
    let with_directory = vec![file("a/b", 2)];
    assert!(diff_sorted(with_file.as_slice(), with_directory.as_slice()).is_none());
    assert!(diff_sorted(with_directory.as_slice(), with_file.as_slice()).is_none());
}

/// A gitlink on either side is declined.
#[test]
fn a_gitlink_is_declined() {
    let mut module = file("m", 1);
    module.mode = 0o160_000;
    assert!(diff_sorted(&[], &[module.clone()]).is_none());
    assert!(diff_sorted(&[module], &[]).is_none());
}

/// Plain names match the path and what is below it;
/// magic,
/// globs and odd spellings are declined.
#[test]
fn pathspecs_are_literal_or_declined() {
    let filter = PathFilter::parse(&[b"dir", b":(top,literal)x.txt"], true).expect("plain");
    assert!(filter.matches(b"dir"));
    assert!(filter.matches(b"dir/inside"));
    assert!(!filter.matches(b"dirt"));
    assert!(filter.matches(b"x.txt"));
    assert!(PathFilter::parse(&[], true).expect("empty").matches(b"anything"));
    assert!(PathFilter::parse(&[b":/"], false).expect("everything").matches(b"anything"));
    for odd in [&b"*.txt"[..], b"a?", b"[a]", b"a\\b", b":!x", b"/abs", b"dir/", b"a//b", b"./a", b"a/../b", b""] {
        assert!(PathFilter::parse(&[odd], true).is_none(), "{odd:?}");
    }
    assert!(PathFilter::parse(&[b"relative"], false).is_none(), "a relative name below the top level");
    assert!(PathFilter::parse(&[b":/", b":(top,exclude)secret"], true).is_none(), "an exclusion after `:/` is still checked");
    assert!(PathFilter::parse(&[b":/", b"*.txt"], true).is_none(), "a glob after `:/` is still checked");
}

/// The raw and name lines are spelled as Git spells them,
/// with the zero name for an absent side.
#[test]
fn lines_are_spelled_as_git_spells_them() {
    let changes = vec![Change { path: b"p".to_vec(), old_mode: 0, new_mode: 0o100_644, old_id: Vec::new(), new_id: vec![0xab; 20], status: b'A' }];
    let zero = "0".repeat(40);
    let expected = format!(":000000 100644 {zero} {} A\0p\0", "ab".repeat(20));
    assert_eq!(raw_lines(changes.as_slice(), zero.as_str()), expected.as_bytes());
    assert_eq!(name_lines(changes.as_slice()), b"p\0".to_vec());
    assert_eq!(parse_diff_filter("AD"), Some(vec![b'A', b'D']));
    assert!(parse_diff_filter("a").is_none());
    assert!(parse_diff_filter("").is_none());
}
