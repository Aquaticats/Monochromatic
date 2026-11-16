//! What:
//!  The operation in progress,
//! the paths the index marks as unmerged,
//! and the copies a command that undoes an operation leaves beside the audit log.
//! Why:
//!  The reading of the operation and of the conflicted set decides what a refusal may claim,
//! and a copy that is missing,
//!  partial or in the wrong mode loses the half-resolved file the policy exists to protect;
//!  a copy failure that is swallowed discards unsaved work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(saveCopies({ root, commonDir, paths: ['a.rs'], milliseconds })[0].savedTo).toBeDefined();
//! ```

use super::{Operation, SaveFailure, SavedCopy, conflicted_paths, operation_in_progress, save_copies, stamp};
use crate::repo_read_index::IndexEntry;
use crate::test_support::{fixture, remove};
use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// An index entry at `stage` for `path`.
fn entry(path: &str, stage: u8) -> IndexEntry {
    return IndexEntry { path: path.as_bytes().to_vec(), mode: 0o100_644, id: vec![0; 20], stage, extended: 0 };
}

/// A set of conflicted paths from text.
fn set(paths: &[&str]) -> BTreeSet<Vec<u8>> {
    return paths.iter().map(|each| { return each.as_bytes().to_vec(); }).collect();
}

/// A path is conflicted when any of its entries is at stage 1,
/// 2 or 3,
/// however many entries it has.
#[test]
fn conflicted_paths_are_the_unmerged_entries() {
    let entries = vec![entry("a.rs", 0), entry("b.rs", 1), entry("b.rs", 2), entry("b.rs", 3), entry("c.rs", 2), entry("d.rs", 0)];
    assert_eq!(conflicted_paths(entries.as_slice()), set(&["b.rs", "c.rs"]));
    assert_eq!(conflicted_paths(&[]), set(&[]));
}

/// Each state file names its operation;
/// `am` shares the rebase directory and is not one of them.
#[test]
fn the_operation_in_progress_is_read_from_the_state_files() {
    let root: PathBuf = fixture("sides-operation");
    assert_eq!(operation_in_progress(root.as_path()), None);
    for (file, expected) in [("MERGE_HEAD", Operation::Merge), ("CHERRY_PICK_HEAD", Operation::CherryPick), ("REVERT_HEAD", Operation::Revert)] {
        std::fs::write(root.join(file), b"0\n").expect("state file");
        assert_eq!(operation_in_progress(root.as_path()), Some(expected), "{file}");
        std::fs::remove_file(root.join(file)).expect("remove");
    }
    std::fs::create_dir(root.join("rebase-merge")).expect("merge backend");
    assert_eq!(operation_in_progress(root.as_path()), Some(Operation::Rebase));
    std::fs::remove_dir(root.join("rebase-merge")).expect("remove");
    std::fs::create_dir(root.join("rebase-apply")).expect("apply backend");
    assert_eq!(operation_in_progress(root.as_path()), None, "an am keeps rebase-apply without rebasing");
    std::fs::write(root.join("rebase-apply/applying"), b"").expect("am");
    assert_eq!(operation_in_progress(root.as_path()), None);
    std::fs::write(root.join("rebase-apply/rebasing"), b"").expect("rebase");
    assert_eq!(operation_in_progress(root.as_path()), Some(Operation::Rebase));
    remove(root.as_path());
}

/// A fixture with a worktree and a common Git directory,
/// and a file of each kind in it.
fn tree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let base: PathBuf = fixture(name);
    let root: PathBuf = base.join("tree");
    let common: PathBuf = base.join("common");
    std::fs::create_dir_all(root.join("sub")).expect("worktree");
    std::fs::create_dir_all(&common).expect("common directory");
    return (base, root, common);
}

