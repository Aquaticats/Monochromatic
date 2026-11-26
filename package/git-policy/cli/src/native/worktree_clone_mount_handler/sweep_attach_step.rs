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

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath_mount_facts::mount_id_of;
use crate::start_probe::probe_errno_names::ErrnoWord;
use crate::worktree_clone_mount::{
    AttachOutcome, AttachPostCheck, AttachRefusal, AttachedMount, CloneSlotRefusal, HeldCloneSlot, HoldCloneSlotRequest, LookAgainAtSlotRequest, MountIdentity, RollbackPlan, WalkRefusalReason, attach_the_clone,
    hold_clone_slot_under_root_control, look_again_at_the_slot, mount_id_at_the_held_target, plan_rollback_of, post_attach_check, unmount_through_descriptor,
};
use crate::worktree_clone_mount_handler::sweep_standing_mounts::StandingMounts;
use std::path::Path;

/// Module tag for diagnostics.
const TAG: &str = "sweepAttachStep";

/// The step a comparison or an identity check is written as,
/// which the design's own step list names.
pub const IDENTITY_COMPARE_STEP: &str = "identity-compare";

/// The step the no-crossing walk is written as.
pub const WALK_STEP: &str = "walk";

/// The step the post-attach proofs are written as,
/// including the read-back of the mount the attach made.
pub const VERIFY_STEP: &str = "verify";

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
///  A refusal also carries the step that failed and its errno,
///  because the receipt names the call rather than "the attach".
///  A clone already at its slot is its own answer:
///  no walk below the slot,
///  no mount this step made and no post-attach proof ran,
///  so it cannot read as an attach this step made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttachStepOutcome = { kind: 'attached'; mount } | { kind: 'already-standing'; mount } | { kind: 'rolled-back'; said } | { kind: 'retained'; said } | { kind: 'refused'; step; errno; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttachStepOutcome {
    /// The mount the attach made is the mount the canonical slot shows.
    Attached {
        /// The mount,
        /// as the kernel lists it.
        mount: MountIdentity,
    },
    /// The clone's own mount already stands at its slot and this step made none.
    AlreadyStanding {
        /// The mount the slot holds,
        /// proved to be this clone's own by the sweep's own standing answer.
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
    /// the walk,
    /// an identity compare or a mount call refused.
    Refused {
        /// The handler's own step that refused,
        /// such as `walk`,
        /// `identity-compare` or `move-mount`.
        step: String,
        /// The errno's name,
        /// where the kernel answered one.
        errno: Option<String>,
        /// The refusal in words.
        said: String,
    },
}

/// What:
///  The two calls a rollback makes once it has its plan.
/// Why:
///  The release path unmounts through the descriptor and reads the canonical
///  slot back,
///  and a control answers both without a mount,
///  so the rule "a rollback is only reported when the slot is proved clear" is
///  provable here rather than only in the guest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RollbackCalls { unmount(target): void; slotMount(request): number }
/// ```
pub trait RollbackCalls {
    /// Take the planned mount down through the descriptor the attach acted on.
    fn unmount(&self, target: &std::fs::File) -> Result<(), crate::worktree_clone_mount::AttachRefusal>;

    /// The mount the canonical slot is on now,
    /// which is the parent folder's own mount when the slot holds none.
    fn slot_mount(&self, request: &AttachRequest<'_>) -> Result<u64, String>;
}

/// What:
///  The release calls:
///  the descriptor-bound unmount and the canonical slot's mount.
/// Why:
///  Both act on the kernel's own numbers rather than on a path the account could
///  have changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const calls = new RealRollbackCalls();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RealRollbackCalls;

impl RollbackCalls for RealRollbackCalls {
    fn unmount(&self, target: &std::fs::File) -> Result<(), crate::worktree_clone_mount::AttachRefusal> {
        return unmount_through_descriptor(target);
    }

