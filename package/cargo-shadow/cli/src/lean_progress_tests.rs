//! The tests of the record of where a pass stopped:
//! a position is read back,
//! a list continues after it,
//! and a record that is missing or unreadable means the front of the list.

use std::{fs, path::{Path, PathBuf}};

use crate::lean_progress::{LongPass, Progress, PROGRESS_FILE_NAME, resume_index, state_directory};
use crate::test_scratch::scratch;

/// Three keys in the order a pass walks them.
fn keys() -> Vec<PathBuf> {
    return vec![PathBuf::from("/base/00/aaaa"), PathBuf::from("/base/00/bbbb"), PathBuf::from("/base/01/cccc")];
}

#[test]
fn with_no_record_a_pass_starts_at_the_front() {
    assert_eq!(resume_index(&keys(), None), 0);
}

#[test]
fn a_pass_continues_after_the_key_the_last_run_stopped_at() {
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/aaaa"))), 1);
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/bbbb"))), 2);
}

#[test]
fn a_key_that_is_gone_still_orders_the_list() {
    // The tree a run stopped at may have been removed by that run, so the key need not be in the list.
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/00/aaab"))), 1);
}

#[test]
fn a_key_past_the_end_means_the_list_is_finished() {
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/02/zzzz"))), keys().len());
    assert_eq!(resume_index(&keys(), Some(Path::new("/base/01/cccc"))), keys().len());
}

#[test]
fn a_position_survives_being_written_and_read() {
    let root = scratch();
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    progress.set(LongPass::Units, Some(PathBuf::from("/base/0b/ab24900891e355/debug")));
    progress.save(root.path()).expect("saved");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(read.after(LongPass::Trees), Some(Path::new("/base/00/bbbb")));
    assert_eq!(read.after(LongPass::Units), Some(Path::new("/base/0b/ab24900891e355/debug")));
}

#[test]
fn a_pass_that_finished_its_list_leaves_no_position_behind() {
    let root = scratch();
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(PathBuf::from("/base/00/bbbb")));
    progress.save(root.path()).expect("saved");
    progress.set(LongPass::Trees, None);
    progress.save(root.path()).expect("saved");
    let (read, _notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None, "the next run starts at the front");
}

#[test]
fn a_missing_record_is_the_first_run_and_is_not_a_failure() {
    let root = scratch();
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(read.after(LongPass::Units), None);
    assert!(notes.is_empty(), "a first run says nothing about a record it never had: {notes:?}");
}

#[test]
fn an_unreadable_record_means_the_front_and_says_so() {
    let root = scratch();
    fs::write(root.path().join(PROGRESS_FILE_NAME), b"\xff\xfe not a record\n").expect("a record nothing can read");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(read.after(LongPass::Units), None);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("starts at the front"), "{notes:?}");
}

#[test]
fn a_record_of_another_shape_means_the_front() {
    let root = scratch();
    fs::write(root.path().join(PROGRESS_FILE_NAME), b"trees /base/00/bbbb\n").expect("a record of no version this build knows");
    let (read, notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), None);
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn a_path_with_a_newline_in_it_is_one_line_in_the_record() {
    let root = scratch();
    let odd: PathBuf = PathBuf::from("/base/00/line\nbreak\\back");
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(odd.clone()));
    progress.save(root.path()).expect("saved");
    let text: String = fs::read_to_string(root.path().join(PROGRESS_FILE_NAME)).expect("the record");
    assert_eq!(text.lines().count(), 2, "the header and one tree line: {text:?}");
    let (read, _notes) = crate::lean_progress::read(root.path());
    assert_eq!(read.after(LongPass::Trees), Some(odd.as_path()));
}

#[test]
fn the_state_directory_is_the_state_home_or_the_home_for_that() {
    // The reader takes both variables from the environment, so this test states what it read.
    if let Some(found) = state_directory() {
        assert!(found.ends_with("cargo-shadow"), "{}", found.display());
    }
}
