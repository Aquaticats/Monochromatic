//! What:
//!  The shapes one walk to a clone slot takes and hands back:
//!  the pinned checkout and the slot's name,
//!  the handles and identities the walk kept,
//!  what the kernel finds at the canonical slot when the walk is repeated,
//!  the `.claude/worktrees` folder an adoption walks,
//!  and whether the walk compares the slot against its record.
//! Why:
//!  The mount must not resolve a path the account can change and must attach to
//!  the opened folder rather than to a path,
//!  so a walk hands its caller the descriptors it proved and the identities it
//!  compared,
//!  and the caller acts on those
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The steps that fill them are `clone_slot_walk_to_the_parent.rs`,
//!  `clone_slot_walk_to_the_slot.rs` and
//!  `clone_slot_walk_after_the_attach.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Request = { checkout: string; slot: string; store: string };
//! const held: HeldCloneSlot = holdCloneSlot({ checkout, slot, store });
//! const again: FreshSlotLookup = lookAgainAtTheSlot({ checkout, slot });
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  One walk to a clone slot:
///  the pinned checkout,
///  the slot's name below `.claude/worktrees`,
///  and the store the recorded identities are read from.
/// Why:
///   The store's path is given by the caller so a control injects a fixture path
///  and nothing on a host is read or written under `/var/lib`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Request = { checkout: string; slot: string; store: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct HoldCloneSlotRequest<'a> {
    /// The checkout the slot hangs below,
    /// as Git reports it.
    pub checkout: &'a Path,
    /// The slot's name below `.claude/worktrees`.
    pub slot: &'a str,
    /// The store of recorded identities.
    pub store: &'a Path,
}

/// What:
///  The handles and identities one walk to a slot produced.
/// Why:
///   The attach and its post-check compare the parent and the slot against these very folders,
///  so a later path lookup that reaches another folder cannot pass unnoticed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldSlot = { checkout, parent, slot, target, parentIdentity, slotIdentity, canonicalSlot };
/// ```
#[derive(Debug)]
pub struct HeldCloneSlot {
    /// The checkout,
    /// held as the walk's root.
    pub checkout: std::fs::File,
    /// The `.claude/worktrees` folder.
    pub parent: std::fs::File,
    /// The slot folder,
    /// opened for reading so its ACLs can be read.
    pub slot: std::fs::File,
    /// The same slot as the mount target,
    /// opened without reading rights.
    pub target: std::fs::File,
    /// The parent folder's identity,
    /// compared with the record and kept for the post-attach check.
    pub parent_identity: CloneFolderIdentity,
    /// The slot folder's identity,
    /// compared with the record and kept for the post-attach check.
    pub slot_identity: CloneFolderIdentity,
    /// The slot's canonical path,
    /// which the fresh walk after the attach must reach.
    pub canonical_slot: PathBuf,
}

/// What:
///  What the kernel finds at the canonical slot when the walk is repeated:
///  the parent folder's identity,
///  the slot folder's identity,
///  and the mount the slot is on.
/// Why:
///   The post-attach check compares this answer with the mount the attach recorded,
///  which is what separates a mount at the slot from one that landed beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FreshLookup = { parentIdentity, slotIdentity, mountId, parentMountId, observedPath };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshSlotLookup {
    /// The `.claude/worktrees` folder's identity now.
    pub parent_identity: CloneFolderIdentity,
    /// The slot folder's identity now,
    /// which after an attach is the mounted clone's root.
    pub slot_identity: CloneFolderIdentity,
    /// The mount the slot is on,
    /// from the descriptor's `/proc/self/fdinfo`.
    pub mount_id: u64,
    /// The mount the walk's parent folder is on.
    pub parent_mount_id: u64,
    /// The unique number of the mount the slot is on,
    /// which the kernel never reuses,
    /// or nothing where this kernel reports none
    /// (`statx(2)`,
    /// `STATX_MNT_ID_UNIQUE`).
    pub unique_mount_number: Option<u64>,
    /// The path the slot's descriptor names now.
    pub observed_path: PathBuf,
}

