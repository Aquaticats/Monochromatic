//! Unit tests of publishing the plan and the lock:
//! a command that holds the scope's guard must not make either publish fail.

/// Imports what this module uses from `std::fs`.
use std::fs;

/// Imports `Rig` from `super::fakes`.
use super::fakes::Rig;
/// Imports what this module uses from `super::publish`.
use super::publish::{acquire_guard, publish_lock, publish_plan, read_optional, read_revisions, revision_of};

#[test]
fn a_held_scope_guard_does_not_fail_a_lock_publish() {
    let rig = Rig::new();
    let expected: super::publish::Revisions = read_revisions(&rig.scope).expect("read the starting revisions");
    // Another command is inside its publish, or stopped inside it.
    let held = acquire_guard(&rig.scope).expect("open the guard").expect("the guard is free");
    let text: &str = "{\"schemaVersion\":1,\"generatedFrom\":\"toolchain.plan.jsonc\",\"tools\":{}}\n";

    let outcome = publish_lock(&rig.scope, &expected, text);

    drop(held);
    assert!(outcome.is_ok(), "a guard holder must not fail a lock publish: {outcome:?}");
    assert_eq!(fs::read_to_string(&rig.scope.lock_file).expect("read the lock"), text, "the lock is the text published");
}

#[test]
fn a_held_scope_guard_does_not_fail_a_plan_publish() {
    let rig = Rig::new();
    let before: Option<Vec<u8>> = read_optional(&rig.scope.plan_file).expect("read the plan");
    let mut text: String = String::from_utf8(before.clone().expect("a plan stands there")).expect("the plan is text");
    text.push('\n');
    let expected: super::publish::Revision = revision_of(&before);
    let held = acquire_guard(&rig.scope).expect("open the guard").expect("the guard is free");

    let outcome = publish_plan(&rig.scope, &expected, &text);

    drop(held);
    assert!(outcome.is_ok(), "a guard holder must not fail a plan publish: {outcome:?}");
    assert_eq!(fs::read_to_string(&rig.scope.plan_file).expect("read the plan"), text, "the plan is the text published");
}
