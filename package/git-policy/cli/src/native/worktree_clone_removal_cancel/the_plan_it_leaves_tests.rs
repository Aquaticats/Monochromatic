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
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::{SettledSnapshot, TheDestroyOutcomes};
use std::path::{Path, PathBuf};

/// The removal fixture's common directory,
/// with a plan written where the removal writes one.
fn a_plan_that_stands(name: &str, plan: &DestructionPlan) -> (PathBuf, PathBuf) {
    let common: PathBuf = fixture(&format!("cancel-plan-{name}"));
    let folder: PathBuf = plans_folder(common.as_path());
    crate::private_storage::create_private_directory_all(folder.as_path()).expect("the plan folder");
    let path: PathBuf = folder.join(plan.facts.clone_guid.file_name());
    std::fs::write(path.as_path(), plan.encode().expect("the plan encodes")).expect("the plan file");
    return (common, path);
}

/// One folder identity,
/// written from its own grammar so a control needs no folder to answer it.
fn an_identity() -> crate::worktree_clone_binding_record::binding_encoding::FolderIdentity {
    return crate::worktree_clone_binding_record::binding_encoding::FolderIdentity::decode("2:3:4").expect("the grammar reads one identity");
}


/// The record the plan names,
/// built from the sample identities so a control changes one thing.
fn recorded() -> RecordedBinding {
    return RecordedBinding {
        slot: String::from("fix-flaky-test"),
        slot_path: PathBuf::from("/repo/.claude/worktrees/fix-flaky-test"),
        pool: String::from("tank"),
        pool_guid: guid(1_234_567_890_123_456_789),
        clone_dataset: String::from("tank/wt/fix-flaky-test"),
        clone_guid: guid(7_828_944_690_444_299_989),
        clone_createtxg: 17,
        origin_dataset: String::from("tank/mono-checkout"),
        origin_dataset_guid: guid(4_815_162_342_109_876_543),
        origin_snapshot: String::from("tank/mono-checkout@origin"),
        origin_snapshot_guid: guid(11_050_373_805_378_911_967),
        origin_snapshot_createtxg: 15,
        registration_id: Vec::from("fix-flaky-test".as_bytes()),
        registration_identity: an_identity(),
    };
}

/// Nothing was destroyed,
/// which is the state a cancellation may remove a plan in.
fn nothing_destroyed() -> TheDestroyOutcomes {
    return TheDestroyOutcomes {
        destroyed: Vec::new(),
        everything: sample_snapshots().iter().map(|snapshot| return SettledSnapshot { name: snapshot.name.clone(), destroyed: false }).collect(),
    };
}

#[test]
fn no_plan_is_no_action() {
    let common: PathBuf = fixture("cancel-plan-none");
    assert_eq!(
        the_plan_a_cancellation_leaves(common.as_path(), &recorded(), None, &nothing_destroyed()).expect("no plan is no action"),
        ThePlansFate::NoPlan
    );
}

#[test]
fn a_plan_that_witnessed_nothing_leaves() {
    let plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    let (common, path): (PathBuf, PathBuf) = a_plan_that_stands("empty", &plan);
    assert!(path.exists(), "positive control: the plan stands before the action");
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &recorded(), Some(&plan), &nothing_destroyed()).expect("the plan leaves");
    assert_eq!(fate, ThePlansFate::Removed { path: path.clone() });
    assert!(!path.exists(), "the plan that witnessed nothing is gone");
}

#[test]
fn a_plan_that_records_a_destroyed_snapshot_stays() {
    let mut plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    let snapshot = plan.named_snapshots()[0].clone();
    plan.record_line(PlanAttempt { snapshot, attempt: 1, outcome: SnapshotOutcome::Destroyed, errno: None }).expect("the destroyed line");
    let (common, path): (PathBuf, PathBuf) = a_plan_that_stands("destroyed", &plan);
    let outcomes = TheDestroyOutcomes {
        destroyed: vec![plan.named_snapshots()[0].name.clone()],
        everything: vec![SettledSnapshot { name: plan.named_snapshots()[0].name.clone(), destroyed: true }],
    };
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &recorded(), Some(&plan), &outcomes).expect("the plan stays");
    assert!(fate.stands());
    assert!(path.exists(), "the loss record stays until the clone's next removal reads it");
    match fate {
        ThePlansFate::Kept { destroyed, .. } => assert_eq!(destroyed.len(), 1),
        other => panic!("a plan with a destroyed line is kept, not {other:?}"),
    }
}

#[test]
fn a_plan_whose_head_is_not_present_stays() {
    let mut plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    plan.head = HeadWord::Requested;
    let (common, path): (PathBuf, PathBuf) = a_plan_that_stands("head-requested", &plan);
    let fate: ThePlansFate = the_plan_a_cancellation_leaves(common.as_path(), &recorded(), Some(&plan), &nothing_destroyed()).expect("the plan stays");
    assert!(fate.stands());
    assert!(path.exists(), "a plan whose head is not present is never removed by a cancellation");
}

#[test]
fn a_plan_of_another_clone_refuses_and_is_left() {
    let mut plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    plan.facts.clone_guid = guid(42);
    let (common, _path): (PathBuf, PathBuf) = a_plan_that_stands("another-clone", &plan);
    match the_plan_a_cancellation_leaves(common.as_path(), &recorded(), Some(&plan), &nothing_destroyed()).expect_err("a plan of another clone refuses") {
        CancelRefusal::ThePlanIsOfAnotherClone { plan_guid, recorded_guid, .. } => {
            assert_eq!(plan_guid, guid(42).file_name());
            assert_eq!(recorded_guid, guid(7_828_944_690_444_299_989).file_name());
        }
        other => panic!("a plan of another clone refuses as one, not {other:?}"),
    }
}

#[test]
fn the_plans_path_is_the_clones_guid_under_the_plan_folder() {
    let plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    let (common, path): (PathBuf, PathBuf) = a_plan_that_stands("path", &plan);
    assert_eq!(path, plans_folder(common.as_path()).join(guid(7_828_944_690_444_299_989).file_name()));
    assert_eq!(Path::new(path.as_path()).file_name().and_then(|name| return name.to_str()), Some("7828944690444299989"));
}
