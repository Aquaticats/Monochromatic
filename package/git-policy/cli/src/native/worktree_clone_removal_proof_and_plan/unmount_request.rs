//! What:
//!  The removal's `unmount` intent and the answer that proves it:
//!  one fresh nonce,
//!  the durable property step 7's writer files,
//!  step 7's wait for the receipt,
//!  and the verification this removal makes of the answer's own fields.
//! Why:
//!  The removal intent is the durable `unmount` property on the clone and nothing else this route
//!  writes on its own
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The removal intent"),
//!  and the plan is written after the intent's answer is verified,
//!  with its `unmountNonce` naming the request it was verified under.
//!  Step 9's code on `main` is what carries the request and the mount work,
//!  and this module calls it:
//!  the property write is `CloneIntentWriter`,
//!  the wait is `wait_for_answer`,
//!  and no mount call,
//!  staging folder or receipt format is written here.
//!  A busy unmount blocks destruction even when the slot is clear,
//!  so a receipt that names a step or an errno is a refusal and never a proof.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = RemovalIntent.fresh();
//! const answer = LiveRemovalUnmount.open(clone, guid).requestTheUnmount(intent.mountIntent());
//! const proof = verifyTheAnswer(answer, expected);
//! ```

use crate::lazy_runtime::run_on_runtime;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{MountIntent, MountOperation};
use crate::worktree_clone_mount_protocol::clone_mount_intent_write::CloneIntentWriter;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{CloneMountReceipt, ReceiptOutcome};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::mount_request_wait::{MountRequest, PROVISIONAL_MOUNT_REQUEST_DEADLINE, wait_for_answer};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;
use crate::zfs_compatibility_gate::LiveGate;
use std::time::Duration;

/// What:
///  One removal's own request:
///  a fresh nonce and the `unmount` operation.
/// Why:
///  The intent is what a receipt is matched on,
///  so the nonce is made here,
///  once,
///  and both the property and the plan take it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = RemovalIntent.fresh();
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovalIntent {
    /// The nonce the request and its receipt carry.
    pub nonce: RequestNonce,
}

impl RemovalIntent {
    /// A fresh intent,
    /// under a nonce the operating system's random source gave.
    ///
    /// # Throws
    /// The random source failed,
    /// so no request can be told from another.
    pub fn fresh() -> Result<RemovalIntent, RemovalStop> {
        let nonce: RequestNonce = RequestNonce::fresh().map_err(|refusal| {
            return RemovalStop::UnmountFailed { step: String::from("nonce"), said: refusal.message() };
        })?;
        return Ok(RemovalIntent { nonce });
    }

    /// The property value's own value:
    /// the operation and the nonce.
    pub fn mount_intent(&self) -> MountIntent {
        return MountIntent { operation: MountOperation::Unmount, nonce: self.nonce.clone() };
    }
}

/// What:
///  What the handler answered,
///  in the fields this removal verifies.
/// Why:
///  The receipt's own shape is step 7's,
///  and this value keeps a control from having to build one:
///  the verification reads four fields and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { dataset; operation; nonce; outcome; mountId?; failedStep?; errno? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnmountAnswer {
    /// The dataset the receipt is filed for.
    pub dataset: DatasetGuid,
    /// The operation the receipt answers.
    pub operation: MountOperation,
    /// The request's nonce.
    pub nonce: RequestNonce,
    /// What the handler did.
    pub outcome: ReceiptOutcome,
    /// The mount number the answer named,
    /// when it named one.
    pub mount_id: Option<u64>,
    /// The step a refusal or failure names.
    pub failed_step: Option<String>,
    /// The errno a failure names.
    pub errno: Option<String>,
}

impl UnmountAnswer {
    /// One whole receipt as the fields this removal verifies.
    pub fn of_receipt(receipt: &CloneMountReceipt) -> UnmountAnswer {
        return UnmountAnswer {
            dataset: receipt.dataset,
            operation: receipt.operation,
            nonce: receipt.nonce.clone(),
            outcome: receipt.outcome,
            mount_id: receipt.mount.as_ref().map(|mount| return mount.mount_id),
            failed_step: receipt.failed_step.clone(),
            errno: receipt.errno.clone(),
        };
    }
}

/// What:
///  The proof the plan records,
///  once the answer is verified.
/// Why:
///  The plan's two fields are exactly this value's,
///  and no other code path can write them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { nonce; mountId? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnmountProof {
    /// The nonce the verified answer carried.
    pub nonce: RequestNonce,
    /// The mount number the answer named absent,
    /// and nothing when it named none,
    /// which the plan writes as `none`.
    pub mount_id: Option<u64>,
}

/// What:
///  The request one answer must answer.
/// Why:
///  A receipt of another dataset,
///  another operation or another nonce is another request's answer,
///  and settling a removal on it would authorize a destroy nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Expected = { cloneGuid; nonce };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedUnmount {
    /// The clone the request names.
    pub clone_guid: DatasetGuid,
    /// The request's nonce.
    pub nonce: RequestNonce,
}

/// What:
///  The unmount request, as the removal makes it.
/// Why:
///  A control answers this seam,
///  and the release answers it with step 7's writer and wait,
///  so the removal's verification is the only behavior this module adds to the mount route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalUnmount { requestTheUnmount(intent: MountIntent): Answer }
/// ```
pub trait RemovalUnmount {
    /// File the `unmount` intent on the clone and wait for its receipt.
    ///
    /// # Throws
    ///
    /// The property write that was refused,
    /// the wait that ended without an answer,
    /// or the deadline that passed.
    fn request_the_unmount(&self, intent: &MountIntent) -> Result<UnmountAnswer, RemovalStop>;
}

