//! Verifies each limit of one pass on its own:
//! a fresh pass lets a move go,
//! two failed moves postpone the rest,
//! the pass's time budget postpones the rest,
//! and a manager that did not answer blocks every move for a while.

/// The gate under test.
use crate::pass_gate::{PassGate, BLOCK_AFTER_TIMEOUT};
/// The instants the cases are built from.
use std::time::{Duration, Instant};

/// A fresh pass lets a move go at once.
#[test]
fn a_fresh_pass_lets_a_move_go() {
    let gate = PassGate::new();
    let now = Instant::now();
    assert_eq!(gate.postponement(now), None);
    assert!(!gate.is_blocked(now));
}

/// A pass that spent its failed attempts postpones the rest,
/// and a new pass lets moves go again.
#[test]
fn two_failed_moves_postpone_the_rest_of_that_pass() {
    let mut gate = PassGate::new();
    let now = Instant::now();
    gate.begin(now);
    gate.note_failed_attempt();
    assert_eq!(gate.postponement(now), None, "one failure is inside the budget");
    gate.note_failed_attempt();
    let why = gate.postponement(now).expect("the pass has spent its failed attempts");
    assert!(why.contains("at most two failed attempts"), "{why}");
    gate.begin(now);
    assert_eq!(gate.postponement(now), None, "the next pass starts with its budget back");
}

/// A pass that ran past its time budget postpones the rest,
/// even with its failed attempts unspent.
#[test]
fn a_pass_past_its_time_budget_postpones_the_rest() {
    let mut gate = PassGate::new();
    let start = Instant::now();
    gate.begin(start);
    assert_eq!(gate.postponement(start), None);
    let late = start + Duration::from_secs(5);
    let why = gate.postponement(late).expect("the pass is past its budget");
    assert!(why.contains("time budget"), "{why}");
}

/// A manager that did not answer blocks every move for `BLOCK_AFTER_TIMEOUT`,
/// and the block ends by itself.
#[test]
fn a_manager_that_did_not_answer_blocks_moves_for_the_block_duration() {
    let mut gate = PassGate::new();
    let now = Instant::now();
    gate.begin(now);
    gate.block(now);
    assert!(gate.is_blocked(now));
    let why = gate.postponement(now).expect("a blocked gate postpones moves");
    assert!(why.contains("did not answer in time"), "{why}");
    assert!(gate.is_blocked(now + BLOCK_AFTER_TIMEOUT - Duration::from_millis(1)), "the block ended early");
    assert!(!gate.is_blocked(now + BLOCK_AFTER_TIMEOUT), "the block outlived its duration");
}
