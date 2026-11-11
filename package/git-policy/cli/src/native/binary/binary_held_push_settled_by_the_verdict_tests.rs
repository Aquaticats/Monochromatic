//! What:
//!  A landed commit whose command died before it settled the push record it holds,
//!  and what the branch's backup does about it.
//! Why:
//!  The service holds a landed commit's push before it answers,
//! and the command that landed the commit is the one that
//!  promotes or forgets that record.
//! A command that stops first leaves a record nothing in the process table will settle,
//! so the
//!  branch's backup waits on a command that cannot answer,
//! and every command said the push was held without naming what would ever
//!  make it
//! (`doc/handover/questions-for-the-owner.md`,
//! "A commit whose command died before its checks ran now stays local").
//! Both halves are checked here against real Git,
//! a real bare remote and a real service:
//! a record whose own words
//!  say the checks passed is released and pushed,
//! a record whose words say they were refused stays local and is named,
//! and a record
//!  with no verdict names the command that ended and the push that sends the commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! killTheWrapperAfterItsVerdict(); const said = await status(); expect(said).toContain('is queued for its push'); await untilOnRemote();
//! ```

use super::commit::remove_settled;
use super::support::{Fixture, Observed, fixture, git, repository, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::service_work::work_directory;
use git_policy_cli::work_item::{HELD, PENDING, state_directory};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// The longest a control waits for anything,
/// far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// The fault point after the service answered and before the wrapper's gate ran,
/// which leaves a landed commit whose checks nothing ever ran.
const AFTER_ANSWER: &str = "after-answer";

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

/// The held records this worktree holds,
/// as the queue names them.
fn held_push_records(setup: &Setup) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(state_directory(work_directory(git_dir(setup).as_path()).as_path(), HELD))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.retain(|name| { return name.starts_with("push-"); });
    names.sort();
    return names;
}

/// The records waiting for the service,
/// as the queue names them.
fn pending_push_records(setup: &Setup) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(state_directory(work_directory(git_dir(setup).as_path()).as_path(), PENDING))
        .map(|listing| { return listing.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); })
        .unwrap_or_default();
    names.retain(|name| { return name.starts_with("push-"); });
    names.sort();
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
///  Commits `content` to `a.txt` and ends the wrapper at the instant `fault` names,
/// so the commit is on the branch and its push record is where the command left it.
/// Why:
///   A fault point is the only way to stand inside a window of the command that lasts microseconds;
/// it is inert unless
///  the variable names it,
/// so no command of a person reaches it.
fn commit_and_die_at(setup: &Setup, content: &str, fault: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    command.env(git_policy_cli::service_fault::FAULT_VARIABLE, fault);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(!output.status.success(), "the wrapper was killed at the fault point: {:?}", output.status);
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// What:
///  A landed commit whose command recorded the gate's pass and then died is backed up by the next command.
/// Why:
///   The checks cleared the commit,
/// so the hold may end,
/// and the words the record keeps are the only statement left that
///  they did:
/// the next command releases it,
/// says the push is queued,
/// and wakes the service that makes it.
#[test]
fn a_commit_whose_command_left_a_pass_is_backed_up_by_the_next_command() {
    let setup: Setup = setup("held-by-verdict-pass");
    let commit: String = commit_and_die_at(&setup, "one\n", git_policy_cli::service_fault::AFTER_THE_VERDICT);
    assert_eq!(held_push_records(&setup).len(), 1, "the command died before it promoted its record");
    assert!(pending_push_records(&setup).is_empty(), "so nothing is runnable yet");
    assert!(!remote_has(&setup, commit.as_str()), "nothing pushed the commit");
    // The command's own words, as they stand in the record it left.
    let record: String = std::fs::read_to_string(
        state_directory(work_directory(git_dir(&setup).as_path()).as_path(), HELD).join(held_push_records(&setup)[0].as_str()),
    )
    .expect("the record is readable");
    assert!(record.contains("\"verdict\":\"passed\""), "the record says the gate approved the backup: {record}");
    // The next command of the worktree settles the record from those words, and the service it wakes makes the push.
    let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    let text: String = stderr_of(&said);
    assert_eq!(said.code, Some(0), "{text}");
    assert!(text.contains(commit.as_str()), "the released push is named: {text}");
    assert!(text.contains("is queued for its push from refs/heads/main"), "and said to be queued: {text}");
    wait_until("the woken service pushed the released record", &|| { return remote_has(&setup, commit.as_str()); });
    assert!(held_push_records(&setup).is_empty(), "the record left the held state");
    // Every later command agrees with Git, and `git status` shows nothing left to push.
    let after: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    assert!(!stderr_of(&after).contains(commit.as_str()), "the commit on the remote is not said to be held: {}", stderr_of(&after));
    let status: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["status", "--porcelain=v1", "--branch"]).stdout).into_owned();
    assert_eq!(status.trim_end(), "## main...origin/main", "Git shows nothing left to push");
    remove_settled(&setup.fixture);
}

