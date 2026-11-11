//! What:
//!  A landed commit whose wrapper was killed between promoting its held push and waking the service.
//! Why:
//!  The promotion makes the record runnable and nothing has been told about it,
//! so two promises meet here:
//! a
//!  service that starts runs the work with no commit needed,
//! and every command names a push that is still waiting
//!  (`SPEC.md`,
//! "After landing",
//! and `service_server_work_loop.rs`).
//! Both are checked against real Git and a real bare remote:
//! the commit is not on the remote after the
//!  kill,
//! the next command says it is queued,
//! and a service that starts pushes it with no further commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! killTheWrapperAfterPromote(); expect(await status()).toContain('is queued for its push'); restartTheService(); await untilOnRemote();
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, fixture, git, repository, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::service_fault::AFTER_PROMOTE;
use git_policy_cli::service_handoff::PUSH_QUEUED_AFTER;
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{PENDING, state_directory};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond what a healthy run needs.
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

/// What:
///  Commits `content` to `a.txt` and ends the wrapper at the instant after its push was promoted,
/// so the record is runnable and nothing was woken for it.
/// Why:
///   The fault point is the only way to stand inside that window;
/// it is inert unless the variable names it,
/// so no
///  command of a person reaches it.
fn commit_and_die_after_promote(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    command.env(git_policy_cli::service_fault::FAULT_VARIABLE, AFTER_PROMOTE);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(!output.status.success(), "the wrapper was killed at the fault point: {:?}", output.status);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// The push record the killed wrapper left runnable,
/// as the queue holds it.
fn pending_push_records(setup: &Setup) -> Vec<String> {
    let pending: PathBuf = state_directory(work_directory(git_dir(setup).as_path()).as_path(), PENDING);
    let mut names: Vec<String> = std::fs::read_dir(pending)
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.retain(|name| { return name.starts_with("push-"); });
    return names;
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

fn stderr_of(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// What:
///  Runs a wrapped command and reads it without waiting for the after-commit work to settle.
/// Why:
///   `run_wrapped` waits for the service to settle the repository's records,
/// and this control is about a record
///  that nothing has settled:
/// the wait would time out and fail the control for the state it checks.
fn run_without_settling(setup: &Setup, arguments: &[&str]) -> Observed {
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), arguments);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped command runs");
    return Observed { code: output.status.code(), stdout: output.stdout, stderr: output.stderr };
}

/// What:
///  The command after the kill says the commit's push is queued,
/// and the remote does not have it.
/// Why:
///   `SPEC.md` accepts a record left for the next wake only when the line the person reads says the push is queued,
/// and
///  here the command that would have said it was killed.
/// The line comes from the record itself,
/// so it is said while the
///  record waits and no longer.
#[test]
fn the_command_after_the_kill_says_the_push_is_queued() {
    let setup: Setup = setup("push-queued-after-promote");
    let commit: String = commit_and_die_after_promote(&setup, "one\n");
    assert_eq!(pending_push_records(&setup).len(), 1, "the promotion left one runnable record");
    assert!(!remote_has(&setup, commit.as_str()), "nothing pushed the commit: the wake never happened");
    // A record that has just been queued is a push in flight, not a push that waits: the line is said once the bound is past.
    let early: Observed = run_without_settling(&setup, &["status", "--short"]);
    assert!(!stderr_of(&early).contains(commit.as_str()), "a record younger than the bound is not named: {}", stderr_of(&early));
    let pending: PathBuf = state_directory(work_directory(git_dir(&setup).as_path()).as_path(), PENDING).join(pending_push_records(&setup)[0].as_str());
    // The wait is made by the clock rather than by sleeping the bound: the record is aged, and everything else is real.
    let aged: std::time::SystemTime = std::time::SystemTime::now() - PUSH_QUEUED_AFTER - Duration::from_secs(1);
    std::fs::File::options().write(true).open(pending.as_path()).expect("the record opens").set_modified(aged).expect("the record is aged");
    let said: Observed = run_without_settling(&setup, &["status", "--short"]);
    let text: String = stderr_of(&said);
    assert_eq!(said.code, Some(0), "{text}");
    assert!(text.contains(commit.as_str()), "the line names the commit: {text}");
    assert!(text.contains("is queued for its push from refs/heads/main"), "the line says the push is queued: {text}");
    assert!(text.contains("has not been made yet"), "and that it has not been made: {text}");
    assert!(text.contains("needs no further commit"), "and that the service needs no further commit: {text}");
    assert!(!remote_has(&setup, commit.as_str()), "still nothing pushed it");
    remove_settled(&setup.fixture);
}

/// What:
///  A service that starts after the kill pushes the record with no further commit,
/// which is the promise `service_server_work_loop.rs` makes.
/// Why:
///   The landing wrote the repository down before it answered,
/// so the next service looks at it without any wake;
/// the wrapper's wake is the only other thing that would have written it,
/// and that is what the kill took.
#[test]
fn a_service_that_starts_pushes_the_queued_record_with_no_further_commit() {
    let setup: Setup = setup("push-queued-service-starts");
    let commit: String = commit_and_die_after_promote(&setup, "one\n");
    let head: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    assert_eq!(head, commit, "the commit landed");
    assert!(!remote_has(&setup, commit.as_str()), "nothing pushed it before the restart");
    // The manager starting the service again after it ended: the fixture has no manager, so it does what one does.
    setup.service.stop_all();
    setup.service.serve(&setup.fixture, git_dir(&setup).as_path());
    wait_until("the restarted service pushed the queued record", &|| { return remote_has(&setup, commit.as_str()); });
    // No commit was made for it: the branch is still the one the killed wrapper left.
    assert_eq!(String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]).stdout).trim(), commit, "no further commit was needed");
    assert!(pending_push_records(&setup).is_empty(), "the record is no longer waiting");
    remove_settled(&setup.fixture);
}

