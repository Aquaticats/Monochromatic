// What:     Unit tests for `thread_priority.rs`, pulled in by
//           `#[cfg(test)] #[path = "thread_priority_tests.rs"] mod tests;` at
//           the bottom of `thread_priority.rs`.
// Why:      The rtkit request itself needs a live system bus, so the tests cover the
//           pure rule that keeps the requested nice level inside rtkit's accepted range.

// What:     `use super::*;`. Bring the module's items into the test scope.
// Why:      Tests use `DECODE_NICE_LEVEL` and `nice_level_is_allowed`.
use super::*;

#[test]
fn decode_nice_level_is_inside_rtkit_range() {
    assert!(nice_level_is_allowed(DECODE_NICE_LEVEL));
}

#[test]
fn nice_levels_outside_rtkit_range_are_rejected() {
    assert!(!nice_level_is_allowed(-16));
    assert!(!nice_level_is_allowed(1));
}

#[test]
fn nice_range_endpoints_are_accepted() {
    assert!(nice_level_is_allowed(-15));
    assert!(nice_level_is_allowed(0));
}
