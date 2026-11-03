//! Controls for stopping a push on no progress, with real Git pushing to a local bare remote whose `pre-receive` hook
//! shapes the progress: it sleeps (no data moves), it prints a line every moment for a while (data keeps moving), or it
//! prints for ever (a trickle that never stalls). No load is generated: the hooks sleep between lines.
//! Why:
//!  A fixed 30 second deadline stopped slow pushes that were still moving data and let a stuck push take its whole
//!  deadline. The stop is now on a stall (no data moved by the push or anything it started), under a ceiling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect((await runBoundedPush(pushToHookThatTicks, { stallMs: 1500, ceilingMs: 60000 })).exitCode).toBe(0);
//! ```
#![cfg(unix)]

use crate::auto_push_single_flight::PushAttempt;
use crate::bounded_push::{
    DEFAULT_PUSH_CEILING, DEFAULT_PUSH_DEADLINE, PUSH_CEILING_VARIABLE, PUSH_DEADLINE_VARIABLE, Patience, STOPPED_EXIT_CODE, push_ceiling,
    push_patience, run_bounded_push,
};
use crate::push_progress::tests::disk_wait_command;
use crate::test_support::{fixture, git, remove, repository, scratch_environment, while_test_lives};
use std::io::Read;
use std::net::TcpListener;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// The stall window of the controls: several sampling intervals, short enough to wait for.
const STALL: Duration = Duration::from_millis(2000);

/// The stall window of the control that keeps moving: a dozen ticks, so a loaded host that delays a few lines does not make
///  the push stall, and both phases below run longer than this window.
const PROGRESS_STALL: Duration = Duration::from_millis(3000);

/// How long the `post-receive` hook of the control that keeps moving waits for its disk, longer than the stall window.
const POST_PHASE_HOLD: Duration = Duration::from_millis(4500);

/// How long a loaded host may take over a stop beyond its limit and the grace.
const STOP_SLACK: Duration = Duration::from_secs(30);

/// The ceiling of the controls: past the stall window, short enough to wait for.
const CEILING: Duration = Duration::from_millis(5000);

/// The grace after the polite stop request.
const GRACE: Duration = Duration::from_secs(2);

/// The pause between two lines of a hook that keeps moving data, much shorter than the stall window.
const TICK: &str = "0.25";

/// What the remote's `pre-receive` hook does.
#[derive(Clone, Copy, Debug)]
enum Hook {
    /// Sleeps: no data moves.
    Silent,
    /// Prints a line every tick for a while longer than the stall window, then lets the push through.
    ProgressThenAccept,
    /// Prints a line every tick for ever.
    TrickleForever,
}

impl Hook {
    /// The hook's script.
    fn script(self, root: &Path) -> String {
        let lives: String = while_test_lives(root);
        return match self {
            // One process that starts none: a loop would start a `sleep` each turn, and the meter counts that as work.
            Hook::Silent => String::from("#!/bin/sh\nexec sleep 600\n"),
            Hook::ProgressThenAccept => format!("#!/bin/sh\ni=0\nwhile [ $i -lt 20 ]; do echo tick >&2; sleep {TICK}; i=$((i+1)); done\nexit 0\n"),
            Hook::TrickleForever => format!("#!/bin/sh\nwhile {lives}; do echo tick >&2; sleep {TICK}; done\n"),
        };
    }
}

/// A repository and a bare remote with the hook, and the push command for them.
struct Stage {
    /// The fixture directory, removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// Variables the push gets besides the wrapper's: the settings of the `post-receive` probe, if the stage has one.
    extra_environment: Vec<(String, String)>,
}

/// Build the stage.
fn stage(name: &str, hook: Hook) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    let script: PathBuf = remote.join("hooks").join("pre-receive");
    std::fs::write(script.as_path(), hook.script(root.as_path())).expect("hook");
    std::fs::set_permissions(script.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    return Stage { root, repo, extra_environment: Vec::new() };
}