    fn slot_mount(&self, request: &AttachRequest<'_>) -> Result<u64, String> {
        let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest { checkout: request.checkout, slot: request.slot }).map_err(|refusal| return refusal.message())?;
        return Ok(fresh.mount_id);
    }
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
///  The one mount call the release step makes.
/// Why:
///  The real call mounts a clone and only the guest may run it,
///  while a control needs the step's own order and its mapping of a refusal to
///  a retained answer;
///  the seam is the mount alone,
///  so the walk,
///  the proofs and the rollback stay the release code in both arms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CloneMountCall { attachTheMount(source, target): AttachedMount }
/// ```
pub trait CloneMountCall {
    /// Attach one clone's dataset to the target the walk held.
    fn attach_the_mount(&self, source: &str, target: &std::fs::File) -> Result<AttachedMount, AttachRefusal>;
}

/// What:
///  The release mount call:
///  step 4's descriptor attach.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const call = new RealCloneMountCall();
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RealCloneMountCall;

impl CloneMountCall for RealCloneMountCall {
    fn attach_the_mount(&self, source: &str, target: &std::fs::File) -> Result<AttachedMount, AttachRefusal> {
        return attach_the_clone(source, target);
    }
}

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
pub struct RealAttachStep<'standing, Calls: RollbackCalls = RealRollbackCalls, Mount: CloneMountCall = RealCloneMountCall> {
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
        return Self { calls: RealRollbackCalls, mount_call: RealCloneMountCall, standing };
    }
}

