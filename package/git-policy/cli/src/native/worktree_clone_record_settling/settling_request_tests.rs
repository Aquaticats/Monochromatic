//! Controls for the grammar of the settling command:
//!  the design's own line parses,
//!  the flags are accepted in either order,
//!  and each malformed form refuses with its own arm.

use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_request::{
    SettlingRequest,
    parse_settling_arguments,
};
use std::ffi::OsString;

/// One argument list built from the words a person types.
fn words(values: &[&str]) -> Vec<OsString> {
    return values
        .iter()
        .map(|value| return OsString::from(*value))
        .collect();
}

/// The record's own values answer with one request,
/// which is the line the design states.
#[test]
fn the_designs_line_parses() {
    let parsed = parse_settling_arguments(
        words(&[
            "add-cache-layer",
            "--confirm-removal-of",
            "add-cache-layer",
            "--confirm-clone-guid",
            "3102",
        ])
        .as_slice(),
    )
    .expect("the design's line parses");
    assert_eq!(
        parsed,
        SettlingRequest {
            slot: String::from("add-cache-layer"),
            confirmed_slot: String::from("add-cache-layer"),
            digits: String::from("3102")
        }
    );
}

/// The two flags may stand in either order after the slot,
/// because the grammar fixes their values and not their places.
#[test]
fn the_flags_stand_in_either_order() {
    let parsed = parse_settling_arguments(
        words(&[
            "topic",
            "--confirm-clone-guid",
            "3102",
            "--confirm-removal-of",
            "topic",
        ])
        .as_slice(),
    )
    .expect("either order parses");
    assert_eq!(parsed.slot, "topic");
    assert_eq!(parsed.digits, "3102");
}

/// A slot typed for another record refuses here,
/// before any file is read.
#[test]
fn the_two_slot_values_must_agree() {
    let refused = parse_settling_arguments(
        words(&[
            "topic",
            "--confirm-removal-of",
            "other",
            "--confirm-clone-guid",
            "3102",
        ])
        .as_slice(),
    )
    .expect_err("two slots refuse");
    assert_eq!(
        refused,
        SettlingStop::TheSlotIsNamedTwice {
            asked: String::from("topic"),
            confirmed: String::from("other")
        }
    );
    assert!(
        refused.sentence().contains("one slot in both places"),
        "{}",
        refused.sentence()
    );
}

/// A paste of the printed line leaves placeholders,
/// and each of the three places refuses it by name.
#[test]
fn a_pasted_placeholder_refuses_by_the_flag_it_stands_in() {
    let slot = parse_settling_arguments(
        words(&[
            "<slot>",
            "--confirm-removal-of",
            "<slot>",
            "--confirm-clone-guid",
            "3102",
        ])
        .as_slice(),
    )
    .expect_err("a placeholder slot refuses");
    assert!(
        matches!(slot, SettlingStop::PlaceholderLeft { .. }),
        "{slot:?}"
    );
    let digits = parse_settling_arguments(
        words(&[
            "topic",
            "--confirm-removal-of",
            "topic",
            "--confirm-clone-guid",
            "<the last 4 decimal digits of cloneGuid=>",
        ])
        .as_slice(),
    )
    .expect_err("a placeholder GUID refuses");
    assert!(
        matches!(digits, SettlingStop::PlaceholderLeft { .. }),
        "{digits:?}"
    );
}

/// A missing value,
/// a repeated flag,
/// an unknown option,
/// a second slot and a separator each refuse as usage,
/// and the sentence prints the line that works.
#[test]
fn each_malformed_form_refuses_as_usage() {
    let forms: [&[&str]; 6] = [
        &["topic", "--confirm-removal-of", "topic"],
        &[
            "topic",
            "--confirm-removal-of",
            "topic",
            "--confirm-removal-of",
            "topic",
            "--confirm-clone-guid",
            "3102",
        ],
        &[
            "topic",
            "--confirm-removal-of",
            "topic",
            "--confirm-clone-guid",
            "3102",
            "--settle",
        ],
        &[
            "topic",
            "other",
            "--confirm-removal-of",
            "topic",
            "--confirm-clone-guid",
            "3102",
        ],
        &[
            "topic",
            "--confirm-removal-of",
            "topic",
            "--confirm-clone-guid",
        ],
        &["--", "topic"],
    ];
    for form in forms {
        let refused =
            parse_settling_arguments(words(form).as_slice()).expect_err("the form refuses");
        assert!(
            matches!(refused, SettlingStop::Usage { .. }),
            "{form:?} answered {refused:?}"
        );
        assert!(
            refused.sentence().contains("--confirm-clone-guid"),
            "{}",
            refused.sentence()
        );
    }
}

/// A slot that holds a separator refuses rather than reaching a folder below the record home.
#[test]
fn a_slot_with_a_slash_refuses() {
    let refused = parse_settling_arguments(
        words(&[
            "some/folder",
            "--confirm-removal-of",
            "some/folder",
            "--confirm-clone-guid",
            "3102",
        ])
        .as_slice(),
    )
    .expect_err("a path refuses");
    assert!(matches!(refused, SettlingStop::Usage { .. }), "{refused:?}");
}
