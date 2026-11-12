//! Controls for the subvolume removal of a worktree:
//! the plan's accepted and declined forms,
//! and,
//! on btrfs,
//! the
//!  removal of a clean subvolume worktree (where the mount allows it) and every case that must be left to Git.

use super::*;
use crate::fs_btrfs_ioctl::subvolume_create_at;
use crate::fs_btrfs_test_support::{btrfs_fixture, removal_allowed};
use crate::held_commit_record::{Held, write as write_held};
use crate::ref_name::RefName;
use crate::test_support::{age, git, git_context, git_text, remove, repository};

/// The batch number of the held commit these controls keep.
const HELD_SEQ: u64 = 7;

/// What:
///  A commit a reference lock held,
///  named only by the worktree's own reference and by its held record.
/// Why:
///   This is the state the wrapper's own hold leaves,
///  and the record is what the keeper reads:
///  a reference without one is no batch any command could settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdACommitOfTheWorktree(repo, worktree, admin, 7);
/// ```
fn hold_a_commit_of_the_worktree(repo: &Path, worktree: &Path, admin: &Path, seq: u64) -> String {
    let tree: String = git_text(worktree, &["rev-parse", "HEAD^{tree}"]);
    let held: String = git_text(worktree, &["commit-tree", tree.as_str(), "-p", "HEAD", "-m", "the held edit"]);
    let reference: String = format!("refs/worktree/cli-git-service/{seq}");
    git(worktree, &["update-ref", "--no-deref", reference.as_str(), held.as_str()]);
    // The branch, its reflog and every other ref name nothing of the held commit, so the worktree's own reference is the only
    //  holder, which is the state a commit whose branch move met a reference lock leaves.
    git(repo, &["reflog", "expire", "--expire=now", "--all"]);
    let common_dir: PathBuf = admin.parent().and_then(Path::parent).expect("the common Git directory of the registration").to_path_buf();
    let record: Held = Held {
        seq,
        journal: admin.join("journal"),
        repository: worktree.to_path_buf(),
        common_dir: common_dir.clone(),
        git_dir: admin.to_path_buf(),
        reference: RefName::from_text("refs/heads/topic"),
        old: git_text(worktree, &["rev-parse", "HEAD"]),
        new: held.clone(),
        commits: vec![held.clone()],
        index: admin.join("index"),
        reflog: b"commit: the held edit".to_vec(),
        locks: vec![common_dir.join("refs/heads/topic.lock")],
        created: String::from("2026-01-01T00:00:00+00:00"),
    };
    write_held(admin, &record).expect("the record of the hold");
    assert_eq!(
        git_text(worktree, &["for-each-ref", "--contains", held.as_str(), "--format=%(refname)"]).trim(),
        reference,
        "nothing but the worktree's own reference names the held commit"
    );
    return held;
}

/// Plant the lock file an update of `name` refused,
/// as a killed or a concurrent Git leaves it.
fn plant_lock(common_dir: &Path, name: &str) {
    let lock: PathBuf = common_dir.join(format!("{name}.lock"));
    std::fs::create_dir_all(lock.parent().expect("the directory of the lock")).expect("the lock's directory");
    std::fs::write(lock.as_path(), b"").expect("the lock file");
}

/// What:
///  The name of the reference of the common Git directory a held commit of batch `seq` is kept on,
///  and the name beside it.
/// Why:
///  The controls read the references the keeper writes by the names the keeper builds,
///  so a change of that shape breaks them rather than passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keptName(seq); const rescue = rescueName(seq);
/// ```
fn the_kept_names(seq: u64) -> (String, String) {
    return (crate::service_scratch_ref_of_a_dropped_batch::kept_ref(seq), crate::service_scratch_ref_of_a_dropped_batch::rescue_ref(seq));
}

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
    return matches!(remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan, common_dir: common_dir.as_path() }), RemoveOutcome::Removed(_));
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

