//! What:
//!  The one request that reverses a removal:
//!  a `mount` intent under a fresh nonce,
//!  the wait for its answer,
//!  and the verification of the answer's own fields.
//! Why:
//!  The design's reversal is one `mount` request written in one place,
//!  with a fresh nonce,
//!  because a receipt is matched by operation,
//!  nonce and boot and no receipt is reused for a later request
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The act" and "One reversing request" sections).
//!  The forced arm never writes the intent itself,
//!  so one place writes the reversing request and this module is that place for the cancellation.
//!  Only a `done` answer whose mount names the clone's own source at its slot authorizes anything:
//!  a refusal,
//!  a failure,
//!  a timeout,
//!  or an answer under another nonce is a refusal and leaves every record as it stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = requestTheMount({ clone, cloneGuid, slotPath }); // the answer's verified fields
//! ```

use crate::lazy_runtime::run_on_runtime;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    MountIntent,
    MountOperation,
};
use crate::worktree_clone_mount_protocol::clone_mount_intent_write::CloneIntentWriter;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::{
    CloneMountReceipt,
    ReceiptOutcome,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder::ReceiptsFolder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::mount_request_wait::{
    MountRequest,
    PROVISIONAL_MOUNT_REQUEST_DEADLINE,
    wait_for_answer,
};
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_cancel::cancel_refusal::CancelRefusal;
use crate::worktree_clone_unmount::staging_calls::BOOT_ID_PATH;
use crate::zfs_compatibility_gate::LiveGate;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// What:
///  The mount one `done` answer observed,
///  in the fields this command verifies.
/// Why:
///  The receipt's mount keys are the source the mount table names and the slot path it was found
///  at,
///  and a `done` answer whose mount is no mount of this clone at this slot authorizes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountIdentity = { mountId: number; source: string; canonicalPath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountIdentity {
    /// The kernel's mount number.
    pub mount_id: u64,
    /// The ZFS source the mount table names.
    pub source: String,
    /// The canonical slot path the mount was found at.
    pub canonical_path: String,
}

/// What:
///  What the handler answered,
///  in the fields this command verifies.
/// Why:
///  A control answers this shape without a handler,
///  and the verification reads five fields and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { dataset; operation; nonce; outcome; mount?; failedStep?; errno? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountAnswer {
    /// The dataset the receipt is filed for.
    pub dataset: DatasetGuid,
    /// The operation the receipt answers.
    pub operation: MountOperation,
    /// The request's nonce.
    pub nonce: RequestNonce,
    /// What the handler did.
    pub outcome: ReceiptOutcome,
    /// The mount the answer observed,
    /// where it observed one.
    pub mount: Option<MountIdentity>,
    /// The step a refusal or failure names.
    pub failed_step: Option<String>,
    /// The errno a failure names.
    pub errno: Option<String>,
}

impl MountAnswer {
    /// One whole receipt as the fields this command verifies.
    pub fn of_receipt(receipt: &CloneMountReceipt) -> MountAnswer {
        return MountAnswer {
            dataset: receipt.dataset,
            operation: receipt.operation,
            nonce: receipt.nonce.clone(),
            outcome: receipt.outcome,
            mount: receipt.mount.as_ref().map(|mount| {
                return MountIdentity {
                    mount_id: mount.mount_id,
                    source: mount.source.clone(),
                    canonical_path: mount.canonical_path.clone(),
                };
            }),
            failed_step: receipt.failed_step.clone(),
            errno: receipt.errno.clone(),
        };
    }
}

/// What:
///  The request one answer must answer.
/// Why:
///  A receipt of another dataset,
///  another operation or another nonce is another request's answer,
///  and attaching on it would put a clone back that nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Expected = { cloneGuid; cloneDataset; slotPath; nonce };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpectedMount {
    /// The clone the request names.
    pub clone_guid: DatasetGuid,
    /// The clone's dataset name,
    /// which the answer's mount must name as its source.
    pub clone_dataset: String,
    /// The slot path the clone must be mounted at.
    pub slot_path: PathBuf,
    /// The request's nonce.
    pub nonce: RequestNonce,
}

