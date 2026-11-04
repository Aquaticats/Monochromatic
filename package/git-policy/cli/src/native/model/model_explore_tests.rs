//! What:
//!  Controls of the schedule explorer on a toy counter model whose answers are known by hand.
//! Why:
//!  A red control proves a model only if the explorer can see a violation, keeps the trace shortest,
//!  reports a hit cap as not exhausted and counts states without repeats.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('explore', () => { test('finds the shortest trace', ...); });
//! ```

// What:     `use crate::...` names items of this test crate's root module (`crate` is the crate's top, like a TS
//           project root); the braces import several names at once.
// Why:      The controls call the explorer and read its result types.
//
// In TS you'd write (pseudocode):
// ```ts
// import { explore, Limits, Report } from './model_explore';
// ```
use crate::model_explore::{Limits, Report, explore};

/// The counter model's state is a single number.
type Counter = u32;

/// What:
///  The two steps of the toy model: add one while below the ceiling, and double while the double fits the ceiling.
/// Why:
///  From 1 the reachable states are exactly 1 to 8, and 4 is first reached by doubling twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function steps(n: number): [string, number][];
/// ```
fn counter_steps(state: &Counter) -> Vec<(String, Counter)> {
    let mut found: Vec<(String, Counter)> = Vec::new();
    if *state < 8 {
        // What:     `("add one".to_string(), *state + 1)` is a pair (a tuple); `.to_string()` makes an owned `String`
        //           from the literal `&str`; `*state` reads the number through the borrow.
        // Why:      A step is its label plus the state it leads to.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // found.push(['add one', state + 1]);
        // ```
        found.push(("add one".to_string(), *state + 1));
    }
    if *state * 2 <= 8 {
        found.push(("double".to_string(), *state * 2));
    }
    return found;
}

