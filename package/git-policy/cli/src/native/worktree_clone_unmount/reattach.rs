//! What:
//!  The checked move of a staged clone back to its slot:
//!  the slot opened without crossing anything,
//!  its recorded identity compared,
//!  one `move_mount`,
//!  and both halves of the accepted proof.
//! Why:
//!  The project answered "Move it back at once (Recommended)" for a clone that
//!  landed on a mount the account stacked on the staging folder,
//!  and the accepted "Misplaced attach" rule is applied to the move back
//!  (`doc/planning/cli-git-zfs-staging-under-cache-design.md`,
//!  "The private bind";
//!  `doc/decision/cli-git-zfs-staging-under-cache.md`,
//!  "What the handler does when the move lands on a stacked mount").
//!  The destination open refuses a mount at the slot with `EXDEV`,
//!  the move runs once,
//!  and the proof is both that the held identity received the exact mount and
//!  that a fresh walk finds it at the canonical slot.
//!  A proof that cannot pass stops the move rather than trying again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = rollBackToSlot(request); // { kind: 'reattached' } | { kind: 'refused' }
//! ```

use super::staged_folder_identity::{StagedFolderField, StagedFolderIdentity};
use super::slot_walk_calls::SlotWalk;
use super::staging_calls::StagingCalls;
use super::unmount_checks::ZFS_FILE_SYSTEM_TYPE;
use crate::fs_mount_options::MountFacts;
use std::path::{Path, PathBuf};

/// What:
///  The slot a clone is moved back to,
///  and every fact the move checks before and after.
/// Why:
///  The destination is the clone's recorded slot,
///  so its identity and its parent mount are the two facts a mount stacked
///  there would change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReattachRequest = { checkout; slot; cloneSource; cloneMountId; slotIdentity; slotParentMountId; cloneAtStaging; calls; walk };
/// ```
pub struct ReattachRequest<'a> {
    /// The verified checkout the slot hangs below.
    pub checkout: &'a Path,
    /// The slot's name.
    pub slot: &'a str,
    /// The clone's dataset source.
    pub clone_source: &'a str,
    /// The clone's mount number.
    pub clone_mount_id: u64,
    /// The slot folder's recorded identity.
    pub slot_identity: &'a StagedFolderIdentity,
    /// The mount the slot's parent folder is on.
    pub slot_parent_mount_id: u64,
    /// A descriptor on the clone's mount at the staging folder.
    pub clone_at_staging: &'a std::fs::File,
    /// The mount calls.
    pub calls: &'a dyn StagingCalls,
    /// The walks.
    pub walk: &'a dyn SlotWalk,
}

/// What:
///  What became of the clone.
/// Why:
///  A successful move leaves the clone at its slot and the bind empty;
///  a refused one leaves it staged and names the step that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReattachResult = { kind: 'reattached'; mountId; slot } | { kind: 'refused'; step; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReattachResult {
    /// The clone stands at its slot,
    /// proved.
    Reattached {
        /// The clone's mount number,
        /// unchanged by the move.
        mount_id: u64,
        /// The slot's canonical path.
        slot: PathBuf,
    },
    /// The clone stays staged,
    /// and the refusal names the step.
    Refused {
        /// The step that refused.
        step: String,
        /// What it answered.
        said: String,
    },
}

impl ReattachResult {
    /// The word a receipt carries.
    pub fn word(&self) -> &'static str {
        return match self {
            ReattachResult::Reattached { .. } => "reattached",
            ReattachResult::Refused { .. } => "refused",
        };
    }
}

/// One field's mismatch.
fn field(fields: &[StagedFolderField], recorded: &StagedFolderIdentity, found: &StagedFolderIdentity) -> String {
    return format!(
        "the slot's folder differs from the recorded identity in the {}; the record says {} and the folder reads {}",
        StagedFolderIdentity::field_names(fields),
        recorded.describe(),
        found.describe()
    );
}

