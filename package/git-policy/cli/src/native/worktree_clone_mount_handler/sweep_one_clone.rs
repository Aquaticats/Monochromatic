//! What:
//!  One clone's whole turn of the sweep:
//!  read what the dataset says,
//!  verify it,
//!  read its durable intent,
//!  take the slot's claim,
//!  record the slot's first observation,
//!  attach,
//!  and publish the receipt that answers the request.
//! Why:
//!  The order is the design's:
//!  the checks,
//!  then the intent,
//!  then the claim under which the desired state and identities are rechecked,
//!  then the attach,
//!  and one receipt per clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The sweep plan step 8").
//!  A malformed intent is refused before any filesystem operation on its behalf,
//!  an inherited intent authorizes nothing,
//!  and a slot another sweep holds is left to that sweep rather than queued.
//!  Every answer that names a step also files it,
//!  so a caller waiting on a nonce reads what the handler decided instead of a
//!  timeout the handler could have answered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = sweepOneClone(context, 'zpcachyos/…/clones/one');
//! ```

use super::sweep_attach_step::{AttachRequest, AttachStepOutcome, CloneAttachStep};
use super::sweep_clone_check::{CloneRefusal, VerifiedClone, verify};
use super::sweep_dataset_facts::{CloneObjectFacts, facts_of};
use super::sweep_dataset_reader::DatasetReader;
use crate::diagnostic_log::{debug, warn};
use crate::worktree_clone_mount::{
    HoldCloneSlotRequest, SlotIdentityStore, StoreRefusal, observe_the_clone_slot_under_root_control, publish_slot_identity_under_root_control,
};
use crate::worktree_clone_mount_handler::per_slot_root_claim::SlotClaimDirectory;
use crate::worktree_clone_mount_protocol::{
    CloneMountReceipt, IntentReading, MountIntent, MountOperation, ReceiptMountIdentity, ReceiptOutcome, ReceiptsFolder, read_intent,
};
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "sweepOneClone";

/// The step a receipt names when an unmount request reaches this build.
///
/// Why:
///  The unmount is plan step 9's,
///  and a caller that asked for one learns that from a filed refusal rather
///  than from a deadline,
///  while the durable intent stays on the clone unchanged.
pub const UNMOUNT_AWAITS_STEP_NINE: &str = "unmount-awaits-step-9";

/// The step a receipt names when the slot's own record could not be published.
pub const RECORD_STEP: &str = "slot-record";

/// The step a receipt names when the receipt itself could not be filed.
pub const RECEIPT_STEP: &str = "receipt";

/// What:
///  Everything one clone's turn reads and writes.
/// Why:
///  The paths and the seams travel together so a caller cannot sweep one
///  checkout's children against another checkout's store,
///  and a control injects fixture folders and a scripted pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepContext = { parentDataset; checkout; store; claims; receipts; boot; time; reader; attach };
/// ```
pub struct SweepContext<'a> {
    /// The pinned clone parent dataset.
    pub parent_dataset: &'a str,
    /// The checkout the walk starts from.
    pub checkout: &'a Path,
    /// The store of recorded identities.
    pub store: &'a Path,
    /// The per-slot claims.
    pub claims: &'a SlotClaimDirectory,
    /// The receipts folder.
    pub receipts: &'a ReceiptsFolder,
    /// This boot's kernel identifier.
    pub boot: &'a str,
    /// The clock a receipt's time is written from.
    pub time: &'a dyn Fn() -> String,
    /// The pool reads.
    pub reader: &'a dyn DatasetReader,
    /// The attach step.
    pub attach: &'a dyn CloneAttachStep,
}

/// What:
///  What one clone's turn answered.
/// Why:
///  A sweep reports every clone it reached,
///  and a clone it attached and a clone it refused must read differently in one
///  line an operator or a guest arm can find.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneOutcome = { kind: 'nothing-asked' | 'attached' | 'refused' | 'failed' | 'held'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneOutcome {
    /// The clone asked for nothing,
    /// or the value it carries is inherited.
    NothingAsked {
        /// The dataset.
        name: String,
        /// The slot its name gave.
        slot: String,
        /// Why nothing was asked.
        why: String,
    },
    /// The clone's mount is at its slot.
    Attached {
        /// The dataset.
        name: String,
        /// The slot.
        slot: String,
        /// The mount the attach made.
        mount_id: u64,
        /// The canonical slot the mount was found at.
        canonical_slot: PathBuf,
    },
    /// Nothing was attached,
    /// and the receipt says why.
    Refused {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the receipt names.
        step: String,
        /// The refusal in words.
        said: String,
    },
    /// A step failed after or during the attach,
    /// and the receipt says which,
    /// or nothing was filed when the receipt itself refused.
    Failed {
        /// The dataset.
        name: String,
        /// The slot,
        /// where the name gave one.
        slot: Option<String>,
        /// The step the receipt names.
        step: String,
        /// What the step answered.
        said: String,
    },
    /// Another sweep holds this slot.
    Held {
        /// The dataset.
        name: String,
        /// The slot another sweep holds.
        slot: String,
        /// What the claim answered.
        said: String,
    },
}

