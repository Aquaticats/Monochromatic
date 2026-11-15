//! What:
//!  The built wrapper on a machine where no commit service can be reached or started,
//! and in a directory that belongs to no
//!  repository.
//! Why:
//!  The service is the only writer of a commit (`doc/planning/cli-git-resident-service.md`,
//! "Provisioning").
//! With no runtime
//!  directory,
//! no socket and no session bus the commit stops with the plain message of its platform,
//! names the remedy,
//!  and changes nothing;
//! the wrapper never lands a commit by another way,
//! and never starts the service itself.
//! Outside a
//!  repository there is no worktree to send a commit for,
//! and the person is told what native Git tells them there.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, fixture, git, observe, repository, run_direct, stderr_of};
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

/// What:
///  A commit run with no runtime directory and no session bus is still made:
///  this account's own folder below its home serves the service, the way a container or a cron job has it,
///  and nothing of the commit is left behind in the Git directory but the commit itself.
/// Why:
///  This branch needs no manager to start its service, so the absence of a bus is no reason for a commit to fail
///  (the review of `896c60d97`, and `runtime_directory` in `service_host_of_a_worktree.rs`). The old subject of this
///  control — a commit that stops with a platform message — is gone from this branch.
#[test]
fn a_commit_with_no_service_manager_stops_plainly_and_changes_nothing() {
    let (fixture, repo) = edited("no-manager-plain");
    let mut command = wrapped_commit(&fixture, &repo, &["commit", "--message=second", "--", "a.txt"]);
    command.env_remove("XDG_RUNTIME_DIR").env_remove("DBUS_SESSION_BUS_ADDRESS");
    let output: Output = command.output().expect("the wrapped commit runs");
    let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(0), "the commit is made with no manager and no runtime directory named: {stderr}");
    assert_eq!(newest_subject(&fixture, &repo), "second", "the commit is on the branch");
    assert!(!repo.join(".git/index.lock").exists(), "no lock is left behind");
    remove_settled(&fixture);
}

/// A commit run outside any repository ends with Git's own words and status for it,
/// as native `git commit` does.
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