/// The copy holds the file's bytes,
/// permission bits and symbolic-link target,
/// below a directory named by the time.
#[test]
fn a_copy_keeps_content_mode_and_link_target() {
    let (base, root, common) = tree("sides-copy");
    std::fs::write(root.join("a.rs"), b"half \xff resolved\n").expect("a");
    std::fs::set_permissions(root.join("a.rs"), std::fs::Permissions::from_mode(0o754)).expect("mode");
    std::fs::write(root.join("sub/b.rs"), b"").expect("b");
    std::os::unix::fs::symlink("target/elsewhere", root.join("link")).expect("link");
    let paths: Vec<Vec<u8>> = vec![b"a.rs".to_vec(), b"sub/b.rs".to_vec(), b"link".to_vec(), b"gone.rs".to_vec()];
    let milliseconds: i64 = 1_793_628_067_000;
    let copies: Vec<SavedCopy> = save_copies(root.as_path(), common.as_path(), paths.as_slice(), milliseconds).expect("saved");
    let directory: PathBuf = common.join("monochromatic-git-policy-cli/discarded").join(stamp(milliseconds));
    assert_eq!(stamp(milliseconds), "2026-11-02T14-01-07");
    assert_eq!(copies[0].saved_to, Some(directory.join("a.rs")));
    assert_eq!(std::fs::read(directory.join("a.rs")).expect("copy"), b"half \xff resolved\n");
    assert_eq!(std::fs::metadata(directory.join("a.rs")).expect("mode").permissions().mode() & 0o777, 0o754);
    assert_eq!(std::fs::read(directory.join("sub/b.rs")).expect("empty copy"), b"");
    assert_eq!(std::fs::read_link(directory.join("link")).expect("link copy"), PathBuf::from("target/elsewhere"));
    assert_eq!(copies[3], SavedCopy { path: String::from("gone.rs"), saved_to: None });
    remove(base.as_path());
}

/// A second invocation in the same second gets its own directory and never overwrites the first copy.
#[test]
fn a_second_save_in_the_same_second_keeps_the_first() {
    let (base, root, common) = tree("sides-twice");
    std::fs::write(root.join("a.rs"), b"first\n").expect("a");
    let milliseconds: i64 = 1_793_628_067_000;
    let first = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], milliseconds).expect("first");
    std::fs::write(root.join("a.rs"), b"second\n").expect("a");
    let second = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], milliseconds).expect("second");
    assert_ne!(first[0].saved_to, second[0].saved_to);
    assert_eq!(std::fs::read(first[0].saved_to.as_ref().expect("first place")).expect("first copy"), b"first\n");
    assert_eq!(std::fs::read(second[0].saved_to.as_ref().expect("second place")).expect("second copy"), b"second\n");
    remove(base.as_path());
}

/// A place that cannot hold the copies,
/// or a file that cannot be copied,
/// is a failure naming the subject;
/// nothing is claimed saved.
#[test]
fn a_copy_that_cannot_be_written_is_a_failure() {
    let (base, root, common) = tree("sides-failure");
    std::fs::write(root.join("a.rs"), b"mine\n").expect("a");
    // `monochromatic-git-policy-cli` is a file, so no directory can be made below it.
    std::fs::write(common.join("monochromatic-git-policy-cli"), b"in the way").expect("blocker");
    let failure: SaveFailure = save_copies(root.as_path(), common.as_path(), &[b"a.rs".to_vec()], 0).expect_err("no place");
    assert!(failure.subject.contains("discarded"), "{failure:?}");
    assert!(!failure.reason.is_empty(), "{failure:?}");
    std::fs::remove_file(common.join("monochromatic-git-policy-cli")).expect("remove blocker");
    // A directory where the conflicted file is expected cannot be copied as a file.
    std::fs::create_dir(root.join("dir.rs")).expect("directory");
    let directory_failure: SaveFailure = save_copies(root.as_path(), common.as_path(), &[b"dir.rs".to_vec()], 0).expect_err("a directory");
    assert_eq!(directory_failure.subject, "dir.rs");
    assert!(directory_failure.reason.contains("neither a file nor a symbolic link"), "{directory_failure:?}");
    remove(base.as_path());
}
