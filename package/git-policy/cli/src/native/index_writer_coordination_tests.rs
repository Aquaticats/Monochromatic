//! Controls for index-writer coordination against real repositories:
//!  which commands coordinate,
//! the lease a held lock produces,
//!  waiting behind a live landing,
//!  and the unproven-owner failure.

use super::*;
use crate::forwarded_command::{ResolvedCommand, Route};
use crate::owner_lock::{SilentWait, acquire_owner_lock, try_acquire_owner_lock};
use std::sync::OnceLock;
use crate::test_support::{REAL_GIT, fixture, git_context, isolated_overlay, remove, repository};
use crate::transaction_git::GitContext;
use crate::transaction_lease::LANDING_LEASE_VARIABLE;
use crate::worktree_identity::resolve_worktree_identity;
use crate::transaction_registry::ensure_transaction_root;
use std::path::PathBuf;

/// A resolved command `-C <repo> <words…>` whose subcommand is the first word.
fn command_in(repo: &Path, words: &[&str]) -> ResolvedCommand {
    let mut arguments: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    for word in words {
        arguments.push(OsString::from(word));
    }
    return ResolvedCommand {
        arguments,
        subcommand_index: 2,
        subcommand: Some(OsString::from(words[0])),
        route: Route::Direct,
    };
}

/// The real-Git context with the isolating overlay.
fn context() -> GitContext {
    return git_context();
}

/// The identity of a repository,
///  resolved with real Git.
fn identity_of(repo: &Path) -> WorktreeIdentity {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    return resolve_worktree_identity(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice())
        .expect("identity of the fixture repository");
}

/// A fixture repository with one commit,
///  and the registry directory beside its git directory.
fn scene(name: &str) -> (PathBuf, PathBuf) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    return (root, repo);
}

/// Hold the repository's landing lock as a live owner (this process) until the returned lock is released.
fn hold_landing(repo: &Path) -> OwnerLock {
    let lock_directory: PathBuf = repo.join(".git/cli-git-transactions/landing.lock");
    std::fs::create_dir_all(lock_directory.parent().expect("registry")).expect("registry");
    // `mut` is required by the notice interface; this wait is silent.
    let mut silent: SilentWait = SilentWait;
    return acquire_owner_lock(lock_directory.as_path(), std::time::Duration::from_millis(5), &mut silent)
        .expect("landing lock held by the test");
}

/// Status and read-only commands never coordinate.
#[test]
fn a_non_writer_takes_no_coordination() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-non-writer");
    let command: ResolvedCommand = command_in(repo.as_path(), &["status"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    assert!(coordinate_index_writer(&input).expect("no failure").is_none());
    remove(root.as_path());
}

/// Outside a repository a writer has no landing to coordinate with.
#[test]
fn a_writer_outside_a_repository_takes_no_coordination() {
    let root: PathBuf = fixture("coordination-outside");
    let command: ResolvedCommand = command_in(root.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &WorktreeIdentity::OutsideWorktree,
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    assert!(coordinate_index_writer(&input).expect("no failure").is_none());
    remove(root.as_path());
}

/// An index other than the real one,
///  selected by `GIT_INDEX_FILE`,
///  takes no coordination.
#[test]
fn an_alternate_index_takes_no_coordination() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-alternate");
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((
        OsString::from("GIT_INDEX_FILE"),
        repo.join("alternate-index").into_os_string(),
    ));
    let alternate: GitContext = GitContext {
        real_git: PathBuf::from(REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &alternate,
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    assert!(coordinate_index_writer(&input).expect("no failure").is_none());
    remove(root.as_path());
}

/// A landing lease from an ancestor that holds the lock means nothing is taken again.
#[test]
fn an_inherited_lease_takes_no_coordination() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-lease");
    let held: OwnerLock = hold_landing(repo.as_path());
    let lease: String = format_landing_lease(
        held.directory().to_string_lossy().as_ref(),
        held.token(),
    );
    let environment: Vec<(OsString, OsString)> = vec![(
        OsString::from(LANDING_LEASE_VARIABLE),
        OsString::from(lease),
    )];
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: environment.as_slice(),
        timeout_ms: 100,
    };
    assert!(coordinate_index_writer(&input).expect("no failure").is_none());
    held.release().expect("release the test's landing lock");
    remove(root.as_path());
}

/// A writer with a free landing lock takes it and hands real Git a lease naming it;
///  releasing frees it.
#[test]
fn a_writer_takes_the_landing_lock_and_hands_over_a_lease() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-free");
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    let coordination: Coordination = coordinate_index_writer(&input)
        .expect("no failure")
        .expect("an index writer coordinates");
    assert_eq!(coordination.lease.0, OsString::from(LANDING_LEASE_VARIABLE));
    let value: String = coordination.lease.1.to_string_lossy().into_owned();
    assert!(value.contains("landing.lock"), "{value}");
    let contested: Option<OwnerLock> = try_acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        None,
    )
    .expect("a readable lock");
    assert!(contested.is_none(), "the coordinated writer holds the landing lock");
    coordination.release().expect("release");
    let freed: Option<OwnerLock> = try_acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        None,
    )
    .expect("a readable lock");
    assert!(freed.is_some(), "release frees the landing lock");
    remove(root.as_path());
}

