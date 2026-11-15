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

//region The ninth review: a publisher that cannot take the guard

/// The sentence the plan file is expected to hold after the interleaving, or the empty text.
fn plan_text(rig: &Rig) -> String {
    return fs::read_to_string(&rig.scope.plan_file).expect("read the plan");
}

#[test]
fn a_publisher_that_cannot_take_the_guard_leaves_the_other_publisher_s_finished_update() {
    use std::sync::{Arc, Mutex};

    let rig = Rig::new();
    let before: Option<Vec<u8>> = read_optional(&rig.scope.plan_file).expect("read the starting plan");
    let expected: super::publish::Revision = revision_of(&before);
    let base: String = String::from_utf8(before.expect("the sample plan stands there")).expect("the plan is text");
    // A third command holds the guard, so neither publisher can take it.
    let held = acquire_guard(&rig.scope).expect("open the guard").expect("the guard is free");
    let other: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let other_in_hook: Arc<Mutex<Vec<String>>> = Arc::clone(&other);
    let scope_in_hook = rig.scope.clone();
    let text_in_hook: String = format!("{base}\nentry-from-the-other-publisher\n");
    let expected_in_hook: super::publish::Revision = expected.clone();
    // The other publisher finishes its whole publish between this one's revision read and its write.
    super::publish::seam::set_after_the_revision_read(Some(Box::new(move || {
        let answer = publish_plan(&scope_in_hook, &expected_in_hook, &text_in_hook);
        other_in_hook.lock().expect("record the other publisher").push(format!("{answer:?}"));
    })));

    let mine = publish_plan(&rig.scope, &expected, &format!("{base}\nentry-from-this-publisher\n"));

    super::publish::seam::set_after_the_revision_read(None);
    drop(held);
    let text: String = plan_text(&rig);
    let this_publisher_wrote: bool = text.contains("entry-from-this-publisher");
    let other_wrote: bool = text.contains("entry-from-the-other-publisher");
    assert!(
        !(this_publisher_wrote && !other_wrote),
        "the other publisher's finished update is gone: this one wrote over it; mine: {mine:?}, the other: {:?}, plan: {text}",
        other.lock().expect("read the other publisher")
    );
}

#[test]
fn a_lock_publisher_that_cannot_take_the_guard_writes_nothing_over_a_finished_lock() {
    let rig = Rig::new();
    let expected: super::publish::Revisions = read_revisions(&rig.scope).expect("read the starting revisions");
    let before: String = fs::read_to_string(&rig.scope.lock_file).expect("read the lock");
    let held = acquire_guard(&rig.scope).expect("open the guard").expect("the guard is free");
    let text: &str = "{\"schemaVersion\":1,\"generatedFrom\":\"toolchain.plan.jsonc\",\"tools\":{}}\n";

    let mine = publish_lock(&rig.scope, &expected, text);

    drop(held);
    assert!(mine.is_ok(), "a publisher that cannot take the guard must not fail the command: {mine:?}");
    let after: String = fs::read_to_string(&rig.scope.lock_file).expect("read the lock again");
    assert!(
        after == before,
        "a publisher that could not take the guard replaced the lock a finished publisher wrote: {} bytes became {}",
        before.len(),
        after.len()
    );
}

//endregion
