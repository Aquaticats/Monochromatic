//! What:
//!  The boot unit's reconciliation:
//!  the start-time compatibility gate,
//!  the keys and the checkout,
//!  the durable intent,
//!  and this boot's readiness,
//!  before the sweep runs.
//! Why:
//!  ZED alone cannot order a sweep after the checkout is ready,
//!  so a systemd unit runs this mode at boot
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  Its first action is the gate both programs run before their first ioctl,
//!  which is plan step 6,
//!  so this build refuses the mode before it reads any intent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = run(); // { refused: 'step 6' } until the gate and the unit land
//! ```

use super::mode_awaiting_its_step::AwaitingItsStep;

/// The mode this build refuses,
/// its plan step and the work that step adds.
pub const AWAITING: AwaitingItsStep = AwaitingItsStep {
    mode: "boot-reconcile",
    step: 6,
    does: "verifies the start-time kernel compatibility gate, the keys and the checkout, reconciles the durable mount intent and publishes this boot's readiness",
};

/// Refuse boot reconciliation until its step lands.
///
/// # Errors
/// Always,
/// with the refusal naming the step that builds it.
pub fn run() -> Result<String, AwaitingItsStep> {
    return Err(AWAITING);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_reconcile_mode_tests.rs"]
mod tests;
