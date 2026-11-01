//! What:
//!  The comparison of two staged-entry listings and the warning over real index files.
//! Why:
//!  A comparison that misses a removed entry, or that flags a path the landing meant to rewrite, would either hide
//!  a loss or cry wolf on every commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(pathsChangedOutside(pre, post, [])).toEqual(['b.txt']);
//! ```

use super::{paths_changed_outside, warn_when_entries_moved};
use crate::test_support::{REAL_GIT, fixture, git_context, remove, repository};
use std::path::Path;
use std::process::Command;

/// One listing record.
fn record(mode: &str, object: &str, stage: u8, path: &str) -> Vec<u8> {
    return format!("{mode} {object} {stage}\t{path}\0").into_bytes();
}

/// A listing from records.
fn listing(records: &[Vec<u8>]) -> Vec<u8> {
    return records.concat();
}

/// Entries that agree give no path; a changed, added or removed entry gives its path, sorted.
#[test]
fn a_changed_added_or_removed_entry_names_its_path() {
    let a = record("100644", "aaaa", 0, "a.txt");
    let b = record("100644", "bbbb", 0, "b.txt");
    let same = listing(&[a.clone(), b.clone()]);
    assert_eq!(paths_changed_outside(&same, &same, &[]), Vec::<Vec<u8>>::new());
    assert_eq!(paths_changed_outside(&[], &[], &[]), Vec::<Vec<u8>>::new());
    let changed = listing(&[a.clone(), record("100644", "cccc", 0, "b.txt")]);
    assert_eq!(paths_changed_outside(&same, &changed, &[]), vec![b"b.txt".to_vec()]);
    let removed = listing(std::slice::from_ref(&a));
    assert_eq!(paths_changed_outside(&same, &removed, &[]), vec![b"b.txt".to_vec()]);
    let added = listing(&[a.clone(), b.clone(), record("100644", "dddd", 0, "0.txt")]);
    assert_eq!(paths_changed_outside(&same, &added, &[]), vec![b"0.txt".to_vec()]);
    let mode = listing(&[a.clone(), record("100755", "bbbb", 0, "b.txt")]);
    assert_eq!(paths_changed_outside(&same, &mode, &[]), vec![b"b.txt".to_vec()]);
    let staged_twice = listing(&[a, b, record("100644", "eeee", 2, "b.txt")]);
    assert_eq!(paths_changed_outside(&same, &staged_twice, &[]), vec![b"b.txt".to_vec()]);
}

/// A path the landing is meant to rewrite is never reported, and each path is reported once.
#[test]
fn allowed_paths_are_not_reported_and_others_once() {
    let pre = listing(&[record("100644", "aaaa", 0, "a.txt"), record("100644", "bbbb", 0, "b.txt")]);
    let post = listing(&[
        record("100644", "1111", 0, "a.txt"),
        record("100644", "2222", 0, "b.txt"),
        record("100644", "3333", 2, "b.txt"),
    ]);
    assert_eq!(paths_changed_outside(&pre, &post, &[b"a.txt".to_vec()]), vec![b"b.txt".to_vec()]);
    assert_eq!(
        paths_changed_outside(&pre, &post, &[b"a.txt".to_vec(), b"b.txt".to_vec()]),
        Vec::<Vec<u8>>::new()
    );
    assert_eq!(paths_changed_outside(b"junk without a tab\0", b"", &[]), Vec::<Vec<u8>>::new());
}

/// Paths that are not UTF-8 compare as bytes.
#[test]
fn paths_compare_as_bytes() {
    let pre = b"100644 aaaa 0\tt\xffp.txt\0".to_vec();
    let post = b"100644 bbbb 0\tt\xffp.txt\0".to_vec();
    assert_eq!(paths_changed_outside(&pre, &post, &[]), vec![b"t\xffp.txt".to_vec()]);
}

/// Run Git in the repository with the given index file.
fn git_index(repo: &Path, index: &Path, arguments: &[&str]) {
    let output = Command::new(REAL_GIT)
        .current_dir(repo)
        .env("GIT_INDEX_FILE", index)
        .args(arguments)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
}

/// Over real index files the warning compares the whole index and does not fail on an unreadable one.
#[test]
fn real_indexes_are_compared_and_an_unreadable_one_is_tolerated() {
    let root = fixture("landing-invariant");
    let repo = repository(root.as_path(), "repo");
    let pre = root.join("pre.index");
    let post = root.join("post.index");
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
    git_index(repo.as_path(), pre.as_path(), &["add", "a.txt"]);
    std::fs::copy(&pre, &post).expect("index copy");
    std::fs::write(repo.join("b.txt"), b"two\n").expect("b.txt");
    git_index(repo.as_path(), post.as_path(), &["add", "b.txt"]);
    let context = git_context();
    // b.txt entered the index after the pre-landing copy, so it is named unless it is allowed.
    let warned = warn_when_entries_moved(&context, repo.as_path(), pre.as_path(), post.as_path(), &[])
        .expect("a moved entry is warned about");
    assert!(warned.starts_with("The landing changed staged entries outside the paths it committed: "), "{warned}");
    assert!(warned.contains("b.txt"), "{warned}");
    assert!(!warned.contains("a.txt"), "{warned}");
    assert!(warned.contains("cli-git-audit.jsonl"), "{warned}");
    assert_eq!(
        warn_when_entries_moved(&context, repo.as_path(), pre.as_path(), post.as_path(), &[b"b.txt".to_vec()]),
        None
    );
    assert_eq!(
        warn_when_entries_moved(&context, repo.as_path(), pre.as_path(), pre.as_path(), &[]),
        None
    );
    // A missing index file reads as an empty index, so every entry of the other one is new.
    let from_nothing = warn_when_entries_moved(&context, repo.as_path(), root.join("absent.index").as_path(), post.as_path(), &[])
        .expect("every entry is new against an empty index");
    assert!(from_nothing.contains("a.txt") && from_nothing.contains("b.txt"), "{from_nothing}");
    // An index file that is not an index cannot be listed, which is tolerated and says nothing.
    let corrupt = root.join("corrupt.index");
    std::fs::write(&corrupt, b"this is not an index").expect("corrupt index");
    assert_eq!(
        warn_when_entries_moved(&context, repo.as_path(), corrupt.as_path(), post.as_path(), &[]),
        None
    );
    assert_eq!(
        warn_when_entries_moved(&context, repo.as_path(), pre.as_path(), corrupt.as_path(), &[]),
        None
    );
    remove(root.as_path());
}