/// A release that cannot prove the lock is still this writer's is reported to the caller, not swallowed.
#[test]
fn a_release_that_finds_the_lock_changed_is_reported() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-release-reported");
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    let coordination: Coordination = coordinate_index_writer(&input)
        .expect("no failure")
        .expect("an index writer coordinates");
    let record: PathBuf = coordination
        .lock
        .directory()
        .join(crate::owner_lock_record::OWNER_LOCK_RECORD_FILENAME);
    std::fs::write(&record, b"{").expect("damage the record");
    let message: String = coordination.release().expect_err("the lock no longer proves its owner");
    assert!(!message.is_empty());
    remove(root.as_path());
}

/// A direct fix holds the repository's landing lock until it releases it,
///  and holds nothing outside a repository.
#[test]
fn a_direct_fix_holds_the_landing_lock() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-hold");
    let held: OwnerLock = hold_landing_lock(&context(), &identity_of(repo.as_path()))
        .expect("no failure")
        .expect("a repository has a landing lock to hold");
    let contested: Option<OwnerLock> = try_acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        None,
    )
    .expect("a readable lock");
    assert!(contested.is_none(), "the direct fix holds the landing lock");
    held.release().expect("release");
    assert!(
        hold_landing_lock(&context(), &WorktreeIdentity::OutsideWorktree)
            .expect("no failure")
            .is_none()
    );
    remove(root.as_path());
}

/// The repository the waiter thread coordinates in;
///  set once by the test that starts it.
static WAITER_REPO: OnceLock<PathBuf> = OnceLock::new();

/// The waiter thread's body:
///  one coordination in the repository the test named.
fn wait_in_thread() -> Result<bool, CoordinationFailure> {
    let repo: PathBuf = match WAITER_REPO.get() {
        Some(path) => path.clone(),
        None => return Err(CoordinationFailure::Stop(String::from("waiter repository not set"))),
    };
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    let found: Option<Coordination> = coordinate_index_writer(&input)?;
    return Ok(found.is_some());
}

/// A writer waits behind a live landing and takes the lock when it is released.
#[test]
fn a_writer_waits_behind_a_live_landing() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-waits");
    let held: OwnerLock = hold_landing(repo.as_path());
    // The waiter thread reads its repository from this slot, which only this test sets.
    let _ = WAITER_REPO.set(repo.clone());
    let waiter: std::thread::JoinHandle<Result<bool, CoordinationFailure>> = std::thread::spawn(wait_in_thread);
    std::thread::sleep(std::time::Duration::from_millis(200));
    held.release().expect("release the landing");
    let outcome: Result<bool, CoordinationFailure> = waiter.join().expect("the waiter finishes");
    assert_eq!(outcome, Ok(true));
    remove(root.as_path());
}

