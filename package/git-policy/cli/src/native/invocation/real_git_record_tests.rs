//! What:
//!  Controls for reading the real Git the wrapper records beside its sockets.
//! Why:
//!  The fallback of `real_git.rs` is only as sound as this reading:
//!  a path taken from the wrong line,
//! a relative path,
//!  or a file past the bound would start the wrong program.

#![cfg(unix)]

use super::{read_recorded_git, recorded_git_bytes, record_file};
use std::os::unix::ffi::OsStrExt;
use crate::real_git_candidate::MAX_SCRIPT_INSPECTION_BYTES;
use crate::test_support::{fixture, remove};
use std::path::{Path, PathBuf};

/// What:
///  The note is one line and the path,
///  a line that is not an absolute path names nothing,
///  and a path that is not text is read as the bytes it holds.
/// Why:
///  A path is bytes, so reading the record as text named a place with U+FFFD in it (`Path::display`).
#[test]
fn the_recorded_path_is_the_first_absolute_line_and_a_path_that_is_not_text_is_one_too() {
    assert_eq!(recorded_git_bytes(b"/usr/bin/git\n"), Some(PathBuf::from("/usr/bin/git")));
    // What no record may name: a path that only starts after a space is not an absolute path.
    assert_eq!(recorded_git_bytes(b"  /opt/git/bin/git  \n"), None);
    assert_eq!(recorded_git_bytes(b"git\n"), None);
    assert_eq!(recorded_git_bytes(b""), None);
    assert_eq!(recorded_git_bytes(b"\n"), None);
    // What a path that is not UTF-8 keeps: every byte the file system gave.
    assert_eq!(recorded_git_bytes(b"/opt/git\xff/git\n"), Some(PathBuf::from(std::ffi::OsStr::from_bytes(b"/opt/git\xff/git"))));
}

/// The record is the note beside the sockets.
#[test]
fn the_record_sits_beside_the_sockets() {
    assert_eq!(record_file(Path::new("/run/user/7")), PathBuf::from("/run/user/7/monochromatic-git-policy-cli/real-git"));
}

/// What the wrapper remembers is what a later command reads,
///  and a missing file,
/// a directory and a file past the bound name nothing.
#[test]
fn what_is_remembered_is_read_and_a_file_that_is_not_a_record_names_nothing() {
    let root: PathBuf = fixture("record-read");
    let git: PathBuf = root.join("bin/git");
    std::fs::create_dir_all(git.parent().expect("the folder of the record")).expect("bin");
    std::fs::write(&git, "# a program").expect("a program");
    super::remember(root.as_path(), git.as_path()).expect("the note is written");
    assert_eq!(read_recorded_git(record_file(root.as_path()).as_path()), Some(git.clone()));
    assert_eq!(read_recorded_git(root.join("missing").as_path()), None);
    assert_eq!(read_recorded_git(root.as_path()), None);
    // What a record of a Git whose path is not text names: the path itself, never a place with U+FFFD in it.
    let binary: PathBuf = root.join("binary");
    std::fs::write(&binary, b"/opt/git\xff/git\n").expect("a path that is not text");
    assert_eq!(read_recorded_git(binary.as_path()), Some(PathBuf::from(std::ffi::OsStr::from_bytes(b"/opt/git\xff/git"))));
    let long: PathBuf = root.join("long");
    let mut text: String = String::from("/opt/git/bin/git\n");
    text.push_str("#".repeat(MAX_SCRIPT_INSPECTION_BYTES).as_str());
    std::fs::write(&long, text).expect("a file past the bound");
    assert_eq!(read_recorded_git(long.as_path()), None);
    remove(root.as_path());
}

/// What:
///  A Git whose path is not text is remembered and read back as itself.
/// Why:
///  What the record held was written as text (`Path::display`), which loses every byte that is not UTF-8, so the fallback
///  named a place that does not exist (the review of `b896c60d97`, finding 6).
#[test]
fn a_git_whose_path_is_not_text_is_remembered_as_itself() {
    let root: PathBuf = fixture("record-bytes");
    let git: PathBuf = root.join(std::ffi::OsStr::from_bytes(b"bin\xff/git"));
    std::fs::create_dir_all(git.parent().expect("the folder of the program")).expect("bin");
    std::fs::write(&git, "# a program").expect("a program");
    super::remember(root.as_path(), git.as_path()).expect("the note is written");
    assert_eq!(read_recorded_git(record_file(root.as_path()).as_path()), Some(git.clone()), "the path is read back as the bytes it has");
    remove(root.as_path());
}
