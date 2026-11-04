//! What:
//!  Controls of the drain request's bound (plan step 2, "hook outlasting an upgrade caller, the caller returns within its bound"): the
//!  caller of an upgrade or restart waits for the service to drain at most the bound, then returns non-zero saying the drain is in
//!  progress and the upgrade is not applied, and the drain goes on.
//! Why:
//!  A hook that runs for ever must delay the drain (it is protected) and must not hold the caller for ever. The bound is a parameter:
//!  the design's `WAIT_DRAIN` is a constant that must be measured, and no measurement exists yet, so none is shipped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('a drain that never ends returns in progress at the bound', ...);
//! ```
#![cfg(feature = "test-seams")]

use git_policy_cli::commit_drain::{DRAIN_IN_PROGRESS, DrainAnswer, wait_for_drain};
use std::time::{Duration, Instant};

/// The longest a test lets a bounded wait run.
const TEST_LIMIT: Duration = Duration::from_secs(10);

/// A drain that completes inside the bound is reported as drained, after as many polls as it took.
#[test]
fn a_drain_that_completes_inside_the_bound_is_reported_drained() {
    let mut polls: u32 = 0;
    let answer: DrainAnswer = wait_for_drain(TEST_LIMIT, Duration::from_millis(1), &mut || { polls += 1; return polls >= 3; });
    assert_eq!(answer, DrainAnswer::Drained);
    assert_eq!(polls, 3);
}

/// A hook that never ends: the caller returns at the bound with the in-progress answer, the drain is never cancelled (the poll is the
/// only thing the caller does to it), and the message says that the upgrade was not applied.
#[test]
fn a_drain_that_never_ends_returns_in_progress_at_the_bound_and_is_not_cancelled() {
    let bound: Duration = Duration::from_millis(40);
    let started: Instant = Instant::now();
    let mut polls: u32 = 0;
    let answer: DrainAnswer = wait_for_drain(bound, Duration::from_millis(2), &mut || { polls += 1; return false; });
    assert_eq!(answer, DrainAnswer::InProgress);
    assert!(started.elapsed() >= bound && started.elapsed() < TEST_LIMIT, "{:?}", started.elapsed());
    assert!(polls >= 2, "the caller kept asking until the bound");
    assert_eq!(DRAIN_IN_PROGRESS, "draining, upgrade not applied");
}

/// The control: a wait with no bound is the failure the row names. The test shows that the bounded wait ends where an unbounded one would
/// not, by giving the unbounded one a drain that ends only after the bound.
#[test]
fn a_drain_that_ends_only_after_the_bound_is_reported_in_progress_not_waited_for() {
    let bound: Duration = Duration::from_millis(30);
    let finishes_at: Instant = Instant::now() + bound * 20;
    let answer: DrainAnswer = wait_for_drain(bound, Duration::from_millis(2), &mut || { return Instant::now() >= finishes_at; });
    assert_eq!(answer, DrainAnswer::InProgress, "the caller waited past its bound for a drain that was still running");
}
