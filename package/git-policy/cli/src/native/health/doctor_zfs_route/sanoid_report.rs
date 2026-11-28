//! What:
//!  The Sanoid coverage comparison as `doctor`'s report takes it:
//!  the one line the check answers with,
//!  and every drift with the correction a person applies below it.
//! Why:
//!  the comparison is built in `health/sanoid_coverage/`,
//!  which already answers `doctor`'s own outcome type,
//!  and this report holds one line per check plus the lines a person acts on,
//!  so the two shapes have to meet somewhere
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 20,
//!  "What it builds";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints",
//!  which asks for the concrete INI or `zfs set` correction for confirmed
//!  drift).
//!  They meet here and not the other way round:
//!  the coverage folder names this folder for the outcome type,
//!  so a name back from here would put the two folders in a loop
//!  (`AGENTS.md` rule LYL).
//!  The coverage folder is therefore what builds this value from its own
//!  comparison (`sanoid_coverage::coverage_checks::sanoid_report`),
//!  and this file only says what the report reads from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = { outcome: { kind: 'finding', field: 'section-order', detail: '…' }, lines: ['finding: …', 'correction: …'] };
//! ```

use super::check_outcome::CheckOutcome;

/// What:
///  What the Sanoid coverage comparison answered,
///  and the lines a person acts on.
/// Why:
///  the report's one line per check cannot hold a correction,
///  so the drifts and their corrections travel beside the answer rather than
///  inside it,
///  and the wiring run prints them under the check's own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = { outcome, lines };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanoidReport {
    /// The check's answer,
    /// which is a pass only over a configuration and a tree both read.
    pub outcome: CheckOutcome,
    /// Every drift and the correction beside it,
    /// in the order the comparison found them,
    /// with the fact that stopped the comparison last.
    pub lines: Vec<String>,
}

impl SanoidReport {
    /// One comparison's answer and the lines below it.
    ///
    /// Why:
    ///  the coverage folder builds this value from its own answer,
    ///  so one constructor is where the two shapes meet.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new SanoidReport(answer.outcome, answer.lines());
    /// ```
    pub fn new(
        outcome: CheckOutcome,
        lines: Vec<String>,
    ) -> Self {
        return SanoidReport {
            outcome,
            lines,
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sanoid_report_tests.rs"]
mod tests;
