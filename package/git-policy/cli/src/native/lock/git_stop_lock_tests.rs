//! Controls for the lock files a stopped Git child must not leave behind.
//! Why:
//!  A Git child stopped at its deadline while it holds a ref lock leaves `<ref>.lock` behind when it cannot clean up
//!  after itself (it was killed, or never ran its signal handler), and every later Git command that updates the same ref
//!  then fails with "Unable to create ... File exists".
//!  Each control holds a real lock with real Git in a disposable repository, stops the holder at its deadline, and
//!  then runs the Git command that failed in practice (`git fetch`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runBoundedPush(holderOfRefLock, { deadlineMs: 1500 });
//! expect(lockFilesUnder(gitDir)).toEqual([]); await git(['fetch', 'origin', 'main']);
//! ```
#![cfg(unix)]

use crate::bounded_push::{run_bounded_push, Patience, STOPPED_EXIT_CODE};
use crate::child_watch::{output_watched, WatchTerms};
use crate::test_support::{fixture, git, git_text, remove, repository, scratch_environment, while_test_lives};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The deadline of the held lock's command: long enough for Git to start and take the lock, short enough to wait for.
const HOLD_DEADLINE: Duration = Duration::from_millis(2500);

/// The grace after the polite stop request: the unresponsive holder never answers it, and the responsive one needs the
///  host to run its handler, which a loaded gate does not do within a few hundred milliseconds.
const SHORT_GRACE: Duration = Duration::from_secs(2);

/// The tracking ref the incident left locked.
const HELD_REF: &str = "refs/remotes/origin/main";

/// Whether the holder can act on `SIGTERM`.
#[derive(Clone, Copy, Debug)]
enum Holder {
    /// Git runs its own signal handler and removes its lock files.
    Responsive,
    /// `SIGTERM` stays blocked in every process of the command, as for a Git that sits in an uninterruptible call.
    Unresponsive,
}

/// A repository whose `origin/main` is behind a bare remote's `main`, so a later fetch has a tracking ref to update.
struct Stage {
    /// The fixture directory, removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The ready marker the holder writes once the lock exists.
    ready: PathBuf,
}

/// Build the stage.
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main"]);
    let behind: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message", "second"]);
    git(repo.as_path(), &["push", "--quiet", "origin", "main"]);
    git(repo.as_path(), &["update-ref", HELD_REF, behind.as_str()]);
    let ready: PathBuf = root.join("ready");
    return Stage { root, repo, ready };
}

/// The command that holds the lock of `HELD_REF`: `git update-ref --stdin` told to prepare, then kept waiting.
fn holder(stage: &Stage, kind: Holder) -> Command {
    let lock: PathBuf = stage.repo.join(".git").join(format!("{HELD_REF}.lock"));
    let target: String = git_text(stage.repo.as_path(), &["rev-parse", "HEAD"]);
    // The wait for the lock ends with the test; the hold after it is one `sleep`, because the stop under test walks the holder's
    //  processes once, and a loop would start new ones after the walk. The fixture ends the sleeper when its test is over.
    let lives: String = while_test_lives(stage.root.as_path());
    let script: String = format!(
        "( printf 'start\\nupdate %s %s\\nprepare\\n' \"$HELD_REF\" \"$TARGET\"; \
         while [ ! -e \"$LOCK\" ] && {lives}; do sleep 0.05; done; : > \"$READY\"; sleep 600 ) \
         | /usr/bin/git update-ref --stdin"
    );
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg(script)
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.lockfilePid")
        .env("GIT_CONFIG_VALUE_0", "true")
        .env("HELD_REF", HELD_REF)
        .env("TARGET", target)
        .env("LOCK", lock.as_os_str())
        .env("READY", stage.ready.as_os_str());
    if matches!(kind, Holder::Unresponsive) {
        // SAFETY: only async-signal-safe calls (`sigemptyset`, `sigaddset`, `sigprocmask`) run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                let mut set: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut set);
                libc::sigaddset(&mut set, libc::SIGTERM);
                libc::sigprocmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
                return Ok(());
            });
        }
    }
    return command;
}

/// Every file under `directory` whose name ends in `.lock`.
fn lock_files(directory: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![directory.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(next.as_path()).expect("read the directory").flatten() {
            let path: PathBuf = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                pending.push(path);
            } else if path.to_string_lossy().ends_with(".lock") {
                found.push(path);
            }
        }
    }
    found.sort();
    return found;
}

/// How long a test lets the thread that sweeps for a tree the host was slow to end take, instead of looking once.
const SWEEP_WAIT: Duration = Duration::from_secs(20);

/// Each leftover file with its size, birth and whether a pid file sits beside it, so a failure on a loaded host says what
/// was left.
fn describe_left(files: &[PathBuf]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for file in files {
        let meta = std::fs::symlink_metadata(file);
        let size: String = meta.as_ref().map_or(String::from("gone"), |metadata| { return metadata.len().to_string(); });
        let born: String = meta.as_ref().ok().and_then(|metadata| { return metadata.created().ok(); }).map_or(String::from("unknown"), |time| { return format!("{time:?}"); });
        let name: String = file.to_string_lossy().into_owned();
        let sibling: String = format!("{}~pid.lock", name.trim_end_matches(".lock"));
        lines.push(format!("{name} size={size} born={born} pid_file_beside={}", Path::new(sibling.as_str()).exists()));
    }
    return lines.join("; ");
}

