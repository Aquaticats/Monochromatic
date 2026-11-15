//! Controls for the instant a control may hold the write of a request's decision at:
//!  the two files it leaves and waits for are named beside the path the control chose,
//!  a hold whose release is already there returns at once,
//!  a board with no variable armed waits for nothing,
//!  and a hold nothing releases ends at its ceiling.

use super::*;

/// The reached file and the release file sit beside the path the control named,
/// each with its own suffix,
/// so one control's directory holds both and nothing else.
#[test]
fn the_two_files_of_one_control_sit_beside_its_path() {
    let held: PathBuf = PathBuf::from("/a/control/held");
    assert_eq!(marker(held.as_path(), "reached"), PathBuf::from("/a/control/held.reached"));
    assert_eq!(marker(held.as_path(), "release"), PathBuf::from("/a/control/held.release"));
    let named_like_a_folder: PathBuf = PathBuf::from("/a/control/held/");
    assert_eq!(marker(named_like_a_folder.as_path(), "release"), PathBuf::from("/a/control/held.release"));
}

/// What:
///  A hold whose release file is already there leaves its reached file and returns,
///  so a control that pre-releases the write of the command that wakes the service is not held by it.
/// Why:
///  The first write of a fixture's service is the one that wakes it,
/// and a control measures a later one:
/// the release file is the only difference between the two.
#[test]
fn a_hold_that_is_already_released_leaves_its_mark_and_returns() {
    let held: PathBuf = crate::test_support::fixture("decision-record-hold-released").join("held");
    std::fs::write(marker(held.as_path(), "release"), b"").expect("the pre-placed release");
    let began: Instant = Instant::now();
    the_decision_record_waits_where_a_control_says_within(Some(held.as_os_str()), CONTROL_PARK_CEILING);
    let took: Duration = began.elapsed();
    assert!(marker(held.as_path(), "reached").exists(), "the board says it reached the write");
    assert!(took < Duration::from_secs(1), "a released hold waits for nothing, and took {took:?}");
}

/// What:
///  A board that carries no variable waits for nothing and leaves no file.
/// Why:
///  The variable is the only thing that arms the point,
///  and the release executable runs with none.
#[test]
fn a_board_with_no_variable_armed_waits_for_nothing() {
    let held: PathBuf = crate::test_support::fixture("decision-record-hold-unarmed").join("held");
    let began: Instant = Instant::now();
    the_decision_record_waits_where_a_control_says_within(None, CONTROL_PARK_CEILING);
    assert!(began.elapsed() < Duration::from_secs(1), "nothing was armed, so nothing waited");
    assert!(!marker(held.as_path(), "reached").exists(), "and no file was left");
}

/// What:
///  A hold nothing releases ends at its ceiling instead of parking the thread for good.
/// Why:
///  A control that armed the variable and died leaves a directory with no release file,
///  and a thread of the service must not wait for it for ever.
#[test]
fn a_hold_nothing_releases_ends_at_its_ceiling() {
    let held: PathBuf = crate::test_support::fixture("decision-record-hold-unreleased").join("held");
    let ceiling: Duration = Duration::from_millis(200);
    let began: Instant = Instant::now();
    the_decision_record_waits_where_a_control_says_within(Some(held.as_os_str()), ceiling);
    let took: Duration = began.elapsed();
    assert!(took >= ceiling, "the hold ended at its ceiling, and took {took:?}");
    assert!(marker(held.as_path(), "reached").exists(), "and it said it reached the write");
}
