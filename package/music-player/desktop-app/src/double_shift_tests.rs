//! Tests for the double tap of Shift detector.
//!
//! What: each test feeds a sequence of timed inputs and checks when a double tap completes.
//! Why: the cases the human named are fast, slow, interrupted, Shift held with another key, and repeat.

use super::*;

/// What: `run` feeds timed inputs through the detector and returns the times at which a double tap completed.
fn run(inputs: &[(u64, ShiftInput)]) -> Vec<u64> {
    let mut state = DoubleShift::default();
    let mut completed = Vec::new();
    for (at, input) in inputs {
        let (next, done) = state.step(*input, *at);
        state = next;
        if done {
            completed.push(*at);
        }
    }
    return completed;
}

/// What: `down` is a first press of Shift.
const DOWN: ShiftInput = ShiftInput::ShiftDown { repeat: false };
/// What: `repeat` is an auto-repeat of a held Shift.
const REPEAT: ShiftInput = ShiftInput::ShiftDown { repeat: true };

#[test]
fn two_fast_taps_complete_a_double_tap() {
    assert_eq!(run(&[(0, DOWN), (60, ShiftInput::ShiftUp), (120, DOWN), (180, ShiftInput::ShiftUp)]), vec![180]);
}

#[test]
fn two_taps_inside_the_window_edge_complete_and_just_past_it_do_not() {
    assert_eq!(run(&[(0, DOWN), (50, ShiftInput::ShiftUp), (300, DOWN), (400, ShiftInput::ShiftUp)]), vec![400]);
    assert!(run(&[(0, DOWN), (50, ShiftInput::ShiftUp), (300, DOWN), (401, ShiftInput::ShiftUp)]).is_empty());
}

#[test]
fn two_slow_taps_do_not_complete_a_double_tap() {
    assert!(run(&[(0, DOWN), (60, ShiftInput::ShiftUp), (900, DOWN), (960, ShiftInput::ShiftUp)]).is_empty());
}

#[test]
fn a_slow_pair_still_starts_a_new_first_tap_that_a_fast_third_tap_completes() {
    let inputs = [
        (0, DOWN),
        (60, ShiftInput::ShiftUp),
        (900, DOWN),
        (960, ShiftInput::ShiftUp),
        (1000, DOWN),
        (1060, ShiftInput::ShiftUp),
    ];
    assert_eq!(run(&inputs), vec![1060]);
}

#[test]
fn a_key_between_the_taps_cancels_the_double_tap() {
    let inputs = [
        (0, DOWN),
        (60, ShiftInput::ShiftUp),
        (90, ShiftInput::OtherKey),
        (120, DOWN),
        (180, ShiftInput::ShiftUp),
    ];
    assert!(run(&inputs).is_empty());
}

#[test]
fn shift_held_with_another_key_is_a_chord_and_not_a_tap() {
    let inputs = [
        (0, DOWN),
        (20, ShiftInput::OtherKey),
        (60, ShiftInput::ShiftUp),
        (100, DOWN),
        (140, ShiftInput::ShiftUp),
    ];
    assert!(run(&inputs).is_empty());
}

#[test]
fn a_chord_between_two_clean_taps_cancels_them() {
    let inputs = [
        (0, DOWN),
        (40, ShiftInput::ShiftUp),
        (60, DOWN),
        (70, ShiftInput::OtherKey),
        (90, ShiftInput::ShiftUp),
        (110, DOWN),
        (130, ShiftInput::ShiftUp),
    ];
    assert!(run(&inputs).is_empty());
}

#[test]
fn a_held_shift_with_repeats_is_one_press_and_never_a_double_tap() {
    let inputs = [(0, DOWN), (30, REPEAT), (60, REPEAT), (90, REPEAT), (120, ShiftInput::ShiftUp)];
    assert!(run(&inputs).is_empty());
}

#[test]
fn repeats_inside_a_tap_do_not_add_a_tap() {
    let inputs = [(0, DOWN), (20, REPEAT), (40, ShiftInput::ShiftUp), (80, DOWN), (100, REPEAT), (120, ShiftInput::ShiftUp)];
    assert_eq!(run(&inputs), vec![120]);
}

#[test]
fn a_release_without_a_press_changes_nothing() {
    assert!(run(&[(0, ShiftInput::ShiftUp), (50, DOWN), (90, ShiftInput::ShiftUp)]).is_empty());
}

#[test]
fn a_third_quick_tap_after_a_completed_double_tap_starts_over() {
    let inputs = [
        (0, DOWN),
        (40, ShiftInput::ShiftUp),
        (80, DOWN),
        (120, ShiftInput::ShiftUp),
        (160, DOWN),
        (200, ShiftInput::ShiftUp),
    ];
    assert_eq!(run(&inputs), vec![120]);
}

#[test]
fn the_right_shift_key_counts_the_same_as_the_left_one_through_the_same_inputs() {
    // The window maps both Shift keys to the same inputs, so alternating keys is the same sequence.
    assert_eq!(run(&[(0, DOWN), (30, ShiftInput::ShiftUp), (60, DOWN), (90, ShiftInput::ShiftUp)]), vec![90]);
}