/// Make the remote's `post-receive` hook hold a thread in disk wait for `hold` after the push was accepted: the phase after
///  the hook is shaped exactly, where it would otherwise be whatever the disk under the test makes of the ref update.
fn hold_disk_wait_after_hook(stage: &mut Stage, hold: Duration) {
    let probe: Command = disk_wait_command(hold, "exit");
    let line: String = std::iter::once(probe.get_program().to_string_lossy().into_owned())
        .chain(probe.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }))
        .map(|word| { return format!("'{word}'"); })
        .collect::<Vec<String>>()
        .join(" ");
    let script: PathBuf = stage.root.join("remote.git").join("hooks").join("post-receive");
    std::fs::write(script.as_path(), format!("#!/bin/sh\nexec {line}\n")).expect("hook");
    std::fs::set_permissions(script.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    for (name, setting) in probe.get_envs() {
        if let Some(value) = setting {
            stage.extra_environment.push((name.to_string_lossy().into_owned(), value.to_string_lossy().into_owned()));
        }
    }
}

/// The push of the stage, with the environment the wrapper gives its pushes.
fn push_command(stage: &Stage) -> Command {
    let mut command: Command = Command::new("/usr/bin/git");
    command
        .args(["push", "--quiet", "origin", "main"])
        .current_dir(stage.repo.as_path())
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.lockfilePid")
        .env("GIT_CONFIG_VALUE_0", "true");
    for (name, value) in &stage.extra_environment {
        command.env(name, value);
    }
    return command;
}

/// Run the push of a stage under a patience, with its time.
fn run(stage: &Stage, patience: Patience) -> (PushAttempt, Duration) {
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(push_command(stage), patience, GRACE, scratch_environment().as_slice()).expect("runs");
    return (attempt, started.elapsed());
}

/// A push that keeps moving data past the stall window, then waits for its disk past it again (the phase after the hook is shaped
/// by a probe, so it is exact), is not stopped, and finishes.
#[test]
fn a_push_that_keeps_moving_data_past_the_stall_window_is_not_stopped() {
    let mut stage: Stage = stage("patience-progress", Hook::ProgressThenAccept);
    hold_disk_wait_after_hook(&mut stage, POST_PHASE_HOLD);
    let (attempt, elapsed): (PushAttempt, Duration) = run(&stage, Patience::Progress { stall: PROGRESS_STALL, ceiling: Duration::from_secs(120) });
    assert!(elapsed > PROGRESS_STALL + POST_PHASE_HOLD, "both phases run, and the second is longer than the window alone: {elapsed:?}");
    assert_eq!(attempt.exit_code, 0, "a push that moves data finishes: {}", attempt.output);
    remove(stage.root.as_path());
}

/// A push that moves nothing is stopped after the stall window, and the stop says which limit it hit, what moved and how long.
#[test]
fn a_push_that_moves_nothing_is_stopped_after_the_stall_window() {
    let stage: Stage = stage("patience-stall", Hook::Silent);
    let (attempt, elapsed): (PushAttempt, Duration) = run(&stage, Patience::Progress { stall: STALL, ceiling: Duration::from_secs(60) });
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "{}", attempt.output);
    assert!(elapsed >= STALL, "not before the stall window: {elapsed:?}");
    assert!(elapsed < STALL + GRACE + STOP_SLACK, "not long after it: {elapsed:?}");
    assert!(attempt.output.contains("made no progress for 2 seconds"), "{}", attempt.output);
    assert!(attempt.output.contains("limit=stall"), "{}", attempt.output);
    assert!(attempt.output.contains("bytes_moved="), "{}", attempt.output);
    assert!(attempt.output.contains("elapsed_seconds="), "{}", attempt.output);
    remove(stage.root.as_path());
}

