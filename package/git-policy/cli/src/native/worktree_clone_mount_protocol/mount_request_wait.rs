//! What:
//!  The wrapper's wait for the receipt that answers one request:
//!  the standing of a receipt it finds,
//!  the mount-request deadline,
//!  the bounded poll,
//!  and the timeout report that shows the last observation and the request.
//! Why:
//!  The design waits on a monotonic deadline and bounded waits,
//!  requires a matching receipt to be confirmed by the caller's own mount view,
//!  and asks a timeout to show the last observation,
//!  the matching request and how to inspect the handler
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Waiting and refusal").
//!  A stale result,
//!  from another request or another boot,
//!  is never the answer and authorizes no filesystem action:
//!  the wait keeps looking until its deadline and then reports what it saw.
//!  A deadline is not cancellation:
//!  the durable request may still act later,
//!  so the report says so rather than promising nothing happened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const receipt = waitForAnswer(folder, request, PROVISIONAL_MOUNT_REQUEST_DEADLINE);
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::CloneMountReceipt;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::ReceiptsRefusal;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::time::Duration;

/// What:
///  The bound on one caller's wait for a matching receipt,
///  in milliseconds.
/// Why:
///   The plan makes this the route's first ZFS wait constant and leaves its
///  size to step 22
///  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
///  "Step 7:
/// The intent property,
/// the receipts and the readiness record",
///  and step 22),
///  so this value is provisional and cites the guest rounds that exist now
///  rather than claiming to be measured for the route.
///  The guest measurement of one property event through a test zedlet and the
///  product receipt is kept in
///  `package/git-policy/cli/doc/zfs-mount-receipt-deadline-measurement-2026-11-21.md`;
///  step 22 replaces this number when it sizes the wait.
///  The number is not derived from that measurement:
///  it is a provisional ceiling chosen to leave room far above the rounds the
///  document records,
///  and no round has yet exercised a product handler that attaches a mount,
///  which step 8 adds.
pub const PROVISIONAL_MOUNT_REQUEST_DEADLINE: Duration = Duration::from_millis(30_000);

/// What:
///  How long one poll waits between reads of the receipts directory.
/// Why:
///   The handler files a receipt once,
///  so the wait has nothing to gain from a tight loop,
///  and a fixed small interval keeps the account's CPU use bounded without
///  a synthetic-load test.
pub const MOUNT_REQUEST_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// What:
///  One request a caller is waiting for:
///  the dataset's GUID,
///  the operation,
///  the nonce and this boot.
/// Why:
///   A receipt answers only when all four agree,
///  so the request is one value the comparison and the timeout report share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountRequest = { dataset; operation; nonce; boot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountRequest {
    /// The clone's dataset GUID.
    pub dataset: DatasetGuid,
    /// What the caller asked for.
    pub operation: MountOperation,
    /// The nonce the request was filed under.
    pub nonce: RequestNonce,
    /// This boot's kernel identifier.
    pub boot: String,
}

/// What:
///  Why a receipt on the file is not this request's answer.
/// Why:
///   A stale result is another request's answer,
///  and the caller must know which field differed rather than wait on a file
///  it believes is its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StaleReason = { kind: 'nonce' | 'operation' | 'boot'; found; expected };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StaleReason {
    /// The receipt was filed for another dataset.
    Dataset {
        /// The GUID the receipt carries.
        found: String,
        /// The GUID the request names.
        expected: String,
    },
    /// The receipt carries another nonce.
    Nonce {
        /// The nonce the receipt carries.
        found: String,
        /// The request's nonce.
        expected: String,
    },
    /// The receipt names another operation.
    Operation {
        /// The operation the receipt names.
        found: String,
        /// What the request asked for.
        expected: String,
    },
    /// The receipt belongs to another boot.
    Boot {
        /// The boot the receipt names.
        found: String,
        /// This boot.
        expected: String,
    },
}

impl StaleReason {
    /// The reason in words.
    pub fn word(&self) -> String {
        return match self {
            StaleReason::Dataset { found, expected } => format!("the receipt names dataset GUID {found}, and the request names {expected}"),
            StaleReason::Nonce { found, expected } => format!("the receipt's nonce is {found}, and the request's is {expected}"),
            StaleReason::Operation { found, expected } => format!("the receipt names `{found}`, and the request asked for `{expected}`"),
            StaleReason::Boot { found, expected } => format!("the receipt belongs to boot {found}, and this is boot {expected}"),
        };
    }
}

/// What:
///  What one receipt on the file is for this request.
/// Why:
///   The three answers are separate:
///  an answer ends the wait,
///  a stale receipt is recorded as the last observation,
///  and an absent one means the handler has not filed yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReceiptStanding = { kind: 'answer' } | { kind: 'stale' } | { kind: 'absent' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiptStanding {
    /// The receipt answers this request.
    Answer(CloneMountReceipt),
    /// The receipt is another request's answer.
    Stale(StaleReason),
}