/// What:
///  The release answer:
///  step 7's property writer,
///  step 7's wait and this boot's identifier.
/// Why:
///  The wait is the route's own and it is bounded,
///  and a deadline is not cancellation:
///  the durable request stays on the clone,
///  which the refusal's sentence says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = LiveRemovalUnmount.open(clone, guid)?.requestTheUnmount(intent);
/// ```
pub struct LiveRemovalUnmount {
    clone: String,
    clone_guid: DatasetGuid,
    boot: String,
    deadline: Duration,
}

impl LiveRemovalUnmount {
    /// What:
    ///  Open the release answer for one clone,
    ///  reading this boot's identifier.
    /// Why:
    ///  A receipt belongs to one boot,
    ///  and a receipt of an earlier boot is another request's answer.
    ///
    /// # Throws
    /// The boot identifier cannot be read,
    /// so no receipt could be matched.
    pub fn open(clone: &str, clone_guid: DatasetGuid) -> Result<LiveRemovalUnmount, RemovalStop> {
        let boot: String = std::fs::read_to_string(BOOT_ID_PATH)
            .map(|text| return text.trim().to_string())
            .map_err(|error| return RemovalStop::UnmountFailed { step: String::from("boot-id"), said: format!("{BOOT_ID_PATH} could not be read ({error})") })?;
        return Ok(LiveRemovalUnmount { clone: String::from(clone), clone_guid, boot, deadline: PROVISIONAL_MOUNT_REQUEST_DEADLINE });
    }

    /// The deadline one wait takes,
    /// which the caller may narrow for a control.
    pub fn with_the_deadline(mut self, deadline: Duration) -> LiveRemovalUnmount {
        self.deadline = deadline;
        return self;
    }
}

impl RemovalUnmount for LiveRemovalUnmount {
    fn request_the_unmount(&self, intent: &MountIntent) -> Result<UnmountAnswer, RemovalStop> {
        let writer: CloneIntentWriter = CloneIntentWriter::open(&LiveGate, self.clone.as_str()).map_err(|refusal| {
            return RemovalStop::UnmountFailed { step: String::from("intent"), said: refusal.message() };
        })?;
        writer.request(intent).map_err(|refusal| return RemovalStop::UnmountFailed { step: String::from("intent"), said: refusal.message() })?;
        let folder: ReceiptsFolder = ReceiptsFolder::live();
        let request: MountRequest = MountRequest { dataset: self.clone_guid, operation: intent.operation, nonce: intent.nonce.clone(), boot: self.boot.clone() };
        // The wait is the route's own bounded poll, so it runs on the process's runtime and not on a worker a command
        // holds.
        let receipt: CloneMountReceipt = match run_on_runtime(wait_for_answer(&folder, &request, self.deadline)) {
            Ok(Ok(found)) => found,
            Ok(Err(refusal)) => return Err(RemovalStop::UnmountFailed { step: String::from("receipt"), said: refusal.message() }),
            Err(refusal) => return Err(RemovalStop::UnmountFailed { step: String::from("receipt"), said: format!("the wait could not run ({refusal})") }),
        };
        return Ok(UnmountAnswer::of_receipt(&receipt));
    }
}

/// What:
///  Verify one answer against the request it must answer.
/// Why:
///  Only an observed absence proves the unmount:
///  another dataset's,
///  another operation's or another nonce's answer authorizes nothing,
///  and a refusal,
///  a failure,
///  or a receipt that puts the clone back at its slot is a stop before the plan is written.
///
/// # Throws
/// The field that differs or the step the answer names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proof = verifyTheAnswer(answer, expected);
/// ```
pub fn verify_the_answer(answer: &UnmountAnswer, expected: &ExpectedUnmount) -> Result<UnmountProof, RemovalStop> {
    if answer.dataset != expected.clone_guid {
        return Err(RemovalStop::UnmountFailed {
            step: String::from("answer-dataset"),
            said: format!("the receipt is filed for GUID {} and this removal asked for {}", answer.dataset.file_name(), expected.clone_guid.file_name()),
        });
    }
    if answer.operation != MountOperation::Unmount {
        return Err(RemovalStop::UnmountFailed { step: String::from("answer-operation"), said: format!("the receipt answers `{}`", answer.operation.word()) });
    }
    if answer.nonce != expected.nonce {
        return Err(RemovalStop::UnmountFailed {
            step: String::from("answer-nonce"),
            said: format!("the receipt carries nonce {} and this removal asked under {}", answer.nonce.as_str(), expected.nonce.as_str()),
        });
    }
    if answer.outcome != ReceiptOutcome::Done {
        let step: String = answer.failed_step.clone().unwrap_or_else(|| return String::from("answer-outcome"));
        let errno: String = answer.errno.clone().unwrap_or_else(|| return String::from("none"));
        return Err(RemovalStop::UnmountFailed {
            step,
            said: format!("the handler answered `{}` with errno {errno}", answer.outcome.word()),
        });
    }
    return Ok(UnmountProof { nonce: answer.nonce.clone(), mount_id: answer.mount_id });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unmount_request_tests.rs"]
mod tests;
