//! What:
//!  Controls of the shutdown unmount's refusal:
//!  its mode word,
//!  the unit it waits for,
//!  and that running it answers the refusal rather than a detached mount.
//! Why:
//!  Unmounting from a fresh process is step 9 and the unit that runs it is step
//! 19's,
//!  so a build before the unit must refuse instead of detaching anything,
//!  and it must not send a reader to step 9's code,
//!  which is in the same binary
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//! step 19).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(AWAITING.arrives_with).toContain('plan step 19'); expect(run()).toEqual({ refused: AWAITING });
//! ```

use super::{
    AWAITING,
    run,
};

/// The refusal names the mode word and the unit that runs it,
/// and never claims that the step whose code landed lands the mode.
#[test]
fn the_refusal_names_the_mode_and_the_unit_that_runs_it() {
    assert_eq!(AWAITING.mode, "shutdown-unmount");
    assert!(
        AWAITING.arrives_with.contains("plan step 19"),
        "{}",
        AWAITING.arrives_with
    );
    let message: String = AWAITING.message();
    assert!(message.contains("shutdown-unmount"), "{message}");
    assert!(
        message.contains("it arrives with plan step 19"),
        "{message}"
    );
    assert!(
        message.contains("the unmount code itself landed with plan step 9"),
        "{message}"
    );
    assert!(
        !message.contains("plan step 9 lands it"),
        "step 9's code is in this binary, so it lands no mode: {message}"
    );
}

/// Running the mode answers the refusal,
/// never a success line.
#[test]
fn running_the_mode_answers_the_refusal() {
    assert_eq!(run(), Err(AWAITING));
}
