//! Controls for the instant a control may hold a start step at:
//!  the value names the step and the path,
//!  a step the value does not name waits for nothing,
//!  and a hold nothing releases ends at its ceiling.

use super::*;

/// The reached file and the release file sit beside the path the control named,
/// each with its own suffix.
#[test]
fn the_two_files_of_one_control_sit_beside_its_path() {
    let held: PathBuf = PathBuf::from("/a/control/held");
    assert_eq!(marker(held.as_path(), "reached"), PathBuf::from("/a/control/held.reached"));
    assert_eq!(marker(held.as_path(), "release"), PathBuf::from("/a/control/held.release"));
}

/// What:
///  A value that names another step leaves this one alone and leaves no file.
/// Why:
///  One control holds one step of a start,
/// so the steps it does not name must run as they always do.
#[test]
fn a_step_the_value_does_not_name_waits_for_nothing() {
    let held: PathBuf = crate::test_support::fixture("start-step-hold-other").join("held");
    let value: std::ffi::OsString = std::ffi::OsString::from(format!("another-step:{}", held.display()));
    let began: Instant = Instant::now();
    the_start_step_waits_where_a_control_says_within(Some(value.as_os_str()), "the step this control measures", CONTROL_PARK_CEILING);
    assert!(began.elapsed() < Duration::from_secs(1), "the step the value does not name waits for nothing");
    assert!(!marker(held.as_path(), "reached").exists(), "and leaves no file");
}

/// What:
///  A hold whose release file is already there leaves its reached file and returns,
///  so a control can let a step through and hold only a later one.
#[test]
fn a_hold_that_is_already_released_leaves_its_mark_and_returns() {
    let held: PathBuf = crate::test_support::fixture("start-step-hold-released").join("held");
    std::fs::write(marker(held.as_path(), "release"), b"").expect("the pre-placed release");
    let value: std::ffi::OsString = std::ffi::OsString::from(format!("the step:{}", held.display()));
    let began: Instant = Instant::now();
    the_start_step_waits_where_a_control_says_within(Some(value.as_os_str()), "the step", CONTROL_PARK_CEILING);
    assert!(marker(held.as_path(), "reached").exists(), "the service says it reached the step");
    assert!(began.elapsed() < Duration::from_secs(1), "a released hold waits for nothing");
}

/// What:
///  A hold nothing releases ends at its ceiling.
/// Why:
///  A control that armed the variable and died leaves a directory with no release file,
///  and no step of a start may wait for it for ever.
#[test]
fn a_hold_nothing_releases_ends_at_its_ceiling() {
    let held: PathBuf = crate::test_support::fixture("start-step-hold-unreleased").join("held");
    let value: std::ffi::OsString = std::ffi::OsString::from(format!("the step:{}", held.display()));
    let ceiling: Duration = Duration::from_millis(200);
    let began: Instant = Instant::now();
    the_start_step_waits_where_a_control_says_within(Some(value.as_os_str()), "the step", ceiling);
    assert!(began.elapsed() >= ceiling, "the hold ended at its ceiling");
}

/// What:
///  A value that is not `<step>:<path>` arms nothing.
/// Why:
///  The variable is a control's own hand,
/// and a malformed one must not hold a start that nobody asked to hold.
#[test]
fn a_value_that_names_no_step_and_path_waits_for_nothing() {
    let began: Instant = Instant::now();
    the_start_step_waits_where_a_control_says_within(Some(OsStr::new("no-colon-in-this-value")), "the step", CONTROL_PARK_CEILING);
    assert!(began.elapsed() < Duration::from_secs(1), "a malformed value waits for nothing");
}
