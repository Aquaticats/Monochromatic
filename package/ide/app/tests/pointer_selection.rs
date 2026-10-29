//! Click counting and unit-wise drag extension for pointer selection,
//!  without a window.

/// What:
///  Import Helix's rope and the production pointer-selection functions.
/// Why:
///  The native pointer callbacks call exactly these functions on the document's rope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope } from 'helix-core';
/// import { CLICK_INTERVAL, CLICK_SLOP, ClickCounter, Granularity, extended, unit } from 'ide-app/pointer-selection';
/// ```
use helix_core::Rope;
/// The counter,
///  its limits,
///  and the drag rule under test.
use ide_app::pointer_selection::{
    CLICK_INTERVAL, CLICK_SLOP, ClickCounter, Granularity, extended, unit,
};
/// What:
///  `Instant` is a point on a monotonic clock;
///  adding a `Duration` gives a later point.
/// Why:
///  Tests state the pause between presses instead of sleeping for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const start = performance.now(); const later = start + 100;
/// ```
use std::time::{Duration, Instant};

/// Quick presses at one place count character,
///  word,
///  line,
///  and then start over.
#[test]
fn quick_presses_cycle_through_character_word_and_line() {
    let mut clicks = ClickCounter::default();
    let start = Instant::now();
    let pause = Duration::from_millis(100);
    assert_eq!(clicks.press(start, 3, 40.0), Granularity::Character);
    assert_eq!(clicks.press(start + pause, 3, 40.0), Granularity::Word);
    assert_eq!(clicks.press(start + pause * 2, 3, 41.0), Granularity::Line);
    assert_eq!(
        clicks.press(start + pause * 3, 3, 41.0),
        Granularity::Character,
        "a fourth press starts a new single click"
    );
    assert_eq!(clicks.press(start + pause * 4, 3, 41.0), Granularity::Word);
}

/// A press that is too late,
///  on another row,
///  or too far away is a single click again.
#[test]
fn slow_or_distant_presses_are_single_clicks() {
    let start = Instant::now();
    let mut slow = ClickCounter::default();
    assert_eq!(slow.press(start, 0, 10.0), Granularity::Character);
    assert_eq!(
        slow.press(start + CLICK_INTERVAL, 0, 10.0),
        Granularity::Word,
        "exactly the interval still continues the multi-click"
    );
    let late = start + CLICK_INTERVAL * 2 + Duration::from_millis(1);
    assert_eq!(
        slow.press(late, 0, 10.0),
        Granularity::Character,
        "a pause longer than the interval must start over"
    );
    let mut moved = ClickCounter::default();
    assert_eq!(moved.press(start, 0, 10.0), Granularity::Character);
    assert_eq!(
        moved.press(start, 1, 10.0),
        Granularity::Character,
        "a press on another row must start over"
    );
    assert_eq!(
        moved.press(start, 1, 10.0 + CLICK_SLOP),
        Granularity::Word,
        "the slop distance still continues the multi-click"
    );
    assert_eq!(
        moved.press(start, 1, 10.0 + CLICK_SLOP * 2.0 + 0.5),
        Granularity::Character,
        "a press farther away than the slop must start over"
    );
    let mut cleared = ClickCounter::default();
    assert_eq!(cleared.press(start, 0, 10.0), Granularity::Character);
    cleared.reset();
    assert_eq!(
        cleared.press(start, 0, 10.0),
        Granularity::Character,
        "a reset counter must start over"
    );
}

/// Each granularity selects its unit at the pressed boundary.
#[test]
fn unit_is_a_caret_a_word_or_a_whole_line() {
    // one(0..3) blank two(4..7) blank three(8..13) LF(13) four(14..18).
    let rope = Rope::from_str("one two three\nfour");
    let text = rope.slice(..);
    assert_eq!(unit(text, 5, Granularity::Character), (5, 5));
    assert_eq!(unit(text, 5, Granularity::Word), (4, 7));
    assert_eq!(unit(text, 5, Granularity::Line), (0, 14));
    assert_eq!(unit(text, 16, Granularity::Line), (14, 18));
    assert_eq!(
        unit(text, 99, Granularity::Character),
        (18, 18),
        "positions clamp to the text"
    );
}

/// A drag keeps the pressed unit selected and grows by whole units in either direction.
#[test]
fn drag_extends_by_whole_units_and_keeps_the_pressed_unit() {
    let rope = Rope::from_str("one two three\nfour");
    let text = rope.slice(..);
    // Character drag: the anchor stays, the head follows the pointer both ways.
    assert_eq!(extended(text, (4, 4), Granularity::Character, 9), (4, 9));
    assert_eq!(extended(text, (4, 4), Granularity::Character, 1), (4, 1));
    assert_eq!(extended(text, (4, 4), Granularity::Character, 4), (4, 4));
    // Word drag from "two".
    assert_eq!(
        extended(text, (4, 7), Granularity::Word, 10),
        (4, 13),
        "dragging right into a word must take the whole word"
    );
    assert_eq!(
        extended(text, (4, 7), Granularity::Word, 1),
        (7, 0),
        "dragging left must keep the pressed word and take the whole earlier word"
    );
    assert_eq!(
        extended(text, (4, 7), Granularity::Word, 5),
        (4, 7),
        "pointer jitter inside the pressed word must not shrink it"
    );
    // Line drag from the first line into the second, and from the second back into the first.
    assert_eq!(extended(text, (0, 14), Granularity::Line, 16), (0, 18));
    assert_eq!(extended(text, (14, 18), Granularity::Line, 2), (18, 0));
}
