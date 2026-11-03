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
    assert!(violation.trace.len() >= 1);
}
