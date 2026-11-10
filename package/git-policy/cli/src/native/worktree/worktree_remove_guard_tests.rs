//! Controls for the guards of the subvolume removal of a worktree,
//! run on a stand-in for the kernel so that they hold on
//!  any filesystem:
//! a worktree of another repository,
//! a name Git resolves by its last components,
//! a lock that cannot be
//!  looked up,
//! and objects swapped between the proof of cleanliness,
//! the destroy and the removal of the registration.

use super::*;
use crate::fs_beneath::handle_path;
use crate::push_failure_notice::{orphaned_push_failures, record_push_failure};
use crate::push_failure_notice_words::PushOutcome;
use crate::test_support::{age, fixture, git, git_context, git_output, remove, repository};
use crate::worktree_removal_interrupted_destroy::{finish_interrupted_destroys, record_destroy_with_owner, record_destroyed};
use std::cell::RefCell;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;

/// A hook that runs once,
/// at a chosen step of the removal.
type Hook = RefCell<Option<Box<dyn FnOnce()>>>;

/// A stand-in for the kernel and the cleanliness proof:
/// the worktrees listed are "subvolume roots",
/// the proof is the real
///  one,
/// and the destroy deletes the directory.
/// Each hook runs once,
/// at the named step,
/// so a control can swap an object
///  exactly where a concurrent process would.
struct Stand {
    /// The directories the stand-in calls subvolume roots.
    subvolumes: Vec<PathBuf>,
    /// Runs after the proof of cleanliness succeeded.
    after_proof: Hook,
    /// Runs inside the destroy,
    /// before the directory is deleted.
    during_destroy: Hook,
}

impl Stand {
    /// A stand-in with no hook.
    fn new(subvolumes: Vec<PathBuf>) -> Stand {
        return Stand { subvolumes, after_proof: RefCell::new(None), during_destroy: RefCell::new(None) };
    }

    /// Run the hook once.
    fn fire(hook: &Hook) {
        if let Some(action) = hook.borrow_mut().take() {
            action();
        }
    }
}

/// The real cleanliness proof,
/// the hooks,
/// and a deletion in place of the kernel call.
impl RemoveOperations for Stand {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return self.subvolumes.iter().any(|known: &PathBuf| { return known == path; });
    }

    fn removal_allowed(&self, _path: &Path) -> bool {
        return true;
    }

    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, _common_dir: &Path) -> Result<(), String> {
        // The proof is made against the repository the worktree's own pointers name, so that a worktree of another
        // repository passes it, and only the scope of the registration can stop it.
        let own_common_dir: &Path = admin.parent().and_then(Path::parent).ok_or_else(|| { return String::from("no common directory"); })?;
        check_clean(request.context, request.plan.target.as_path(), admin, own_common_dir).map_err(|refusal| { return refusal.reason; })?;
        Stand::fire(&self.after_proof);
        return Ok(());
    }

    fn destroy(&self, held: &Held) -> Result<(), String> {
        Stand::fire(&self.during_destroy);
        // Below the held parent, as the kernel call is: a path swapped since cannot redirect it.
        return std::fs::remove_dir_all(handle_path(&held.parent).join(held.name.as_os_str())).map_err(|error: std::io::Error| { return error.to_string(); });
    }
}

/// A repository with one tracked file.
fn repository_with_file(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = repository(root, name);
    std::fs::write(repo.join("tracked.txt"), b"tracked\n").expect("tracked");
    git(repo.as_path(), &["add", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=tracked"]);
    return repo;
}

/// A linked worktree at `directory`,
/// provably clean,
/// and its real path.
fn clean_worktree(repo: &Path, directory: &Path, branch: &str) -> PathBuf {
    git(repo, &["worktree", "add", "--quiet", "-b", branch, directory.to_str().expect("UTF-8")]);
    let canonical: PathBuf = std::fs::canonicalize(directory).expect("canonical");
    age(canonical.join("tracked.txt").as_path());
    git(canonical.as_path(), &["update-index", "--refresh"]);
    return canonical;
}

/// A plan that names `target` by `operand`.
fn plan_named(target: &Path, operand: &str) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: OsString::from(operand), force: false };
}

/// The same,
/// with the force the person passes as `-f`/`--force`.
fn forced_plan_named(target: &Path, operand: &str) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: OsString::from(operand), force: true };
}

/// Run the removal of `plan` in the repository `repo` on the stand-in.
fn run(stand: &Stand, repo: &Path, plan: &RemovePlan) -> RemoveOutcome {
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    return remove_with(stand, &RemoveRequest { context: &git_context(), plan, common_dir: common_dir.as_path() });
}

/// The administrative directory of the worktree registered at `worktree` in `repo`.
fn admin_of(repo: &Path, name: &str) -> PathBuf {
    return std::fs::canonicalize(repo.join(".git/worktrees").join(name)).expect("administrative directory");
}

