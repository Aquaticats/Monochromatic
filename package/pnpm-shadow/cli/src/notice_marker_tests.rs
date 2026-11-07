//! The unit tests of the notice marker:
//! a marker left for a process id is claimed once,
//! by that id only.

/// Imports the functions under test.
use crate::notice_marker::{claim_marker_in, leave_marker_in, marker_path_in};
/// Imports the constant the name derives from.
use crate::constants::NOTICE_MARKER_PREFIX;
/// Imports the scratch directories.
use crate::test_scratch::scratch;

/// A process id and a second one apart from it,
/// so equality is never an accident.
const OUTER_PID: u32 = 7001;
const OTHER_PID: u32 = 7002;

#[test]
fn a_marker_left_for_a_process_id_is_claimed_once() {
    let dir = scratch();

    leave_marker_in(dir.path(), OUTER_PID);

    assert!(claim_marker_in(dir.path(), OUTER_PID));
    assert!(!claim_marker_in(dir.path(), OUTER_PID), "a marker is claimed only once");
}

#[test]
fn a_marker_is_for_that_process_id_only() {
    let dir = scratch();

    leave_marker_in(dir.path(), OUTER_PID);

    assert!(!claim_marker_in(dir.path(), OTHER_PID));
    assert!(claim_marker_in(dir.path(), OUTER_PID));
}

#[test]
fn claiming_with_no_marker_is_false() {
    assert!(!claim_marker_in(scratch().path(), OUTER_PID));
}

#[test]
fn leaving_a_marker_in_a_missing_directory_fails_quietly() {
    let dir = scratch();
    let missing = dir.path().join("no-such-directory");

    leave_marker_in(&missing, OUTER_PID);

    assert!(!claim_marker_in(&missing, OUTER_PID));
}

#[test]
fn the_marker_name_is_the_prefix_and_the_process_id() {
    let dir = scratch();

    let path = marker_path_in(dir.path(), OUTER_PID);

    assert_eq!(path, dir.path().join(format!("{NOTICE_MARKER_PREFIX}{OUTER_PID}")));
}
