//! What:
//!  The shutdown unit's verified unmount:
//!  withdraw readiness,
//!  close admission to new attaches,
//!  and unmount only the mounts this boot verified,
//!  without changing durable intent.
//! Why:
//!  The export command unmounts datasets before it issues the export,
//!  and the export event is posted too late to order those unmounts,
//!  so the shutdown pass is a systemd unit and not a ZED event
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  Unmounting from a fresh process is plan step 9,
//!  and step 19 installs the unit that runs it,
//!  so this build refuses the mode rather than detach anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = run(); // { refused: 'step 9' } until the unmount half lands
//! ```

use super::mode_awaiting_its_step::AwaitingItsStep;

/// The mode this build refuses,
/// its plan step and the work that step adds.
pub const AWAITING: AwaitingItsStep = AwaitingItsStep {
    mode: "shutdown-unmount",
    step: 9,
    does: "withdraws readiness, closes admission to new attaches and unmounts only the mounts this boot verified, without changing durable intent",
};

/// Refuse the shutdown unmount until its step lands.
///
/// # Errors
/// Always,
/// with the refusal naming the step that builds it.
pub fn run() -> Result<String, AwaitingItsStep> {
    return Err(AWAITING);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "shutdown_unmount_mode_tests.rs"]
mod tests;
