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

    assert_eq!(remove_if_still(&link, judged, &|| return true).expect("the removal runs"), Removal::Retired);
    assert!(fs::symlink_metadata(&link).is_err());
}

#[test]
fn an_entry_that_is_already_gone_is_not_an_error() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");
    fs::remove_file(&link).expect("another install removed it first");

    assert_eq!(remove_if_still(&link, judged, &|| return true).expect("the removal runs"), Removal::Gone);
}

#[test]
fn a_file_that_replaced_the_judged_one_is_put_back() {
    let directory = scratch();
    let (link, _target) = a_link(directory.path());
    let judged = judged_of(&link).expect("the link is inspected");
    fs::remove_file(&link).expect("take the judged link away");
    let persons_text = "#!/bin/sh\necho a person's own script\n";
    fs::write(&link, persons_text).expect("write the person's file");

    assert_eq!(remove_if_still(&link, judged, &|| return true).expect("the removal runs"), Removal::Replaced);
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
fn a_directory_that_took_the_judged_entrys_name_is_left_and_said() {
    let directory = scratch();
    let entry: PathBuf = directory.path().join("entry");
    fs::write(&entry, "the launcher mpm-shadow placed\n").expect("write the judged entry");
    let judged = judged_of(&entry).expect("the entry is inspected");
    // A person's directory takes the name. It is not an entry this tool removes, and it cannot be hard-linked, so the
    // removal refuses and leaves it exactly as it is.
    fs::remove_file(&entry).expect("the judged entry goes");
    fs::create_dir(&entry).expect("the person's directory takes the name");

    let outcome = super::removal::remove_if_still(&entry, judged, &|| return true);

    assert!(outcome.is_err(), "a directory is not taken aside: {outcome:?}");
    assert!(fs::symlink_metadata(&entry).expect("still there").is_dir(), "and it is left where it is");
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

    let outcome = remove_if_still(&link, judged, &|| return true);

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

//region The fifth review: the three steps of the removal, and what a kill after each leaves
// A second actor is placed by the code's own seam (`removal::seam`, compiled only in tests), never by a sleep, and a kill
// is the state the code leaves after a step (a hidden name whose maker is gone).

#[test]
fn a_file_that_takes_the_hidden_name_between_the_choice_and_the_link_is_not_replaced() {
    let directory = scratch();
    let (path, _target) = a_link(directory.path());
    let judged = judged_of(&path).expect("the link is inspected");
    let hidden: PathBuf = directory.path().join(format!(".link.{}.0.retired", std::process::id()));
    let persons_text = "a person's file at the hidden name\n";
    let taken: PathBuf = hidden.clone();
    super::removal::seam::set_before_the_move(Some(Box::new(move || {
        fs::write(&taken, persons_text).expect("the second actor's write at the hidden name");
    })));

    let outcome = super::removal::remove_if_still(&path, judged, &|| return true);

    super::removal::seam::set_before_the_move(None);
    assert_eq!(
        fs::read_to_string(&hidden).expect("the person's file is still there"),
        persons_text,
        "the link step replaced a person's file at the hidden name: {outcome:?}"
    );
}

#[test]
fn a_file_rewritten_in_place_after_the_judgement_is_not_removed() {
    let directory = scratch();
    let path: PathBuf = directory.path().join("launcher");
    fs::write(&path, "the launcher mpm-shadow placed\n").expect("write the launcher");
    let judged = judged_of(&path).expect("the launcher is inspected");
    let persons_text = "a person's own content, written in place\n";
    // An edit in place: the same path, the same inode, other bytes.
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    use std::os::unix::fs::MetadataExt;
    let inode_before = fs::symlink_metadata(&path).expect("inspect").ino();
    fs::write(&path, persons_text).expect("the person rewrites the file in place");
    let inode_after = fs::symlink_metadata(&path).expect("inspect").ino();
    assert_eq!(inode_before, inode_after, "the in-place edit keeps the inode, which is the point");

    let outcome = super::removal::remove_if_still(&path, judged, &|| return true);

    assert_eq!(outcome.expect("the removal runs"), super::removal::Removal::Replaced);
    assert_eq!(fs::read_to_string(&path).expect("the person's content is still there"), persons_text);
}

//endregion

//region The seventh review: a replacement at the visible name, an edit in place, and a second name after a kill
// Every second actor is placed by the code's own seam (`removal::seam`, compiled only in tests) or is the file system's
// state, and never by a sleep.

#[test]
fn a_replacement_at_the_visible_name_after_the_move_is_not_unlinked() {
    let directory = scratch();
    let path: PathBuf = directory.path().join("launcher");
    let judged_text = "the launcher mpm-shadow placed\n";
    fs::write(&path, judged_text).expect("write the judged entry");
    let judged = judged_of(&path).expect("the entry is inspected");
    let replacement_text = "a person's file, moved onto the name while the removal held the judged one\n";
    let replacement: PathBuf = directory.path().join("replacement");
    fs::write(&replacement, replacement_text).expect("write the replacement");
    let moved_from: PathBuf = replacement.clone();
    let moved_to: PathBuf = path.clone();
    super::removal::seam::set_after_the_move(Some(Box::new(move || {
        fs::rename(&moved_from, &moved_to).expect("another command moves its file onto the visible name");
    })));

    let outcome = super::removal::remove_if_still(&path, judged, &|| return true);

    super::removal::seam::set_after_the_move(None);
    assert_eq!(outcome.expect("the removal runs"), Removal::Retired, "the entry that was judged went");
    assert_eq!(
        fs::read_to_string(&path).expect("the replacement at the visible name survives the removal"),
        replacement_text,
        "the replacement was unlinked"
    );
}

#[test]
fn an_edit_in_place_whose_writer_put_the_modification_time_back_is_not_removed() {
    /// Imports `MetadataExt` from `std::os::unix::fs`,
    /// for the modification time the second actor puts back.
    use std::os::unix::fs::MetadataExt;
    let directory = scratch();
    let path: PathBuf = directory.path().join("launch-record");
    fs::write(&path, "AAAA").expect("write the judged entry");
    let judged = judged_of(&path).expect("the entry is inspected");
    let before: fs::Metadata = fs::symlink_metadata(&path).expect("read the original time");
    let seconds: u64 = before.mtime().try_into().expect("a time after the epoch");
    let nanoseconds: u32 = before.mtime_nsec().try_into().expect("a nanosecond count");
    let edited: PathBuf = path.clone();
    super::removal::seam::set_before_the_move(Some(Box::new(move || {
        fs::write(&edited, "BBBB").expect("another command edits the bytes, keeping their length");
        let file: fs::File = fs::OpenOptions::new().write(true).open(&edited).expect("open the edited file");
        let kept: std::time::SystemTime = std::time::UNIX_EPOCH + std::time::Duration::new(seconds, nanoseconds);
        file.set_times(fs::FileTimes::new().set_modified(kept)).expect("put the original modification time back");
    })));

    let outcome = super::removal::remove_if_still(&path, judged, &|| return true);

    super::removal::seam::set_before_the_move(None);
    assert_eq!(outcome.expect("the removal runs"), Removal::Replaced, "the edited entry is not the judged one");
    assert_eq!(fs::read_to_string(&path).expect("the edit survives"), "BBBB", "the edit was removed with the entry");
}

//endregion

//region The eighth review: an edit through a descriptor that was opened before the move
// The writer is placed by the caller's last question (`may_remove`), which is the instruction the eighth review used:
// the removal's last judgement comes after it, so an edit made there is seen and put back. A write after that judgement
// is the limit the module doc states, which no order of steps closes.

#[test]
fn an_edit_through_an_open_descriptor_before_the_last_judgement_is_put_back() {
    /// Imports `Write` from `std::io`,
    /// for the descriptor the move does not disturb.
    use std::io::Write;
    let directory = scratch();
    let path: PathBuf = directory.path().join("entry");
    fs::write(&path, "original bytes").expect("write the judged entry");
    let writer: fs::File = fs::OpenOptions::new().write(true).open(&path).expect("keep a writer open");
    let judged = judged_of(&path).expect("judge the entry");

    let outcome = super::removal::remove_if_still(&path, judged, &|| {
        // `Write` is implemented for `&File`, so the shared borrow the question's signature allows still writes.
        let mut through: &fs::File = &writer;
        through.write_all(b"an edit through the descriptor the move did not disturb").expect("write through the open descriptor");
        through.sync_all().expect("flush the edit");
        return true;
    });

    assert_eq!(outcome.expect("the removal runs"), Removal::Replaced, "the edit is not the entry that was judged");
    assert_eq!(
        fs::read_to_string(&path).expect("the edit survives at the visible name"),
        "an edit through the descriptor the move did not disturb"
    );
}

//endregion
