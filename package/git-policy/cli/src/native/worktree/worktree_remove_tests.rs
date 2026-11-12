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

/// The same plan with the force a person passes as `-f`/`--force`.
fn forced_plan_for(target: &Path) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: target.as_os_str().to_os_string(), force: true };
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

/// What:
///  One snapshot the wrapper made ahead of an add,
///  as the real subvolume the kernel owns,
///  in the pool of the registration `admin`.
/// Why:
///   A member of the pool is a subvolume of the worktree,
/// and a control that planted a plain directory would prove
///  nothing about the refusal the kernel makes,
///  so every control of this pool is made of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = aSnapshotAhead(admin); // the pool folder, with one member holding one file
/// ```
fn a_snapshot_ahead(admin: &Path) -> PathBuf {
    let pool: PathBuf = crate::worktree_snapshot_ahead::pool_folder(admin);
    std::fs::create_dir_all(pool.as_path()).expect("the folder of the pool");
    let member: PathBuf = pool.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("a snapshot the wrapper made ahead of an add");
    std::fs::write(member.join("tracked.txt"), b"the worktree's own file\n").expect("its files");
    return pool;
}

/// What:
///  A worktree whose Git directory holds one of the snapshots the wrapper made ahead of an add is destroyed with it,
///  registration and snapshot together,
///  and no record of an interrupted removal is left.
/// Why:
///   The destroy is one kernel call while the registration is emptied after it,
/// and that emptying refuses a nested
///  subvolume:
/// without the clear the pool made every such removal end with the files gone and the
///  registration kept,
/// with a notice on every later command (the incident of 2026-11-11,
/// `git worktree remove` of a
///  worktree whose own Git directory held this program's snapshot ahead).
///  A real subvolume is the only object that proves it.
#[test]
fn btrfs_a_worktree_whose_git_directory_holds_a_snapshot_ahead_is_removed_with_it() {
    let Some(root) = btrfs_fixture("remove-snapshot-ahead") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/wt");
    let pool: PathBuf = a_snapshot_ahead(admin.as_path());
    let removed: bool = served(&repo, &worktree);
    assert_eq!(removed, removal_allowed(root.as_path()), "the removal is served exactly where the mount allows the destroy");
    if removed {
        assert!(!worktree.exists(), "the worktree is gone");
        assert!(!admin.exists(), "the registration went with it");
        assert!(!pool.exists(), "and the snapshot this program made ahead of an add went before the destroy");
        assert!(!repo.join(".git/monochromatic-git-policy-cli/recorded-destroys").exists(), "nothing of the removal is left for a later command to finish");
    } else {
        assert!(worktree.join("tracked.txt").is_file(), "a mount that cannot destroy takes nothing");
        assert!(pool.exists(), "and the pool is where it was");
    }
    remove(root.as_path());
}

/// What:
///  A subvolume the person put in the Git directory of the worktree refuses the removal while the worktree is whole,
///  and the report names it and what it holds.
/// Why:
///   The registration is emptied after the destroy,
/// and a registration holding a volume this program did not make can
///  never be emptied,
/// so a removal that destroyed first left the files gone,
/// the registration stuck,
/// and a notice on
///  every later command,
/// with nothing the person could do while their worktree was already gone.
///  The refusal comes before anything irreversible and names the one thing that stands in the way.
#[test]
fn btrfs_a_subvolume_of_the_persons_own_in_the_git_directory_refuses_before_the_destroy() {
    let Some(root) = btrfs_fixture("remove-git-dir-foreign") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/wt");
    let foreign: PathBuf = admin.join("a-volume-of-my-own");
    subvolume_create_at(foreign.as_path()).expect("a subvolume the person made");
    std::fs::write(foreign.join("notes.txt"), b"what I was in the middle of\n").expect("a file the person wrote");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path() });
    if !removal_allowed(root.as_path()) {
        // Nothing here can destroy a subvolume, so the removal is Git's and nothing of the wrapper's runs.
        assert_eq!(outcome, RemoveOutcome::LeftToGit);
        return;
    }
    let RemoveOutcome::Refused(report) = outcome else { panic!("a registration the kernel can never empty is refused, but it was {outcome:?}") };
    assert!(report.contains(foreign.to_str().expect("UTF-8")), "the report names the subvolume that stands in the way: {report}");
    assert!(report.contains("Nothing was removed"), "and says nothing changed: {report}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is where it was");
    assert!(admin.is_dir(), "and so is its registration");
    assert_eq!(std::fs::read(foreign.join("notes.txt")).expect("the file the person wrote is still there"), b"what I was in the middle of\n");
    remove(root.as_path());
}

/// What:
///  The pool a clear renamed and was killed before emptying,
/// in the shape a kill in that moment leaves:
///  a folder under a taken name holding the member the clear had not reached.
/// Why:
///   The rename is what a clear does before it deletes a member,
///  so a control that has no such folder proves nothing
///  about the leftover,
/// and the member has to be a real subvolume to stand where the walk refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = aPoolAKilledClearLeft(admin); // the folder the clear had renamed
/// ```
fn a_pool_a_killed_clear_left(admin: &Path) -> PathBuf {
    let data: PathBuf = admin.join("monochromatic-git-policy-cli");
    let left: PathBuf = data.join("removing-0f0e0d0c-0b0a-4908-8706-050403020100");
    std::fs::create_dir_all(left.as_path()).expect("the pool a killed clear had renamed");
    let member: PathBuf = left.join(".cli-git-worktree-ahead-0123456789abcdef-000001a1cae683dc-0123456789abcdef0123456789abcdef");
    subvolume_create_at(member.as_path()).expect("the member a killed clear left under its taken name");
    std::fs::write(member.join("tracked.txt"), b"a copy of the worktree\n").expect("its files");
    return left;
}

/// What:
///  A pool a clear renamed and was killed before emptying is taken with the next removal,
///  so the removal a person runs again finishes by itself.
/// Why:
///   A clear renames a pool to a name no fill reads before it deletes a member;
/// a command killed in that moment leaves
///  a member under that name,
/// and a removal that took only the pool's own name would refuse its own leftover for
///  ever (the incident of 2026-11-11 named exactly such a member).
#[test]
fn btrfs_a_pool_a_killed_clear_left_under_its_taken_name_goes_with_the_removal() {
    let Some(root) = btrfs_fixture("remove-pool-left-taken") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/wt");
    let left: PathBuf = a_pool_a_killed_clear_left(admin.as_path());
    let removed: bool = served(&repo, &worktree);
    assert_eq!(removed, removal_allowed(root.as_path()), "the removal is served exactly where the mount allows the destroy");
    if removed {
        assert!(!admin.exists(), "the registration went with the leftover of the killed clear");
        assert!(!left.exists(), "and the leftover is gone");
    } else {
        assert!(left.exists(), "a mount that cannot destroy takes nothing");
    }
    remove(root.as_path());
}
