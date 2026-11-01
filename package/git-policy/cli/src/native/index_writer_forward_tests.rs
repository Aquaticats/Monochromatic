//! Controls for the coordinated forward:
//!  a plain writer outside a repository runs unlocked,
//!  a
//! writer inside one runs as a child that holds the landing lock and hands over the lease,
//!  and a
//! failed coordination never runs Git.

use super::*;
use std::path::Path;
use crate::forwarded_command::Route;
use crate::owner_lock::try_acquire_owner_lock;
use crate::test_support::{REAL_GIT, fixture, isolated_overlay, remove, repository};
use crate::action::ENGINE_FAILURE_EXIT_CODE;
use crate::worktree_identity::resolve_worktree_identity;

/// The forward for `git add <path>` in a repository,
///  with a short unproven budget.
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
        timeout_ms: 50,
        copy: None,
    };
}

/// A writer with a free landing lock runs real Git under the lock,
///  which is free again afterwards.
#[test]
fn a_coordinated_add_runs_git_and_releases_the_lock() {
    let root: PathBuf = fixture("forward-add");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    assert_eq!(run_coordinated_forward(&forward), CoordinatedRun::Ran(0));
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "a.txt");
    let free: Option<crate::owner_lock::OwnerLock> = try_acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        None,
    )
    .expect("a readable lock");
    assert!(free.is_some(), "the coordinated run released the landing lock");
    remove(root.as_path());
}

/// A command whose coordination fails does not run Git:
///  the failure is the result.
#[test]
fn a_failed_coordination_does_not_run_git() {
    let root: PathBuf = fixture("forward-refused");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"content\n").expect("file to add");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("foreign lock");
    std::fs::write(repo.join(".git/index~pid.lock"), "pid 2147483646\n").expect("dead PID file");
    let forward: CoordinatedForward = add_forward(repo.as_path());
    match run_coordinated_forward(&forward) {
        CoordinatedRun::Failed { stderr, code } => {
            assert_eq!(code, ENGINE_FAILURE_EXIT_CODE);
            assert!(stderr.contains("\"code\":\"index-lock-unproven-owner\""), "{stderr}");
        }
        other => panic!("expected the refusal, got {other:?}"),
    }
    let staged: String = crate::test_support::git_text(repo.as_path(), &["ls-files", "--", "a.txt"]);
    assert_eq!(staged, "", "Git did not run, so nothing was staged");
    remove(root.as_path());
}

/// A non-writer that carries the linked-worktree copy still runs here, as a settled child, not as an exec.
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
        creates_or_moves: false,
        main_checkout_paths: vec![String::from(".cache/forbidden-strings.rules.txt")],
        global_prefix: vec![OsString::from("-C"), repo.as_os_str().to_owned()],
        environment: Vec::new(),
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