impl CloneOutcome {
    /// The dataset this outcome is about.
    pub fn name(&self) -> &str {
        return match self {
            CloneOutcome::NothingAsked { name, .. } | CloneOutcome::Attached { name, .. } | CloneOutcome::Refused { name, .. } | CloneOutcome::Failed { name, .. } | CloneOutcome::Held { name, .. } => {
                name.as_str()
            }
        };
    }

    /// The one line a sweep's report carries for this clone.
    /// It begins with the answer's own word,
    /// names the dataset and the slot,
    /// and says what a person does next when the answer was no.
    pub fn line(&self) -> String {
        return match self {
            CloneOutcome::NothingAsked { name, slot, why } => format!("nothing {name} slot={slot} why={why}"),
            CloneOutcome::Attached { name, slot, mount_id, canonical_slot } => format!("attached {name} slot={slot} mount={mount_id} at={}", canonical_slot.display()),
            CloneOutcome::Refused { name, slot, step, said } => format!("refused {name} slot={} step={step}: {said}", slot.as_deref().unwrap_or("none")),
            CloneOutcome::Failed { name, slot, step, said } => format!("failed {name} slot={} step={step}: {said}", slot.as_deref().unwrap_or("none")),
            CloneOutcome::Held { name, slot, said } => format!("held {name} slot={slot}: {said}"),
        };
    }
}

/// What:
///  Read one clone,
///  verify it,
///  and answer what its intent asked for.
/// Why:
///  One function is the whole turn so the order cannot differ between the
///  sweep a boot runs and the sweep a ZED event starts,
///  and every refusal files the receipt a caller waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = sweepOneClone(context, name);
/// ```
pub fn sweep_one_clone(context: &SweepContext<'_>, name: &str) -> CloneOutcome {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => {
            warn(TAG, format!("`{name}` could not be read: {reason}").as_str());
            return CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from("read"), said: reason };
        }
    };
    let verified: VerifiedClone = match verify(&facts, context.parent_dataset) {
        Ok(verified) => verified,
        Err(refusal) => return refused_by_the_checks(name, &refusal),
    };
    let request: MountIntent = match read_intent(Some(&facts.properties), name) {
        IntentReading::Absent => return CloneOutcome::NothingAsked { name: String::from(name), slot: verified.slot, why: String::from("no local mount intent") },
        IntentReading::Inherited { from } => {
            return CloneOutcome::NothingAsked { name: String::from(name), slot: verified.slot, why: format!("the mount intent is inherited from {from}") };
        }
        IntentReading::Refused(refusal) => return CloneOutcome::Refused { name: String::from(name), slot: Some(verified.slot), step: String::from("intent"), said: refusal.message() },
        IntentReading::Request(request) => request,
    };
    if request.operation == MountOperation::Unmount {
        return answer_unmount(context, &verified, &request);
    }
    return mount_the_clone(context, name, &verified, &request);
}

/// The answer to a clone the checks refused,
/// which filed nothing because no name or nonce reached the receipt's grammar.
fn refused_by_the_checks(name: &str, refusal: &CloneRefusal) -> CloneOutcome {
    warn(TAG, refusal.message().as_str());
    return CloneOutcome::Refused { name: String::from(name), slot: None, step: String::from(refusal.step()), said: refusal.message() };
}

/// What:
///  Answer an unmount request with the step that builds it,
///  and leave the durable intent as it is.
/// Why:
///  The design answers every request with a receipt,
///  and an unmount that this build cannot carry out is answered as a refusal
///  naming plan step 9 rather than by silence the caller reads as a timeout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerUnmount(context, verified, request);
/// ```
fn answer_unmount(context: &SweepContext<'_>, verified: &VerifiedClone, request: &MountIntent) -> CloneOutcome {
    let said: String = format!(
        "the clone `{}` asks for its mount to be unmounted under nonce {}, and the unmount is plan step 9's, so nothing was taken down and the durable intent stays on the clone",
        verified.dataset_name,
        request.nonce.as_str()
    );
    let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Refused, None, Some(UNMOUNT_AWAITS_STEP_NINE), None);
    return match context.receipts.publish(&receipt) {
        Ok(()) => CloneOutcome::Refused { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(UNMOUNT_AWAITS_STEP_NINE), said },
        Err(refusal) => CloneOutcome::Failed { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(RECEIPT_STEP), said: format!("{said}, and the receipt could not be filed: {}", refusal.message()) },
    };
}

