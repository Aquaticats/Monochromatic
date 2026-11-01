//! Binary-level experiments for the claim that the post-commit auto-push can hang on the Git LFS upload.
//! Each control builds the worst shape that the wrapper's own coordination allows and measures it: an upload that
//!  slows two worktrees' auto-pushes at once, and an upload whose helper detaches from the process tree and keeps the
//!  push's output file open. The verdicts are in `doc/planning/cli-git-no-self-invocation.md`, section "Hang analysis".

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, executable, fixture, git, observe, repository, stderr_of};
use git_policy_cli::bounded_push::{LOCK_WAIT_VARIABLE, PUSH_DEADLINE_VARIABLE, STOP_GRACE};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The delay of the slow upload, in whole seconds as the stand-in's `sleep` takes it.
const UPLOAD_DELAY_SECONDS: u64 = 2;

/// The push deadline the detaching control sets, in whole seconds.
const SHORT_DEADLINE_SECONDS: u64 = 2;

/// Extra time a control allows beyond a derived bound for process starts under host load.
const LOAD_ALLOWANCE: Duration = Duration::from_secs(60);

/// A repository with a bare origin, an LFS key, and a stand-in `git-lfs` made from `source`.
fn world(fixture: &Fixture, source: &str) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("UTF-8")]);
    git(fixture, repo.as_path(), &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
    git(fixture, repo.as_path(), &["push", "--quiet", "--no-verify", "--set-upstream", "origin", "main"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
    return (repo, origin);
}

/// Stage a file and commit it through the wrapper with extra environment, returning what the caller saw.
fn commit_file(fixture: &Fixture, directory: &Path, name: &str, environment: &[(&str, String)]) -> Observed {
    std::fs::write(directory.join(name), format!("{name}\n")).expect("file");
    git(fixture, directory, &["add", "--", name]);
    let mut command = wrapped_commit(fixture, directory, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
    for (key, value) in environment {
        command.env(key, value);
    }
    return observe(&mut command, b"");
}

/// Two worktrees of one repository commit at the same moment while every upload takes a while: both finish, neither
///  waits for the other (their branches have separate push locks and the upload runs under no lock another wrapper
///  process takes), and both branches reach the origin.
#[test]
fn two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other() {
    let fixture: Fixture = fixture("lfs-hang-two-worktrees");
    let (repo, origin) = world(&fixture, &format!("#!/bin/sh\ncat > /dev/null\nsleep {UPLOAD_DELAY_SECONDS}\nexit 0\n"));
    let second: PathBuf = fixture.root.join("second");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", second.to_str().expect("UTF-8")]);
    git(&fixture, second.as_path(), &["push", "--quiet", "--no-verify", "--set-upstream", "origin", "other"]);
    let started: Instant = Instant::now();
    let (first_seen, second_seen) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            return commit_file(&fixture, repo.as_path(), "first.txt", &[]);
        });
        let other = scope.spawn(|| {
            return commit_file(&fixture, second.as_path(), "second.txt", &[]);
        });
        return (first.join().expect("first commit"), other.join().expect("second commit"));
    });
    let elapsed: Duration = started.elapsed();
    for (label, seen) in [("first", &first_seen), ("second", &second_seen)] {
        assert_eq!(seen.code, Some(0), "{label}: {}", stderr_of(seen));
        assert!(!stderr_of(seen).contains("waiting for"), "{label} waited for the other: {}", stderr_of(seen));
    }
    // Serial pushes would take two delays; the bound is generous (a delay and the load allowance) and still
    //  separates "ran at once" from "waited in turn" only through the absence of a wait notice, asserted above.
    assert!(
        elapsed < Duration::from_secs(UPLOAD_DELAY_SECONDS) + LOAD_ALLOWANCE,
        "both commits finished within one upload plus the allowance: {elapsed:?}"
    );
    for branch in ["main", "other"] {
        let head: String = git_text(&fixture, origin.as_path(), &["rev-parse", &format!("refs/heads/{branch}")]);
        assert_eq!(head.trim().len(), 40, "{branch} reached the origin");
    }
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An upload helper that detaches from the whole process tree (a new session, parent gone) and keeps the push's
///  output file open cannot hold the commit: the push is stopped at its deadline and the wrapper returns within the
///  deadline plus the stop grace, because it reads a file to its current end and waits on no pipe.
///  The detached helper itself is a leak that no tree walk can find; the control ends it so the test leaves nothing.
#[test]
fn a_detached_upload_helper_holding_the_output_open_does_not_hold_the_commit() {
    let fixture: Fixture = fixture("lfs-hang-detached");
    let pid_file: PathBuf = fixture.root.join("detached.pid");
    let source: String = format!(
        "#!/bin/sh\ncat > /dev/null\n( setsid sh -c 'echo $$ > \"{}\"; exec sleep 600' & ) \nexec sleep 600\n",
        pid_file.display()
    );
    let (repo, _origin) = world(&fixture, source.as_str());
    let started: Instant = Instant::now();
    let seen: Observed = commit_file(
        &fixture,
        repo.as_path(),
        "a.txt",
        &[(PUSH_DEADLINE_VARIABLE, SHORT_DEADLINE_SECONDS.to_string()), (LOCK_WAIT_VARIABLE, String::from("20"))],
    );
    let elapsed: Duration = started.elapsed();
    let stderr: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "the commit itself succeeds: {stderr}");
    assert!(stderr.contains(&format!("did not finish within {SHORT_DEADLINE_SECONDS} seconds and was stopped")), "{stderr}");
    assert!(
        elapsed < Duration::from_secs(SHORT_DEADLINE_SECONDS) + STOP_GRACE + LOAD_ALLOWANCE,
        "the wrapper returned at the deadline plus the grace, not when the detached helper ended: {elapsed:?}"
    );
    // Finish the experiment: the helper is outside the tree, so it is still running; end it by its recorded number.
    let recorded: String = std::fs::read_to_string(&pid_file).unwrap_or_default();
    if let Ok(pid) = recorded.trim().parse::<i32>() {
        // SAFETY: `kill` takes no pointers; the number was written by the helper this control started.
        unsafe { libc::kill(pid, libc::SIGKILL) };
    }
    remove_settled(&fixture);
}
