//! What:
//!  The check that runs after an attach:
//!  whether the mount the attach made is the one the canonical slot now shows,
//!  and what moved when it is not.
//! Why:
//!  The design requires both proofs:
//!  that the held folder received the expected mount,
//!  and that a fresh walk from the checkout finds that mount at the canonical slot
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount").
//!  The measured trials moved a held descriptor with a renamed `.claude` and the attach landed there,
//!  and only the fresh walk detected it;
//!  that is a refusal,
//!  not a worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = postAttachCheck({ heldParent, heldParentMountId: 40, heldTargetMountId: 41, attachedMountId: 41, canonicalSlot }, lookAgainAtSlot(...));
//! if (outcome.kind === 'misplaced') rollback(outcome);
//! ```

use super::attached_mount_identity::{MountIdentity, MountIdentityRefusal, mount_facts_of};
use super::clone_folder_identity::{CloneFolderField, CloneFolderIdentity, field_names};
use super::clone_slot_walk::FreshSlotLookup;
use super::clone_slot_walk_refusals::CloneSlotRefusal;
use crate::diagnostic_log::debug;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "attachPostCheck";

/// What:
///  What the attach knows about the folders it acted on:
///  the parent folder's identity and mount,
///  the mount the held target is on after the move,
///  the mount the attach made,
///  and the slot's canonical path.
/// Why:
///   Those five facts decide whether the fresh walk found the attach,
///  so they travel together from the walk to the check.
///  The held target's own mount is the design's first proof taken literally:
///  the folder the attach acted through must be the folder that received the mount,
///  whatever a later lookup of the slot finds.
///  It is read at the target's name beneath the held parent,
///  because the held target's descriptor keeps naming the mount it was opened on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Check = { heldParentIdentity; heldParentMountId; heldTargetMountId; attachedMountId; canonicalSlot };
/// ```
#[derive(Clone, Debug)]
pub struct AttachPostCheck<'a> {
    /// The identity of the `.claude/worktrees` folder the walk held.
    pub held_parent_identity: &'a CloneFolderIdentity,
    /// The mount the walk's parent folder is on.
    pub held_parent_mount_id: u64,
    /// The mount the held target is on,
    /// read at the target's name beneath the held parent after `move_mount` returned
    /// (`mount_id_at_the_held_target`).
    /// The held target's own descriptor is not the reader:
    /// a descriptor keeps naming the mount it was opened on,
    /// so it answers the parent's mount after a real attach,
    /// and the guest measured exactly that.
    pub held_target_mount_id: u64,
    /// The mount ID the attach recorded for the mount it made,
    /// read from the descriptor `fsmount` returned.
    pub attached_mount_id: u64,
    /// The slot's canonical path below the checkout.
    pub canonical_slot: &'a Path,
}

/// What:
///  Why a mount is not at the slot the attach holds.
/// Why:
///   Each cause sends a person somewhere else:
///  a parent folder replaced,
///  a mount attached beside the slot,
///  a slot on another mount,
///  or a folder renamed after the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MisplacedReason = { kind: 'walk' } | { kind: 'parent'; fields } | { kind: 'held-target'; held; attached } | { kind: 'mount'; fresh; attached } | { kind: 'parent-mount'; fresh; held } | { kind: 'path'; observed };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MisplacedReason {
    /// The fresh walk stopped where the slot should be.
    WalkRefused {
        /// What the walk refused.
        refusal: Box<CloneSlotRefusal>,
    },
    /// The folder above the canonical slot is not the one the walk held.
    ParentChanged {
        /// The fields in which the folders differ.
        fields: Vec<CloneFolderField>,
    },
    /// The held target is not on the mount the attach made.
    HeldTargetElsewhere {
        /// The mount resolved at the held target's name beneath the held parent.
        held: u64,
        /// The mount the attach recorded.
        attached: u64,
    },
    /// The mount at the canonical slot is not the mount the attach made.
    MountElsewhere {
        /// The mount the fresh lookup found.
        fresh: u64,
        /// The mount the attach recorded.
        attached: u64,
    },
    /// The slot's mount does not hang on the mount the walk's parent is on.
    OtherParentMount {
        /// The fresh lookup's parent mount.
        fresh: u64,
        /// The mount the walk's parent folder is on.
        held: u64,
    },
    /// The slot's descriptor names another path than the canonical slot.
    ObservedPathMoved {
        /// The path the descriptor names.
        observed: PathBuf,
        /// The canonical slot.
        canonical: PathBuf,
    },
}

impl MisplacedReason {
    /// The reason in words,
    /// naming the field or the two numbers that differ.
    pub fn message(&self) -> String {
        return match self {
            MisplacedReason::WalkRefused { refusal } => format!("the canonical slot could not be walked after the attach: {}", refusal.message()),
            MisplacedReason::ParentChanged { fields } => format!(
                "the folder above the canonical slot is not the one the attach held: the {} differs. The mount was attached through the held descriptor, so the canonical slot does not show it.",
                field_names(fields)
            ),
            MisplacedReason::HeldTargetElsewhere { held, attached } => format!(
                "the held target's name beneath the held parent resolves to mount {held}, not mount {attached} that the attach made, so the folder the walk judged did not receive this mount."
            ),
            MisplacedReason::MountElsewhere { fresh, attached } => format!("the mount at the canonical slot is mount {fresh}, not mount {attached} that the attach made."),
            MisplacedReason::OtherParentMount { fresh, held } => format!("the slot's mount hangs on mount {fresh}, and the walk's parent folder is on mount {held}, so this is not the mount this walk made."),
            MisplacedReason::ObservedPathMoved { observed, canonical } => format!("the slot's descriptor names {} now, not the canonical slot {}.", observed.display(), canonical.display()),
        };
    }
}

