//! What:
//!  The walk to one clone slot:
//!  the parent folder and the slot compared against the identities the store
//!  recorded,
//!  the two descriptors that must name one folder,
//!  and the first observation that records a slot the store holds nothing for.
//! Why:
//!  Every check the design asks for before an attach runs here:
//!  the no-crossing walk,
//!  the recorded parent identity,
//!  the recorded slot identity,
//!  and the proof that the reading descriptor and the mount target name one
//!  folder,
//!  so a later path lookup that reaches another folder cannot pass unnoticed
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
//!  "The step").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdCloneSlot({ checkout, slot, store });
//! const observed = observeTheCloneSlot({ checkout, slot, store });   // no slot record is compared
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::clone_slot_walk::{
    HeldCloneSlot,
    HoldCloneSlotRequest,
    SlotRecord,
};
use super::clone_slot_walk_names::{
    WORKTREES_PARENT_NAMES,
    folder_refusal,
    slot_component,
    slot_name_walk_refusal,
};
use super::clone_slot_walk_refusals::{
    CloneSlotRefusal,
    WalkRefusal,
    WalkRefusalReason,
    fields_refusal,
};
use super::clone_slot_walk_to_the_parent::walk_to_the_parent;
use super::slot_identity_store::SlotIdentityStore;
use crate::diagnostic_log::debug;
use crate::fs_beneath::{
    ObjectId,
    device_of,
    handle_directory_at,
    identity_of,
    open_directory_at,
};
use std::ffi::OsStr;
use std::path::PathBuf;

/// Import the tag this module's diagnostics carry.
use crate::worktree_clone_mount::clone_slot_walk_to_the_parent::TAG;

// What:     the hook a control sets to catch the moment the slot's reading
//           descriptor is open and the target is not, declared with the shapes
//           it holds the walk between; a test build only, because the walk
//           reads it only in a test build.
// Why:      the control makes the slot's name hold another folder at that
//           moment, which is where an account renaming the slot ends one walk.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AFTER_THE_SLOT_IS_READ } from './cloneSlotWalk.js';
// ```
#[cfg(test)]
use super::clone_slot_walk::AFTER_THE_SLOT_IS_READ;

/// What:
///  Walk to the slot,
///  compare the parent folder and the slot against the recorded identities,
///  and keep the handles the attach acts on.
/// Why:
///   Every check the design asks for before an attach runs here:
///  the no-crossing walk,
///  the recorded parent identity,
///  the recorded slot identity,
///  and the proof that the reading descriptor and the mount target name one folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdCloneSlot({ checkout, slot, store });
/// ```
///
/// # Errors
/// The walk's refusal,
/// the store's refusal,
/// a parent or slot that is not the recorded folder,
/// or a slot whose two descriptors name different folders.
pub fn hold_clone_slot(
    request: &HoldCloneSlotRequest<'_>
) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    // The checkout is what a person who ran the command in a moved folder must be told about, so its refusal
    // comes before the store's: a store the walk never reached is not their problem.
    let (checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(request.checkout)?;
    let store: SlotIdentityStore =
        SlotIdentityStore::open(request.store).map_err(CloneSlotRefusal::Store)?;
    return hold_the_walk(
        request,
        checkout_handle,
        parent,
        canonical,
        &store,
        SlotRecord::Compared,
    );
}

