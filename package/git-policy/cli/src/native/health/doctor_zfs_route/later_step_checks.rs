//! What:
//!  Every check plan step 18 lists that a later step builds,
//!  answered unknown with the step that builds it named.
//! Why:
//!  The step says it reports any check it cannot read or prove as unknown,
//!  and that the deferred-origin check is reported until step 15 lands
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "Open items").
//!  Step 15 has landed, so the deferred-origin check is `clone-origin`'s own
//!  line now,
//!  and the removal plan, the clone's own snapshots and the blocked-removal
//!  state are `removal-plan`'s, `clone-snapshots`' and `blocked-removal`'s.
//!  A check answered unknown here is not a gap in this build:
//!  it is the piece's boundary made visible,
//!  so a person reading the report learns which step to wait for rather than
//!  reading a pass this tree cannot prove.
//!  The list is one enum,
//!  so a later step's wiring changes one arm and no wording elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_later_step(LaterStep.CancelCommand); // { kind: 'unknown', reason: 'step 17 ...' }
//! ```

use super::check_outcome::{Check, CheckOutcome, unknown};

/// What:
///  One check a later plan step builds,
///  named by what it proves.
/// Why:
///  The report needs the check's stable handle and the step,
///  and a caller can list them without spelling a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LaterStep = 'cancel-command' | 'boot-protection' | 'service-unit' | 'backup-coverage' | 'sanoid';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaterStep {
    /// Step 17's cancel command.
    CancelCommand,
    /// Step 19's checkout boot protection and its immutable covered entry.
    BootProtection,
    /// Step 19's boot and shutdown units.
    ServiceUnit,
    /// Step 19's backup coverage.
    BackupCoverage,
    /// Step 20's Sanoid coverage and exclusions.
    Sanoid,
}

impl LaterStep {
    /// Every later-step check the report answers.
    pub const ALL: [LaterStep; 5] = [LaterStep::CancelCommand, LaterStep::BootProtection, LaterStep::ServiceUnit, LaterStep::BackupCoverage, LaterStep::Sanoid];

    /// The check's stable name.
    pub fn name(self) -> &'static str {
        return self.check().name();
    }

    /// The report check this answer belongs to.
    pub fn check(self) -> Check {
        return match self {
            LaterStep::CancelCommand => Check::CancelCommand,
            LaterStep::BootProtection => Check::BootProtection,
            LaterStep::ServiceUnit => Check::ServiceUnit,
            LaterStep::BackupCoverage => Check::BackupCoverage,
            LaterStep::Sanoid => Check::Sanoid,
        };
    }

    /// The plan step that builds it,
    /// and what that step proves.
    pub fn reason(self) -> String {
        return match self {
            LaterStep::CancelCommand => String::from(
                "step 17 builds the cancel command and what it proves, so a blocked removal's way back is not reported here and its plan and intent stay until that command exists",
            ),
            LaterStep::BootProtection => String::from(
                "step 19 builds the one-time migration and the immutable entry the checkout's underlay is protected by, so whether that protection stands is unknown here",
            ),
            LaterStep::ServiceUnit => String::from(
                "step 19 builds reconcile-git-worktree-clone-mounts.service and its shutdown unit, so the route's own unit is unknown here while ZED's own wiring is checked",
            ),
            LaterStep::BackupCoverage => String::from(
                "step 19 builds the backup coverage arm, so whether the configured backup job can see the checkout is unknown here rather than a failure",
            ),
            LaterStep::Sanoid => String::from(
                "step 20 builds the Sanoid coverage, the exclusion sections and the guard's report, so the dataset tree is not compared against recursive coverage here",
            ),
        };
    }
}

/// What:
///  The unknown answer for one check a later step builds.
/// Why:
///  The reason names the step,
///  so a person reading the report knows what to wait for and does not take
///  the unknown for a defect of this build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_later_step(step); // unknown('step 15 builds ...')
/// ```
pub fn check_later_step(step: LaterStep) -> CheckOutcome {
    return unknown(step.reason());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "later_step_checks_tests.rs"]
mod tests;