/// What:
///  A mount that is not where the attach intended it:
///  why,
///  what the fresh walk found,
///  and the canonical slot it should have reached.
/// Why:
///   The rollback needs the mount's identity and both paths,
///  and the report it leaves must name what was found
///  when the walk found nothing at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Misplaced = { reason; attachedMountId; canonicalSlot; fresh? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MisplacedAttach {
    /// What moved.
    pub reason: MisplacedReason,
    /// The mount the attach recorded.
    pub attached_mount_id: u64,
    /// The canonical slot the mount should have reached.
    pub canonical_slot: PathBuf,
    /// What the fresh walk found,
    /// or nothing when it stopped.
    pub fresh: Option<FreshSlotLookup>,
}

/// What:
///  What the post-attach check concluded.
/// Why:
///   Only one conclusion lets the worktree be used;
///  the other is the rollback's input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'attached'; mount; parentIdentity; observedPath } | { kind: 'misplaced'; misplaced };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachOutcome {
    /// The mount the attach made is the mount the canonical slot shows.
    Attached {
        /// The mount,
        /// as the kernel lists it.
        mount: MountIdentity,
        /// The parent folder's identity the fresh walk read.
        parent_identity: CloneFolderIdentity,
        /// The canonical slot the mount was expected at.
        canonical_slot: PathBuf,
        /// The path the slot's descriptor named,
        /// which equals the canonical slot when the check passes.
        observed_path: PathBuf,
    },
    /// The mount is somewhere else,
    /// and is rolled back.
    Misplaced(MisplacedAttach),
}

/// What:
///  Compare the mount the attach made with what a fresh walk of the canonical slot finds.
/// Why:
///   Every arm is a refusal rather than a worktree,
/// and each names the field,
/// the mount or the path that differs,
///  so the report of a misplaced attach says what happened.
///  The held target's mount is compared first because it is the one fact a stopped walk or another
///  mount at the slot cannot explain away:
///  the folder the attach acted through is the folder that had to receive the mount,
///  and its number is read at that folder's name beneath the held parent rather than from the held descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = postAttachCheck(check, lookAgainAtTheSlot({ checkout, slot })); // throws when the mount table cannot be read
/// ```
///
/// # Errors
/// The mount table could not be read after the walk matched,
/// so the mount cannot be recorded and the attach is rolled back.
pub fn post_attach_check(check: &AttachPostCheck<'_>, fresh: Result<FreshSlotLookup, CloneSlotRefusal>) -> Result<AttachOutcome, MountIdentityRefusal> {
    let misplaced = |reason: MisplacedReason, found: Option<FreshSlotLookup>, attached: u64| return AttachOutcome::Misplaced(MisplacedAttach { reason, attached_mount_id: attached, canonical_slot: check.canonical_slot.to_path_buf(), fresh: found });
    if check.held_target_mount_id != check.attached_mount_id {
        return Ok(misplaced(MisplacedReason::HeldTargetElsewhere { held: check.held_target_mount_id, attached: check.attached_mount_id }, None, check.attached_mount_id));
    }
    let found: FreshSlotLookup = match fresh {
        Ok(lookup) => lookup,
        Err(refusal) => return Ok(misplaced(MisplacedReason::WalkRefused { refusal: Box::new(refusal) }, None, check.attached_mount_id)),
    };
    let fields: Vec<CloneFolderField> = check.held_parent_identity.differences(&found.parent_identity);
    if !fields.is_empty() {
        return Ok(misplaced(MisplacedReason::ParentChanged { fields }, Some(found), check.attached_mount_id));
    }
    if found.mount_id != check.attached_mount_id {
        return Ok(misplaced(MisplacedReason::MountElsewhere { fresh: found.mount_id, attached: check.attached_mount_id }, Some(found), check.attached_mount_id));
    }
    if found.parent_mount_id != check.held_parent_mount_id {
        return Ok(misplaced(MisplacedReason::OtherParentMount { fresh: found.parent_mount_id, held: check.held_parent_mount_id }, Some(found), check.attached_mount_id));
    }
    if found.observed_path != check.canonical_slot {
        let reason: MisplacedReason = MisplacedReason::ObservedPathMoved { observed: found.observed_path.clone(), canonical: check.canonical_slot.to_path_buf() };
        return Ok(misplaced(reason, Some(found), check.attached_mount_id));
    }
    let facts = mount_facts_of(found.mount_id)?;
    let mount: MountIdentity = MountIdentity {
        mount_id: found.mount_id,
        parent_mount_id: found.parent_mount_id,
        mount_point: facts.mount_point,
        source: facts.source,
        file_system_type: facts.filesystem_type,
        device: found.slot_identity.device,
        inode: found.slot_identity.inode,
        observed_path: found.observed_path.clone(),
    };
    debug(TAG, "the mount at the canonical slot is the one the attach made");
    return Ok(AttachOutcome::Attached { mount, parent_identity: found.parent_identity, canonical_slot: check.canonical_slot.to_path_buf(), observed_path: found.observed_path });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attach_post_check_tests.rs"]
mod tests;
