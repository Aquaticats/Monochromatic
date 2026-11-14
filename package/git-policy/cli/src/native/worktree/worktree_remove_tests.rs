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
    return remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan, common_dir: common_dir.as_path(), last_look: None }) == RemoveOutcome::Removed;
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
///  A worktree that holds a subvolume below its root is removed around it:
///  the subvolume and the directories that lead to it stay,
///  everything else of the worktree goes,
///  and the worktree is forgotten in Git.
/// Why:
///   The kernel refuses `BTRFS_IOC_SNAP_DESTROY` for a subvolume that contains another one,
/// so the destroy could never
///  complete,
/// and Git's own removal unlinks the inner worktree's files:
///  the project's decision of 2026-11-12 ("Remove around it") is that the rest goes and the volume stays.
/// A nested linked worktree is the shape a person meets,
/// and a subvolume of their own is the same to the kernel.
#[test]
fn btrfs_a_worktree_holding_a_nested_subvolume_is_removed_around_it() {
    let Some(root) = btrfs_fixture("remove-nested-volume") else { return };
    let (repo, worktree, inner) = a_worktree_holding_a_nested_worktree(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    if !removal_allowed(root.as_path()) {
        // Nothing here can destroy a subvolume, and a tree holding a volume is never Git's, so it refuses and changes nothing.
        assert!(matches!(outcome, RemoveOutcome::Refused(_)), "it was {outcome:?}");
        assert!(worktree.join("tracked.txt").is_file(), "nothing of the worktree went");
        return;
    }
    let RemoveOutcome::RemovedAround(left) = outcome else { panic!("a worktree holding a subvolume is removed around it, but it was {outcome:?}") };
    assert_eq!(left.len(), 1, "the report names the one volume: {left:?}");
    assert_eq!(left[0].path, inner, "the nested worktree is what was left");
    let report: String = crate::worktree_removal_goes_around_a_foreign_volume::around_removal_report(worktree.as_path(), left.as_slice());
    assert!(report.contains(inner.to_str().expect("UTF-8")), "the report names the volume: {report}");
    assert!(report.contains("Git no longer lists it"), "and that the worktree is gone from Git: {report}");
    assert!(report.contains("free for `git worktree add`"), "and what frees the path: {report}");
    assert!(inner.join("notes.txt").is_file(), "the worktree inside it keeps the file the person wrote");
    assert_eq!(std::fs::read(inner.join("tracked.txt")).expect("the change there"), b"not committed yet\n", "and the change the person made");
    assert!(!worktree.join("tracked.txt").exists(), "every other entry of the worktree is removed");
    assert!(!repo.join(".git/worktrees/wt").exists(), "the worktree's own registration is gone");
    let mut kept: Vec<String> = Vec::new();
    for found in std::fs::read_dir(repo.join(".git/worktrees")).expect("the registrations").flatten() {
        let text: String = std::fs::read_to_string(found.path().join("gitdir")).expect("a registration's back-link");
        if text.contains(inner.to_str().expect("a UTF-8 fixture path")) {
            kept.push(found.file_name().to_string_lossy().into_owned());
        }
    }
    assert_eq!(kept.len(), 1, "the nested worktree's own registration stays: {kept:?}");
    remove(root.as_path());
}

/// What:
///  A stand-in for the kernel and Git that delegates every step to the real one,
///  except that it plants a linked worktree inside the worktree being removed
///  at the moment the removal hands its subvolume over,
///  which is between the walk that refuses a volume below the worktree and the destroy.
/// Why:
///   The walk and the destroy are not one step,
/// and the review's finding is the window between them:
/// an
///  add that makes a nested worktree there makes the kernel refuse the destroy (`ENOTEMPTY`),
/// and the
///  removal then handed the tree to Git,
/// whose own removal unlinks the nested worktree's files (a review finding,
///  F1).
/// A control cannot time a race,
/// so it makes the second step stand for the moment the first one ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlantedAtTheHandOver implements RemoveOperations { handOver() { plant(); return Err('destroy here') } }
/// ```
struct PlantedAtTheHandOver<'a> {
    /// The repository every registration is made in.
    repo: &'a Path,
    /// The folder the planted worktree stands in.
    folder: &'a Path,
    /// The branch the planted worktree has checked out.
    branch: &'a str,
    /// Whether the plant happened,
    /// so the stand-in plants once.
    planted: std::cell::Cell<bool>,
}

