//! Controls for the record of a batch's scratch reference:
//! the worktree it names is the one it was written for,
//! and a record an earlier release wrote still names its worktree.

use super::*;
use crate::test_support::{fixture, remove};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

/// What:
///  A record keeps the bytes of the worktree and the Git directory it names.
/// Why:
///   The sweep compares the record's Git directory with the instance it sweeps for,
/// which is a hash of that directory's bytes,
/// so a directory that lost them names another instance,
/// and the reference of the dropped batch is never removed.
#[test]
fn a_record_keeps_the_bytes_of_the_paths_it_names() {
    let journal: PathBuf = fixture("scratch-record-non-utf8");
    let repository: PathBuf = journal.join(OsStr::from_bytes(b"w\xffrktree"));
    let git_dir: PathBuf = journal.join(OsStr::from_bytes(b"c\xffmmon"));
    record_scratch_ref(journal.as_path(), 4, repository.as_path(), git_dir.as_path());
    let read: String = std::fs::read_to_string(record_path(journal.as_path(), 4)).expect("the record");
    let named: (PathBuf, PathBuf) = decode(read.as_str()).expect("the record names a worktree");
    assert_eq!(named, (repository, git_dir.clone()), "the record names the worktree it was written for");
    assert_eq!(instance_of(named.1.as_path()), instance_of(git_dir.as_path()), "the instance the sweep compares is that directory's");
    remove(journal.as_path());
}

/// What:
///  A record written by a release that spelled its paths as plain text still names its worktree.
/// Why:
///   A service that starts across a dropped batch of an upgrade must sweep that batch's reference,
/// not leave it.
#[test]
fn a_record_of_plain_text_paths_is_still_read() {
    let earlier: &str = "{\"v\":1,\"repository\":\"/srv/repo\",\"gitDir\":\"/srv/repo/.git\"}\n";
    assert_eq!(decode(earlier), Some((PathBuf::from("/srv/repo"), PathBuf::from("/srv/repo/.git"))));
}

/// A record that names no worktree,
/// or names one in a field that is not the bytes of a path,
/// is no proof of where a reference is.
#[test]
fn a_damaged_record_names_no_worktree() {
    for damaged in [
        "{\"v\":2,\"repository\":\"/srv/repo\",\"gitDir\":\"/srv/repo/.git\"}\n",
        "{\"v\":1,\"repository\":\"/srv/repo\"}\n",
        "{\"v\":1,\"repository\":\"/srv/repo\",\"gitDir\":\"/srv/repo/.git\",\"gitDirHex\":\"2f6\"}\n",
        "not json",
    ] {
        assert_eq!(decode(damaged), None, "{damaged}");
    }
}
