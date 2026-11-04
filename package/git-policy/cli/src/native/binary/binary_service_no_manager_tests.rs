//! What:
//!  The built wrapper on a machine where no commit service can be reached or started, and in a directory that belongs to no
//!  repository.
//! Why:
//!  The service is the only writer of a commit (`doc/planning/cli-git-resident-service.md`, "Provisioning"). With no runtime
//!  directory, no socket and no session bus the commit stops with the plain message of its platform, names the remedy,
//!  and changes nothing; the wrapper never lands a commit by another way, and never starts the service itself. Outside a
//!  repository there is no worktree to send a commit for, and the person is told what native Git tells them there.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, observe, repository, run_direct, stderr_of};
use git_policy_cli::service_messages::{LINGER_COMMAND, NOTHING_CHANGED};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

/// A repository with one committed file and an edit of it waiting for a commit.
fn edited(name: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), "base\n").expect("a");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "base\nmore\n").expect("edit");
    return (fixture, repo);
}

/// The subject of the newest commit.
fn newest_subject(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["log", "-1", "--format=%s"]).trim().to_string();
}

/// A commit run with no runtime directory and no session bus stops with the platform's message, names the way out, says that
///  nothing changed, and leaves the commit unmade, the edit in place and no record of an unfinished commit behind.
#[test]
fn a_commit_with_no_service_manager_stops_plainly_and_changes_nothing() {
    let (fixture, repo) = edited("no-manager-plain");
    let mut command = wrapped_commit(&fixture, &repo, &["commit", "--message=second", "--", "a.txt"]);
    command.env_remove("XDG_RUNTIME_DIR").env_remove("DBUS_SESSION_BUS_ADDRESS");
    let output: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("this commit was not made"), "{stderr}");
    assert!(stderr.contains(LINGER_COMMAND), "the remedy is named: {stderr}");
    assert!(stderr.contains(NOTHING_CHANGED), "{stderr}");
    for internal in ["XDG_RUNTIME_DIR", "socket", "D-Bus", "dbus"] {
        assert!(!stderr.contains(internal), "no internal term ({internal}) reaches the person: {stderr}");
    }
    assert_eq!(newest_subject(&fixture, &repo), "base", "no commit was made");
    assert_eq!(git_text(&fixture, &repo, &["diff", "--name-only"]).trim(), "a.txt", "the edit is still in the worktree");
    assert!(!repo.join(".git/index.lock").exists(), "no lock is left behind");
    let leftovers: Vec<String> = std::fs::read_dir(repo.join(".git"))
        .expect("git directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.starts_with("cli-git"); })
        .collect();
    assert!(leftovers.is_empty(), "no scratch directory or record is left: {leftovers:?}");
    remove_settled(&fixture);
}

/// A commit run outside any repository ends with Git's own words and status for it, as native `git commit` does.
#[test]
fn a_commit_outside_a_repository_says_what_native_git_says() {
    let fixture: Fixture = fixture("no-manager-outside");
    let plain: PathBuf = fixture.root.join("plain");
    std::fs::create_dir_all(&plain).expect("a directory that belongs to no repository");
    let wrapped: Observed = observe(&mut wrapped_commit(&fixture, &plain, &["commit", "--message=x"]), b"");
    let native: Observed = run_direct(&fixture, &plain, &["commit", "--message=x"]);
    assert_eq!(native.code, Some(128), "{}", stderr_of(&native));
    assert_eq!(wrapped.code, native.code, "{}", stderr_of(&wrapped));
    assert_eq!(stderr_of(&wrapped), stderr_of(&native), "the same words as native Git");
    assert!(wrapped.stdout.is_empty());
    remove_settled(&fixture);
}
