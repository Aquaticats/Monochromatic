//! What:
//!  The rollback of a mount the attach made that landed beside its slot:
//!  the plan,
//!  the unmount through the descriptor the attach acted on,
//!  and the read-back that makes the success text true.
//! Why:
//!  A mount that cannot be recorded is taken down at once,
//!  and one that cannot be taken down is reported with both paths and nothing
//!  else tried,
//!  so this is the half of the step that a misplaced attach always runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = rollBackAMount(request, held, attachedMountId, parentMountId, why, calls);
//! ```

use super::sweep_attach_step_answers::{
    AttachRequest,
    AttachStepOutcome,
    TAG,
};
use super::sweep_attach_step_calls::RollbackCalls;
use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::worktree_clone_mount::{
    HeldCloneSlot,
    RollbackPlan,
    plan_rollback_of,
};

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
pub fn roll_back_a_mount<Calls: RollbackCalls>(
    request: &AttachRequest<'_>,
    held: &HeldCloneSlot,
    attached_mount_id: u64,
    parent_mount_id: u64,
    why: String,
    calls: &Calls,
) -> AttachStepOutcome {
    warn(
        TAG,
        format!("a clone's mount is not where the attach intended it: {why}").as_str(),
    );
    let target: std::fs::File = match held.target.try_clone() {
        Ok(target) => target,
        Err(error) => {
            return AttachStepOutcome::Retained {
                said: format!(
                    "{why} The attach made mount {attached_mount_id}, whose target's descriptor could not be held for the rollback ({error}), so nothing was tried."
                ),
            };
        }
    };
    let plan: RollbackPlan = match plan_rollback_of(
        attached_mount_id,
        held.canonical_slot.clone(),
        target,
        parent_mount_id,
    ) {
        Ok(plan) => plan,
        Err(refusal) => {
            return AttachStepOutcome::Retained {
                said: format!("{why} {}", refusal.message()),
            };
        }
    };
    let unmounted: Result<(), crate::worktree_clone_mount::AttachRefusal> =
        calls.unmount(&plan.target);
    return match unmounted {
        Ok(()) => {
            let observed: Result<u64, String> = calls.slot_mount(request);
            return match observed {
                Ok(mount_id) if mount_id == parent_mount_id => {
                    debug(
                        TAG,
                        "a misplaced mount left the kernel through the descriptor the attach acted on, and the slot holds no mount",
                    );
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
