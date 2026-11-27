//! What:
//!  One clone's whole turn of the sweep:
//!  read what the dataset says,
//!  verify it,
//!  read its durable intent,
//!  take the slot's claim,
//!  read the clone again under the claim,
//!  recognise a mount of this clone that already stands at the slot,
//!  claim this boot's first attach,
//!  record the slot's first observation,
//!  attach,
//!  record the first attach's outcome,
//!  answer the request with its receipt (the attach's answer before the release,
//!  a standing mount's after it),
//!  release the claim,
//!  and read the intent once more.
//! Why:
//!  The order is the design's:
//!  the checks,
//!  then the intent,
//!  then the claim under which the desired state and identities are rechecked,
//!  then the attach,
//!  and one receipt per clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  which says "Recheck desired state and identities under a per-slot root
//!  claim";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The sweep plan step 8").
//!  A malformed intent is refused before any filesystem operation on its behalf,
//!  an inherited intent authorizes nothing,
//!  and a slot another sweep holds is left to that sweep rather than queued.
//!  A refusal files no receipt,
//!  except an unavailable key,
//!  which files a `refused` receipt naming that step so the waiting command
//!  learns of it at once
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Receipts for refusals").
//!  The holder reads the intent again after it releases the claim,
//!  so a request that arrived while it held the slot is acted on in a second
//!  turn rather than lost
//!  (the same decision,
//!  "A slot whose claim is held").
//!  A clone whose durable intent is `mount` keeps that intent after it
//!  attached,
//!  so the ordinary sweep is one over a clone already mounted at its slot:
//!  that mount is recognised as this clone's own and the `done` receipt is
//!  kept,
//!  instead of the walk refusing the slot and a `refused` receipt overwriting
//!  it.
//!  Where that read answers nothing and the walk later stops at the slot,
//!  the attach step asks the same proof and answers the mount it finds,
//!  so a mount this clone already holds at its slot is answered `done` under
//!  the request's nonce rather than refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = sweepOneClone(context, 'zpcachyos/…/clones/one');
//! ```

use super::sweep_attach_answer::answer_the_attach;
use super::sweep_attach_step::{
    AttachRequest,
    AttachStepOutcome,
};
use super::sweep_clone_check::{
    VerifiedClone,
    verify,
};
use super::sweep_dataset_facts::{
    CloneObjectFacts,
    facts_of,
};
use super::sweep_one_clone_answers::{
    answer_the_standing_mount,
    record_the_first_attach,
    the_first_attach_outcome,
    the_first_attach_refusal,
};
use super::sweep_refused_clone_answers::refused_by_the_checks;
use super::sweep_unmount_under_the_claim::answer_unmount;
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::start_probe::first_attach_record::FirstAttachClaim;
use crate::worktree_clone_mount::{
    HoldCloneSlotRequest,
    LookAgainAtSlotRequest,
    SlotIdentityStore,
    StoreRefusal,
    observe_the_clone_slot_under_root_control,
    publish_slot_identity_under_root_control,
    read_the_slot_git_pointer,
};
use crate::worktree_clone_mount_protocol::{
    IntentReading,
    MountIntent,
    MountOperation,
    read_intent,
};
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "sweepOneClone";

/// What one clone's turn reads and writes,
/// the outcome it answers with,
/// and the step words a receipt names,
/// which every caller of this turn names by this file.
pub use super::sweep_turn_context_and_outcome::{
    CloneOutcome,
    IDENTITY_RECORD_STEP,
    INTENT_STEP,
    RECEIPT_STEP,
    SweepContext,
};

#[cfg(test)]
pub(crate) use super::sweep_standing_mount_source::{
    MountSourceAnswer,
    MountSourceByNumber,
};
#[cfg(test)]
pub(crate) use super::sweep_standing_mounts::{
    LiveMountTableSource,
    MountTableSource,
    is_this_clones_own_mount,
};
pub use super::sweep_standing_mounts::{
    LiveStandingMounts,
    StandingMounts,
};

/// What:
///  Read one clone,
///  verify it,
///  and answer what its intent asked for.
/// Why:
///  One function is the whole turn so the order cannot differ between the
///  sweep a boot runs and the sweep a ZED event starts,
///  and the holder reads the intent again after it releases the claim at most
///  once,
///  so a change made during the turn is acted on and a change made during the
///  second turn waits for the next trigger rather than looping here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = sweepOneClone(context, name);
/// ```
pub fn sweep_one_clone(
    context: &SweepContext<'_>,
    name: &str,
) -> CloneOutcome {
    return sweep_one_clone_once(context, name, true);
}

