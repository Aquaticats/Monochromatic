//! Controls for the settling command's words:
//!  the digits a record's GUID is confirmed by,
//!  the placeholder a printed line carries,
//!  and the usage line's placeholders.

use crate::worktree_clone_binding_record::add_state_commands::SETTLING_COMMAND_WORDS;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_record_settling::settling_words::{CONFIRM_GUID_FLAG, CONFIRM_REMOVAL_FLAG, carries_a_placeholder, the_last_four_digits, the_usage_line};

/// One GUID built from its decimal digits.
fn guid(digits: &str) -> DatasetGuid {
    return DatasetGuid::parse(digits).expect("the control's digits are a GUID");
}

/// The record's own GUID answers with its last four decimal digits,
/// which is the example the design's veto entry V21 states.
#[test]
fn the_last_four_digits_are_the_records_own() {
    assert_eq!(the_last_four_digits(guid("4821193048571903102")), "3102");
}

/// A GUID shorter than four digits answers with every digit it has,
/// so a control's small number is not refused as nothing.
#[test]
fn a_short_guid_answers_with_every_digit_it_has() {
    assert_eq!(the_last_four_digits(guid("17")), "17");
}

/// A printed line's placeholders are what a blind paste carries,
/// and a value read from a record is not.
#[test]
fn a_placeholder_is_recognized_in_either_flag() {
    assert!(carries_a_placeholder("<slot>"));
    assert!(carries_a_placeholder("<the last 4 decimal digits of cloneGuid=>"));
    assert!(!carries_a_placeholder("3102"));
    assert!(!carries_a_placeholder("fix-flaky-test"));
}

/// The usage line names the wrapper's own command and shows both values as placeholders,
/// so the line a person copies carries neither value.
#[test]
fn the_usage_line_carries_no_value() {
    let usage: String = the_usage_line();
    assert!(usage.starts_with(SETTLING_COMMAND_WORDS), "{usage}");
    assert!(usage.contains(CONFIRM_REMOVAL_FLAG), "{usage}");
    assert!(usage.contains(CONFIRM_GUID_FLAG), "{usage}");
    assert!(carries_a_placeholder(usage.as_str()), "{usage}");
}