/// A check that never fires.
fn never(_state: &Counter) -> Option<String> {
    // What:     `None` is the absent case of `Option`.
    // Why:      This check accepts every state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}

/// The check that fires on the state 4.
fn forbids_four(state: &Counter) -> Option<String> {
    if *state == 4 {
        // What:     `Some("...".to_string())` wraps an owned text as the present case of `Option`.
        // Why:      Report why the state is refused.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return 'four reached';
        // ```
        return Some("four reached".to_string());
    }
    return None;
}

/// The terminal check that refuses every end state.
fn refuses_ends(_state: &Counter) -> Option<String> {
    return Some("stuck".to_string());
}

/// Generous limits for the toy model.
fn wide() -> Limits {
    return Limits { max_states: 1000, max_depth: 1000 };
}

/// The explorer finds a violation and its trace is the shortest schedule.
#[test]
fn finds_the_shortest_trace_to_a_violation() {
    let report: Report = explore(vec![1], counter_steps, forbids_four, never, wide());
    // What:     `.expect("...")` unwraps an `Option`, panicking with the text when it is `None`.
    // Why:      The control must fail loudly if no violation was found.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (report.violation === undefined) throw new Error('a violation');
    // ```
    let violation = report.violation.expect("a violation");
    assert_eq!(violation.message, "four reached");
    // Two steps reach 4 (add one then double, or double twice); no schedule is shorter, and the first found is kept.
    assert_eq!(violation.trace.len(), 2);
    assert!(violation.contains_in_order(&["double"]));
    assert!(!violation.contains_in_order(&["double", "double", "double"]));
    assert!(!report.exhausted);
}

/// With no violation the whole reachable set is counted once each, repeated start states included.
#[test]
fn counts_each_reachable_state_once_and_reports_exhausted() {
    let report: Report = explore(vec![1, 1], counter_steps, never, never, wide());
    assert!(report.violation.is_none());
    assert!(report.exhausted);
    assert_eq!(report.states, 8);
    // The state 7 is reached only through 6, which doubling from 3 reaches at depth 3, so 7 lies at depth 4.
    assert_eq!(report.deepest, 4);
}

/// A state cap and a depth cap each turn the report into not exhausted.
#[test]
fn a_hit_cap_is_not_exhausted() {
    let few_states: Report = explore(vec![1], counter_steps, never, never, Limits { max_states: 3, max_depth: 1000 });
    assert!(!few_states.exhausted);
    assert!(few_states.violation.is_none());
    let shallow: Report = explore(vec![1], counter_steps, never, never, Limits { max_states: 1000, max_depth: 1 });
    assert!(!shallow.exhausted);
}

/// A state with no enabled step is checked by the terminal check.
#[test]
fn a_refused_end_state_is_a_violation() {
    let report: Report = explore(vec![1], counter_steps, never, refuses_ends, wide());
    let violation = report.violation.expect("a stuck state");
    assert_eq!(violation.message, "stuck");
    assert!(!violation.trace.is_empty());
}

/// What:
///  The toy model of the bound: a chain 0 to 4 whose states 1 to 3 are a wait, a two-state retry loop 10 and 11 that is a wait,
///  and a state 20 that only the environment can leave.
/// Why:
///  It has one wait that ends within a known number of steps, one that never ends by the protocol's own steps, and one that
///  nothing of the protocol's can end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function waitSteps(n: number): [string, number][];
/// ```
fn wait_steps(state: &Counter) -> Vec<(String, Counter)> {
    let mut found: Vec<(String, Counter)> = Vec::new();
    if *state < 4 {
        found.push(("advance".to_string(), *state + 1));
    }
    if *state == 10 {
        found.push(("retry".to_string(), 11));
    }
    if *state == 11 {
        found.push(("retry".to_string(), 10));
    }
    if *state == 20 {
        found.push(("environment: the foreign lock frees".to_string(), 0));
    }
    return found;
}

/// The states of the toy model that are a wait.
fn is_waiting(state: &Counter) -> bool {
    return (1..=3).contains(state) || *state == 10 || *state == 11 || *state == 20;
}

/// A wait that ends by the protocol's own steps within the bound passes; one step shorter, it fails with the trace.
#[test]
fn a_wait_within_its_bound_passes_and_one_over_it_fails() {
    let limits = || { return Limits { max_states: 1000, max_depth: 1000 }; };
    let within = crate::model_explore::explore_bounded(vec![0], wait_steps, never, is_waiting, 3, limits());
    assert!(within.violation.is_none() && within.exhausted);
    let over = crate::model_explore::explore_bounded(vec![0], wait_steps, never, is_waiting, 2, limits());
    let violation = over.violation.expect("a wait of three steps against a bound of two");
    assert_eq!(violation.message, "a wait lasted more than 2 steps of the protocol's own");
    assert_eq!(violation.trace.len(), 3);
}

/// A retry loop never leaves the wait, and a state only the environment can leave has nothing of the protocol's to end it.
#[test]
fn a_loop_and_an_environment_only_wait_both_fail() {
    let limits = || { return Limits { max_states: 1000, max_depth: 1000 }; };
    let looping = crate::model_explore::explore_bounded(vec![10], wait_steps, never, is_waiting, 50, limits());
    assert_eq!(looping.violation.expect("a loop").message, "a wait lasted more than 50 steps of the protocol's own");
    let stuck = crate::model_explore::explore_bounded(vec![20], wait_steps, never, is_waiting, 50, limits());
    assert_eq!(stuck.violation.expect("environment only").message, "a wait with nothing of the protocol's own left that can end it");
}

/// The reviewer's example for `explore_bounded_by`: a waiting state whose only successor is another process's retry, which does not
/// count as a step of the waiting process. The waiting process has no step of its own at all.
fn other_lane_steps(state: &Counter) -> Vec<(String, Counter)> {
    return vec![("other lane: retries".to_string(), *state)];
}

/// Only steps labelled as the waiting process's own count.
fn counts_the_waiters_steps(label: &str) -> bool {
    return label.starts_with("waiter:");
}

/// Always waiting.
fn always_waiting(_state: &Counter) -> bool {
    return true;
}

/// Review 2a P1: another process's retry loop is the only successor; the wait is not bounded and must not be reported green.
#[test]
fn a_wait_whose_only_successor_is_anothers_retry_is_not_bounded() {
    let limits = Limits { max_states: 100, max_depth: 100 };
    let report = crate::model_explore::explore_bounded_by(vec![0], other_lane_steps, never, always_waiting, counts_the_waiters_steps, 3, limits);
    let violation = report.violation.expect("a wait with no step of the waiting process's own");
    assert_eq!(violation.message, "a wait with nothing of the protocol's own left that can end it");
}

/// A cycle of the other process's steps over several waiting states, with a step of the waiter's own always available, passes only
/// if that step ends the wait within the bound; here it does not (it loops among waiting states), so the bound is exceeded.
fn cycle_steps(state: &Counter) -> Vec<(String, Counter)> {
    return vec![("other lane: retries".to_string(), (*state + 1) % 2), ("waiter: looks again".to_string(), (*state + 1) % 2)];
}

/// Review 2a: noncounted multi-state cycles, and a counted step that does not end the wait.
#[test]
fn a_waiters_own_step_that_does_not_end_the_wait_exceeds_the_bound() {
    let limits = Limits { max_states: 100, max_depth: 100 };
    let report = crate::model_explore::explore_bounded_by(vec![0], cycle_steps, never, always_waiting, counts_the_waiters_steps, 3, limits);
    assert_eq!(report.violation.expect("exceeded").message, "a wait lasted more than 3 steps of the protocol's own");
}

/// The toy model of the liveness check: 0 to 2 is a chain to the goal 3, 10 and 11 are a retry loop with no exit, 20 is stuck.
fn finish_steps(state: &Counter) -> Vec<(String, Counter)> {
    let mut found: Vec<(String, Counter)> = Vec::new();
    if *state < 3 {
        found.push(("advance".to_string(), *state + 1));
    }
    if *state == 10 {
        found.push(("retry".to_string(), 11));
    }
    if *state == 11 {
        found.push(("retry".to_string(), 10));
    }
    if *state == 20 {
        found.push(("environment: someone helps".to_string(), 3));
    }
    return found;
}

/// The goal of the toy model.
fn at_three(state: &Counter) -> bool {
    return *state == 3;
}

/// Review 2a P2: a cycle of own steps with no exit to a goal, and a state only the environment can leave, are both reported; a chain
/// to the goal is not.
#[test]
fn a_cycle_with_no_exit_and_a_state_only_the_environment_can_leave_never_complete() {
    let limits = || { return Limits { max_states: 100, max_depth: 100 }; };
    assert!(crate::model_explore::explore_eventually(vec![0], finish_steps, at_three, limits()).violation.is_none());
    let looping = crate::model_explore::explore_eventually(vec![10], finish_steps, at_three, limits());
    assert_eq!(looping.violation.expect("a loop").message, "a state from which the protocol's own steps can never complete");
    let stuck = crate::model_explore::explore_eventually(vec![20], finish_steps, at_three, limits());
    assert!(stuck.violation.is_some());
}

/// Review 3a P2: a state or depth cap that omits edges does not manufacture a liveness violation; the report is inconclusive
/// (`exhausted == false`), and `deepest` is the deepest state reached.
#[test]
fn a_truncated_search_reports_no_liveness_violation_it_cannot_support() {
    let narrow = Limits { max_states: 2, max_depth: 100 };
    let truncated = crate::model_explore::explore_eventually(vec![0], finish_steps, at_three, narrow);
    assert!(truncated.violation.is_none() && !truncated.exhausted);
    let shallow = Limits { max_states: 100, max_depth: 1 };
    let cut = crate::model_explore::explore_eventually(vec![0], finish_steps, at_three, shallow);
    assert!(cut.violation.is_none() && !cut.exhausted);
    let full = crate::model_explore::explore_eventually(vec![0], finish_steps, at_three, Limits { max_states: 100, max_depth: 100 });
    assert_eq!(full.deepest, 3);
}
