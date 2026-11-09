//! What:
//!  What is left to do once the ignored state of a new worktree is in place:
//! its package-manager repair,
//! and the trash a
//!  snapshot left there,
//! emptied beside that repair instead of before it.
//! Why:
//!  A copy or a snapshot keeps links and build state exactly as the source had them,
//! so each tool's own offline
//!  repair makes them work at the new location;
//! the trash and the repair touch different directories and neither needs the
//!  other's result,
//! so they run side by side and the person waits for the slower of the two.
//! Both write to the stream the
//!  command was given,
//! through its one lock (`shared_notice_stream.rs`),
//! so the person reads the whole report on the
//!  stream they chose.
//! The disposal runs there instead of on a thread when the repair has nothing to do,
//! because a thread
//!  nothing runs beside buys nothing.
//! The code sits in a file of its own so
//!  `worktree_copy.rs` stays under the line limit,
//! and that file re-exports what its caller needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const after = { sourceRoot, created, trash };
//! await finishTrashAndRepair(after); // the disposal on the runtime beside the repair
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::call_catching_panics::call_catching_panics;
use crate::shared_notice_stream::{NoticeStream, SharedNoticeStream};
use crate::transaction_git::GitContext;
use crate::worktree_copy::Invocation;
#[cfg(target_os = "linux")]
use crate::package_repair::{RepairEnvironment, RepairRequest, repair_worktree, repair_worktree_awaiting};
#[cfg(target_os = "linux")]
use crate::worktree_copy_after_lock_disposal::{dispose_timed, dispose_timed_awaiting, nothing_to_repair, report_disposal};
#[cfg(target_os = "linux")]
use crate::worktree_trash::PendingTrash;
use std::io::Write;
use std::path::PathBuf;

/// The thread the last disposal ran on,
/// for the controls that tell a thread of its own from the calling thread.
#[cfg(all(test, target_os = "linux"))]
pub(crate) use crate::worktree_copy_after_lock_disposal::DISPOSED_ON;

/// What:
///  What is left to do once the ignored state is in place:
/// the source the ignored files came from and the worktrees the
///  command created.
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
    /// which is emptied beside the repair of that worktree.
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
///  `repair_created` on the process's runtime:
/// every tool is awaited rather than blocked on,
/// so the repair and the disposal beside it advance together.
/// Why:
///   The repair's tools are waits of a task of the runtime
/// (`repair_worktree_awaiting`),
/// and a task of the runtime cannot enter it again,
/// so a caller that is itself a task awaits this face.
/// The findings reach the same stream,
/// and a repair never fails the add either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const root of roots) await repairWorktreeAwaiting({ git, source, destination: root, switch, environment }, notices);
/// ```
#[cfg(target_os = "linux")]
async fn repair_created_awaiting(invocation: &Invocation, after: &AfterLock, notices: &mut dyn Write) {
    let Some(source) = after.source_root.as_deref() else { return };
    let environment: RepairEnvironment = RepairEnvironment::from_process();
    for root in &after.created {
        let request: RepairRequest<'_> = RepairRequest { git: &invocation.context, source, destination: root.as_path(), switch: invocation.repair, environment: &environment };
        repair_worktree_awaiting(&request, notices).await;
    }
    return;
}

/// What:
///  The trash of a snapshot beside the package repair of the worktrees the command created,
/// with the stream both of them
///  report to,
/// and the two as waits of one task of the runtime.
/// Why:
///   The two touch different directories and neither needs the other's result,
/// so both are driven by one task and the
///  person waits for the slower of the two.
/// Their storage work is done by jobs of the blocking pool
/// (`dispose_timed_awaiting`,
///  `repair_worktree_awaiting`),
/// so neither holds the task while it waits,
/// and the stream is one borrow both write through its one lock,
///  behind no second lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [disposed] = await Promise.all([disposeAwaiting(pending, stream), repairAwaiting(invocation, after, stream)]);
/// ```
#[cfg(target_os = "linux")]
pub(crate) async fn trash_beside_repair_awaiting(
    invocation: &Invocation,
    after: &AfterLock,
    pending: &PendingTrash,
    stream: &NoticeStream<'_>,
) {
    let state: (&GitContext, &PendingTrash, &NoticeStream<'_>) = (&invocation.context, pending, stream);
    let (disposed, ()): (Result<(), String>, ()) = tokio::join!(
        dispose_timed_awaiting(state),
        async {
            let repair_entered: bool = begin_step("repair");
            let mut findings = SharedNoticeStream { held: stream, quiet: false };
            repair_created_awaiting(invocation, after, &mut findings).await;
            end_step(repair_entered);
        },
    );
    // The wait for the disposal is its own step: what it did beside the repair is not time the person waited for.
    let wait_entered: bool = begin_step("disposeTrashWait");
    report_disposal(Ok(disposed));
    end_step(wait_entered);
}

/// What:
///  Empty the trash a snapshot left beside the package repair of the worktrees the command created,
/// or repair them
///  alone when it left none,
/// or empty the trash here when there is nothing to repair beside it.
/// Why:
///   The person waits for the slower of the two instead of for both,
/// and an add with nothing to repair still
///  empties its trash before the command ends.
/// Where nothing runs beside the disposal it is done here with no runtime and no wait;
/// otherwise the two are
///  waits of one task of the process's runtime,
/// which is entered once.
/// A disposal that fails is reported instead of failing the add,
/// and a disposal that panics is reported by the runtime that ran it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pending ? await Promise.all([dispose(pending), repair(after)]) : repair(after);
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn finish_trash_and_repair(invocation: &Invocation, after: &AfterLock, stderr: &mut (dyn Write + Send)) {
    let Some(pending) = after.trash.as_ref() else {
        let repair_entered: bool = begin_step("repair");
        repair_created(invocation, after, stderr);
        end_step(repair_entered);
        return;
    };
    // The writer is lent for this call's own span, so the lock and the writer it holds are one lifetime, and both waits
    //  name the same stream.
    let held: &mut (dyn Write + Send) = stderr;
    let stream: NoticeStream<'_> = std::sync::Mutex::new(held);
    // Something running beside the disposal is what a wait is for; with nothing there it runs here and starts nothing.
    if nothing_to_repair(invocation, after) {
        let wait_entered: bool = begin_step("disposeTrashWait");
        let outcome: std::thread::Result<Result<(), String>> = call_catching_panics((&invocation.context, pending, &stream), dispose_timed);
        end_step(wait_entered);
        report_disposal(outcome);
        return;
    }
    match crate::lazy_runtime::run_on_runtime(trash_beside_repair_awaiting(invocation, after, pending, &stream)) {
        Ok(()) => return,
        // No runtime could be built, so the two are done one after the other on this thread, as the scoped thread's work
        //  was done in turn when the system refused it.
        Err(_refusal) => {
            let wait_entered: bool = begin_step("disposeTrashWait");
            let outcome: std::thread::Result<Result<(), String>> = call_catching_panics((&invocation.context, pending, &stream), dispose_timed);
            end_step(wait_entered);
            report_disposal(outcome);
            let repair_entered: bool = begin_step("repair");
            let mut findings = SharedNoticeStream { held: &stream, quiet: false };
            repair_created(invocation, after, &mut findings);
            end_step(repair_entered);
        }
    }
}

/// Trash-beside-repair controls stay out of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "worktree_copy_after_lock_tests.rs"]
mod tests;
