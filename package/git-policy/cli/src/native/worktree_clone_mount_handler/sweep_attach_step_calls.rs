//! What:
//!  The calls one attach step makes:
//!  the rollback's two calls and the one mount call.
//! Why:
//!  The real calls mount and unmount a clone,
//!  which no control on a person's machine may do,
//!  so each is a trait a control answers and a release type behind it,
//!  and the step's own order and its mapping of a refusal stay the release code
//!  in both arms.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface RollbackCalls { unmount(target): void; slotMount(request): number }
//! interface CloneMountCall { attachTheMount(source, target): AttachedMount }
//! interface CloneAttachStep { attach(request): AttachStepOutcome }
//! ```

use super::sweep_attach_step_answers::{
    AttachRequest,
    AttachStepOutcome,
};
use crate::worktree_clone_mount::{
    AttachRefusal,
    AttachedMount,
    LookAgainAtSlotRequest,
    attach_the_clone,
    look_again_at_the_slot,
    unmount_through_descriptor,
};

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
    fn unmount(
        &self,
        target: &std::fs::File,
    ) -> Result<(), crate::worktree_clone_mount::AttachRefusal>;

    /// The mount the canonical slot is on now,
    /// which is the parent folder's own mount when the slot holds none.
    fn slot_mount(
        &self,
        request: &AttachRequest<'_>,
    ) -> Result<u64, String>;
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
    fn unmount(
        &self,
        target: &std::fs::File,
    ) -> Result<(), crate::worktree_clone_mount::AttachRefusal> {
        return unmount_through_descriptor(target);
    }

    fn slot_mount(
        &self,
        request: &AttachRequest<'_>,
    ) -> Result<u64, String> {
        let fresh = look_again_at_the_slot(&LookAgainAtSlotRequest {
            checkout: request.checkout,
            slot: request.slot,
        })
        .map_err(|refusal| return refusal.message())?;
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
    fn attach(
        &self,
        request: &AttachRequest<'_>,
    ) -> AttachStepOutcome;
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
    fn attach_the_mount(
        &self,
        source: &str,
        target: &std::fs::File,
    ) -> Result<AttachedMount, AttachRefusal>;
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
    fn attach_the_mount(
        &self,
        source: &str,
        target: &std::fs::File,
    ) -> Result<AttachedMount, AttachRefusal> {
        return attach_the_clone(source, target);
    }
}
