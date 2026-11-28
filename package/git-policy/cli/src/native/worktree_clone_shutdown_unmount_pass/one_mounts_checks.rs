//! What:
//!  The comparisons one shutdown pass makes before it unmounts anything:
//!  the folder the slot hangs in,
//!  the descriptor the walk opened across the slot's mount,
//!  the mount number a fresh walk reaches,
//!  the device and inode of the folder the slot shows,
//!  the path the kernel reports for the descriptor,
//!  and the mount table's row for that mount number.
//! Why:
//!  Unmounting is a kernel call on whatever stands at a path when the call
//!  runs,
//!  and every one of these is a way that mount can be another mount than the
//!  leak of the container this pass read a moment earlier:
//!  a folder the account replaced,
//!  a slot the route did not record,
//!  a mount that moved,
//!  a row replaced under the same number,
//!  and a bind of a folder inside the clone rather than the clone
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount":
//!  "Compare mount ID from `fdinfo`,
//! device and inode from `fstat`,
//! and the expected slot,
//! parent mount ID,
//! ZFS root and source in the handler's `mountinfo`").
//!  Each comparison refuses rather than unmounting what it found,
//!  so a mount that changed between two reads is a line saying so,
//!  never a call.
//!  The pass knows no repository and so no binding record:
//!  what it has is the route's own record of the `.claude/worktrees` folder,
//!  the kernel's table and the walk the removal already makes,
//!  and that is what it compares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const checked = verifyTheMount(candidate, checkout, recordedParent, lookups, walk, calls);
//! checked; // { target, mountId } or thrown refusal
//! ```

use super::shutdown_unmount_sources::SlotLookups;
use super::unmountable_mounts::CandidateMount;
use crate::fs_mount_options::MountFacts;
use crate::worktree_clone_mount::FreshSlotLookup;
use crate::worktree_clone_mount::clone_folder_identity::CloneFolderIdentity;
use crate::worktree_clone_unmount::slot_walk_calls::SlotWalk;
use crate::worktree_clone_unmount::staging_calls::StagingCalls;
use crate::worktree_clone_unmount::unmount_checks::ZFS_FILE_SYSTEM_TYPE;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
/// The descriptor the walk opened across the slot's mount and the mount number
///  every comparison agreed on.
/// Why:
///  The unmount is named by the descriptor rather than by a path,
///  so the mount that goes is the one the comparisons proved and not whatever
///  a path resolves to when the call runs
/// (`crate::worktree_clone_unmount::staging_calls::StagingCalls::unmount_top`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckedMount = { target: File; mountId: number };
/// ```
#[derive(Debug)]
pub(crate) struct CheckedMount {
    /// The slot's descriptor,
    /// which is on the clone's own mount.
    pub(crate) target: std::fs::File,
    /// The mount's kernel number.
    pub(crate) mount_id: u64,
}

