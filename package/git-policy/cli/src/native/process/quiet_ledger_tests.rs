//! Controls for the quiet-gap ledger: the longest gap of a step is kept, a step ended for being quiet is kept apart,
//!  draining forgets, and the JSON is one object.

use super::*;

/// The ledger is process-wide, so the controls take turns.
static TURN: Mutex<()> = Mutex::new(());

/// The longest gap of a step is kept, whatever order the gaps arrive in.
#[test]
fn the_longest_gap_of_a_step_is_kept() {
    let _turn = TURN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    drain();
    note("git add", Duration::from_millis(30), false);
    note("git add", Duration::from_millis(90), false);
    note("git add", Duration::from_millis(60), false);
    assert_eq!(drain(), vec![(String::from("git add"), Duration::from_millis(90))]);
}

/// A step that was ended for being quiet is kept apart from the same step ending by itself.
#[test]
fn an_ended_step_is_kept_apart_from_a_healthy_one() {
    let _turn = TURN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    drain();
    note("git add", Duration::from_millis(10), false);
    note("git add", Duration::from_millis(500), true);
    let figures = drain();
    assert_eq!(figures.len(), 2, "{figures:?}");
    assert_eq!(longest_healthy(figures.as_slice()), "10.000", "only the healthy step counts");
}

/// Draining forgets: the next command starts from nothing.
#[test]
fn draining_forgets() {
    let _turn = TURN.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    drain();
    note("pnpm", Duration::from_millis(5), false);
    assert_eq!(drain().len(), 1);
    assert!(drain().is_empty());
}

/// The JSON is one object from the escaped step name to milliseconds, and an empty list is an empty object.
#[test]
fn the_json_is_one_object() {
    let figures = vec![(String::from("git \"x\""), Duration::from_micros(1500)), (String::from("pnpm"), Duration::from_millis(2))];
    assert_eq!(object_json(figures.as_slice()), "{\"git \\\"x\\\"\":1.500,\"pnpm\":2.000}");
    assert_eq!(object_json(&[]), "{}");
    assert_eq!(longest_healthy(&[]), "0.000");
}
