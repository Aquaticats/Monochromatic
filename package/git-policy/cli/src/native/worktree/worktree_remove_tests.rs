//! Controls for the subvolume removal of a worktree:
//! the plan's accepted and declined forms,
//! and,
//! on btrfs,
//! the
//!  removal of a clean subvolume worktree (where the mount allows it) and every case that must be left to Git.

use super::*;
use crate::worktree_remove_plan::ForceAsked;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::test_support::{age, git, git_context, remove, repository};

/// The plan for a target typed as its real path.
fn plan_for(target: &Path) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: target.as_os_str().to_os_string(), force: ForceAsked::No };
}

/// The same plan with the force a person passes as `-f`/`--force`.
fn forced_plan_for(target: &Path) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: target.as_os_str().to_os_string(), force: ForceAsked::Once };
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

/// What:
///  A subvolume worktree that holds a change is destroyed by a forced removal where the mount allows the call,
///  and left whole to Git when the person did not force it.
/// Why:
///   Git's own removal skips its cleanliness proof,
///  populated submodules included,
/// only when the force is set,
///  so the forced removal answers for exactly the worktree it was given;
///  this runs the real kernel call and the real mount option,
/// which no stand-in can.
#[test]
fn btrfs_a_forced_removal_of_a_changed_subvolume_worktree_is_destroyed_where_the_mount_allows_it() {
    let Some(root) = btrfs_fixture("remove-forced") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let changed: PathBuf = worktree.join("tracked.txt");
    std::fs::write(changed.as_path(), b"unsaved work\n").expect("a change");
    // The unforced form is Git's, exactly as it is without the change.
    assert!(!served_plan(&repo, &plan_for(worktree.as_path())), "a changed worktree is Git's without the force");
    assert_eq!(std::fs::read(changed.as_path()).expect("the change is still there"), b"unsaved work\n");
    let forced: bool = served_plan(&repo, &forced_plan_for(worktree.as_path()));
    assert_eq!(forced, removal_allowed(root.as_path()));
    if forced {
        assert!(!worktree.exists(), "the forced removal destroyed the worktree and its change with it");
        assert!(!repo.join(".git/worktrees/wt").exists(), "the registration is gone too");
    } else {
        assert_eq!(std::fs::read(changed.as_path()).expect("a refusal changes nothing"), b"unsaved work\n");
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

/// What:
///  A worktree that holds a subvolume below its root is refused before anything is handed over or destroyed,
///  and nothing of that subvolume is deleted.
/// Why:
///   The kernel refuses `BTRFS_IOC_SNAP_DESTROY` for a subvolume that contains another one,
/// so the destroy could never
///  complete:
/// the rename took the worktree out of the person's way,
/// the directory stayed under the service's
///  own name for ever with every file of the worktree inside it,
/// and the command said the files were gone
///  (a Sonnet finding,
/// F5).
/// Git's own removal is no answer here either:
/// it unlinks the inner worktree's files and leaves its empty
///  subvolume.
/// A nested linked worktree is the shape a person meets,
/// and a subvolume of their own is the same to the kernel.
#[test]
fn btrfs_a_worktree_holding_a_nested_subvolume_is_refused_before_the_hand_over() {
    let Some(root) = btrfs_fixture("remove-nested-volume") else { return };
    let (repo, worktree, inner) = a_worktree_holding_a_nested_worktree(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path() });
    if !removal_allowed(root.as_path()) {
        // Nothing here can destroy a subvolume, so the removal is Git's, as it always was on such a mount.
        assert_eq!(outcome, RemoveOutcome::LeftToGit);
        return;
    }
    let RemoveOutcome::Refused(report) = outcome else { panic!("a worktree the kernel can never destroy is refused, but it was {outcome:?}") };
    assert!(report.contains(inner.to_str().expect("UTF-8")), "the report names the subvolume that stands in the way: {report}");
    assert!(report.contains("refs/heads/inner"), "and the branch the worktree there has checked out: {report}");
    assert!(report.contains("Nothing was removed"), "and says nothing changed: {report}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is where it was");
    assert!(inner.join("notes.txt").is_file(), "and nothing of the worktree inside it went");
    assert!(repo.join(".git/worktrees/wt").is_dir(), "the registration is where it was");
    remove(root.as_path());
}

/// What:
///  A repository with one clean subvolume worktree,
///  which holds a subvolume of its own with work in it,
///  as a linked worktree inside a worktree is.
/// Why:
///   The nested subvolume is what the kernel refuses to destroy through its parent,
///  and its own files are what a removal
///  must never take,
///  so a control builds both rather than trusting one to bring the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, worktree, inner } = aWorktreeHoldingANestedWorktree(root);
/// ```
fn a_worktree_holding_a_nested_worktree(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    // Git calls an entry below `.claude` ignored in every repository that adds a worktree inside another one, and the
    // cleanliness proof refuses an untracked file.
    std::fs::write(repo.join(".gitignore"), b"/.claude/\n").expect("the ignore rule");
    git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join("wt");
    subvolume_create_at(worktree.as_path()).expect("subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let outer: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("canonical");
    let nested: PathBuf = outer.join(".claude/worktrees/inner");
    std::fs::create_dir_all(nested.parent().expect("the folder the worktree inside it stands in")).expect("the folder of a nested worktree");
    subvolume_create_at(nested.as_path()).expect("the subvolume of the worktree inside it");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "inner", nested.to_str().expect("a UTF-8 fixture path")]);
    let inner: PathBuf = std::fs::canonicalize(nested.as_path()).expect("canonical");
    // The work that is nobody else's, in both shapes a control can tell from a deletion of the directory.
    std::fs::write(inner.join("notes.txt"), b"what I was in the middle of\n").expect("a file the person wrote");
    std::fs::write(inner.join("tracked.txt"), b"not committed yet\n").expect("a change the person made");
    for name in ["tracked.txt", ".gitignore"] {
        age(outer.join(name).as_path());
    }
    git(outer.as_path(), &["update-index", "--refresh"]);
    return (repo, outer, inner);
}
