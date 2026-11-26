//! What:
//!  Control of a plan that does not carry one of the fields the check compares:
//!  the refusal names the field,
//!  the plan's own file and the slot,
//!  and no object is read.
//! Why:
//!  A plan missing a field is a refusal rather than a check with a missing expectation,
//!  because nothing proves which object the removal planned, and nothing is deleted
//!  (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Where the planned identities come from", control 9).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fields = { ...read, cloneGuid: undefined };
//! expect(() => PlannedOrigin.require('fix-flaky-test', fields)).toThrow('the clone's dataset GUID');
//! ```

use super::super::fixtures::{
    CLONE,
    PLAN_HOME,
    SLOT,
    fields,
};
use super::super::origin_refusal::OriginRefusal;
use super::super::origin_refusals::{
    PlanField,
    RefusalForm,
    WhatARefusalLeaves,
};
use super::PlannedOrigin;
use std::path::PathBuf;

#[test]
fn a_whole_plan_gives_every_identity_the_check_compares() {
    let planned: PlannedOrigin =
        PlannedOrigin::require(SLOT, fields()).expect("a whole plan gives the tuple");
    assert_eq!(planned.slot, SLOT);
    assert_eq!(planned.clone_name, CLONE);
    assert_eq!(
        planned.origin_snapshot_name,
        super::super::fixtures::ORIGIN_SNAPSHOT
    );
}

#[test]
fn each_missing_field_names_itself_and_the_plans_own_file() {
    let cases: Vec<(PlanField, super::super::planned_origin::PlannedOriginFields)> = vec![
        (
            PlanField::CloneName,
            super::super::planned_origin::PlannedOriginFields {
                clone_name: None,
                ..fields()
            },
        ),
        (
            PlanField::CloneGuid,
            super::super::planned_origin::PlannedOriginFields {
                clone_guid: None,
                ..fields()
            },
        ),
        (
            PlanField::OriginDatasetName,
            super::super::planned_origin::PlannedOriginFields {
                origin_dataset_name: None,
                ..fields()
            },
        ),
        (
            PlanField::OriginDatasetGuid,
            super::super::planned_origin::PlannedOriginFields {
                origin_dataset_guid: None,
                ..fields()
            },
        ),
        (
            PlanField::OriginSnapshotName,
            super::super::planned_origin::PlannedOriginFields {
                origin_snapshot_name: None,
                ..fields()
            },
        ),
        (
            PlanField::OriginSnapshotGuid,
            super::super::planned_origin::PlannedOriginFields {
                origin_snapshot_guid: None,
                ..fields()
            },
        ),
        (
            PlanField::OriginSnapshotCreatetxg,
            super::super::planned_origin::PlannedOriginFields {
                origin_snapshot_createtxg: None,
                ..fields()
            },
        ),
    ];
    for (field, incomplete) in cases {
        let missing: super::super::planned_origin::MissingField =
            PlannedOrigin::require(SLOT, incomplete)
                .expect_err("the plan lacks the field the control removed");
        assert_eq!(
            missing.field, field,
            "the answer names the field the control removed"
        );
        match OriginRefusal::from(missing) {
            OriginRefusal::PlanIncomplete {
                slot,
                field: named,
                plan_home,
            } => {
                assert_eq!(
                    named, field,
                    "the refusal names the field the control removed"
                );
                assert_eq!(slot, SLOT);
                assert_eq!(
                    plan_home,
                    PathBuf::from(PLAN_HOME),
                    "the refusal names the plan it read"
                );
            }
            other => panic!("a missing {} must refuse, not {other:?}", field.words()),
        }
    }
}

#[test]
fn an_incomplete_plan_is_a_refusal_of_the_form_that_compared_nothing() {
    let incomplete: super::super::planned_origin::PlannedOriginFields =
        super::super::planned_origin::PlannedOriginFields {
            clone_name: None,
            ..fields()
        };
    let refusal: OriginRefusal = OriginRefusal::from(
        PlannedOrigin::require(SLOT, incomplete).expect_err("the plan is incomplete"),
    );
    assert_eq!(refusal.form(), RefusalForm::NothingCompared);
    assert_eq!(
        refusal.what_it_leaves(),
        WhatARefusalLeaves::PendingWithItsRecord
    );
    assert_eq!(refusal.step(), "plan");
    assert_eq!(
        refusal.clone_name(),
        None,
        "no plan field means no clone the refusal can name"
    );
    assert_eq!(refusal.slot(), SLOT);
}
