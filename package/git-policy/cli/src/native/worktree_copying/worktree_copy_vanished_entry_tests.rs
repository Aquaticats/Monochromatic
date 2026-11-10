//! Controls for the question a walk of a live checkout asks about each entry:
//! is it still there,
//! and is every other failure still a failure.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// An absent entry is an answer:
/// the walk has nothing to copy there.
#[test]
fn an_absent_entry_is_no_metadata_and_no_failure() {
    let root: PathBuf = fixture("vanished-absent");
    assert!(lstat_or_absent(root.join("never-made").as_path()).expect("absence is an answer").is_none());
    std::fs::write(root.join("made"), b"x").expect("made");
    assert!(lstat_or_absent(root.join("made").as_path()).expect("presence is an answer").is_some());
    remove(root.as_path());
}

/// An absent directory lists no names,
/// and a present one lists the bytes its entries have.
#[test]
fn an_absent_directory_lists_nothing_and_a_present_one_lists_its_names() {
    let root: PathBuf = fixture("vanished-names");
    assert!(names_or_absent(root.join("never-made").as_path()).expect("absence is an answer").is_none());
    std::fs::create_dir_all(root.join("present")).expect("present");
    std::fs::write(root.join("present/b"), b"b").expect("b");
    std::fs::write(root.join("present/a"), b"a").expect("a");
    let mut names: Vec<Vec<u8>> = names_or_absent(root.join("present").as_path()).expect("presence is an answer").expect("a listing");
    names.sort();
    assert_eq!(names, vec![b"a".to_vec(), b"b".to_vec()]);
    remove(root.as_path());
}

/// A path that is not a directory at all is a failure with its cause,
/// not the answer that the entry went away.
#[test]
fn a_path_that_is_not_a_directory_is_still_a_failure() {
    let root: PathBuf = fixture("vanished-not-a-directory");
    std::fs::write(root.join("file"), b"x").expect("file");
    let failure: WorktreeCopyError = names_or_absent(root.join("file").as_path()).expect_err("a file is no listing");
    assert!(failure.0.contains("Not a directory"), "{}", failure.0);
    remove(root.as_path());
}
