//! What:
//!  The report a reconcile owes the person when it published a merge but could not move the local branch,
//! or moved it beside uncommitted work.
//! Why:
//!  `SPEC.md` "Reconcile and bounds" says the report names the reason,
//! with the overlapping paths,
//! and that
//!  the note names `git merge --ff-only <remote>/<branch>`.
//! The push runs in the service after the commit command returned,
//! so the words have to
//!  reach the person through a record the next command shows,
//! once.
//! Everything here is real:
//! a bare remote,
//! a second clone that advances it,
//! a real service,
//! and the built wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await otherCloneAdvances(); await commitThroughService(); expect(await status()).toContain('git merge --ff-only origin/main');
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, fixture, git, repository, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::push_failure_notice::notice_directory;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{CLAIMED, DONE, FAILED, RECOVERING};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// The longest a control waits for anything.
const PATIENCE: Duration = Duration::from_secs(90);

/// A repository with one tracked file,
/// a bare remote that has the branch,
/// a second clone that can move it,
/// and a service the commits go through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
    other: PathBuf,
    service: FixtureService,
}

fn setup(name: &str) -> Setup {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&fixture, &repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    // The second clone is made by the first control that moves the remote: a clone made here would lack the commits the
    //  repository under test pushes before that, and its push would be refused for the wrong reason.
    let other: PathBuf = fixture.root.join("other");
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, other, service };
}

fn git_dir(setup: &Setup) -> PathBuf {
    return setup.repo.join(".git");
}

fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// The second clone adds a file and pushes it,
/// so the next push of the branch is refused as stale and reconciled.
fn advance_the_remote(setup: &Setup, file: &str, content: &str) {
    if !setup.other.exists() {
        git(&setup.fixture, setup.fixture.root.as_path(), &["clone", "--quiet", setup.remote.to_str().expect("UTF-8"), setup.other.to_str().expect("UTF-8")]);
    }
    std::fs::write(setup.other.join(file), content).expect("the other clone's file");
    git(&setup.fixture, setup.other.as_path(), &["add", file]);
    git(&setup.fixture, setup.other.as_path(), &["commit", "--quiet", "--message=remote side"]);
    git(&setup.fixture, setup.other.as_path(), &["push", "--quiet", "origin", "main"]);
}

/// Commits `content` to `a.txt` through the service and returns the new commit's name.
fn commit_through(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Whether the service holds a record of this repository in any state a push passes through.
fn push_still_running(setup: &Setup) -> bool {
    let work: PathBuf = work_directory(git_dir(setup).as_path());
    return [CLAIMED, RECOVERING].iter().any(|state| {
        let pending: PathBuf = git_policy_cli::work_item::state_directory(work.as_path(), state);
        return std::fs::read_dir(pending).is_ok_and(|listing| { return listing.flatten().any(|entry| { return entry.file_name().to_string_lossy().starts_with("push-"); }); });
    });
}

/// Waits until the service has settled the push of `commit`.
fn wait_until_settled(setup: &Setup) {
    wait_until("the service settled the push", &|| {
        let work: PathBuf = work_directory(git_dir(setup).as_path());
        let done: PathBuf = git_policy_cli::work_item::state_directory(work.as_path(), DONE);
        let failed: PathBuf = git_policy_cli::work_item::state_directory(work.as_path(), FAILED);
        let settled = |directory: &Path| {
            return std::fs::read_dir(directory).is_ok_and(|listing| { return listing.flatten().any(|entry| { return entry.file_name().to_string_lossy().starts_with("push-"); }); });
        };
        return !push_still_running(setup) && (settled(done.as_path()) || settled(failed.as_path()));
    });
}

/// The report a reconcile owes when it published a merge and could not move the branch,
/// with the reason and the command that finishes the move,
/// is shown by the next command and only once.
#[test]
fn a_reconcile_that_could_not_move_the_branch_shows_its_report_once() {
    let setup: Setup = setup("reconcile-report-shown");
    advance_the_remote(&setup, "b.txt", "remote side\n");
    // The file the merge writes is untracked here, so Git refuses to overwrite it and the branch stays behind.
    std::fs::write(setup.repo.join("b.txt"), "local untracked data\n").expect("an untracked file in the way");
    let commit: String = commit_through(&setup, "local side\n");
    wait_until_settled(&setup);
    // The merge was published and the branch was not moved: what Git shows is the state the report explains.
    let status: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["status", "--porcelain=v1", "--branch"]).stdout).into_owned();
    assert!(status.contains("[behind 2]"), "the branch is behind the merge: {status}");
    assert_eq!(std::fs::read_to_string(setup.repo.join("b.txt")).expect("the untracked file"), "local untracked data\n", "the person's file was not overwritten");
    assert_eq!(
        std::fs::read_to_string(setup.repo.join("a.txt")).expect("a.txt"),
        "local side\n",
        "the branch was not moved"
    );
    let head: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    assert_eq!(head, commit, "the branch stayed at the commit it was pushed from");
    let shown: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    let text: String = stderr_of(&shown);
    assert_eq!(shown.code, Some(0), "{text}");
    assert!(text.contains("was not fast-forwarded") || text.contains("unchanged"), "the report says what became of the branch: {text}");
    assert!(text.contains("b.txt"), "the report names the overlapping path: {text}");
    assert!(text.contains("git merge --ff-only origin/main"), "the report names the command that finishes the move: {text}");
    // Shown once: the record went with the words, and the command after it says nothing about the merge.
    let again: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    assert!(!stderr_of(&again).contains("git merge --ff-only"), "the report is shown once: {}", stderr_of(&again));
    assert!(!notice_directory(git_dir(&setup).as_path()).is_dir() || std::fs::read_dir(notice_directory(git_dir(&setup).as_path())).expect("the notice directory").next().is_none(), "no record waits");
    remove_settled(&setup.fixture);
}

/// The report a reconcile owes when it did move the branch beside uncommitted work says the work was left as it is.
#[test]
fn a_reconcile_that_moved_the_branch_beside_uncommitted_work_says_so() {
    let setup: Setup = setup("reconcile-report-left-as-it-is");
    // A file the merge does not write, edited and not staged: the move changes none of it, so it follows.
    std::fs::write(setup.repo.join("c.txt"), "base c\n").expect("c.txt");
    git(&setup.fixture, &setup.repo, &["add", "c.txt"]);
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message=c base"]);
    git(&setup.fixture, &setup.repo, &["push", "--quiet", "origin", "main"]);
    advance_the_remote(&setup, "b.txt", "remote side\n");
    std::fs::write(setup.repo.join("c.txt"), "LOCAL UNCOMMITTED EDIT\n").expect("an unstaged edit");
    let commit: String = commit_through(&setup, "local side\n");
    wait_until_settled(&setup);
    let remote_tip: String = String::from_utf8_lossy(&git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "refs/heads/main"]).stdout).trim().to_string();
    let head: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    assert_ne!(head, commit, "the branch followed the merge");
    assert_eq!(head, remote_tip, "the branch is the published merge");
    assert_eq!(std::fs::read_to_string(setup.repo.join("c.txt")).expect("c.txt"), "LOCAL UNCOMMITTED EDIT\n", "the edit is intact");
    let shown: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    let text: String = stderr_of(&shown);
    assert!(text.contains("your uncommitted changes were left as they are"), "the report says the work was left as it is: {text}");
    remove_settled(&setup.fixture);
}