/// The configuration that makes the commit-time policy refuse content it would only warn about.
const STRICT_FINAL_NEWLINE: &str = "{ \"policies\": { \"final-newline\": \"error\" } }\n";

/// What:
///  A landed commit whose command recorded the gate's refusal and then died stays local,
/// and every command names what holds it.
/// Why:
///   A commit a policy blocked stays local,
/// so the record must never become runnable;
/// the verdict the next push of the branch stops
///  before is written from the words the record kept,
/// and the commit never reaches the remote.
#[test]
fn a_commit_whose_command_left_a_refusal_stays_local_and_is_named() {
    let setup: Setup = setup("held-by-verdict-refusal");
    std::fs::write(setup.repo.join("monochromatic-git-policy-cli.config.jsonc"), STRICT_FINAL_NEWLINE).expect("the configuration");
    git(&setup.fixture, &setup.repo, &["add", "monochromatic-git-policy-cli.config.jsonc"]);
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message=the strict configuration"]);
    // Two commits a person's own Git made, so neither went through a gate: the second carries content the strict policy refuses, and
    //  the amend below keeps that content while changing only `a.txt`, so the pre-commit pass sees nothing to refuse.
    std::fs::write(setup.repo.join("a.txt"), "before\n").expect("a");
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message=before", "a.txt"]);
    std::fs::write(setup.repo.join("bad.txt"), "no final newline").expect("bad.txt");
    git(&setup.fixture, &setup.repo, &["add", "bad.txt"]);
    git(&setup.fixture, &setup.repo, &["commit", "--quiet", "--message=refused content"]);
    // The amend lands a commit the gate then refuses, and the fault ends the command between that refusal and its verdict.
    std::fs::write(setup.repo.join("a.txt"), "amended\n").expect("a");
    let mut amend: Command = super::commit::wrapped_commit(&setup.fixture, setup.repo.as_path(), &["commit", "--amend", "--no-edit", "-m", "amended", "--", "a.txt"]);
    amend.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    amend.env(git_policy_cli::service_fault::FAULT_VARIABLE, git_policy_cli::service_fault::AFTER_THE_VERDICT);
    let refused: Output = amend.output().expect("the wrapped amend runs");
    assert!(!refused.status.success(), "the wrapper was killed at the fault point: {:?}", refused.status);
    let refused_commit: String = String::from_utf8_lossy(&git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]).stdout).trim().to_string();
    let record: String = std::fs::read_to_string(
        state_directory(work_directory(git_dir(&setup).as_path()).as_path(), HELD).join(held_push_records(&setup)[0].as_str()),
    )
    .expect("the record is readable");
    assert!(record.contains("\"verdict\":\"blocked\""), "the record says the gate refused the backup: {record}");
    assert!(record.contains("final-newline"), "and keeps the gate's own words: {record}");
    // The next command settles the record into the verdict, and the commit stays where it is.
    let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    let text: String = stderr_of(&said);
    assert!(held_push_records(&setup).is_empty(), "the record left the held state: {text}");
    assert!(pending_push_records(&setup).is_empty(), "and never became runnable: {text}");
    assert!(text.contains("main is held back by the commit"), "the commit is named as one that holds the branch back: {text}");
    assert!(text.contains(refused_commit.as_str()), "the line names the commit: {text}");
    assert!(text.contains("final-newline"), "the line names the check: {text}");
    assert!(text.contains("a person's `git push` still sends it"), "the line names the way out: {text}");
    assert!(!remote_has(&setup, refused_commit.as_str()), "a commit a policy blocked stays local");
    remove_settled(&setup.fixture);
}

/// What:
///  A landed commit whose command died before its checks ran names that command and what sends it.
/// Why:
///   Nothing may publish a commit whose checks never ran,
/// and the person must still be told which commit waits,
/// which
///  command ended,
/// and what sends it instead,
/// rather than a next push nothing in this worktree makes.
#[test]
fn a_commit_whose_command_died_before_its_checks_names_what_sends_it() {
    let setup: Setup = setup("held-by-no-verdict");
    let commit: String = commit_and_die_at(&setup, "one\n", AFTER_ANSWER);
    let record: String = std::fs::read_to_string(
        state_directory(work_directory(git_dir(&setup).as_path()).as_path(), HELD).join(held_push_records(&setup)[0].as_str()),
    )
    .expect("the record is readable");
    assert!(record.contains("\"verdict\":\"undecided\""), "the command died before it decided: {record}");
    let said: Observed = run_wrapped(&setup.fixture, setup.repo.as_path(), &["status", "--short"]);
    let text: String = stderr_of(&said);
    assert!(text.contains(commit.as_str()), "the commit is named: {text}");
    assert!(text.contains("the command that landed it"), "the line names what holds it: {text}");
    assert!(text.contains("ended before its checks ran"), "{}", text);
    assert!(text.contains("a push you make of refs/heads/main sends it"), "the remedy is named: {text}");
    assert_eq!(held_push_records(&setup).len(), 1, "no verdict was written, so the commit stays local");
    assert!(!remote_has(&setup, commit.as_str()), "and nothing published it");
    remove_settled(&setup.fixture);
}
