//! What:
//!  Controls of the two mark counts and the sentence that says what the next destroy does,
//!  for one clone and no hold,
//!  another clone,
//!  a user hold,
//!  and a hold count the reply does not carry.
//! Why:
//!  The two numbers are what separate a loss from a no-op for a person,
//!  and the kernel's condition is one clone and no user hold
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Reading the mark" and veto V8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(whatTheDestroyDoes(1, 0)).toContain('in the same transaction');
//! ```

use super::super::origin_mark::UserHolds;
use super::{
    holds_phrase,
    mark_numbers,
    noun,
    what_the_destroy_does,
};

#[test]
fn one_clone_and_no_hold_is_the_reap_the_kernel_performs() {
    let sentence: String = what_the_destroy_does(1, &UserHolds::Counted(0));
    assert_eq!(
        sentence,
        "so destroying this clone destroys that snapshot in the same transaction"
    );
}

#[test]
fn a_single_user_hold_keeps_the_snapshot() {
    assert_eq!(
        what_the_destroy_does(1, &UserHolds::Counted(1)),
        "so that snapshot stands after this removal, because one user hold pins it"
    );
}

#[test]
fn several_user_holds_keep_the_snapshot() {
    assert_eq!(
        what_the_destroy_does(1, &UserHolds::Counted(2)),
        "so that snapshot stands after this removal, because 2 user holds pin it"
    );
}

#[test]
fn a_second_clone_keeps_the_snapshot() {
    assert_eq!(
        what_the_destroy_does(2, &UserHolds::Counted(0)),
        "so that snapshot stands after this removal, because another clone of it is in place"
    );
}

#[test]
fn a_second_clone_and_a_hold_are_both_named() {
    assert_eq!(
        what_the_destroy_does(2, &UserHolds::Counted(1)),
        "so that snapshot stands after this removal, because another clone of it is in place and one user hold pins it"
    );
}

#[test]
fn no_clone_at_all_leaves_the_snapshot_standing() {
    assert_eq!(
        what_the_destroy_does(0, &UserHolds::Counted(0)),
        "so that snapshot stands after this removal, because no clone of it is here to destroy"
    );
}

#[test]
fn a_hold_count_that_was_not_read_makes_the_outcome_unknown_rather_than_assumed() {
    let unreadable: UserHolds = UserHolds::Unreadable {
        said: String::from("`userrefs` is not in the reply"),
    };
    assert_eq!(
        what_the_destroy_does(1, &unreadable),
        "so whether destroying this clone would destroy that snapshot cannot be told from this reply"
    );
    assert_eq!(
        what_the_destroy_does(3, &unreadable),
        "so whether destroying this clone would destroy that snapshot cannot be told from this reply"
    );
}

#[test]
fn the_two_counts_are_printed_with_the_nouns_that_agree_with_them() {
    assert_eq!(
        mark_numbers(1, &UserHolds::Counted(0)),
        "the kernel reports 1 clone of it and 0 user holds"
    );
    assert_eq!(
        mark_numbers(2, &UserHolds::Counted(1)),
        "the kernel reports 2 clones of it and 1 user hold"
    );
    let unreadable: UserHolds = UserHolds::Unreadable {
        said: String::from("`userrefs` is not in the reply"),
    };
    assert_eq!(
        mark_numbers(1, &unreadable),
        "the kernel reports 1 clone of it, and its hold count could not be read (`userrefs` is not in the reply)"
    );
}

#[test]
fn one_noun_agrees_with_its_count() {
    assert_eq!(noun(1, "clone", "clones"), "1 clone");
    assert_eq!(noun(0, "clone", "clones"), "0 clones");
    assert_eq!(noun(1, "user hold", "user holds"), "1 user hold");
    assert_eq!(noun(5, "user hold", "user holds"), "5 user holds");
    assert_eq!(holds_phrase(1), "one user hold pins it");
    assert_eq!(holds_phrase(2), "2 user holds pin it");
}
