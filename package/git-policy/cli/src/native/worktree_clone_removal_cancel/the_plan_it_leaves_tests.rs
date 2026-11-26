//! What:
//!  Controls for the one plan action:
//!  a plan that witnessed nothing leaves,
//!  and a plan that records a destroyed snapshot stays as the loss record.
//! Why:
//!  The plan design gives both the cancelling removal and the cancel command one rule:
//!  remove an all-`planned`,
//!  `head=present` plan,
//!  and remove no other
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "A cancellation,
//! and the plan it leaves").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(thePlanACancellationLeaves(...)).toEqual({ kind: 'removed', path });
//! ```

use super::{ThePlansFate, the_plan_a_cancellation_leaves};
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{guid, sample_facts, sample_snapshots};
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanAttempt};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{HeadWord, SnapshotOutcome};
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{a_plan_that_stands, a_recorded_binding};
use std::path::PathBuf;

/// The plan a removal wrote before its first request.
fn the_plan() -> DestructionPlan {
    return DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
}

#[test]
fn no_plan_is_no_action() {
    let common: PathBuf = fixture("cancel-plan-none");
    assert_eq!(
        the_plan_a_cancellation_leaves(common.as_path(), &a_recorded_binding(), None, &[]).expect("no plan is no action"),
        ThePlansFate::NoPlan
    );
}

#[test]
fn a_plan_that_witnessed_nothing_leaves() {
    let plan: DestructionPlan = the_plan();
    let common: PathBuf = fixture("cancel-plan-empty");
    let path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    assert!(path.exists(), "positive control: the plan stands before the action");
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &a_recorded_binding(), Some(&plan), &[]).expect("the plan leaves");
    assert_eq!(fate, ThePlansFate::Removed { path: path.clone() });
    assert!(!path.exists(), "the plan that witnessed nothing is gone");
}

#[test]
fn a_plan_that_records_a_destroyed_snapshot_stays() {
    let mut plan: DestructionPlan = the_plan();
    let snapshot = plan.named_snapshots()[0].clone();
    let destroyed: Vec<String> = vec![snapshot.name.clone()];
    plan.record_line(PlanAttempt { snapshot, attempt: 1, outcome: SnapshotOutcome::Destroyed, errno: None }).expect("the destroyed line");
    let common: PathBuf = fixture("cancel-plan-destroyed");
    let path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &a_recorded_binding(), Some(&plan), destroyed.as_slice()).expect("the plan stays");
    assert!(fate.stands());
    assert!(path.exists(), "the loss record stays until the clone's next removal reads it");
    assert_eq!(fate, ThePlansFate::Kept { path });
}

#[test]
fn a_plan_whose_head_is_not_present_stays() {
    let mut plan: DestructionPlan = the_plan();
    plan.head = HeadWord::Requested;
    let common: PathBuf = fixture("cancel-plan-head-requested");
    let path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &a_recorded_binding(), Some(&plan), &[]).expect("the plan stays");
    assert!(fate.stands());
    assert!(path.exists(), "a plan whose head is not present is never removed by a cancellation");
}

#[test]
fn a_plan_of_another_clone_refuses_and_is_left() {
    let mut plan: DestructionPlan = the_plan();
    plan.facts.clone_guid = guid(42);
    let common: PathBuf = fixture("cancel-plan-another-clone");
    let path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    match the_plan_a_cancellation_leaves(common.as_path(), &a_recorded_binding(), Some(&plan), &[]).expect_err("a plan of another clone refuses") {
        CancelRefusal::ThePlanIsOfAnotherClone { plan_guid, recorded_guid, .. } => {
            assert_eq!(plan_guid, guid(42).file_name());
            assert_eq!(recorded_guid, guid(7_828_944_690_444_299_989).file_name());
        }
        other => panic!("a plan of another clone refuses as one, not {other:?}"),
    }
    assert!(path.exists(), "a refused plan is left where it stands");
}

#[test]
fn the_plans_path_is_the_clones_guid_under_the_plan_folder() {
    let plan: DestructionPlan = the_plan();
    let common: PathBuf = fixture("cancel-plan-path");
    let path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    assert_eq!(path, plans_folder(common.as_path()).join(guid(7_828_944_690_444_299_989).file_name()));
}
