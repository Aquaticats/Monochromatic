//! What:
//!  What is left to do once the ignored state of a new worktree is in place:
//! hand the trash a snapshot left there to
//!  the commit service,
//! and repair the package-manager state.
//! Why:
//!  A copy or a snapshot keeps links and build state exactly as the source had them,
//! so each tool's own offline
//!  repair makes them work at the new location;
//! and the trash of the leave-out is the service's work,
//! not the add's:
//! the add
//!  writes the record and returns,
//! so the person never waits for hundreds of thousands of deletions
//!  (`doc/planning/cli-git-worktree-add-cost-of-its-trash.md`).
//! Neither step can fail the add:
//! a repair that cannot finish
//!  names the command to run,
//! and a trash the service could not be told about is named with the record that still
//!  empties it later.
//! The code sits in a file of its own so `worktree_copy.rs` stays under the line limit,
//! and that file
//!  re-exports what its caller needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! handOverTrash(after.trash); repairCreated(after); // the trash's emptying is the service's
//! ```

use crate::shared_notice_stream::SharedNoticeStream;
use crate::worktree_copy::Invocation;
#[cfg(target_os = "linux")]
use crate::package_repair::{RepairEnvironment, RepairRequest, repair_worktree};
#[cfg(target_os = "linux")]
use crate::worktree_trash::PendingTrash;
use std::io::Write;
use std::path::PathBuf;

/// What:
///  What is left to do once the ignored state is in place:
/// the source the ignored files came from,
/// the worktrees the
///  command created,
/// and the trash a snapshot left.
/// Why:
///   The package-manager repairs touch only the new worktrees and read the source,
/// so no other add has to wait for them;
/// they
///  run last,
/// with the facts the copy found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AfterLock = { sourceRoot?: string; created: string[]; trash?: PendingTrash };
/// ```
pub(crate) struct AfterLock {
    /// The checkout the ignored files came from;
    /// none for a bare repository,
    /// which copies nothing.
    pub(crate) source_root: Option<PathBuf>,
    /// The roots of every worktree the command created,
    /// a snapshot's included;
    /// empty after a failed copy.
    pub(crate) created: Vec<PathBuf>,
    /// The trash a snapshot left in the worktree it carried,
    /// which the commit service empties after the command returns.
    #[cfg(target_os = "linux")]
    pub(crate) trash: Option<PendingTrash>,
}

/// What:
///  Repair the package-manager state of every worktree the command created,
/// once its ignored state is in place.
/// Why:
///   A copy or a snapshot keeps links and build state exactly as the source had them;
/// each tool's own offline repair makes
///  them work at the new location (`package_repair.rs`).
/// It never fails the add:
/// a repair that cannot finish says so in one
///  line that names the command to run.
/// A bare repository has no source worktree,
/// so nothing was copied and nothing is repaired.
///  It runs on the worktree the registration binding confirmed,
/// and no lock stands around it,
/// so a slow tool never holds
///  another add.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const root of roots) repairWorktree({ git, source, destination: root, switch, environment }, stderr);
/// ```
#[cfg(target_os = "linux")]
fn repair_created(invocation: &Invocation, after: &AfterLock, stderr: &mut dyn Write) {
    let Some(source) = after.source_root.as_deref() else { return };
    let environment: RepairEnvironment = RepairEnvironment::from_process();
    for root in &after.created {
        let request: RepairRequest<'_> = RepairRequest { git: &invocation.context, source, destination: root.as_path(), switch: invocation.repair, environment: &environment };
        repair_worktree(&request, stderr);
    }
}

/// What:
///  Hand the trash a snapshot left to the commit service,
/// then repair the package-manager state of every worktree the command
///  created.
/// Why:
///   The two are one after the other because neither waits for the other:
/// the hand-over is a record and a wake,
///  and the emptying it asks for runs in the service after this command has returned.
/// The repair's lines go to the
///  stream the command was given through its one lock,
/// as before,
/// and the hand-over's single line goes beside it.
/// A hand-over
///  that cannot reach the service is reported and never fails the add:
/// its record still names the trash,
/// so the
///  service empties it when it next runs for the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOver(after.trash); repairCreated(after);
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn finish_trash_and_repair(invocation: &Invocation, after: &AfterLock, stderr: &mut (dyn Write + Send)) {
    let stream: crate::shared_notice_stream::NoticeStream<'_> = std::sync::Mutex::new(stderr);
    if let Some(pending) = after.trash.as_ref() {
        // The hand-over is a step of its own: a record and a wake, timed beside the repair so the record of the add says what
        //  the service was told and how long that took.
        let hand_over_entered: bool = crate::add_timing::begin_step("handOverTrash");
        let mut notices = SharedNoticeStream { held: &stream, quiet: false };
        let told: Result<(), String> = crate::worktree_trash::hand_over_to_the_service(&invocation.context, pending, &mut notices);
        crate::add_timing::end_step(hand_over_entered);
        if let Err(reason) = told {
            crate::diagnostic_log::warn("worktreeCopyAfterLock", format!("monochromatic-git-policy-cli: {reason}").as_str());
        }
    }
    let mut findings = SharedNoticeStream { held: &stream, quiet: false };
    repair_created(invocation, after, &mut findings);
}

/// What:
///  The same two steps where no target of this build has a trash or a package repair.
/// Why:
///   The caller of this file is the same on every platform,
/// so the shape of the call is too,
/// and the work that this build cannot do is
///  nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finishTrashAndRepair = (invocation, after, stderr) => {};
/// ```
#[cfg(not(target_os = "linux"))]
pub(crate) fn finish_trash_and_repair(_invocation: &Invocation, _after: &AfterLock, _stderr: &mut (dyn Write + Send)) {
    return;
}

/// Trash-and-repair controls stay out of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "worktree_copy_after_lock_tests.rs"]
mod tests;