/// The turn itself,
/// with the one re-read after the claim's release allowed or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = sweepOneCloneOnce(context, name, mayReadAgain);
/// ```
fn sweep_one_clone_once(
    context: &SweepContext<'_>,
    name: &str,
    may_read_again: bool,
) -> CloneOutcome {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => {
            warn(
                TAG,
                format!("`{name}` could not be read: {reason}").as_str(),
            );
            return CloneOutcome::Refused {
                name: String::from(name),
                slot: None,
                step: String::from("read"),
                said: reason.message(),
            };
        }
    };
    let verified: VerifiedClone =
        match verify(&facts, context.parent_dataset, context.checkout_dataset) {
            Ok(verified) => verified,
            Err(refusal) => return refused_by_the_checks(context, name, &facts, &refusal),
        };
    let request: MountIntent = match read_intent(Some(&facts.properties), name) {
        IntentReading::Absent => {
            return CloneOutcome::NothingAsked {
                name: String::from(name),
                slot: verified.slot,
                why: String::from("no local mount intent"),
            };
        }
        IntentReading::Inherited {
            from,
        } => {
            return CloneOutcome::NothingAsked {
                name: String::from(name),
                slot: verified.slot,
                why: format!("the mount intent is inherited from {from}"),
            };
        }
        IntentReading::Refused(refusal) => {
            return CloneOutcome::Refused {
                name: String::from(name),
                slot: Some(verified.slot),
                step: String::from(INTENT_STEP),
                said: refusal.message(),
            };
        }
        IntentReading::Request(request) => request,
    };
    if request.operation == MountOperation::Unmount {
        return answer_unmount(context, &verified, &request);
    }
    return mount_the_clone(context, name, &verified, &request, may_read_again);
}

/// What:
///  Take the claim,
///  read the clone again under it,
///  recognise a mount of this clone that already stands at the slot,
///  claim this boot's first attach,
///  record the slot's first observation,
///  attach,
///  record the outcome,
///  and release.
/// Why:
///  The claim is what lets parallel sweeps work without acting on the same
///  mount twice,
///  and the desired state and the identities are rechecked under it so a sweep
///  that waited attaches nothing an intent changed in the meantime asked it not
///  to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = mountTheClone(context, name, verified, request, mayReadAgain);
/// ```
fn mount_the_clone(
    context: &SweepContext<'_>,
    name: &str,
    verified: &VerifiedClone,
    request: &MountIntent,
    may_read_again: bool,
) -> CloneOutcome {
    let claim = match context.claims.claim(verified.slot.as_str()) {
        Ok(claim) => claim,
        Err(refusal) => {
            return CloneOutcome::Held {
                name: String::from(name),
                slot: verified.slot.clone(),
                said: refusal.message(),
            };
        }
    };
    let under_the_claim: (VerifiedClone, MountIntent) = match reread_under_the_claim(context, name)
    {
        Ok(reread) => reread,
        Err(outcome) => {
            claim.release();
            return turn_again_after_the_release(context, name, request, outcome, may_read_again);
        }
    };
    if under_the_claim.0.dataset != verified.dataset
        || under_the_claim.1.nonce != request.nonce
        || under_the_claim.1.operation != request.operation
    {
        claim.release();
        let after_the_release: &str = if may_read_again {
            "and the intent is read again after the claim is released"
        } else {
            "and the next trigger reads it again, because this turn has already read it once more"
        };
        let said: String = format!(
            "the clone `{name}` was read as GUID {} asking `{}` under nonce {} before this sweep took slot `{}`, and under the claim it reads as GUID {} asking `{}` under nonce {}. Nothing was attached on the first reading, {after_the_release}",
            verified.dataset.file_name(),
            request.operation.word(),
            request.nonce.as_str(),
            verified.slot,
            under_the_claim.0.dataset.file_name(),
            under_the_claim.1.operation.word(),
            under_the_claim.1.nonce.as_str()
        );
        warn(TAG, said.as_str());
        let refused: CloneOutcome = CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot.clone()),
            step: String::from(INTENT_STEP),
            said,
        };
        return turn_again_after_the_release(context, name, request, refused, may_read_again);
    }
    if let Some(mount) = context.standing.standing_mount_of(
        &LookAgainAtSlotRequest {
            checkout: context.checkout,
            slot: verified.slot.as_str(),
        },
        verified.dataset_name.as_str(),
    ) {
        claim.release();
        let answered: CloneOutcome = answer_the_standing_mount(context, verified, request, mount);
        return turn_again_after_the_release(context, name, request, answered, may_read_again);
    }
    // This boot's first attach is claimed after the key check that the re-read ran and before the walk that opens the
    // slot, which is the order the start-probe design states; a sweep that finds this clone already mounted above has
    // claimed nothing, because it attached nothing.
    let first_attach: Option<FirstAttachClaim> = context.first_attach.claim();
    if let Err(said) = ensure_the_slot_record(context, verified) {
        claim.release();
        record_the_first_attach(
            context,
            first_attach,
            &the_first_attach_refusal(verified, IDENTITY_RECORD_STEP, None),
        );
        let refused: CloneOutcome = CloneOutcome::Refused {
            name: verified.dataset_name.clone(),
            slot: Some(verified.slot.clone()),
            step: String::from(IDENTITY_RECORD_STEP),
            said,
        };
        return turn_again_after_the_release(context, name, request, refused, may_read_again);
    }
    let outcome: AttachStepOutcome = context.attach.attach(&AttachRequest {
        checkout: context.checkout,
        slot: verified.slot.as_str(),
        clone: name,
        store: context.store,
    });
    record_the_first_attach(
        context,
        first_attach,
        &the_first_attach_outcome(verified, &outcome),
    );
    // The answer is filed before the intent is read once more, so a mount this turn made is answered under the nonce it
    // was made for even when a request arrived meanwhile.
    let answered: CloneOutcome = answer_the_attach(context, verified, request, outcome);
    claim.release();
    return turn_again_after_the_release(context, name, request, answered, may_read_again);
}

