//! What:
//!  The report lines of the checks plan steps 17,
//! 19 and 20 add,
//!  in the order the report prints them:
//!  the cancel command,
//!  the checkout's boot protection,
//!  the boot setup's units,
//!  the backup coverage,
//!  and Sanoid's coverage.
//! Why:
//!  these five lines follow every clone's group and read facts rather than
//!  clones,
//!  so they are built here and `route_checks` keeps to its own order
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 18,
//!  "What it builds";
//!  step 19's boot setup and backup coverage;
//!  step 20's coverage).
//!  The order is the plan's,
//!  so a person reading a report finds the way back from a blocked removal
//!  before the setup that keeps the checkout mounted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = laterChecksReports(facts); // [{ check: 'cancel-command', outcome }, …]
//! ```

use super::backup_coverage_checks::check_backup_coverage;
use super::boot_protection_checks::{
    BootProtectionFacts,
    check_boot_protection,
};
use super::boot_setup_checks::check_service_unit;
use super::boot_setup_facts::BootSetupFacts;
use super::cancel_command_checks::{
    CancelCommandFacts,
    check_cancel_command,
};
use super::check_outcome::{
    Check,
    CheckReport,
};
use super::sanoid_report::SanoidReport;

/// What:
///  Every fact one of these lines reads,
///  gathered once.
/// Why:
///  the five answers belong to one machine's setup,
///  so one value holds them and a control changes one field to move one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { bootProtection, bootSetup, cancelCommand, sanoid };
/// ```
pub struct LaterCheckFacts<'a> {
    /// The checkout's covered entry and the account that must own it.
    pub boot_protection: &'a BootProtectionFacts,
    /// The boot setup's plan,
    /// the files installed from it and the datasets under the checkout.
    pub boot_setup: &'a BootSetupFacts,
    /// The cancel command this build carries,
    /// as its own grammar states it.
    pub cancel_command: &'a CancelCommandFacts,
    /// Sanoid's coverage comparison as the coverage folder answered it.
    pub sanoid: &'a SanoidReport,
}

/// What:
///  The five report lines,
///  in the order the plan lists the checks.
/// Why:
///  `route_checks` prints them after every clone's group and nothing else
///  decides their order,
///  so two runs of one machine are comparable and a control finds a line by
///  its check's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = laterChecksReports({ bootProtection, bootSetup, cancelCommand, sanoid });
/// ```
pub fn later_checks_reports(facts: &LaterCheckFacts<'_>) -> Vec<CheckReport> {
    return vec![
        CheckReport::new(
            Check::CancelCommand,
            check_cancel_command(facts.cancel_command),
        ),
        CheckReport::new(
            Check::BootProtection,
            check_boot_protection(facts.boot_protection),
        ),
        CheckReport::new(Check::ServiceUnit, check_service_unit(facts.boot_setup)),
        CheckReport::new(Check::BackupCoverage, check_backup_coverage()),
        CheckReport::new(Check::Sanoid, facts.sanoid.outcome.clone()),
    ];
}
