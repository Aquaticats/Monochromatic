//! What:
//!  The attach step one clone goes through:
//!  the walk from the checkout to the slot,
//!  the mount the walk held,
//!  the fresh canonical check after it,
//!  and the rollback of a mount that landed beside the slot.
//! Why:
//!  Step 4's module holds the walk and the checks and step 8 is the first step
//!  that attaches a clone,
//!  so the order the design states lives here:
//!  walk,
//!  attach,
//!  both post-attach proofs,
//!  and the immediate verified rollback
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  the project answered "Roll back at once (Recommended)" under "Misplaced
//!  attach").
//!  A step whose answer is a mount the post-check refused is never reported as
//!  an attach:
//!  a mount that cannot be recorded is taken down,
//!  and one that cannot be taken down is reported with both paths and nothing
//!  else tried.
//!  The trait is what lets a control drive the whole sweep against fixture
//!  folders,
//!  because the real mount calls need the throwaway guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = attachStep.attach({ checkout, slot, clone, store }); // attached | rolled-back | retained | refused
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::worktree_clone_mount::{
    AttachOutcome, AttachPostCheck, HeldCloneSlot, HoldCloneSlotRequest, LookAgainAtSlotRequest, MountIdentity, RollbackPlan, attach_the_clone, hold_clone_slot_under_root_control, look_again_at_the_slot,
    mount_id_at_the_held_target, plan_rollback_of, post_attach_check, unmount_through_descriptor,
};
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "sweepAttachStep";

/// What:
///  What one clone's attach step is asked to do.
/// Why:
///  Every field is a name the step must not invent:
///  the checkout it walks,
///  the slot the clone's name gave,
///  the dataset it attaches and the store it compares,
///  so a caller cannot attach one clone at another clone's slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachRequest = { checkout: string; slot: string; clone: string; store: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct AttachRequest<'a> {
    /// The checkout the walk starts at.
    pub checkout: &'a Path,
    /// The slot the clone's name gave.
    pub slot: &'a str,
    /// The clone's full dataset name.
    pub clone: &'a str,
    /// The store of recorded identities.
    pub store: &'a Path,
}

/// What:
///  What became of one attach step.
/// Why:
///  A sweep reports each clone's own outcome separately,
///  and a refusal that carries a mount the rollback could not take down must
///  not look like a refusal that left nothing behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachStepOutcome = { kind: 'attached'; mount } | { kind: 'rolled-back'; said } | { kind: 'retained'; said } | { kind: 'refused'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachStepOutcome {
    /// The mount the attach made is the mount the canonical slot shows.
    Attached {
        /// The mount,
        /// as the kernel lists it.
        mount: MountIdentity,
    },
    /// The attach made a mount that landed elsewhere and the rollback took it
    /// down.
    RolledBack {
        /// The rollback's own report.
        said: String,
    },
    /// The attach made a mount that landed elsewhere and the rollback could not
    /// take it down.
    Retained {
        /// The rollback's own report,
        /// naming the mount and both paths.
        said: String,
    },
    /// Nothing was attached:
    /// the walk or the mount call refused.
    Refused {
        /// The refusal in words.
        said: String,
    },
}

/// What:
///  One attach step,
///  as the sweep calls it.
/// Why:
///  The real step mounts a real clone,
///  which no control on a person's machine may do,
///  and a control drives the sweep's order against fixture folders with this
///  seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CloneAttachStep { attach(request: AttachRequest): AttachStepOutcome }
/// ```
pub trait CloneAttachStep {
    /// Attach one clone at the slot its name gave.
    fn attach(&self, request: &AttachRequest<'_>) -> AttachStepOutcome;
}

/// What:
///  The release step:
///  the walk,
///  the mount and both post-attach proofs,
///  and the rollback.
/// Why:
///  This is the shape the design states,
///  and it is one implementation so the order cannot differ between the sweep
///  a boot runs and the sweep a ZED event starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = realAttachStep.attach(request);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RealAttachStep;

impl CloneAttachStep for RealAttachStep {
    fn attach(&self, request: &AttachRequest<'_>) -> AttachStepOutcome {
        let walk: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: request.checkout, slot: request.slot, store: request.store };
        let held: HeldCloneSlot = match hold_clone_slot_under_root_control(&walk) {
            Ok(held) => held,
            Err(refusal) => return AttachStepOutcome::Refused { said: refusal.message() },
        };
        let attached_mount_id: u64 = match attach_the_clone(request.clone, &held.target) {
            Ok(attached) => attached.mount_id,
            Err(refusal) => return AttachStepOutcome::Refused { said: refusal.message() },
        };
        return check_place_the_mount(request, &held, attached_mount_id);
    }
}

