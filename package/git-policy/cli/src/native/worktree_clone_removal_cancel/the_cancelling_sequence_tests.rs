//! What:
//!  Controls for the order the two actors share:
//!  the request is written first,
//!  the staged state is settled on the answer,
//!  and the plan is the last thing touched,
//!  so a refusal at one step leaves the steps after it undone.
//! Why:
//!  The design says the intent write is the first act after the block,
//!  because a standing `unmount` request must not be acted on again
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  "The arm blocked by a scan").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(steps(order)).toEqual(['request:mount:<nonce>', 'staged']);
//! ```

use super::{TheReversalRequest, reverse_the_removal};
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{guid, sample_facts, sample_snapshots};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{RecordingMountRequest, ScriptedStaged, an_order, steps};
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::TheDestroyOutcomes;
use crate::worktree_clone_removal_cancel::the_reversing_request::{MountAnswer, MountIdentity};
use crate::worktree_clone_removal_cancel::the_staged_state::StagedSettlement;
use std::path::PathBuf;

/// One folder identity,
/// written from its own grammar so a control needs no folder to answer it.
fn an_identity() -> crate::worktree_clone_binding_record::binding_encoding::FolderIdentity {
    return crate::worktree_clone_binding_record::binding_encoding::FolderIdentity::decode("2:3:4").expect("the grammar reads one identity");
}


/// The record the reversal names.
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

/// A whole answer about the fixture's clone.
fn an_answer() -> MountAnswer {
    return MountAnswer {
        dataset: guid(7_828_944_690_444_299_989),
        operation: MountOperation::Mount,
        nonce: crate::worktree_clone_mount_protocol::request_nonce::RequestNonce::parse("9c41000000000000000000000c410000").expect("a nonce"),
        outcome: ReceiptOutcome::Done,
        mount: Some(MountIdentity { mount_id: 7, source: String::from("tank/wt/fix-flaky-test"), canonical_path: String::from("/repo/.claude/worktrees/fix-flaky-test") }),
        failed_step: None,
        errno: None,
    };
}

/// Nothing was destroyed,
/// with an all-`planned` plan written where a removal writes one.
fn a_plan_that_witnessed_nothing(common: &std::path::Path) -> DestructionPlan {
    let plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    let folder: PathBuf = plans_folder(common);
    crate::private_storage::create_private_directory_all(folder.as_path()).expect("the plan folder");
    std::fs::write(folder.join(plan.facts.clone_guid.file_name()), plan.encode().expect("the plan encodes")).expect("the plan file");
    return plan;
}

/// The outcomes an all-planned plan settles to.
fn nothing_destroyed() -> TheDestroyOutcomes {
    return TheDestroyOutcomes { destroyed: Vec::new(), everything: Vec::new() };
}

#[test]
fn the_request_is_written_first_the_folder_settled_on_the_answer_and_the_plan_last() {
    let common: PathBuf = fixture("cancel-order");
    let a_record: RecordedBinding = recorded();
    let plan: DestructionPlan = a_plan_that_witnessed_nothing(common.as_path());
    let plan_path: PathBuf = plans_folder(common.as_path()).join(plan.facts.clone_guid.file_name());
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_answer()) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::Removed { record: true, claim: true } };
    let outcomes: TheDestroyOutcomes = nothing_destroyed();
    let done = reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_record,
        slot_path: std::path::Path::new("/repo/.claude/worktrees/fix-flaky-test"),
        plan: Some(&plan),
        outcomes: &outcomes,
        mount_request: &request,
        staged: &staged,
    })
    .expect("the reversal runs");
    let taken: Vec<String> = steps(&order);
    assert_eq!(taken.len(), 2, "{taken:?}");
    assert!(taken[0].starts_with("request:mount:"), "the request is the first act: {taken:?}");
    assert_eq!(taken[1], "staged", "the folder is settled on the answer: {taken:?}");
    assert_eq!(done.nonce, taken[0].trim_start_matches("request:mount:").to_string());
    assert_eq!(done.destroyed, Vec::<String>::new());
    assert!(!plan_path.exists(), "the plan that witnessed nothing left after the settlement");
}

#[test]
fn the_answer_carries_the_mount_the_request_and_the_plan_is_kept_when_a_snapshot_is_gone() {
    let common: PathBuf = fixture("cancel-order-kept");
    let a_record: RecordedBinding = recorded();
    let plan: DestructionPlan = DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
    let plan_path: PathBuf = plans_folder(common.as_path()).join(plan.facts.clone_guid.file_name());
    crate::private_storage::create_private_directory_all(plans_folder(common.as_path()).as_path()).expect("the plan folder");
    std::fs::write(plan_path.as_path(), plan.encode().expect("the plan encodes")).expect("the plan file");
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_answer()) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let outcomes: TheDestroyOutcomes = TheDestroyOutcomes {
        destroyed: vec![String::from("tank/wt/fix-flaky-test@sanoid-hourly")],
        everything: Vec::new(),
    };
    let done = reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_record,
        slot_path: std::path::Path::new("/repo/.claude/worktrees/fix-flaky-test"),
        plan: Some(&plan),
        outcomes: &outcomes,
        mount_request: &request,
        staged: &staged,
    })
    .expect("the reversal runs");
    assert_eq!(done.destroyed, vec![String::from("tank/wt/fix-flaky-test@sanoid-hourly")]);
    assert!(plan_path.exists(), "a plan that records a destroyed snapshot is the loss record and stays");
    assert_eq!(done.mount_id, 7);
}

#[test]
fn a_refused_answer_leaves_the_folder_and_the_plan_untouched() {
    let common: PathBuf = fixture("cancel-order-refused");
    let a_record: RecordedBinding = recorded();
    let plan: DestructionPlan = a_plan_that_witnessed_nothing(common.as_path());
    let plan_path: PathBuf = plans_folder(common.as_path()).join(plan.facts.clone_guid.file_name());
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest {
        order: order.clone(),
        answer: Err(CancelRefusal::NoAnswerAtTheDeadline { said: String::from("No receipt was on the file at the deadline.") }),
    };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let outcomes: TheDestroyOutcomes = nothing_destroyed();
    match reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_record,
        slot_path: std::path::Path::new("/repo/.claude/worktrees/fix-flaky-test"),
        plan: Some(&plan),
        outcomes: &outcomes,
        mount_request: &request,
        staged: &staged,
    })
    .expect_err("no answer is no reversal")
    {
        CancelRefusal::NoAnswerAtTheDeadline { .. } => {}
        other => panic!("a missing answer refuses as a deadline, not {other:?}"),
    }
    assert_eq!(steps(&order).len(), 1, "nothing after the request ran: {:?}", steps(&order));
    assert!(plan_path.exists(), "the plan is untouched by a reversal that did not answer");
}
