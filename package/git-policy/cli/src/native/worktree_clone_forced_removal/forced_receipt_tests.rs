//! What:
//!  Controls for the forced arm's receipt:
//!  a finished act files `done` under the unmount intent,
//!  a refusal files its own word in the failed step,
//!  the word is read back by the receipts folder's own reader,
//!  and every refusal word fits the field the receipt allows.
//! Why:
//!  The wrapper decides what to print from this file alone,
//!  and a word the field cuts would be read back as another word,
//!  so both are shown here in a fixture folder this account owns,
//!  which is the same write the live handler makes with root.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = refusalAnswer(guid, nonce, RefusalWord.HolderLocked, Some(EINVAL)); folder.publish(receiptOf(answer, boot, time));
//! ```

use super::{FilingRefusal, TheActsAnswer, file_the_receipt, receipt_of, refusal_answer};
use crate::testing::test_support::fixture;
use crate::worktree_clone_forced_removal::forced_removal_words::{FAILED_STEP_MAXIMUM_BYTES, RefusalWord};
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, ReceiptOutcome};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// A canonical dataset GUID for a control's receipt.
fn a_dataset() -> DatasetGuid {
    return DatasetGuid::parse("18446744073709551615").expect("a canonical GUID");
}

/// The request's nonce.
fn a_nonce() -> RequestNonce {
    return RequestNonce::parse("00112233445566778899aabbccddeeff").expect("a canonical nonce");
}

/// A receipts folder at a fixture directory,
/// owned by this account.
fn a_folder(name: &str) -> ReceiptsFolder {
    use std::os::unix::fs::MetadataExt;
    let directory: std::path::PathBuf = fixture(name);
    let owner: u32 = std::fs::metadata(directory.as_path()).expect("the fixture folder's own facts").uid();
    return ReceiptsFolder::at(directory, owner);
}

#[test]
fn a_finished_act_files_done_under_the_unmount_intent() {
    let folder: ReceiptsFolder = a_folder("forced_receipt_done");
    let answer: TheActsAnswer = TheActsAnswer {
        dataset: a_dataset(),
        nonce: a_nonce(),
        outcome: ReceiptOutcome::Done,
        failed_step: None,
        errno: None,
    };
    file_the_receipt(&folder, &answer).expect("the folder takes the answer");
    let read: CloneMountReceipt = folder.read(a_dataset()).expect("the folder reads its own receipt").expect("the receipt is there");
    assert_eq!(read.outcome, ReceiptOutcome::Done);
    assert_eq!(read.operation, MountOperation::Unmount, "the forced act answers the standing unmount intent");
    assert_eq!(read.nonce, a_nonce());
    assert_eq!(read.failed_step, None);
    let text: String = read.text();
    assert!(text.contains("failed-step none"), "a finished act names no failed step, and the file reads {text}");
}

#[test]
fn a_refusal_files_its_own_word_in_the_failed_step() {
    let folder: ReceiptsFolder = a_folder("forced_receipt_refused");
    let answer: TheActsAnswer = refusal_answer(a_dataset(), a_nonce(), RefusalWord::HolderLocked, Some(libc::EINVAL));
    file_the_receipt(&folder, &answer).expect("the folder takes the refusal");
    let read: CloneMountReceipt = folder.read(a_dataset()).expect("the folder reads its own refusal").expect("the receipt is there");
    assert_eq!(read.outcome, ReceiptOutcome::Refused);
    assert_eq!(read.failed_step.as_deref(), Some("holder-locked"));
    assert_eq!(read.errno.as_deref(), Some("22"), "the errno travels as the receipt's own field, and it read {:?}", read.errno);
}

#[test]
fn every_refusal_word_fits_the_receipt_s_own_field() {
    let words: [RefusalWord; 7] = [
        RefusalWord::CallerProtectedHolds,
        RefusalWord::HolderUnendable,
        RefusalWord::HolderUnseen,
        RefusalWord::HolderBlind,
        RefusalWord::HolderLocked,
        RefusalWord::HolderStructural,
        RefusalWord::CallerGone,
    ];
    for word in words {
        let answer: TheActsAnswer = refusal_answer(a_dataset(), a_nonce(), word, None);
        let receipt: CloneMountReceipt = receipt_of(&answer, "b0b0b0b0-0000-4000-8000-000000000000", "2026-11-25T00:00:00.000-05:00").expect("every word fits the field");
        assert_eq!(receipt.failed_step.as_deref(), Some(word.word()));
        assert!(word.word().len() <= FAILED_STEP_MAXIMUM_BYTES);
        assert!(RefusalWord::parse(word.word()).is_some(), "the word the receipt carries is one the grammar reads back");
    }
}

#[test]
fn a_word_the_field_would_cut_is_refused_rather_than_written() {
    let too_long: RefusalWord = RefusalWord::HolderUnendable;
    let receipt: CloneMountReceipt = receipt_of(
        &TheActsAnswer { dataset: a_dataset(), nonce: a_nonce(), outcome: ReceiptOutcome::Refused, failed_step: Some(too_long), errno: None },
        "b0b0b0b0-0000-4000-8000-000000000000",
        "2026-11-25T00:00:00.000-05:00",
    )
    .expect("the design's longest word fits");
    assert_eq!(receipt.failed_step.as_deref(), Some(too_long.word()));
    let refusal: String = FilingRefusal::WordTooLong { word: "a-word-longer-than-the-field-holds-and-holds-and-holds-and-holds-and-holds-and-holds" }.to_string();
    assert!(refusal.contains("longer than the receipt"), "the refusal names the field's bound, and it read {refusal}");
}

#[test]
fn the_boot_and_the_time_are_the_callers_and_not_read_again() {
    let answer: TheActsAnswer = TheActsAnswer { dataset: a_dataset(), nonce: a_nonce(), outcome: ReceiptOutcome::Failed, failed_step: Some(RefusalWord::HolderStructural), errno: Some(libc::EBUSY) };
    let receipt: CloneMountReceipt = receipt_of(&answer, "b0b0b0b0-0000-4000-8000-000000000000", "2026-11-25T00:00:00.000-05:00").expect("the answer is filed with the boot it was given");
    assert_eq!(receipt.boot, "b0b0b0b0-0000-4000-8000-000000000000");
    assert_eq!(receipt.time, "2026-11-25T00:00:00.000-05:00");
    assert_eq!(receipt.outcome, ReceiptOutcome::Failed);
}
