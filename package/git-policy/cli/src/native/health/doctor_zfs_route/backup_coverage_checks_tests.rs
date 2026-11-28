//! What:
//!  Controls of the backup coverage's answer:
//!  it is unknown,
//!  it names plan step 19,
//!  and it says which arm of that step is missing rather than promising a
//!  pass.
//! Why:
//!  a pass this tree cannot prove would tell a person their checkout is saved
//!  when nobody has asked the backup job anything,
//!  so the state is asserted whole
//!  (`AGENTS.md` rules PKG and TCV).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(checkBackupCoverage()).toEqual({ kind: 'unknown', reason: expect.stringContaining('step 19') });
//! ```

use super::check_backup_coverage;
use crate::health::doctor_zfs_route::check_outcome::{
    Check,
    CheckOutcome,
    CheckReport,
};

/// The backup coverage answers unknown.
#[test]
fn the_backup_coverage_is_unknown() {
    let outcome: CheckOutcome = check_backup_coverage();
    assert!(
        matches!(outcome, CheckOutcome::Unknown { .. }),
        "{outcome:?}"
    );
}

/// The reason names the plan step and the arm that is missing,
/// and what nobody has read.
#[test]
fn the_reason_names_the_step_and_the_missing_arm() {
    let outcome: CheckOutcome = check_backup_coverage();
    let report: CheckReport = CheckReport::new(Check::BackupCoverage, outcome);
    let line: String = report.line();
    assert!(
        line.contains("step 19's backup coverage arm is not built"),
        "{line}"
    );
    for word in [
        "which paths the configured backup job can see",
        "crosses the ZFS boundary",
        "origins still pinned",
    ] {
        assert!(line.contains(word), "{word} is in {line}");
    }
}

/// The line reports under the coverage's own check name,
/// so a person finds it where the step says it stands.
#[test]
fn the_line_reports_under_the_backup_coverage_name() {
    let report: CheckReport = CheckReport::new(Check::BackupCoverage, check_backup_coverage());
    assert!(
        report.line().starts_with("zfs: backup-coverage: unknown: "),
        "{}",
        report.line()
    );
}
