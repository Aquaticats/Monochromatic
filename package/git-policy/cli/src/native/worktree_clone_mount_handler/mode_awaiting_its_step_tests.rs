//! What:
//!  Controls of the refusal a mode whose step has not landed gives.
//! Why:
//!  The sentence is what an operator reads out of a failed boot unit,
//!  so it must name the mode,
//!  what runs it,
//!  that nothing was touched,
//!  and the way forward,
//!  and the one mode whose step has not landed must arrive with the unit that
//!  runs it
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 19).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(awaiting.message()).toContain('arrives with plan step 19');
//! ```

use super::AwaitingItsStep;
use crate::worktree_clone_mount_handler::shutdown_unmount_mode;

/// The sentence names the mode,
/// what it will do,
/// what runs it,
/// and that nothing was touched.
#[test]
fn the_refusal_names_the_mode_the_arrival_and_that_nothing_was_touched() {
    let awaiting = AwaitingItsStep { mode: "shutdown-unmount", arrives_with: "plan step 19, whose boot unit runs this mode", does: "unmounts only the mounts this boot verified" };
    let message: String = awaiting.message();
    assert!(message.contains("shutdown-unmount"), "{message}");
    assert!(message.contains("it unmounts only the mounts this boot verified"), "{message}");
    assert!(message.contains("it arrives with plan step 19"), "{message}");
    assert!(message.contains("Nothing was opened"), "{message}");
    assert!(message.contains("--help"), "{message}");
}

/// The one mode whose step has not landed arrives with the unit that runs it,
/// and refuses when it is run.
#[test]
fn the_unbuilt_mode_refuses_and_names_its_arrival() {
    assert!(shutdown_unmount_mode::AWAITING.arrives_with.contains("plan step 19"), "{}", shutdown_unmount_mode::AWAITING.arrives_with);
    assert_eq!(shutdown_unmount_mode::run(), Err(shutdown_unmount_mode::AWAITING));
}