/// What:
///  Turn once more after every release of a claim this turn took,
///  when the intent changed while the claim was held.
/// Why:
///  The project decided the holder reads the intent again after it releases
///  the claim,
///  so a request that arrived during a sweep is acted on rather than left to a
///  trigger that may not come.
///  The same rule holds wherever the turn releases a claim it took,
///  not only after an attach:
///  a standing mount,
///  a changed reading under the claim and a refused re-read all release it.
///  One turn may re-read,
///  so a request that keeps changing cannot make the sweep loop.
///  A turn whose own answer is a failure keeps that answer,
///  because a retained mount must reach the run's failure count rather than be
///  replaced by the second turn's refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = turnAgainAfterTheRelease(context, name, request, answered, mayReadAgain);
/// ```
fn turn_again_after_the_release(
    context: &SweepContext<'_>,
    name: &str,
    request: &MountIntent,
    answered: CloneOutcome,
    may_read_again: bool,
) -> CloneOutcome {
    if !may_read_again || !the_intent_changed(context, name, request) {
        return answered;
    }
    debug(TAG, format!("the intent of `{name}` changed while the slot was claimed, so the sweep turns once more").as_str());
    let second: CloneOutcome = sweep_one_clone_once(context, name, false);
    if matches!(answered, CloneOutcome::Failed { .. }) {
        return answered;
    }
    return second;
}

