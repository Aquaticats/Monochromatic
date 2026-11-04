//! Controls for the signs of a thread: a thread that sleeps is judged quiet after the window, a thread that keeps writing is
//!  never judged quiet, and a measured job leaves its longest gap in the ledger.
#![cfg(target_os = "linux")]

use super::*;
use crate::test_support::{fixture, remove};
use std::path::PathBuf;
use std::sync::mpsc::channel;

/// A window past the sampling interval, so a thread that works shows its work at a sample before the window runs out.
const WINDOW: Duration = Duration::from_millis(1200);

/// How long the quiet fixture sleeps: far past the window, and ended early by the control when it has its answer.
const QUIET_FIXTURE: Duration = Duration::from_secs(4);

/// How long a control waits for a verdict before it fails.
const PATIENCE: Duration = Duration::from_secs(20);

/// The thread's own number is a number the kernel knows.
#[test]
fn the_calling_thread_has_a_number() {
    let tid: i32 = current_thread_id().expect("Linux names threads");
    assert!(std::path::Path::new(format!("/proc/self/task/{tid}").as_str()).exists());
}

/// A thread that sleeps is judged quiet once the window has passed, and its quiet gap is at least the window.
#[test]
fn a_sleeping_thread_is_judged_quiet_after_the_window() {
    let (sender, receiver) = channel::<i32>();
    let sleeper = std::thread::spawn(move || {
        sender.send(current_thread_id().expect("a thread number")).expect("tell the control");
        std::thread::sleep(QUIET_FIXTURE);
    });
    let tid: i32 = receiver.recv().expect("the sleeper's number");
    let mut clock: ThreadClock = ThreadClock::start(tid, WINDOW, false);
    let began: Instant = Instant::now();
    while !clock.quiet() {
        assert!(began.elapsed() < PATIENCE, "a sleeping thread was never judged quiet");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(clock.longest_quiet() >= WINDOW, "{:?}", clock.longest_quiet());
    sleeper.join().expect("the sleeper ends");
}

/// A thread that keeps writing a file is never judged quiet however long it runs, and its gap stays under the window.
#[test]
fn a_writing_thread_is_never_judged_quiet() {
    let root: PathBuf = fixture("thread-quiet-writer");
    let target: PathBuf = root.join("progress");
    let (sender, receiver) = channel::<i32>();
    let rounds: u32 = 15;
    let writer = std::thread::spawn(move || {
        sender.send(current_thread_id().expect("a thread number")).expect("tell the control");
        for round in 0..rounds {
            std::fs::write(&target, round.to_string()).expect("write");
            std::thread::sleep(Duration::from_millis(200));
        }
    });
    let tid: i32 = receiver.recv().expect("the writer's number");
    let mut clock: ThreadClock = ThreadClock::start(tid, WINDOW, false);
    let mut judged_quiet: bool = false;
    while !writer.is_finished() {
        judged_quiet |= clock.quiet();
        std::thread::sleep(Duration::from_millis(20));
    }
    let longest: Duration = clock.longest_quiet();
    writer.join().expect("the writer ends");
    assert!(!judged_quiet, "a thread that kept writing was judged quiet");
    assert!(longest < WINDOW, "{longest:?}");
    remove(root.as_path());
}

/// A thread that cannot be read is never judged quiet: nothing was measured, so nothing is known.
#[test]
fn an_unreadable_thread_is_never_judged_quiet() {
    let mut clock: ThreadClock = ThreadClock::start(i32::MAX, Duration::ZERO, false);
    std::thread::sleep(Duration::from_millis(5));
    assert!(!clock.quiet());
}

/// A measured job runs to its end and leaves the gap of its sleep in the ledger, under the step's name.
#[test]
fn a_measured_job_leaves_its_gap_in_the_ledger() {
    crate::quiet_ledger::drain();
    let pause: Duration = Duration::from_millis(900);
    let value: u8 = measured_quiet("measured fixture", || {
        std::thread::sleep(pause);
        return 3;
    });
    assert_eq!(value, 3);
    let figures = crate::quiet_ledger::drain();
    let kept = figures.iter().find(|(step, _)| { return step == "measured fixture"; }).expect("the step is in the ledger");
    assert!(kept.1 >= pause / 2 && kept.1 <= pause * 4, "{kept:?}");
}