/// A foreign `index.lock` whose PID file names a process that no longer runs fails once the budget is spent,
/// and the landing lock taken for the attempt is released.
#[test]
fn a_dead_foreign_owner_fails_and_releases_the_landing_lock() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-dead-owner");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("foreign lock");
    std::fs::write(repo.join(".git/index~pid.lock"), "pid 2147483646\n").expect("dead PID file");
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 50,
    };
    let failure: CoordinationFailure = match coordinate_index_writer(&input) {
        Err(found) => found,
        Ok(_) => panic!("a dead owner must fail the coordination"),
    };
    match &failure {
        CoordinationFailure::UnprovenOwner(message) => {
            assert!(message.contains("no longer runs"), "{message}");
            assert!(message.contains("did not run git add"), "{message}");
        }
        other => panic!("expected the unproven owner failure, got {other:?}"),
    }
    assert!(failure.stderr().contains("\"code\":\"index-lock-unproven-owner\""));
    let mut silent: SilentWait = SilentWait;
    let retaken: OwnerLock = acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        std::time::Duration::from_millis(5),
        &mut silent,
    )
    .expect("the failed attempt released the landing lock");
    retaken.release().expect("release");
    remove(root.as_path());
}

/// Failures to read state render as content-unavailable engine failures.
#[test]
fn a_read_failure_renders_as_content_unavailable() {
    let failure: CoordinationFailure = CoordinationFailure::Stop(String::from("registry unsafe"));
    assert!(failure.stderr().contains("\"code\":\"content-unavailable\""));
    assert!(failure.stderr().contains("registry unsafe"));
}

/// A Git that reports a release before 2.55.0,
///  as the version answer.
fn old_git(_context: &GitContext) -> Result<crate::git_version::GitRelease, String> {
    return Err(String::from("real Git reports version 2.54.0; cli-git needs Git 2.55.0 or newer"));
}

/// A Git that reports the minimum release,
///  as the version answer.
fn minimum_git(_context: &GitContext) -> Result<crate::git_version::GitRelease, String> {
    return Ok((2, 55, 0));
}

/// An index writer on an older Git is refused before the landing lock is taken:
///  the lock is free
/// afterwards and nothing is coordinated.
///  The minimum release goes through to the lock.
#[test]
fn an_older_git_refuses_an_index_writer_before_the_landing_lock() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-old-git");
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    let refused: CoordinationFailure = match coordinate_index_writer_checked(&input, old_git) {
        Err(found) => found,
        Ok(_) => panic!("an older Git must refuse the index writer"),
    };
    assert!(refused.stderr().contains("2.55.0 or newer"), "{}", refused.stderr());
    ensure_transaction_root(repo.join(".git/cli-git-transactions").as_path()).expect("registry");
    let free: Option<OwnerLock> = try_acquire_owner_lock(
        repo.join(".git/cli-git-transactions/landing.lock").as_path(),
        None,
    )
    .expect("a readable lock");
    assert!(free.is_some(), "the refused writer never took the landing lock");
    remove(root.as_path());
}

/// The minimum release goes through to the landing lock,
///  which the coordination then holds.
#[test]
fn the_minimum_release_is_accepted_and_locks() {
    let (root, repo): (PathBuf, PathBuf) = scene("coordination-minimum-git");
    let command: ResolvedCommand = command_in(repo.as_path(), &["add", "a.txt"]);
    let input: CoordinationInput<'_> = CoordinationInput {
        context: &context(),
        identity: &identity_of(repo.as_path()),
        command: &command,
        environment: &[],
        timeout_ms: 100,
    };
    let held: Coordination = match coordinate_index_writer_checked(&input, minimum_git) {
        Ok(Some(found)) => found,
        Ok(None) => panic!("an index writer coordinates"),
        Err(failure) => panic!("the minimum release is accepted: {}", failure.stderr()),
    };
    held.release().expect("release");
    remove(root.as_path());
}

/// A quiet release frees the landing lock, so the next owner acquires it at once.
#[test]
fn a_quiet_release_frees_the_landing_lock() {
    let root: PathBuf = fixture("landing-quiet-release");
    let lock_directory: PathBuf = root.join("landing.lock");
    let mut silent: SilentWait = SilentWait;
    let lock: OwnerLock = acquire_owner_lock(lock_directory.as_path(), std::time::Duration::from_millis(5), &mut silent)
        .expect("the first owner acquires the landing lock");
    release_quietly(lock);
    let again: Option<OwnerLock> = try_acquire_owner_lock(lock_directory.as_path(), None).expect("the probe runs");
    assert!(again.is_some(), "the quiet release freed the landing lock");
    remove(root.as_path());
}
