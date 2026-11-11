//! What:
//!  A push record held for a commit that a later push of the branch carried to the remote anyway.
//! Why:
//!  A held record is not stranded by a kill:
//! the queue pushes the branch,
//! so the next push from that branch carries the
//!  held commit.
//! Once it has,
//! every command that still says the commit is held says the opposite of what
//!  `git status` shows,
//! and nothing but deleting the record by hand ends it.
//! The record is settled on Git's
//!  own proof here,
//! against real Git and a real bare remote:
//! the commit reaches the remote with a later push,
//! and the
//!  next command stops naming it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! killTheWrapperAfterTheAnswer(); await commitB(); expect(remoteHas(A)); expect(await status()).not.toContain(A);
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, fixture, git, repository, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::service_fault::FAULT_VARIABLE;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{DONE, HELD, WorkItem, WorkKind, state_directory};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// The fault point after the service answered and before the wrapper's gate promoted the push.
const AFTER_ANSWER: &str = "after-answer";

/// The longest a control waits for anything.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with one tracked file,
/// a bare remote that has the branch,
/// and a service the commits go through.
struct Setup {
    fixture: Fixture,
    repo: PathBuf,
    remote: PathBuf,
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
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, repo.join(".git").as_path());
    return Setup { fixture, repo, remote, service };
}

fn git_dir(setup: &Setup) -> PathBuf {
    return setup.repo.join(".git");
}

fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// What:
///  Commits `content` and ends the wrapper right after the service answered,
/// so the push record stays held.
/// Why:
///   The gate is what promotes the record,
/// and a kill before it is the state the held line exists for.
fn commit_and_die_after_the_answer(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    command.env(FAULT_VARIABLE, AFTER_ANSWER);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(!output.status.success(), "the wrapper was killed at the fault point: {:?}", output.status);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// Commits `content` through the service and returns the new commit's name.
fn commit_through(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// Whether the bare remote holds `commit`.
/// The question is asked of real Git and a "no" is an answer,
/// not a failure of the command.
fn remote_has(setup: &Setup, commit: &str) -> bool {
    let asked: Output = super::support::direct(&setup.fixture)
        .current_dir(setup.remote.as_path())
        .args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()])
        .output()
        .expect("real Git answers");
    return asked.status.success();
}

fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The records of the work queue of the setup,
/// by state.
fn records_in(setup: &Setup, state: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(state_directory(work_directory(git_dir(setup).as_path()).as_path(), state))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.sort();
    return names;
}

/// The record's own id for the push of `commit` on `main`.
fn push_id(commit: &str) -> String {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::new(),
        git_dir: PathBuf::new(),
        branch_ref: String::from("refs/heads/main"),
        oid: commit.to_string(),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    }
    .id();
}

/// What:
///  A held commit that a later push carried to the remote is no longer said to be held,
/// and its record is settled.
/// Why:
///   The line and `git status` must agree:
/// after the later push the branch is on the remote,
/// and the record's
///  own state is the only thing that could still say otherwise.
#[test]
fn a_held_commit_a_later_push_carried_is_no_longer_said_to_be_held() {
    let setup: Setup = setup("held-carried-by-a-later-push");
    let held: String = commit_and_die_after_the_answer(&setup, "one\n");
    let id: String = push_id(held.as_str());
    assert_eq!(records_in(&setup, HELD), vec![id.clone()], "the killed wrapper left its push held");
    assert!(!remote_has(&setup, held.as_str()), "nothing pushed it");
    // The held line is true here, and it is the state the next assertion is about.
    let before: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    assert!(stderr_of(&before).contains(held.as_str()), "the held commit is named: {}", stderr_of(&before));
    assert!(stderr_of(&before).contains("the command that landed it"), "the line names what holds it: {}", stderr_of(&before));
    assert!(stderr_of(&before).contains("ended before its checks ran"), "{}", stderr_of(&before));
    assert!(stderr_of(&before).contains("a push you make of refs/heads/main sends it"), "the remedy is named: {}", stderr_of(&before));
    // A later commit of the branch pushes the branch, and the queue pushes the branch: the held commit goes with it.
    let later: String = commit_through(&setup, "two\n");
    wait_until("the remote branch is the later commit", &|| {
        let published: String = String::from_utf8_lossy(&git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "refs/heads/main"]).stdout).trim().to_string();
        return published == later;
    });
    assert!(remote_has(&setup, held.as_str()), "the later push carried the held commit");
    // The proof is Git's own record of the remote branch, so the control waits until that record agrees with the remote: a push
    //  that was stopped after it delivered the commits leaves the remote-tracking ref behind, and the held line is then still
    //  true of the record.
    wait_until("the remote-tracking ref is the remote's tip", &|| {
        let tracked: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "refs/remotes/origin/main"]).stdout).trim().to_string();
        let published: String = String::from_utf8_lossy(&git(&setup.fixture, setup.remote.as_path(), &["rev-parse", "refs/heads/main"]).stdout).trim().to_string();
        return tracked == published;
    });
    let status: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["status", "--porcelain=v1", "--branch"]).stdout).into_owned();
    assert_eq!(status.trim_end(), "## main...origin/main", "Git shows nothing left to push");
    // Every later command agrees with Git from here on.
    let after: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    assert!(!stderr_of(&after).contains(held.as_str()), "the commit the remote holds is not said to be held: {}", stderr_of(&after));
    assert!(!stderr_of(&after).contains("ended before its checks ran"), "{}", stderr_of(&after));
    assert!(records_in(&setup, HELD).is_empty(), "the record left the held state: {:?}", records_in(&setup, HELD));
    assert!(records_in(&setup, DONE).contains(&id), "and is known to the queue as a push that was made: {:?}", records_in(&setup, DONE));
    // Settled once: the command after it says nothing either.
    let again: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["log", "--oneline"]);
    assert!(!stderr_of(&again).contains(held.as_str()), "{}", stderr_of(&again));
    remove_settled(&setup.fixture);
}

/// What:
///  A held commit the remote does not hold keeps its line,
/// which is the control of the proof's "no".
/// Why:
///   The settlement must not be a guess:
/// a record whose commit is only local is the one thing that tells the person
///  their backup is missing,
/// and it stays said however often a command runs.
#[test]
fn a_held_commit_the_remote_does_not_hold_keeps_its_line() {
    let setup: Setup = setup("held-not-carried");
    let held: String = commit_and_die_after_the_answer(&setup, "one\n");
    assert!(!remote_has(&setup, held.as_str()), "nothing pushed it");
    for _ in 0..2 {
        let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
        assert!(stderr_of(&said).contains(held.as_str()), "the held commit is still named: {}", stderr_of(&said));
        assert!(stderr_of(&said).contains("ended before its checks ran"), "{}", stderr_of(&said));
    }
    assert_eq!(records_in(&setup, HELD), vec![push_id(held.as_str())], "the record stays held");
    remove_settled(&setup.fixture);
}