/// What:
///  A worktree that holds a change is destroyed when the person forced the removal,
///  and left to Git when they did not.
/// Why:
///   Git deletes a worktree that holds changes only with `-f`/`--force` (`remove_worktree` in
/// `builtin/worktree.c`
///  calls `check_clean_worktree` only when `force` is zero),
///  so before this the forced removal,
/// which the wrapper's own
///  reports tell a person to run,
/// was the one removal that always unlinked every entry of the tree.
#[test]
fn a_forced_removal_of_a_changed_worktree_is_served_and_an_unforced_one_is_not() {
    let root: PathBuf = fixture("remove-guard-forced");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    std::fs::write(worktree.join("tracked.txt"), b"unsaved work\n").expect("a change");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    // Without the force the proof refuses the change, and Git words that refusal itself.
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert_eq!(std::fs::read(worktree.join("tracked.txt")).expect("the change is still there"), b"unsaved work\n");
    // With it the removal answers for the worktree, as Git's own `--force` does.
    assert_eq!(run(&stand, repo.as_path(), &forced_plan_named(worktree.as_path(), "../wt")), RemoveOutcome::Removed);
    assert!(!worktree.exists(), "the forced removal destroyed it");
    remove(root.as_path());
}

/// What:
///  A forced removal of a locked worktree is still Git's,
///  and nothing is destroyed.
/// Why:
///   Git requires the force twice for a locked worktree
/// (`if (force < 2) reason = worktree_lock_reason(wt)`),
/// and the
///  wrapper reads one force only,
/// so an entry left in the registration,
/// or a lock taken meanwhile,
/// has to stop the destroy.
#[test]
fn a_forced_removal_of_a_locked_worktree_is_left_to_git() {
    let root: PathBuf = fixture("remove-guard-forced-locked");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    std::fs::write(admin.join("locked"), b"keep this one\n").expect("lock");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &forced_plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert!(worktree.join("tracked.txt").is_file(), "the locked worktree is untouched");
    assert!(admin.join("locked").is_file(), "and so is its lock");
    remove(root.as_path());
}

/// A worktree that another repository registered is not this repository's to remove:
/// Git answers "is not a working tree".
#[test]
fn a_worktree_of_another_repository_is_left_to_git() {
    let root: PathBuf = fixture("remove-guard-other-repository");
    let repo_a: PathBuf = repository_with_file(root.as_path(), "a");
    let repo_b: PathBuf = repository_with_file(root.as_path(), "b");
    let foreign: PathBuf = clean_worktree(repo_b.as_path(), root.join("wb").as_path(), "tb");
    let stand: Stand = Stand::new(vec![foreign.clone()]);
    assert_eq!(run(&stand, repo_a.as_path(), &plan_named(foreign.as_path(), "../wb")), RemoveOutcome::LeftToGit);
    assert!(foreign.join("tracked.txt").is_file(), "the foreign worktree is untouched");
    assert!(repo_b.join(".git/worktrees/wb").is_dir(), "its registration too");
    // The same worktree asked of the repository that owns it is served.
    assert_eq!(run(&stand, repo_b.as_path(), &plan_named(foreign.as_path(), "../wb")), RemoveOutcome::Removed);
    remove(root.as_path());
}

/// Git matches the operand against the last components of every registered path before it resolves it as a path,
/// so a
///  directory that exists under the operand's name is not what Git removes when another worktree ends in that name.
#[test]
fn a_name_that_git_resolves_by_its_last_components_is_left_to_git() {
    let root: PathBuf = fixture("remove-guard-suffix");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let other: PathBuf = clean_worktree(repo.as_path(), root.join("x/other").as_path(), "to");
    let topic: PathBuf = clean_worktree(repo.as_path(), root.join("y/topic").as_path(), "tt");
    std::os::unix::fs::symlink("../x/other", repo.join("topic")).expect("link");
    let stand: Stand = Stand::new(vec![other.clone(), topic.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &plan_named(other.as_path(), "topic")), RemoveOutcome::LeftToGit);
    assert!(other.join("tracked.txt").is_file() && topic.join("tracked.txt").is_file(), "both worktrees are untouched");
    // A name whose only match is the target itself, and a path that matches no registered name, are served.
    assert_eq!(run(&stand, repo.as_path(), &plan_named(other.as_path(), "other")), RemoveOutcome::Removed);
    assert!(!other.exists());
    assert_eq!(run(&stand, repo.as_path(), &plan_named(topic.as_path(), "../y/topic")), RemoveOutcome::Removed);
    assert!(!topic.exists());
    remove(root.as_path());
}

/// A lock entry that cannot be looked up is not an absent one.
#[test]
fn a_lock_entry_that_cannot_be_looked_up_is_not_an_absent_lock() {
    let root: PathBuf = fixture("remove-guard-lock-lookup");
    std::fs::write(root.join("not-a-directory"), b"x").expect("file");
    std::fs::create_dir(root.join("admin")).expect("admin");
    assert!(lock_state(root.join("not-a-directory").as_path()).is_err(), "the lookup fails with ENOTDIR");
    assert_eq!(lock_state(root.join("admin").as_path()), Ok(false));
    std::fs::write(root.join("admin/locked"), b"reason\n").expect("locked");
    assert_eq!(lock_state(root.join("admin").as_path()), Ok(true));
    remove(root.as_path());
}

