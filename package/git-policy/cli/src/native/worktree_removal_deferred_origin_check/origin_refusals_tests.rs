//! What:
//!  Controls of the small vocabulary a refusal rests on:
//!  each plan field's words,
//!  each object's words,
//!  each form's name,
//!  and the one sentence the record rule prints.
//! Why:
//!  A message that named a field in the plan grammar's hyphens would send a person to a line
//!  they cannot find,
//!  and the record rule is the project's answer of 2026-11-22,
//!  so both are compared here rather than through a whole message
//!  (`doc/decision/cli-git-zfs-deferred-destroy-refusals-and-the-early-look.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(PlanField.CloneGuid.words()).toBe("the clone's dataset GUID");
//! ```

use super::{
    JudgedObject,
    PlanField,
    RefusalForm,
    WhatARefusalLeaves,
};

#[test]
fn every_plan_field_has_the_words_a_person_reads_on_the_plan() {
    let fields: Vec<PlanField> = vec![
        PlanField::CloneName,
        PlanField::CloneGuid,
        PlanField::OriginDatasetName,
        PlanField::OriginDatasetGuid,
        PlanField::OriginSnapshotName,
        PlanField::OriginSnapshotGuid,
        PlanField::OriginSnapshotCreatetxg,
    ];
    let words: Vec<&str> = fields.iter().map(|field| return field.words()).collect();
    assert_eq!(words.len(), 7, "every field the tuple needs has words");
    assert!(
        words.iter().all(|word| return !word.contains('-')),
        "no field is named in the grammar's hyphens: {words:?}"
    );
    assert!(
        words.iter().all(|word| return !word.is_empty()),
        "no field has empty words"
    );
}

#[test]
fn every_object_a_cause_names_has_its_words() {
    assert_eq!(JudgedObject::Clone.words(), "the planned clone");
    assert_eq!(JudgedObject::OriginDataset.words(), "the origin dataset");
    assert_eq!(JudgedObject::OriginSnapshot.words(), "the origin snapshot");
}

#[test]
fn every_form_has_its_own_name() {
    assert_eq!(RefusalForm::NothingTakenDown.name(), "nothing-taken-down");
    assert_eq!(
        RefusalForm::PendingAfterTheUnmount.name(),
        "pending-after-the-unmount"
    );
    assert_eq!(RefusalForm::NothingCompared.name(), "nothing-compared");
}

#[test]
fn the_record_rule_says_what_the_project_decided() {
    assert_eq!(
        WhatARefusalLeaves::NothingPending.sentence("fix-flaky-test"),
        "nothing is pending and there is nothing to cancel"
    );
    let kept: String = WhatARefusalLeaves::PendingWithItsRecord.sentence("fix-flaky-test");
    assert!(
        kept.contains("the removal stays pending with its own record"),
        "the intent and plan stand: {kept}"
    );
    assert!(
        kept.contains("the clone stays unmounted"),
        "the clone stays unmounted: {kept}"
    );
    assert!(
        kept.contains("`cancel-worktree-removal fix-flaky-test`"),
        "the slot's cancel command is named: {kept}"
    );
    assert!(
        kept.contains("for a removal that is not forced"),
        "the cancel sentence is scoped to an unforced removal: {kept}"
    );
    assert!(
        kept.contains("reports any snapshot already destroyed"),
        "a cancel reports the loss: {kept}"
    );
}