impl<'standing, Calls: RollbackCalls, Mount: CloneMountCall> RealAttachStep<'standing, Calls, Mount> {
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
    fn the_answer_a_refused_walk_gets(&self, request: &AttachRequest<'_>, refusal: &CloneSlotRefusal) -> AttachStepOutcome {
        if the_walk_stopped_at_the_slots_mount(refusal) {
            let at_the_slot: LookAgainAtSlotRequest<'_> = LookAgainAtSlotRequest { checkout: request.checkout, slot: request.slot };
            if let Some(mount) = self.standing.standing_mount_of(&at_the_slot, request.clone) {
                debug(TAG, format!("the clone `{}` already holds its own mount {} at slot `{}`, so nothing was attached", request.clone, mount.mount_id, request.slot).as_str());
                return AttachStepOutcome::AlreadyStanding { mount };
            }
        }
        let (step, errno): (&str, Option<String>) = the_step_of_a_walk_refusal(refusal);
        return AttachStepOutcome::Refused { step: String::from(step), errno, said: refusal.message() };
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const atTheSlot = theWalkStoppedAtTheSlotsMount(refusal); // refusal.kind === 'walk' && refusal.reason === 'boundary'
/// ```
fn the_walk_stopped_at_the_slots_mount(refusal: &CloneSlotRefusal) -> bool {
    return match refusal {
        CloneSlotRefusal::Walk(walk) => walk.reason == WalkRefusalReason::BoundaryCrossed,
        CloneSlotRefusal::Store(_) | CloneSlotRefusal::ParentIdentity { .. } | CloneSlotRefusal::SlotIdentity { .. } | CloneSlotRefusal::TargetAnotherObject { .. } => false,
    };
}

impl<'standing, Calls: RollbackCalls, Mount: CloneMountCall> CloneAttachStep for RealAttachStep<'standing, Calls, Mount> {
    fn attach(&self, request: &AttachRequest<'_>) -> AttachStepOutcome {
        let walk: HoldCloneSlotRequest<'_> = HoldCloneSlotRequest { checkout: request.checkout, slot: request.slot, store: request.store };
        let held: HeldCloneSlot = match hold_clone_slot_under_root_control(&walk) {
            Ok(held) => held,
            Err(refusal) => return self.the_answer_a_refused_walk_gets(request, &refusal),
        };
        let attached_mount_id: u64 = match self.mount_call.attach_the_mount(request.clone, &held.target) {
            Ok(attached) => attached.mount_id,
            Err(refusal) => return the_outcome_of_a_refused_attach(&refusal),
        };
        return check_place_the_mount(request, &held, attached_mount_id, &self.calls);
    }
}

/// What:
///  What one refused attach call answers a sweep.
/// Why:
///  A call after the move can have placed a mount that could not be taken down
///  again,
///  and that answer must read as a retained mount,
///  which files a failure receipt,
///  rather than as a refusal that left nothing behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = theOutcomeOfARefusedAttach(refusal); // refused | retained
/// ```
pub fn the_outcome_of_a_refused_attach(refusal: &crate::worktree_clone_mount::AttachRefusal) -> AttachStepOutcome {
    if refusal.placement == crate::worktree_clone_mount::Placement::PlacedAndRetained {
        return AttachStepOutcome::Retained { said: refusal.message() };
    }
    return AttachStepOutcome::Refused { step: String::from(refusal.call.word()), errno: the_errno_name(refusal.errno), said: refusal.message() };
}

/// The step and errno one walk refusal is written as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [step, errno] = stepOfWalkRefusal(refusal); // ['walk', undefined] or ['identity-compare', undefined]
/// ```
fn the_step_of_a_walk_refusal(refusal: &CloneSlotRefusal) -> (&'static str, Option<String>) {
    return match refusal {
        CloneSlotRefusal::Walk(_walk) => (WALK_STEP, None),
        CloneSlotRefusal::Store(_) | CloneSlotRefusal::ParentIdentity { .. } | CloneSlotRefusal::SlotIdentity { .. } | CloneSlotRefusal::TargetAnotherObject { .. } => {
            (IDENTITY_COMPARE_STEP, None)
        }
    };
}

/// The errno's name,
/// where a call answered one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = refusal.errno ? errnoName(refusal.errno) : undefined;
/// ```
fn the_errno_name(errno: i32) -> Option<String> {
    return if errno == 0 { None } else { Some(ErrnoWord::of_number(Some(errno)).text()) };
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
fn check_place_the_mount<Calls: RollbackCalls>(request: &AttachRequest<'_>, held: &HeldCloneSlot, attached_mount_id: u64, calls: &Calls) -> AttachStepOutcome {
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
        Err(refusal) => return roll_back_a_mount(request, held, attached_mount_id, held_parent_mount_id, refusal.message(), calls),
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
        Ok(AttachOutcome::Misplaced(misplaced)) => roll_back_a_mount(request, held, misplaced.attached_mount_id, held_parent_mount_id, misplaced.reason.message(), calls),
        Err(refusal) => roll_back_a_mount(request, held, attached_mount_id, held_parent_mount_id, refusal.message(), calls),
    };
}

/// What:
///  Take a mount the attach made down through the descriptor it acted on,
///  read the slot back,
///  and report what the rollback did.
/// Why:
///  The plan carries the held target,
///  so the unmount acts on the very folder the attach judged rather than on a
///  path the account could have changed,
///  and a refusal that proves no mount is retained and reported instead of
///  retried.
///  The read-back is what makes the success text true:
///  an unmount that answered zero is reported as a rollback only when the
///  canonical slot is on the parent folder's own mount again,
///  and a slot that still holds a mount is retained and named by its number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = rollBackAMount(request, held, attachedMountId, parentMountId, why, calls);
/// ```
pub fn roll_back_a_mount<Calls: RollbackCalls>(request: &AttachRequest<'_>, held: &HeldCloneSlot, attached_mount_id: u64, parent_mount_id: u64, why: String, calls: &Calls) -> AttachStepOutcome {
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
    let unmounted: Result<(), crate::worktree_clone_mount::AttachRefusal> = calls.unmount(&plan.target);
    return match unmounted {
        Ok(()) => {
            let observed: Result<u64, String> = calls.slot_mount(request);
            return match observed {
                Ok(mount_id) if mount_id == parent_mount_id => {
                    debug(TAG, "a misplaced mount left the kernel through the descriptor the attach acted on, and the slot holds no mount");
                    AttachStepOutcome::RolledBack {
                        said: format!(
                            "{why} The mount {attached_mount_id} left the kernel through the descriptor the attach acted on, and the slot {} holds no mount.",
                            plan.canonical_slot.display()
                        ),
                    }
                }
                Ok(mount_id) => AttachStepOutcome::Retained {
                    said: format!(
                        "{why} The unmount answered success and the slot {} is still on mount {mount_id}, so the rollback is not reported as done. Nothing else was tried. Ways forward: an administrator reads `{}` for the slot's mount and takes mount {mount_id} down through the descriptor route.",
                        plan.canonical_slot.display(),
                        crate::fs_mount_options::MOUNTINFO_PATH
                    ),
                },
                Err(refusal) => AttachStepOutcome::Retained {
                    said: format!(
                        "{why} The unmount answered success and the slot {} could not be read back ({refusal}), so nothing proves the mount left. Nothing else was tried. Ways forward: an administrator reads `{}` for the slot's mount and takes it down through the descriptor route.",
                        plan.canonical_slot.display(),
                        crate::fs_mount_options::MOUNTINFO_PATH
                    ),
                },
            };
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

/// The four answers an occupied or empty slot gets,
/// against a real mount no control on a person's machine may make of a ZFS
/// dataset.
#[cfg(test)]
#[cfg(target_os = "linux")]
#[path = "sweep_attach_step_standing_tests.rs"]
mod standing_tests;
