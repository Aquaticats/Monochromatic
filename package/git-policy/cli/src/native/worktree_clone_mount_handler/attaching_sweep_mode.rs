//! What:
//!  The clone-mount sweep:
//!  what the installed ZED event runs when it starts the program with no argument,
//!  and what `boot-reconcile` runs after its own checks.
//! Why:
//!  The sweep enumerates the pinned parent's clone datasets,
//!  reads each clone's durable mount intent and attaches only what that intent asks for
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering").
//!  It is plan step 8,
//! so this build refuses the mode instead of attaching anything:
//!  a partial sweep at root would be a mount nobody designed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = run(); // { refused: 'step 8' } until the sweep lands
//! ```

use super::mode_awaiting_its_step::AwaitingItsStep;

/// The mode this build refuses,
/// its plan step and the work that step adds.
pub const AWAITING: AwaitingItsStep = AwaitingItsStep {
    mode: "sweep (no argument)",
    step: 8,
    does: "enumerates the pinned parent's clone datasets, reads each clone's durable mount intent and attaches only the clones that ask for a mount",
};

/// Refuse the sweep until its step lands.
///
/// # Errors
/// Always,
/// with the refusal naming the step that builds it.
pub fn run() -> Result<String, AwaitingItsStep> {
    return Err(AWAITING);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "attaching_sweep_mode_tests.rs"]
mod tests;
