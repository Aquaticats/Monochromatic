//! What:
//!  What one clone's turn answers and files once its reading is decided:
//!  the refusal that files a receipt only for an unavailable key,
//!  the answer to an unmount request,
//!  the `done` receipt of a mount that already stands at the slot,
//!  this boot's `first-attach` record,
//!  the receipt of an attach and the failure receipts of a rollback.
//! Why:
//!  The turn itself is `sweep_one_clone.rs`'s,
//!  and the answers it gives are their own file so the turn stays short enough
//!  to read whole
//!  (rule `MXL`).
//!  The project's receipt rule lives here:
//!  a refusal files no receipt but an unavailable key's,
//!  a `done` mount files the identity a caller confirms,
//!  and a mount the rollback could not take down files a failure rather than a
//!  refusal
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = answerTheAttach(context, verified, request, stepOutcome);
//! ```

use super::sweep_attach_step::{AttachStepOutcome, VERIFY_STEP};
use super::sweep_clone_check::{CloneRefusal, VerifiedClone, slot_of};
use super::sweep_dataset_facts::CloneObjectFacts;
use super::sweep_one_clone::{CloneOutcome, RECEIPT_STEP, SweepContext, UNMOUNT_AWAITS_STEP_NINE};
use crate::diagnostic_log::{debug, warn};
use crate::start_probe::first_attach_record::{FirstAttachClaim, FirstAttachOutcome, NONE as FIRST_ATTACH_NONE};
use crate::worktree_clone_mount::{LookAgainAtSlotRequest, MountIdentity, mount_facts_of};
use crate::worktree_clone_mount_protocol::{CloneMountReceipt, IntentReading, MountIntent, ReceiptMountIdentity, ReceiptOutcome, read_intent};

/// Module tag for diagnostics.
const TAG: &str = "sweepOneCloneAnswers";

/// The answer to a clone the checks refused.
/// An unavailable key is the one refusal that files a receipt,
/// because it is the one a waiting command can act on.
pub(super) fn refused_by_the_checks(context: &SweepContext<'_>, name: &str, facts: &CloneObjectFacts, refusal: &CloneRefusal) -> CloneOutcome {
    warn(TAG, refusal.message().as_str());
    // A clone whose own mount already stands at its slot keeps the done receipt it filed: the key question that now
    // refuses its next attach is not the answer to the request that mount answered.
    if matches!(refusal, CloneRefusal::KeyUnavailable { .. })
        && !the_refused_clones_mount_stands(context, name, facts)
        && let Some(receipt) = the_key_refusals_receipt(context, facts, name, refusal.step())
        && let Err(filed) = context.receipts.publish(&receipt)
    {
        return CloneOutcome::Failed {
            name: String::from(name),
            slot: None,
            step: String::from(RECEIPT_STEP),
            said: format!("{} The receipt could not be filed either: {}", refusal.message(), filed.message()),
        };
    }
    return CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from(refusal.step()), said: refusal.message() };
}

/// Whether the clone a refusal is about already holds its own mount at its
/// slot,
/// which is when a `done` receipt must be kept rather than overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = theRefusedClonesMountStands(context, name, facts);
/// ```
fn the_refused_clones_mount_stands(context: &SweepContext<'_>, name: &str, facts: &CloneObjectFacts) -> bool {
    let Ok(slot) = slot_of(facts.name.as_str(), context.parent_dataset) else {
        return false;
    };
    let request: LookAgainAtSlotRequest<'_> = LookAgainAtSlotRequest { checkout: context.checkout, slot: slot.as_str() };
    return context.standing.standing_mount_of(&request, name).is_some();
}

/// The receipt an unavailable key files,
/// where the clone's GUID and its request both parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = keyRefusalsReceipt(context, facts, name, 'key-unavailable');
/// ```
fn the_key_refusals_receipt(context: &SweepContext<'_>, facts: &CloneObjectFacts, name: &str, step: &str) -> Option<CloneMountReceipt> {
    let dataset = crate::worktree_clone_mount_protocol::DatasetGuid::parse(facts.guid.to_string().as_str()).ok()?;
    let IntentReading::Request(request) = read_intent(Some(&facts.properties), name) else {
        return None;
    };
    return Some(CloneMountReceipt {
        dataset,
        operation: request.operation,
        nonce: request.nonce,
        outcome: ReceiptOutcome::Refused,
        mount: None,
        failed_step: Some(String::from(step)),
        errno: None,
        boot: String::from(context.boot),
        time: (context.time)(),
    });
}