/// `git worktree lock` after the proof of cleanliness stops the removal.
#[test]
fn a_lock_taken_after_the_proof_stops_the_removal() {
    let root: PathBuf = fixture("remove-guard-late-lock");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    *stand.after_proof.borrow_mut() = Some(Box::new(move || { std::fs::write(admin.join("locked"), b"late\n").expect("lock"); }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert!(worktree.join("tracked.txt").is_file(), "the locked worktree is untouched");
    remove(root.as_path());
}

/// A different directory put at the path after the proof is not destroyed in place of the one that was proven.
#[test]
fn a_directory_swapped_in_after_the_proof_is_not_destroyed() {
    let root: PathBuf = fixture("remove-guard-swapped-target");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let (moved, at) = (root.join("wt.moved"), worktree.clone());
    *stand.after_proof.borrow_mut() = Some(Box::new(move || {
        std::fs::rename(at.as_path(), moved.as_path()).expect("move the proven directory away");
        std::fs::create_dir(at.as_path()).expect("replacement");
        std::fs::write(at.join("precious.txt"), b"unsaved work\n").expect("replacement file");
    }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert_eq!(std::fs::read(worktree.join("precious.txt")).expect("the replacement is kept"), b"unsaved work\n");
    assert!(root.join("wt.moved/tracked.txt").is_file(), "the proven directory is untouched");
    remove(root.as_path());
}

/// An ancestor replaced by a link after the proof does not send the destroy to the directory behind the link.
#[test]
fn an_ancestor_replaced_by_a_link_after_the_proof_does_not_redirect_the_destroy() {
    let root: PathBuf = fixture("remove-guard-swapped-ancestor");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("sub/wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let (sub, moved, elsewhere) = (root.join("sub"), root.join("sub.moved"), root.join("elsewhere"));
    *stand.after_proof.borrow_mut() = Some(Box::new(move || {
        std::fs::rename(sub.as_path(), moved.as_path()).expect("move the ancestor away");
        std::fs::create_dir_all(elsewhere.join("wt")).expect("replacement");
        std::fs::write(elsewhere.join("wt/precious.txt"), b"unsaved work\n").expect("replacement file");
        std::os::unix::fs::symlink(elsewhere.as_path(), sub.as_path()).expect("link");
    }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../sub/wt")), RemoveOutcome::LeftToGit);
    assert_eq!(std::fs::read(root.join("elsewhere/wt/precious.txt")).expect("the directory behind the link is kept"), b"unsaved work\n");
    assert!(root.join("sub.moved/wt/tracked.txt").is_file(), "the proven directory is untouched too");
    remove(root.as_path());
}

/// An ancestor replaced by another real directory after the proof is not the directory that held the worktree:
/// the held
///  parent still leads to the proven worktree,
/// but the path the person typed no longer does.
#[test]
fn an_ancestor_replaced_by_a_directory_after_the_proof_stops_the_removal() {
    let root: PathBuf = fixture("remove-guard-replaced-ancestor");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("sub/wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let (sub, moved) = (root.join("sub"), root.join("sub.moved"));
    *stand.after_proof.borrow_mut() = Some(Box::new(move || {
        std::fs::rename(sub.as_path(), moved.as_path()).expect("move the ancestor away");
        std::fs::create_dir(sub.as_path()).expect("another directory at the same path");
    }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../sub/wt")), RemoveOutcome::LeftToGit);
    assert!(root.join("sub.moved/wt/tracked.txt").is_file(), "the proven directory is untouched");
    remove(root.as_path());
}

/// Git refuses a worktree with a populated submodule ("working trees containing submodules cannot be moved or removed"),
///  so the fast path must too.
#[test]
fn a_worktree_with_a_populated_submodule_is_left_to_git() {
    let root: PathBuf = fixture("remove-guard-submodule");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let library: PathBuf = repository_with_file(root.as_path(), "library");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    git(worktree.as_path(), &["-c", "protocol.file.allow=always", "submodule", "add", "--quiet", library.to_str().expect("UTF-8"), "lib"]);
    git(worktree.as_path(), &["commit", "--quiet", "--message=submodule"]);
    for name in ["tracked.txt", ".gitmodules"] {
        age(worktree.join(name).as_path());
    }
    git(worktree.as_path(), &["update-index", "--refresh"]);
    // Which rule refuses it, so that the result is not an accident of file times.
    let common_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let proof = check_clean(&git_context(), worktree.as_path(), admin_of(repo.as_path(), "wt").as_path(), common_dir.as_path());
    assert_eq!(proof.map_err(|refusal| { return refusal.reason; }), Err(String::from("lib is a submodule")));
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert!(worktree.join("lib/tracked.txt").is_file(), "the submodule's files are untouched");
    remove(root.as_path());
}

/// A registration that cannot be removed after the destroy is reported as an incomplete removal that names the leftover
///  and the way to finish,
/// never as a success.
#[test]
fn a_registration_that_cannot_be_removed_after_the_destroy_is_an_incomplete_removal() {
    let root: PathBuf = fixture("remove-guard-incomplete");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let worktrees: PathBuf = repo.join(".git/worktrees");
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o555)).expect("read-only registrations");
    let outcome: RemoveOutcome = run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt"));
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o755)).expect("restore");
    let RemoveOutcome::Incomplete(report) = outcome else { panic!("a leftover registration is not a success: {outcome:?}") };
    assert!(!worktree.exists(), "the files are gone");
    assert!(report.contains(admin.to_str().expect("UTF-8")), "the report names the leftover: {report}");
    assert!(report.contains(worktree.to_str().expect("UTF-8")), "and the removed worktree: {report}");
    assert!(report.contains(format!("git -C {} worktree prune", repo.display()).as_str()), "and the command that finishes: {report}");
    remove(root.as_path());
}

/// What:
///  A removal whose files are gone and whose registration stayed is finished by the next command,
///  so `git worktree add` at that path works again without the person running `prune`,
///  and the records the registration still holds are shown instead of dying with it.
/// Why:
///   The state is what a kill between the destroy and the removal of the registration leaves:
/// the destroy is recorded
///  before the kernel call,
/// and the worktree directory is gone.
#[test]
fn a_removal_whose_files_are_gone_is_finished_by_the_next_command_and_keeps_its_records() {
    let root: PathBuf = fixture("remove-finish-interrupted");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    // A push failure nobody has seen yet lives in that registration and would go with it.
    record_push_failure(admin.as_path(), "refs/heads/topic", "0123456789abcdef0123456789abcdef01234567", PushOutcome::Refused, "refused by test policy").expect("notice");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    record_destroyed(common.as_path(), std::ffi::OsStr::new("wt"));
    remove(worktree.as_path());
    let refused = git_output(repo.as_path(), &["worktree", "add", "--quiet", "--detach", worktree.to_str().expect("UTF-8")]);
    assert!(!refused.status.success(), "a stale registration stops the add, which is the state to repair");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    assert!(problems.is_empty(), "{problems:?}");
    assert!(!admin.exists(), "the registration the destroy left is gone");
    let shown: Vec<String> = crate::test_support::shown_notices(orphaned_push_failures(common.as_path(), &|commit: &str| { return commit.to_string(); }));
    assert!(shown.iter().any(|text: &String| { return text.contains("refused by test policy"); }), "the notice the registration held is shown: {shown:?}");
    // Git accepts the path again, which is what the person asked for.
    git(repo.as_path(), &["worktree", "add", "--quiet", "--detach", worktree.to_str().expect("UTF-8")]);
    assert!(worktree.join("tracked.txt").is_file(), "the add at the reused path made the worktree");
    remove(root.as_path());
}

/// The one registration id under `common/worktrees`.
fn only_registration_id(common: &Path) -> std::ffi::OsString {
    let mut names: Vec<std::ffi::OsString> = std::fs::read_dir(common.join("worktrees")).expect("registrations").flatten().map(|entry: std::fs::DirEntry| { return entry.file_name(); }).collect();
    names.sort();
    assert_eq!(names.len(), 1, "one registration: {names:?}");
    return names.remove(0);
}

/// What:
///  A worktree whose pathname ends with the bytes of the finished mark is not read as one whose destroy finished:
///  the
///  mark is the record's first line,
///  so no byte of a path can be taken for it,
///  and the registration and the worktree are kept.
/// Why:
///   The mark was stripped from the end of the record,
/// so a path ending in those bytes made recovery believe the files
///  were gone and delete the registration of a worktree that still exists.
#[test]
fn a_pathname_that_ends_with_the_finished_mark_is_not_proof_of_destruction() {
    let root: PathBuf = fixture("remove-record-mark-name");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let name: &str = "wtdestroyed=1
";
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join(name).as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let id: std::ffi::OsString = only_registration_id(common.as_path());
    let admin: PathBuf = common.join("worktrees").join(&id);
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), id.as_os_str(), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    // The worktree is where the record names it, so nothing was destroyed and the record is dropped without a word, as for
    // every removal that did not happen.
    assert!(problems.is_empty(), "{problems:?}");
    assert!(admin.join("gitdir").is_file(), "the registration of a worktree that still exists is kept");
    assert!(worktree.join("tracked.txt").is_file(), "and so is the worktree");
    remove(root.as_path());
}

/// What:
///  A symbolic link planted at the mark writer's temporary name is not written through:
///  the person's file keeps its bytes and the record is marked all the same.
/// Why:
///   The temporary name was the record's own name plus the process number,
/// so another program could guess it,
/// replace it
///  with a link to a file of the person's own,
/// and have the writer truncate that file before the rename.
#[test]
fn a_link_at_the_mark_writers_temporary_name_is_not_written_through() {
    let root: PathBuf = fixture("remove-record-mark-link");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    let record: PathBuf = common.join(crate::worktree_removal_interrupted_destroy::RECORD_FOLDER).join("wt");
    let notes: PathBuf = root.join("notes.txt");
    std::fs::write(notes.as_path(), b"a person's notes\n").expect("their file");
    let planted: PathBuf = record.with_extension(format!("recorded.{}", std::process::id()));
    std::os::unix::fs::symlink(notes.as_path(), planted.as_path()).expect("a link at the temporary name");
    record_destroyed(common.as_path(), std::ffi::OsStr::new("wt"));
    assert_eq!(std::fs::read(notes.as_path()).expect("their file"), b"a person's notes\n", "nothing is written through the link");
    // The mark is there, so the removal this record names is finished and the registration goes.
    remove(worktree.as_path());
    assert!(finish_interrupted_destroys(common.as_path()).is_empty());
    assert!(!admin.exists(), "the marked record still finished the removal");
    remove(root.as_path());
}

/// A record's owner that no process has:
/// a process number whose birth identity is not the one written.
const ENDED_OWNER: &str = "1:ended:0";

/// A named pipe at `path`,
/// created with the platform's own call.
fn named_pipe(path: &Path) {
    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("path text");
    // SAFETY: `name` is NUL-terminated and outlives the call; mode 0600 is this test's own.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "the named pipe {}", path.display());
}

/// What:
///  A named pipe among the destroy records does not hold the command that recovers them:
///  it is skipped,
///  and the record beside it is still finished.
/// Why:
///   Every wrapped command reads that directory,
/// so an entry that is not a plain file would make a commit wait for
///  a writer that never comes,
/// before anything checks whether the entry names a registration.
#[test]
fn a_named_pipe_among_the_destroy_records_does_not_hold_the_command() {
    let root: PathBuf = fixture("remove-destroy-record-pipe");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    record_destroyed(common.as_path(), std::ffi::OsStr::new("wt"));
    remove(worktree.as_path());
    let folder: PathBuf = common.join(crate::worktree_removal_interrupted_destroy::RECORD_FOLDER);
    named_pipe(folder.join("pipe-not-a-record").as_path());
    let (sender, receiver) = std::sync::mpsc::channel();
    let scanned: PathBuf = common.clone();
    let runner = std::thread::spawn(move || {
        let _said = sender.send(finish_interrupted_destroys(scanned.as_path()));
    });
    let problems: Vec<String> = match receiver.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(said) => said,
        Err(_still_waiting) => panic!("the recovery waited on a named pipe instead of skipping it"),
    };
    runner.join().expect("the scan ends");
    assert!(problems.is_empty(), "{problems:?}");
    assert!(folder.join("pipe-not-a-record").exists(), "the named pipe itself is left alone");
    assert!(!admin.exists(), "the record beside it is still finished");
    remove(root.as_path());
}

/// What:
///  A worktree with a conversion's leftover copy beside it is not removed:
///  the copy would be left behind with nothing naming it.
/// Why:
///   The removal deletes the registration,
/// and the journal that names the copy lives in it,
/// so the person is sent
///  to finish or discard the copy first,
/// which the report says with both commands.
#[test]
fn a_worktree_with_a_leftover_conversion_stage_is_refused() {
    let root: PathBuf = fixture("remove-leftover-stage");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let converting: PathBuf = crate::worktree_convert::converting_path(worktree.as_path());
    std::fs::create_dir_all(converting.join("d")).expect("a copy beside the worktree");
    std::fs::write(converting.join("d/copy"), b"a copy\n").expect("its file");
    // A record beside the worktree is what proves the copy is a conversion's; without one the removal is not refused.
    let stage = crate::worktree_convert_leftover_stage_record::LeftoverStage {
        original: crate::fs_beneath::identity_of(&std::fs::File::open(converting.as_path()).expect("the copy")).expect("identity"),
        created: None,
    };
    crate::worktree_convert_leftover_stage_record::record_leftover_stage(worktree.as_path(), &stage);
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let outcome: RemoveOutcome = run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt"));
    let RemoveOutcome::Refused(report) = outcome else { panic!("the removal is refused while the copy is there, but it was {outcome:?}") };
    assert!(report.contains(converting.to_str().expect("UTF-8")), "the report names the copy: {report}");
    assert!(report.contains("convert-worktree-to-subvolume"), "and the command that finishes or discards it: {report}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is untouched");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is untouched");
    assert!(converting.join("d/copy").is_file(), "and so is the copy");
    remove(root.as_path());
}

/// What:
///  A record whose stage is gone does not stop the removal:
///  the record alone says what a conversion may have left,
///  not what is
///  there now,
///  and nothing clears a stale record,
/// so refusing on it would block the removal for good.
/// Why:
///   `--abandon` removes the record after it destroys the stage,
/// so a kill in that instant leaves the record behind;
/// a removal
///  must not be blocked by it.
#[test]
fn a_stale_record_whose_stage_is_gone_does_not_stop_the_removal() {
    let root: PathBuf = fixture("remove-stale-record-no-stage");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stage = crate::worktree_convert_leftover_stage_record::LeftoverStage {
        original: crate::fs_beneath::identity_of(&std::fs::File::open(worktree.as_path()).expect("the worktree")).expect("identity"),
        created: None,
    };
    crate::worktree_convert_leftover_stage_record::record_leftover_stage(worktree.as_path(), &stage);
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::Removed);
    assert!(!worktree.exists(), "the removal ran");
    remove(root.as_path());
}

/// What:
///  A record whose stage name holds another object does not stop the removal either:
///  the object is not the one the
///  record names,
/// so it is the person's and the removal leaves it alone.
#[test]
fn a_record_whose_stage_name_holds_another_object_does_not_stop_the_removal() {
    let root: PathBuf = fixture("remove-stale-record-other-object");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let sibling: PathBuf = crate::worktree_convert::converting_path(worktree.as_path());
    std::fs::create_dir_all(sibling.join("d")).expect("a directory of the person's own");
    std::fs::write(sibling.join("d/theirs"), b"theirs\n").expect("its file");
    let stage = crate::worktree_convert_leftover_stage_record::LeftoverStage {
        original: crate::fs_beneath::identity_of(&std::fs::File::open(worktree.as_path()).expect("the worktree")).expect("identity"),
        created: None,
    };
    crate::worktree_convert_leftover_stage_record::record_leftover_stage(worktree.as_path(), &stage);
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::Removed);
    assert!(sibling.join("d/theirs").is_file(), "the object that is not the recorded one is left alone");
    remove(root.as_path());
}

/// What:
///  A directory beside a worktree that no conversion recorded does not stop its removal:
///  the suffix proves nothing,
///  and refusing on it would block a legitimate removal with neither advertised command able to clear it.
/// Why:
///   A conversion writes a record beside the worktree before its swap;
/// a directory without one is the person's own,
///  and the removal leaves it where it is.
#[test]
fn a_directory_beside_a_worktree_that_no_conversion_recorded_does_not_stop_its_removal() {
    let root: PathBuf = fixture("remove-unrecorded-sibling");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let sibling: PathBuf = crate::worktree_convert::converting_path(worktree.as_path());
    std::fs::create_dir_all(sibling.join("d")).expect("a directory of the person's own beside the worktree");
    std::fs::write(sibling.join("d/theirs"), b"theirs\n").expect("its file");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::Removed);
    assert!(sibling.join("d/theirs").is_file(), "the directory that no conversion recorded is left alone");
    remove(root.as_path());
}

/// What:
///  A removal that destroyed the files but could not remove the registration records its destroy,
///  so a later command finishes what it could not,
///  and a removal that finished records nothing.
#[test]
fn a_removal_records_a_destroy_it_could_not_finish_and_no_other() {
    let root: PathBuf = fixture("remove-finish-records");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let worktrees: PathBuf = repo.join(".git/worktrees");
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o555)).expect("read-only registrations");
    let outcome: RemoveOutcome = run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt"));
    std::fs::set_permissions(worktrees.as_path(), std::fs::Permissions::from_mode(0o755)).expect("restore");
    assert!(matches!(outcome, RemoveOutcome::Incomplete(_)), "the registration cannot be removed: {outcome:?}");
    assert!(!worktree.exists() && admin.is_dir(), "the destroy happened and the registration stayed");
    let record: PathBuf = common.join(crate::worktree_removal_interrupted_destroy::RECORD_FOLDER).join("wt");
    assert!(record.is_file(), "the destroy this removal made is recorded for the next command");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    assert!(problems.is_empty(), "{problems:?}");
    assert!(!admin.exists() && !record.exists(), "the next command finished it and dropped the record");
    // A removal that finished leaves nothing behind for a later command to do.
    let other: PathBuf = clean_worktree(repo.as_path(), root.join("wt2").as_path(), "other");
    let second: Stand = Stand::new(vec![other.clone()]);
    assert_eq!(run(&second, repo.as_path(), &plan_named(other.as_path(), "../wt2")), RemoveOutcome::Removed);
    assert!(finish_interrupted_destroys(common.as_path()).is_empty());
    assert!(!record.parent().expect("record folder").exists(), "a finished removal records nothing");
    remove(root.as_path());
}

