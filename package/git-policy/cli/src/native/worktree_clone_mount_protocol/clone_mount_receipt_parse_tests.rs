//! What:
//!  Controls of the receipt's text and its parse:
//!  the three outcomes,
//!  the mount identity a done result carries,
//!  and every shape that is refused.
//! Why:
//!  A caller acts on this file,
//!  so the round trip and each refusal are held here:
//!  a wrong key order,
//!  a stale-looking field and a malformed number must never be read as a
//!  result
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseReceipt(done.text()).outcome).toBe('done');
//! ```

use super::parse_receipt;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    ABSENT_WORD, CloneMountReceipt, MAXIMUM_RECEIPT_BYTES, RECEIPT_KEYS, RECEIPT_KIND, ReceiptMountIdentity, ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipt_refusals::ReceiptRefusal;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The dataset GUID every control files under.
const GUID: u64 = 1234567890123456789;

/// The nonce every control matches.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// The boot every control belongs to.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The time every control was filed at.
const TIME: &str = "2026-11-21T15:00:00Z";

/// The canonical slot path.
const SLOT: &str = "/home/user/repo/.claude/worktrees/one";

/// The source dataset.
const SOURCE: &str = "zpcachyos/nosnap/clones/one";

/// The nonce every control carries.
fn nonce() -> RequestNonce {
    return RequestNonce::parse(NONCE).expect("the control nonce is valid");
}

/// A done receipt with a real mount identity.
fn done() -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        operation: MountOperation::Mount,
        nonce: nonce(),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 41,
            device: 2049,
            inode: 987654321,
            source: String::from(SOURCE),
            canonical_path: String::from(SLOT),
        }),
        failed_step: None,
        errno: None,
        boot: String::from(BOOT),
        time: String::from(TIME),
    }
}

/// Each outcome's text parses back into the receipt it was written from.
#[test]
fn each_outcome_round_trips() {
    let mut refused: CloneMountReceipt = done();
    refused.operation = MountOperation::Unmount;
    refused.outcome = ReceiptOutcome::Refused;
    refused.mount = None;
    refused.failed_step = Some(String::from("read-intent"));
    let mut failed: CloneMountReceipt = done();
    failed.outcome = ReceiptOutcome::Failed;
    failed.mount = None;
    failed.failed_step = Some(String::from("attach"));
    failed.errno = Some(String::from("EACCES"));
    for receipt in [done(), refused, failed] {
        let text: String = receipt.text();
        let parsed: CloneMountReceipt = parse_receipt(text.as_str()).expect("the route's own text parses");
        assert_eq!(parsed, receipt);
    }
}

/// The file names the dataset by its canonical GUID and carries the nonce as
/// data,
/// never as part of a name or a key.
#[test]
fn the_file_carries_the_guid_and_the_nonce_as_data() {
    let text: String = done().text();
    assert!(text.starts_with("v1\nkind clone-mount-result\n"));
    assert!(text.contains(&format!("dataset {GUID}\n")), "{text}");
    assert!(text.contains(&format!("nonce {NONCE}\n")), "{text}");
    assert!(!format!("{GUID}").contains(NONCE));
    assert_eq!(text.lines().count(), RECEIPT_KEYS.len());
    assert_eq!(RECEIPT_KEYS[1], "kind");
    assert!(text.contains(RECEIPT_KIND));
}

/// A file over the byte bound is refused before any line is read.
#[test]
fn an_oversized_file_is_refused() {
    let text: String = format!("{}{}", done().text(), "x".repeat(MAXIMUM_RECEIPT_BYTES));
    let refusal: ReceiptRefusal = parse_receipt(text.as_str()).expect_err("a huge file is refused");
    assert_eq!(refusal, ReceiptRefusal::TooLarge { bytes: text.len(), bound: MAXIMUM_RECEIPT_BYTES });
}

/// A missing or extra line is refused by count.
#[test]
fn a_wrong_line_count_is_refused() {
    let text: String = done().text();
    let missing: String = text.lines().take(RECEIPT_KEYS.len() - 1).map(|line| format!("{line}\n")).collect();
    assert!(matches!(parse_receipt(missing.as_str()), Err(ReceiptRefusal::LineCount { .. })));
    let extra: String = format!("{text}extra line\n");
    assert!(matches!(parse_receipt(extra.as_str()), Err(ReceiptRefusal::LineCount { .. })));
}

