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
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_removal_proof_and_plan::fixtures::{guid, sample_facts, sample_snapshots};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_removal_cancel::cancel_stand_ins::{RecordingMountRequest, ScriptedStaged, a_nonce, a_plan_that_stands, a_recorded_binding, an_order, steps};
use crate::worktree_clone_removal_cancel::the_reversing_request::{MountAnswer, MountIdentity};
use crate::worktree_clone_removal_cancel::the_staged_state::StagedSettlement;
use std::path::PathBuf;

/// The slot the reversal's record names.
const SLOT_PATH: &str = "/fixture/repo/.claude/worktrees/fix-flaky-test";

/// A whole answer about the fixture's clone.
fn an_answer() -> MountAnswer {
    return MountAnswer {
        dataset: guid(7_828_944_690_444_299_989),
        operation: MountOperation::Mount,
        nonce: a_nonce(),
        outcome: ReceiptOutcome::Done,
        mount: Some(MountIdentity { mount_id: 7, source: String::from("tank/wt/fix-flaky-test"), canonical_path: String::from(SLOT_PATH) }),
        failed_step: None,
        errno: None,
    };
}

/// The plan a removal wrote before its first request.
fn the_plan() -> DestructionPlan {
    return DestructionPlan::first_write(sample_facts(), sample_snapshots().as_slice());
}

#[test]
fn the_request_is_written_first_the_folder_settled_on_the_answer_and_the_plan_last() {
    let common: PathBuf = fixture("cancel-order");
    let plan: DestructionPlan = the_plan();
    let plan_path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_answer()) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::Removed { record: true, claim: true } };
    let done = reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_recorded_binding(),
        slot_path: std::path::Path::new(SLOT_PATH),
        plan: Some(&plan),
        destroyed: &[],
        mount_request: &request,
        staged: &staged,
    })
    .expect("the reversal runs");
    let taken: Vec<String> = steps(&order);
    assert_eq!(taken.len(), 2, "{taken:?}");
    assert!(taken[0].starts_with("request:mount:"), "the request is the first act: {taken:?}");
    assert_eq!(taken[1], "staged", "the folder is settled on the answer: {taken:?}");
    assert_eq!(done.nonce, taken[0].trim_start_matches("request:mount:").to_string());
    assert!(done.destroyed.is_empty());
    assert!(!plan_path.exists(), "the plan that witnessed nothing left after the settlement");
}

#[test]
fn the_answer_carries_the_mount_the_request_and_the_plan_is_kept_when_a_snapshot_is_gone() {
    let common: PathBuf = fixture("cancel-order-kept");
    let plan: DestructionPlan = the_plan();
    let plan_path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest { order: order.clone(), answer: Ok(an_answer()) };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    let destroyed: Vec<String> = vec![String::from("tank/wt/fix-flaky-test@sanoid-hourly")];
    let done = reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_recorded_binding(),
        slot_path: std::path::Path::new(SLOT_PATH),
        plan: Some(&plan),
        destroyed: destroyed.as_slice(),
        mount_request: &request,
        staged: &staged,
    })
    .expect("the reversal runs");
    assert_eq!(done.destroyed, destroyed);
    assert!(plan_path.exists(), "a plan that records a destroyed snapshot is the loss record and stays");
    assert_eq!(done.mount_id, 7);
}

#[test]
fn a_refused_answer_leaves_the_folder_and_the_plan_untouched() {
    let common: PathBuf = fixture("cancel-order-refused");
    let plan: DestructionPlan = the_plan();
    let plan_path: PathBuf = a_plan_that_stands(common.as_path(), &plan);
    let order = an_order();
    let request: RecordingMountRequest = RecordingMountRequest {
        order: order.clone(),
        answer: Err(CancelRefusal::NoAnswerAtTheDeadline { said: String::from("No receipt was on the file at the deadline.") }),
    };
    let staged: ScriptedStaged = ScriptedStaged { order: order.clone(), settlement: StagedSettlement::NoStagedFolder };
    match reverse_the_removal(&TheReversalRequest {
        common_dir: common.as_path(),
        recorded: &a_recorded_binding(),
        slot_path: std::path::Path::new(SLOT_PATH),
        plan: Some(&plan),
        destroyed: &[],
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
