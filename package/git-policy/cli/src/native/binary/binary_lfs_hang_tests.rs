//! Binary-level experiments for the claim that the post-commit auto-push can hang on the Git LFS upload.
//! Each control builds the worst shape that the wrapper's own coordination allows and measures it:
//! an upload that
//!  slows two worktrees' auto-pushes at once,
//! and an upload whose helper detaches from the process tree and keeps the
//!  push's output file open.
//! The verdicts are in `doc/planning/cli-git-no-self-invocation.md`,
//! section "Hang analysis".

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, decline_in_process_lfs_scan, executable, fixture, git, observe, repository, stderr_of};
use git_policy_cli::bounded_push::{PUSH_DEADLINE_VARIABLE, STOP_GRACE};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The push deadline the detaching control sets,
/// in whole seconds.
const SHORT_DEADLINE_SECONDS: u64 = 2;

/// How long the stand-in and its detached helper would run if nothing stopped them,
/// in whole seconds;
/// far past every
///  bound the controls assert,
/// and the control ends the helper itself.
const HOLD_SECONDS: u64 = 3_000;

/// Extra time a control allows beyond a derived bound for process starts under host load.
const LOAD_ALLOWANCE: Duration = Duration::from_secs(60);

/// How long the second worktree's upload takes before it announces itself,
/// in whole seconds.
/// The two worktrees of the control commit at once,
/// so the wait after the first commit cannot stand in for the second push:
/// a control that waits for the wrong worktree reads the origin while this upload is still running,
/// every run,
/// and its assertion fails (issue 6336).
const SECOND_WORKTREE_UPLOAD_SECONDS: u64 = 1;

