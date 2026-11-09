//! Controls for the new-side span derivation,
//! each one a differential against real Git's `diff -U0` for the same file:
//! the relation under test is that the three-line-context patch yields the spans a zero-context diff's new-side headers
//! name.

use super::*;
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// Lines in the fixture file;
/// large enough that every edit below sits apart from the others.
const FILE_LINES: usize = 120;

/// The numbered lines of the fixture file.
fn numbered(count: usize) -> String {
    return (1..=count).map(|number| return format!("line {number}\n")).collect::<String>();
}

/// The spans a zero-context diff's new-side headers name:
/// `+c,d` is lines `c` to `c + d - 1`,
/// and `+c,0` is the line
/// after `c`,
/// where the removed lines stood.
fn headers_spans(zero_context_diff: &str) -> Vec<NewSpan> {
    let mut spans: Vec<NewSpan> = Vec::new();
    for line in zero_context_diff.split('\n') {
        let Some(rest) = line.strip_prefix("@@ -") else {
            continue;
        };
        let new: &str = rest.split(' ').nth(1).and_then(|side| return side.strip_prefix('+')).expect("new side");
        let (start_text, length_text) = new.split_once(',').unwrap_or((new, "1"));
        let start: u64 = start_text.parse::<u64>().expect("start");
        let length: u64 = length_text.parse::<u64>().expect("length");
        spans.push(if length == 0 { NewSpan { first: start + 1, last: start + 1 } } else { NewSpan { first: start, last: start + length - 1 } });
    }
    return spans;
}

/// A repository whose last commit changed `name` from `before` to `after`.
fn changed_repository(test: &str, name: &str, before: &str, after: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(test);
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::create_dir_all(root.join(name).parent().expect("parent")).expect("directories");
    std::fs::write(root.join(name), before).expect("write before");
    git(root.as_path(), &["add", "--", name]);
    git(root.as_path(), &["commit", "--quiet", "--message=before"]);
    std::fs::write(root.join(name), after).expect("write after");
    git(root.as_path(), &["add", "--", name]);
    git(root.as_path(), &["commit", "--quiet", "--message=after"]);
    return (parent, root);
}

/// Both sides of the differential for one file of the last commit.
fn both_sides(root: &Path, name: &str) -> (Vec<NewSpan>, Vec<NewSpan>) {
    let patch: String = git_text(root, &["show", "--format=", "--patch", "HEAD"]);
    let patch_lines: Vec<&[u8]> = patch.as_bytes().split(is_line_feed_byte).collect();
    let zero: String = git_text(root, &["diff", "-U0", "HEAD^", "HEAD", "--", name]);
    return (new_spans(patch_lines.as_slice(), name.as_bytes()), headers_spans(zero.as_str()));
}

/// Replacement,
/// insertion,
/// deletion,
/// an edit at the first line and an append all derive the same spans.
#[test]
fn edits_derive_the_zero_context_spans() {
    let before: String = numbered(FILE_LINES);
    let mut lines: Vec<String> = before.lines().map(String::from).collect();
    lines[4] = String::from("replaced");
    lines.insert(20, String::from("inserted"));
    lines.drain(40..43);
    lines[0] = String::from("first replaced");
    lines.push(String::from("appended"));
    let after: String = format!("{}\n", lines.join("\n"));
    let (parent, root) = changed_repository("patch-edits", "src/a.ts", before.as_str(), after.as_str());
    let (derived, expected) = both_sides(root.as_path(), "src/a.ts");
    assert!(expected.len() > 1);
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// Two edits close enough for Git to merge into one three-line-context hunk stay two spans.
#[test]
fn near_edits_in_one_hunk_stay_separate() {
    let before: String = numbered(FILE_LINES);
    let mut lines: Vec<String> = before.lines().map(String::from).collect();
    lines[10] = String::from("one");
    lines[13] = String::from("two");
    let after: String = format!("{}\n", lines.join("\n"));
    let (parent, root) = changed_repository("patch-near", "a.rs", before.as_str(), after.as_str());
    let patch: String = git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]);
    assert_eq!(patch.matches("\n@@ ").count() + usize::from(patch.starts_with("@@ ")), 1, "one patch hunk");
    let (derived, expected) = both_sides(root.as_path(), "a.rs");
    assert_eq!(derived.len(), 2);
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// A file without a trailing newline and a path with a space still match their section.
#[test]
fn unusual_endings_and_spaces_match() {
    let before: String = numbered(FILE_LINES);
    let after: String = format!("{}no newline", before.replace("line 60\n", "sixty\n"));
    let (parent, root) = changed_repository("patch-space", "dir name/b c.mjs", before.as_str(), after.as_str());
    let (derived, expected) = both_sides(root.as_path(), "dir name/b c.mjs");
    assert!(!expected.is_empty());
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// A new file is one span over all its lines.
#[test]
fn a_new_file_is_one_span_over_all_lines() {
    let parent: PathBuf = fixture("patch-new");
    let root: PathBuf = repository(parent.as_path(), "repo");
    let count: usize = 5;
    std::fs::write(root.join("new.rs"), numbered(count)).expect("write");
    git(root.as_path(), &["add", "--", "new.rs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=new"]);
    let (derived, expected) = both_sides(root.as_path(), "new.rs");
    assert_eq!(derived, vec![NewSpan { first: 1, last: u64::try_from(count).expect("small") }]);
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// A rename with an edit is read under its new name,
/// with the spans a rename-detecting zero-context diff shows.
#[test]
fn a_rename_is_read_under_its_new_name() {
    let before: String = numbered(FILE_LINES);
    let after: String = before.replace("line 60\n", "sixty\n");
    let parent: PathBuf = fixture("patch-rename");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("old.ts"), before.as_str()).expect("write");
    git(root.as_path(), &["add", "--", "old.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=old"]);
    git(root.as_path(), &["mv", "old.ts", "renamed.ts"]);
    std::fs::write(root.join("renamed.ts"), after.as_str()).expect("write");
    git(root.as_path(), &["add", "--", "renamed.ts"]);
    git(root.as_path(), &["commit", "--quiet", "--message=rename"]);
    let patch: String = git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]);
    assert!(patch.contains("rename to renamed.ts"), "{patch}");
    let patch_lines: Vec<&[u8]> = patch.as_bytes().split(is_line_feed_byte).collect();
    let zero: String = git_text(root.as_path(), &["diff", "-U0", "-M", "HEAD^", "HEAD"]);
    let derived: Vec<NewSpan> = new_spans(patch_lines.as_slice(), b"renamed.ts");
    assert!(!derived.is_empty());
    assert_eq!(derived, headers_spans(zero.as_str()));
    remove(parent.as_path());
}