/// What:
///  Answer an unmount request with the step that builds it,
///  and leave the durable intent as it is.
/// Why:
///  The design answers every request with a receipt,
///  and an unmount that this build cannot carry out is answered rather than by
///  silence the caller reads as a timeout;
///  the project's receipt rule files nothing for a refusal but an unavailable
///  key,
///  so the answer is the sweep's line and the handler's journal,
///  and the durable intent stands for plan step 9.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerUnmount(verified, request);
/// ```
pub(super) fn answer_unmount(verified: &VerifiedClone, request: &MountIntent) -> CloneOutcome {
    let said: String = format!(
        "the clone `{}` asks for its mount to be unmounted under nonce {}, and the unmount is plan step 9's, so nothing was taken down and the durable intent stays on the clone",
        verified.dataset_name,
        request.nonce.as_str()
    );
    warn(TAG, said.as_str());
    return CloneOutcome::Refused { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(UNMOUNT_AWAITS_STEP_NINE), said };
}

/// What:
///  File the `done` receipt of a clone whose own mount already stands at the
///  slot.
/// Why:
///  The receipt is what a waiting command reads,
///  and rewriting the same `done` for the same nonce keeps it rather than
///  replacing it with a refusal,
///  which is what every later sweep of a mounted clone would otherwise do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerTheStandingMount(context, verified, request, mount);
/// ```
pub(super) fn answer_the_standing_mount(context: &SweepContext<'_>, verified: &VerifiedClone, request: &MountIntent, mount: MountIdentity) -> CloneOutcome {
    let identity: ReceiptMountIdentity = the_receipt_identity_of(&mount);
    let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Done, Some(identity), None, None);
    debug(TAG, format!("the clone `{}` already holds its own mount {} at slot `{}`", verified.dataset_name, mount.mount_id, verified.slot).as_str());
    if the_same_done_receipt_stands(context, verified, &receipt) {
        debug(TAG, format!("the clone `{}` already filed this request's done receipt, so this sweep writes nothing", verified.dataset_name).as_str());
        return CloneOutcome::Attached {
            name: verified.dataset_name.clone(),
            slot: verified.slot.clone(),
            mount_id: mount.mount_id,
            canonical_slot: mount.observed_path,
        };
    }
    return match context.receipts.publish(&receipt) {
        Ok(()) => CloneOutcome::Attached {
            name: verified.dataset_name.clone(),
            slot: verified.slot.clone(),
            mount_id: mount.mount_id,
            canonical_slot: mount.observed_path,
        },
        Err(refusal) => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!("the clone's mount {} stands at its slot and the receipt could not be filed: {}", mount.mount_id, refusal.message()),
        },
    };
}

/// What:
///  Whether the receipt this request would file is already the one on disk.
/// Why:
///  A steady sweep answers every clone whose mount stands,
///  and the receipt's only changing field is the time;
///  writing the same answer again costs one staged write,
///  two syncs and a rename per standing child,
///  so an unchanged `done` receipt is left exactly as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stands = theSameDoneReceiptStands(context, verified, receipt);
/// ```
fn the_same_done_receipt_stands(context: &SweepContext<'_>, verified: &VerifiedClone, receipt: &CloneMountReceipt) -> bool {
    let Ok(Some(filed)) = context.receipts.read(verified.dataset) else {
        return false;
    };
    return filed.operation == receipt.operation
        && filed.nonce == receipt.nonce
        && filed.outcome == receipt.outcome
        && filed.mount == receipt.mount
        && filed.failed_step == receipt.failed_step
        && filed.errno == receipt.errno
        && filed.boot == receipt.boot;
}

/// The mount a done receipt carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = receiptIdentityOf(mount);
/// ```
fn the_receipt_identity_of(mount: &MountIdentity) -> ReceiptMountIdentity {
    return ReceiptMountIdentity {
        mount_id: mount.mount_id,
        device: mount.device,
        inode: mount.inode,
        source: mount.source.clone(),
        canonical_path: mount.observed_path.display().to_string(),
    };
}

/// The outcome a refusal before the attach writes to this boot's `first-attach`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theFirstAttachRefusal(verified, 'identity-record', undefined);
/// ```
pub(super) fn the_first_attach_refusal(verified: &VerifiedClone, step: &str, errno: Option<&str>) -> FirstAttachOutcome {
    return FirstAttachOutcome::Refused {
        slot: verified.slot.clone(),
        clone: verified.dataset_name.clone(),
        guid: verified.dataset.file_name(),
        step: String::from(step),
        errno: errno.map_or_else(|| return String::from(FIRST_ATTACH_NONE), String::from),
    };
}

