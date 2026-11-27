//! What:
//!  The attach step one clone goes through:
//!  the walk from the checkout to the slot,
//!  the answer a slot already standing on this clone's own mount gets instead
//!  of a refusal,
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
//!  The rollback reads the slot back before it says the slot holds no mount,
//!  because an unmount that answered success is not evidence of absence and an
//!  attach is only rolled back when the slot is proved clear again.
//!  A clone already attached at its slot is answered `done` rather than refused
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  A5):
//!  the walk stops at a mount it never crosses,
//!  so the slot's own mount is asked of the same proof the sweep's standing
//!  answer uses,
//!  and the answer names that mount instead of attempting a second one.
//!  Anything else at the slot keeps the walk's refusal.
//!  The two seams are what let a control drive the whole sweep and the
//!  rollback against fixture folders,
//!  because the real mount calls need the throwaway guest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = attachStep.attach({ checkout, slot, clone, store }); // attached | already-standing | rolled-back | retained | refused
//! ```

use super::sweep_attach_step_answers::{
    TAG,
    the_step_of_a_walk_refusal,
};
use crate::diagnostic_log::debug;
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::worktree_clone_mount::{
    AttachOutcome,
    AttachPostCheck,
    CloneSlotRefusal,
    HeldCloneSlot,
    HoldCloneSlotRequest,
    LookAgainAtSlotRequest,
    WalkRefusalReason,
    hold_clone_slot_under_root_control,
    look_again_at_the_slot,
    mount_id_at_the_held_target,
    post_attach_check,
};
use crate::worktree_clone_mount_handler::sweep_standing_mounts::StandingMounts;

/// The request one step is asked and the answers it gives,
/// the step each refusal names,
/// and the answer a refused attach call gets.
pub use super::sweep_attach_step_answers::{
    AttachRequest,
    AttachStepOutcome,
    IDENTITY_COMPARE_STEP,
    VERIFY_STEP,
    WALK_STEP,
    the_outcome_of_a_refused_attach,
};

/// The two rollback calls and the one mount call,
/// each as the seam a control answers and as the release call behind it,
/// with the step a sweep drives.
pub use super::sweep_attach_step_calls::{
    CloneAttachStep,
    CloneMountCall,
    RealCloneMountCall,
    RealRollbackCalls,
    RollbackCalls,
};

/// The rollback of a mount the attach made that landed beside its slot.
pub use super::sweep_attach_step_rollback::roll_back_a_mount;

/// What:
///  The release step:
///  the walk,
///  the answer an occupied slot gets,
///  the mount and both post-attach proofs,
///  and the rollback.
/// Why:
///  This is the shape the design states,
///  and it is one implementation so the order cannot differ between the sweep
///  a boot runs and the sweep a ZED event starts.
///  The rollback's own calls are a type parameter so a control can answer them,
///  the standing-mount proof is the sweep's own reader so the slot's mount is
///  judged by one proof and not a second copy of it,
///  and the release types are the defaults.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = RealAttachStep.release(standing).attach(request);
/// ```
#[derive(Clone)]
pub struct RealAttachStep<
    'standing,
    Calls: RollbackCalls = RealRollbackCalls,
    Mount: CloneMountCall = RealCloneMountCall,
> {
    /// The rollback's two calls.
    pub calls: Calls,
    /// The one mount call.
    pub mount_call: Mount,
    /// The proof that the slot's mount is this clone's own,
    /// which is the reader the sweep answers a standing clone from.
    pub standing: &'standing dyn StandingMounts,
}

impl<'standing> RealAttachStep<'standing, RealRollbackCalls, RealCloneMountCall> {
    /// What:
    ///  The release step over the sweep's standing-mount reader.
    /// Why:
    ///  One reader serves the whole run,
    ///  so the answer before the walk and the answer at the walk read one copy
    ///  of the kernel's table rather than two.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const step = RealAttachStep.release(standing);
    /// ```
    pub fn release(standing: &'standing dyn StandingMounts) -> Self {
        return Self {
            calls: RealRollbackCalls,
            mount_call: RealCloneMountCall,
            standing,
        };
    }
}

/// What:
///  Whether a walk stopped because a mount stands at the slot it was asked for.
/// Why:
///  Only that one refusal can be answered by the clone's own mount:
///  a link,
///  a missing folder or a folder that moved is a different fault,
///  and `openat2`'s `EXDEV` is what the walk reports both for a mount point and
///  for a folder on another filesystem,
///  so the refusal's own kind is what this reads rather than its sentence.
///  A boundary above the slot is no risk of a false answer:
///  the standing proof walks the same chain,
///  which crosses only the slot's own mount,
///  so it answers nothing there and the walk's refusal stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const atTheSlot = theWalkStoppedAtTheSlotsMount(refusal); // refusal.kind === 'walk' && refusal.reason === 'boundary'
/// ```
fn the_walk_stopped_at_the_slots_mount(refusal: &CloneSlotRefusal) -> bool {
    return match refusal {
        CloneSlotRefusal::Walk(walk) => walk.reason == WalkRefusalReason::BoundaryCrossed,
        CloneSlotRefusal::Store(_)
        | CloneSlotRefusal::ParentIdentity {
            ..
        }
        | CloneSlotRefusal::SlotIdentity {
            ..
        }
        | CloneSlotRefusal::TargetAnotherObject {
            ..
        } => false,
    };
}

