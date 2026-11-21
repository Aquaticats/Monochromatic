//! What:
//!  Controls of the boot reconciliation's refusal:
//!  its mode word,
//!  its plan step,
//!  and that running it answers the refusal rather than a reconciled boot.
//! Why:
//!  The mode's first action is the start gate,
//!  which step 6 built,
//!  and the sweep that reconciles intent and publishes readiness is step 8,
//!  so this build must refuse instead of recording a readiness it never
//!  verified
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  steps 6 and 8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING.step).toBe(8); expect(run()).toEqual({ refused: AWAITING });
//! ```

use super::{AWAITING, run};

/// The refusal names the mode word and step 8.
#[test]
fn the_refusal_names_the_mode_and_its_step() {
    assert_eq!(AWAITING.mode, "boot-reconcile");
    assert_eq!(AWAITING.step, 8);
    assert!(AWAITING.message().contains("plan step 8"), "{}", AWAITING.message());
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}
