//! What:
//!  The controls of the removal intent and the answer that proves it:
//!  the `unmount` operation,
//!  a matching answer,
//!  and each answer that authorizes nothing.
//! Why:
//!  A receipt of another dataset,
//!  another operation or another nonce is another request's answer,
//!  and a receipt that names a step or an errno means the clone may still be mounted,
//!  so only an observed absence becomes the plan's proof
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The removal intent").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(verifyTheAnswer(answer, expected).cause).toBe('answer-nonce');
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{ExpectedUnmount, RemovalIntent, UnmountAnswer, UnmountProof, verify_the_answer};

/// The clone the fixture's removal is about.
fn clone_guid() -> DatasetGuid {
    return DatasetGuid::parse("7828944690444299989").expect("a canonical GUID");
}

/// One answer that answers the intent the control made.
fn a_done_answer(intent: &RemovalIntent) -> UnmountAnswer {
    return UnmountAnswer {
        dataset: clone_guid(),
        operation: MountOperation::Unmount,
        nonce: intent.nonce.clone(),
        outcome: ReceiptOutcome::Done,
        mount_id: None,
        failed_step: None,
        errno: None,
    };
}

/// The request one intent's answer must answer.
fn expected(intent: &RemovalIntent) -> ExpectedUnmount {
    return ExpectedUnmount { clone_guid: clone_guid(), nonce: intent.nonce.clone() };
}

#[test]
fn a_fresh_intent_asks_for_the_unmount_and_carries_thirty_two_digits() {
    let intent: RemovalIntent = RemovalIntent::fresh().expect("the random source answers");
    assert_eq!(intent.nonce.as_str().len(), 32);
    assert_eq!(intent.mount_intent().operation, MountOperation::Unmount);
    assert_eq!(intent.mount_intent().nonce, intent.nonce);
    let second: RemovalIntent = RemovalIntent::fresh().expect("the random source answers");
    assert_ne!(intent.nonce, second.nonce, "two requests are told apart by their nonces");
}

#[test]
fn a_matching_answer_is_the_plans_proof_of_an_absent_mount() {
    let intent: RemovalIntent = RemovalIntent::fresh().expect("the random source answers");
    let proof: UnmountProof = verify_the_answer(&a_done_answer(&intent), &expected(&intent)).expect("an observed absence proves the unmount");
    assert_eq!(proof.nonce, intent.nonce);
    assert_eq!(proof.mount_id, None, "an unmount receipt names no mount, and the plan writes `none`");
}

#[test]
fn an_answer_of_another_dataset_operation_or_nonce_authorizes_nothing() {
    let intent: RemovalIntent = RemovalIntent::fresh().expect("the random source answers");
    let other_dataset: UnmountAnswer = UnmountAnswer { dataset: DatasetGuid::parse("7828944690444299988").expect("a GUID"), ..a_done_answer(&intent) };
    match verify_the_answer(&other_dataset, &expected(&intent)) {
        Err(RemovalStop::UnmountFailed { step, .. }) => assert_eq!(step, "answer-dataset"),
        other => panic!("another dataset's answer refuses, not {other:?}"),
    }
    let other_operation: UnmountAnswer = UnmountAnswer { operation: MountOperation::Mount, ..a_done_answer(&intent) };
    match verify_the_answer(&other_operation, &expected(&intent)) {
        Err(RemovalStop::UnmountFailed { step, .. }) => assert_eq!(step, "answer-operation"),
        other => panic!("a mount's answer refuses, not {other:?}"),
    }
    let other_nonce: UnmountAnswer = UnmountAnswer { nonce: RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a nonce"), ..a_done_answer(&intent) };
    match verify_the_answer(&other_nonce, &expected(&intent)) {
        Err(RemovalStop::UnmountFailed { step, .. }) => assert_eq!(step, "answer-nonce"),
        other => panic!("another nonce's answer refuses, not {other:?}"),
    }
}

#[test]
fn an_answer_that_names_a_step_or_an_errno_is_a_stop_before_the_plan() {
    let intent: RemovalIntent = RemovalIntent::fresh().expect("the random source answers");
    for (outcome, failed_step, errno, wanted_step) in [
        (ReceiptOutcome::Refused, Some("misplaced-rolled-back"), Some("0"), "misplaced-rolled-back"),
        (ReceiptOutcome::Failed, Some("unmount"), Some("16"), "unmount"),
        (ReceiptOutcome::Cancelled, None, None, "answer-outcome"),
    ] {
        let answer: UnmountAnswer = UnmountAnswer { outcome, failed_step: failed_step.map(String::from), errno: errno.map(String::from), ..a_done_answer(&intent) };
        match verify_the_answer(&answer, &expected(&intent)) {
            Err(RemovalStop::UnmountFailed { step, said }) => {
                assert_eq!(step, wanted_step);
                assert!(said.contains(outcome.word()), "the refusal names the outcome it read: {said}");
            }
            other => panic!("an answer that is not a done unmount refuses, not {other:?}"),
        }
    }
}
