//! What:
//!  Controls for the one reversing request:
//!  the writer sends exactly `v1:<32 hex>:mount`,
//!  two cancellations never share a nonce,
//!  and only a `done` answer about this clone at this slot authorizes anything.
//! Why:
//!  A receipt is matched by operation,
//!  nonce and boot,
//!  and the design's rule is that no receipt is reused for a later request
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" section).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(intentText(intent.operation, intent.nonce)).toMatch(/^v1:[0-9a-f]{32}:mount$/);
//! ```

use super::{ExpectedMount, MountAnswer, MountIdentity, the_reversing_intent, verify_the_mount_answer};
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountOperation, intent_text};
use crate::worktree_clone_mount_protocol::clone_mount_receipt::ReceiptOutcome;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use std::path::PathBuf;

/// The clone the controls ask about.
fn guid() -> DatasetGuid {
    return DatasetGuid::parse("7828944690444299989").expect("a canonical GUID");
}

/// The clone's slot.
fn slot() -> PathBuf {
    return PathBuf::from("/repo/.claude/worktrees/one");
}

/// One request as the controls expect it.
fn expected(nonce: RequestNonce) -> ExpectedMount {
    return ExpectedMount { clone_guid: guid(), clone_dataset: String::from("tank/clone"), slot_path: slot(), nonce };
}

/// One whole answer about this clone at its slot.
fn an_answer(nonce: &RequestNonce) -> MountAnswer {
    return MountAnswer {
        dataset: guid(),
        operation: MountOperation::Mount,
        nonce: nonce.clone(),
        outcome: ReceiptOutcome::Done,
        mount: Some(MountIdentity { mount_id: 4242, source: String::from("tank/clone"), canonical_path: String::from("/repo/.claude/worktrees/one") }),
        failed_step: None,
        errno: None,
    };
}

#[test]
fn the_writer_sends_exactly_a_mount_under_thirty_two_hexadecimal_digits() {
    let intent = the_reversing_intent().expect("the random source makes a nonce");
    assert_eq!(intent.operation, MountOperation::Mount);
    let text: String = intent_text(intent.operation, &intent.nonce);
    let digits: &str = text.strip_prefix("v1:").and_then(|rest| return rest.strip_suffix(":mount")).expect("the text is one mount request");
    assert_eq!(digits.len(), 32, "{text}");
    assert!(digits.bytes().all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)), "{text}");
}

#[test]
fn two_cancellations_never_share_a_nonce() {
    let first = the_reversing_intent().expect("the first nonce");
    let second = the_reversing_intent().expect("the second nonce");
    assert_ne!(first.nonce.as_str(), second.nonce.as_str(), "each request is matched by its own nonce");
    assert_eq!(first.operation, MountOperation::Mount, "a reused operation is fine; only the nonce must be fresh");
    assert_ne!(intent_text(first.operation, &first.nonce), intent_text(second.operation, &second.nonce));
}

#[test]
fn a_done_answer_about_this_clone_at_its_slot_is_the_proof() {
    let intent = the_reversing_intent().expect("a nonce");
    let proof = verify_the_mount_answer(&an_answer(&intent.nonce), &expected(intent.nonce.clone())).expect("the answer proves the mount");
    assert_eq!(proof.mount.mount_id, 4242);
    assert_eq!(proof.nonce.as_str(), intent.nonce.as_str());
}

#[test]
fn another_datasets_operation_or_nonces_answer_authorizes_nothing() {
    let intent = the_reversing_intent().expect("a nonce");
    let another_dataset: MountAnswer = MountAnswer { dataset: DatasetGuid::parse("1").expect("a GUID"), ..an_answer(&intent.nonce) };
    assert!(matches!(verify_the_mount_answer(&another_dataset, &expected(intent.nonce.clone())), Err(CancelRefusal::TheRecordsDisagree { .. })));
    let another_operation: MountAnswer = MountAnswer { operation: MountOperation::Unmount, ..an_answer(&intent.nonce) };
    assert!(matches!(verify_the_mount_answer(&another_operation, &expected(intent.nonce.clone())), Err(CancelRefusal::TheRecordsDisagree { .. })));
    let another_nonce = RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a nonce");
    assert!(matches!(verify_the_mount_answer(&an_answer(&another_nonce), &expected(intent.nonce.clone())), Err(CancelRefusal::TheRecordsDisagree { .. })));
}

#[test]
fn a_refusal_or_a_failure_names_its_step_and_errno() {
    let intent = the_reversing_intent().expect("a nonce");
    for outcome in [ReceiptOutcome::Refused, ReceiptOutcome::Failed, ReceiptOutcome::Cancelled] {
        let answer: MountAnswer = MountAnswer {
            outcome,
            mount: None,
            failed_step: Some(String::from("slot-walk")),
            errno: Some(String::from("EBUSY")),
            ..an_answer(&intent.nonce)
        };
        match verify_the_mount_answer(&answer, &expected(intent.nonce.clone())).expect_err("only `done` proves the mount") {
            CancelRefusal::TheHandlerRefused { outcome: word, step, errno } => {
                assert_eq!(word, outcome.word());
                assert_eq!(step, "slot-walk");
                assert_eq!(errno, "EBUSY");
            }
            other => panic!("a {outcome:?} answer refuses as the handler's, not {other:?}"),
        }
    }
}

#[test]
fn a_done_answer_with_no_mount_or_another_mount_refuses() {
    let intent = the_reversing_intent().expect("a nonce");
    let no_mount: MountAnswer = MountAnswer { mount: None, ..an_answer(&intent.nonce) };
    assert!(matches!(verify_the_mount_answer(&no_mount, &expected(intent.nonce.clone())), Err(CancelRefusal::TheMountStateIsNotTheAnswers { .. })));
    let another_source: MountAnswer = MountAnswer {
        mount: Some(MountIdentity { mount_id: 4242, source: String::from("tank/other"), canonical_path: String::from("/repo/.claude/worktrees/one") }),
        ..an_answer(&intent.nonce)
    };
    assert!(matches!(verify_the_mount_answer(&another_source, &expected(intent.nonce.clone())), Err(CancelRefusal::TheMountStateIsNotTheAnswers { .. })));
    let another_slot: MountAnswer = MountAnswer {
        mount: Some(MountIdentity { mount_id: 4242, source: String::from("tank/clone"), canonical_path: String::from("/repo/.claude/worktrees/two") }),
        ..an_answer(&intent.nonce)
    };
    assert!(matches!(verify_the_mount_answer(&another_slot, &expected(intent.nonce.clone())), Err(CancelRefusal::TheMountStateIsNotTheAnswers { .. })));
}
