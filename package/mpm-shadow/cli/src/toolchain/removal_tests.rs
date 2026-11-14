//! Unit tests of removing one entry only when it is still the entry that was judged:
//! the three outcomes,
//! and the sentence for a file that could not be put back,
//! which must name the path it is at and the command that returns it.

/// Imports `fs` from `std`.
use std::fs;
/// Imports `symlink` from `std::os::unix::fs`.
use std::os::unix::fs::symlink;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `TempDir` from `tempfile`.
use tempfile::TempDir;

/// Imports what this module uses from `super::removal`.
use super::removal::{Removal, could_not_put_back, judged_of, remove_if_still};

/// A disposable directory for one test.
fn scratch() -> TempDir {
    return crate::test_scratch::scratch();
}

/// A link and its target in `directory`.
fn a_link(directory: &Path) -> (PathBuf, PathBuf) {
    let target: PathBuf = directory.join("target");
    fs::write(&target, "the judged target").expect("write the target");
    let link: PathBuf = directory.join("link");
    symlink(&target, &link).expect("make the link");
    return (link, target);
}

#[test]
fn the_entry_that_was_judged_is_removed() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");

    assert_eq!(remove_if_still(&link, judged).expect("the removal runs"), Removal::Retired);
    assert!(fs::symlink_metadata(&link).is_err());
}

#[test]
fn an_entry_that_is_already_gone_is_not_an_error() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");
    fs::remove_file(&link).expect("another install removed it first");

    assert_eq!(remove_if_still(&link, judged).expect("the removal runs"), Removal::Gone);
}

#[test]
fn a_file_that_replaced_the_judged_one_is_put_back() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");
    fs::remove_file(&link).expect("take the judged link away");
    let persons_text = "#!/bin/sh\necho a person's own script\n";
    fs::write(&link, persons_text).expect("write the person's file");

    assert_eq!(remove_if_still(&link, judged).expect("the removal runs"), Removal::Replaced);
    assert_eq!(fs::read_to_string(&link).expect("the person's file is back in place"), persons_text);
}

#[test]
fn a_put_back_that_cannot_happen_names_the_path_the_file_is_at_and_the_command_that_returns_it() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    fs::remove_file(&link).expect("take the judged link away");
    fs::create_dir(&link).expect("a directory took the link's name");
    let put_back = fs::rename(directory.path().join("nothing-there"), &link).expect_err("nothing to move");
    let temporary: PathBuf = directory.path().join(".link.4242.0.retired");

    let sentence = could_not_put_back(&link, &temporary, &put_back);

    assert!(sentence.contains(&format!("{} instead of {}", temporary.display(), link.display())), "{sentence}");
    assert!(sentence.contains(&format!("`mv -- {} {}`", temporary.display(), link.display())), "{sentence}");
    assert!(sentence.contains("cannot be put back"), "{sentence}");
}

#[test]
fn a_put_back_sentence_quotes_a_path_a_shell_would_split() {
    let put_back = fs::rename("/nowhere", "/nowhere-either").expect_err("nothing to move");
    let sentence = could_not_put_back(Path::new("/a folder/a file"), Path::new("/a folder/.a file.1.0.retired"), &put_back);

    assert!(sentence.contains("`mv -- '/a folder/.a file.1.0.retired' '/a folder/a file'`"), "{sentence}");
}

#[test]
fn a_directory_that_took_the_judged_entrys_name_is_put_back_too() {
    let directory = scratch();
    let entry: PathBuf = directory.path().join("entry");
    fs::create_dir(&entry).expect("make the judged directory");
    let judged = judged_of(&entry).expect("the directory is inspected");
    // A person's own directory takes the name. It is made under another name first, so it can never be the entry whose
    // numbers were judged (a directory number can be reused as soon as its directory is gone).
    let theirs: PathBuf = directory.path().join("their-own");
    fs::create_dir(&theirs).expect("make the person's directory");
    fs::remove_dir(&entry).expect("the judged directory goes");
    fs::rename(&theirs, &entry).expect("the person's directory takes the name");

    assert_eq!(remove_if_still(&entry, judged).expect("the removal runs"), Removal::Replaced);
    assert!(fs::symlink_metadata(&entry).expect("still there").is_dir());
}

#[test]
fn a_removal_whose_hidden_names_are_all_taken_refuses_to_move_the_entry_aside() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");
    // Kept files of earlier runs at every hidden name a removal may reach for.
    let persons_text = "a person's own kept file\n";
    for attempt in 0..64 {
        fs::write(directory.path().join(format!(".link.{}.{attempt}.retired", std::process::id())), persons_text).expect("keep a file");
    }
    fs::write(directory.path().join(format!(".link.{}.retired", std::process::id())), persons_text).expect("keep the fallback file");

    let outcome = remove_if_still(&link, judged);

    assert!(outcome.is_err(), "with every hidden name taken the entry must not be moved aside at all: {outcome:?}");
    assert!(fs::symlink_metadata(&link).is_ok(), "the entry that was judged is still where it was");
    let fallback: PathBuf = directory.path().join(format!(".link.{}.retired", std::process::id()));
    assert_eq!(fs::read_to_string(&fallback).expect("the kept file stays"), persons_text);
}

#[test]
fn a_put_back_never_writes_over_something_that_took_the_path() {
    let directory = scratch();
    let (path, _target) = a_link(directory.path());
    let moved: PathBuf = directory.path().join(".link.4242.0.retired");
    fs::write(&moved, "what stood at the path first\n").expect("the moved entry");
    // Something newer stands where the entry was, as a person's or another install's write would leave it.
    fs::remove_file(&path).expect("the judged link goes");
    let newer = "#!/bin/sh\necho what took the path\n";
    fs::write(&path, newer).expect("what took the path");

    let outcome = super::removal::put_back(&path, &moved);

    assert!(outcome.is_err(), "the put-back must refuse rather than write over it: {outcome:?}");
    assert_eq!(fs::read_to_string(&path).expect("what took the path stays"), newer);
    assert_eq!(fs::read_to_string(&moved).expect("and the moved entry is kept"), "what stood at the path first\n");
    let sentence = outcome.expect_err("refused");
    assert!(sentence.contains(&moved.display().to_string()), "the sentence names where it is kept: {sentence}");
    assert!(sentence.contains("nothing removes it"), "the sentence says who removes it: {sentence}");
}