/// What:
///  The verified fields of the answer,
///  which the report and the staged settlement read.
/// Why:
///  A `done` answer that passed the checks is the only authorization this command has,
///  and the mount identity it named is what the report prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { nonce; mount };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountProof {
    /// The nonce the verified answer carried.
    pub nonce: RequestNonce,
    /// The mount the answer observed.
    pub mount: MountIdentity,
}

/// What:
///  The reversing request,
///  as the removal and the command make it.
/// Why:
///  A control answers this seam and the release answers it with the route's own writer and wait,
///  so the verification is the only behavior this module adds to the mount route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheMountRequest { requestTheMount(intent: MountIntent): MountAnswer }
/// ```
pub trait TheMountRequest {
    /// File the `mount` intent on the clone and wait for its answer.
    ///
    /// # Errors
    /// The property write that was refused,
    /// or the wait that ended without a matching answer.
    fn request_the_mount(
        &self,
        intent: &MountIntent,
    ) -> Result<MountAnswer, CancelRefusal>;
}

/// What:
///  The release answer:
///  the route's property writer,
///  the route's bounded wait and this boot's identifier.
/// Why:
///  The wait is the route's own and it is bounded,
///  and a deadline is not cancellation:
///  the durable request stays on the clone,
///  which the refusal's sentence says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = LiveTheMountRequest.open(clone, guid)?.requestTheMount(intent);
/// ```
pub struct LiveTheMountRequest {
    clone: String,
    clone_guid: DatasetGuid,
    boot: String,
    deadline: Duration,
}

impl LiveTheMountRequest {
    /// What:
    ///  Open the release answer for one clone,
    ///  reading this boot's identifier.
    /// Why:
    ///  A receipt belongs to one boot,
    ///  and a receipt of an earlier boot is another request's answer.
    ///
    /// # Errors
    /// The boot identifier cannot be read,
    /// so no receipt could be matched.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const request = LiveTheMountRequest.open('pool/clones/one', guid);
    /// ```
    pub fn open(
        clone: &str,
        clone_guid: DatasetGuid,
    ) -> Result<LiveTheMountRequest, CancelRefusal> {
        let boot: String = std::fs::read_to_string(BOOT_ID_PATH).map(|text| return text.trim().to_string()).map_err(|error| {
            return CancelRefusal::NoAnswerAtTheDeadline { said: format!("the answer would be matched by this boot's identifier, and {BOOT_ID_PATH} could not be read ({error}).") };
        })?;
        return Ok(LiveTheMountRequest {
            clone: String::from(clone),
            clone_guid,
            boot,
            deadline: PROVISIONAL_MOUNT_REQUEST_DEADLINE,
        });
    }

    /// The deadline one wait takes,
    /// which a caller may narrow for a control.
    pub fn with_the_deadline(
        mut self,
        deadline: Duration,
    ) -> LiveTheMountRequest {
        self.deadline = deadline;
        return self;
    }
}

impl TheMountRequest for LiveTheMountRequest {
    fn request_the_mount(
        &self,
        intent: &MountIntent,
    ) -> Result<MountAnswer, CancelRefusal> {
        let writer: CloneIntentWriter = CloneIntentWriter::open(&LiveGate, self.clone.as_str())
            .map_err(|refusal| {
                return CancelRefusal::TheMountRequestCouldNotBeWritten {
                    said: refusal.message(),
                };
            })?;
        writer.request(intent).map_err(|refusal| {
            return CancelRefusal::TheMountRequestCouldNotBeWritten {
                said: refusal.message(),
            };
        })?;
        let folder: ReceiptsFolder = ReceiptsFolder::live();
        let request: MountRequest = MountRequest {
            dataset: self.clone_guid,
            operation: intent.operation,
            nonce: intent.nonce.clone(),
            boot: self.boot.clone(),
        };
        let receipt: CloneMountReceipt =
            match run_on_runtime(wait_for_answer(&folder, &request, self.deadline)) {
                Ok(Ok(found)) => found,
                Ok(Err(refusal)) => {
                    return Err(CancelRefusal::NoAnswerAtTheDeadline {
                        said: refusal.message(),
                    });
                }
                Err(refusal) => {
                    return Err(CancelRefusal::NoAnswerAtTheDeadline {
                        said: format!("the wait could not run ({refusal})."),
                    });
                }
            };
        return Ok(MountAnswer::of_receipt(&receipt));
    }
}

