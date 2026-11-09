//! Controls for the writers the migration's lists share:
//! a name that holds a link is replaced,
//! never written through,
//! and a failed write leaves no temporary file.

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A symbolic link at the target's name is replaced,
///  and the file it points at keeps its bytes.
/// Why:
///   A plain write follows the link and truncates the target,
/// so a link planted at a list's name would let the command
///  overwrite a file of the person's own.
#[test]
fn a_link_at_the_targets_name_is_replaced_and_its_target_is_untouched() {
    let root: PathBuf = fixture("file-write-link");
    let notes: PathBuf = root.join("notes.txt");
    std::fs::write(notes.as_path(), b"a person's notes\n").expect("their file");
    let target: PathBuf = root.join("list.txt");
    std::os::unix::fs::symlink(notes.as_path(), target.as_path()).expect("a link at the target's name");
    write_beside_then_rename(target.as_path(), b"ignored/restored.txt\n").expect("written");
    assert_eq!(std::fs::read(notes.as_path()).expect("their file"), b"a person's notes\n", "the target of the link is untouched");
    assert!(!std::fs::symlink_metadata(target.as_path()).expect("the name").file_type().is_symlink(), "the name holds the bytes, not the link");
    assert_eq!(std::fs::read(target.as_path()).expect("the list"), b"ignored/restored.txt\n");
    assert_eq!(std::fs::read_dir(root.as_path()).expect("listing").flatten().count(), 2, "no temporary file stays");
    remove(root.as_path());
}

/// A write into a directory that cannot be listed as a parent is reported,
/// and nothing this call made stays.
#[test]
fn a_write_that_cannot_be_put_in_place_leaves_nothing_behind() {
    let root: PathBuf = fixture("file-write-missing-parent");
    let target: PathBuf = root.join("gone").join("list.txt");
    let error: std::io::Error = write_beside_then_rename(target.as_path(), b"x").expect_err("no such directory");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound, "the failure is the missing directory");
    assert_eq!(std::fs::read_dir(root.as_path()).expect("listing").flatten().count(), 0, "nothing is left in the directory that exists");
    remove(root.as_path());
}
