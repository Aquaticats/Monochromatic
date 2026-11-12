//! What:
//!  How long a commit whose branch moved under its build is built again,
//! and the line that says so at each rebuild.
//! Why:
//!  A count of builds ended a commit on 2026-11-08 that conflicted with nothing,
//! so the bound is the time since the first build (`MOVED_BRANCH_REBUILD_CEILING`,
//! `service/wait_ceilings.rs`),
//! and every rebuild tells the person what it is for and when it stops.
//! The decision is split from `service_commit.rs`,
//! which holds the loop that uses it and is at its line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!rebuildMayContinue(sinceFirstBuild, rebuildCeiling())) fail(); sayRebuilding(branch, builds, sinceFirstBuild);
//! ```

use crate::service_messages::rebuild_notice;
use crate::wait_ceilings::MOVED_BRANCH_REBUILD_CEILING;

/// The environment variable that sets the ceiling of the rebuilding in whole seconds,
/// for a control that must watch the ceiling end a branch that never stops moving without waiting the measured twenty minutes.
pub const REBUILD_CEILING_VARIABLE: &str = "CLI_GIT_REBUILD_CEILING_SECONDS";

/// The whole seconds a setting of `REBUILD_CEILING_VARIABLE` names.
fn seconds_in(text: String) -> Option<u64> {
    return text.trim().parse::<u64>().ok();
}

/// The ceiling of the rebuilding:
/// `REBUILD_CEILING_VARIABLE` when it names a positive number of seconds,
/// else `MOVED_BRANCH_REBUILD_CEILING`.
pub(crate) fn rebuild_ceiling() -> std::time::Duration {
    return match std::env::var(REBUILD_CEILING_VARIABLE).ok().and_then(seconds_in) {
        Some(seconds) if seconds > 0 => std::time::Duration::from_secs(seconds),
        _ => MOVED_BRANCH_REBUILD_CEILING,
    };
}

/// Whether a commit whose branch moved under it is built again,
/// given how long ago its first build began and the ceiling.
/// Why:
///   The bound is time,
/// not a count of builds:
/// a count ended a commit that conflicted with nothing while other agents landed on the same branch,
/// and a build holds no lock,
/// so only the total time needs a limit (`MOVED_BRANCH_REBUILD_CEILING`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waited < ceiling
/// ```
pub fn rebuild_may_continue(waited: std::time::Duration, ceiling: std::time::Duration) -> bool {
    return waited < ceiling;
}

/// Tells the person that the commit is built again because the branch moved,
/// how many builds were made,
/// how long it has been,
/// and when the rebuilding stops.
fn say_rebuilding(branch: &str, builds: usize, waited: std::time::Duration) {
    // A closed terminal is not a reason to stop the commit.
    let _written = std::io::Write::write_all(&mut std::io::stderr(), rebuild_notice(branch, builds, waited, rebuild_ceiling()).as_bytes());
}

/// Whether a commit whose branch moved is built again now:
/// when the ceiling allows it,
/// the person is told and the answer is yes.
pub(crate) fn may_rebuild(branch: &str, builds: usize, first_build_began: std::time::Instant) -> bool {
    let waited: std::time::Duration = first_build_began.elapsed();
    if !rebuild_may_continue(waited, rebuild_ceiling()) {
        return false;
    }
    say_rebuilding(branch, builds, waited);
    return true;
}

/// The rebuilding decision is controlled in the test module.
#[cfg(test)]
#[path = "service_commit_rebuild_ceiling_tests.rs"]
mod tests;

/// What:
///  What a service's `prepare-again` answer comes to:
///  the retry the next preparation makes,
///  or the stop that names the branch.
/// Why:
///   A commit Git built,
/// and an amend,
/// are never sent for another tip,
/// so the answer is a moved branch for them;
/// and every other command may rebuild what the preparation captured onto the branch as it is,
/// while the rebuild ceiling allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const after = afterAPrepareAgain({ native, amend, ready, attempt });
/// ```
pub(crate) enum AfterAPrepareAgain {
    /// What this preparation captured is merged onto the branch by the next one.
    Retry(crate::service_replay::Captured),
    /// The command stops,
    /// naming the branch.
    Stopped(crate::commit_transaction::TransactionStop),
}

/// What:
///  The retry a `prepare-again` answer makes,
/// or the stop that names the branch.
pub(crate) fn after_a_prepare_again(
    native: bool,
    amend: bool,
    ready: &crate::service_prepare::Ready,
    invocation: &crate::commit_transaction::CommitInvocation<'_>,
    landed: Vec<crate::service_protocol::LandedCapture>,
    attempt: usize,
    first_build_began: std::time::Instant,
) -> AfterAPrepareAgain {
    if native || amend {
        return AfterAPrepareAgain::Stopped(crate::service_branch_shows::branch_moved(ready));
    }
    let branch: String = ready.reference.short().spoken();
    if !may_rebuild(branch.as_str(), attempt + 1, first_build_began) {
        let failure: crate::service_messages::ClientFailure = crate::service_messages::ClientFailure::BranchKeepsChanging { branch, times: attempt + 1 };
        return AfterAPrepareAgain::Stopped(crate::commit_transaction::TransactionStop::Message(crate::service_messages::render(&failure)));
    }
    return match crate::service_replay::retry_of(invocation.context, invocation.cwd, ready, landed) {
        Ok(next) => AfterAPrepareAgain::Retry(next),
        Err(stop) => AfterAPrepareAgain::Stopped(stop),
    };
}