/// What:
///  One reversing intent:
///  a fresh nonce and the `mount` operation.
/// Why:
///  The nonce is made once and both the property and the expectation take it,
///  so a receipt under another nonce can only be another request's.
///
/// # Errors
/// The random source failed,
/// so no request could be told from another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = theReversingIntent(); // { operation: 'mount', nonce }
/// ```
pub fn the_reversing_intent() -> Result<MountIntent, CancelRefusal> {
    let nonce: RequestNonce = RequestNonce::fresh().map_err(|refusal| {
        return CancelRefusal::TheMountRequestCouldNotBeWritten {
            said: format!(
                "this process cannot make the nonce its request carries ({})",
                refusal.message()
            ),
        };
    })?;
    return Ok(MountIntent {
        operation: MountOperation::Mount,
        nonce,
    });
}

/// What:
///  Verify one answer against the request it must answer,
///  and against the clone's own source and slot.
/// Why:
///  Only an observed mount of this clone at this slot proves the reversal:
///  another dataset's,
///  another operation's or another nonce's answer authorizes nothing,
///  and a refusal,
///  a failure,
///  or a `done` answer with no mount is a refusal to reverse.
///
/// # Errors
/// The field that differs,
/// the step the answer names,
/// or the mount that is not the clone's at its slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proof = verifyTheMountAnswer(answer, expected);
/// ```
pub fn verify_the_mount_answer(
    answer: &MountAnswer,
    expected: &ExpectedMount,
) -> Result<MountProof, CancelRefusal> {
    if answer.dataset != expected.clone_guid {
        return Err(CancelRefusal::TheRecordsDisagree {
            what: String::from("the answer is filed for a dataset this command did not ask about"),
            first: answer.dataset.file_name(),
            second: expected.clone_guid.file_name(),
        });
    }
    if answer.operation != MountOperation::Mount {
        return Err(CancelRefusal::TheRecordsDisagree {
            what: String::from(
                "the answer answers another operation than the `mount` this command asked for",
            ),
            first: answer.operation.word().to_string(),
            second: MountOperation::Mount.word().to_string(),
        });
    }
    if answer.nonce != expected.nonce {
        return Err(CancelRefusal::TheRecordsDisagree {
            what: String::from("the answer carries another nonce than the request's"),
            first: answer.nonce.as_str().to_string(),
            second: expected.nonce.as_str().to_string(),
        });
    }
    if answer.outcome != ReceiptOutcome::Done {
        return Err(CancelRefusal::TheHandlerRefused {
            outcome: String::from(answer.outcome.word()),
            step: answer
                .failed_step
                .clone()
                .unwrap_or_else(|| return String::from("answer")),
            errno: answer
                .errno
                .clone()
                .unwrap_or_else(|| return String::from("none")),
        });
    }
    let Some(mount) = &answer.mount else {
        return Err(CancelRefusal::TheMountStateIsNotTheAnswers {
            said: String::from(
                "the handler answered `done` and named no mount, so nothing proves the clone stands at its slot.",
            ),
        });
    };
    if mount.source != expected.clone_dataset {
        return Err(CancelRefusal::TheMountStateIsNotTheAnswers {
            said: format!(
                "the answer's mount is a mount of `{}`, and the clone is `{}`.",
                mount.source, expected.clone_dataset
            ),
        });
    }
    if Path::new(mount.canonical_path.as_str()) != expected.slot_path {
        return Err(CancelRefusal::TheMountStateIsNotTheAnswers {
            said: format!(
                "the answer's mount is at `{}`, and the clone's slot is `{}`.",
                mount.canonical_path,
                expected.slot_path.display()
            ),
        });
    }
    return Ok(MountProof {
        nonce: answer.nonce.clone(),
        mount: mount.clone(),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_reversing_request_tests.rs"]
mod tests;