/// A push that trickles data for ever is stopped at the ceiling, and the stop says so.
#[test]
fn a_push_that_trickles_is_stopped_at_the_ceiling() {
    let stage: Stage = stage("patience-ceiling", Hook::TrickleForever);
    let (attempt, elapsed): (PushAttempt, Duration) = run(&stage, Patience::Progress { stall: STALL, ceiling: CEILING });
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "{}", attempt.output);
    assert!(elapsed >= CEILING, "a trickle outlives the stall window and is stopped at the ceiling: {elapsed:?}");
    assert!(elapsed < CEILING + GRACE + STOP_SLACK, "{elapsed:?}");
    assert!(attempt.output.contains("limit=ceiling"), "{}", attempt.output);
    assert!(attempt.output.contains("bytes_moved="), "{}", attempt.output);
    assert!(attempt.output.contains("elapsed_seconds="), "{}", attempt.output);
    remove(stage.root.as_path());
}

/// The ceiling comes from its variable or the default, and a push's patience carries the stall window and the ceiling.
#[test]
fn the_ceiling_and_the_patience_come_from_the_environment() {
    let named = |name: &str, value: &str| -> Vec<(OsString, OsString)> { return vec![(OsString::from(name), OsString::from(value))]; };
    assert_eq!(push_ceiling(&[]), DEFAULT_PUSH_CEILING);
    assert!(DEFAULT_PUSH_CEILING > DEFAULT_PUSH_DEADLINE, "the ceiling is past the stall window");
    assert_eq!(push_ceiling(&named(PUSH_CEILING_VARIABLE, "90")), Duration::from_secs(90));
    for unusable in ["0", "-1", "1.5", "", "soon"] {
        assert_eq!(push_ceiling(&named(PUSH_CEILING_VARIABLE, unusable)), DEFAULT_PUSH_CEILING, "{unusable:?}");
    }
    let both: Vec<(OsString, OsString)> = vec![
        (OsString::from(PUSH_DEADLINE_VARIABLE), OsString::from("7")),
        (OsString::from(PUSH_CEILING_VARIABLE), OsString::from("70")),
    ];
    assert_eq!(push_patience(&both), Patience::Progress { stall: Duration::from_secs(7), ceiling: Duration::from_secs(70) });
}

/// A push that waits for its disk for longer than the stall window (an fsync of an object or a ref on a slow volume) moves no
/// byte and uses no processor time, but is not stalled: it finishes. The wait is a thread held in uninterruptible sleep by
/// the probe, so its length is exact and nothing depends on the speed of the disk under the test.
#[test]
fn a_push_in_disk_wait_longer_than_the_stall_window_is_not_stopped() {
    let hold: Duration = Duration::from_millis(5000);
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(
        disk_wait_command(hold, "exit"),
        Patience::Progress { stall: STALL, ceiling: Duration::from_secs(120) },
        GRACE,
        scratch_environment().as_slice(),
    )
    .expect("runs");
    assert!(started.elapsed() >= hold, "the push waited for its disk the whole hold: {:?}", started.elapsed());
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
}

/// A push that sleeps on a socket with nothing moving (a server that took the connection and says nothing) is a stall, and is
/// stopped after the window: real Git, a real connection to a listener that never answers.
#[test]
fn a_push_sleeping_on_a_silent_socket_is_stopped_after_the_stall_window() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a local listener");
    let port: u16 = listener.local_addr().expect("its address").port();
    let silent = std::thread::spawn(move || {
        // Take the connection and read what the client sends, and never answer: the client waits on the socket.
        if let Ok((mut stream, _)) = listener.accept() {
            let mut sink: [u8; 4096] = [0; 4096];
            while stream.read(&mut sink).is_ok_and(|count| { return count > 0; }) {}
        }
    });
    let root: PathBuf = fixture("patience-silent-socket");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", format!("http://127.0.0.1:{port}/silent.git").as_str()]);
    let stage: Stage = Stage { root, repo, extra_environment: Vec::new() };
    let (attempt, elapsed): (PushAttempt, Duration) = run(&stage, Patience::Progress { stall: STALL, ceiling: Duration::from_secs(120) });
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "{}", attempt.output);
    assert!(attempt.output.contains("limit=stall"), "{}", attempt.output);
    assert!(elapsed >= STALL, "{elapsed:?}");
    assert!(elapsed < STALL + GRACE + STOP_SLACK, "{elapsed:?}");
    remove(stage.root.as_path());
    drop(silent);
}
