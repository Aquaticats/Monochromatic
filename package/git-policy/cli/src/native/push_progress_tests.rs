//! Controls for the meter and the clock of a push's progress, on real short-lived processes that write, sleep and start
//! others at known times.
//! Why:
//!  The stop on no progress is only as exact as the bytes it counts and the limits it keeps, so each is pinned on both sides:
//!  bytes of the root and of what it started count, an idle tree counts nothing, a moving tree never stalls, a stalled tree
//!  stops at its window, a trickle stops at its ceiling, and where nothing is measured the window is a fixed deadline.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(meter.sample(pid)).toBe(true); await sleep(idle); expect(meter.sample(pid)).toBe(false);
//! ```
#![cfg(unix)]

use super::*;
use crate::test_support::{fixture, remove};
use std::process::{Child, Command, Stdio};

/// Pause between two looks of a control, shorter than every limit it uses.
const LOOK: Duration = Duration::from_millis(50);

/// How long a control waits for something that must happen, so a broken meter fails it and never holds it.
const PATIENCE: Duration = Duration::from_secs(20);

/// A shell command running `script` in `directory`, with its output discarded.
fn shell(script: &str, directory: &std::path::Path) -> Child {
    return Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start the shell");
}

/// End a child the control started.
fn end(mut child: Child) {
    child.kill().expect("end the child");
    child.wait().expect("reap the child");
}

/// Look at the tree until a sample shows nothing new, which an idle tree does within a few looks of its start.
fn settle(meter: &mut Meter, root: i32) {
    let began: Instant = Instant::now();
    while meter.sample(root) {
        assert!(began.elapsed() < PATIENCE, "the tree never went idle");
        std::thread::sleep(LOOK);
    }
}

/// An idle tree moves nothing between looks, and a write of the root shows at the next look.
#[test]
fn a_write_of_the_root_is_progress_and_an_idle_tree_is_not() {
    let root: std::path::PathBuf = fixture("progress-root");
    let child: Child = shell("sleep 0.6; echo a >> log; sleep 60", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let idle: u64 = meter.moved();
    assert!(!meter.sample(pid), "nothing moved between two looks of an idle tree");
    assert_eq!(meter.moved(), idle);
    let began: Instant = Instant::now();
    let mut seen: bool = false;
    while !seen && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
        seen = meter.sample(pid);
    }
    assert!(seen, "the write of the root was seen");
    assert!(meter.moved() > idle);
    end(child);
    remove(root.as_path());
}

/// What a process started by the root writes counts too: the root itself only waits.
#[test]
fn a_write_of_a_process_the_root_started_is_progress() {
    let root: std::path::PathBuf = fixture("progress-descendant");
    let child: Child = shell("(sleep 0.6; echo a >> log; sleep 60) & wait", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let idle: u64 = meter.moved();
    let began: Instant = Instant::now();
    let mut seen: bool = false;
    while !seen && began.elapsed() < PATIENCE {
        std::thread::sleep(LOOK);
        seen = meter.sample(pid);
    }
    assert!(seen, "the write of the descendant was seen");
    assert!(meter.moved() > idle);
    end(child);
    remove(root.as_path());
}

/// A process that ends takes its counters with it, and the bytes it moved stay counted.
#[test]
fn the_bytes_of_a_process_that_ended_stay_counted() {
    let root: std::path::PathBuf = fixture("progress-ended");
    let child: Child = shell("(echo a >> log; echo b >> log) ; sleep 60", root.as_path());
    let pid: i32 = i32::try_from(child.id()).expect("pid");
    let mut meter: Meter = Meter::new();
    settle(&mut meter, pid);
    let counted: u64 = meter.moved();
    assert!(counted > 0, "the shell and its subshell moved bytes");
    assert!(!meter.sample(pid));
    assert_eq!(meter.moved(), counted, "a subshell that has ended does not take its bytes away");
    end(child);
    remove(root.as_path());
}

/// A process that does not exist moves nothing, and the system counts bytes for a process that does.
#[test]
fn a_missing_process_moves_nothing_and_this_system_counts_bytes() {
    assert_eq!(bytes_moved_by(i32::MAX), None);
    assert!(bytes_moved_by(i32::try_from(std::process::id()).expect("pid")).is_some());
    assert!(system_measures());
    assert!(!Meter::new().sample(i32::MAX));
}

/// A fixed patience stops at its limit, and not before.
#[test]
fn a_fixed_patience_stops_at_its_limit() {
    let child: Child = Command::new("sleep").arg("60").spawn().expect("sleep");
    let limit: Duration = Duration::from_millis(400);
    let mut clock: Clock = Clock::start(child.id(), Patience::Fixed(limit));
    let began: Instant = Instant::now();
    assert!(clock.check().is_none(), "not yet");
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    assert!(matches!(cause, StopCause::Deadline));
    assert!(began.elapsed() >= limit);
    end(child);
}

/// A tree that moves nothing stalls at its window, with the window, the bytes moved and the time in the report.
#[test]
fn a_tree_that_moves_nothing_stalls_at_its_window() {
    let child: Child = Command::new("sleep").arg("60").spawn().expect("sleep");
    let stall: Duration = Duration::from_millis(1200);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall, ceiling: Duration::from_secs(600) });
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    let StopCause::Stalled(report) = cause else { panic!("a stall, not another limit") };
    assert_eq!(report.limit, stall);
    assert!(report.elapsed >= stall, "{:?}", report.elapsed);
    assert!(report.elapsed < stall + SAMPLE_INTERVAL * 6, "{:?}", report.elapsed);
    end(child);
}

/// A tree that keeps moving never stalls, and a trickle stops at the ceiling with the ceiling in the report.
#[test]
fn a_tree_that_keeps_moving_stops_only_at_the_ceiling() {
    let root: std::path::PathBuf = fixture("progress-ceiling");
    let child: Child = shell("while :; do echo x >> log; sleep 0.1; done", root.as_path());
    let stall: Duration = Duration::from_millis(1500);
    let ceiling: Duration = Duration::from_millis(4000);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall, ceiling });
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    let StopCause::Ceiling(report) = cause else { panic!("the ceiling, not a stall") };
    assert_eq!(report.limit, ceiling);
    assert!(report.elapsed >= ceiling, "the stall window of {stall:?} passed with data moving and nothing stopped it");
    assert!(report.moved > 0);
    end(child);
    remove(root.as_path());
}

/// Where nothing is measured the stall window is a fixed deadline, even for a tree that writes all the time.
#[test]
fn without_a_measure_the_stall_window_is_a_fixed_deadline() {
    let root: std::path::PathBuf = fixture("progress-unmeasured");
    let child: Child = shell("while :; do echo x >> log; sleep 0.1; done", root.as_path());
    let stall: Duration = Duration::from_millis(600);
    let mut clock: Clock = Clock::start(child.id(), Patience::Progress { stall, ceiling: Duration::from_secs(600) }).unmeasured();
    let began: Instant = Instant::now();
    let cause: StopCause = loop {
        if let Some(cause) = clock.check() {
            break cause;
        }
        assert!(began.elapsed() < PATIENCE);
        std::thread::sleep(LOOK);
    };
    assert!(matches!(cause, StopCause::Deadline), "no measure, so the window is the whole deadline");
    assert!(began.elapsed() >= stall);
    end(child);
    remove(root.as_path());
}
