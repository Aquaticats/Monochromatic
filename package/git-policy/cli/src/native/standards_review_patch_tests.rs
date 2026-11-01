//! Controls for the old-range derivation, each one a differential against real Git's `diff -U0` for the same file:
//! the relation under test is that the three-line-context patch yields the ranges a zero-context diff prints.

use super::*;
use crate::test_support::{fixture, git, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// Lines in the fixture file; large enough that every edit below sits apart from the others.
const FILE_LINES: usize = 120;

/// The numbered lines of the fixture file.
fn numbered(count: usize) -> String {
    return (1..=count).map(|number| return format!("line {number}\n")).collect::<String>();
}

/// The script's reading of a zero-context diff: every `@@ -start[,length]` header, with an absent length meaning 1.
fn script_ranges(zero_context_diff: &str) -> Vec<OldRange> {
    let mut ranges: Vec<OldRange> = Vec::new();
    for line in zero_context_diff.split('\n') {
        let Some(rest) = line.strip_prefix("@@ -") else {
            continue;
        };
        let old: &str = rest.split(' ').next().expect("old side");
        let (start, length) = old.split_once(',').unwrap_or((old, "1"));
        ranges.push(OldRange {
            start: start.parse::<u64>().expect("start"),
            length: length.parse::<u64>().expect("length"),
        });
    }
    return ranges;
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
fn both_sides(root: &Path, name: &str) -> (Vec<OldRange>, Vec<OldRange>) {
    let patch: String = git_text(root, &["show", "--format=", "--patch", "HEAD"]);
    let patch_lines: Vec<&str> = patch.split('\n').collect();
    let zero: String = git_text(root, &["diff", "-U0", "HEAD^", "HEAD", "--", name]);
    return (old_ranges(patch_lines.as_slice(), name), script_ranges(zero.as_str()));
}

/// Replacement, insertion, deletion, an edit at the first line and an append all derive the same ranges.
#[test]
fn edits_derive_the_zero_context_ranges() {
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

/// Two edits close enough for Git to merge into one three-line-context hunk stay two ranges.
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

/// A new file derives the zero-context diff's `-0,0` range.
#[test]
fn a_new_file_is_a_range_at_zero() {
    let parent: PathBuf = fixture("patch-new");
    let root: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(root.join("new.rs"), numbered(5)).expect("write");
    git(root.as_path(), &["add", "--", "new.rs"]);
    git(root.as_path(), &["commit", "--quiet", "--message=new"]);
    let (derived, expected) = both_sides(root.as_path(), "new.rs");
    assert_eq!(derived, vec![OldRange { start: 0, length: 0 }]);
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// A rename with an edit is read as an addition of the new path, which is what a path-limited `diff -U0` shows.
#[test]
fn a_rename_is_read_as_an_addition() {
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
    let (derived, expected) = both_sides(root.as_path(), "renamed.ts");
    assert_eq!(derived, expected);
    remove(parent.as_path());
}

/// A path the patch does not mention, and a mode-only change, have no ranges.
#[test]
fn absent_paths_and_hunkless_sections_have_no_ranges() {
    let before: String = numbered(FILE_LINES);
    let (parent, root) = changed_repository("patch-absent", "a.ts", before.as_str(), format!("{before}x\n").as_str());
    let patch: String = git_text(root.as_path(), &["show", "--format=", "--patch", "HEAD"]);
    let patch_lines: Vec<&str> = patch.split('\n').collect();
    assert!(old_ranges(patch_lines.as_slice(), "other.ts").is_empty());
    assert!(old_ranges(&[], "a.ts").is_empty());
    let hunkless: Vec<&str> = vec!["diff --git a/m.ts b/m.ts", "old mode 100644", "new mode 100755"];
    assert!(old_ranges(hunkless.as_slice(), "m.ts").is_empty());
    remove(parent.as_path());
}
