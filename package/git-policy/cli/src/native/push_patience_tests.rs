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
use crate::test_support::{fixture, git, remove, repository, scratch_environment};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// The stall window of the controls: several sampling intervals, short enough to wait for.
const STALL: Duration = Duration::from_millis(2000);

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
    fn script(self) -> String {
        return match self {
            Hook::Silent => String::from("#!/bin/sh\nexec sleep 600\n"),
            Hook::ProgressThenAccept => format!("#!/bin/sh\ni=0\nwhile [ $i -lt 16 ]; do echo tick >&2; sleep {TICK}; i=$((i+1)); done\nexit 0\n"),
            Hook::TrickleForever => format!("#!/bin/sh\nwhile :; do echo tick >&2; sleep {TICK}; done\n"),
        };
    }
}

/// A repository and a bare remote with the hook, and the push command for them.
struct Stage {
    /// The fixture directory, removed at the end.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
}

/// Build the stage.
fn stage(name: &str, hook: Hook) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    let script: PathBuf = remote.join("hooks").join("pre-receive");
    std::fs::write(script.as_path(), hook.script()).expect("hook");
    std::fs::set_permissions(script.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    return Stage { root, repo };
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
    return command;
}

/// Run the push of a stage under a patience, with its time.
fn run(stage: &Stage, patience: Patience) -> (PushAttempt, Duration) {
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(push_command(stage), patience, GRACE, scratch_environment().as_slice()).expect("runs");
    return (attempt, started.elapsed());
}

/// A push that keeps moving data past the stall window is not stopped, and finishes.
#[test]
fn a_push_that_keeps_moving_data_past_the_stall_window_is_not_stopped() {
    let stage: Stage = stage("patience-progress", Hook::ProgressThenAccept);
    let (attempt, elapsed): (PushAttempt, Duration) = run(&stage, Patience::Progress { stall: STALL, ceiling: Duration::from_secs(60) });
    assert!(elapsed > STALL, "the hook runs longer than the stall window: {elapsed:?}");
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
    assert!(elapsed < STALL + GRACE * 4, "not long after it: {elapsed:?}");
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
    assert!(elapsed < CEILING + GRACE * 4, "{elapsed:?}");
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
