//! What:
//!  Controls of the shape the Sanoid comparison travels in:
//!  the answer and the corrections below it are kept as they were given,
//!  in the order the comparison found them.
//! Why:
//!  the wiring run prints the corrections under this check's own line,
//!  so a shape that dropped or reordered them would hide the one line a
//!  person edits
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  `AGENTS.md` rules PKG and TCV).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = new SanoidReport({ kind: 'unknown', reason }, ['finding: …', 'correction: …']);
//! expect(report.lines).toEqual(['finding: …', 'correction: …']);
//! ```

use super::SanoidReport;
use crate::health::doctor_zfs_route::check_outcome::{
    CheckOutcome,
    finding,
    unknown,
};

/// The answer and the lines are kept as the comparison built them.
#[test]
fn the_answer_and_its_lines_are_kept() {
    let lines: Vec<String> = vec![
        String::from("finding: section-order: the [a] section stands before [b]"),
        String::from("correction: move the [a] section below [b]"),
    ];
    let outcome: CheckOutcome = finding("section-order", "1 finding(s): section-order");
    let report: SanoidReport = SanoidReport::new(outcome.clone(), lines.clone());
    assert_eq!(report.outcome, outcome);
    assert_eq!(report.lines, lines);
}

/// A comparison that stopped answers unknown,
/// and the fact that stopped it is the last line,
/// so the corrections it did find still print above it.
#[test]
fn an_unproved_answer_keeps_the_fact_that_stopped_it_last() {
    let lines: Vec<String> = vec![
        String::from("finding: recursion: `recursive` is `no`"),
        String::from("correction: set `recursive = yes` under [zpcachyos/home]"),
        String::from("unknown: the configuration holds a setting this build cannot judge"),
    ];
    let outcome: CheckOutcome =
        unknown("the configuration holds a setting this build cannot judge");
    let report: SanoidReport = SanoidReport::new(outcome, lines.clone());
    assert_eq!(report.lines, lines);
    assert_eq!(
        report.lines.last().map(String::as_str),
        Some(lines[2].as_str())
    );
}
