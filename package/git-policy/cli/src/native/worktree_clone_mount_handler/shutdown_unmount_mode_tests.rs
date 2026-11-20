//! What:
//!  Controls of the shutdown unmount's refusal:
//!  its mode word,
//!  its plan step,
//!  and that running it answers the refusal rather than a detached mount.
//! Why:
//!  Unmounting from a fresh process is step 9,
//!  so a build before it must refuse instead of detaching anything
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 9).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING.step).toBe(9); expect(run()).toEqual({ refused: AWAITING });
//! ```

use super::{AWAITING, run};

/// The refusal names the mode word and step 9.
#[test]
fn the_refusal_names_the_mode_and_its_step() {
    assert_eq!(AWAITING.mode, "shutdown-unmount");
    assert_eq!(AWAITING.step, 9);
    assert!(AWAITING.message().contains("plan step 9"), "{}", AWAITING.message());
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}
