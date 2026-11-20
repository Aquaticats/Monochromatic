//! What:
//!  The plan that takes down a mount which landed beside its slot,
//!  and the report a completed or a refused rollback leaves.
//! Why:
//!  The project decided "Roll back at once (Recommended)":
//!  the exact mount is taken down by the verified descriptor,
//!  and it is retained and reported only when the rollback fails
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Misplaced attach" under "Project decisions").
//!  A report that named only one of the two paths would hide where the clone really hangs,
//!  and a retry against an unverified mount is what the design forbids.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planRollback(misplaced, heldTarget, parentMountId); // heldTarget stays open in the plan
//! const outcome = await unmountThroughDescriptor(plan); // Retained names both paths when it fails
//! ```

use super::attached_mount_identity::MountIdentity;
use super::attach_post_check::MisplacedAttach;
use crate::fs_beneath_mount_facts::observed_path;
use std::path::PathBuf;

/// What:
///  The mount to take down and the two paths a report names.
/// Why:
///   The rollback acts on the descriptor the attach used,
///  so the plan carries the mount's number and where the attach landed,
///  never a path to resolve again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Plan = { attachedMountId: bigint; mountedAt: string; canonicalSlot: string; target: FileHandle };
/// ```
///
/// Why the descriptor and not only its path:
///  an unmount that named a path would resolve it again and could take down whatever stands there by then,
///  so the plan carries the very target the attach acted through and the unmount uses that handle.
///  A plan is therefore not copyable:
///  one mount has one holder,
///  and the type says so.
#[derive(Debug)]
pub struct RollbackPlan {
    /// The mount the attach recorded.
    pub attached_mount_id: u64,
    /// The path the held target's descriptor names now,
    /// which is where the mount landed.
    pub mounted_at: PathBuf,
    /// The slot the mount should have reached.
    pub canonical_slot: PathBuf,
    /// The held target the attach acted through,
    /// kept open so the unmount acts on the descriptor rather than on a path.
    pub target: std::fs::File,
}

/// What:
///  Why no rollback is planned.
/// Why:
///   A mount number equal to the folder's own mount is no attach,
///  and a place that is no path cannot be reported to a person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RollbackRefusal = { kind: 'no-mount' | 'no-place'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RollbackRefusal {
    /// The recorded mount is the mount the folders were already on,
    /// so no attach is proved.
    NoVerifiedMount {
        /// The mount the attach recorded.
        attached_mount_id: u64,
        /// The mount the walk's parent folder is on.
        parent_mount_id: u64,
    },
    /// The place the attach landed is no path a report can name.
    NoPlaceRecorded {
        /// The mount the attach recorded.
        attached_mount_id: u64,
    },
}

impl RollbackRefusal {
    /// The refusal in words,
    /// naming the numbers and every way forward.
    pub fn message(&self) -> String {
        return match self {
            RollbackRefusal::NoVerifiedMount { attached_mount_id, parent_mount_id } => format!(
                "no rollback is planned: the recorded mount {attached_mount_id} is the mount {parent_mount_id} the slot's folders were already on, so no attach is proved. Nothing is unmounted on an identity that proves no mount of this command. Ways forward: leave the folders as they are and have an administrator read `{}` for the slot's mount before anything else runs.",
                crate::fs_mount_options::MOUNTINFO_PATH
            ),
            RollbackRefusal::NoPlaceRecorded { attached_mount_id } => format!(
                "no rollback is planned for mount {attached_mount_id}: the held target's descriptor names no path a report can carry, so a report and an administrator would both be sent nowhere. Ways forward: an administrator reads the mount's point from `{}` and takes the clone's mount down through the descriptor route.",
                crate::fs_mount_options::MOUNTINFO_PATH
            ),
        };
    }
}

/// What:
///  What became of the misplaced mount.
/// Why:
///   The retained state is the stranded state the acceptance asks for:
///  the mount stays where it landed,
///  the report says where,
///  and the person learns that nothing was retried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { kind: 'rolled-back'; plan; mount } | { kind: 'retained'; plan; mount; reason };
/// ```
#[derive(Debug)]
pub enum RollbackOutcome {
    /// The mount left the kernel,
    /// acting on the descriptor the attach used.
    RolledBack {
        /// The plan that was carried out.
        plan: RollbackPlan,
        /// The mount that left.
        mount: MountIdentity,
    },
    /// The mount is still attached,
    /// and nothing else was tried.
    Retained {
        /// The plan that failed.
        plan: RollbackPlan,
        /// The mount that is still attached.
        mount: MountIdentity,
        /// What the rollback answered.
        reason: String,
    },
}

impl RollbackOutcome {
    /// The report in words,
    /// naming the mount,
    /// both paths and every way forward.
    pub fn report(&self) -> String {
        return match self {
            RollbackOutcome::RolledBack { plan, mount } => format!(
                "the mount {} that landed at {} instead of the slot {} left the kernel through the descriptor the attach acted on ({}); nothing of the clone is attached and the slot holds no mount.",
                plan.attached_mount_id,
                plan.mounted_at.display(),
                plan.canonical_slot.display(),
                mount.describe(),
            ),
            RollbackOutcome::Retained { plan, mount, reason } => format!(
                "the mount {} is still attached at {} and not at the slot {}; the rollback failed: {reason}. Nothing else was tried, because a second move against an unverified mount is not issued. It stays exactly as the attach left it: {}. Ways forward: an administrator takes this exact mount down through the descriptor unmount route (a fresh process reopens the recorded parent, verifies mount {} and the clone's source, and unmounts without force or lazy detach), then checks the slot with the design's mount facts and runs the command again.",
                plan.attached_mount_id,
                plan.mounted_at.display(),
                plan.canonical_slot.display(),
                mount.describe(),
                plan.attached_mount_id,
            ),
        };
    }
}

/// What:
///  Plan the rollback of a misplaced attach.
/// Why:
///   The rollback acts on the mount the attach recorded and on the descriptor it acted through,
///  so the plan is made from those facts and refuses an identity that proves no mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = planRollback(misplaced, heldTarget, parentMountId);
/// ```
///
/// # Errors
/// The recorded mount is the parent folder's own mount,
/// or the held target's descriptor names no absolute path a report can carry.
pub fn plan_rollback(misplaced: &MisplacedAttach, target: std::fs::File, parent_mount_id: u64) -> Result<RollbackPlan, RollbackRefusal> {
    if misplaced.attached_mount_id == parent_mount_id {
        return Err(RollbackRefusal::NoVerifiedMount { attached_mount_id: misplaced.attached_mount_id, parent_mount_id });
    }
    let mounted_at: PathBuf = observed_path(&target).map_err(|_error| return RollbackRefusal::NoPlaceRecorded { attached_mount_id: misplaced.attached_mount_id })?;
    if mounted_at.as_os_str().is_empty() || !mounted_at.is_absolute() {
        return Err(RollbackRefusal::NoPlaceRecorded { attached_mount_id: misplaced.attached_mount_id });
    }
    return Ok(RollbackPlan { attached_mount_id: misplaced.attached_mount_id, mounted_at, canonical_slot: misplaced.canonical_slot.clone(), target });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "misplaced_attach_rollback_tests.rs"]
mod tests;
