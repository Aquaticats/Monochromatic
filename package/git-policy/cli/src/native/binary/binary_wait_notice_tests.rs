//! Binary-level controls for the notices of a commit that waits for Git's own `index.lock` (issue 1811):
//!  the wait names what it waits for and who holds it,
//!  repeats at a steady interval,
//! and waits for a holder proven alive,
//! and for an owner nothing proves alive,
//! until the ceiling,
//!  saying what is known about the second.
//! The holder is this test process,
//! holding the lock file open.
//! A control of a repeated notice reads the wrapper's standard error line by line and ends the wait on the notice that
//!  counts,
//! so the count holds however slowly a loaded host runs the wrapper;
//! a control of the backoff only asks for a
//!  notice before the refusal,
//! which the wait guarantees.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::commit::{remove_settled, wrapped_commit};
use super::support::{
    Fixture, fixture, git, repository,
};
use git_policy_cli::wait_ceilings::{LANDING_LOCK_CEILING, WAITING_NOTICE_VARIABLE};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The repeat interval the controls ask for,
/// in seconds:
/// the smallest the variable accepts.
const NOTICE_SECONDS: &str = "1";
/// The words a commit the service named no place for is told,
/// as the person reads them.
///
/// A control asserts on the part of the line that states no seconds,
/// because the line may name them.
const UNPLACED_SAID: &str = "has not said where this commit stands";


/// What a watched commit showed its caller.
pub(super) struct Watched {
    /// Exit code,
    /// absent when a signal ended it.
    pub(super) code: Option<i32>,
    /// Every line of standard error.
    pub(super) stderr: String,
}

/// A repository with one tracked file whose worktree copy holds an uncommitted edit.
fn edited_repository(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    return repo;
}

/// A waiting commit's command:
/// the notice interval of the controls.
fn waiting_commit(fixture: &Fixture, repo: &Path) -> Command {
    // The wait for the index lock is the service's, so its cadence is given to the service, before it starts.
    super::support::give_service_environment(fixture, "CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS);
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--message=waiter", "a.txt"]);
    command.env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS);
    return command;
}

/// The lines of standard error that contain `needle`.
fn lines_with<'text>(stderr: &'text str, needle: &str) -> Vec<&'text str> {
    return stderr.lines().filter(|line| { return line.contains(needle); }).collect();
}

/// What:
///  Run `command` with standard error read line by line,
/// and run `on_count` once when `count` lines containing `needle`
///  have been read.
/// Why:
///   The action ends the wait the notices describe (it releases the lock the commit waits for),
///  so the wait stops on the counted notice and not on a timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of child.stderr.lines()) { if (line.includes(needle) && ++seen === count) onCount(); }
/// ```
pub(super) fn watch_notices(command: &mut Command, needle: &str, count: usize, on_count: impl FnOnce()) -> Watched {
    let mut child: Child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the commit starts");
    let reader: BufReader<std::process::ChildStderr> = BufReader::new(child.stderr.take().expect("piped"));
    let mut stderr: String = String::new();
    let mut seen: usize = 0;
    let mut action: Option<_> = Some(on_count);
    for read in reader.lines() {
        let Ok(line) = read else {
            break;
        };
        stderr.push_str(line.as_str());
        stderr.push('\n');
        if line.contains(needle) {
            seen += 1;
            if seen >= count && let Some(act) = action.take() {
                act();
            }
        }
    }
    let status = child.wait().expect("the commit ends");
    return Watched { code: status.code(), stderr };
}

/// The wait the standing notice is set to in this control,
/// in seconds:
/// the smallest the variable accepts.
const STANDING_SECONDS: &str = "1";