/// A key at another place is refused and named,
/// so a reordered file is not read as a receipt.
#[test]
fn a_reordered_key_is_refused_and_named() {
    let text: String = done().text();
    let swapped: String = text.replacen("dataset ", "datasets ", 1);
    let refusal: ReceiptRefusal = parse_receipt(swapped.as_str()).expect_err("another key is refused");
    assert!(matches!(refusal, ReceiptRefusal::Key { expected: "dataset", .. }), "{refusal:?}");
    assert!(refusal.message().contains("`dataset`"), "{}", refusal.message());
}

/// A dataset GUID or nonce that is not canonical is refused.
#[test]
fn a_non_canonical_guid_or_nonce_is_refused() {
    let without_zero: String = done().text().replace(&format!("dataset {GUID}"), "dataset 0123");
    assert!(matches!(parse_receipt(without_zero.as_str()), Err(ReceiptRefusal::Dataset(_))));
    let uppercase: String = done().text().replace(NONCE, NONCE.to_uppercase().as_str());
    assert!(matches!(parse_receipt(uppercase.as_str()), Err(ReceiptRefusal::Nonce(_))));
}

/// An operation or outcome word outside the closed set is refused.
#[test]
fn an_unknown_operation_or_outcome_is_refused() {
    let operation: String = done().text().replace("operation mount", "operation attach");
    assert!(matches!(parse_receipt(operation.as_str()), Err(ReceiptRefusal::Value { key: "operation", .. })));
    let outcome: String = done().text().replace("outcome done", "outcome maybe");
    assert!(matches!(parse_receipt(outcome.as_str()), Err(ReceiptRefusal::Value { key: "outcome", .. })));
}

/// A done result without its mount identity,
/// and a refused one that carries an identity,
/// are both refused.
#[test]
fn the_mount_identity_belongs_only_to_a_done_result() {
    let without_identity: String = done().text().replace("mount-id 41", &format!("mount-id {ABSENT_WORD}"));
    let identity_refusal: ReceiptRefusal = parse_receipt(without_identity.as_str()).expect_err("a done result names its mount");
    assert!(matches!(identity_refusal, ReceiptRefusal::MountField { key: "mount-id", wanted: true }), "{identity_refusal:?}");
    let mut refused: CloneMountReceipt = done();
    refused.outcome = ReceiptOutcome::Refused;
    refused.failed_step = Some(String::from("read-intent"));
    let with_identity: String = refused.text().replace(&format!("mount-id {ABSENT_WORD}"), "mount-id 41");
    let refusal: ReceiptRefusal = parse_receipt(with_identity.as_str()).expect_err("a refused result names no mount");
    assert!(matches!(refusal, ReceiptRefusal::MountField { key: "mount-id", wanted: false }), "{refusal:?}");
}

/// What:
///  A done unmount carries `none` for the mount identity and parses,
///  and a done unmount that carries one is refused.
/// Why:
///   An unmount that verified absence has no mount to identify,
///  and the design asks for the identity "where applicable"
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Requests,
/// results and ordering").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(parseReceipt(doneUnmount.text()).mount).toBeNull();
/// ```
#[test]
fn a_done_unmount_carries_no_mount_identity() {
    let mut unmount: CloneMountReceipt = done();
    unmount.operation = MountOperation::Unmount;
    unmount.mount = None;
    let parsed: CloneMountReceipt = parse_receipt(unmount.text().as_str()).expect("a done unmount without a mount parses");
    assert_eq!(parsed, unmount);
    assert_eq!(parsed.mount, None, "a done unmount identifies no mount");
    let with_identity: String = unmount.text().replace(&format!("mount-id {ABSENT_WORD}"), "mount-id 41");
    let refusal: ReceiptRefusal = parse_receipt(with_identity.as_str()).expect_err("a done unmount carries no mount");
    assert!(matches!(refusal, ReceiptRefusal::MountField { key: "mount-id", wanted: false }), "{refusal:?}");
}

/// A done result names no failing step,
/// and a refusal names one.
#[test]
fn the_failure_words_agree_with_the_outcome() {
    let done_with_step: String = done().text().replace(&format!("failed-step {ABSENT_WORD}"), "failed-step attach");
    assert!(matches!(parse_receipt(done_with_step.as_str()), Err(ReceiptRefusal::Value { key: "failed-step", .. })));
    let mut refused: CloneMountReceipt = done();
    refused.outcome = ReceiptOutcome::Refused;
    refused.mount = None;
    let without_step: String = refused.text();
    assert!(matches!(
        parse_receipt(without_step.as_str()),
        Err(ReceiptRefusal::MountField { key: "failed-step", wanted: true })
    ));
}