/// What:
///  The rest of the walk with the folders it opened and the store the caller holds,
///  which is how the route opens the folder under root control once and walks through it.
/// Why:
///  The folder is opened once and every record is named relative to it,
///  so a caller that opened it under root control walks against the very folder it judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdTheWalk({ checkout, slot }, checkoutHandle, parent, canonical, store);
/// ```
///
/// # Errors
/// Every refusal the walk answers after the parent folder is reached,
/// and every record's read refusal.
fn hold_the_walk(
    request: &HoldCloneSlotRequest<'_>,
    checkout_handle: std::fs::File,
    parent: std::fs::File,
    canonical: PathBuf,
    store: &SlotIdentityStore,
    slot_record: SlotRecord,
) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    let parent_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&parent)
        .map_err(|error| return fields_refusal(".claude/worktrees", &error))?;
    let slot_name: &OsStr = OsStr::new(request.slot);
    let component: String = slot_component(request.slot);
    if let Some(refusal) = slot_name_walk_refusal(component.as_str(), request.slot) {
        return Err(refusal);
    }
    let reading: std::fs::File = open_directory_at(&parent, slot_name)
        .map_err(|error| return folder_refusal(component.as_str(), error))?;
    let checkout_device: u64 = device_of(&checkout_handle)
        .map_err(|error| return fields_refusal(canonical.display().to_string().as_str(), &error))?;
    let slot_device: u64 =
        device_of(&reading).map_err(|error| return fields_refusal(component.as_str(), &error))?;
    if slot_device != checkout_device {
        return Err(CloneSlotRefusal::Walk(WalkRefusal {
            component,
            reason: WalkRefusalReason::BoundaryCrossed,
            detail: String::new(),
        }));
    }
    let slot_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&reading)
        .map_err(|error| return fields_refusal(component.as_str(), &error))?;
    let recorded_parent: CloneFolderIdentity = store.parent().map_err(CloneSlotRefusal::Store)?;
    let parent_fields = recorded_parent.differences(&parent_identity);
    if !parent_fields.is_empty() {
        return Err(CloneSlotRefusal::ParentIdentity {
            checkout: canonical.clone(),
            recorded: Box::new(recorded_parent),
            found: Box::new(parent_identity),
            fields: parent_fields,
        });
    }
    if slot_record == SlotRecord::Compared {
        let recorded_slot: CloneFolderIdentity =
            store.slot(request.slot).map_err(CloneSlotRefusal::Store)?;
        let slot_fields = recorded_slot.differences(&slot_identity);
        if !slot_fields.is_empty() {
            return Err(CloneSlotRefusal::SlotIdentity {
                slot: request.slot.to_string(),
                recorded: Box::new(recorded_slot),
                found: Box::new(slot_identity),
                fields: slot_fields,
            });
        }
    }
    #[cfg(test)]
    AFTER_THE_SLOT_IS_READ.with(|hook| {
        if let Some(after) = hook.borrow().as_ref() {
            after();
        }
    });
    let target: std::fs::File = handle_directory_at(&parent, slot_name)
        .map_err(|error| return folder_refusal(component.as_str(), error))?;
    let first: ObjectId =
        identity_of(&reading).map_err(|error| return fields_refusal(component.as_str(), &error))?;
    let second: ObjectId =
        identity_of(&target).map_err(|error| return fields_refusal(component.as_str(), &error))?;
    if first != second {
        return Err(CloneSlotRefusal::TargetAnotherObject {
            slot: request.slot.to_string(),
            first: first.encode(),
            second: second.encode(),
        });
    }
    let canonical_slot: PathBuf = canonical
        .join(WORKTREES_PARENT_NAMES[0])
        .join(WORKTREES_PARENT_NAMES[1])
        .join(request.slot);
    debug(
        TAG,
        "held the checkout, the worktrees folder and one slot by descriptor",
    );
    return Ok(HeldCloneSlot {
        checkout: checkout_handle,
        parent,
        slot: reading,
        target,
        parent_identity,
        slot_identity,
        canonical_slot,
    });
}

/// What:
///  The route's walk to a clone slot:
///  the recorded identities are answered as root's own before the walk compares anything,
///  and then the walk runs.
/// Why:
///   The route runs as root and the store decides which folder a clone may be attached to,
///  so a store this account could rewrite refuses here rather than deciding the walk;
///  `hold_clone_slot` is the walk itself and reads the store with the grammar's reader,
///  which is what a control on fixture files drives.
///  The folder is opened once,
///  checked by `open_under_root_control`,
///  and every record is named relative to that one descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdCloneSlotUnderRootControl({ checkout, slot, store });
/// ```
///
/// # Errors
/// The store's refusal,
/// and every refusal `hold_clone_slot` answers.
pub fn hold_clone_slot_under_root_control(
    request: &HoldCloneSlotRequest<'_>
) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(request.store)
        .map_err(CloneSlotRefusal::Store)?;
    let (checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(request.checkout)?;
    return hold_the_walk(
        request,
        checkout_handle,
        parent,
        canonical,
        &store,
        SlotRecord::Compared,
    );
}

/// What:
///  Walk to a slot the store holds no record for yet,
///  comparing the parent and recording nothing,
///  so the sweep can publish the slot's first observation from the folder it
///  opened.
/// Why:
///  The first observation is the plan's step 8 record:
///  root opens and checks the account's underlay before the attach,
///  records its identity once,
///  and every later walk compares that record
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount";
///  `doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
///  "The step").
///  The parent is still compared,
///  because a record taken through a parent nobody compared is exactly what the
///  publication's own compare exists to prevent,
///  and the walk refuses before it reaches the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const observed = observeTheCloneSlot({ checkout, slot, store }); // no slot record is compared
/// publishSlotIdentity(store, slot, observed.slotIdentity, observed.parentIdentity);
/// ```
///
/// # Errors
/// The walk's refusal,
/// the store's refusal,
/// and a parent folder that is not the recorded one.
pub fn observe_the_clone_slot(
    request: &HoldCloneSlotRequest<'_>
) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    let store: SlotIdentityStore =
        SlotIdentityStore::open(request.store).map_err(CloneSlotRefusal::Store)?;
    let (checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(request.checkout)?;
    return hold_the_walk(
        request,
        checkout_handle,
        parent,
        canonical,
        &store,
        SlotRecord::FirstObservation,
    );
}

/// What:
///  The route's first observation of a slot:
///  the store is answered as root's own before the walk reaches anything,
///  and then the walk records the slot it opened.
/// Why:
///  The first observation is what every later walk compares,
///  so it is taken only into a folder this account could not have rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const observed = observeTheCloneSlotUnderRootControl({ checkout, slot, store });
/// ```
///
/// # Errors
/// The store's root-control refusal,
/// and every refusal the first observation answers.
pub fn observe_the_clone_slot_under_root_control(
    request: &HoldCloneSlotRequest<'_>
) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    let store: SlotIdentityStore = SlotIdentityStore::open_under_root_control(request.store)
        .map_err(CloneSlotRefusal::Store)?;
    let (checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(request.checkout)?;
    return hold_the_walk(
        request,
        checkout_handle,
        parent,
        canonical,
        &store,
        SlotRecord::FirstObservation,
    );
}