/// An insertion early in a file moves every later span down with it,
/// which the old-side reading missed.
#[test]
fn spans_after_an_insertion_are_in_new_file_lines() {
    let before: String = numbered(FILE_LINES);
    let inserted: usize = 7;
    let mut lines: Vec<String> = before.lines().map(String::from).collect();
    for number in 0..inserted {
        lines.insert(2, format!("inserted {number}"));
    }
    lines[100] = String::from("late edit");
    let after: String = format!("{}\n", lines.join("\n"));
    let (parent, root) = changed_repository("patch-shift", "s.ts", before.as_str(), after.as_str());
    let (derived, expected) = both_sides(root.as_path(), "s.ts");
    assert_eq!(derived, expected);
    let late: NewSpan = *derived.last().expect("a late span");
    assert_eq!(after.lines().nth(usize::try_from(late.first).expect("fits") - 1), Some("late edit"));
    remove(parent.as_path());
}

/// A path the patch does not mention,
/// and a mode-only change,
/// have no spans.
#[test]
fn absent_paths_and_hunkless_sections_have_no_spans() {
    let before: String = numbered(FILE_LINES);
    let (parent, root) = changed_repository("patch-absent", "a.ts", before.as_str(), format!("{before}x\n").as_str());
    let patch: String = git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]);
    let patch_lines: Vec<&[u8]> = patch.as_bytes().split(is_line_feed_byte).collect();
    assert!(new_spans(patch_lines.as_slice(), b"other.ts").is_empty());
    assert!(new_spans(&[], b"a.ts").is_empty());
    let hunkless: Vec<&[u8]> = vec![b"diff --git a/m.ts b/m.ts".as_slice(), b"old mode 100644".as_slice(), b"new mode 100755".as_slice()];
    assert!(new_spans(hunkless.as_slice(), b"m.ts").is_empty());
    remove(parent.as_path());
}

/// Names Git quotes in its patch header:
/// a quote it escapes and a non-ASCII byte it escapes,
/// both of which `core.quotePath` quotes by default.
const QUOTED_NAMES: [&str; 2] = ["quote\".rs", "caf\u{e9}.rs"];

/// A changed path Git quotes in the patch header is still that file's section:
/// the spans are the changed lines',
/// as the zero-context diff names them,
/// never none.
#[test]
fn a_quoted_path_derives_its_changed_lines() {
    let before: String = numbered(FILE_LINES);
    let mut lines: Vec<String> = before.lines().map(String::from).collect();
    lines.insert(FILE_LINES / 2, String::from("inserted"));
    lines[0] = String::from("first replaced");
    let after: String = format!("{}\n", lines.join("\n"));
    for name in QUOTED_NAMES {
        let (parent, root) = changed_repository("patch-quoted", name, before.as_str(), after.as_str());
        let patch: String = git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]);
        assert!(patch.contains("diff --git \""), "{name} is quoted in the header: {patch}");
        let (derived, expected) = both_sides(root.as_path(), name);
        assert!(!expected.is_empty(), "{name} changed lines: {expected:?}");
        assert_eq!(derived, expected);
        remove(parent.as_path());
    }
}
