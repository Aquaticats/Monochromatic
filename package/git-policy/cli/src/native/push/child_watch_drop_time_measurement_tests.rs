//! A measurement for the handover,
//! not a control:
//! how long dropping a watch takes (the watcher task ends and,
//! after a stop,
//! sweeps
//!  the lock files of the stopped step).
//! `WATCHER_END_BOUND` cites its result.
//!  Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture measure_how_long_dropping_a_watch`.
#![cfg(unix)]

use super::*;
use crate::test_support::{SharedSink, fixture, remove, repository};
use std::path::PathBuf;
use std::process::Stdio;

/// The longest time a stopped step took past its window until the call returned,
/// in microseconds,
/// in any run of the measurements
///  below on 2026-11-07 (a child that answers the stop request at once;
/// the signal,
/// the 10 ms look,
/// the lock sweep and the drop).
const LONGEST_STOP_TAIL_SEEN_MICROS: u64 = 41_680;

/// How many times that tail the end bound adds beyond the stop's grace:
/// a stop that must wait out the grace still has to be
///  followed by a kill,
/// a sweep and the drop.
const STOP_TAIL_MARGIN: u32 = 20;

/// The longest delay seen of a check and a rename,
/// in microseconds,
/// on which the owner's margin below is applied to a stop
///  (`foundation/private_storage_hold_time_measurement_tests.rs`,
/// `LONGEST_HOLD_SEEN_MICROS`).
const LONGEST_DELAY_SEEN_MICROS: u64 = 6_031_986;

/// The owner's pick,
/// 2026-11-07,
/// "3x worst case",
/// for how long a stop waits for a Git that cannot be interrupted before it forces the
///  end.
const OWNER_MARGIN: u32 = 3;

/// Dropping a watch may wait for the watcher task for as long as a stop can legitimately last:
/// the longest a stop waits for a member
///  that cannot be interrupted (the owner's multiple of the longest delay seen),
/// then the longest tail measured with its margin.
///  A bound below that would cancel the kill of a stubborn descendant that holds no pipe.
#[test]
fn the_end_bound_covers_a_stop_at_its_longest_and_the_longest_tail_measured() {
    let longest_stop: Duration = Duration::from_micros(LONGEST_DELAY_SEEN_MICROS) * OWNER_MARGIN;
    let covered: Duration = longest_stop + Duration::from_micros(LONGEST_STOP_TAIL_SEEN_MICROS) * STOP_TAIL_MARGIN;
    assert!(WATCHER_END_BOUND >= covered, "{WATCHER_END_BOUND:?} does not cover {covered:?}");
}

/// How many drops of each kind are timed.
const DROPS: usize = 300;

/// How many stops are timed (each takes a notice,
/// a signal and the grace of a child that answers it).
const STOPS: usize = 20;

/// Prints the spread of `micros`:
/// how many,
/// the middle,
/// the 99th and 99.9th percentile and the largest,
/// in microseconds.
fn print_spread(label: &str, mut micros: Vec<u128>) {
    micros.sort_unstable();
    let at = |per_thousand: usize| -> u128 { return micros[(micros.len() * per_thousand / 1000).min(micros.len() - 1)]; };
    println!(
        "{label}: n={} median={}us p99={}us p99.9={}us max={}us",
        micros.len(),
        at(500),
        at(990),
        at(999),
        micros.last().copied().unwrap_or(0)
    );
}

/// Terms with a long window and a quick cadence.
fn terms(limit: Duration, sink: &SharedSink) -> WatchTerms {
    return WatchTerms {
        limit,
        cadence: crate::wait_progress::Cadence { grace: Duration::from_millis(60), interval: Duration::from_millis(60) },
        notices: Box::new(sink.clone()),
    };
}

/// Times the drop of a watch between waits,
/// in the middle of a wait,
/// and the whole of a stop;
/// and the sweep alone in a repository.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_how_long_dropping_a_watch_takes() {
    let mut child: std::process::Child =
        crate::test_support::spawn_owned(Command::new("sleep").arg("120").stdin(Stdio::null())).expect("a child to name");
    let sink: SharedSink = SharedSink::default();
    let mut idle: Vec<u128> = Vec::new();
    let mut in_a_wait: Vec<u128> = Vec::new();
    for _ in 0..DROPS {
        let idle_watch: ChildWatch = ChildWatch::start("an idle watch", child.id(), terms(Duration::from_secs(3600), &sink));
        // The task is started on a worker; let it reach its first wait for a mark.
        std::thread::sleep(Duration::from_millis(2));
        let idle_started: Instant = Instant::now();
        drop(idle_watch);
        idle.push(idle_started.elapsed().as_micros());
        let waiting_watch: ChildWatch = ChildWatch::start("a watch in a wait", child.id(), terms(Duration::from_secs(3600), &sink));
        waiting_watch.begin();
        std::thread::sleep(Duration::from_millis(80));
        let waiting_started: Instant = Instant::now();
        drop(waiting_watch);
        in_a_wait.push(waiting_started.elapsed().as_micros());
    }
    child.kill().expect("end the child");
    child.wait().expect("collect the child");
    print_spread("drop between waits", idle);
    print_spread("drop in the middle of a wait (a notice may be out)", in_a_wait);
    let window: Duration = Duration::from_millis(300);
    let mut overshoot: Vec<u128> = Vec::new();
    for _ in 0..STOPS {
        let started: Instant = Instant::now();
        output_watched(Command::new("sleep").arg("120").stdin(Stdio::null()), "a silent step", terms(window, &sink)).expect_err("ended");
        overshoot.push(started.elapsed().saturating_sub(window).as_micros());
    }
    print_spread("a stopped step: time past its window until the call returns (signal, grace, lock sweep, drop)", overshoot);
    let root: PathBuf = fixture("drop-time-measurement");
    let repo: PathBuf = repository(root.as_path(), "repo");
    for number in 0..40 {
        crate::test_support::git(repo.as_path(), &["branch", format!("b{number}").as_str()]);
    }
    let mut sweeps: Vec<u128> = Vec::new();
    for _ in 0..DROPS {
        let mut command: Command = Command::new("git");
        command.current_dir(repo.as_path());
        let ledger: LockLedger = LockLedger::before_start(&command);
        let started: Instant = Instant::now();
        ledger.sweep();
        sweeps.push(started.elapsed().as_micros());
    }
    print_spread("the lock sweep alone, in a repository of forty branches", sweeps);
    remove(root.as_path());
}
