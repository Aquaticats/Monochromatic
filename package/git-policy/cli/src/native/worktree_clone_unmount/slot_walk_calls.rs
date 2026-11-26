//! What:
//!  The walks and dataset reads the unmount turn needs:
//!  reopening the recorded parent,
//!  opening the final slot with the attach flags or with the crossing the
//!  unmount's hop needs,
//!  a fresh canonical lookup,
//!  and one dataset's live GUID.
//! Why:
//!  The design makes the final slot hop the unmount's own:
//!  the attach's flags refuse the clone's mount with `EXDEV`,
//!  and the unmount must cross exactly that one mount
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "What step 4 begins").
//!  Every hop above the slot keeps the no-crossing walk,
//!  so an account replacing a parent folder is still seen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const slot = walk.openTheSlot(checkout, 'one');
//! const now = walk.lookAgain(checkout, 'one'); // a second whole walk
//! ```

use super::staging_calls::{
    StagingCall,
    StagingCallRefusal,
};
use crate::fs_beneath::{
    handle_directory_at,
    open_handle_at_a_mount,
    open_root,
};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::fs_beneath_walk_chain::open_chain_beneath;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::OsString;
use std::path::Path;

/// What:
///  The two walks the turn makes.
/// Why:
///  A control cannot mount,
///  and it still has to prove that a mismatch refuses before the move,
///  so the walk is a seam exactly as the mount calls are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface SlotWalk { openTheSlot(checkout, slot); openTheSlotUncrossed(checkout, slot); lookAgain(checkout, slot); }
/// ```
pub trait SlotWalk {
    /// The slot's path handle,
    /// crossing the clone's own mount at the final hop.
    fn open_the_slot(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal>;
    /// The slot's path handle,
    /// refusing a mount at the final hop.
    fn open_the_slot_uncrossed(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal>;
    /// The mount a fresh whole walk of the slot reaches.
    fn look_again(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<u64, StagingCallRefusal>;
}

/// What:
///  One dataset's live GUID.
/// Why:
///  The design requires the source's live dataset GUID from the binding,
///  so a dataset replaced under the same name is refused
///  ("Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface DatasetGuids { liveGuid(dataset): bigint }
/// ```
pub trait DatasetGuids {
    /// The dataset's GUID as the pool reports it now.
    fn live_guid(
        &self,
        dataset: &str,
    ) -> Result<DatasetGuid, String>;
}

/// The `.claude/worktrees` names above a slot.
fn worktrees_names() -> Vec<OsString> {
    return vec![OsString::from(".claude"), OsString::from("worktrees")];
}

/// The held `.claude/worktrees` folder of one checkout.
fn parent_of(checkout: &Path) -> Result<std::fs::File, StagingCallRefusal> {
    let root: std::fs::File = open_root(checkout).map_err(|error| {
        return StagingCallRefusal {
            call: StagingCall::OpenSlot,
            errno: error.raw_os_error().unwrap_or(0),
            detail: format!(
                "the checkout {} could not be held: {error}",
                checkout.display()
            ),
        };
    })?;
    let mut chain = open_chain_beneath(&root, worktrees_names().as_slice()).map_err(|refusal| {
        return StagingCallRefusal {
            call: StagingCall::OpenSlot,
            errno: 0,
            detail: format!("the walk to `.claude/worktrees` refused: {refusal:?}"),
        };
    })?;
    return chain.pop().ok_or_else(|| {
        return StagingCallRefusal {
            call: StagingCall::OpenSlot,
            errno: libc::ENOENT,
            detail: String::from("the walk answered no worktrees folder"),
        };
    });
}

/// What:
///  The release walk:
///  the checkout's root,
///  the no-crossing walk to `.claude/worktrees`,
///  and the one final hop.
/// Why:
///  The final hop's flags are the only difference between the attach's walk and
///  the unmount's,
///  so both take the same hops above it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walk = new LiveSlotWalk();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LiveSlotWalk;

impl SlotWalk for LiveSlotWalk {
    fn open_the_slot(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        let parent: std::fs::File = parent_of(checkout)?;
        return open_handle_at_a_mount(&parent, std::ffi::OsStr::new(slot)).map_err(|error| {
            return StagingCallRefusal {
                call: StagingCall::OpenSlot,
                errno: error.raw_os_error().unwrap_or(0),
                detail: format!("the slot `{slot}` could not be opened across its mount: {error}"),
            };
        });
    }

    fn open_the_slot_uncrossed(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<std::fs::File, StagingCallRefusal> {
        let parent: std::fs::File = parent_of(checkout)?;
        return handle_directory_at(&parent, std::ffi::OsStr::new(slot)).map_err(|error| {
            return StagingCallRefusal {
                call: StagingCall::OpenSlot,
                errno: error.raw_os_error().unwrap_or(0),
                detail: format!(
                    "the slot `{slot}` could not be opened without crossing its mount: {error}"
                ),
            };
        });
    }

    fn look_again(
        &self,
        checkout: &Path,
        slot: &str,
    ) -> Result<u64, StagingCallRefusal> {
        let fresh: std::fs::File = self.open_the_slot(checkout, slot)?;
        return mount_id_of(&fresh).map_err(|error| {
            return StagingCallRefusal {
                call: StagingCall::LookAgain,
                errno: error.raw_os_error().unwrap_or(0),
                detail: error.to_string(),
            };
        });
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "slot_walk_calls_tests.rs"]
mod tests;
