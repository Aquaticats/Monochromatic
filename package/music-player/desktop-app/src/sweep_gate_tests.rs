//! Tests for `sweep_gate.rs`, pulled in by `#[cfg(test)] #[path = "sweep_gate_tests.rs"] mod tests;` at the bottom of
//! that file. Parked workers are real threads; "still parked" is checked by waiting a short while for a signal that must
//! not come, and "released" by waiting for one that must.

use super::*;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// How long a worker that must stay parked is watched.
const PARKED_WATCH: Duration = Duration::from_millis(150);

/// How long a worker that must wake is waited for.
const WAKE_WITHIN: Duration = Duration::from_secs(5);

/// What: `park_worker` starts a thread that asks the gate for admission once and reports the answer.
/// Why: the answer arrives on a channel, so a test can tell "still parked" from "released".
fn park_worker(gate: &Arc<SweepGate>) -> mpsc::Receiver<Admission> {
    let (sender, receiver) = mpsc::channel();
    let worker_gate = Arc::clone(gate);
    thread::spawn(move || {
        let _ = sender.send(worker_gate.admit());
    });
    return receiver;
}

// What: a running gate admits at once.
// Why: a sweep nobody paused must not wait.
#[test]
fn a_running_gate_admits_at_once() {
    let gate = SweepGate::new();
    assert_eq!(gate.admit(), Admission::Proceed);
    assert!(!gate.is_paused());
}

// What: a paused gate holds a worker until resume, then lets it proceed.
// Why: pausing must stop new claims, and resume must continue them.
#[test]
fn pausing_parks_a_worker_and_resuming_releases_it() {
    let gate = Arc::new(SweepGate::new());
    gate.set_paused(true);
    assert!(gate.is_paused());
    let answer = park_worker(&gate);
    assert!(answer.recv_timeout(PARKED_WATCH).is_err(), "a paused gate admitted a worker");
    gate.set_paused(false);
    assert_eq!(answer.recv_timeout(WAKE_WITHIN), Ok(Admission::Proceed));
    assert!(!gate.is_paused());
}

// What: one resume wakes every parked worker.
// Why: the sweep runs one worker per core, and all of them must continue.
#[test]
fn resuming_releases_every_parked_worker() {
    let gate = Arc::new(SweepGate::new());
    gate.set_paused(true);
    let answers: Vec<_> = (0..6).map(|_| return park_worker(&gate)).collect();
    gate.set_paused(false);
    for answer in answers {
        assert_eq!(answer.recv_timeout(WAKE_WITHIN), Ok(Admission::Proceed));
    }
}

// What: closing the gate while paused releases parked workers with Stop.
// Why: shutdown while paused must not hang the sweep's coordinator.
#[test]
fn closing_while_paused_releases_parked_workers_with_stop() {
    let gate = Arc::new(SweepGate::new());
    gate.set_paused(true);
    let answer = park_worker(&gate);
    assert!(answer.recv_timeout(PARKED_WATCH).is_err());
    gate.close();
    assert_eq!(answer.recv_timeout(WAKE_WITHIN), Ok(Admission::Stop));
}

// What: a closed gate stays closed when a late pause or resume arrives.
// Why: the indicator may report a pause change after shutdown began.
#[test]
fn a_closed_gate_ignores_later_pause_changes() {
    let gate = SweepGate::new();
    gate.close();
    gate.set_paused(true);
    assert!(!gate.is_paused());
    assert_eq!(gate.admit(), Admission::Stop);
    gate.set_paused(false);
    assert_eq!(gate.admit(), Admission::Stop);
}

// What: pausing twice and resuming twice change nothing the second time.
// Why: the indicator's signal and its click can both arrive.
#[test]
fn repeated_pause_changes_are_harmless() {
    let gate = SweepGate::new();
    gate.set_paused(true);
    gate.set_paused(true);
    assert!(gate.is_paused());
    gate.set_paused(false);
    gate.set_paused(false);
    assert!(!gate.is_paused());
    assert_eq!(gate.admit(), Admission::Proceed);
}

// What: the shared gate is one value.
// Why: the indicator and the controller must reach the same gate.
#[test]
fn the_shared_gate_is_a_single_instance() {
    assert!(Arc::ptr_eq(&SweepGate::shared(), &SweepGate::shared()));
}