/// What:
///  Take the claim,
///  record the slot's first observation,
///  attach,
///  and file the receipt.
/// Why:
///  The claim is what lets parallel sweeps work without acting on the same
///  mount twice,
///  and the first observation is published before the attach so every later
///  walk compares the folder this sweep judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = mountTheClone(context, name, verified, request);
/// ```
fn mount_the_clone(context: &SweepContext<'_>, name: &str, verified: &VerifiedClone, request: &MountIntent) -> CloneOutcome {
    let claim = match context.claims.claim(verified.slot.as_str()) {
        Ok(claim) => claim,
        Err(refusal) => return CloneOutcome::Held { name: String::from(name), slot: verified.slot.clone(), said: refusal.message() },
    };
    if let Err(said) = ensure_the_slot_record(context, verified) {
        claim.release();
        let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Refused, None, Some(RECORD_STEP), None);
        return file_a_refusal(context, verified, said, RECORD_STEP, &receipt);
    }
    let outcome: AttachStepOutcome = context.attach.attach(&AttachRequest { checkout: context.checkout, slot: verified.slot.as_str(), clone: name, store: context.store });
    claim.release();
    return answer_the_attach(context, verified, request, outcome);
}

/// What:
///  Publish the slot's first-observation record when the store holds none,
///  and leave a record another sweep published standing.
/// Why:
///  The record is the folder's identity taken before the attach,
///  and a slot another sweep recorded first is the store's own loser path:
///  this sweep reads the record that won and carries on with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensureTheSlotRecord(context, verified);
/// ```
///
/// # Errors
/// The store's own refusal,
/// the first observation's walk refusal,
/// or a publication the store did not keep.
fn ensure_the_slot_record(context: &SweepContext<'_>, verified: &VerifiedClone) -> Result<(), String> {
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(context.store).map_err(|refusal| return refusal.message())?;
    return match store.slot(verified.slot.as_str()) {
        Ok(_recorded) => Ok(()),
        Err(StoreRefusal::NoSlotRecord { .. }) => publish_the_first_observation(context, verified),
        Err(refusal) => Err(refusal.message()),
    };
}

/// The publication of one slot's first observation.
///
/// # Errors
/// The first observation's walk refusal,
/// or a publication the store did not keep.
fn publish_the_first_observation(context: &SweepContext<'_>, verified: &VerifiedClone) -> Result<(), String> {
    let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: context.checkout, slot: verified.slot.as_str(), store: context.store };
    let observed = observe_the_clone_slot_under_root_control(&request).map_err(|refusal| return refusal.message())?;
    let published = publish_slot_identity_under_root_control(context.store, verified.slot.as_str(), &observed.slot_identity, &observed.parent_identity);
    return match published {
        Ok(()) => {
            debug(TAG, format!("recorded the first observation of slot `{}`", verified.slot).as_str());
            Ok(())
        }
        // Another sweep recorded the slot between this sweep's read and its link, and its record is the one that stands:
        // the attach that follows compares the folder against it, so a folder that moved is refused there.
        Err(StoreRefusal::SlotAlreadyRecorded { .. }) => Ok(()),
        Err(refusal) => Err(refusal.message()),
    };
}

/// What:
///  Turn one attach step's answer into the receipt and the sweep's outcome.
/// Why:
///  A done mount files the mount identity a caller confirms,
///  a mount the rollback took down files the failed step,
///  and a mount that could not be taken down is reported as retained with both
///  paths rather than as "nothing happened".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = answerTheAttach(context, verified, request, stepOutcome);
/// ```
fn answer_the_attach(context: &SweepContext<'_>, verified: &VerifiedClone, request: &MountIntent, outcome: AttachStepOutcome) -> CloneOutcome {
    return match outcome {
        AttachStepOutcome::Attached { mount } => {
            let identity: ReceiptMountIdentity = ReceiptMountIdentity {
                mount_id: mount.mount_id,
                device: mount.device,
                inode: mount.inode,
                source: mount.source,
                canonical_path: mount.observed_path.display().to_string(),
            };
            let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Done, Some(identity), None, None);
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
        AttachStepOutcome::Refused { said } => {
            let receipt: CloneMountReceipt = receipt_of(context, verified, request, ReceiptOutcome::Refused, None, Some("attach"), None);
            file_a_refusal(context, verified, said, "attach", &receipt)
        }
    };
}

/// File a refusal and answer it,
/// naming the receipt when even that could not be filed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = fileARefusal(context, verified, said, step, receipt);
/// ```
fn file_a_refusal(context: &SweepContext<'_>, verified: &VerifiedClone, said: String, step: &str, receipt: &CloneMountReceipt) -> CloneOutcome {
    warn(TAG, format!("slot `{}` refused at {step}: {said}", verified.slot).as_str());
    return match context.receipts.publish(receipt) {
        Ok(()) => CloneOutcome::Refused { name: verified.dataset_name.clone(), slot: Some(verified.slot.clone()), step: String::from(step), said },
        Err(refusal) => CloneOutcome::Failed {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(RECEIPT_STEP),
            said: format!("{said} The receipt could not be filed either: {}", refusal.message()),
        },
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

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_one_clone_tests.rs"]
mod tests;