/// What:
///  A commit the post-commit gate blocks leaves no runnable record and no queued line.
/// Why:
///   The negative control of the line:
///  the state it is read from is the promotion,
/// and a blocked gate never promotes.
/// A blocked commit is a separate question that is open with the owner,
/// so this control only proves the line does not invent one.
#[test]
fn a_commit_that_never_landed_a_runnable_push_is_not_named_as_queued() {
    let setup: Setup = setup("push-queued-negative");
    let commit: String = commit_through(&setup, "one\n");
    wait_until("the push settled", &|| { return pending_push_records(&setup).is_empty(); });
    wait_until("the remote has the commit", &|| { return remote_has(&setup, commit.as_str()); });
    let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    assert!(!stderr_of(&said).contains(commit.as_str()), "a delivered commit is not named as queued: {}", stderr_of(&said));
    remove_settled(&setup.fixture);
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

/// What:
///  The push records a worktree holds,
///  as the queue keeps them.
/// Why:
///  The state the command is about lives in the records,
///  so a case reads them rather than what a command printed.
fn held_push_records(setup: &Setup) -> Vec<String> {
    let held: PathBuf = state_directory(work_directory(git_dir(setup).as_path()).as_path(), git_policy_cli::work_item::HELD);
    let mut names: Vec<String> = std::fs::read_dir(held)
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.retain(|name| { return name.starts_with("push-"); });
    return names;
}

/// What:
///  Commits `content` to `a.txt` and ends the wrapper at the instant after it wrote its pass into the push record,
///  before it made the record runnable.
/// Why:
///  That is the state a killed command leaves:
///  the record says the checks passed,
///  its command is gone,
///  and nothing has been woken for it.
fn commit_and_die_after_the_verdict(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    command.env(git_policy_cli::service_fault::FAULT_VARIABLE, git_policy_cli::service_fault::AFTER_THE_VERDICT);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(!output.status.success(), "the wrapper was killed at the fault point: {:?}", output.status);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// What:
///  A command whose output a program reads releases the push an approved record holds,
///  so an approved push never waits for a person to type a command only a program will read.
/// Why:
///  The record is the only place the state exists,
///  and the command that would have promoted it is gone;
///  a workflow of scripts runs no other kind of command,
///  so leaving the settlement to the next command a person types leaves the push waiting forever.
///  The command still shows nothing and takes no failure notice:
///  its output is read by a program.
#[test]
fn a_machine_read_command_releases_a_held_push_whose_command_left_a_pass() {
    let setup: Setup = setup("push-held-released-by-a-program");
    let commit: String = commit_and_die_after_the_verdict(&setup, "one\n");
    assert_eq!(held_push_records(&setup).len(), 1, "the killed command left one held record");
    assert!(pending_push_records(&setup).is_empty(), "and nothing runnable");
    assert!(!remote_has(&setup, commit.as_str()), "nothing pushed the commit yet");

    // `rev-parse` is a command whose output a program reads.
    let read: Observed = run_without_settling(&setup, &["rev-parse", "HEAD"]);

    assert_eq!(read.code, Some(0), "{}", stderr_of(&read));
    assert_eq!(String::from_utf8_lossy(read.stdout.as_slice()).trim(), commit, "the command's own answer is unchanged");
    assert_eq!(held_push_records(&setup).len(), 0, "the record the command found is settled");
    // The record became runnable and the service was woken for it: the service claims it at once, so the record's own state
    //  is read from the push it makes rather than from `pending`, which the service empties as fast as it fills.
    wait_until("the service pushed the record the program's command released", &|| { return remote_has(&setup, commit.as_str()); });
    remove_settled(&setup.fixture);
}
