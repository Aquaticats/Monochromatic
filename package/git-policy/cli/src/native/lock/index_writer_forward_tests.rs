//! Controls for the forward that stays alive around Git:
//!  a plain writer runs as a child and reports Git's status,
//!  a foreign `index.lock` is Git's to refuse and is not waited for,
//!  and a command that needs nothing around Git is handed back for an exec.

use super::*;
use std::path::Path;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, fixture, git, git_text, isolated_overlay, remove, repository};
use crate::worktree_identity::resolve_worktree_identity;
use crate::worktree_copy_forward::CopyFacts;

/// The forward for `git add <path>` in a repository.
fn add_forward(repo: &Path) -> CoordinatedForward {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let identity: WorktreeIdentity = resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("identity");
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        repo.as_os_str().to_owned(),
        OsString::from("add"),
        OsString::from("a.txt"),
    ];
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("add")),
        route: Route::Direct,
    };
    return CoordinatedForward {
        real_git: PathBuf::from(REAL_GIT),
        arguments,
        overlay: isolated_overlay(),
        identity,
        command,
        environment: Vec::new(),
        copy: None,
    };
}

/// A writer runs real Git as a child and reports its status.
#[test]
fn a_coordinated_add_runs_git_and_reports_its_status() {
    let root: PathBuf = fixture("forward-add");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "a.txt");
    remove(root.as_path());
}

/// A foreign `index.lock` is Git's own refusal:
/// the writer runs Git at once,
/// Git fails as it fails without the wrapper,
/// and
///  nothing waits for the lock or removes it.
#[test]
fn a_foreign_index_lock_is_gits_refusal_and_is_not_waited_for() {
    let root: PathBuf = fixture("forward-foreign-lock");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("foreign lock");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    let began: std::time::Instant = std::time::Instant::now();
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(128), "Git's status for a lock it cannot create");
    assert!(began.elapsed() < std::time::Duration::from_secs(10), "no wait for a foreign holder");
    assert!(repo.join(".git/index.lock").exists(), "the foreign lock is left in place");
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "", "Git could not take the lock, so nothing was staged");
    remove(root.as_path());
}

/// A non-writer that carries the linked-worktree copy still runs here,
/// as a settled child,
/// not as an exec.
#[test]
fn a_non_writer_with_a_copy_runs_settled() {
    let root: PathBuf = fixture("forward-status-copy");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    forward.command = ResolvedCommand {
        arguments: forward.arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("status")),
        route: Route::Direct,
    };
    forward.arguments = vec![
        OsString::from("-C"),
        repo.as_os_str().to_owned(),
        OsString::from("status"),
        OsString::from("--short"),
    ];
    forward.copy = Some(CopyFacts {
        identity: forward.identity.clone(),
        creates: false,
        destination: None,
        add_plan: None,
        remove_plan: None,
    });
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    remove(root.as_path());
}

/// A non-writer is not coordinated and is handed back for an ordinary exec.
#[test]
fn a_non_writer_is_not_coordinated() {
    let root: PathBuf = fixture("forward-status");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut forward: CoordinatedForward = add_forward(repo.as_path());
    forward.command = ResolvedCommand {
        arguments: forward.arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("status")),
        route: Route::Direct,
    };
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::NotCoordinated);
    remove(root.as_path());
}

/// A writer of another index file than the worktree's own has no hold of the service to wait for and is handed back for an exec.
#[test]
fn a_writer_of_another_index_file_is_not_coordinated() {
    let root: PathBuf = fixture("forward-other-index");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    let mut other: CoordinatedForward = forward.clone();
    other.overlay.push((OsString::from("GIT_INDEX_FILE"), repo.join("other-index").into_os_string()));
    assert_eq!(run_coordinated_forward(&other), CoordinatedRun::NotCoordinated);
    remove(root.as_path());
}

/// A repository where `feature` and `main` have diverged and another session left `u.txt` staged,
/// which native Git refuses to merge over.
fn diverged_with_staged_work(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    for file in ["u.txt", "f.txt", "h.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\nwith enough lines\nfor a rename to be found\n")).expect("file");
        git(repo.as_path(), &["add", "--", file]);
    }
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    std::fs::write(repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(repo.as_path(), &["add", "--", "u.txt"]);
    return (root, repo);
}

/// The forward for `git merge` with `after` written behind the subcommand,
/// in the repository at `repo`.
fn merge_forward(repo: &Path, after: &[&str]) -> CoordinatedForward {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let identity: WorktreeIdentity = resolve_worktree_identity(
        Path::new(REAL_GIT),
        prefix.as_slice(),
        isolated_overlay().as_slice(),
    )
    .expect("identity");
    let mut arguments: Vec<OsString> = prefix;
    arguments.push(OsString::from("merge"));
    arguments.extend(after.iter().map(OsString::from));
    let command: ResolvedCommand = ResolvedCommand {
        arguments: arguments.clone(),
        subcommand_index: 2,
        subcommand: Some(OsString::from("merge")),
        route: Route::Direct,
    };
    return CoordinatedForward {
        real_git: PathBuf::from(REAL_GIT),
        arguments,
        overlay: isolated_overlay(),
        identity,
        command,
        environment: Vec::new(),
        copy: None,
    };
}

/// A merge that native Git would refuse only because another session left work staged is completed here:
/// the dispatch reports the merge's own status instead of running Git,
/// the two sides are both in the tree,
/// and the staged work is still staged.
#[test]
fn a_merge_completed_in_isolation_reports_its_own_status() {
    let (root, repo): (PathBuf, PathBuf) = diverged_with_staged_work("forward-merge-isolated");
    let forward: CoordinatedForward = merge_forward(repo.as_path(), &["--no-edit", "feature"]);
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    assert_eq!(git_text(repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1", "the merge is a commit");
    assert_eq!(git_text(repo.as_path(), &["diff", "--cached", "--name-only"]), "u.txt", "the staged work stays staged");
    assert_eq!(std::fs::read_to_string(repo.join("f.txt")).expect("feature's file"), "f.txt feature\n");
    assert_eq!(std::fs::read_to_string(repo.join("h.txt")).expect("main's file"), "h.txt main\n");
    remove(root.as_path());
}

/// A merge the wrapper defers to native Git runs here,
/// and the status is Git's:
/// the setting that defers it is read,
/// the command falls through,
/// and native Git refuses the merge,
/// because none of the commits it would merge carries a signature.
#[test]
fn a_merge_the_wrapper_defers_runs_as_native_git_and_reports_its_status() {
    let (root, repo): (PathBuf, PathBuf) = diverged_with_staged_work("forward-merge-deferred");
    git(repo.as_path(), &["config", "merge.verifysignatures", "yes"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let forward: CoordinatedForward = merge_forward(repo.as_path(), &["--no-edit", "feature"]);
    let ended: CoordinatedRun = run_coordinated_forward(&forward);
    let CoordinatedRun::Ran(code) = ended else {
        panic!("the deferred merge runs as native Git: {ended:?}");
    };
    assert_ne!(code, 0, "native Git's refusal of a merge of unsigned commits");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    assert_eq!(git_text(repo.as_path(), &["diff", "--cached", "--name-only"]), "u.txt", "the staged work stays staged");
    remove(root.as_path());
}
