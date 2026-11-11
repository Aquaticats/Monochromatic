//! What:
//!  Controls of the hang guard the fault sweeps run their commands under.
//! Why:
//!  The guard decides whether a traced commit that is slow is a hang,
//! and its window is what a real sweep pays for a stuck one.
//! A clock ended eighteen `fault_trace::` controls that were moving all the time
//! (the suite of 2026-11-10,
//! `${HOME}/temp/agent/plk/suite2.log`:
//! 121 to 300 s of wall time each,
//! every one of them a working run under `strace`).
//! These controls take a window of their own,
//! so they prove the judgement in a moment rather than in two minutes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [output, ended] = spawnWatched(cmd, window).finishReporting(); // a command that keeps moving ends by itself
//! ```

use super::fault_run::spawn_watched_with;
use super::supervisor::Reason;
use super::support::fixture;
use std::process::Command;
use std::time::{Duration, Instant};

/// The window these controls give the guard:
/// long enough that a command which keeps moving is looked at several times,
/// so one slow look of a loaded host cannot pass for a hang.
const WINDOW: Duration = Duration::from_millis(400);

/// What a command that keeps moving does:
/// it writes a number,
/// waits a moment and writes the next,
/// so the bytes it moves change at every look.
const WORKING: &str = "n=0; while [ $n -lt 12 ]; do echo $n > \"$1\"; n=$((n + 1)); sleep 0.1; done";

/// What a command that stops moving does:
/// it waits for something that never comes,
/// showing nothing after it started.
const STOPPED: &str = "sleep 600";

/// A command that keeps moving past its window is never ended:
/// it ends by itself and the guard names no reason.
#[test]
fn a_command_that_keeps_moving_is_never_ended() {
    let world = fixture("fault-run-moving");
    let written = world.root.join("counter");
    let argued: &str = written.to_str().expect("a UTF-8 fixture path");
    let mut command: Command = Command::new("sh");
    command.args(["-c", WORKING, "working", argued]);
    let began: Instant = Instant::now();
    let (output, ended) = spawn_watched_with(&world, &mut command, WINDOW).finish_reporting();
    let waited: Duration = began.elapsed();
    assert!(ended.is_none(), "a command that kept moving was ended: {}", ended.expect("a reason"));
    assert!(output.status.success(), "the command ended by itself: {:?}", output.status);
    assert!(waited >= Duration::from_secs(1), "the command ran past the window, in {waited:?}");
    assert_eq!(std::fs::read_to_string(written.as_path()).expect("the counter").trim(), "11", "the last number the command wrote");
}

/// A command that stops moving is ended at its window,
/// and the words name the silence rather than a clock.
#[test]
fn a_command_that_stops_moving_is_ended_and_the_words_name_the_silence() {
    let world = fixture("fault-run-stopped");
    let mut command: Command = Command::new("sh");
    command.args(["-c", STOPPED, "stopped"]);
    let began: Instant = Instant::now();
    let (_output, ended) = spawn_watched_with(&world, &mut command, WINDOW).finish_reporting();
    let waited: Duration = began.elapsed();
    let reason: Reason = ended.expect("a command that showed nothing was ended");
    let said: String = reason.to_string();
    assert!(said.contains("no progress for"), "the guard names the silence, not a clock: {said}");
    assert!(waited < Duration::from_secs(30), "the guard ended the command at its window, in {waited:?}");
}
