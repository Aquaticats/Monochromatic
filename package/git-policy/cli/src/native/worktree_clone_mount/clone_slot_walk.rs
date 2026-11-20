//! What:
//!  The walk from the pinned checkout to one clone slot,
//!  holding a descriptor for the checkout,
//!  the `.claude/worktrees` parent folder and the slot,
//!  and comparing the two folders against the identities the store recorded;
//!  and the second walk that reads what stands at the canonical slot after an attach.
//! Why:
//!  The mount must not resolve a path the account can change, must check the target folder's recorded
//!  identity rather than its kind, and must attach to the opened folder rather than to a path
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount",
//!  applying item 3 of the nested-clone research).
//!  Opening one plain component at a time with `openat2` keeps every step on the object the walk looked at;
//!  the identity compare is what tells the folder the plan expects from a replacement the account made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdCloneSlot({ checkout, slot, store }); // checkout, parent, slot and target handles plus both identities
//! const again = lookAgainAtTheSlot({ checkout, slot });  // what the kernel finds at the canonical slot now
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::clone_slot_walk_refusals::{CloneSlotRefusal, WalkRefusal, WalkRefusalReason, chain_walk_refusal, one_component_refusal};
use super::slot_identity_store::SlotIdentityStore;
use super::slot_identity_store_refusals::StoreRefusal;
use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, handle_directory_at, identity_of, mount_id_of, observed_path, open_chain_beneath, open_directory_at, open_directory_at_a_mount, open_root};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "cloneSlotWalk";

/// The folder below the checkout that holds the wrapper's own records,
/// and the folder below it whose identity the setup run records.
pub const WORKTREES_PARENT_NAMES: [&str; 2] = [".claude", "worktrees"];

/// The names `WORKTREES_PARENT_NAMES` as the walk takes them.
fn worktrees_parent_names() -> Vec<OsString> {
    let mut names: Vec<OsString> = Vec::with_capacity(WORKTREES_PARENT_NAMES.len());
    for name in WORKTREES_PARENT_NAMES {
        names.push(OsString::from(name));
    }
    return names;
}

/// The slot's name below the worktrees folder,
/// as a person reads it in a refusal.
fn slot_component(slot: &str) -> String {
    return format!("{}/{slot}", WORKTREES_PARENT_NAMES.join("/"));
}

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