/// What:
///  Read the clone's identity and its intent again under the claim,
///  refusing where the first reading passed and the second does not.
/// Why:
///  A sweep that waited on a claim,
///  or ran slowly,
///  can act on facts the clone no longer says,
///  and the removal protocol relies on a standing `unmount` intent to exclude
///  new attaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [verified, request] = rereadUnderTheClaim(context, name);
/// ```
///
/// # Errors
/// The clone could not be read,
/// the checks refused it,
/// or its intent is absent,
/// inherited or malformed now.
fn reread_under_the_claim(
    context: &SweepContext<'_>,
    name: &str,
) -> Result<(VerifiedClone, MountIntent), CloneOutcome> {
    let facts: CloneObjectFacts = match context.reader.identity(name) {
        Ok(identity) => facts_of(name, &identity),
        Err(reason) => {
            return Err(CloneOutcome::Refused {
                name: String::from(name),
                slot: None,
                step: String::from("read"),
                said: reason.message(),
            });
        }
    };
    let verified: VerifiedClone = verify(&facts, context.parent_dataset, context.checkout_dataset)
        .map_err(|refusal| {
            warn(TAG, refusal.message().as_str());
            return CloneOutcome::Refused {
                name: String::from(name),
                slot: None,
                step: String::from(refusal.step()),
                said: refusal.message(),
            };
        })?;
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(request) => Ok((verified, request)),
        IntentReading::Absent => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: String::from(
                "the clone's mount intent is gone under the claim, so nothing was attached on the reading that saw it",
            ),
        }),
        IntentReading::Inherited {
            from,
        } => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: format!(
                "the clone's mount intent is inherited from {from} under the claim, so nothing was attached"
            ),
        }),
        IntentReading::Refused(refusal) => Err(CloneOutcome::Refused {
            name: String::from(name),
            slot: Some(verified.slot),
            step: String::from(INTENT_STEP),
            said: refusal.message(),
        }),
    };
}

/// Whether the clone's intent changed after the request this turn acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = theIntentChanged(context, name, request); // true when a read answers another request
/// ```
fn the_intent_changed(
    context: &SweepContext<'_>,
    name: &str,
    request: &MountIntent,
) -> bool {
    let Ok(identity) = context.reader.identity(name) else {
        return false;
    };
    let facts: CloneObjectFacts = facts_of(name, &identity);
    return match read_intent(Some(&facts.properties), name) {
        IntentReading::Request(now) => {
            now.nonce != request.nonce || now.operation != request.operation
        }
        IntentReading::Absent
        | IntentReading::Inherited {
            ..
        }
        | IntentReading::Refused(_) => true,
    };
}

/// What:
///  Publish the slot's first-observation record when the store holds none,
///  and leave a record another sweep published standing.
/// Why:
///  The record is the folder's identity taken before the attach,
///  and the first observation also checks what the slot holds:
///  a bounded,
/// regular,
/// no-follow read of its Git pointer,
///  which must name this checkout's registration of the slot
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///  A slot another sweep recorded first is the store's own loser path:
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
/// a slot whose Git pointer is not this checkout's registration,
/// or a publication the store did not keep.
fn ensure_the_slot_record(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
) -> Result<(), String> {
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(context.store)
        .map_err(|refusal| return refusal.message())?;
    return match store.slot(verified.slot.as_str()) {
        Ok(_recorded) => Ok(()),
        Err(StoreRefusal::NoSlotRecord {
            ..
        }) => publish_the_first_observation(context, verified),
        Err(refusal) => Err(refusal.message()),
    };
}

/// The publication of one slot's first observation.
///
/// # Errors
/// The first observation's walk refusal,
/// a Git pointer this route does not accept,
/// or a publication the store did not keep.
fn publish_the_first_observation(
    context: &SweepContext<'_>,
    verified: &VerifiedClone,
) -> Result<(), String> {
    let request: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest {
        checkout: context.checkout,
        slot: verified.slot.as_str(),
        store: context.store,
    };
    let observed = observe_the_clone_slot_under_root_control(&request)
        .map_err(|refusal| return refusal.message())?;
    let registration: PathBuf =
        read_the_slot_git_pointer(&observed.slot, context.checkout, verified.slot.as_str())
            .map_err(|refusal| return refusal.message())?;
    debug(
        TAG,
        format!(
            "the slot `{}` holds the registration {}",
            verified.slot,
            registration.display()
        )
        .as_str(),
    );
    let published = publish_slot_identity_under_root_control(
        context.store,
        verified.slot.as_str(),
        &observed.slot_identity,
        &observed.parent_identity,
    );
    return match published {
        Ok(()) => {
            debug(
                TAG,
                format!("recorded the first observation of slot `{}`", verified.slot).as_str(),
            );
            Ok(())
        }
        // Another sweep recorded the slot between this sweep's read and its link, and its record is the one that stands:
        // the attach that follows compares the folder against it, so a folder that moved is refused there.
        Err(StoreRefusal::SlotAlreadyRecorded {
            ..
        }) => Ok(()),
        Err(refusal) => Err(refusal.message()),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_one_clone_tests.rs"]
mod tests;
