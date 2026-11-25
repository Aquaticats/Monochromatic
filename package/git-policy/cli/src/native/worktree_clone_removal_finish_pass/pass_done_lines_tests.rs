//! What:
//!  The controls of the pass's finished lines:
//!  a settle's answer read as words,
//!  a finished removal naming what went in its order,
//!  and a plan removed after a reused name.
//! Why:
//!  The line is the only trace of a tail no person's command ran,
//!  so it has to say which record went and whether a registration moved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theSettlementOf(answer)).toContain('settled');
//! ```

use crate::worktree_clone_removal_finish_pass::pass_done_lines::{the_plan_was_removed_after_a_reuse, the_plan_was_removed_beside_a_new_binding, the_removal_was_finished, the_settlement_of};
use crate::worktree_clone_removal_proof_and_plan::removal_settle::SettleAnswer;
use std::path::Path;

/// One settle's answer with the fields a control names.
fn an_answer(wrote_the_lost_lock: bool, released_the_lock: bool, removed_the_registration: bool) -> SettleAnswer {
    return SettleAnswer { wrote_the_lost_lock, released_the_lock, removed_the_registration, removed_the_underlay: false, lines: Vec::new() };
}

#[test]
fn a_settle_that_removed_the_registration_says_so() {
    assert!(the_settlement_of(&an_answer(false, true, true)).contains("the registration and the slot's pointer were settled"));
    assert!(the_settlement_of(&an_answer(true, true, true)).contains("written again and released"));
    assert!(the_settlement_of(&an_answer(false, false, true)).contains("were settled"));
}

#[test]
fn a_settle_that_removed_no_registration_says_that_instead() {
    assert!(the_settlement_of(&an_answer(false, false, false)).contains("no registration of this plan stood"));
    assert!(the_settlement_of(&an_answer(false, true, false)).contains("the managed lock was released"));
}

#[test]
fn a_finished_removal_names_every_file_it_removed_in_order() {
    let files: Vec<String> = vec![String::from("fix-flaky-test.origin-inventory"), String::from("fix-flaky-test.clone-inventory"), String::from("fix-flaky-test.binding")];
    let line: String = the_removal_was_finished(Path::new("/repo/plan"), "fix-flaky-test", files.as_slice(), "no registration of this plan stood");
    assert!(line.contains("fix-flaky-test.origin-inventory, fix-flaky-test.clone-inventory, fix-flaky-test.binding"), "the line names the files in the order they went: {line}");
    assert!(line.contains("removed the plan /repo/plan last"), "the line says the plan went last: {line}");
}

#[test]
fn a_finished_removal_with_no_binding_file_says_the_binding_stood_nowhere() {
    let line: String = the_removal_was_finished(Path::new("/repo/plan"), "fix-flaky-test", Vec::new().as_slice(), "no registration of this plan stood");
    assert!(line.contains("no record of the binding stood"), "the line says no binding was there: {line}");
}

#[test]
fn the_removal_after_a_reused_name_names_both_guids() {
    let line: String = the_plan_was_removed_after_a_reuse(Path::new("/repo/plan"), "fix-flaky-test", "tank/wt/fix-flaky-test", "7828944690444299989", "1");
    assert!(line.contains("GUID 1") && line.contains("7828944690444299989"), "the line names the live GUID and the recorded one: {line}");
    assert!(line.contains("left untouched"), "the line says what it did not touch: {line}");
}

#[test]
fn a_plan_removed_beside_a_new_binding_names_the_new_clone() {
    let line: String = the_plan_was_removed_beside_a_new_binding(Path::new("/repo/plan"), "fix-flaky-test");
    assert!(line.contains("names another clone"), "the line says the binding names another clone: {line}");
}