/// What:
///  The attach step's answer as this boot's `first-attach` record keeps it.
/// Why:
///  The file carries one of two shapes and no free text,
///  and a rollback or a retained mount is the `verify` step's refusal because
///  the mount was made and did not prove where it landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theFirstAttachOutcome(verified, stepOutcome);
/// ```
pub(super) fn the_first_attach_outcome(verified: &VerifiedClone, outcome: &AttachStepOutcome) -> FirstAttachOutcome {
    return match outcome {
        AttachStepOutcome::Attached { mount } => FirstAttachOutcome::Attached {
            mount_id: mount.mount_id,
            flags: the_flags_read_back(mount.mount_id),
            slot: verified.slot.clone(),
            clone: verified.dataset_name.clone(),
            guid: verified.dataset.file_name(),
        },
        AttachStepOutcome::RolledBack { .. } | AttachStepOutcome::Retained { .. } => the_first_attach_refusal(verified, VERIFY_STEP, None),
        AttachStepOutcome::Refused { step, errno, .. } => the_first_attach_refusal(verified, the_designs_step(step.as_str()), errno.as_deref()),
    };
}

/// The flags the kernel lists for a mount,
/// read back after the move,
/// or `none` when the table cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const flags = theFlagsReadBack(mountId); // 'rw,nosuid,nodev'
/// ```
fn the_flags_read_back(mount_id: u64) -> String {
    return match mount_facts_of(mount_id) {
        Ok(facts) => facts.mount_options.join(","),
        Err(_refusal) => String::from(FIRST_ATTACH_NONE),
    };
}

/// What:
///  The design's own step name for one attach step's word.
/// Why:
///  The design fixes the file's step vocabulary,
///  and this build's call words are the receipts' rather than the file's:
///  the source write is the design's `fsconfig`,
///  and the mount number read after the move belongs to the `verify` step that
///  acts on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const step = theDesignsStep('fsconfig-source'); // 'fsconfig'
/// ```
fn the_designs_step(step: &str) -> &str {
    return match step {
        "fsconfig-source" => "fsconfig",
        "mount-id" => VERIFY_STEP,
        other => other,
    };
}

/// Record one outcome on the claimed `first-attach`,
/// where this process won the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordTheFirstAttach(context, claim, outcome);
/// ```
pub(super) fn record_the_first_attach(context: &SweepContext<'_>, claim: Option<FirstAttachClaim>, outcome: &FirstAttachOutcome) {
    if let Some(won) = claim {
        context.first_attach.record(&won, outcome);
    }
}

/// What:
///  Turn one attach step's answer into the receipt and the sweep's outcome.
/// Why:
///  A done mount files the mount identity a caller confirms,
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
pub(super) fn answer_the_attach(context: &SweepContext<'_>, verified: &VerifiedClone, request: &MountIntent, outcome: AttachStepOutcome) -> CloneOutcome {
    return match outcome {
        AttachStepOutcome::Attached { mount } => {
            let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Done, Some(the_receipt_identity_of(&mount)), None, None);
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
                    said: format!("the clone's mount {} is at its slot and the receipt could not be filed: {}", mount.mount_id, refusal.message()),
                },
            }
        }
        AttachStepOutcome::RolledBack { said } => {
            let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Failed, None, Some("post-check"), None);
            file_a_failure(context, verified, said, "post-check", &receipt)
        }
        AttachStepOutcome::Retained { said } => {
            let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Failed, None, Some("rollback"), None);
            file_a_failure(context, verified, said, "rollback", &receipt)
        }
        AttachStepOutcome::Refused { step, errno, said } => {
            warn(TAG, format!("slot `{}` refused at {step}: {said}", verified.slot).as_str());
            let with_errno: String = match errno {
                Some(word) => format!("{said} The call answered {word}."),
                None => said,
            };
            CloneOutcome::Refused { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step, said: with_errno }
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
fn file_a_failure(context: &SweepContext<'_>, verified: &VerifiedClone, said: String, step: &str, receipt: &CloneMountReceipt) -> CloneOutcome {
    warn(TAG, format!("slot `{}` failed at {step}: {said}", verified.slot).as_str());
    return match context.receipts.publish(receipt) {
        Ok(()) => CloneOutcome::Failed { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(step), said },
        Err(refusal) => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!("{said} The receipt could not be filed either: {}", refusal.message()),
        },
    };
}

/// One receipt for one request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = receiptOf(context, verified, request, 'done', mount, undefined, undefined);
/// ```
fn receipt_of(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
    outcome: ReceiptOutcome,
    mount: Option<ReceiptMountIdentity>,
    failed_step: Option<&str>,
    errno: Option<String>,
) -> CloneMountReceipt {
    return CloneMountReceipt {
        dataset: verified.dataset.clone(),
        operation: request.operation,
        nonce: request.nonce.clone(),
        outcome,
        mount,
        failed_step: failed_step.map(String::from),
        errno,
        boot: String::from(context.boot),
        time: (context.time)(),
    };
}
