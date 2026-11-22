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
//!  When that step lands the run reads `.parent` and the record of each slot it reaches,
//!  and it lists no name:
//!  one folder listing per run is what the accepted design's choice 9 does not take.
//!  Until then a sweep run opens no folder and reads no record,
//!  so a staging file a killed writer left stays where it is.
//!  The adoption removes the two known staging forms,
//!  and plan step 18's `doctor` is to report and remove them
//!  (`doc/decision/cli-git-zfs-slot-identity-store-format.md`,
//!  "Who removes a killed writer's staging file"),
//!  so the adoption is their only remover until that step lands.
//!  A leftover standing until then affects no reader,
//! because no reader reads a staging name.
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
