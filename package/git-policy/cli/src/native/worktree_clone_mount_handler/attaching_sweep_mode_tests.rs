//! What:
//!  Controls of the sweep's refusal:
//!  its mode name,
//!  its plan step,
//!  and that running it answers the refusal rather than a sweep's line.
//! Why:
//!  The default invocation is what the installed ZED event starts,
//!  so a build that answered it with success would report a sweep nobody ran
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, step 8).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING).toEqual({ mode: 'sweep (no argument)', step: 8, does: expect.any(String) });
//! ```

use super::{AWAITING, run};

/// The refusal names the default invocation and step 8.
#[test]
fn the_refusal_names_the_default_invocation_and_its_step() {
    assert_eq!(AWAITING.mode, "sweep (no argument)");
    assert_eq!(AWAITING.step, 8);
    assert!(AWAITING.message().contains("plan step 8"), "{}", AWAITING.message());
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}