/// What:
///  A commit a reference lock holds in the worktree this destroy removes reaches a reference of the common Git
///  directory before the worktree's own Git directory goes,
///  and the person reads where it is kept.
/// Why:
///  The held record and the per-worktree reference live in the administrative directory this removal empties,
///  so the destroy takes both and the commit is named by nothing,
///  and Git's own maintenance then removes it.
///  A landed sweep keeps every held commit at its start,
///  and a batch can meet a reference lock in the whole time after that,
///  so the keeping belongs in the process that destroys,
///  as it already is in the service's own removal
///  (a review of the landed removal,
///  finding 1;
///  a review of the landed sweep,
///  finding 1).
#[test]
fn btrfs_a_held_commit_of_the_worktree_is_kept_before_the_destroy() {
    let Some(root) = btrfs_fixture("remove-held-commit") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees").join("wt");
    let held: String = hold_a_commit_of_the_worktree(repo.as_path(), worktree.as_path(), admin.as_path(), HELD_SEQ);
    let plan: RemovePlan = plan_for(worktree.as_path());
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path() });
    if !removal_allowed(root.as_path()) {
        // A mount that forbids the destroy leaves the removal to Git, and nothing here may take the holder either.
        assert_eq!(outcome, RemoveOutcome::LeftToGit, "a mount without the removal option is Git's removal");
        assert!(admin.join(format!("refs/worktree/cli-git-service/{HELD_SEQ}")).exists(), "the only holder is where it was");
        remove(root.as_path());
        return;
    }
    let kept_name: String = the_kept_names(HELD_SEQ).0;
    let RemoveOutcome::Removed(kept) = outcome else { panic!("a clean subvolume worktree is destroyed, but the removal answered {outcome:?}") };
    assert!(kept.iter().any(|line| return line.contains(kept_name.as_str())), "the person reads where the commit is kept: {kept:?}");
    assert!(!worktree.exists(), "the worktree is gone");
    assert!(!admin.exists(), "and its administrative directory with it");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", kept_name.as_str()]).trim(), held, "the kept reference names the commit the removal would have lost");
    git(repo.as_path(), &["gc", "--prune=now"]);
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", held.as_str()]), "commit", "Git's own maintenance keeps the commit");
    remove(root.as_path());
}

/// What:
///  A worktree whose held commit no reference of the repository can keep is not destroyed,
///  and the person reads which commit still needs a name.
/// Why:
///  The destroy deletes the only two things that name the commit,
///  so nothing may be destroyed while no reference names it:
///  the removal refuses before anything happened,
///  and the worktree,
///  its held record and its per-worktree reference stay exactly as they were.
#[test]
fn btrfs_a_worktree_whose_held_commit_no_reference_can_keep_is_not_destroyed() {
    let Some(root) = btrfs_fixture("remove-held-commit-refused") else { return };
    let (repo, worktree) = clean_subvolume_worktree(root.as_path());
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = common_dir.join("worktrees").join("wt");
    let held: String = hold_a_commit_of_the_worktree(repo.as_path(), worktree.as_path(), admin.as_path(), HELD_SEQ);
    let held_reference: PathBuf = admin.join(format!("refs/worktree/cli-git-service/{HELD_SEQ}"));
    assert!(held_reference.is_file(), "the control holds the commit the way the hold path leaves it");
    let (kept_name, rescue_name) = the_kept_names(HELD_SEQ);
    for name in [kept_name.as_str(), rescue_name.as_str()] {
        plant_lock(common_dir.as_path(), name);
    }
    let plan: RemovePlan = plan_for(worktree.as_path());
    let outcome: RemoveOutcome = remove_subvolume_worktree(&RemoveRequest { context: &git_context(), plan: &plan, common_dir: common_dir.as_path() });
    if !removal_allowed(root.as_path()) {
        assert_eq!(outcome, RemoveOutcome::LeftToGit, "a mount without the removal option is Git's removal");
        remove(root.as_path());
        return;
    }
    let RemoveOutcome::Refused(report) = outcome else { panic!("neither kept reference can be written, so the removal refuses, but it answered {outcome:?}") };
    assert!(report.contains(held.as_str()), "the report names the commit no reference could hold: {report}");
    assert!(report.contains("nothing was removed"), "and says nothing changed: {report}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is where it was");
    assert!(admin.is_dir(), "and its administrative directory");
    let per_worktree: String = format!("refs/worktree/cli-git-service/{HELD_SEQ}");
    assert_eq!(git_text(worktree.as_path(), &["rev-parse", per_worktree.as_str()]).trim(), held, "the per-worktree reference still names the commit");
    assert!(!common_dir.join(kept_name.as_str()).exists(), "and no kept reference was written");
    remove(root.as_path());
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