impl<'standing, Calls: RollbackCalls, Mount: CloneMountCall>
    RealAttachStep<'standing, Calls, Mount>
{
    /// What:
    ///  The answer a walk that stopped before the slot gets:
    ///  this clone's own mount where the walk stopped at the slot's mount,
    ///  and the walk's own refusal otherwise.
    /// Why:
    ///  The request is for a clone attached at its slot,
    ///  and a second attach would be refused by the kernel at that mount;
    ///  the slot's mount is therefore proved by the sweep's standing answer,
    ///  which reads the kernel's table and asks the kernel for that mount's
    ///  source,
    ///  and only a mount it names as this clone's own is answered.
    ///  A slot holding another dataset's mount,
    ///  a bind of this clone or a mount without the two attributes the attach
    ///  itself enforces is none of those answers,
    ///  so the walk's boundary refusal stands exactly as it did.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const outcome = step.theAnswerARefusedWalkGets(request, walkRefusal);
    /// ```
    fn the_answer_a_refused_walk_gets(
        &self,
        request: &AttachRequest<'_>,
        refusal: &CloneSlotRefusal,
    ) -> AttachStepOutcome {
        if the_walk_stopped_at_the_slots_mount(refusal) {
            let at_the_slot: LookAgainAtSlotRequest<'_> = LookAgainAtSlotRequest {
                checkout: request.checkout,
                slot: request.slot,
            };
            if let Some(mount) = self.standing.standing_mount_of(&at_the_slot, request.clone) {
                debug(TAG, format!("the clone `{}` already holds its own mount {} at slot `{}`, so nothing was attached", request.clone, mount.mount_id, request.slot).as_str());
                return AttachStepOutcome::AlreadyStanding {
                    mount,
                };
            }
        }
        let (step, errno): (&str, Option<String>) = the_step_of_a_walk_refusal(refusal);
        return AttachStepOutcome::Refused {
            step: String::from(step),
            errno,
            said: refusal.message(),
        };
    }
}

impl<'standing, Calls: RollbackCalls, Mount: CloneMountCall> CloneAttachStep
    for RealAttachStep<'standing, Calls, Mount>
{
    fn attach(
        &self,
        request: &AttachRequest<'_>,
    ) -> AttachStepOutcome {
        let walk: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest {
            checkout: request.checkout,
            slot: request.slot,
            store: request.store,
        };
        let held: HeldCloneSlot = match hold_clone_slot_under_root_control(&walk) {
            Ok(held) => held,
            Err(refusal) => return self.the_answer_a_refused_walk_gets(request, &refusal),
        };
        let attached_mount_id: u64 = match self
            .mount_call
            .attach_the_mount(request.clone, &held.target)
        {
            Ok(attached) => attached.mount_id,
            Err(refusal) => return the_outcome_of_a_refused_attach(&refusal),
        };
        return check_place_the_mount(request, &held, attached_mount_id, &self.calls);
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
/// const outcome = checkPlace(request, held(slot), attachedMountId, calls);
/// ```
fn check_place_the_mount<Calls: RollbackCalls>(
    request: &AttachRequest<'_>,
    held: &HeldCloneSlot,
    attached_mount_id: u64,
    calls: &Calls,
) -> AttachStepOutcome {
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
        Err(refusal) => {
            return roll_back_a_mount(
                request,
                held,
                attached_mount_id,
                held_parent_mount_id,
                refusal.message(),
                calls,
            );
        }
    };
    let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest {
        checkout: request.checkout,
        slot: request.slot,
    });
    let check: AttachPostCheck<'_> = AttachPostCheck {
        held_parent_identity: &held.parent_identity,
        held_parent_mount_id,
        held_target_mount_id,
        attached_mount_id,
        canonical_slot: &held.canonical_slot,
    };
    return match post_attach_check(&check, fresh) {
        Ok(AttachOutcome::Attached {
            mount,
            ..
        }) => {
            debug(
                TAG,
                format!("a clone's mount is at its canonical slot as mount {attached_mount_id}")
                    .as_str(),
            );
            AttachStepOutcome::Attached {
                mount,
            }
        }
        Ok(AttachOutcome::Misplaced(misplaced)) => roll_back_a_mount(
            request,
            held,
            misplaced.attached_mount_id,
            held_parent_mount_id,
            misplaced.reason.message(),
            calls,
        ),
        Err(refusal) => roll_back_a_mount(
            request,
            held,
            attached_mount_id,
            held_parent_mount_id,
            refusal.message(),
            calls,
        ),
    };
}
/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_attach_step_tests.rs"]
mod tests;

/// The four answers an occupied or empty slot gets,
/// against a real mount no control on a person's machine may make of a ZFS
/// dataset.
#[cfg(test)]
#[cfg(target_os = "linux")]
#[path = "sweep_attach_step_standing_tests.rs"]
mod standing_tests;