/// What:
///  Move one staged clone back to its recorded slot and prove it there.
/// Why:
///  The move is issued once,
///  after a destination open that refuses a covered slot,
///  and the proof checks the row's parent as well as the fresh walk,
///  because the fresh walk alone lands on whatever is topmost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const result = rollBackToSlot(request);
/// ```
pub fn roll_back_to_slot(request: &ReattachRequest<'_>) -> ReattachResult {
    let slot: std::fs::File = match request.walk.open_the_slot_uncrossed(request.checkout, request.slot) {
        Ok(slot) => slot,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's destination could not be opened without crossing a mount") },
    };
    let found: StagedFolderIdentity = match StagedFolderIdentity::of(&slot) {
        Ok(found) => found,
        Err(refusal) => return ReattachResult::Refused { step: String::from("slot-identity"), said: refusal.message() },
    };
    let moved: Vec<StagedFolderField> = found.differences(request.slot_identity);
    if !moved.is_empty() {
        return ReattachResult::Refused { step: String::from("slot-identity"), said: field(moved.as_slice(), request.slot_identity, &found) };
    }
    let parent_mount: u64 = match request.calls.mount_id(&slot) {
        Ok(read) => read,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's destination mount could not be read") },
    };
    if parent_mount != request.slot_parent_mount_id {
        return ReattachResult::Refused { step: String::from("slot-parent"), said: format!("the rollback's destination is on mount {parent_mount}, and the recorded slot's parent is mount {}", request.slot_parent_mount_id) };
    }
    if let Err(refusal) = request.calls.move_clone(request.clone_at_staging, &slot) {
        return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's move refused") };
    }
    return prove_the_slot(request);
}

/// The proof after one move back.
fn prove_the_slot(request: &ReattachRequest<'_>) -> ReattachResult {
    let slot: std::fs::File = match request.walk.open_the_slot(request.checkout, request.slot) {
        Ok(slot) => slot,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's proof could not walk the canonical slot") },
    };
    let mount_id: u64 = match request.calls.mount_id(&slot) {
        Ok(mount_id) => mount_id,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's proof could not read the slot's mount") },
    };
    if mount_id != request.clone_mount_id {
        return ReattachResult::Refused { step: String::from("post-move-check"), said: format!("the slot's topmost mount is {mount_id}, and the clone's mount is {}", request.clone_mount_id) };
    }
    let row: MountFacts = match request.calls.mount_facts(mount_id) {
        Ok(row) => row,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's proof could not read the slot's row") },
    };
    if row.source != request.clone_source || row.filesystem_type != ZFS_FILE_SYSTEM_TYPE || row.root != Path::new("/") {
        return ReattachResult::Refused {
            step: String::from("post-move-check"),
            said: format!("the slot's topmost row is source `{}`, type `{}` and root {}, and the clone is `{}` of type `{ZFS_FILE_SYSTEM_TYPE}` at root `/`", row.source, row.filesystem_type, row.root.display(), request.clone_source),
        };
    }
    if row.parent_id != request.slot_parent_mount_id {
        return ReattachResult::Refused { step: String::from("post-move-check"), said: format!("the slot's topmost row hangs on mount {}, and the slot's parent is mount {}", row.parent_id, request.slot_parent_mount_id) };
    }
    let again: u64 = match request.walk.look_again(request.checkout, request.slot) {
        Ok(again) => again,
        Err(refusal) => return ReattachResult::Refused { step: String::from(refusal.call.word()), said: refusal.message("the rollback's fresh walk refused") },
    };
    if again != request.clone_mount_id {
        return ReattachResult::Refused { step: String::from("post-move-check"), said: format!("a fresh walk of the canonical slot reaches mount {again}, and the moved clone is mount {}", request.clone_mount_id) };
    }
    return ReattachResult::Reattached { mount_id, slot: row.mount_point };
}

/// The step a move back failure names.
pub const REATTACH_STEP: &str = "reattach";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "reattach_tests.rs"]
mod tests;