impl RemoveOperations for PlantedAtTheHandOver<'_> {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return LinuxOperations.is_subvolume_root(path);
    }

    fn removal_allowed(&self, path: &Path) -> bool {
        return LinuxOperations.removal_allowed(path);
    }

    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String> {
        return LinuxOperations.check_clean(request, admin, common_dir);
    }

    fn destroy(&self, held: &Held) -> Result<(), String> {
        // The real call, so the refusal a nested subvolume causes is the kernel's own.
        return LinuxOperations.destroy(held);
    }

    fn hand_over(&self, _context: &GitContext, _common_dir: &Path, _held: &Held, _target: &Path, _registration: &std::ffi::OsStr) -> Result<(), String> {
        if !self.planted.replace(true) {
            // The folder above the worktree, and then the subvolume at the name it takes, as a nested worktree stands.
            std::fs::create_dir_all(self.folder.parent().expect("the folder the planted worktree stands in")).expect("the folder the planted worktree stands in");
            subvolume_create_at(self.folder).expect("the subvolume of the planted worktree");
            git(self.repo, &["worktree", "add", "--quiet", "-b", self.branch, self.folder.to_str().expect("a UTF-8 fixture path")]);
            std::fs::write(self.folder.join("notes.txt"), b"what I was in the middle of\n").expect("a file the person wrote");
        }
        return Err(String::from("this stand-in hands nothing to the service, so the subvolume is destroyed here"));
    }
}

/// What:
///  A linked worktree added between the walk that looks for a volume and the destroy
///  is left by the removal and the rest of the worktree goes,
///  so the tree is never handed to Git,
///  whose removal would unlink the worktree's files.
/// Why:
///   The walk proves what stands below the worktree at one moment and the destroy acts at another,
/// and the
///  worst case of that window must stay no worse than Git's own for the files a person wrote:
/// the destroy is refused by
///  the kernel,
///  a volume the tree now holds is gone around,
///  and the worktree is forgotten while what was not the removal's stays.
#[test]
fn btrfs_a_worktree_added_between_the_walk_and_the_destroy_is_left_and_the_rest_is_removed() {
    let Some(root) = btrfs_fixture("remove-nested-volume-window") else { return };
    if !removal_allowed(root.as_path()) {
        // Nothing here can destroy a subvolume, so the walk never runs and the removal is Git's, as it always was.
        return;
    }
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    // What Git calls an entry below `.claude` is ignored here, so the outer worktree stays provably clean when the
    // planted worktree appears below it.
    std::fs::write(repo.join(".gitignore"), b"/.claude/\n").expect("the ignore rule");
    git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join("wt");
    subvolume_create_at(worktree.as_path()).expect("subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let outer: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("canonical");
    for name in ["tracked.txt", ".gitignore"] {
        age(outer.join(name).as_path());
    }
    git(outer.as_path(), &["update-index", "--refresh"]);
    let planted: PathBuf = outer.join(".claude/worktrees/inner");
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&outer);
    let operations = PlantedAtTheHandOver { repo: repo.as_path(), folder: planted.as_path(), branch: "inner", planted: std::cell::Cell::new(false) };
    let outcome: RemoveOutcome = remove_with(&operations, &RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    let RemoveOutcome::RemovedAround(left) = outcome else { panic!("the worktree is removed around the worktree that appeared, but it was {outcome:?}") };
    assert_eq!(left.len(), 1, "the report names the one volume: {left:?}");
    assert_eq!(left[0].path, std::fs::canonicalize(planted.as_path()).expect("canonical"), "the worktree that appeared is what was left");
    assert_eq!(std::fs::read(planted.join("notes.txt")).expect("the file the person wrote is still there"), b"what I was in the middle of\n");
    assert!(!outer.join("tracked.txt").exists(), "every other entry of the worktree is removed");
    assert!(!repo.join(".git/worktrees/wt").exists(), "and its registration is gone");
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
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
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

/// What:
///  The names of the entries of `folder` that begin with `prefix`,
///  which is how a control finds what a removal left behind without knowing an id.
/// Why:
///  A name built from a subvolume id is no number a control can guess,
/// and what it asserts is that one entry of that shape stands where the
///  service will delete it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = readdirSync(folder).filter((name) => name.startsWith(prefix)).map((name) => join(folder, name));
/// ```
fn entries_with_prefix(folder: &Path, prefix: &str) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let name: std::ffi::OsString = entry.file_name();
        if name.to_str().is_some_and(|text| { return text.starts_with(prefix) }) {
            found.push(entry.path());
        }
    }
    return found;
}

