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
//!  Its first action is the gate the handler runs before its first ioctl,
//!  which plan step 6 built,
//!  and the sweep that reads intent and publishes readiness is plan step 8,
//!  so this build refuses the mode rather than publish a readiness record for
//!  a boot it never verified.
//!  The mode is step 8's,
//!  not step 6's and not step 19's:
//!  step 8 owns the gate's use,
//!  the intent reconciliation and the readiness publish this mode runs,
//!  and its "What it builds" names this mode,
//!  while step 19 only installs the unit that calls it
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  "Step 8:
//! The attaching sweep for a clone that carries a mount request",
//!  "What it builds",
//!  and "Step 19:
//! The one-time migration,
//! the immutable entry and the boot and shutdown units").
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
    step: 8,
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