/// What:
///  The standing of one receipt against one request.
/// Why:
///   The comparison is one place,
///  so the live wait and every control decide the same way,
///  and the dataset GUID is checked first because the file name already
///  names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standing = standingOf(request, receipt);
/// ```
pub fn standing_of(request: &MountRequest, receipt: &CloneMountReceipt) -> ReceiptStanding {
    if receipt.dataset != request.dataset {
        return ReceiptStanding::Stale(StaleReason::Dataset {
            found: receipt.dataset.file_name(),
            expected: request.dataset.file_name(),
        });
    }
    if receipt.nonce != request.nonce {
        return ReceiptStanding::Stale(StaleReason::Nonce {
            found: String::from(receipt.nonce.as_str()),
            expected: String::from(request.nonce.as_str()),
        });
    }
    if receipt.operation != request.operation {
        return ReceiptStanding::Stale(StaleReason::Operation {
            found: String::from(receipt.operation.word()),
            expected: String::from(request.operation.word()),
        });
    }
    if receipt.boot != request.boot {
        return ReceiptStanding::Stale(StaleReason::Boot { found: receipt.boot.clone(), expected: request.boot.clone() });
    }
    return ReceiptStanding::Answer(receipt.clone());
}

/// What:
///  Why a wait ended without an answer.
/// Why:
///   A timeout and a refused directory are different answers,
///  and the timeout must retain the request and the last receipt it saw,
///  so a person can tell "the handler has not filed" from "the handler filed
///  another request's result".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WaitRefusal = { kind: 'timed-out'; last?; request } | { kind: 'receipts'; refusal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WaitRefusal {
    /// The deadline passed with no matching receipt.
    TimedOut {
        /// The request that was waited for.
        request: Box<MountRequest>,
        /// The last stale receipt's reason,
        /// where one was on the file.
        last: Option<StaleReason>,
        /// The deadline,
        /// in milliseconds.
        deadline_millis: u128,
    },
    /// The receipts directory or a file could not be used.
    Receipts(ReceiptsRefusal),
}

impl WaitRefusal {
    /// The refusal in words,
    /// naming the request,
    /// the last observation and the way to inspect the handler.
    pub fn message(&self) -> String {
        return match self {
            WaitRefusal::Receipts(refusal) => refusal.message(),
            WaitRefusal::TimedOut { request, last, deadline_millis } => {
                let observed: String = match last {
                    Some(reason) => format!("The last receipt on the file is not this request's answer: {}.", reason.word()),
                    None => String::from("No receipt was on the file at the deadline."),
                };
                format!(
                    "no result for this request was filed within {deadline_millis} milliseconds, so the mount was not confirmed in the caller's view. The request is operation `{}` under nonce {} for dataset GUID {} in boot {}. {observed} A deadline is not cancellation: the durable request stays on the clone and a later sweep may still act on it, so inspect before writing another request. Ways forward: run `monochromatic-git-policy-cli doctor` and read the handler's journal with `journalctl -u reconcile-git-worktree-clone-mounts.service`; do not retry a conflicting request until the clone's intent and mount state have been verified.",
                    request.operation.word(),
                    request.nonce.as_str(),
                    request.dataset.file_name(),
                    request.boot
                )
            }
        };
    }
}

/// Wait for the receipt that answers `request` until `deadline` passes.
///
/// # Errors
/// The receipts directory or a file could not be used,
/// or the deadline passed with no matching receipt.
pub async fn wait_for_answer(folder: &ReceiptsFolder, request: &MountRequest, deadline: Duration) -> Result<CloneMountReceipt, WaitRefusal> {
    let started: tokio::time::Instant = tokio::time::Instant::now();
    let mut last: Option<StaleReason> = None;
    loop {
        let found: Option<CloneMountReceipt> = folder.read(request.dataset).map_err(WaitRefusal::Receipts)?;
        if let Some(receipt) = found {
            match standing_of(request, &receipt) {
                ReceiptStanding::Answer(answer) => return Ok(answer),
                ReceiptStanding::Stale(reason) => last = Some(reason),
            }
        }
        let Some(left) = remaining(started.elapsed(), deadline) else {
            return Err(WaitRefusal::TimedOut { request: Box::new(request.clone()), last, deadline_millis: deadline.as_millis() });
        };
        tokio::time::sleep(MOUNT_REQUEST_POLL_INTERVAL.min(left)).await;
    }
}

/// What:
///  How long one wait may still sleep,
///  or nothing once the deadline passed.
/// Why:
///   The comparison is one place,
///  so the loop and the timeout agree on the boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = remaining(now, deadline); // null when expired
/// ```
fn remaining(now: Duration, deadline: Duration) -> Option<Duration> {
    if now >= deadline {
        return None;
    }
    return Some(deadline - now);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "mount_request_wait_tests.rs"]
mod tests;