/// What:
///  The refusal for a conversion that stopped past its exchange names the remedy that works there,
///  and not `--abandon`,
///  which refuses every conversion at or past the exchange.
/// Why:
///   The remedy is what the person acts on:
/// a report that names a command which answers "past the swap:
/// run the
///  command again" leaves the removal blocked with no way forward.
#[test]
fn the_refusal_for_a_post_swap_stage_names_the_finishing_command() {
    let root: PathBuf = fixture("remove-leftover-stage-past-swap");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let platform: crate::subvolume_test_support::PlainPlatform = crate::subvolume_test_support::PlainPlatform::new();
    let options = crate::worktree_convert::ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };
    // A conversion stopped right after its exchange: the original tree waits at the converting name, and the journal says
    // `Exchanged`, which is the state `--abandon` refuses.
    let killed = crate::worktree_convert::convert_worktree(&platform, worktree.as_path(), options, &crate::subvolume_error::Interrupt::at("exchanged", 0), &mut |_text: &str| {});
    assert!(killed.is_err(), "the conversion is stopped at its exchange");
    let converting: PathBuf = crate::worktree_convert::converting_path(worktree.as_path());
    assert!(std::fs::symlink_metadata(converting.as_path()).is_ok(), "the original tree is at the converting name");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let outcome: RemoveOutcome = run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt"));
    let RemoveOutcome::Refused(report) = outcome else { panic!("the removal is refused while the copy is there, but it was {outcome:?}") };
    assert!(report.contains("which completes it in place"), "the report names the command that finishes a post-swap conversion: {report}");
    assert!(!report.contains("or discard"), "and not the remedy for a copy that is still there: {report}");
    assert!(worktree.join("tracked.txt").is_file(), "the worktree is untouched");
    remove(root.as_path());
}

