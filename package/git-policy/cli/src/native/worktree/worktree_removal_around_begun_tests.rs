//! What:
//!  Controls for the record a removal writes before its pass empties a worktree around a volume.
//! Why:
//!   The record is the only thing that proves the entries a half-emptied worktree is missing are this
//!   program's own,
//! so it must match the registration and the worktree it was written for and nothing else;
//! a record read for
//!  another worktree would let a removal finish a tree it never began.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordAroundBegun(common, which); expect(aroundBegunBy(common, which)).toBe(true);
//! expect(aroundBegunBy(common, anotherWorktree)).toBe(false);
//! ```

use super::*;
use crate::fs_beneath::identity_of;
use crate::fs_beneath::open_root;
use crate::test_support::{fixture, remove};
use std::ffi::OsStr;

/// An identity of the directory at `path`,
/// as the removal reads one.
fn identity(path: &Path) -> ObjectId {
    let held: std::fs::File = open_root(path).expect("the directory opens");
    return identity_of(&held).expect("the directory has an identity");
}

/// What:
///  A record is read back for the registration and the worktree it was written for,
///  and not for another one of either.
/// Why:
///   A path is used again and again,
///  so the identities are what tell one worktree from another:
/// a person who removed this
///  worktree and added another is two other objects.
#[test]
fn a_record_is_read_only_for_the_removal_that_wrote_it() {
    let root: PathBuf = fixture("around-begun-match");
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("wt");
    let other: PathBuf = root.join("other");
    std::fs::create_dir_all(common.as_path()).expect("a common Git directory");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    std::fs::create_dir_all(other.as_path()).expect("another directory");
    let id: &OsStr = OsStr::new("topic");
    let which: WhichRemoval<'_> = WhichRemoval { id, admin: identity(common.as_path()), worktree: identity(worktree.as_path()), target: worktree.as_path() };
    assert!(!around_begun_by(common.as_path(), which), "no record says so yet");
    record_around_begun(common.as_path(), which).expect("the record is written");
    assert!(around_begun_by(common.as_path(), which), "the record names this removal");

    let another_worktree: WhichRemoval<'_> = WhichRemoval { worktree: identity(other.as_path()), ..which };
    assert!(!around_begun_by(common.as_path(), another_worktree), "another worktree at another identity is not this one");
    let another_registration: WhichRemoval<'_> = WhichRemoval { admin: identity(other.as_path()), ..which };
    assert!(!around_begun_by(common.as_path(), another_registration), "a registration that is not the one it was proven against is not this one");
    let another_path: WhichRemoval<'_> = WhichRemoval { target: other.as_path(), ..which };
    assert!(!around_begun_by(common.as_path(), another_path), "the path the removal was given is part of it too");

    forget_around_begun(common.as_path(), id);
    assert!(!around_begun_by(common.as_path(), which), "a record that is forgotten proves nothing");
    assert!(!common.join(BEGUN_FOLDER).exists(), "and the folder goes with the last record");
    remove(root.as_path());
}

/// What:
///  A file in that folder that this build did not write is left alone,
///  and proves nothing.
/// Why:
///   The folder is read by a removal of the same registration id,
/// and a file of another build or of a person must
///  never be read as a record of this one.
#[test]
fn a_file_this_build_did_not_write_proves_nothing_and_is_kept() {
    let root: PathBuf = fixture("around-begun-stranger");
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("wt");
    std::fs::create_dir_all(common.as_path()).expect("a common Git directory");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    let id: &OsStr = OsStr::new("topic");
    let which: WhichRemoval<'_> = WhichRemoval { id, admin: identity(common.as_path()), worktree: identity(worktree.as_path()), target: worktree.as_path() };
    let folder: PathBuf = common.join(BEGUN_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the records' folder");
    let stranger: PathBuf = folder.join(id);
    std::fs::write(stranger.as_path(), b"another build's record\n").expect("a file of another shape");
    assert!(!around_begun_by(common.as_path(), which), "a file this build cannot read proves nothing");
    assert!(stranger.is_file(), "and it is left where it is");
    remove(root.as_path());
}

/// What:
///  A record whose registration is no longer in the repository is dropped,
///  and one whose registration is there is kept.
/// Why:
///   A worktree a person removed with `--force` through Git leaves a record nothing will finish,
/// and a folder that
///  grows with every such removal is read by every command.
#[test]
fn a_record_of_a_registration_that_is_gone_is_dropped() {
    let root: PathBuf = fixture("around-begun-prune");
    let common: PathBuf = root.join("common");
    let worktree: PathBuf = root.join("wt");
    let other: PathBuf = root.join("other");
    for path in [common.as_path(), worktree.as_path(), other.as_path()] {
        std::fs::create_dir_all(path).expect("a directory");
    }
    let live: &OsStr = OsStr::new("live");
    let gone: &OsStr = OsStr::new("gone");
    record_around_begun(common.as_path(), WhichRemoval { id: live, admin: identity(common.as_path()), worktree: identity(worktree.as_path()), target: worktree.as_path() }).expect("the record");
    record_around_begun(common.as_path(), WhichRemoval { id: gone, admin: identity(common.as_path()), worktree: identity(other.as_path()), target: other.as_path() }).expect("the record");
    std::fs::create_dir_all(common.join("worktrees/live")).expect("the registration that is still there");
    prune_around_begun(common.as_path());
    assert!(common.join(BEGUN_FOLDER).join(live).is_file(), "the registration that is still there keeps its record");
    assert!(!common.join(BEGUN_FOLDER).join(gone).exists(), "the one that is gone does not");
    remove(root.as_path());
}
