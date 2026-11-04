//! Controls for the guards of the subvolume removal of a worktree, run on a stand-in for the kernel so that they hold on
//!  any filesystem: a worktree of another repository, a name Git resolves by its last components, a lock that cannot be
//!  looked up, and objects swapped between the proof of cleanliness, the destroy and the removal of the registration.

use super::*;
use crate::fs_beneath::handle_path;
use crate::test_support::{age, fixture, git, git_context, remove, repository};
use std::cell::RefCell;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;

/// A hook that runs once, at a chosen step of the removal.
type Hook = RefCell<Option<Box<dyn FnOnce()>>>;

/// A stand-in for the kernel and the cleanliness proof: the worktrees listed are "subvolume roots", the proof is the real
///  one, and the destroy deletes the directory. Each hook runs once, at the named step, so a control can swap an object
///  exactly where a concurrent process would.
struct Stand {
    /// The directories the stand-in calls subvolume roots.
    subvolumes: Vec<PathBuf>,
    /// Runs after the proof of cleanliness succeeded.
    after_proof: Hook,
    /// Runs inside the destroy, before the directory is deleted.
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

/// The real cleanliness proof, the hooks, and a deletion in place of the kernel call.
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

/// A linked worktree at `directory`, provably clean, and its real path.
fn clean_worktree(repo: &Path, directory: &Path, branch: &str) -> PathBuf {
    git(repo, &["worktree", "add", "--quiet", "-b", branch, directory.to_str().expect("UTF-8")]);
    let canonical: PathBuf = std::fs::canonicalize(directory).expect("canonical");
    age(canonical.join("tracked.txt").as_path());
    git(canonical.as_path(), &["update-index", "--refresh"]);
    return canonical;
}

/// A plan that names `target` by `operand`.
fn plan_named(target: &Path, operand: &str) -> RemovePlan {
    return RemovePlan { target: target.to_path_buf(), operand: OsString::from(operand) };
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

/// A worktree that another repository registered is not this repository's to remove: Git answers "is not a working tree".
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

/// Git matches the operand against the last components of every registered path before it resolves it as a path, so a
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

/// An ancestor replaced by another real directory after the proof is not the directory that held the worktree: the held
///  parent still leads to the proven worktree, but the path the person typed no longer does.
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
///  and the way to finish, never as a success.
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
