//! Controls for the subvolume removal of a worktree:
//! the plan's accepted and declined forms,
//! and,
//! on btrfs,
//! the
//!  removal of a clean subvolume worktree (where the mount allows it) and every case that must be left to Git.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::test_support::{age, git, git_context, remove, repository};

/// The plan for a target typed as its real path.
fn plan_for(target: &Path) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: target.as_os_str().to_os_string(), force: false };
}

/// Whether the wrapper served the plan:
/// the removal ran to its end.
fn served_plan(repo: &Path, plan: &RemovePlan) -> bool {
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    return remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan, common_dir: common_dir.as_path() }) == RemoveOutcome::Removed;
}

/// Whether the wrapper served the removal of `target` typed as its real path.
fn served(repo: &Path, target: &Path) -> bool {
    return served_plan(repo, &plan_for(target));
}

/// A repository with one tracked file,
/// and a linked worktree added into a fresh subvolume with every file aged and the
///  index refreshed,
/// so that the worktree is provably clean.
fn clean_subvolume_worktree(root: &Path) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join("wt");
    subvolume_create_at(worktree.as_path()).expect("subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let canonical: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("canonical");
    age(canonical.join("tracked.txt").as_path());
    git(canonical.as_path(), &["update-index", "--refresh"]);
    return (repo, canonical);
}

/// A clean subvolume worktree is destroyed with its registration where the mount allows it,
/// and is left whole to Git
///  where it does not.
#[test]
fn btrfs_a_clean_subvolume_worktree_is_destroyed_where_the_mount_allows_it() {
    let Some(root) = btrfs_fixture("remove-clean") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let removed: bool = served(&repo, &worktree);
    assert_eq!(removed, removal_allowed(root.as_path()));
    if removed {
        assert!(!worktree.exists());
        assert!(!repo.join(".git/worktrees/wt").exists(), "the registration is gone too");
        let listing: String = String::from_utf8_lossy(&git(repo.as_path(), &["worktree", "list", "--porcelain"]).stdout).into_owned();
        assert!(!listing.contains("wt"), "{listing}");
    } else {
        assert!(worktree.join("tracked.txt").is_file(), "a refusal changes nothing");
    }
    remove(root.as_path());
}

/// Every condition that Git's removal checks keeps the wrapper's hands off:
/// a modified file,
/// an untracked file,
/// a lock,
///  and a worktree that is an ordinary directory.
#[test]
fn btrfs_a_worktree_git_would_refuse_or_that_is_no_subvolume_is_left_alone() {
    let Some(root) = btrfs_fixture("remove-refused") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let plan: RemovePlan = plan_for(&worktree);
    std::fs::write(worktree.join("tracked.txt"), b"TRACKED\n").expect("modify");
    assert!(!served_plan(&repo, &plan), "a modified file");
    std::fs::write(worktree.join("tracked.txt"), b"tracked\n").expect("restore the bytes");
    age(worktree.join("tracked.txt").as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    std::fs::write(worktree.join("untracked.txt"), b"u").expect("untracked");
    assert!(!served_plan(&repo, &plan), "an untracked file");
    std::fs::remove_file(worktree.join("untracked.txt")).expect("remove it");
    git(repo.as_path(), &["worktree", "lock", "../wt"]);
    assert!(!served_plan(&repo, &plan), "a locked worktree");
    git(repo.as_path(), &["worktree", "unlock", "../wt"]);
    assert!(worktree.join("tracked.txt").is_file(), "nothing was removed");
    // The same worktree as an ordinary directory is Git's.
    let ordinary: PathBuf = root.join("plain");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "plain", "../plain"]);
    let canonical: PathBuf = std::fs::canonicalize(ordinary.as_path()).expect("canonical");
    age(canonical.join("tracked.txt").as_path());
    git(canonical.as_path(), &["update-index", "--refresh"]);
    assert!(!served(&repo, &canonical), "an ordinary directory");
    assert!(canonical.join("tracked.txt").is_file());
    // The main checkout has no administrative directory under `worktrees`.
    assert!(!served(&repo, &repo), "the main checkout");
    remove(root.as_path());
}
