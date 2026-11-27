//! What:
//!  What one clone's turn files:
//!  the `done` receipt of a mount that already stands at the slot,
//!  the unchanged receipt a steady sweep leaves exactly as it is,
//!  this boot's `first-attach` record,
//!  and the one receipt a request is answered with.
//! Why:
//!  The turn itself is `sweep_one_clone.rs`'s,
//!  and what it files is its own file so the turn stays short enough to read
//!  whole
//!  (rule `MXL`).
//!  The project's receipt rule lives with these answers:
//!  a `done` mount files the identity a caller confirms,
//!  and the `first-attach` record names the first attempt to claim the name
//!  rather than the first mount this process made
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals";
//!  `start_probe/first_attach_record.rs`,
//!  the comment on the record's name).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const receipt = receiptOf(context, verified, request, 'done', mount, undefined, undefined);
//! ```

use super::sweep_attach_step::{
    AttachStepOutcome,
    VERIFY_STEP,
};
use super::sweep_clone_check::VerifiedClone;
use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    RECEIPT_STEP,
    SweepContext,
};
use crate::diagnostic_log::debug;
use crate::start_probe::first_attach_record::{
    FirstAttachClaim,
    FirstAttachOutcome,
    NONE as FIRST_ATTACH_NONE,
};
use crate::worktree_clone_mount::{
    MountIdentity,
    mount_facts_of,
};
use crate::worktree_clone_mount_protocol::{
    CloneMountReceipt,
    MountIntent,
    ReceiptMountIdentity,
    ReceiptOutcome,
};

/// Module tag for diagnostics.
const TAG: &str = "sweepOneCloneAnswers";

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
pub(super) fn answer_the_standing_mount(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    request: &MountIntent,
    mount: MountIdentity,
) -> CloneOutcome {
    let identity: ReceiptMountIdentity = the_receipt_identity_of(&mount);
    let receipt: CloneMountReceipt = receipt_of(
        context,
        verified,
        request,
        ReceiptOutcome::Done,
        Some(identity),
        None,
        None,
    );
    debug(
        TAG,
        format!(
            "the clone `{}` already holds its own mount {} at slot `{}`",
            verified.dataset_name, mount.mount_id, verified.slot
        )
        .as_str(),
    );
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
            said: format!(
                "the clone's mount {} stands at its slot and the receipt could not be filed: {}",
                mount.mount_id,
                refusal.message()
            ),
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
fn the_same_done_receipt_stands(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
    receipt: &CloneMountReceipt,
) -> bool {
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
pub(super) fn the_receipt_identity_of(mount: &MountIdentity) -> ReceiptMountIdentity {
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
pub(super) fn the_first_attach_refusal(
    verified: &VerifiedClone,
    step: &str,
    errno: Option<&str>,
) -> FirstAttachOutcome {
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
///  A clone whose own mount already stood at its slot is recorded with that
///  mount's own number and flags,
///  because the record names the real clone mount at a real slot the probe
///  compares its own attach with,
///  and its comment in `first_attach_record.rs` says the name goes to the first
///  attempt to claim it rather than the first mount this process made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theFirstAttachOutcome(verified, stepOutcome);
/// ```
pub(super) fn the_first_attach_outcome(
    verified: &VerifiedClone,
    outcome: &AttachStepOutcome,
) -> FirstAttachOutcome {
    return match outcome {
        AttachStepOutcome::Attached {
            mount,
        }
        | AttachStepOutcome::AlreadyStanding {
            mount,
        } => FirstAttachOutcome::Attached {
            mount_id: mount.mount_id,
            flags: the_flags_read_back(mount.mount_id),
            slot: verified.slot.clone(),
            clone: verified.dataset_name.clone(),
            guid: verified.dataset.file_name(),
        },
        AttachStepOutcome::RolledBack {
            ..
        }
        | AttachStepOutcome::Retained {
            ..
        } => the_first_attach_refusal(verified, VERIFY_STEP, None),
        AttachStepOutcome::Refused {
            step,
            errno,
            ..
        } => the_first_attach_refusal(verified, the_designs_step(step.as_str()), errno.as_deref()),
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
pub(super) fn record_the_first_attach(
    context: &SweepContext<'_>,
    claim: Option<FirstAttachClaim>,
    outcome: &FirstAttachOutcome,
) {
    if let Some(won) = claim {
        context.first_attach.record(&won, outcome);
    }
}

/// One receipt for one request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipt = receiptOf(context, verified, request, 'done', mount, undefined, undefined);
/// ```
pub(super) fn receipt_of(
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
