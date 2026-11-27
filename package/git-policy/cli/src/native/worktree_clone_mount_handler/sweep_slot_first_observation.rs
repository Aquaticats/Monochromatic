//! What:
//!  The slot's first observation:
//!  the record published when the store holds none for the slot,
//!  the walk that proves what the slot holds,
//!  and the Git pointer that must name this checkout's registration.
//! Why:
//!  The record is the folder's identity taken before the attach,
//!  and the first observation also checks what the slot holds:
//!  a bounded,
//! regular,
//! no-follow read of its Git pointer
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The sweep plan step 8").
//!  A record another sweep published first stands,
//!  and this sweep carries on with it rather than replacing it,
//!  so the attach compares its folder against the record that won.
//!  It is its own file so the turn stays short enough to read whole
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! ensureTheSlotRecord(context, verified);
//! ```

use super::sweep_clone_check::VerifiedClone;
use super::sweep_turn_context_and_outcome::SweepContext;
use crate::diagnostic_log::debug;
use crate::worktree_clone_mount::{
    HoldCloneSlotRequest,
    SlotIdentityStore,
    StoreRefusal,
    observe_the_clone_slot_under_root_control,
    publish_slot_identity_under_root_control,
    read_the_slot_git_pointer,
};
use std::path::PathBuf;

/// Module tag for diagnostics.
const TAG: &str = "sweepSlotFirstObservation";

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
pub(super) fn ensure_the_slot_record(
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
pub(super) fn publish_the_first_observation(
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
