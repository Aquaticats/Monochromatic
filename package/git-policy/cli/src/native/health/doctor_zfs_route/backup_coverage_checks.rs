//! What:
//!  The backup coverage plan step 19 owes `doctor`,
//!  answered unknown because the arm that reads it is not built.
//! Why:
//!  the step's acceptance test asks which paths the configured backup job can
//!  see,
//!  whether it crosses the ZFS boundary at the checkout,
//!  and whether the checkout and its configuration restore with origins still
//!  pinned
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "Acceptance tests").
//!  No code on this branch reads a backup job's configuration,
//!  its snapshot history or a restore,
//!  so the check answers unknown with the fact that nobody gathered one,
//!  which is what the step asks a check it cannot prove to say
//!  (`package/git-policy/cli/SPEC.md`,
//!  "### Doctor").
//!  The reason names the step and the arm rather than promising a pass,
//!  because a backup nobody configures is the state that costs a person their
//!  work
//!  and a pass this tree cannot prove would hide it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkBackupCoverage(); // { kind: 'unknown', reason: "step 19's backup coverage arm is not built …" }
//! ```

use super::check_outcome::{
    CheckOutcome,
    unknown,
};

/// What:
///  What the backup coverage answers:
///  unknown,
///  naming the step and the arm this branch does not carry.
/// Why:
///  a check that could not read a fact is reported as unknown and never as a
///  pass or a failure,
///  and a person has to learn that nobody has asked the backup job anything
///  rather than read a green line over a checkout nothing saves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkBackupCoverage(); // unknown
/// ```
pub fn check_backup_coverage() -> CheckOutcome {
    return unknown(String::from(
        "step 19's backup coverage arm is not built, so which paths the configured backup job can see, whether it crosses the ZFS boundary at the checkout, and whether the checkout and its configuration restore with origins still pinned are all unread, and no value this build holds answers them",
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "backup_coverage_checks_tests.rs"]
mod tests;