/// What:
///  Recovery leaves the record of a removal whose owner may still be running,
///  so that owner can still mark its
///  success,
///  and drops the record of one whose owner has ended.
/// Why:
///   A removal writes its record,
/// destroys the tree and marks it;
/// a second command that read the record in that time
///  and forgot it took away the only place the first could mark its success,
/// so a kill after the destroy left nothing for
///  the next command to finish.
#[test]
fn another_commands_recovery_leaves_the_record_of_a_running_removal() {
    let root: PathBuf = fixture("remove-record-running-owner");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    let record: PathBuf = common.join(crate::worktree_removal_interrupted_destroy::RECORD_FOLDER).join("wt");
    // A removal that is running: the record is this process's, and the worktree is still there until its kernel call.
    crate::worktree_removal_interrupted_destroy::record_destroy(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path()).expect("record");
    assert!(finish_interrupted_destroys(common.as_path()).is_empty(), "a removal that is running is not a problem to report");
    assert!(record.is_file(), "the record of a running removal stays, so its owner can mark it");
    // The same record with an owner that has ended is dropped, as before.
    std::fs::remove_file(record.as_path()).expect("drop the running record this control wrote");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    assert!(finish_interrupted_destroys(common.as_path()).is_empty());
    assert!(!record.exists(), "a record whose owner has ended is dropped");
    remove(root.as_path());
}