/// Each numeric and text field has its own rule.
#[test]
fn each_identity_field_has_its_rule() {
    let device: String = done().text().replace("device 2049", "device 20 49");
    assert!(matches!(parse_receipt(device.as_str()), Err(ReceiptRefusal::Value { key: "device", .. })));
    let path: String = done().text().replace(SLOT, "relative/one");
    assert!(matches!(parse_receipt(path.as_str()), Err(ReceiptRefusal::Value { key: "canonical-path", .. })));
    let empty_source: String = done().text().replace(&format!("source {SOURCE}"), "source  ");
    assert!(matches!(parse_receipt(empty_source.as_str()), Err(ReceiptRefusal::Value { key: "source", .. })));
}

/// A boot that is not the kernel's identifier and a time without an offset
/// are refused.
#[test]
fn a_foreign_boot_or_time_is_refused() {
    let boot: String = done().text().replace(BOOT, "not-a-boot-id");
    assert!(matches!(parse_receipt(boot.as_str()), Err(ReceiptRefusal::Value { key: "boot", .. })));
    let time: String = done().text().replace(TIME, "yesterday");
    assert!(matches!(parse_receipt(time.as_str()), Err(ReceiptRefusal::Time { .. })));
}

/// Every refusal tells the reader where the handler's own account is,
/// and that nothing acted on the file.
#[test]
fn every_refusal_names_the_journal_and_authorizes_nothing() {
    let text: String = done().text().replace("outcome done", "outcome maybe");
    let refusal: ReceiptRefusal = parse_receipt(text.as_str()).expect_err("another outcome is refused");
    let message: String = refusal.message();
    assert!(message.contains("journalctl -u reconcile-git-worktree-clone-mounts.service"), "{message}");
    assert!(message.contains("Nothing was matched"), "{message}");
}

/// A cancelled unmount carries the clone at its slot and round-trips.
#[test]
fn a_cancelled_unmount_with_a_mount_round_trips() {
    let mut cancelled: CloneMountReceipt = done();
    cancelled.operation = MountOperation::Unmount;
    cancelled.outcome = ReceiptOutcome::Cancelled;
    cancelled.failed_step = Some(String::from("bind-gone"));
    let text: String = cancelled.text();
    assert_eq!(parse_receipt(text.as_str()).expect("the route's own cancelled text parses"), cancelled);
}

/// A cancelled unmount with no mount at all round-trips.
#[test]
fn a_cancelled_unmount_without_a_mount_round_trips() {
    let mut cancelled: CloneMountReceipt = done();
    cancelled.operation = MountOperation::Unmount;
    cancelled.outcome = ReceiptOutcome::Cancelled;
    cancelled.mount = None;
    cancelled.failed_step = Some(String::from("staging-folder"));
    let text: String = cancelled.text();
    assert_eq!(parse_receipt(text.as_str()).expect("the route's own cancelled text parses"), cancelled);
}

/// An unknown outcome word is refused,
/// and a word one byte over the bound with it.
#[test]
fn an_unknown_or_oversized_outcome_word_is_refused() {
    for word in ["cancelledx", "finished"] {
        let text: String = done().text().replace("outcome done", format!("outcome {word}").as_str());
        assert!(matches!(parse_receipt(text.as_str()), Err(ReceiptRefusal::Value { key: "outcome", .. })), "{word}");
    }
}

/// A mount field on a cancelled answer must be all present or all absent.
#[test]
fn a_partly_filled_mount_on_a_cancelled_answer_is_refused() {
    let mut cancelled: CloneMountReceipt = done();
    cancelled.operation = MountOperation::Unmount;
    cancelled.outcome = ReceiptOutcome::Cancelled;
    cancelled.failed_step = Some(String::from("bind-gone"));
    let text: String = cancelled.text().replace("mount-id 41", "mount-id none");
    assert!(matches!(parse_receipt(text.as_str()), Err(ReceiptRefusal::MountField { .. })));
}

/// A missing field,
/// a newline inside a field and a NUL are refused.
#[test]
fn a_missing_field_a_newline_and_a_nul_are_refused() {
    let text: String = done().text();
    let missing: String = text.replace("boot 0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b\n", "");
    assert!(parse_receipt(missing.as_str()).is_err());
    let newline: String = text.replace("source zpcachyos/nosnap/clones/one", "source zpcachyos/nosnap/clones\none");
    assert!(parse_receipt(newline.as_str()).is_err());
    let nul: String = text.replace("source zpcachyos/nosnap/clones/one", "source zpcachyos/nosnap/clones\u{0}one");
    assert!(parse_receipt(nul.as_str()).is_err());
}
