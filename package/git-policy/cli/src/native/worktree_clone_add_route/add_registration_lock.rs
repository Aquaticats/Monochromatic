//! What:
//!  The lock of the registration the scan has just adopted: take the whole one,
//!  complete the cut-short one in place, write a lost one again and answer the
//!  one line, and refuse the words this add did not write.
//! Why:
//!  The project decided on 2026-11-22 that any wrapped command which holds the
//!  slot's claim and has proved the registration writes a lost lock again and says
//!  so,
//!  and the lock's own state is the classification's answer rather than a second
//!  reading here
//!  (`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`;
//!  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The states, the checks and the words").
//!  `add_registration.rs` adopts through this and would go over the file-size
//!  limit carrying it,
//!  and the lock's words are the classification's own, so a reader meets them
//!  here rather than in the scan's decision.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = settleTheLock(scan, registration); // undefined: the lock was whole
//! ```

use super::add_registration_reads::ScanInput;
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::registration_recipe::complete_a_cut_short_lock;
use crate::worktree_clone_managed_lock::lock_classification::classify_with_a_known_slot_identity;
use crate::worktree_clone_managed_lock::lock_installation::{
    InstallAnswer,
    write_the_lost_lock,
};
use crate::worktree_clone_managed_lock::lock_state::{
    Classification,
    ManagedLockState,
};
use crate::worktree_clone_managed_lock::lock_state_words::the_lines;
use crate::zfs_mounted_dataset::mounted_dataset_instance::LiveMountedDatasets;
use std::path::Path;

/// What:
///  Settle the lock of the registration the scan has just adopted:
///  take the whole one,
///  complete the cut-short one in place,
///  write a lost one again and answer the one line,
///  and refuse words this add did not write.
/// Why:
///  The project decided on 2026-11-22 that any wrapped command which holds the
///  slot's claim and has proved the registration writes a lost lock again and says
///  so,
///  and the lock's own state is the classification's answer rather than a second
///  reading here
///  (`doc/decision/cli-git-zfs-add-time-git-lock-put-back-when-lost.md`;
///  `doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
///  "The states, the checks and the words").
///  A lock the classification calls a person's is never overwritten,
///  and the sentence a person reads is the classification's own.
pub(super) fn settle_the_lock(
    input: &ScanInput<'_>,
    registration: &Path,
) -> Result<Option<String>, String> {
    let mounts: LiveMountedDatasets = LiveMountedDatasets;
    // A record whose add was killed before its own identity write stands at phase `claimed`,
    // and this command made the folder, so the identity the proof compares is its own reading.
    let known: Option<FolderIdentity> = FolderIdentity::of_folder(input.slot_path).ok();
    let classification: Classification = classify_with_a_known_slot_identity(
        input.common_dir,
        input.slot,
        input.slot_path,
        &mounts,
        input.guids,
        known,
    );
    match &classification.state {
        ManagedLockState::Managed {
            ..
        } => return Ok(None),
        ManagedLockState::CutShort {
            ..
        } => {
            complete_a_cut_short_lock(registration, input.nonce).map_err(|error| {
                return format!(
                    "the cut-short lock in {} cannot be completed ({error})",
                    registration.display()
                );
            })?;
            return Ok(None);
        }
        ManagedLockState::NoLock {
            ..
        } => {
            return match write_the_lost_lock(registration, input.slot, input.nonce) {
                // A lock the classification read as absent and this write found standing:
                // the state is named by the classification rather than replaced by a write.
                InstallAnswer::ALockStands => Err(format!(
                    "the lock of `{}` stands and this command cannot attribute it, so nothing was rewritten. Ways forward: run `git worktree unlock {}` if no person holds that lock, then run the add again",
                    registration.display(),
                    input.slot
                )),
                InstallAnswer::Refused {
                    reason,
                } => Err(format!(
                    "the lost managed Git lock of `{}` cannot be written again ({reason}), so nothing beyond the claim was changed. Ways forward: inspect the registration's folder and run the add again",
                    registration.display()
                )),
                InstallAnswer::Wrote {
                    line,
                } => Ok(Some(line.clone())),
            };
        }
        _refusing => {}
    }
    return Err(format!(
        "{} Nothing was removed or rewritten. Ways forward: read the line above, settle the lock it names, then run the add again",
        the_lines(&classification, input.slot).join(" ")
    ));
}