#[cfg(test)]
thread_local! {
    /// A control may make the slot's name hold another folder after its reading descriptor was opened,
    /// which is where an account renaming the slot ends one walk.
    pub(crate) static AFTER_THE_SLOT_IS_READ: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// The checkout's canonical path,
/// or the refusal of the path given.
fn canonical_checkout(checkout: &Path) -> Result<PathBuf, CloneSlotRefusal> {
    return match std::fs::canonicalize(checkout) {
        Ok(canonical) => Ok(canonical),
        Err(error) => Err(CloneSlotRefusal::Walk(WalkRefusal { component: checkout.display().to_string(), reason: WalkRefusalReason::Checkout, detail: error.to_string() })),
    };
}

/// The refusal of the checkout's own root handle.
fn checkout_root_refusal(canonical: &Path, error: std::io::Error) -> CloneSlotRefusal {
    if error.kind() == std::io::ErrorKind::InvalidInput {
        return CloneSlotRefusal::Walk(WalkRefusal { component: canonical.display().to_string(), reason: WalkRefusalReason::LinkAmongTheAncestors, detail: error.to_string() });
    }
    return CloneSlotRefusal::Walk(WalkRefusal { component: canonical.display().to_string(), reason: WalkRefusalReason::Checkout, detail: error.to_string() });
}

/// The refusal of one folder the walk opens by name.
fn folder_refusal(component: &str, error: std::io::Error) -> CloneSlotRefusal {
    return CloneSlotRefusal::Walk(one_component_refusal(component, &error));
}

/// The checkout's root handle and the chain down to `.claude/worktrees`.
fn walk_to_the_parent(checkout: &Path) -> Result<(std::fs::File, std::fs::File, PathBuf), CloneSlotRefusal> {
    let canonical: PathBuf = canonical_checkout(checkout)?;
    let root: std::fs::File = open_root(canonical.as_path()).map_err(|error| return checkout_root_refusal(canonical.as_path(), error))?;
    let names: Vec<OsString> = worktrees_parent_names();
    let mut chain: Vec<std::fs::File> = open_chain_beneath(&root, names.as_slice()).map_err(|refusal| return CloneSlotRefusal::Walk(chain_walk_refusal(WORKTREES_PARENT_NAMES.as_slice(), &refusal)))?;
    let parent: std::fs::File = match chain.pop() {
        Some(held) => held,
        None => return Err(CloneSlotRefusal::Walk(WalkRefusal { component: String::from("the worktrees folder"), reason: WalkRefusalReason::Missing, detail: String::new() })),
    };
    return Ok((root, parent, canonical));
}

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
pub fn hold_clone_slot(request: &HoldCloneSlotRequest<'_>) -> Result<HeldCloneSlot, CloneSlotRefusal> {
    let (checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) = walk_to_the_parent(request.checkout)?;
    let parent_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&parent).map_err(|error| return folder_refusal(".claude/worktrees", error))?;
    let slot_name: &OsStr = OsStr::new(request.slot);
    let component: String = slot_component(request.slot);
    let reading: std::fs::File = open_directory_at(&parent, slot_name).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let slot_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&reading).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let store: SlotIdentityStore = SlotIdentityStore::read(request.store).map_err(CloneSlotRefusal::Store)?;
    let recorded_parent: &CloneFolderIdentity = match store.parent() {
        Some(recorded) => recorded,
        None => return Err(CloneSlotRefusal::Store(StoreRefusal::NoParentRecord { path: request.store.to_path_buf() })),
    };
    let parent_fields = recorded_parent.differences(&parent_identity);
    if !parent_fields.is_empty() {
        return Err(CloneSlotRefusal::ParentIdentity { recorded: Box::new(recorded_parent.clone()), found: Box::new(parent_identity), fields: parent_fields });
    }
    let recorded_slot: &CloneFolderIdentity = match store.slot(request.slot) {
        Some(recorded) => recorded,
        None => return Err(CloneSlotRefusal::Store(StoreRefusal::NoSlotRecord { path: request.store.to_path_buf(), slot: request.slot.to_string() })),
    };
    let slot_fields = recorded_slot.differences(&slot_identity);
    if !slot_fields.is_empty() {
        return Err(CloneSlotRefusal::SlotIdentity { slot: request.slot.to_string(), recorded: Box::new(recorded_slot.clone()), found: Box::new(slot_identity), fields: slot_fields });
    }
    #[cfg(test)]
    AFTER_THE_SLOT_IS_READ.with(|hook| { if let Some(after) = hook.borrow().as_ref() { after(); } });
    let target: std::fs::File = handle_directory_at(&parent, slot_name).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let first: ObjectId = identity_of(&reading).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let second: ObjectId = identity_of(&target).map_err(|error| return folder_refusal(component.as_str(), error))?;
    if first != second {
        return Err(CloneSlotRefusal::TargetAnotherObject { slot: request.slot.to_string(), first: first.encode(), second: second.encode() });
    }
    let canonical_slot: PathBuf = canonical.join(WORKTREES_PARENT_NAMES[0]).join(WORKTREES_PARENT_NAMES[1]).join(request.slot);
    debug(TAG, "held the checkout, the worktrees folder and one slot by descriptor");
    return Ok(HeldCloneSlot { checkout: checkout_handle, parent, slot: reading, target, parent_identity, slot_identity, canonical_slot });
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
pub fn look_again_at_the_slot(request: &LookAgainAtSlotRequest<'_>) -> Result<FreshSlotLookup, CloneSlotRefusal> {
    let (_checkout, parent, _canonical): (std::fs::File, std::fs::File, PathBuf) = walk_to_the_parent(request.checkout)?;
    let parent_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&parent).map_err(|error| return folder_refusal(".claude/worktrees", error))?;
    let slot_name: &OsStr = OsStr::new(request.slot);
    let component: String = slot_component(request.slot);
    let slot: std::fs::File = open_directory_at_a_mount(&parent, slot_name).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let slot_identity: CloneFolderIdentity = CloneFolderIdentity::capture(&slot).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let mount_id: u64 = mount_id_of(&slot).map_err(|error| return folder_refusal(component.as_str(), error))?;
    let parent_mount_id: u64 = mount_id_of(&parent).map_err(|error| return folder_refusal(".claude/worktrees", error))?;
    let observed: PathBuf = observed_path(&slot).map_err(|error| return folder_refusal(component.as_str(), error))?;
    return Ok(FreshSlotLookup { parent_identity, slot_identity, mount_id, parent_mount_id, observed_path: observed });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_slot_walk_tests.rs"]
mod tests;
