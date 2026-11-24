//! What:
//!  Every check plan step 18 lists that a later step builds,
//!  answered unknown with the step that builds it named.
//! Why:
//!  The step says it reports the deferred-origin check before step 15 lands
//!  and any check it cannot read or prove as unknown
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "Open items").
//!  A check answered unknown here is not a gap in this build:
//!  it is the piece's boundary made visible,
//!  so a person reading the report learns which step to wait for rather than
//!  reading a pass this tree cannot prove.
//!  The list is one enum,
//!  so a later step's wiring changes one arm and no wording elsewhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_later_step(LaterStep.DeferredOrigin); // { kind: 'unknown', reason: 'step 15 ...' }
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
/// type LaterStep = 'deferred-origin' | 'removal-plans' | 'cancel-command' | 'backup-coverage' | 'sanoid';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaterStep {
    /// Step 15's deferred-origin check.
    DeferredOrigin,
    /// Step 14's lineage proof and removal plan,
    /// and step 16's snapshot batch and head destroy.
    RemovalPlans,
    /// Step 17's cancel command.
    CancelCommand,
    /// Step 19's backup coverage.
    BackupCoverage,
    /// Step 20's Sanoid coverage and exclusions.
    Sanoid,
}

impl LaterStep {
    /// Every later-step check the report answers.
    pub const ALL: [LaterStep; 5] = [LaterStep::DeferredOrigin, LaterStep::RemovalPlans, LaterStep::CancelCommand, LaterStep::BackupCoverage, LaterStep::Sanoid];

    /// The check's stable name.
    pub fn name(self) -> &'static str {
        return self.check().name();
    }

    /// The report check this answer belongs to.
    pub fn check(self) -> Check {
        return match self {
            LaterStep::DeferredOrigin => Check::DeferredOrigin,
            LaterStep::RemovalPlans => Check::RemovalPlans,
            LaterStep::CancelCommand => Check::CancelCommand,
            LaterStep::BackupCoverage => Check::BackupCoverage,
            LaterStep::Sanoid => Check::Sanoid,
        };
    }

    /// The plan step that builds it,
    /// and what that step proves.
    pub fn reason(self) -> String {
        return match self {
            LaterStep::DeferredOrigin => String::from(
                "step 15 builds the deferred-origin check, so whether a clone's origin is marked for deferred destroy is not read here and unknown is the honest answer",
            ),
            LaterStep::RemovalPlans => String::from(
                "step 14 builds the lineage proof and the removal plan and step 16 builds the snapshot batch, so no plan or lineage state is read here",
            ),
            LaterStep::CancelCommand => String::from(
                "step 17 builds the cancel command and what it proves, so a blocked removal's way back is not reported here",
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
