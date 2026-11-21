//! What:
//!  Controls of the wait and the stale result:
//!  a matching receipt ends the wait,
//!  another request's receipt is the last observation and not an answer,
//!  and the deadline reports the request and how to inspect the handler.
//! Why:
//!  The plan's acceptance test asks that stale results cause no unauthorized
//!  filesystem action,
//!  and the design says a matching receipt is confirmed by the caller's own
//!  mount view
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Waiting and refusal").
//!  The frozen clock reaches the deadline without waiting it out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(waitForAnswerWith(folder, request, 0n, frozen, 0n).kind).toBe('timed-out');
//! ```

use super::{MountRequest, ReceiptStanding, StaleReason, WaitClock, WaitRefusal, standing_of, wait_for_answer_with};
use crate::testing::test_support::fixture;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, ReceiptMountIdentity, ReceiptOutcome};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

/// The dataset every control files under.
const GUID: u64 = 7;

/// The boot this test runs in.
const BOOT: &str = "0f3a5c7e-1b2d-4f60-8a9b-0c1d2e3f4a5b";

/// The request's nonce.
const NONCE: &str = "0123456789abcdef0123456789abcdef";

/// Another request's nonce.
const OTHER_NONCE: &str = "fedcba9876543210fedcba9876543210";

/// This account's number.
fn this_uid() -> u32 {
    // SAFETY: `getuid` takes no argument and changes nothing.
    return unsafe { libc::getuid() };
}

/// The request the controls wait on.
fn request() -> MountRequest {
    return MountRequest {
        dataset: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        operation: MountOperation::Mount,
        nonce: RequestNonce::parse(NONCE).expect("the control nonce is valid"),
        boot: String::from(BOOT),
    };
}

/// A receipt for the request,
/// with the nonce and operation the caller chooses.
fn receipt(nonce: &str, operation: MountOperation) -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: DatasetGuid::parse(GUID.to_string().as_str()).expect("the control GUID is canonical"),
        operation,
        nonce: RequestNonce::parse(nonce).expect("the control nonce is valid"),
        outcome: ReceiptOutcome::Done,
        mount: Some(ReceiptMountIdentity {
            mount_id: 9,
            device: 2049,
            inode: 12,
            source: String::from("zpcachyos/nosnap/clones/one"),
            canonical_path: String::from("/home/user/repo/.claude/worktrees/one"),
        }),
        failed_step: None,
        errno: None,
        boot: String::from(BOOT),
        time: String::from("2026-11-21T15:00:00Z"),
    };
}

/// A fixture folder with the designed mode,
/// owned by this account.
fn folder(name: &str) -> ReceiptsFolder {
    let root: PathBuf = fixture(name);
    std::fs::set_permissions(root.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the fixture takes the directory mode");
    return ReceiptsFolder::at(root, this_uid());
}

/// A clock that never advances,
/// so the first deadline check expires.
struct FrozenClock;

impl WaitClock for FrozenClock {
    fn now(&self) -> Duration {
        return Duration::ZERO;
    }

    fn sleep(&self, _interval: Duration) {}
}

/// A matching receipt ends the wait and is the caller's answer.
#[test]
fn a_matching_receipt_ends_the_wait() {
    let folder: ReceiptsFolder = folder("wait-answer");
    let answer: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    folder.publish(&answer).expect("the receipt is filed");
    let found: CloneMountReceipt = wait_for_answer_with(&folder, &request(), Duration::from_secs(1), &FrozenClock, Duration::ZERO)
        .expect("the matching receipt is the answer");
    assert_eq!(found, answer);
}

/// A receipt under another nonce is stale:
/// the wait does not accept it and the deadline names it as the last
/// observation,
/// so nothing acted on it.
#[test]
fn a_stale_receipt_is_not_an_answer_and_is_reported() {
    let folder: ReceiptsFolder = folder("wait-stale");
    folder.publish(&receipt(OTHER_NONCE, MountOperation::Mount)).expect("the stale receipt is filed");
    let refusal: WaitRefusal = wait_for_answer_with(&folder, &request(), Duration::ZERO, &FrozenClock, Duration::ZERO)
        .expect_err("a stale receipt is not this request's answer");
    let WaitRefusal::TimedOut { request: waited, last, .. } = &refusal else {
        panic!("the wait times out: {refusal:?}");
    };
    assert_eq!(waited, &request());
    assert!(matches!(last, Some(StaleReason::Nonce { .. })), "{last:?}");
    let message: String = refusal.message();
    assert!(message.contains(NONCE), "{message}");
    assert!(message.contains("not this request's answer"), "{message}");
    assert!(message.contains("doctor"), "{message}");
    assert!(message.contains("A deadline is not cancellation"), "{message}");
}

/// A receipt for the same nonce but the other operation is stale,
/// and so is one from another boot.
#[test]
fn the_other_fields_are_compared_too() {
    let answer: CloneMountReceipt = receipt(NONCE, MountOperation::Mount);
    assert_eq!(standing_of(&request(), &answer), ReceiptStanding::Answer(answer.clone()));
    let other_operation: CloneMountReceipt = receipt(NONCE, MountOperation::Unmount);
    assert!(matches!(standing_of(&request(), &other_operation), ReceiptStanding::Stale(StaleReason::Operation { .. })));
    let mut other_boot: CloneMountReceipt = answer;
    other_boot.boot = String::from("11111111-2222-4333-8444-555555555555");
    assert!(matches!(standing_of(&request(), &other_boot), ReceiptStanding::Stale(StaleReason::Boot { .. })));
    let mut other_nonce: CloneMountReceipt = other_operation;
    other_nonce.nonce = RequestNonce::parse(OTHER_NONCE).expect("the other nonce is valid");
    assert!(matches!(standing_of(&request(), &other_nonce), ReceiptStanding::Stale(StaleReason::Nonce { .. })));
}

/// The provisional deadline is the plan's open value,
/// and it is longer than the probe's own measured bound,
/// so it is not a value a fast machine can trip on.
#[test]
fn the_deadline_is_provisional_and_generous() {
    assert_eq!(super::PROVISIONAL_MOUNT_REQUEST_DEADLINE, Duration::from_millis(30_000));
    assert!(super::PROVISIONAL_MOUNT_REQUEST_DEADLINE > Duration::from_millis(350));
    assert_eq!(super::MOUNT_REQUEST_POLL_INTERVAL, Duration::from_millis(50));
}
