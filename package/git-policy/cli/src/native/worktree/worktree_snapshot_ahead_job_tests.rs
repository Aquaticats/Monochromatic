//! Controls for the service's own snapshot of a worktree:
//! the tree the record names and the copy it makes,
//! the count an add keeps it to,
//! and the records it refuses for good.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::test_support::{git, remove};
use crate::worktree_ladder_test_support::{commit_all, subvolume_repository, tracked};
use crate::worktree_snapshot_ahead::{AHEAD_KEPT, hand_over_with, kept_in, pool_folder, request_item};
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
    assert!(reason.contains("is no worktree this service can name"), "the reason names what the record got wrong: {reason}");
    assert!(kept_in(request_pool(repo.as_path()).as_path(), elsewhere.as_path()).is_empty());
    // A record that names a worktree this service can name, beside a Git directory that belongs to another one, is
    //  refused too, and the reason names the directory the worktree really has.
    let foreign: WorkItem = request_item(repo.join("elsewhere/.git").as_path(), repo.as_path(), request_pool(repo.as_path()).as_path()).expect("a record");
    let Verdict::Failed(second) = snapshot_ahead(&foreign) else { panic!("a record with another worktree's Git directory is refused for good") };
    assert!(second.contains("the Git directory of that worktree is"), "the reason names what the record got wrong: {second}");
    assert!(kept_in(request_pool(repo.as_path()).as_path(), repo.as_path()).is_empty());
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

/// The Git directory of a linked worktree is the one its `.git` pointer file names,
/// which is
///  what the service copies for an add run in one.
#[test]
fn btrfs_a_linked_worktrees_pointer_names_its_git_directory() {
    let Some(root) = btrfs_fixture("ahead-job-linked") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("utf8"), "HEAD"]);
    assert_eq!(git_directory_of_worktree(linked.as_path()), Some(repo.join(".git/worktrees/linked")));
    assert_eq!(git_directory_of_worktree(repo.as_path()), Some(repo.join(".git")));
    remove(root.as_path());
}
/// A pool that is full of copies an add already handed to the service for deletion has no room kept for an add:
/// the replacement the add asked for is made beside them.
/// Why:
///  The copies leave as the service's next step,
/// and a request the service answers with "the pool is full" while those copies are queued is a request
///  it never runs again:
/// the add that asked for it waits out its ceiling and makes its worktree with Git's own checkout.
#[test]
fn btrfs_a_pool_full_of_copies_being_deleted_does_not_stop_the_replacement() {
    let Some(root) = btrfs_fixture("ahead-job-replacing") else { return };
    let repo: PathBuf = subvolume_repository(root.as_path());
    for made in 0..AHEAD_KEPT {
        assert_eq!(snapshot_ahead(&request(repo.as_path())), Verdict::Done, "snapshot {made}");
    }
    let stale: Vec<OsString> = kept_in(request_pool(repo.as_path()).as_path(), repo.as_path());
    assert_eq!(stale.len(), AHEAD_KEPT, "the pool holds what an add keeps");
    // The add proved every one of them stale and handed them over, and the replacement it asked for arrives while
    //  their deletes are still queued: the deletes run after the copies are made in one round of the service.
    hand_over_with(repo.join(".git").as_path(), request_pool(repo.as_path()).as_path(), stale.as_slice()).expect("the deletes are recorded");
    let verdict: Verdict = snapshot_ahead(&request(repo.as_path()));
    assert_eq!(verdict, Verdict::Done, "the copies are on their way out, so the pool has room for the replacement: {verdict:?}");
    assert_eq!(
        kept_in(request_pool(repo.as_path()).as_path(), repo.as_path()).len(),
        AHEAD_KEPT + 1,
        "the replacement stands beside the copies that are being deleted"
    );
    remove(root.as_path());
}

/// A main checkout whose Git directory was made elsewhere (`git init --separate-git-dir`) is a worktree this service
/// can name,
/// so an add run in it is served the copies the service makes ahead.
/// Why:
///  Its `.git` is a pointer file rather than a directory,
/// and the Git directory itself records no worktree,
/// so a
///  resolution that reads only a directory named `.git` refuses every request of such a repository,
/// and every add
///  there waits out its ceiling for a copy that can never come.
#[test]
fn btrfs_a_main_checkout_whose_git_directory_is_elsewhere_names_its_worktree() {
    let Some(root) = btrfs_fixture("ahead-job-separate-git-dir") else { return };
    let checkout: PathBuf = root.join("checkout");
    let metadata: PathBuf = root.join("metadata");
    crate::fs_btrfs_ioctl::subvolume_create_at(checkout.as_path()).expect("a checkout that is a subvolume");
    std::fs::create_dir_all(metadata.as_path()).expect("the folder the Git directory goes in");
    git(
        root.as_path(),
        &[
            "init",
            "--quiet",
            "--initial-branch=main",
            "--separate-git-dir",
            metadata.to_str().expect("utf8 metadata"),
            checkout.to_str().expect("utf8 checkout"),
        ],
    );
    tracked(checkout.as_path(), "shared.txt", b"first\n");
    commit_all(checkout.as_path(), "first");
    assert!(std::fs::symlink_metadata(checkout.join(".git")).expect("a .git pointer").is_file(), "the shape this control is about: .git is a pointer file");
    assert_eq!(git_directory_of_worktree(checkout.as_path()), Some(metadata.clone()), "the pointer names the Git directory this checkout uses");
    let item: WorkItem = request_item(metadata.as_path(), checkout.as_path(), pool_folder(metadata.as_path()).as_path()).expect("a record");
    assert_eq!(snapshot_ahead(&item), Verdict::Done, "the service copies the checkout its record names");
    assert_eq!(kept_in(pool_folder(metadata.as_path()).as_path(), checkout.as_path()).len(), 1, "one copy stands in the folder the record named");
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
