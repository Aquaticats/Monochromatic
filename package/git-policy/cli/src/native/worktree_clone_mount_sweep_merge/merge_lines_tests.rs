//! What:
//!  Controls of the lines the device answers with:
//!  each names what it did,
//!  who covers a request it left standing,
//!  and which record could not be read.
//! Why:
//!  A held trigger and a trigger a second pass covered are the two states a
//!  person meets most,
//!  and neither may read as a fault
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "What the marks guarantee,
//! and what they do not";
//!  V7's first option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(extraPassLine()).toContain('one more pass');
//! ```

use super::{
    claim_not_released_line,
    device_unreadable_line,
    extra_pass_line,
    held_line,
    mark_left_standing_line,
    mark_left_with_no_pass_line,
    mark_not_removed_line,
    mark_unreadable_line,
};
use crate::single_owner_claim::BusyOwner;
use std::path::PathBuf;

/// A pass that holds the claim,
/// as the lines about it name it.
fn owner() -> BusyOwner {
    return BusyOwner {
        pid: 4242,
        path: PathBuf::from("/run/git-worktree-clone-mount-sweeps/run.7"),
    };
}

/// Every line the device writes is one line and names what it is about.
#[test]
fn every_device_line_is_one_line_about_its_own_subject() {
    let cases: &[(String, &str)] = &[
        (held_line(&owner()), "process 4242"),
        (
            mark_left_with_no_pass_line(&owner()),
            "another-pass-is-due stands",
        ),
        (extra_pass_line(), "one more pass"),
        (mark_left_standing_line(), "next trigger"),
        (
            device_unreadable_line("the run claim could not be read"),
            "the sweep merge device could not be read",
        ),
        (
            mark_unreadable_line("the mark could not be looked at"),
            "treated the mark as a request",
        ),
        (
            mark_not_removed_line("the mark could not be removed"),
            "next trigger's pass runs for a request",
        ),
        (
            claim_not_released_line("the run claim could not be released"),
            "the next trigger takes the claim",
        ),
    ];
    for (line, named) in cases {
        assert!(line.contains(named), "{named:?} is in the line: {line}");
        assert_eq!(line.lines().count(), 1, "one line: {line}");
    }
}

/// The line about a held claim names the file the other pass holds.
#[test]
fn a_held_line_names_the_file_the_other_pass_holds() {
    let line: String = held_line(&owner());
    assert!(line.contains("run.7"), "the claim's file is named: {line}");
    assert!(
        line.contains("ran no pass"),
        "the line says this trigger ran no pass: {line}"
    );
}
