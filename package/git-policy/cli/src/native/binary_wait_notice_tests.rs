//! Binary-level controls for the notices of a commit that waits for Git's own `index.lock` (issue 1811):
//!  the wait names what it waits for and who holds it,
//!  repeats at a steady interval, and ends at its bound with a diagnostic that names the holder.
//! The holder is this test process, holding the lock file open.
//! A control of a repeated notice reads the wrapper's standard error line by line and ends the wait on the notice that
//!  counts, so the count holds however slowly a loaded host runs the wrapper; a control of a bound only asks for a
//!  notice before the refusal, which the wait guarantees.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::commit::{remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, fixture, git, observe_raw, repository, stderr_of,
};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The repeat interval the controls ask for, in seconds: the smallest the variable accepts.
const NOTICE_SECONDS: &str = "1";

/// The lock wait limit of a control that ends its wait on a notice: far beyond what the control needs.
const GENEROUS_LIMIT_SECONDS: &str = "120";

/// What a watched commit showed its caller.
struct Watched {
    /// Exit code, absent when a signal ended it.
    code: Option<i32>,
    /// Every line of standard error.
    stderr: String,
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

/// A waiting commit's command: the notice interval of the controls and the given lock wait limit.
fn waiting_commit(fixture: &Fixture, repo: &Path, limit_seconds: &str) -> Command {
    // The wait for the index lock is the service's, so its cadence and limit are given to the service, before it starts.
    super::support::give_service_environment(fixture, "CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS);
    super::support::give_service_environment(fixture, "CLI_GIT_LOCK_WAIT_SECONDS", limit_seconds);
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--message=waiter", "a.txt"]);
    command
        .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS)
        .env("CLI_GIT_LOCK_WAIT_SECONDS", limit_seconds);
    return command;
}

/// The lines of standard error that contain `needle`.
fn lines_with<'text>(stderr: &'text str, needle: &str) -> Vec<&'text str> {
    return stderr.lines().filter(|line| { return line.contains(needle); }).collect();
}

/// What:
///  Run `command` with standard error read line by line, and run `on_count` once when `count` lines containing `needle`
///  have been read.
/// Why:
///   The action ends the wait the notices describe (it releases the lock the commit waits for),
///  so the wait stops on the counted notice and not on a timer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of child.stderr.lines()) { if (line.includes(needle) && ++seen === count) onCount(); }
/// ```
fn watch_notices(command: &mut Command, needle: &str, count: usize, on_count: impl FnOnce()) -> Watched {
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

/// A commit that waits for a live `index.lock` holder repeats the line that names it, with the elapsed time and the
///  limit; the second notice makes the holder let go, and the commit then lands.
#[test]
fn a_commit_waiting_for_a_live_index_lock_holder_repeats_its_notice() {
    let fixture: Fixture = fixture("wait-notice-index-live");
    let repo: PathBuf = edited_repository(&fixture);
    let lock_path: PathBuf = repo.join(".git/index.lock");
    // This process holds the lock file open, which is what proves it alive to the wrapper.
    let lock: File = File::create(&lock_path).expect("index lock");
    let needle: String = format!("waiting for PID {}", std::process::id());
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        needle.as_str(),
        2,
        move || {
            drop(lock);
            std::fs::remove_file(&lock_path).expect("the holder lets go");
        },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle.as_str());
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains("giving up after 120 seconds"); }), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}

/// A commit that waits for a live `index.lock` holder past its limit was announced (the first line and a repeat)
///  and stops with the holder, the limit and the way forward named.
#[test]
fn a_commit_that_gives_up_on_a_live_index_lock_holder_names_it() {
    let fixture: Fixture = fixture("wait-notice-index-live-bound");
    let repo: PathBuf = edited_repository(&fixture);
    let lock: File = File::create(repo.join(".git/index.lock")).expect("index lock");
    let observed: Observed = observe_raw(&mut waiting_commit(&fixture, repo.as_path(), "3"), b"");
    let stderr: String = stderr_of(&observed);
    let notices: Vec<&str> = lines_with(stderr.as_str(), format!("waiting for PID {}", std::process::id()).as_str());
    assert!(notices.len() >= 2, "a first notice and a repeat: {stderr}");
    assert_ne!(observed.code, Some(0), "{stderr}");
    assert!(stderr.contains("is using this repository's index") && stderr.contains("was not made"), "the holder is named and nothing changed: {stderr}");
    assert!(stderr.contains(&format!("Process {}", std::process::id())), "{stderr}");
    drop(lock);
    remove_settled(&fixture);
}

/// A commit that waits for an `index.lock` nobody can be proven to hold says so while it retries;
///  the second notice removes the lock, and the commit then lands.
#[test]
fn a_commit_waiting_for_an_unproven_index_lock_says_so_while_it_retries() {
    let fixture: Fixture = fixture("wait-notice-index-unproven");
    let repo: PathBuf = edited_repository(&fixture);
    std::fs::write(repo.join("cli-git.config.jsonc"), "{ \"indexLock\": { \"unprovenOwnerTimeoutMs\": 120000 } }\n")
        .expect("configuration");
    let lock_path: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock_path, b"").expect("index lock nobody holds");
    // The service's wait names its limit.
    let (needle, limit_words): (&str, &str) = ("cannot be proven alive", "giving up after 120 seconds");
    let watched: Watched = watch_notices(
        &mut waiting_commit(&fixture, repo.as_path(), GENEROUS_LIMIT_SECONDS),
        needle,
        2,
        move || { std::fs::remove_file(&lock_path).expect("the stale lock is removed"); },
    );
    let notices: Vec<&str> = lines_with(watched.stderr.as_str(), needle);
    assert!(notices.len() >= 2, "a first notice and a repeat: {}", watched.stderr);
    assert!(notices.iter().all(|line| { return line.contains(limit_words); }), "{}", watched.stderr);
    assert_eq!(watched.code, Some(0), "{}", watched.stderr);
    remove_settled(&fixture);
}
