// What:     Unit tests for `progress.rs`, pulled in by
//           `#[cfg(test)] #[path = "progress_tests.rs"] mod tests;` at
//           the bottom of `progress.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of progress.
// Why:      Keep the tests beside the code without inflating
//           `progress.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::{...};` imports names from the parent module into the
//           test module.
// Why:      Tests call the debouncer without long `super::` prefixes.
use super::{
    PROGRESS_UPDATE_DEBOUNCE_INTERVAL,
    ProgressDebouncer,
    ProgressUpdateKind,
};

// What:     `use std::time::Duration;`. Import the time-span type for test
//           timestamps.
// Why:      Test inputs are explicit elapsed times.
use std::time::Duration;

// What:     `#[test] fn first_debounced_update_surfaces()`. A unit test function.
// Why:      A fresh player still needs its first progress value to appear.
#[test]
fn first_debounced_update_surfaces() {
    // What:     `let mut debouncer = ProgressDebouncer::new();`. A mutable
    //           debouncer instance.
    // Why:      `should_surface` updates its internal baseline.
    let mut debouncer = ProgressDebouncer::new();

    // What:     `assert!(...)` fails the test unless the expression is true.
    // Why:      The first progress update must not be hidden.
    assert!(debouncer.should_surface(Duration::from_millis(0), ProgressUpdateKind::Debounced));
}

// What:     `#[test] fn rapid_debounced_updates_wait_for_interval()`. A unit
//           test for the normal debounce window.
// Why:      Repeated position ticks inside the window should not repaint.
#[test]
fn rapid_debounced_updates_wait_for_interval() {
    // What:     `let mut debouncer = ProgressDebouncer::new();`. Fresh state.
    // Why:      Start with no accepted progress baseline.
    let mut debouncer = ProgressDebouncer::new();

    // What:     First call at `0ms`.
    // Why:      Establish the baseline.
    assert!(debouncer.should_surface(Duration::from_millis(0), ProgressUpdateKind::Debounced));
    // What:     Second call before `PROGRESS_UPDATE_DEBOUNCE_INTERVAL`.
    // Why:      This is the flicker-prone rapid update case.
    assert!(!debouncer.should_surface(Duration::from_millis(100), ProgressUpdateKind::Debounced));
    // What:     Call exactly at the configured interval.
    // Why:      The next visible progress update is allowed once the gap elapsed.
    assert!(debouncer.should_surface(
        PROGRESS_UPDATE_DEBOUNCE_INTERVAL,
        ProgressUpdateKind::Debounced,
    ));
}

// What:     `#[test] fn immediate_update_surfaces_and_suppresses_following_reset()`.
//           A unit test for play-state or visibility changes.
// Why:      The taskbar must hide/show immediately, but a zero-position reset a
//           millisecond later should not flicker the progress surfaces.
#[test]
fn immediate_update_surfaces_and_suppresses_following_reset() {
    // What:     `let mut debouncer = ProgressDebouncer::new();`. Fresh state.
    // Why:      Test the exact state sequence from startup or play.
    let mut debouncer = ProgressDebouncer::new();

    // What:     Immediate update at `10ms`.
    // Why:      Visibility state must be delivered right away.
    assert!(debouncer.should_surface(Duration::from_millis(10), ProgressUpdateKind::Immediate));
    // What:     Debounced reset at `11ms`.
    // Why:      This is the rapid zero-duration track reset that flickers bars.
    assert!(!debouncer.should_surface(Duration::from_millis(11), ProgressUpdateKind::Debounced));
}
