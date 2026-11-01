//! What:
//!  A backup push the resident service made after a commit returned and the remote refused, shown by the next wrapper
//!  command in the repository.
//! Why:
//!  The service pushes after the commit command exited, so nothing the commit printed can carry the refusal. Without a
//!  record shown by a later command the person would never learn that their backup is missing
//!  (`push_failure_notice.rs`). The remote here is a real bare repository whose `pre-receive` hook refuses every push, the
//!  service is a real process, and the commands are the built wrapper; nothing is a stand-in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await commitThroughService(); await untilRecorded(); expect(stderrOf(await run('git status'))).toContain('backup push');
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, repository, run_direct, run_wrapped, start_fixture_service, FixtureService};
use git_policy_cli::push_failure_notice::notice_directory;
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// What the remote's hook says when it refuses a push; the test looks for it in what the person is shown.
const REFUSAL: &str = "refusing this push by test policy";

/// The longest a control waits for the service's push, far beyond what a healthy run needs.
const PATIENCE: Duration = Duration::from_secs(60);

/// A repository with a tracked file, a bare remote that has the branch, and a service the commits go through.
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
    return Setup { fixture, repo, remote, service };
}

/// The remote refuses every push from now on.
fn refuse_pushes(remote: &Path) {
    let hook: PathBuf = remote.join("hooks/pre-receive");
    std::fs::create_dir_all(hook.parent().expect("hooks directory")).expect("hooks");
    std::fs::write(&hook, format!("#!/bin/sh\necho '{REFUSAL}' >&2\nexit 1\n")).expect("hook");
    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).expect("executable");
}

/// The remote takes pushes again.
fn accept_pushes(remote: &Path) {
    std::fs::remove_file(remote.join("hooks/pre-receive")).expect("hook removed");
}

/// Commits `content` to `a.txt` through the service and returns the new commit's name.
fn commit_through_service(setup: &Setup, content: &str) -> String {
    std::fs::write(setup.repo.join("a.txt"), content).expect("edit");
    let mut command: Command = wrapped_commit(&setup.fixture, &setup.repo, &["commit", "--quiet", "-m", content.trim(), "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", &setup.service.runtime);
    let output: Output = command.output().expect("the wrapped commit runs");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let named = git(&setup.fixture, &setup.repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(&named.stdout).trim().to_string();
}

/// Waits until the condition holds; the service's push has no signal a control could block on.
fn wait_until(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "never happened: {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn common_dir(setup: &Setup) -> PathBuf {
    return setup.repo.join(".git");
}

/// How many failure records wait in the repository.
fn waiting_records(setup: &Setup) -> usize {
    return std::fs::read_dir(notice_directory(common_dir(setup).as_path())).map_or(0, |listing| { return listing.flatten().count(); });
}

/// Waits until the service has pushed `commit` and the item is settled.
fn wait_until_settled(setup: &Setup, commit: &str) {
    let work: PathBuf = common_dir(setup).join("cli-git/work");
    wait_until("the service settled the push", &|| {
        return ["done", "failed"].iter().any(|state| { return work.join(state).join(format!("push-{commit}")).exists(); });
    });
}

fn stderr_text(observed: &Observed) -> String {
    return String::from_utf8_lossy(observed.stderr.as_slice()).into_owned();
}

/// A push the remote refused after the commit returned is shown by the next wrapper command, with the branch, the commit,
///  Git's own reason, the fact that the commit is saved here, and what to do.
#[test]
fn a_rejected_service_push_is_shown_on_the_next_command() {
    let setup: Setup = setup("push-failure-shown");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let shown: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    let text: String = stderr_text(&shown);
    assert_eq!(shown.code, Some(0), "{text}");
    for expected in ["branch main", &commit[..7], REFUSAL, "saved in this repository", "`git push`"] {
        assert!(text.contains(expected), "missing {expected:?} in: {text}");
    }
    remove_settled(&setup.fixture);
}

/// The failure is shown once: the command after the one that showed it prints nothing about it.
#[test]
fn a_shown_failure_is_not_shown_again() {
    let setup: Setup = setup("push-failure-once");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    let first: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(first.contains(REFUSAL), "{first}");
    let second: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&second), "", "the same failure again");
    let third: Observed = run_wrapped(&setup.fixture, &setup.repo, &["log", "--oneline"]);
    assert_eq!(stderr_text(&third), "", "the same failure again, in another command");
    assert_eq!(waiting_records(&setup), 0);
    remove_settled(&setup.fixture);
}

/// A later push of the branch that goes through clears a failure nobody has seen yet, so the person is not told about a
///  backup that has since been made.
#[test]
fn a_later_successful_push_clears_the_unshown_failure() {
    let setup: Setup = setup("push-failure-cleared");
    refuse_pushes(setup.remote.as_path());
    let failed: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, failed.as_str());
    assert_eq!(waiting_records(&setup), 1, "the refusal was recorded");
    accept_pushes(setup.remote.as_path());
    let pushed: String = commit_through_service(&setup, "third\n");
    wait_until_settled(&setup, pushed.as_str());
    assert_eq!(waiting_records(&setup), 0, "the push that went through cleared the record");
    let after: Observed = run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]);
    assert_eq!(stderr_text(&after), "");
    remove_settled(&setup.fixture);
}

/// A command whose output a program reads gets exactly Git's bytes on both streams and leaves the record for a person:
///  porcelain status, null-separated status and plumbing all pass untouched, and the next ordinary command still shows it.
#[test]
fn machine_read_output_is_untouched_and_keeps_the_failure_for_a_person() {
    let setup: Setup = setup("push-failure-porcelain");
    refuse_pushes(setup.remote.as_path());
    let commit: String = commit_through_service(&setup, "second\n");
    wait_until_settled(&setup, commit.as_str());
    std::fs::write(setup.repo.join("untracked.txt"), "new\n").expect("untracked");
    for arguments in [&["status", "--porcelain=v2", "--branch"][..], &["status", "-z"], &["rev-parse", "HEAD"], &["log", "-1", "--format=%H"]] {
        let wrapped: Observed = run_wrapped(&setup.fixture, &setup.repo, arguments);
        let direct: Observed = run_direct(&setup.fixture, &setup.repo, arguments);
        assert_eq!(wrapped.stdout, direct.stdout, "{arguments:?}");
        assert_eq!(stderr_text(&wrapped), stderr_text(&direct), "{arguments:?}");
        assert_eq!(waiting_records(&setup), 1, "{arguments:?} took the record meant for a person");
    }
    let human: String = stderr_text(&run_wrapped(&setup.fixture, &setup.repo, &["status", "--short"]));
    assert!(human.contains(REFUSAL), "{human}");
    remove_settled(&setup.fixture);
}