/// What:
///  A registration a repair has pointed at a restored checkout is kept,
///  even though the destroy record names a path that is gone.
/// Why:
///   The record says which path the destroy was aimed at,
/// not that no checkout claims the registration now:
/// a repaired
///  worktree keeps its index,
/// its reflog and its per-worktree refs in that registration,
/// and emptying it leaves the restored
///  worktree's `.git` pointing at nothing.
#[test]
fn a_registration_a_repair_pointed_at_a_restored_checkout_is_kept() {
    let root: PathBuf = fixture("remove-record-then-repaired");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    record_destroyed(common.as_path(), std::ffi::OsStr::new("wt"));
    // The checkout is destroyed, and a backup of it is restored at another path, which real Git then repairs the
    // registration to name.
    let restored: PathBuf = root.join("restored");
    std::fs::rename(worktree.as_path(), restored.as_path()).expect("the checkout is restored at another path");
    git(repo.as_path(), &["worktree", "repair", restored.to_str().expect("UTF-8")]);
    let pointer: String = std::fs::read_to_string(admin.join("gitdir")).expect("the registration's gitdir");
    assert!(pointer.contains("restored"), "the repair pointed the registration at the restored checkout: {pointer}");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    assert!(problems.iter().any(|text: &String| { return text.contains("restored"); }), "the person is told what the registration names now: {problems:?}");
    assert!(admin.join("gitdir").is_file(), "the registration of a restored checkout is kept");
    assert!(restored.join("tracked.txt").is_file(), "the restored checkout is untouched");
    remove(root.as_path());
}