/// The checks that follow a stop: the holder really held the lock, none is left (the sweep for a tree the host was slow to
/// end runs on a thread and is waited for), and the fetch that failed in practice works.
fn assert_clean_after_stop(stage: &Stage, kind: Holder, how: &str) {
    assert!(stage.ready.exists(), "{how} {kind:?}: the holder took the lock before it was stopped");
    let waited: Instant = Instant::now();
    let mut left: Vec<PathBuf> = lock_files(stage.repo.join(".git").as_path());
    while !left.is_empty() && waited.elapsed() < SWEEP_WAIT {
        std::thread::sleep(Duration::from_millis(50));
        left = lock_files(stage.repo.join(".git").as_path());
    }
    assert!(left.is_empty(), "{how} {kind:?}: a stopped Git child left lock files: {}", describe_left(left.as_slice()));
    let fetched = std::process::Command::new("/usr/bin/git")
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["fetch", "--quiet", "origin", "main"])
        .output()
        .expect("start git fetch");
    assert!(fetched.status.success(), "{how} {kind:?}: the following fetch failed: {}", String::from_utf8_lossy(&fetched.stderr));
}

/// A push-style bounded run stops the holder at its deadline and leaves no lock.
fn bounded_run_leaves_no_lock(kind: Holder, name: &str) {
    let stage: Stage = stage(name);
    let attempt = run_bounded_push(holder(&stage, kind), Patience::Fixed(HOLD_DEADLINE), SHORT_GRACE, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped at its deadline");
    assert_clean_after_stop(&stage, kind, "bounded run");
    remove(stage.root.as_path());
}

/// A watched step stops the holder at its bound and leaves no lock.
fn watched_step_leaves_no_lock(kind: Holder, name: &str) {
    let stage: Stage = stage(name);
    let mut command: Command = holder(&stage, kind);
    let terms: WatchTerms = WatchTerms {
        limit: HOLD_DEADLINE,
        cadence: crate::wait_progress::Cadence { grace: HOLD_DEADLINE, interval: HOLD_DEADLINE },
        notices: Box::new(std::io::sink()),
    };
    let ended = output_watched(&mut command, "git update-ref --stdin", terms);
    assert_eq!(ended.expect_err("the step is stopped at its bound").kind(), std::io::ErrorKind::TimedOut);
    assert_clean_after_stop(&stage, kind, "watched step");
    remove(stage.root.as_path());
}

/// A push stopped on no progress leaves no lock either.
#[test]
fn a_stall_stop_of_an_unresponsive_holder_leaves_no_lock() {
    let stage: Stage = stage("stop-lock-push-stall");
    let patience: Patience = Patience::Progress { stall: HOLD_DEADLINE };
    let attempt = run_bounded_push(holder(&stage, Holder::Unresponsive), patience, SHORT_GRACE, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped on no progress");
    assert_clean_after_stop(&stage, Holder::Unresponsive, "stalled run");
    remove(stage.root.as_path());
}

/// A holder that answers `SIGTERM` is cleaned up by Git itself in the push path.
#[test]
fn a_push_stop_of_a_responsive_holder_leaves_no_lock() {
    bounded_run_leaves_no_lock(Holder::Responsive, "stop-lock-push-responsive");
}

/// A holder that cannot answer `SIGTERM` is killed, and the wrapper leaves no lock behind it.
#[test]
fn a_push_stop_of_an_unresponsive_holder_leaves_no_lock() {
    bounded_run_leaves_no_lock(Holder::Unresponsive, "stop-lock-push-unresponsive");
}

/// A holder that answers `SIGTERM` gets the chance in the watched path.
#[test]
fn a_watched_stop_of_a_responsive_holder_leaves_no_lock() {
    watched_step_leaves_no_lock(Holder::Responsive, "stop-lock-watched-responsive");
}

/// A holder that cannot answer `SIGTERM` leaves no lock in the watched path either.
#[test]
fn a_watched_stop_of_an_unresponsive_holder_leaves_no_lock() {
    watched_step_leaves_no_lock(Holder::Unresponsive, "stop-lock-watched-unresponsive");
}

/// The watched path asks with a catchable signal before it kills: a shell with a `TERM` trap runs the trap.
///  The shell sits silent in a bounded `sleep` it waits for (a loop that starts a process every pause would show work at every
///  look, and a step that works is never stopped), so the quiet window is what stops it.
#[test]
fn a_watched_stop_gives_a_catchable_signal_first() {
    let root: PathBuf = fixture("stop-lock-watched-catchable");
    let marker: PathBuf = root.join("trapped");
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg("trap ': > \"$MARKER\"; exit 0' TERM; sleep 30 & wait")
        .env("MARKER", marker.as_os_str());
    let terms: WatchTerms = WatchTerms {
        limit: Duration::from_millis(500),
        cadence: crate::wait_progress::Cadence { grace: Duration::from_millis(500), interval: Duration::from_millis(500) },
        notices: Box::new(std::io::sink()),
    };
    let ended = output_watched(&mut command, "a trapping shell", terms);
    assert_eq!(ended.expect_err("stopped for being quiet").kind(), std::io::ErrorKind::TimedOut);
    assert!(marker.exists(), "the stop reached the TERM trap before any kill");
    remove(root.as_path());
}

/// A stop with no grace kills at once and hands the child to a thread, which sweeps its locks when it has ended.
#[test]
fn a_stop_without_grace_sweeps_the_locks_on_a_thread_of_its_own() {
    let stage: Stage = stage("stop-lock-push-no-grace");
    let attempt = run_bounded_push(holder(&stage, Holder::Unresponsive), Patience::Fixed(HOLD_DEADLINE), Duration::ZERO, scratch_environment().as_slice()).expect("runs");
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "the holder is stopped at its deadline");
    assert_clean_after_stop(&stage, Holder::Unresponsive, "bounded run without grace");
    remove(stage.root.as_path());
}
