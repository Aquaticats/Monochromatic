//! What:
//!  The dataset GUID read and the refusal answer the unmount arm needs.
//! Why:
//!  They are the unmount's own and the sweep's answer file is over its code
//!  bound;
//! one file per part keeps both readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! HandlerDatasetGuids.liveGuid(dataset); refuseAnUnmount(context, verified, request, step, said);
//! ```

use super::sweep_clone_check::VerifiedClone;
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    RECEIPT_STEP,
    SweepContext,
};
use crate::diagnostic_log::warn;
use crate::worktree_clone_mount_protocol::{
    CloneMountReceipt,
    MountIntent,
    ReceiptMountIdentity,
    ReceiptOutcome,
};
use crate::worktree_clone_unmount::slot_walk_calls::DatasetGuids;
use crate::worktree_clone_unmount::unmount_turn::{
    ReattachCause,
    UnmountOutcome,
};

/// Module tag for diagnostics.
const TAG: &str = "sweepUnmountAnswers";

/// What:
///  The live dataset GUID read,
///  from the sweep's own pool reader.
/// Why:
///  The unmount's GUID comparison needs the pool's answer,
///  and the handler already holds the one reader a control can script.
pub(super) struct HandlerDatasetGuids<'a> {
    pub(super) reader:
        &'a dyn crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader,
}

impl DatasetGuids for HandlerDatasetGuids<'_> {
    fn live_guid(
        &self,
        dataset: &str,
    ) -> Result<crate::worktree_clone_mount_protocol::DatasetGuid, String> {
        let identity = self
            .reader
            .identity(dataset)
            .map_err(|refusal| return refusal.message())?;
        return crate::worktree_clone_mount_protocol::DatasetGuid::parse(
            identity.stats.dds_guid.to_string().as_str(),
        )
        .map_err(|refusal| return refusal.message());
    }
}

/// Answer a refusal that stopped the unmount before its turn could build a request.
pub(super) fn refuse_an_unmount(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
    step: &str,
    said: String,
) -> CloneOutcome {
    warn(
        TAG,
        format!(
            "slot `{}` refused the unmount at {step}: {said}",
            verified.slot
        )
        .as_str(),
    );
    let receipt = super::sweep_one_clone_answers::receipt_of(
        context,
        verified,
        request,
        ReceiptOutcome::Cancelled,
        None,
        Some(step),
        None,
    );
    if let Err(filed) = context.receipts.publish(&receipt) {
        return CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!(
                "{said} The receipt could not be filed either: {}",
                filed.message()
            ),
        };
    }
    return CloneOutcome::Refused {
        name: verified.dataset_name.clone(),
        slot: Some(verified.slot.clone()),
        step: String::from(step),
        said,
    };
}

/// What:
///  Answer one unmount outcome,
///  filing only the receipts the accepted decisions give.
/// Why:
///  Step 8's decision files no receipt for a refusal but an unavailable key's,
///  and the staging record gives one unmount outcome a receipt:
///  the successful rollback.
///  A done unmount files its `done` receipt because the wrapper's cleanup reads it,
///  and every other refusal is left to the sweep's line and the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerTheUnmount(context, verified, request, turnOutcome);
/// ```
pub(super) fn answer_the_unmount(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
    outcome: UnmountOutcome,
) -> CloneOutcome {
    let receipt: Option<CloneMountReceipt> = match &outcome {
        UnmountOutcome::Unmounted {
            ..
        } => Some(super::sweep_one_clone_answers::receipt_of(
            context,
            verified,
            request,
            ReceiptOutcome::Done,
            None,
            None,
            None,
        )),
        UnmountOutcome::Reattached {
            cause: ReattachCause::MisplacedRolledBack,
            ..
        } => Some(super::sweep_one_clone_answers::receipt_of(
            context,
            verified,
            request,
            ReceiptOutcome::Refused,
            None,
            Some("misplaced-rolled-back"),
            None,
        )),
        UnmountOutcome::Reattached {
            mount_id,
            slot,
            bind_gone,
            device,
            inode,
            cause: ReattachCause::Busy,
        } => Some(super::sweep_one_clone_answers::receipt_of(
            context,
            verified,
            request,
            ReceiptOutcome::Cancelled,
            Some(ReceiptMountIdentity {
                mount_id: *mount_id,
                device: *device,
                inode: *inode,
                source: verified.dataset_name.clone(),
                canonical_path: slot.display().to_string(),
            }),
            Some(if *bind_gone {
                "bind-gone"
            } else {
                "bind-present"
            }),
            None,
        )),
        UnmountOutcome::Retained {
            step,
            ..
        }
        | UnmountOutcome::Refused {
            step,
            ..
        } => Some(super::sweep_one_clone_answers::receipt_of(
            context,
            verified,
            request,
            ReceiptOutcome::Cancelled,
            None,
            Some(step.as_str()),
            None,
        )),
    };
    if let Some(filed_receipt) = receipt
        && let Err(filed) = context.receipts.publish(&filed_receipt)
    {
        return CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!(
                "the unmount's receipt could not be filed: {}",
                filed.message()
            ),
        };
    }
    return match outcome {
        UnmountOutcome::Unmounted {
            mount_id,
            ..
        } => CloneOutcome::Unmounted {
            name: verified.dataset_name.clone(),
            slot: verified.slot.clone(),
            mount_id,
        },
        UnmountOutcome::Reattached {
            mount_id,
            bind_gone,
            ..
        } => CloneOutcome::Reattached {
            name: verified.dataset_name.clone(),
            slot: verified.slot.clone(),
            mount_id,
            bind_gone,
        },
        UnmountOutcome::Retained {
            step,
            said,
            ..
        } => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step,
            said,
        },
        UnmountOutcome::Refused {
            step,
            said,
            ..
        } => CloneOutcome::Refused {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step,
            said,
        },
    };
}