/// What:
///  A registration whose `gitdir` cannot be read is kept and named,
///  instead of being emptied as though it named the destroyed worktree.
/// Why:
///   The record says which path the destroy was aimed at,
/// not which checkout uses the registration now,
/// and an answer that could not be read proves nothing about that:
/// the registration holds a live checkout's index,
/// its reflog and its per-worktree refs,
/// and emptying it leaves that checkout's `.git` pointing at nothing.
/// The registration's own directory is writable,
/// so its unreadable file can still be unlinked.
#[test]
fn a_registration_whose_gitdir_cannot_be_read_is_kept_and_named() {
    // A user who is root reads a file whatever its mode, so this control has nothing to check there.
    // SAFETY: `geteuid` takes no arguments and cannot fail.
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let root: PathBuf = fixture("remove-record-unreadable-gitdir");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    record_destroyed(common.as_path(), std::ffi::OsStr::new("wt"));
    // A backup of the checkout is restored at another path, which real Git repairs the registration to name.
    let restored: PathBuf = root.join("restored");
    std::fs::rename(worktree.as_path(), restored.as_path()).expect("the checkout is restored at another path");
    git(repo.as_path(), &["worktree", "repair", restored.to_str().expect("UTF-8")]);
    let index: PathBuf = admin.join("index");
    assert!(index.is_file(), "the registration holds the restored checkout's index");
    // What a mode a person cannot read does: the answer is there and this user cannot have it.
    std::fs::set_permissions(admin.join("gitdir").as_path(), std::fs::Permissions::from_mode(0o000)).expect("unreadable gitdir");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    assert!(problems.iter().any(|text: &String| { return text.contains(admin.to_str().expect("UTF-8")); }), "the person is told which registration is kept: {problems:?}");
    assert!(admin.join("gitdir").is_file(), "the registration is kept");
    assert!(index.is_file(), "and so is the restored checkout's index");
    assert!(restored.join("tracked.txt").is_file(), "the restored checkout is untouched");
    std::fs::set_permissions(admin.join("gitdir").as_path(), std::fs::Permissions::from_mode(0o644)).expect("readable again");
    remove(root.as_path());
}

/// What:
///  A worktree moved away after the destroy was recorded is not a destroyed one:
///  its registration is kept,
///  because the moved worktree's `.git` file still names it and its index and logs live there.
/// Why:
///   The record is written before the kernel call,
/// so an absent pathname proves only that nothing is at that name,
/// not that
///  the subvolume was destroyed;
/// emptying the registration of a worktree that still exists breaks it,
/// and `git worktree
///  repair` can only mend a worktree whose registration is still there.
#[test]
fn a_worktree_moved_away_after_the_record_keeps_its_registration() {
    let root: PathBuf = fixture("remove-record-then-moved");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    let moved: PathBuf = root.join("wt.moved");
    std::fs::rename(worktree.as_path(), moved.as_path()).expect("the worktree is moved, not destroyed");
    let problems: Vec<String> = finish_interrupted_destroys(common.as_path());
    assert!(!problems.is_empty(), "the person is told the removal did not finish");
    assert!(admin.join("gitdir").is_file(), "the registration a moved worktree still names is kept");
    let pointer: String = std::fs::read_to_string(moved.join(".git")).expect("the moved worktree's .git file");
    assert!(pointer.contains(admin.to_str().expect("UTF-8")), "and it still names a registration that is there: {pointer}");
    assert!(moved.join("tracked.txt").is_file(), "the worktree's files are untouched");
    remove(root.as_path());
}

