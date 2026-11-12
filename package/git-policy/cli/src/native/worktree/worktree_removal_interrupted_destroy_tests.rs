//! What:
//!  Controls for finishing a removal whose destroy happened and whose registration did not go.
//! Why:
//!   The finish is the one path that ends a removal a kill left half done,
//! and it refuses anything below the registration it
//!  would not delete,
//! so the wrapper's own state in that registration has to be out of its way before it looks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(finishInterruptedDestroys(common)).toEqual([]); expect(admin.exists()).toBe(false);
//! ```

use super::*;
use crate::fs_beneath::identity_of;
use crate::fs_btrfs_ioctl::{subvolume_create_at, subvolume_destroy_at};
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::test_support::{git, remove, repository};
use std::path::PathBuf;

/// What:
///  A removal whose destroy happened and whose registration stayed is finished by the next command even when the only
///  thing below that registration is the pool of snapshots the wrapper made ahead of an add.
/// Why:
///   A worktree that was the source of one `git worktree add` holds that pool,
/// whose members are subvolumes,
/// and the
///  registration's emptying refuses a nested subvolume,
/// so every such removal ended as "the files are gone and the
///  registration is not",
/// with the whole pool left behind until the person ran `git worktree prune`
///  (a Sonnet finding,
/// F1).
///  A pool is the wrapper's own cache of the worktree,
/// and a later add that finds none makes its own.
#[test]
fn btrfs_a_recorded_destroy_is_finished_when_the_snapshots_ahead_are_the_only_thing_left() {
    let Some(root) = btrfs_fixture("recorded-destroy-ahead-pool") else { return };
    if !removal_allowed(root.as_path()) {
        // The destroy this record names cannot be made here at all, so there is nothing to finish.
        return;
    }
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    subvolume_create_at(root.join("wt").as_path()).expect("the subvolume of the worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt").as_path()).expect("canonical");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    // The pool the commit service fills for a worktree an add copies from, with one finished snapshot in it.
    let pool: PathBuf = crate::worktree_snapshot_ahead::pool_folder(admin.as_path());
    std::fs::create_dir_all(pool.as_path()).expect("the folder of the snapshots ahead");
    let member: PathBuf = pool.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("a snapshot the wrapper made ahead of an add");
    std::fs::write(member.join("tracked.txt"), b"the worktree's own file\n").expect("its files");
    // The state a removal killed between its kernel call and the emptying of the registration leaves.
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    record_destroy(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("the record of the destroy");
    subvolume_destroy_at(worktree.as_path()).expect("the destroy the record names");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "the pool is the wrapper's own, so nothing stands in the way of finishing: {problems:?}");
    assert!(!admin.exists(), "the registration is gone and the path is free again");
    assert!(!pool.exists(), "and the pool went with it");
    remove(root.as_path());
}

/// What:
///  A removal whose destroy happened and whose registration stayed is finished by the next command even when the only
///  thing below that registration is a pool a killed clear left under its taken name.
/// Why:
///   A clear renames the pool of snapshots ahead to a name no fill reads before it deletes a member of it,
///  so a command
///  killed in that moment leaves a member under that name,
///  and the finish's walk refuses every nested subvolume:
///  a
///  clear that took only the pool's own name would refuse its own leftover for ever,
///  and every later command would say the
///  removal cannot be finished (the incident of 2026-11-11 named exactly such a member under the pool's
///  own name).
#[test]
fn btrfs_a_recorded_destroy_is_finished_when_a_pool_a_killed_clear_left_is_the_only_thing_left() {
    let Some(root) = btrfs_fixture("recorded-destroy-pool-left-taken") else { return };
    if !removal_allowed(root.as_path()) {
        // The destroy this record names cannot be made here at all, so there is nothing to finish.
        return;
    }
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    subvolume_create_at(root.join("wt").as_path()).expect("the subvolume of the worktree");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt").as_path()).expect("canonical");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees/wt");
    // The state a clear killed between its rename and its deletions leaves: a member of the pool under the taken name.
    let data: PathBuf = admin.join("monochromatic-git-policy-cli");
    let left: PathBuf = data.join("removing-0f0e0d0c-0b0a-4908-8706-050403020100");
    std::fs::create_dir_all(left.as_path()).expect("the folder a killed clear had renamed");
    let member: PathBuf = left.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("the member a killed clear left under its taken name");
    std::fs::write(member.join("tracked.txt"), b"a copy of the worktree\n").expect("its files");
    let identity = identity_of(&std::fs::File::open(admin.as_path()).expect("the registration")).expect("its identity");
    record_destroy(common_dir.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("the record of the destroy");
    subvolume_destroy_at(worktree.as_path()).expect("the destroy the record names");
    record_destroyed(common_dir.as_path(), std::ffi::OsStr::new("wt"));
    let problems: Vec<String> = finish_interrupted_destroys(common_dir.as_path());
    assert!(problems.is_empty(), "the leftover of the wrapper's own clear is not something to report: {problems:?}");
    assert!(!admin.exists(), "the registration is gone and the path is free again");
    assert!(!left.exists(), "and the pool the killed clear left went with it");
    remove(root.as_path());
}
