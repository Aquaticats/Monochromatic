//! What:
//!  The answer to one attach step's outcome:
//!  the `done` receipt of the mount it made,
//!  the answer of a mount that already stood at its slot,
//!  the failure receipt of a rollback,
//!  the retained mount that could not be taken down,
//!  and the refusal that files nothing.
//! Why:
//!  A done mount files the mount identity a caller confirms,
//!  a mount the rollback took down files the failed step,
//!  a mount that could not be taken down is retained with both paths rather
//!  than "nothing happened",
//!  and a refusal files no receipt but an unavailable key's
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals").
//!  It is its own file so a reader of the attach's answer need not read the
//!  unmount's or the first attach's
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = answerTheAttach(context, verified, request, stepOutcome);
//! ```

use super::sweep_attach_step::AttachStepOutcome;
use super::sweep_clone_check::VerifiedClone;
use super::sweep_one_clone_answers::{
    answer_the_standing_mount,
    receipt_of,
    the_receipt_identity_of,
};
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    RECEIPT_STEP,
    SweepContext,
};
use crate::diagnostic_log::warn;
use crate::worktree_clone_mount_protocol::{
    CloneMountReceipt,
    MountIntent,
    ReceiptOutcome,
};

/// Module tag for diagnostics.
const TAG: &str = "sweepAttachAnswer";

/// What:
///  Turn one attach step's answer into the receipt and the sweep's outcome.
/// Why:
///  A done mount files the mount identity a caller confirms,
///  a clone whose own mount already stood at its slot is answered from that
///  mount exactly as the sweep answers a standing clone before the walk,
///  a mount the rollback took down files the failed step,
///  and a mount that could not be taken down is reported as retained with both
///  paths rather than as "nothing happened".
///  A refusal files no receipt,
///  which is the project's rule for everything but an unavailable key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerTheAttach(context, verified, request, stepOutcome);
/// ```
pub(super) fn answer_the_attach(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
    outcome: AttachStepOutcome,
) -> CloneOutcome {
    return match outcome {
        AttachStepOutcome::Attached {
            mount,
        } => {
            let receipt: CloneMountReceipt = receipt_of(
                context,
                verified,
                request,
                ReceiptOutcome::Done,
                Some(the_receipt_identity_of(&mount)),
                None,
                None,
            );
            match context.receipts.publish(&receipt) {
                Ok(()) => CloneOutcome::Attached {
                    name: verified.dataset_name.clone(),
                    slot: verified.slot.clone(),
                    mount_id: mount.mount_id,
                    canonical_slot: mount.observed_path.clone(),
                },
                Err(refusal) => CloneOutcome::Failed {
                    name: verified.dataset_name.clone(),
                    slot: Some(verified.slot.clone()),
                    step: String::from(RECEIPT_STEP),
                    said: format!(
                        "the clone's mount {} is at its slot and the receipt could not be filed: {}",
                        mount.mount_id,
                        refusal.message()
                    ),
                },
            }
        }
        // The attach step found this clone's own mount at the slot and made none, so the request is answered from that
        // mount by the same answer a clone whose mount stood before the walk gets: the receipt keeps this request's nonce
        // and an unchanged one is left as it is.
        AttachStepOutcome::AlreadyStanding {
            mount,
        } => answer_the_standing_mount(context, verified, request, mount),
        AttachStepOutcome::RolledBack {
            said,
        } => {
            let receipt: CloneMountReceipt = receipt_of(
                context,
                verified,
                request,
                ReceiptOutcome::Failed,
                None,
                Some("post-check"),
                None,
            );
            file_a_failure(context, verified, said, "post-check", &receipt)
        }
        AttachStepOutcome::Retained {
            said,
        } => {
            let receipt: CloneMountReceipt = receipt_of(
                context,
                verified,
                request,
                ReceiptOutcome::Failed,
                None,
                Some("rollback"),
                None,
            );
            file_a_failure(context, verified, said, "rollback", &receipt)
        }
        AttachStepOutcome::Refused {
            step,
            errno,
            said,
        } => {
            warn(
                TAG,
                format!("slot `{}` refused at {step}: {said}", verified.slot).as_str(),
            );
            let with_errno: String = match errno {
                Some(word) => format!("{said} The call answered {word}."),
                None => said,
            };
            CloneOutcome::Refused {
                name: verified.dataset_name.clone(),
                slot: Some(verified.slot.clone()),
                step,
                said: with_errno,
            }
        }
    };
}

/// File a failure and answer it,
/// naming the receipt when even that could not be filed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = fileAFailure(context, verified, said, step, receipt);
/// ```
fn file_a_failure(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    said: String,
    step: &str,
    receipt: &CloneMountReceipt,
) -> CloneOutcome {
    warn(
        TAG,
        format!("slot `{}` failed at {step}: {said}", verified.slot).as_str(),
    );
    return match context.receipts.publish(receipt) {
        Ok(()) => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(step),
            said,
        },
        Err(refusal) => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!(
                "{said} The receipt could not be filed either: {}",
                refusal.message()
            ),
        },
    };
}
