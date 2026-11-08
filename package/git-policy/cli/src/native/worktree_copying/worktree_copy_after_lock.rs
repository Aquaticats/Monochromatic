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
//! await finishTrashAndRepair(after); // the disposal on a thread beside the repair
//! ```

use crate::add_timing::{begin_step, end_step};
use crate::call_catching_panics::call_catching_panics;
use crate::scoped_start::{Started, start_or_run_with};
use crate::shared_notice_stream::{NoticeStream, SharedNoticeStream};
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};
use crate::transaction_git::GitContext;
use crate::worktree_copy::Invocation;
#[cfg(target_os = "linux")]
use crate::package_repair::{RepairEnvironment, RepairRequest, repair_worktree};
#[cfg(target_os = "linux")]
use crate::worktree_copy_after_lock_disposal::{Disposal, dispose_timed, nothing_to_repair, report_disposal};
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
///  The trash of a snapshot beside the package repair of the worktrees the command created,
/// with the stream both of them
///  report to.
/// Why:
///   The two touch different directories and neither needs the other's result;
/// a `ScopeBody` names the borrows the
///  thread reads,
/// and the stream is one borrow both threads write through its one lock,
/// so both lines reach the
///  person's stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TrashBesideRepair implements ScopeBody { run(scope) { ... } }
/// ```
#[cfg(target_os = "linux")]
pub(crate) struct TrashBesideRepair<'a, 'stream> {
    /// The command whose worktree holds the trash and whose repair runs here.
    pub(crate) invocation: &'a Invocation,
    /// What the command created,
    /// with the trash to empty.
    pub(crate) after: &'a AfterLock,
    /// The trash to empty.
    pub(crate) pending: &'a PendingTrash,
    /// The stream the repair writes its findings to and the disposal its notices,
    /// behind its one lock.
    pub(crate) stream: &'a NoticeStream<'stream>,
    /// The job that empties the trash:
    /// `dispose_timed` in the shipped program,
    /// and a recording stand-in in a control.
    pub(crate) dispose: Disposal,
}

#[cfg(target_os = "linux")]
impl<'env, 'a: 'env, 'stream: 'env> ScopeBody<'env> for TrashBesideRepair<'a, 'stream> {
    type Output = ();

    /// Start the disposal,
    /// repair the created worktrees here,
    /// then wait for the disposal,
    /// which is its own step:
    /// what it did
    ///  beside the repair is not time the person waited for.
    fn run<'scope>(self, scope: &'scope std::thread::Scope<'scope, 'env>) {
        let state: (&GitContext, &PendingTrash, &NoticeStream<'stream>) = (&self.invocation.context, self.pending, self.stream);
        let started: Started<'scope, Result<(), String>> = start_or_run_with(scope, state, self.dispose);
        let repair_entered: bool = begin_step("repair");
        let mut findings = SharedNoticeStream { held: self.stream, quiet: false };
        repair_created(self.invocation, self.after, &mut findings);
        end_step(repair_entered);
        let wait_entered: bool = begin_step("disposeTrashWait");
        let outcome: std::thread::Result<Result<(), String>> = started.join();
        end_step(wait_entered);
        report_disposal(outcome);
    }
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
/// A disposal that runs here writes to the stream the command was given as it
///  happens,
/// and its panic is reported instead of failing the add,
/// as the thread's was.
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
    let held: &mut (dyn Write + Send) = stderr;
    let stream: NoticeStream<'_> = std::sync::Mutex::new(held);
    // A thread with nothing running beside it is a thread only waited for, so the disposal runs here instead.
    if nothing_to_repair(invocation, after) {
        let wait_entered: bool = begin_step("disposeTrashWait");
        let outcome: std::thread::Result<Result<(), String>> = call_catching_panics((&invocation.context, pending, &stream), dispose_timed);
        end_step(wait_entered);
        report_disposal(outcome);
        return;
    }
    in_thread_scope(TrashBesideRepair { invocation, after, pending, stream: &stream, dispose: dispose_timed });
}

/// Trash-beside-repair controls stay out of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "worktree_copy_after_lock_tests.rs"]
mod tests;
