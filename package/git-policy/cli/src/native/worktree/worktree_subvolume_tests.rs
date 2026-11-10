//! Controls for the subvolume destination of a `git worktree add`:
//! creation on btrfs,
//! the refusals that leave Git to
//!  make its ordinary directory,
//! Git filling and removing a created subvolume,
//! and the cleanup after a failed Git,
//! for a subvolume and for the ordinary directory the copy route reserves.

use super::*;
use crate::fs_btrfs_test_support::btrfs_fixture;
use crate::fs_kind::{BTRFS_SUBVOLUME_ROOT_INODE, is_subvolume_root};
use crate::test_support::{fixture, git, git_output, remove, repository};
use std::os::unix::fs::MetadataExt;
use std::process::Output;

/// A plan for a destination,
/// with an insertion point no control here reads.
fn plan_for(destination: PathBuf) -> AddPlan {
    return AddPlan { destination, insert_at: 2, quiet: false };
}

/// A destination that does not exist on btrfs becomes an empty subvolume root,
/// and settling removes it while it is
///  still empty.
#[test]
fn btrfs_a_missing_destination_becomes_an_empty_subvolume_and_settles_away() {
    let Some(root) = btrfs_fixture("worktree-subvolume-create") else { return };
    let plan: AddPlan = plan_for(root.join("topic"));
    let created: CreatedSubvolume = prepare_subvolume_destination(&plan).expect("a subvolume is created");
    assert_eq!(created.path, plan.destination);
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"));
    assert_eq!(std::fs::metadata(plan.destination.as_path()).expect("metadata").ino(), BTRFS_SUBVOLUME_ROOT_INODE);
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::Removed);
    assert!(!plan.destination.exists(), "an empty subvolume is removed");
    remove(root.as_path());
}

/// A destination that exists,
/// empty or not,
/// is left to Git,
/// and a destination whose parent is missing has no
///  filesystem to ask.
#[test]
fn btrfs_an_existing_or_unplaceable_destination_is_left_to_git() {
    let Some(root) = btrfs_fixture("worktree-subvolume-existing") else { return };
    std::fs::create_dir(root.join("empty")).expect("empty directory");
    std::fs::create_dir(root.join("full")).expect("full directory");
    std::fs::write(root.join("full/file"), b"x").expect("file");
    std::os::unix::fs::symlink(root.join("full"), root.join("link")).expect("link");
    for name in ["empty", "full", "link"] {
        assert_eq!(prepare_subvolume_destination(&plan_for(root.join(name))), None, "{name}");
    }
    assert!(!is_subvolume_root(root.join("empty").as_path()).expect("probe"), "the existing directory is untouched");
    assert_eq!(prepare_subvolume_destination(&plan_for(root.join("absent/topic"))), None);
    assert!(!root.join("absent").exists());
    remove(root.as_path());
}

/// A subvolume that holds anything is Git's result and stays;
/// this is the guard against removing a worktree.
#[test]
fn btrfs_a_filled_destination_is_not_removed() {
    let Some(root) = btrfs_fixture("worktree-subvolume-filled") else { return };
    let plan: AddPlan = plan_for(root.join("topic"));
    let created: CreatedSubvolume = prepare_subvolume_destination(&plan).expect("created");
    std::fs::write(plan.destination.join(".git"), b"gitdir: elsewhere\n").expect("a file Git wrote");
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::HoldsSomething);
    assert!(plan.destination.join(".git").is_file());
    // A destination that is already gone is not an error either.
    std::fs::remove_file(plan.destination.join(".git")).expect("empty it");
    std::fs::remove_dir(plan.destination.as_path()).expect("remove it");
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::Gone);
    remove(root.as_path());
}

/// Wherever the fixture root is,
/// creation happens exactly when the parent is on btrfs;
/// this is the control that runs
///  on the gate's overlay,
/// where the answer is nothing.
#[test]
fn creation_happens_exactly_when_the_parent_is_on_btrfs() {
    let root: PathBuf = fixture("worktree-subvolume-kind");
    let on_btrfs: bool = filesystem_kind(root.as_path()) == Ok(FilesystemKind::Btrfs);
    let created: Option<CreatedSubvolume> = prepare_subvolume_destination(&plan_for(root.join("topic")));
    assert_eq!(created.is_some(), on_btrfs);
    assert_eq!(root.join("topic").exists(), on_btrfs);
    if let Some(found) = created {
        settle_created_destination(&found);
    }
    remove(root.as_path());
}

