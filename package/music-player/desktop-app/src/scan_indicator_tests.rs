// What:     Unit tests for `scan_indicator.rs`, pulled in by
//           `#[cfg(test)] #[path = "scan_indicator_tests.rs"] mod tests;` at the bottom of `scan_indicator.rs`.
//           Mirrors every case of the Kotlin `ScanIndicatorModelTest` with the same counts and expected text.
// Why:      Keep the tests beside the code. Phase choice, clamping, visibility, and text are each pinned.

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need `scan_indicator_for`, the phase enum, the model, and the text functions.
use super::*;

// What:     `#[test] fn idle_when_not_loading_and_no_total()` checks an empty library with no loading shows nothing.
// Why:      With no loading and no total there is nothing to report, so the bar stays hidden.
#[test]
fn idle_when_not_loading_and_no_total() {
    let model = scan_indicator_for(false, 0, 0);
    assert_eq!(model.phase, ScanIndicatorPhase::Idle);
    assert!(!model.visible());
}

// What:     `#[test] fn idle_when_negative_total()` checks a negative total is treated as no total.
// Why:      A negative count must never reach the bar, so it falls back to the idle phase.
#[test]
fn idle_when_negative_total() {
    let model = scan_indicator_for(false, 3, -5);
    assert_eq!(model.phase, ScanIndicatorPhase::Idle);
    assert_eq!(model.total, 0);
    assert!(!model.visible());
}

// What:     `#[test] fn scanning_without_total()` checks the library read phase with no known total.
// Why:      While the library is read, no track count exists yet, so the bar is indeterminate.
#[test]
fn scanning_without_total() {
    let model = scan_indicator_for(true, 0, 0);
    assert_eq!(model.phase, ScanIndicatorPhase::ScanningLibrary);
    assert_eq!(model.total, 0);
    assert!(model.visible());
}

// What:     `#[test] fn scanning_ignores_sweep_counts()` checks that loading wins over sweep counts.
// Why:      Counts from an earlier sweep must not make a loading library look measured.
#[test]
fn scanning_ignores_sweep_counts() {
    let model = scan_indicator_for(true, 3, 10);
    assert_eq!(model.phase, ScanIndicatorPhase::ScanningLibrary);
    assert_eq!(model.done, 0);
    assert_eq!(model.total, 0);
}

// What:     `#[test] fn measuring_with_counts()` checks the mid-sweep state with known counts.
// Why:      A partial sweep is the visible measuring state, and it carries its counts.
#[test]
fn measuring_with_counts() {
    let model = scan_indicator_for(false, 412, 1218);
    assert_eq!(model.phase, ScanIndicatorPhase::MeasuringTruePeak);
    assert_eq!(model.done, 412);
    assert_eq!(model.total, 1218);
    assert!(model.visible());
}

// What:     `#[test] fn measuring_at_start()` checks a sweep that has visited nothing yet.
// Why:      The bar appears as soon as measurement starts, so zero of a known total is visible.
#[test]
fn measuring_at_start() {
    let model = scan_indicator_for(false, 0, 1218);
    assert_eq!(model.phase, ScanIndicatorPhase::MeasuringTruePeak);
    assert_eq!(model.done, 0);
    assert!(model.visible());
}

// What:     `#[test] fn complete_when_measured_equals_total()` checks the last track finishes the sweep.
// Why:      Reaching the total ends the measurement, so the indicator leaves.
#[test]
fn complete_when_measured_equals_total() {
    let model = scan_indicator_for(false, 1218, 1218);
    assert_eq!(model.phase, ScanIndicatorPhase::Done);
    assert_eq!(model.done, 1218);
    assert!(!model.visible());
}

// What:     `#[test] fn zero_total_with_measured_count_is_idle()` checks counts with no total do not show a bar.
// Why:      A measured count without a total has no meaningful denominator, so nothing is shown.
#[test]
fn zero_total_with_measured_count_is_idle() {
    let model = scan_indicator_for(false, 5, 0);
    assert_eq!(model.phase, ScanIndicatorPhase::Idle);
    assert!(!model.visible());
}

// What:     `#[test] fn measured_above_total_clamps()` checks an overshooting count stops at the total.
// Why:      A cached or double-counted visit must never show more tracks than exist.
#[test]
fn measured_above_total_clamps() {
    let model = scan_indicator_for(false, 15, 10);
    assert_eq!(model.phase, ScanIndicatorPhase::Done);
    assert_eq!(model.done, 10);
    assert_eq!(model.total, 10);
}

// What:     `#[test] fn negative_measured_clamps()` checks a negative count stops at zero.
// Why:      A negative visit count must not reach the bar or its fraction.
#[test]
fn negative_measured_clamps() {
    let model = scan_indicator_for(false, -3, 10);
    assert_eq!(model.phase, ScanIndicatorPhase::MeasuringTruePeak);
    assert_eq!(model.done, 0);
}

// What:     `#[test] fn status_text_while_scanning()` checks the library read line.
// Why:      The scanning phase has its own line so the user can tell it from measurement.
#[test]
fn status_text_while_scanning() {
    let model = scan_indicator_for(true, 0, 0);
    assert_eq!(scan_indicator_status_text(&model), "Scanning library");
}

// What:     `#[test] fn status_text_while_measuring()` checks the accepted copy with grouped counts.
// Why:      The study's line is "Analyzing true peak · 412 of 1,218", and the text must match it.
#[test]
fn status_text_while_measuring() {
    let model = scan_indicator_for(false, 412, 1218);
    assert_eq!(
        scan_indicator_status_text(&model),
        "Analyzing true peak · 412 of 1,218"
    );
}

// What:     `#[test] fn status_text_wide_count()` checks the widest count the study stresses.
// Why:      Wider counts keep comma grouping and ASCII digits in the same line.
#[test]
fn status_text_wide_count() {
    let model = scan_indicator_for(false, 9_999_998, 9_999_999);
    assert_eq!(
        scan_indicator_status_text(&model),
        "Analyzing true peak · 9,999,998 of 9,999,999"
    );
}

// What:     `#[test] fn status_text_empty_when_hidden()` checks idle and done produce no text.
// Why:      A hidden indicator must not leave a stale line for the caller to draw.
#[test]
fn status_text_empty_when_hidden() {
    let idle = scan_indicator_for(false, 0, 0);
    let done = scan_indicator_for(false, 4, 4);
    assert_eq!(scan_indicator_status_text(&idle), "");
    assert_eq!(scan_indicator_status_text(&done), "");
}

// What:     `#[test] fn count_text_groups_thousands()` checks grouping at the digit boundaries.
// Why:      Grouping starts only above three digits and never adds a leading comma.
#[test]
fn count_text_groups_thousands() {
    assert_eq!(scan_indicator_count_text(0), "0");
    assert_eq!(scan_indicator_count_text(999), "999");
    assert_eq!(scan_indicator_count_text(1000), "1,000");
    assert_eq!(scan_indicator_count_text(i32::MAX), "2,147,483,647");
}