/// What:
///  The subvolume of one snapshot ahead of an add is moved out of the registration for the commit service to delete,
///  with a record that names where it stands,
///  and nothing of it is deleted by the person's command.
/// Why:
///   The destroy is a btrfs call that takes the filesystem's write lock against every snapshot in flight,
/// so a
///  removal that made it waited for another add's snapshot to commit,
/// which is what the hand-over of the worktree's own
///  subvolume exists to keep off the person's path
///  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`,
/// a review finding,
/// F5).
#[test]
fn btrfs_a_snapshot_ahead_is_left_for_the_service_instead_of_destroyed_by_the_removal() {
    let Some(root) = btrfs_fixture("remove-snapshot-ahead-left") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let admin: PathBuf = repo.join(".git/worktrees/wt");
    let pool: PathBuf = a_snapshot_ahead(admin.as_path());
    let mut members: Vec<PathBuf> = entries_with_prefix(pool.as_path(), ".cli-git-worktree-ahead-");
    let member: PathBuf = members.remove(0);
    let removed: bool = served(&repo, &worktree);
    assert_eq!(removed, removal_allowed(root.as_path()), "the removal is served exactly where the mount allows the destroy");
    if !removed {
        assert!(member.is_dir(), "a mount that cannot destroy takes nothing");
        remove(root.as_path());
        return;
    }
    assert!(!pool.exists(), "no fill finds a pool under its own name any more");
    assert!(!member.exists(), "and the member is not in the registration");
    let common: PathBuf = repo.join(".git");
    let left: Vec<PathBuf> = entries_with_prefix(crate::worktree_removal_clears_the_ahead_pool::common_data_folder(common.as_path()).as_path(), "cli-git-deleting-");
    assert_eq!(left.len(), 1, "the member stands where the service deletes it, and nowhere else: {left:?}");
    assert_eq!(std::fs::read(left[0].join("tracked.txt")).expect("the member still holds its files"), b"the worktree's own file\n", "nothing of the snapshot was deleted");
    let pending: PathBuf = crate::service_work::work_directory(common.as_path()).join("pending");
    let records: String = entries_with_prefix(pending.as_path(), "subvolume-delete-").iter().filter_map(|path| { return std::fs::read_to_string(path).ok() }).collect();
    assert!(records.contains(left[0].display().to_string().as_str()), "a record names the place the member stands, so the service deletes it: {records}");
    remove(root.as_path());
}