/// What:
///  A destroy that was recorded and then never happened leaves the registration alone and drops the record.
/// Why:
///   The record is written before the kernel call,
/// so a kill in that step leaves a worktree whose directory is still there:
///  nothing was destroyed,
/// and emptying its registration would discard the worktree of a person who asked for nothing.
#[test]
fn a_recorded_destroy_whose_worktree_is_still_there_changes_nothing() {
    let root: PathBuf = fixture("remove-finish-not-destroyed");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("common directory");
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let identity = crate::fs_beneath::identity_of(&std::fs::File::open(admin.as_path()).expect("registration")).expect("identity");
    record_destroy_with_owner(common.as_path(), std::ffi::OsStr::new("wt"), &identity, worktree.as_path(), Some(ENDED_OWNER)).expect("record");
    assert!(finish_interrupted_destroys(common.as_path()).is_empty());
    assert!(admin.join("gitdir").is_file() && worktree.join("tracked.txt").is_file(), "nothing was removed");
    assert!(finish_interrupted_destroys(common.as_path()).is_empty(), "the record is gone, so the next command looks at nothing");
    remove(root.as_path());
}

/// An administrative directory replaced after the proof is not the registration of this worktree and is kept whole.
#[test]
fn an_administrative_directory_replaced_after_the_proof_is_kept() {
    let root: PathBuf = fixture("remove-guard-swapped-admin");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let moved: PathBuf = repo.join(".git/worktrees/wt.moved");
    let at: PathBuf = admin.clone();
    *stand.after_proof.borrow_mut() = Some(Box::new(move || {
        std::fs::rename(at.as_path(), moved.as_path()).expect("move the registration away");
        std::fs::create_dir(at.as_path()).expect("replacement registration");
        std::fs::write(at.join("gitdir"), b"/elsewhere/.git\n").expect("replacement gitdir");
    }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert_eq!(std::fs::read(admin.join("gitdir")).expect("the replacement registration is kept"), b"/elsewhere/.git\n");
    assert!(worktree.join("tracked.txt").is_file(), "nothing was destroyed");
    remove(root.as_path());
}

/// A `gitdir` file rewritten after the proof means the registration is no longer the one that was proven.
#[test]
fn a_registration_rewritten_after_the_proof_stops_the_removal() {
    let root: PathBuf = fixture("remove-guard-rewritten-gitdir");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let pointer: Vec<u8> = format!("{}\n", worktree.join(".git").display()).into_bytes();
    // Same bytes plus trailing spaces: still a record of this worktree for Git, but not the proven text.
    *stand.after_proof.borrow_mut() = Some(Box::new(move || { std::fs::write(admin.join("gitdir"), [pointer.as_slice(), b"  \n"].concat()).expect("rewrite"); }));
    assert_eq!(run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt")), RemoveOutcome::LeftToGit);
    assert!(worktree.join("tracked.txt").is_file(), "nothing was destroyed");
    remove(root.as_path());
}

/// The same replacement during the destroy leaves the new registration whole and reports the removal as incomplete.
#[test]
fn an_administrative_directory_replaced_during_the_destroy_is_kept_and_reported() {
    let root: PathBuf = fixture("remove-guard-swapped-admin-late");
    let repo: PathBuf = repository_with_file(root.as_path(), "repo");
    let worktree: PathBuf = clean_worktree(repo.as_path(), root.join("wt").as_path(), "topic");
    let stand: Stand = Stand::new(vec![worktree.clone()]);
    let admin: PathBuf = admin_of(repo.as_path(), "wt");
    let moved: PathBuf = repo.join(".git/worktrees/wt.moved");
    let at: PathBuf = admin.clone();
    *stand.during_destroy.borrow_mut() = Some(Box::new(move || {
        std::fs::rename(at.as_path(), moved.as_path()).expect("move the registration away");
        std::fs::create_dir(at.as_path()).expect("replacement registration");
        std::fs::write(at.join("gitdir"), b"/elsewhere/.git\n").expect("replacement gitdir");
    }));
    let outcome: RemoveOutcome = run(&stand, repo.as_path(), &plan_named(worktree.as_path(), "../wt"));
    let RemoveOutcome::Incomplete(report) = outcome else { panic!("a registration that was not removed is not a success: {outcome:?}") };
    assert!(report.contains(admin.to_str().expect("UTF-8")), "{report}");
    assert_eq!(std::fs::read(admin.join("gitdir")).expect("the replacement registration is kept"), b"/elsewhere/.git\n");
    assert!(repo.join(".git/worktrees/wt.moved/gitdir").is_file(), "the registration that was moved away is not emptied under its new name");
    remove(root.as_path());
}
