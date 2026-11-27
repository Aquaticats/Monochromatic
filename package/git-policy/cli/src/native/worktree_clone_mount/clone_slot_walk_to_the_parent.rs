//! What:
//!  The walk from the pinned checkout down to its `.claude/worktrees` folder,
//!  holding a descriptor for the checkout and for that folder through the
//!  no-crossing chain,
//!  and the administrator's own walk of that folder,
//!  which reads its identity and compares nothing.
//! Why:
//!  Opening one plain component at a time keeps every step on the object the
//!  walk looked at,
//!  so the folder a later call acts on is the folder this walk proved,
//!  and the adoption's walk must read rather than compare,
//!  because a walk that compared the parent against the store would refuse the
//!  very replacement it is asked to judge
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount",
//!  "The administrator's adoption").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { root, parent, canonical } = walkToTheParent(checkout);
//! const adopted = holdTheWorktreesParent(checkout);   // the identity the walk read
//! ```

use super::clone_folder_identity::CloneFolderIdentity;
use super::clone_slot_walk::HeldWorktreesParent;
use super::clone_slot_walk_names::{
    WORKTREES_PARENT_NAMES,
    canonical_checkout,
    canonical_or_given,
    checkout_root_refusal,
    worktrees_parent_names,
};
use super::clone_slot_walk_refusals::{
    CloneSlotRefusal,
    WalkRefusal,
    WalkRefusalReason,
    chain_walk_refusal,
};
use crate::diagnostic_log::debug;
use crate::fs_beneath::open_root;
use crate::fs_beneath_walk_chain::open_chain_beneath;
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// Module tag for diagnostics.
const TAG: &str = "cloneSlotWalk";

/// The checkout's root handle and the chain down to `.claude/worktrees`.
pub(super) fn walk_to_the_parent(
    checkout: &Path
) -> Result<(std::fs::File, std::fs::File, PathBuf), CloneSlotRefusal> {
    let canonical: PathBuf = canonical_checkout(checkout)?;
    let root: std::fs::File = open_root(canonical.as_path())
        .map_err(|error| return checkout_root_refusal(canonical.as_path(), error))?;
    let names: Vec<OsString> = worktrees_parent_names();
    let mut chain: Vec<std::fs::File> =
        open_chain_beneath(&root, names.as_slice()).map_err(|refusal| {
            return CloneSlotRefusal::Walk(chain_walk_refusal(WORKTREES_PARENT_NAMES, &refusal));
        })?;
    let parent: std::fs::File = match chain.pop() {
        Some(held) => held,
        None => {
            return Err(CloneSlotRefusal::Walk(WalkRefusal {
                component: String::from("the worktrees folder"),
                reason: WalkRefusalReason::Missing,
                detail: String::new(),
            }));
        }
    };
    return Ok((root, parent, canonical));
}

/// What:
///  The `.claude/worktrees` folder,
///  held by descriptor through the no-crossing walk,
///  with the identity the walk read for it and its canonical path.
/// Why:
///   The administrator's adoption walks the parent folder and compares it with the recorded identity
///  and with a fresh lookup of the same folder,
///  and it does so while that folder is the one that changed:
///  a walk that compared the parent against the store would refuse the very replacement it is asked to judge,
///  so this walk reads the identity and compares nothing
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount",
///  "The administrator's adoption").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = holdTheWorktreesParent(checkout); // the held folder and the identity the walk read
/// ```
///
/// # Errors
/// The walk's refusal at the checkout or at any canonical component.
pub fn hold_the_worktrees_parent(checkout: &Path) -> Result<HeldWorktreesParent, WalkRefusal> {
    let (_checkout_handle, parent, canonical): (std::fs::File, std::fs::File, PathBuf) =
        walk_to_the_parent(checkout).map_err(|refusal| {
            return match refusal {
                CloneSlotRefusal::Walk(walk) => walk,
                other => WalkRefusal {
                    component: canonical_or_given(checkout),
                    reason: WalkRefusalReason::Checkout,
                    detail: other.message(),
                },
            };
        })?;
    let identity: CloneFolderIdentity = CloneFolderIdentity::capture(&parent).map_err(|error| {
        return WalkRefusal {
            component: WORKTREES_PARENT_NAMES.join("/"),
            reason: WalkRefusalReason::UnreadableFields,
            detail: error.to_string(),
        };
    })?;
    let canonical_folder: PathBuf = canonical
        .join(WORKTREES_PARENT_NAMES[0])
        .join(WORKTREES_PARENT_NAMES[1]);
    debug(
        TAG,
        "held the worktrees folder by descriptor and read its identity",
    );
    return Ok(HeldWorktreesParent {
        folder: parent,
        identity,
        canonical: canonical_folder,
    });
}
