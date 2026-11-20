//! What:
//!  Controls of the refusal a mode whose step has not landed gives.
//! Why:
//!  The sentence is what an operator reads out of a failed boot unit,
//!  so it must name the mode,
//!  the step and that nothing was touched,
//!  and the three unbuilt modes must name their own steps
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING.message()).toContain('boot-reconcile'); expect(AWAITING.step).toBe(6);
//! ```

use super::AwaitingItsStep;
use crate::worktree_clone_mount_handler::attaching_sweep_mode;
use crate::worktree_clone_mount_handler::boot_reconcile_mode;
use crate::worktree_clone_mount_handler::shutdown_unmount_mode;

/// The sentence names the mode,
/// what it will do,
/// the step that lands it,
/// and that nothing was touched.
#[test]
fn the_refusal_names_the_mode_the_step_and_that_nothing_was_touched() {
    let awaiting = AwaitingItsStep { mode: "shutdown-unmount", step: 9, does: "unmounts only the mounts this boot verified" };
    let message: String = awaiting.message();
    assert!(message.contains("shutdown-unmount"), "{message}");
    assert!(message.contains("plan step 9"), "{message}");
    assert!(message.contains("Nothing was opened"), "{message}");
    assert!(message.contains("--help"), "{message}");
}

/// Each unbuilt mode names the step the plan builds it in,
/// and refuses when it is run.
#[test]
fn each_unbuilt_mode_refuses_with_its_own_step() {
    assert_eq!(attaching_sweep_mode::AWAITING.step, 8);
    assert_eq!(boot_reconcile_mode::AWAITING.step, 6);
    assert_eq!(shutdown_unmount_mode::AWAITING.step, 9);
    assert_eq!(attaching_sweep_mode::run(), Err(attaching_sweep_mode::AWAITING));
    assert_eq!(boot_reconcile_mode::run(), Err(boot_reconcile_mode::AWAITING));
    assert_eq!(shutdown_unmount_mode::run(), Err(shutdown_unmount_mode::AWAITING));
}