/// A repository with a bare origin,
/// an LFS key,
/// and a stand-in `git-lfs` made from `source`.
fn world(fixture: &Fixture, source: &str) -> (PathBuf, PathBuf) {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    let origin: PathBuf = fixture.root.join("origin.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("UTF-8")]);
    git(fixture, repo.as_path(), &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
    git(fixture, repo.as_path(), &["push", "--quiet", "--no-verify", "--set-upstream", "origin", "main"]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    decline_in_process_lfs_scan(fixture, repo.as_path());
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
    return (repo, origin);
}

/// Stage a file and commit it through the wrapper with extra environment,
/// returning what the caller saw.
fn commit_file(fixture: &Fixture, directory: &Path, name: &str, environment: &[(&str, String)]) -> Observed {
    std::fs::write(directory.join(name), format!("{name}\n")).expect("file");
    git(fixture, directory, &["add", "--", name]);
    // The push runs in the service's environment, so what the control sets for it is given there, before the service starts.
    for (key, value) in environment {
        super::support::give_service_environment(fixture, key, value);
    }
    let mut command = wrapped_commit(fixture, directory, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
    for (key, value) in environment {
        command.env(key, value);
    }
    return observe(&mut command, b"");
}

/// Two worktrees of one repository commit at the same moment while every upload takes a while,
/// the second worktree's upload the longer:
/// both finish,
/// neither
///  waits for the other (the upload runs under no lock another wrapper process takes),
/// and both branches reach the origin.
///  The second upload is slowed so that the wait after the first worktree's commit cannot be mistaken for the second's:
/// the control fails every run on a helper that waits for no worktree whose `.git` is a file.
#[test]
fn two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other() {
    let fixture: Fixture = fixture("lfs-hang-two-worktrees");
    // Each upload announces itself and then waits for the other to announce itself; an upload that never sees the
    //  other start (uploads taken in turn) leaves a mark and fails the control.
    let marks: PathBuf = fixture.root.join("marks");
    std::fs::create_dir(&marks).expect("marks directory");
    let barrier: String = format!(
        "#!/bin/sh\ncase \"$PWD\" in */second) sleep {slow} ;; esac\ntouch '{dir}/started.'$$\ni=0\nwhile [ \"$(ls '{dir}' | grep -c '^started\\.')\" -lt 2 ] && [ $i -lt {tries} ]; do sleep 0.1; i=$((i+1)); done\n\
         if [ \"$(ls '{dir}' | grep -c '^started\\.')\" -lt 2 ]; then touch '{dir}/alone'; fi\nexit 0\n",
        dir = marks.display(),
        // The service pushes the branches of one repository one after another, so its uploads cannot wait for each other.
        tries = 1,
        slow = SECOND_WORKTREE_UPLOAD_SECONDS
    );
    let (repo, origin) = world(&fixture, barrier.as_str());
    let second: PathBuf = fixture.root.join("second");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "other", second.to_str().expect("UTF-8")]);
    git(&fixture, second.as_path(), &["push", "--quiet", "--no-verify", "--set-upstream", "origin", "other"]);
    let (first_seen, second_seen, second_waited) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            return commit_file(&fixture, repo.as_path(), "first.txt", &[]);
        });
        let other = scope.spawn(|| {
            let began: Instant = Instant::now();
            let seen: Observed = commit_file(&fixture, second.as_path(), "second.txt", &[]);
            return (seen, began.elapsed());
        });
        let (other_seen, waited) = other.join().expect("second commit");
        return (first.join().expect("first commit"), other_seen, waited);
    });
    for (label, seen) in [("first", &first_seen), ("second", &second_seen)] {
        assert_eq!(seen.code, Some(0), "{label}: {}", stderr_of(seen));
        assert!(!stderr_of(seen).contains("waiting for"), "{label} waited for the other: {}", stderr_of(seen));
    }
    for (branch, worktree) in [("main", repo.as_path()), ("other", second.as_path())] {
        let published: String = git_text(&fixture, origin.as_path(), &["rev-parse", &format!("refs/heads/{branch}")]);
        let landed: String = git_text(&fixture, worktree, &["rev-parse", "HEAD"]);
        assert_eq!(published.trim(), landed.trim(), "{branch}: the new commit, not the earlier one, reached the origin");
    }
    // The forcing itself is controlled: the wait after the second worktree's commit outlasted its slowed upload, so the
    //  wait above covered that push and not only the commit.
    assert!(
        second_waited >= Duration::from_secs(SECOND_WORKTREE_UPLOAD_SECONDS),
        "the wait after the second worktree's commit covered its slowed upload: {second_waited:?}"
    );
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An upload helper that detaches from the whole process tree (a new session,
/// parent gone) and keeps the push's
///  output file open cannot hold the commit:
/// the push is stopped at its deadline and the wrapper returns within the
///  deadline plus the stop grace,
/// because it reads a file to its current end and waits on no pipe.
///  The detached helper itself is a leak that no tree walk can find;
/// the control ends it so the test leaves nothing.
#[test]
fn a_detached_upload_helper_holding_the_output_open_does_not_hold_the_commit() {
    let fixture: Fixture = fixture("lfs-hang-detached");
    let pid_file: PathBuf = fixture.root.join("detached.pid");
    let source: String = format!(
        "#!/bin/sh\ncat > /dev/null\n( setsid sh -c 'echo $$ > \"{}\"; exec sleep {HOLD_SECONDS}' & ) \nexec sleep {HOLD_SECONDS}\n",
        pid_file.display()
    );
    let (repo, _origin) = world(&fixture, source.as_str());
    let started: Instant = Instant::now();
    let seen: Observed = commit_file(
        &fixture,
        repo.as_path(),
        "a.txt",
        &[(PUSH_DEADLINE_VARIABLE, SHORT_DEADLINE_SECONDS.to_string())],
    );
    let elapsed: Duration = started.elapsed();
    let stderr: String = stderr_of(&seen);
    assert_eq!(seen.code, Some(0), "the commit itself succeeds: {stderr}");
    // With the service the push is stopped after the command returned, and the stop is the notice of the next commit.
    let report: String = super::support::push_report(repo.as_path(), &stderr);
    assert!(report.contains(&format!("made no progress for {SHORT_DEADLINE_SECONDS} seconds and was stopped")), "{report}");
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