/// What:
///  A subvolume this program already left for the service,
///  standing inside the worktree being removed,
///  is moved out of that tree as well,
///  so the removal of the outer worktree finishes.
/// Why:
///  A removal renames a nested worktree's subvolume inside the folder it holds,
/// and that folder was the outer worktree
///  when the nested one stood inside it:
/// the outer destroy then met a subvolume the wrapper itself made,
///  refused,
///  and told the person to delete the wrapper's own leftover with the btrfs tool (a reported case,
///  2026-11-11T22:30:57-05:00).
#[test]
fn btrfs_a_pending_deletion_inside_a_worktree_is_taken_with_the_removal() {
    let Some(root) = btrfs_fixture("remove-pending-deletion-inside") else { return };
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    std::fs::write(repo.join(".gitignore"), b"/.claude/\n").expect("the ignore rule");
    git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join("wt");
    subvolume_create_at(worktree.as_path()).expect("subvolume");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let outer: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("canonical");
    let leftover: PathBuf = outer.join(".claude/worktrees/cli-git-deleting-000000000001f5f7");
    std::fs::create_dir_all(leftover.parent().expect("the folder the leftover stands in")).expect("the folder a nested worktree stands in");
    subvolume_create_at(leftover.as_path()).expect("a subvolume this program left for the service");
    std::fs::write(leftover.join("tracked.txt"), b"a copy nobody owns any more\n").expect("its files");
    for name in ["tracked.txt", ".gitignore"] {
        age(outer.join(name).as_path());
    }
    git(outer.as_path(), &["update-index", "--refresh"]);
    let removed: bool = served(&repo, &outer);
    assert_eq!(removed, removal_allowed(root.as_path()), "the removal is served exactly where the mount allows the destroy");
    if !removed {
        assert!(leftover.is_dir(), "a mount that cannot destroy takes nothing");
        remove(root.as_path());
        return;
    }
    assert!(!outer.exists(), "the worktree is gone, so its own subvolume and the leftover both left it");
    assert!(!leftover.exists(), "the wrapper's own leftover is not below the worktree any more");
    let left: Vec<PathBuf> = entries_with_prefix(root.as_path(), "cli-git-deleting-");
    assert_eq!(left.len(), 2, "the worktree's subvolume and its leftover stand beside it, named by their ids: {left:?}");
    let copies: Vec<Vec<u8>> = left.iter().filter_map(|path| { return std::fs::read(path.join("tracked.txt")).ok() }).collect();
    assert!(copies.contains(&b"a copy nobody owns any more\n".to_vec()), "and the leftover still holds what it held: {copies:?}");
    remove(root.as_path());
}

