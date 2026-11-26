//! What:
//!  Controls of each cause's own answers:
//!  the step word a receipt files it under,
//!  the form of message it takes,
//!  the record rule that follows from its point,
//!  and the clone and slot it names.
//! Why:
//!  A receipt,
//!  a control and `doctor` must agree on which read refused without comparing sentences,
//!  so every cause is compared against the table the design gives
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "The refusals").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(cause(CLONE_GONE).step()).toBe('clone');
//! ```

use super::super::fixtures::{
    CLONE,
    SLOT,
    every_cause,
};
use super::super::origin_refusals::{
    RefusalForm,
    WhatARefusalLeaves,
};
use super::OriginRefusal;

/// The step and the form one cause answers,
/// for the controls to compare.
fn step_and_form(label: &str) -> (&'static str, RefusalForm, WhatARefusalLeaves) {
    let found: Vec<(&'static str, OriginRefusal)> = every_cause();
    let Some((_label, cause)) = found.iter().find(|(named, _)| return *named == label) else {
        panic!("the fixture list holds no cause called {label}");
    };
    return (cause.step(), cause.form(), cause.what_it_leaves());
}

#[test]
fn each_cause_answers_with_its_own_step_and_form() {
    let expected: Vec<(&str, &str, RefusalForm, WhatARefusalLeaves)> = vec![
        (
            "plan-incomplete",
            "plan",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "marked-before-the-unmount",
            "origin-mark",
            RefusalForm::NothingTakenDown,
            WhatARefusalLeaves::NothingPending,
        ),
        (
            "marked-at-a-check",
            "origin-mark",
            RefusalForm::PendingAfterTheUnmount,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "unreadable-mark-before-the-unmount",
            "origin-mark",
            RefusalForm::NothingTakenDown,
            WhatARefusalLeaves::NothingPending,
        ),
        (
            "unreadable-mark-at-a-check",
            "origin-mark",
            RefusalForm::PendingAfterTheUnmount,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-gone",
            "clone",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-not-a-filesystem",
            "clone-type",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-is-a-snapshot",
            "clone-type",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-not-the-planned-one",
            "clone-guid",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-has-no-origin",
            "clone-origin",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "clone-origin-differs",
            "clone-origin",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-dataset-gone",
            "origin-dataset",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-dataset-not-a-filesystem",
            "origin-dataset-type",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-dataset-not-the-planned-one",
            "origin-dataset-guid",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-snapshot-gone",
            "origin-snapshot",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-is-not-a-snapshot",
            "origin-snapshot-type",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-snapshot-not-the-planned-one",
            "origin-snapshot-guid",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "origin-snapshot-createtxg-differs",
            "origin-snapshot-createtxg",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "read-failed-on-the-clone",
            "read",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "read-failed-on-the-origin-dataset",
            "read",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
        (
            "read-failed-on-the-origin-snapshot",
            "read",
            RefusalForm::NothingCompared,
            WhatARefusalLeaves::PendingWithItsRecord,
        ),
    ];
    let labels: Vec<&str> = every_cause()
        .iter()
        .map(|(label, _)| return *label)
        .collect();
    assert_eq!(
        labels.len(),
        expected.len(),
        "every cause in the fixture list is in this table and no other"
    );
    for (label, step, form, leaves) in expected {
        assert_eq!(
            step_and_form(label),
            (step, form, leaves),
            "the cause {label}"
        );
    }
}

#[test]
fn every_cause_has_a_name_of_its_own_and_none_repeats() {
    let mut names: Vec<&'static str> = Vec::new();
    for (label, cause) in every_cause() {
        let name: &'static str = cause.cause_name();
        assert!(!name.is_empty(), "the cause {label} has a name");
        assert!(
            name.bytes().all(|byte| return byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'-'),
            "the cause {label} is named in lowercase words and hyphens: {name}"
        );
        if !names.contains(&name) {
            names.push(name);
        }
    }
    let distinct: usize = names.len();
    assert_eq!(
        distinct, 17,
        "one name per cause and no cause shares one, found {names:?}"
    );
}

#[test]
fn a_check_names_the_clone_and_the_slot_and_the_look_names_both_too() {
    for (label, cause) in every_cause() {
        if label == "plan-incomplete" {
            assert_eq!(
                cause.clone_name(),
                None,
                "a plan with no clone field names no clone"
            );
            assert_eq!(
                cause.slot(),
                SLOT,
                "it still names the slot the removal was asked for"
            );
            continue;
        }
        assert_eq!(
            cause.clone_name(),
            Some(CLONE),
            "the cause {label} names the clone"
        );
        assert_eq!(cause.slot(), SLOT, "the cause {label} names the slot");
    }
}

#[test]
fn no_refusal_claims_a_state_its_point_did_not_read() {
    for (label, cause) in every_cause() {
        let sentence: String = cause.what_it_leaves().sentence(cause.slot());
        match cause.form() {
            RefusalForm::NothingTakenDown => {
                assert!(
                    !sentence.contains("unmounted"),
                    "the look reads no mount, so it claims none: {label} says {sentence}"
                );
                assert!(
                    !sentence.contains("the removal stays pending"),
                    "the look leaves nothing pending: {label} says {sentence}"
                );
            }
            RefusalForm::PendingAfterTheUnmount | RefusalForm::NothingCompared => {
                assert!(
                    sentence.contains("the removal stays pending"),
                    "a check keeps the removal pending: {label} says {sentence}"
                );
                assert!(
                    sentence.contains("the clone stays unmounted"),
                    "a check runs after the unmount: {label} says {sentence}"
                );
            }
        }
    }
}