/// What:
///  Open the slot across its mount and compare every field the pass expects.
/// Why:
///  A refusal here makes no kernel call,
///  so nothing is unmounted on a mount this pass cannot name
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit":
///  "Unmount the verified managed mounts only").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const checked = verifyTheMount(candidate, checkout, recordedParent, lookups, walk, calls); // throws a refusal
/// ```
///
/// # Errors
/// The slot's parent is not the folder the route recorded,
/// a walk or a read refused,
/// the descriptor is on another mount than the fresh walk reached,
/// or the row is not the clone's own mount at its slot.
pub(crate) fn verify_the_mount(
    candidate: &CandidateMount,
    checkout: &Path,
    recorded_parent: &CloneFolderIdentity,
    lookups: &dyn SlotLookups,
    walk: &dyn SlotWalk,
    calls: &dyn StagingCalls,
) -> Result<CheckedMount, String> {
    let fresh: FreshSlotLookup = lookups.fresh_lookup(checkout, candidate.slot.as_str())?;
    let moved: Vec<&'static str> = fresh
        .parent_identity
        .differences(recorded_parent)
        .iter()
        .map(|field| return field.name())
        .collect();
    if !moved.is_empty() {
        return Err(format!(
            "the folder {} hangs in is not the `.claude/worktrees` folder this route recorded: {} differ(s) ({}, and the record says {}), so a mount below it is no slot this boot verified. Ways forward: run the administrator's `adopt-worktree-parent` for the folder that stands there, and read `journalctl -u reconcile-git-worktree-clone-mounts.service` for the pass that recorded it",
            candidate.slot_path.display(),
            moved.join(", "),
            fresh.parent_identity.describe(),
            recorded_parent.describe()
        ));
    }
    let target: std::fs::File = walk
        .open_the_slot(checkout, candidate.slot.as_str())
        .map_err(|refusal| {
            return refusal.message("the slot could not be opened across its mount");
        })?;
    let on_the_descriptor: u64 = calls
        .mount_id(&target)
        .map_err(|refusal| return refusal.message("the slot's descriptor could not be read"))?;
    if on_the_descriptor != fresh.mount_id {
        return Err(format!(
            "the fresh walk of slot `{}` reached mount {}, and the descriptor opened across it is on mount {on_the_descriptor}, so the mount moved while this pass read it. Nothing was unmounted",
            candidate.slot, fresh.mount_id
        ));
    }
    let facts: std::fs::Metadata = target.metadata().map_err(|error| {
        return format!(
            "the slot `{}` could not be examined ({error}), so nothing was unmounted there",
            candidate.slot
        );
    })?;
    if facts.dev() != fresh.slot_identity.device || facts.ino() != fresh.slot_identity.inode {
        return Err(format!(
            "the slot `{}` reports device {} inode {} and the fresh walk reports device {} inode {}, so the folder was replaced while this pass read it. Nothing was unmounted there",
            candidate.slot,
            facts.dev(),
            facts.ino(),
            fresh.slot_identity.device,
            fresh.slot_identity.inode
        ));
    }
    let observed: std::path::PathBuf = calls
        .observed_path(&target)
        .map_err(|refusal| return refusal.message("the slot's observed path could not be read"))?;
    if observed != candidate.slot_path {
        return Err(format!(
            "the slot's descriptor shows {} and the slot this pass reads is {}, so the walk reached another place. Nothing was unmounted there",
            observed.display(),
            candidate.slot_path.display()
        ));
    }
    let again: u64 = walk
        .look_again(checkout, candidate.slot.as_str())
        .map_err(|refusal| return refusal.message("the slot could not be walked again"))?;
    if again != fresh.mount_id {
        return Err(format!(
            "a second walk of slot `{}` reaches mount {again} and the first reached {}, so the mount moved while this pass read it. Nothing was unmounted there",
            candidate.slot, fresh.mount_id
        ));
    }
    let row: MountFacts = calls
        .mount_facts(fresh.mount_id)
        .map_err(|refusal| return refusal.message("the mount's own row could not be read"))?;
    let refusal: Option<String> = compare_the_row(&row, candidate, fresh.parent_mount_id);
    if let Some(said) = refusal {
        return Err(said);
    }
    return Ok(CheckedMount {
        target,
        mount_id: fresh.mount_id,
    });
}

/// What:
///  The row's source,
///  filesystem type,
///  root,
///  mount point and parent,
///  each compared with what the pass expects.
/// Why:
///  A row from another dataset,
///  a filesystem that is no ZFS mount,
///  a bind of a folder inside the clone and a mount attached beside the slot
///  are each a different thing standing where a slot should be,
///  and each sends a person somewhere else
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Descriptor mount and unmount").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// compareTheRow(row, candidate, parentMountId); // a sentence, or null when every field agrees
/// ```
fn compare_the_row(
    row: &MountFacts,
    candidate: &CandidateMount,
    parent_mount_id: u64,
) -> Option<String> {
    if row.source != candidate.clone {
        return Some(format!(
            "the mount at {} has source `{}` and this pass reads `{}`, so it is another dataset's mount and nothing was unmounted there",
            candidate.slot_path.display(),
            row.source,
            candidate.clone
        ));
    }
    if row.filesystem_type != ZFS_FILE_SYSTEM_TYPE {
        return Some(format!(
            "the mount at {} is a `{}` mount and a clone's own mount is a `{ZFS_FILE_SYSTEM_TYPE}` mount, so nothing was unmounted there",
            candidate.slot_path.display(),
            row.filesystem_type
        ));
    }
    if row.root != Path::new("/") {
        return Some(format!(
            "the mount at {} shows root {} of `{}` and a clone's own mount shows `/`, so this is a bind of a folder inside the clone rather than the clone, and nothing was unmounted there",
            candidate.slot_path.display(),
            row.root.display(),
            candidate.clone
        ));
    }
    if row.mount_point != candidate.slot_path {
        return Some(format!(
            "mount {} stands at {} and this pass reads the slot {}, so the mount moved and nothing was unmounted there",
            row.mount_id,
            row.mount_point.display(),
            candidate.slot_path.display()
        ));
    }
    if row.parent_id != parent_mount_id {
        return Some(format!(
            "the mount at {} hangs from mount {} and the slot's folder is on mount {parent_mount_id}, so a mount stands between the slot and its folder and nothing was unmounted there",
            candidate.slot_path.display(),
            row.parent_id
        ));
    }
    return None;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "one_mounts_checks_tests.rs"]
mod tests;