/// What:
///  A volume that somebody else removes while the removal's pass runs leaves an emptied tree,
///  and the removal ends as the around route's ending:
///  the root went with one `rmdir`,
///  nothing was destroyed in one step,
///  and nothing waits for the commit service.
/// Why:
///   `Removed` is the ending of a removal whose subvolume a destroy took,
/// and the command that prints a line for it promises a
///  delete the commit service still has to make.
/// On this path there is none,
/// so an ending of its own is what keeps the words true;
/// the control
///  makes the volume go at the one moment the state can change, between the pass and the look,
/// rather than racing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// actAfterTheAroundPass((target) => emptyAndRemove(`${target}/snap`));
/// expect(outcome.name).toBe('RemovedAroundNothingLeft');
/// ```
#[test]
fn btrfs_a_volume_gone_when_the_pass_ends_leaves_the_around_routes_own_ending() {
    let Some(root) = btrfs_fixture("remove-around-volume-gone") else { return };
    let (repo, worktree, volume) = a_worktree_holding_a_volume_nobody_will_delete(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let emptied: PathBuf = volume.clone();
    crate::subvolume_remove_listing_test_support::act_after_the_around_pass(move |_target: &Path| {
        // The volume the pass went around is emptied and its root removed, as another command's `btrfs subvolume delete` of an
        //  emptied one does: the kernel allows `rmdir` of an empty subvolume root.
        std::fs::remove_file(emptied.join("precious.txt")).expect("the file the person put there");
        std::fs::remove_dir(emptied.as_path()).expect("an empty subvolume root goes with rmdir");
    });
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    crate::subvolume_remove_listing_test_support::stop_acting();
    let ended: String = format!("{outcome:?}");
    assert!(
        ended.starts_with("RemovedAroundNothingLeft"),
        "the removal tells the around route's ending apart from a destroy's: {ended}"
    );
    assert!(!worktree.exists(), "the emptied root is gone");
    assert!(!repo.join(".git/worktrees/wt").exists(), "and its registration is gone");
    remove(root.as_path());
}

/// What:
///  A volume somebody else removes while the pass runs ends the same way in a worktree that is no subvolume,
///  where no destroy could ever have run.
/// Why:
///   The line for a destroy would tell a person of a subvolume that never was,
/// and the ending has to be the same one either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(plainWorktreeOutcome.name).toBe('RemovedAroundNothingLeft');
/// ```
#[test]
fn btrfs_a_volume_gone_when_the_pass_ends_ends_the_same_way_in_a_plain_directory_worktree() {
    let Some(root) = btrfs_fixture("remove-around-volume-gone-plain") else { return };
    let (repo, worktree, volume) = a_worktree_holding_a_volume_nobody_will_delete_in_a_plain_directory(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let emptied: PathBuf = volume.clone();
    crate::subvolume_remove_listing_test_support::act_after_the_around_pass(move |_target: &Path| {
        std::fs::remove_file(emptied.join("precious.txt")).expect("the file the person put there");
        std::fs::remove_dir(emptied.as_path()).expect("an empty subvolume root goes with rmdir");
    });
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    crate::subvolume_remove_listing_test_support::stop_acting();
    let ended: String = format!("{outcome:?}");
    assert!(
        ended.starts_with("RemovedAroundNothingLeft"),
        "a worktree that is no subvolume ends the same way: {ended}"
    );
    assert!(!worktree.exists(), "the emptied root is gone");
    assert!(!repo.join(".git/worktrees/plain").exists(), "and its registration is gone");
    remove(root.as_path());
}

/// What:
///  A repository with one worktree that is a subvolume,
///  and a subvolume of the person's inside it that nothing will delete:
///  the control's act does.
/// Why:
///   Both controls of the emptied ending need the same state,
///  and only the shape of the worktree differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, worktree, volume } = aWorktreeHoldingAVolumeNobodyWillDelete(root);
/// ```
fn a_worktree_holding_a_volume_nobody_will_delete(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    return a_worktree_with_a_volume(root, "wt", "topic", true);
}

/// The same for a worktree Git makes as an ordinary directory,
/// which is no subvolume at all.
fn a_worktree_holding_a_volume_nobody_will_delete_in_a_plain_directory(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    return a_worktree_with_a_volume(root, "plain", "plain", false);
}

/// What:
///  A repository with one clean worktree that holds a subvolume a person put there.
/// Why:
///   The volume is what puts the removal on the around route;
///  the worktree's own entries are deleted by the pass,
///  so no control may
///  leave one of them untracked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { repo, worktree, volume } = aWorktreeWithAVolume(root, 'wt', 'topic', asSubvolume);
/// ```
fn a_worktree_with_a_volume(root: &Path, name: &str, branch: &str, as_subvolume: bool) -> (PathBuf, PathBuf, PathBuf) {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    // The volume stands under an ignored name, because a volume Git lists as untracked keeps an unforced removal from happening.
    std::fs::write(repo.join(".gitignore"), b"/snap/\n").expect("the ignore rule");
    git(repo.as_path(), &["add", "--", "tracked.txt", ".gitignore"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    let worktree: PathBuf = root.join(name);
    if as_subvolume {
        subvolume_create_at(worktree.as_path()).expect("subvolume");
    }
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", branch, format!("../{name}").as_str()]);
    let outer: PathBuf = std::fs::canonicalize(worktree.as_path()).expect("canonical");
    let volume: PathBuf = outer.join("snap");
    subvolume_create_at(volume.as_path()).expect("a subvolume the removal did not make");
    std::fs::write(volume.join("precious.txt"), b"what I put there\n").expect("a file the person put there");
    for tracked in ["tracked.txt", ".gitignore"] {
        age(outer.join(tracked).as_path());
    }
    git(outer.as_path(), &["update-index", "--refresh"]);
    return (repo, outer, volume);
}

/// A stand-in for the kernel and Git that fails the hand-over to the commit service
/// and lets everything else happen as it does,
/// which is the removal that destroys the subvolume in place.
///
/// Why:
///  One subvolume can be handed over only once,
/// and a second command that proved the same worktree clean finds the hand-over
///  refused;
/// it then destroys the subvolume here.
/// That is a race between two removals, so a control makes it a stated step.
struct HandOverRefused;

impl RemoveOperations for HandOverRefused {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return LinuxOperations.is_subvolume_root(path);
    }

    fn removal_allowed(&self, path: &Path) -> bool {
        return LinuxOperations.removal_allowed(path);
    }

    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String> {
        return LinuxOperations.check_clean(request, admin, common_dir);
    }

    fn destroy(&self, held: &Held) -> Result<(), String> {
        return LinuxOperations.destroy(held);
    }

    fn hand_over(&self, _context: &GitContext, _common_dir: &Path, _held: &Held, _target: &Path, _registration: &std::ffi::OsStr) -> Result<(), String> {
        return Err(String::from("another command took the subvolume this one proved clean, so it is destroyed here"));
    }
}

/// What:
///  A removal that destroys the subvolume here,
///  because another command took the hand-over first,
///  ends as its own ending and not as the hand-over's.
/// Why:
///   The line the person's command prints for a hand-over promises a delete the commit service still has to make,
/// and on this
///  path the files are already gone when the command returns;
/// a control reads the ending, and the words control beside it
///  reads what that ending says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(outcome.name).toBe('DestroyedInPlace');
/// ```
#[test]
fn btrfs_a_destroy_here_after_a_refused_hand_over_ends_as_its_own_ending() {
    let Some(root) = btrfs_fixture("remove-destroy-here") else { return };
    if !removal_allowed(root.as_path()) {
        // Without the mount option no destroy runs at all, so the hand-over is the only route and this state is unreachable.
        return;
    }
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    let outcome: RemoveOutcome = remove_with(&HandOverRefused, &RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    let ended: String = format!("{outcome:?}");
    assert!(ended.starts_with("DestroyedInPlace"), "the removal tells a destroy here apart from a hand-over: {ended}");
    assert!(!worktree.exists(), "the files are gone when the command returns");
    assert!(!repo.join(".git/worktrees/wt").exists(), "and the registration is gone");
    remove(root.as_path());
}

/// What:
///  A removal of this worktree that was cut short in its pass and is run again prints no warning,
///  because the record of that pass still stands with the same content.
/// Why:
///   The record was published create-new,
/// so writing it again on the resume failed `EEXIST` and the command said the pass was
///  *not recorded* - that a command killed in it leaves a worktree only `--force` finishes - over a record
///  that was there all along (the third review's finding 1,
/// `repro2-kill-resume.ts`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(linesWritten.join('\n')).not.toContain('is not recorded');
/// ```
#[test]
fn btrfs_a_resume_writes_no_warning_that_its_pass_is_not_recorded() {
    let Some(root) = btrfs_fixture("remove-resume-quiet") else { return };
    let (repo, worktree, _volume) = a_worktree_holding_a_volume_nobody_will_delete(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let plan: RemovePlan = plan_for(&worktree);
    // The state a command killed inside the pass leaves: the record of that pass, and the entries it removed.
    let id: std::ffi::OsString = std::fs::read_dir(common_dir.join("worktrees")).expect("the registrations").flatten().next().expect("one registration").file_name();
    let admin_identity = crate::fs_beneath::identity_of(&crate::fs_beneath::open_root(common_dir.join("worktrees").join(id.as_os_str()).as_path()).expect("the registration")).expect("its identity");
    let worktree_identity = crate::fs_beneath::identity_of(&crate::fs_beneath::open_root(worktree.as_path()).expect("the worktree")).expect("its identity");
    crate::worktree_removal_around_begun::record_around_begun(
        common_dir.as_path(),
        crate::worktree_removal_around_begun::WhichRemoval { id: id.as_os_str(), admin: admin_identity, worktree: worktree_identity, target: worktree.as_path() },
    )
    .expect("the record a killed pass leaves");
    for tracked in ["tracked.txt", ".gitignore"] {
        std::fs::remove_file(worktree.join(tracked)).expect("an entry the pass removed");
    }
    let _ = crate::diagnostic_log::take_lines_written();
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path(), last_look: None });
    let written: String = crate::diagnostic_log::take_lines_written().join("\n");
    assert!(matches!(outcome, RemoveOutcome::RemovedAround(_)), "the resume finishes the removal: {outcome:?}");
    assert!(!written.contains("is not recorded"), "and says nothing about a pass that is not recorded, because the record stands: {written}");
    remove(root.as_path());
}
