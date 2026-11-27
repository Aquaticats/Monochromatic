//! What:
//!  What the kernel shows at the slot after an attach:
//!  the mount the held target's name resolves to beneath the held parent,
//!  and the walk repeated at the canonical slot,
//!  which crosses a mount at the slot itself and nowhere above it.
//! Why:
//!  The held target's own descriptor cannot say which mount stands at the
//!  slot,
//!  because a descriptor keeps naming the mount it was opened on,
//!  so the number is read through the parent's name,
//!  and the post-attach check compares that with the mount the attach recorded
//!  (`package/git-policy/cli/SPEC.md`,
//!  "The post-attach check").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mount = mountIdAtTheHeldTarget(held.parent, 'one'); // the mount now at that name
//! const again = lookAgainAtTheSlot({ checkout, slot });
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::clone_slot_walk::{
    FreshSlotLookup,
    LookAgainAtSlotRequest,
};
use super::clone_slot_walk_names::{
    folder_refusal,
    slot_component,
    slot_name_walk_refusal,
};
use super::clone_slot_walk_refusals::{
    CloneSlotRefusal,
    fields_refusal,
};
use super::clone_slot_walk_to_the_parent::walk_to_the_parent;
use crate::fs_beneath_mount_facts::{
    mount_id_of,
    observed_path,
    unique_mount_number_of,
};
use crate::fs_beneath_walk_chain::open_directory_at_a_mount;
use std::ffi::OsStr;
use std::path::PathBuf;

/// What:
///  The mount the kernel resolves at the held target's name beneath the held parent.
/// Why:
///   After `move_mount` the attach's mount stands at that name inside the parent folder the walk held,
///  so this is the number that proves the held folder received the mount;
///  the held target's own descriptor cannot answer it,
///  because a descriptor keeps naming the mount it was opened on,
///  and the guest read the parent's mount from it after a real attach
///  (`package/git-policy/cli/SPEC.md`,
/// "The post-attach check").
///  The hop is `open_directory_at_a_mount`,
///  which crosses a mount at the slot's own name while every folder above stays the one the walk reached,
///  so a mount attached beside the slot is not reached here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mount = mountIdAtTheHeldTarget(held.parent, 'one'); // the mount now at that name, not the parent's
/// ```
///
/// # Errors
/// The walk's refusal of the slot's name,
/// an open that names another object or no folder,
/// or the folder's own facts not being readable.
pub fn mount_id_at_the_held_target(
    held_parent: &std::fs::File,
    slot: &str,
) -> Result<u64, CloneSlotRefusal> {
    let component: String = slot_component(slot);
    if let Some(refusal) = slot_name_walk_refusal(component.as_str(), slot) {
        return Err(refusal);
    }
    let slot_folder: std::fs::File = open_directory_at_a_mount(held_parent, OsStr::new(slot))
        .map_err(|error| return folder_refusal(component.as_str(), error))?;
    return mount_id_of(&slot_folder)
        .map_err(|error| return fields_refusal(component.as_str(), &error));
}

/// What:
///  Walk to the canonical slot again and read what the kernel finds there,
///  crossing a mount only at the slot itself.
/// Why:
///   After an attach the slot is a mount point,
///  so the last hop must cross it while every folder above stays the one the walk reached;
///  what this returns is compared with the mount the attach recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const again = lookAgainAtTheSlot({ checkout, slot });
/// ```
///
/// # Errors
/// The walk's refusal at any component,
/// including a slot that is not there.
pub fn look_again_at_the_slot(
    request: &LookAgainAtSlotRequest<'_>
) -> Result<FreshSlotLookup, CloneSlotRefusal> {
    let (_checkout, parent, _canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(request.checkout)?;
    let parent_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&parent)
        .map_err(|error| return fields_refusal(".claude/worktrees", &error))?;
    let slot_name: &OsStr = OsStr::new(request.slot);
    let component: String = slot_component(request.slot);
    if let Some(refusal) = slot_name_walk_refusal(component.as_str(), request.slot) {
        return Err(refusal);
    }
    let slot: std::fs::File = open_directory_at_a_mount(&parent, slot_name)
        .map_err(|error| return folder_refusal(component.as_str(), error))?;
    let slot_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&slot)
        .map_err(|error| return fields_refusal(component.as_str(), &error))?;
    let mount_id: u64 =
        mount_id_of(&slot).map_err(|error| return fields_refusal(component.as_str(), &error))?;
    let parent_mount_id: u64 =
        mount_id_of(&parent).map_err(|error| return fields_refusal(".claude/worktrees", &error))?;
    let observed: PathBuf =
        observed_path(&slot).map_err(|error| return fields_refusal(component.as_str(), &error))?;
    // A kernel that reports no unique number leaves it out rather than answering the reused one, and the caller then
    // reads the live table for this mount.
    let unique_mount_number: Option<u64> = unique_mount_number_of(&slot).ok();
    return Ok(FreshSlotLookup {
        parent_identity,
        slot_identity,
        mount_id,
        parent_mount_id,
        unique_mount_number,
        observed_path: observed,
    });
}