/// Real Git 2.56.0 fills the empty subvolume as a worktree,
/// the subvolume stays a subvolume root,
/// and
///  `git worktree remove` removes it again.
#[test]
fn btrfs_real_git_fills_the_subvolume_and_removes_it() {
    let Some(root) = btrfs_fixture("worktree-subvolume-git") else { return };
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked file");
    git(repo.as_path(), &["add", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let plan: AddPlan = plan_for(root.join("wt"));
    let created: CreatedSubvolume = prepare_subvolume_destination(&plan).expect("created");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::HoldsSomething, "the checkout stays");
    assert!(is_subvolume_root(plan.destination.as_path()).expect("probe"), "Git filled the subvolume in place");
    assert_eq!(std::fs::read(plan.destination.join("tracked.txt")).expect("checked out"), b"tracked\n");
    git(repo.as_path(), &["worktree", "remove", "../wt"]);
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// A Git that fails before it starts to fill the destination (a branch that exists) leaves the empty subvolume,
/// and
///  settling removes it;
/// without the settle the leftover would stay,
/// which this control shows first.
#[test]
fn btrfs_a_failed_git_leaves_nothing_after_settling() {
    let Some(root) = btrfs_fixture("worktree-subvolume-failed") else { return };
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["branch", "taken"]);
    let plan: AddPlan = plan_for(root.join("wt"));
    let created: CreatedSubvolume = prepare_subvolume_destination(&plan).expect("created");
    let failed: Output = git_output(repo.as_path(), &["worktree", "add", "-b", "taken", "../wt"]);
    assert!(!failed.status.success());
    assert!(plan.destination.exists(), "Git itself leaves the destination it did not create");
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::Removed);
    assert!(!plan.destination.exists());
    remove(root.as_path());
}

/// A destination that was moved away and replaced by another empty subvolume is not the one this process created,
/// and its
///  settlement leaves the replacement alone.
#[test]
fn btrfs_a_replacement_subvolume_is_not_settled_away() {
    let Some(root) = btrfs_fixture("worktree-subvolume-replaced") else { return };
    let plan: AddPlan = plan_for(root.join("topic"));
    let created: CreatedSubvolume = prepare_subvolume_destination(&plan).expect("created");
    std::fs::rename(plan.destination.as_path(), root.join("topic.moved")).expect("move the created subvolume away");
    crate::fs_btrfs_ioctl::subvolume_create_at(plan.destination.as_path()).expect("another subvolume at the same path");
    assert_eq!(settle_created_destination(&created), CreatedDestinationSettlement::LeftAlone);
    assert!(plan.destination.is_dir(), "the replacement is not ours to remove");
    assert!(root.join("topic.moved").is_dir(), "neither is the moved one");
    remove(root.as_path());
}

/// The ordinary directory the copy route reserves is removed again while it is empty,
/// and a directory another program wrote into stays with everything in it:
/// this is the guard that keeps a refused add from deleting anything.
#[test]
fn a_reserved_destination_is_removed_while_empty_and_kept_when_it_holds_anything() {
    let root: PathBuf = fixture("created-destination-settlement");
    let empty: PathBuf = root.join("empty");
    let Reservation::Mine(made) = reserve_destination(empty.as_path()) else { panic!("the first creation owns its destination") };
    assert_eq!(settle_created_destination(&made), CreatedDestinationSettlement::Removed);
    assert!(!empty.exists(), "the empty directory this command made is removed");
    let holding: PathBuf = root.join("holding");
    let Reservation::Mine(reserved) = reserve_destination(holding.as_path()) else { panic!("the first creation owns its destination") };
    std::fs::write(holding.join("someone-elses-file"), b"theirs").expect("another program's file");
    assert_eq!(settle_created_destination(&reserved), CreatedDestinationSettlement::HoldsSomething);
    assert_eq!(std::fs::read(holding.join("someone-elses-file")).expect("the other program's file"), b"theirs");
    remove(root.as_path());
}

/// A directory that replaced the one this command made is not this command's to remove.
#[test]
fn a_replacement_directory_is_left_alone() {
    let root: PathBuf = fixture("created-destination-replaced");
    let destination: PathBuf = root.join("topic");
    let moved: PathBuf = root.join("topic.moved");
    let Reservation::Mine(made) = reserve_destination(destination.as_path()) else { panic!("the first creation owns its destination") };
    std::fs::rename(destination.as_path(), moved.as_path()).expect("move the created directory away");
    std::fs::create_dir(destination.as_path()).expect("another directory at the same name");
    assert_eq!(settle_created_destination(&made), CreatedDestinationSettlement::LeftAlone);
    assert!(destination.is_dir(), "the replacement is not ours to remove");
    assert!(moved.is_dir(), "neither is the moved one");
    remove(root.as_path());
}
