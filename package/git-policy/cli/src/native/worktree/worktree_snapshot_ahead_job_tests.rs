//! Controls for the service's own snapshot of a worktree:
//! the tree the record names and the copy it makes,
//! the count an add keeps it to,
//! and the records it refuses for good.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{git, remove};
use crate::worktree_ladder_test_support::subvolume_repository;
use crate::worktree_snapshot_ahead::{AHEAD_KEPT, kept_in, request_item};
use std::path::PathBuf;

/// A request for one snapshot of the fixture's repository,
/// naming the folder this repository keeps the snapshots ahead in.
fn request(repo: &Path) -> WorkItem {
    let git_dir: PathBuf = repo.join(".git");
    return request_item(git_dir.as_path(), repo, request_pool(repo).as_path()).expect("a request record");
}

/// The folder one repository keeps its snapshots ahead in.
fn request_pool(repo: &Path) -> PathBuf {
    return crate::worktree_snapshot_ahead::pool_folder(repo.join(".git").as_path());
}

/// The service makes a snapshot of the worktree its record names,
/// in the folder the record names,
/// and the copy holds
///  the source's tracked and ignored files.
#[test]
fn btrfs_the_service_makes_one_snapshot_of_the_worktree_its_record_names() {
    let Some(root) = btrfs_fixture("ahead-job") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let item: WorkItem = request(repo.as_path());
    assert_eq!(snapshot_ahead(&item), Verdict::Done);
    let made: Vec<OsString> = kept_in(request_pool(repo.as_path()).as_path(), repo.as_path());
    assert_eq!(made.len(), 1, "one snapshot ahead stands where the record named it");
    let copy: PathBuf = request_pool(repo.as_path()).join(&made[0]);
    assert!(crate::fs_kind::is_subvolume_root(copy.as_path()).expect("probe"), "the copy is a subvolume of the source");
    assert_eq!(std::fs::read(copy.join("shared.txt")).expect("the copied file"), b"second, longer\n");
    assert_eq!(std::fs::read(copy.join("ignored/pkg/index.js")).expect("the copied ignored state"), b"module.exports = 1;\n");
    remove(root.as_path());
}

/// A record whose source is not the worktree its Git directory names is refused for good,
/// and it makes nothing.
#[test]
fn btrfs_a_record_that_names_a_tree_its_git_directory_does_not_own_is_refused() {
    let Some(root) = btrfs_fixture("ahead-job-foreign") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let elsewhere: PathBuf = root.join("elsewhere");
    std::fs::create_dir_all(elsewhere.as_path()).expect("a folder to name");
    let item: WorkItem = request_item(repo.join(".git").as_path(), elsewhere.as_path(), request_pool(repo.as_path()).as_path()).expect("a record");
    let Verdict::Failed(reason) = snapshot_ahead(&item) else { panic!("a record of another tree is refused for good") };
    assert!(reason.contains("Git directory belongs to"), "the reason names what the record got wrong: {reason}");
    assert!(kept_in(request_pool(repo.as_path()).as_path(), elsewhere.as_path()).is_empty());
    remove(root.as_path());
}

/// A record that names no folder that exists is refused,
/// so a snapshot is never made where no add looks for one.
#[test]
fn btrfs_a_record_that_names_no_existing_folder_is_refused() {
    let Some(root) = btrfs_fixture("ahead-job-folder") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    // A record that names another folder than the repository's own is a record this service may not act on.
    let mut item: WorkItem = request(repo.as_path());
    item.parent = root.join("gone").display().to_string();
    assert!(matches!(snapshot_ahead(&item), Verdict::Failed(_)), "no folder means nowhere to make it");
    remove(root.as_path());
}

/// A source that is no subvolume is refused once with the reason,
/// not retried:
/// nothing about it can
///  change by trying again.
#[test]
fn btrfs_a_source_that_is_no_subvolume_is_refused_once_with_the_reason() {
    let Some(root) = btrfs_fixture("ahead-job-plain") else { return };
    let repo: PathBuf = root.join("ordinary");
    std::fs::create_dir_all(repo.join(".git").as_path()).expect("a plain directory with a Git directory");
    let item: WorkItem = request(repo.as_path());
    let Verdict::Failed(reason) = snapshot_ahead(&item) else { panic!("a plain directory is refused for good") };
    assert!(reason.contains("not a btrfs subvolume"), "the reason says what a person can change: {reason}");
    remove(root.as_path());
}

/// The worktree root of a linked worktree is read from the `gitdir` its own Git directory records,
/// which is
///  the only root the service copies for an add run in one.
#[test]
fn btrfs_a_linked_worktrees_own_git_directory_names_its_root() {
    let Some(root) = btrfs_fixture("ahead-job-linked") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("utf8"), "HEAD"]);
    assert_eq!(worktree_root_of(repo.join(".git/worktrees/linked").as_path()), Some(linked.clone()));
    assert_eq!(worktree_root_of(repo.join(".git").as_path()), Some(repo.clone()));
    remove(root.as_path());
}
/// The count the service stops at is the count an add keeps,
/// so a burst of records cannot fill a full volume with copies of one tree.
#[test]
fn btrfs_the_service_stops_at_the_count_an_add_keeps() {
    let Some(root) = btrfs_fixture("ahead-job-count") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    for made in 0..AHEAD_KEPT {
        let verdict: Verdict = snapshot_ahead(&request(repo.as_path()));
        assert_eq!(verdict, Verdict::Done, "snapshot {made}");
    }
    let beyond: Verdict = snapshot_ahead(&request(repo.as_path()));
    assert!(matches!(beyond, Verdict::Skipped(_)), "the pool is full: {beyond:?}");
    assert_eq!(kept_in(request_pool(repo.as_path()).as_path(), repo.as_path()).len(), AHEAD_KEPT);
    remove(root.as_path());
}