/// What:
///  The request of the second walk,
///  which reads the canonical slot without comparing it with any record.
/// Why:
///   A post-attach check must see what the kernel finds now,
///  not what the store expected,
///  so this walk takes no store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LookAgainRequest = { checkout: string; slot: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LookAgainAtSlotRequest<'a> {
    /// The checkout the slot hangs below.
    pub checkout: &'a Path,
    /// The slot's name below `.claude/worktrees`.
    pub slot: &'a str,
}

/// What:
///  The `.claude/worktrees` folder the parent walk held:
///  its descriptor,
///  the identity the walk read from it,
///  and the canonical path it was reached by.
/// Why:
///   The descriptor is what keeps the judged folder the one that was judged,
///  even while the account above it renames or replaces a name;
///  the identity is what the adoption compares with the record and with a fresh lookup,
///  and the path is what a message names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HeldParent = { folder: File; identity: CloneFolderIdentity; canonical: string };
/// ```
#[derive(Debug)]
pub struct HeldWorktreesParent {
    /// The `.claude/worktrees` folder,
    /// held open for reading.
    pub folder: std::fs::File,
    /// The device,
    /// inode,
    /// owner,
    /// mode and ACLs the walk read for it.
    pub identity: CloneFolderIdentity,
    /// The canonical path of that folder,
    /// as the walk reached it.
    pub canonical: PathBuf,
}

/// What:
///  Whether the walk compares the slot against the record the store holds.
/// Why:
///  The sweep's first observation of a slot is the walk that records it,
///  so that one walk reaches the slot before any record exists for it
///  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
///  "The sweep plan step 8").
///  Every other walk of a slot compares it,
///  and the type says which walk a caller is taking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SlotRecord = 'compare' | 'first-observation';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotRecord {
    /// The store holds this slot's record,
    /// and the walk refuses any folder that is not it.
    Compared,
    /// The sweep is about to publish this slot's first record,
    /// so there is nothing to compare yet.
    FirstObservation,
}

#[cfg(test)]
thread_local! {
    /// A control may make the slot's name hold another folder after its reading descriptor was opened,
    /// which is where an account renaming the slot ends one walk.
    pub(crate) static AFTER_THE_SLOT_IS_READ: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

// What:     every name a control reaches through this module, re-exported where
//           it was declared before the split, because the walk's steps and the
//           two names it answers with now live in siblings; a test build only,
//           because no code here reads them.
// Why:      the control file globs this module and the guest arms name it by
//           path, so a step or a name a control drives must still reach it
//           through the module the control reads.
//
// In TS you'd write (pseudocode):
// ```ts
// export { CloneSlotRefusal, WalkRefusal, WalkRefusalReason, identityOf } from './theWalk.js';   // for the controls
// export * from './cloneSlotWalkToTheParent.js'; export * from './cloneSlotWalkToTheSlot.js';
// ```
#[cfg(test)]
pub use super::clone_slot_walk_after_the_attach::{
    look_again_at_the_slot,
    mount_id_at_the_held_target,
};
#[cfg(test)]
pub use super::clone_slot_walk_names::WORKTREES_PARENT_NAMES;
#[cfg(test)]
pub use super::clone_slot_walk_refusals::{
    CloneSlotRefusal,
    WalkRefusal,
    WalkRefusalReason,
};
#[cfg(test)]
pub use super::clone_slot_walk_to_the_parent::hold_the_worktrees_parent;
#[cfg(test)]
pub use super::clone_slot_walk_to_the_slot::{
    hold_clone_slot,
    hold_clone_slot_under_root_control,
    observe_the_clone_slot,
    observe_the_clone_slot_under_root_control,
};
#[cfg(test)]
pub use crate::fs_beneath::identity_of;

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_slot_walk_tests.rs"]
mod tests;
