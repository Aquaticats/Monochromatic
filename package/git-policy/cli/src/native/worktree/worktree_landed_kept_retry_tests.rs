//! What:
//!  Controls for the record of a landed worktree a sweep kept,
//!  which the next sweep examines again.
//! Why:
//!   A record that a later sweep cannot read leaves a worktree in place for ever,
//!  and one that names another directory would remove something nobody kept,
//!  so both the round trip and the refusal to guess are pinned here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberKept(common, '/x/y', 'refs/heads/y', 'process 4242 is working in it');
//! expect(rememberedKept(common)).toEqual([{ worktree: '/x/y', branch: 'refs/heads/y' }]);
//! ```

use super::{KEPT_RETRY_FOLDER, forget_kept, remembered_kept, remember_kept};
use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::test_support::fixture;
use std::path::PathBuf;

#[test]
fn a_kept_worktree_is_remembered_with_its_branch_until_a_later_sweep_reads_it() {
    let common: PathBuf = fixture("kept-retry-round-trip");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "process 4242 is working in it");
    let remembered: Vec<super::RememberedKept> = remembered_kept(common.as_path());
    assert_eq!(remembered.len(), 1, "one record for one worktree: {remembered:?}");
    assert_eq!(remembered[0].worktree, worktree, "the record names the worktree");
    assert_eq!(remembered[0].branch, "refs/heads/feature", "and the branch it had checked out");
}

#[test]
fn remembering_the_same_worktree_twice_writes_one_record() {
    let common: PathBuf = fixture("kept-retry-twice");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "a lock is a promise to keep it");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "a bisect is unfinished there");
    let remembered: Vec<super::RememberedKept> = remembered_kept(common.as_path());
    assert_eq!(remembered.len(), 1, "a second sweep of one landing writes one record: {remembered:?}");
    let record: String = std::fs::read_to_string(path_for_reading(common.as_path(), KEPT_RETRY_FOLDER).read_dir().expect("the folder").flatten().next().expect("the record").path()).expect("the record's text");
    assert!(record.contains("a bisect is unfinished there"), "the last reason is the one a person reads: {record}");
}

#[test]
fn forgetting_a_worktree_leaves_no_record_for_it() {
    let common: PathBuf = fixture("kept-retry-forget");
    let worktree: PathBuf = common.join("outer");
    remember_kept(common.as_path(), worktree.as_path(), "refs/heads/feature", "process 4242 is working in it");
    forget_kept(common.as_path(), worktree.as_path());
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "nothing is examined again once the question is answered");
    forget_kept(common.as_path(), worktree.as_path());
}

#[test]
fn a_file_this_release_did_not_write_names_no_worktree() {
    let common: PathBuf = fixture("kept-retry-foreign");
    let folder: PathBuf = path_for_reading(common.as_path(), KEPT_RETRY_FOLDER);
    std::fs::create_dir_all(folder.as_path()).expect("the folder");
    std::fs::write(folder.join("0123456789abcdef.txt"), b"another program's note\n").expect("a file that is not a record");
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "a reader that guessed would remove a folder nobody kept");
}

#[test]
fn a_repository_with_no_such_folder_remembers_nothing() {
    let common: PathBuf = fixture("kept-retry-empty");
    assert_eq!(remembered_kept(common.as_path()), Vec::new(), "no folder, no kept worktree");
}