/// What:
///  Prove the mount the attach made is the mount the canonical slot shows,
///  and roll it back when it is not.
/// Why:
///  Both proofs the design asks for run before a sweep reports a clone as
///  attached,
///  and a mount that cannot be recorded is rolled back rather than left at a
///  name nobody verified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkPlace(request, held(slot), attachedMountId);
/// ```
fn check_place_the_mount(request: &AttachRequest<'_>, held: &HeldCloneSlot, attached_mount_id: u64) -> AttachStepOutcome {
    let held_parent_mount_id: u64 = match mount_id_of(&held.parent) {
        Ok(found) => found,
        Err(error) => {
            return AttachStepOutcome::Retained {
                said: format!(
                    "the attach made mount {} and the mount the parent folder is on could not be read ({error}), so nothing proved where it landed and nothing else was tried. Ways forward: an administrator reads the mount's point from {} and takes mount {} down through the descriptor unmount route.",
                    attached_mount_id,
                    crate::fs_mount_options::MOUNTINFO_PATH,
                    attached_mount_id
                ),
            };
        }
    };
    let held_target_mount_id: u64 = match mount_id_at_the_held_target(&held.parent, request.slot) {
        Ok(found) => found,
        Err(refusal) => return roll_back(held, attached_mount_id, held_parent_mount_id, refusal.message()),
    };
    let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: request.checkout, slot: request.slot });
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id,
        held_target_mount_id,
        attached_mount_id,
        canonical_slot: &held.canonical_slot,
    };
    return match post_attach_check(&check, fresh) {
        Ok(AttachOutcome::Attached { mount, .. }) => {
            debug(TAG, format!("a clone's mount is at its canonical slot as mount {attached_mount_id}").as_str());
            AttachStepOutcome::Attached { mount }
        }
        Ok(AttachOutcome::Misplaced(misplaced)) => roll_back(held, misplaced.attached_mount_id, held_parent_mount_id, misplaced.reason.message()),
        Err(refusal) => roll_back(held, attached_mount_id, held_parent_mount_id, refusal.message()),
    };
}

/// What:
///  Take a mount the attach made down through the descriptor it acted on,
///  and report what the rollback did.
/// Why:
///  The plan carries the held target,
///  so the unmount acts on the very folder the attach judged rather than on a
///  path the account could have changed,
///  and a refusal that proves no mount is retained and reported instead of
///  retried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = rollBack(held, attachedMountId, parentMountId, why);
/// ```
fn roll_back(held: &HeldCloneSlot, attached_mount_id: u64, parent_mount_id: u64, why: String) -> AttachStepOutcome {
    warn(TAG, format!("a clone's mount is not where the attach intended it: {why}").as_str());
    let target: std::fs::File = match held.target.try_clone() {
        Ok(target) => target,
        Err(error) => {
            return AttachStepOutcome::Retained {
                said: format!("{why} The attach made mount {attached_mount_id}, whose target's descriptor could not be held for the rollback ({error}), so nothing was tried."),
            };
        }
    };
    let plan: RollbackPlan = match plan_rollback_of(attached_mount_id, held.canonical_slot.clone(), target, parent_mount_id) {
        Ok(plan) => plan,
        Err(refusal) => return AttachStepOutcome::Retained { said: format!("{why} {}", refusal.message()) },
    };
    return match unmount_through_descriptor(&plan.target) {
        Ok(()) => {
            debug(TAG, "a misplaced mount left the kernel through the descriptor the attach acted on");
            AttachStepOutcome::RolledBack {
                said: format!(
                    "{why} The mount {attached_mount_id} left the kernel through the descriptor the attach acted on, and the slot {} holds no mount.",
                    plan.canonical_slot.display()
                ),
            }
        }
        Err(refusal) => AttachStepOutcome::Retained {
            said: format!(
                "{why} The rollback failed: {}. Nothing else was tried, and mount {attached_mount_id} is still attached where the attach left it, at {}. Ways forward: an administrator takes this exact mount down through the descriptor unmount route after verifying mount {attached_mount_id} and the clone's source, then runs the sweep again.",
                refusal.message(),
                plan.mounted_at.display()
            ),
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_attach_step_tests.rs"]
mod tests;