/// What:
///  A commit whose landing waits behind another process's `index.lock` is told only what the client knows:
/// that the
///  service has not said where it stands,
/// and not that the service has not started on it.
/// Why:
///   The landing has started by then:
/// the line that names the holder is the landing's own,
/// so words that say the service has not
///  begun on the commit are false.
/// What the client knows is that no answer has come and that the service named no place.
#[test]
fn a_commit_that_waits_is_told_the_service_has_not_said_where_it_stands() {
    let fixture: Fixture = fixture("wait-notice-standing");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    // This process holds the lock file open, so the service's landing waits behind it and the answer waits with it.
    let lock: File = File::create(&lock_path).expect("index lock");
    let needle: String = format!("waiting for PID {}", std::process::id());
    let mut command: Command = waiting_commit(&fixture, repo.as_path());
    command.env(WAITING_NOTICE_VARIABLE, STANDING_SECONDS);
    let watched: Watched = watch_notices(&mut command, needle.as_str(), 2, move || {
        drop(lock);
        std::fs::remove_file(&lock_path).expect("the holder lets go");
    });
    assert!(watched.stderr.contains(UNPLACED_SAID), "the person is told the service has not said where the commit stands: {}", watched.stderr);
    assert!(!watched.stderr.contains("has not started on this commit"), "the landing had started, so the words may not say it had not: {}", watched.stderr);
    // The landing had started: this line is the landing's own, and it names the holder of the lock.
    assert!(watched.stderr.contains(needle.as_str()), "the wait names the holder of the index: {}", watched.stderr);
    assert_eq!(watched.code, Some(0), "the commit landed once the holder let go: {}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder repeats the line that names it,
/// with the elapsed time and without any
///  time at which it gives up;
/// the second notice makes the holder let go,
/// and the commit then lands.
#[test]
fn a_commit_waiting_for_a_live_index_lock_holder_repeats_its_notice() {
    let fixture: Fixture = fixture("wait-notice-index-live");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    // This process holds the lock file open, which is what proves it alive to the wrapper.
    let lock: File = File::create(&lock_path).expect("index lock");
    let needle: String = format!("waiting for PID {}", std::process::id());
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path()),
        needle.as_str(),
        2,
        move || {
            drop(lock);
            std::fs::remove_file(&lock_path).expect("the holder lets go");
        },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle.as_str());
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains(format!("for at most {} seconds", LANDING_LOCK_CEILING.as_secs()).as_str()); }), "every notice says how long the wait lasts: {}", watched.stderr);
    assert!(!watched.stderr.contains("giving up"), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder is still waiting after more notices than any earlier limit of a few
///  seconds allowed:
/// the holder is announced again and again,
/// nothing gives up,
/// and the commit lands when the holder lets go.
#[test]
fn a_commit_waits_on_a_live_index_lock_holder_until_it_lets_go() {
    let fixture: Fixture = fixture("wait-notice-index-live-no-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    let lock: File = File::create(&lock_path).expect("index lock");
    let needle: String = format!("waiting for PID {}", std::process::id());
    let watched: Watched = watch_notices(&mut waiting_commit(&fixture, repo.as_path()), needle.as_str(), LONG_WAIT_NOTICES, move || {
        drop(lock);
        std::fs::remove_file(&lock_path).expect("the holder lets go");
    });
    assert!(lines_with(watched.stderr.as_str(), needle.as_str()).len() >= LONG_WAIT_NOTICES, "{}", watched.stderr);
    assert!(!watched.stderr.contains("giving up") && !watched.stderr.contains("is using this repository's index"), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "the commit landed once the holder let go: {}", watched.stderr);
    remove_settled(&fixture);
}

/// The notices a control waits for before it lets the holder go:
/// more seconds of waiting,
/// at one notice a second,
/// than the
///  second an owner nothing proves alive used to be given,
/// so a wait that lasts past that second is what the control reads.
const LONG_WAIT_NOTICES: usize = 4;

/// A commit that waits for an `index.lock` nobody can be proven to hold says so,
/// and a lock that goes during the backoff lets
///  the commit land.
#[test]
fn a_commit_waiting_for_an_unproven_index_lock_lands_when_it_goes() {
    let fixture: Fixture = fixture("wait-notice-index-unproven");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock_path, b"a lock with content\n").expect("index lock nobody holds");
    let needle: &str = "cannot be proven alive";
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path()),
        needle,
        1,
        move || { std::fs::remove_file(&lock_path).expect("the stale lock is removed"); },
    );
    assert!(!lines_with(watched.stderr.as_str(), needle).is_empty(), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for an `index.lock` nobody can be proven to hold waits for it like one proven alive:
/// through more notices
///  than the second it used to be given,
/// each saying what is known and naming the file to remove if no program is using the
///  repository,
/// and it lands when the lock goes.
/// The person's own `git commit` with its editor open holds the lock the same way.
#[test]
fn a_commit_waits_on_an_unproven_index_lock_and_says_what_it_knows() {
    let fixture: Fixture = fixture("wait-notice-index-unproven-no-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock_path, b"a lock with content\n").expect("index lock nobody holds");
    let needle: &str = "cannot be proven alive";
    let watched: Watched = watch_notices(&mut waiting_commit(&fixture, repo.as_path()), needle, LONG_WAIT_NOTICES, {
        let released: PathBuf = lock_path.clone();
        move || { std::fs::remove_file(&released).expect("the lock goes"); }
    });
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle);
    assert!(notices.len() >= LONG_WAIT_NOTICES, "{}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("no Git lock PID file") && line.contains("has the lock open"); }), "what is known is said: {}", watched.stderr);
    assert!(watched.stderr.contains(format!("removing the file {}", lock_path.display()).as_str()), "the way out is named: {}", watched.stderr);
    assert!(!watched.stderr.contains("was not made") && !watched.stderr.contains("stops waiting"), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "the commit landed once the lock went: {}", watched.stderr);
    remove_settled(&fixture);
}
