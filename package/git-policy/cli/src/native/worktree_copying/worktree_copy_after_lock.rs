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
//! The code sits in a file of its own so
//!  `worktree_copy.rs` stays under the line limit,
//! and that file re-exports what its caller needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const after = { sourceRoot, created, trash };
//! await finishTrashAndRepair(after); // the disposal on a thread beside the repair
//! ```

use crate::add_timing::{begin_step, begin_work, end_step, end_work};
use crate::diagnostic_log::warn;
use crate::scoped_start::{Started, start_or_run_with};
use crate::side_by_side_threads::{ScopeBody, in_thread_scope};
use crate::transaction_git::GitContext;
use crate::worktree_copy::Invocation;
#[cfg(target_os = "linux")]
use crate::package_repair::{RepairEnvironment, RepairRequest, repair_worktree};
#[cfg(target_os = "linux")]
use crate::worktree_trash::{PendingTrash, dispose_pending_trash};
use std::io::Write;
use std::path::PathBuf;

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCopyAfterLock";

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
///  Empty the trash a snapshot left,
/// or hand it to the commit service,
/// timed as the work `disposeTrash` because
///  it runs on a thread of its own;
/// the reason is what the person is told when it cannot be emptied.
/// Why:
///   The work is timed per thread,
/// as every repair tool is,
/// so the record still says how long the emptying took
///  beside the step that waited for it.
/// The notices go to the process's standard error,
/// which is where the add's own notices go,
/// and `--quiet`
///  silences them as it does the rest.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function disposeTimed([git, pending]: [GitContext, PendingTrash]): string | undefined;
/// ```
#[cfg(target_os = "linux")]
fn dispose_timed((context, pending): (&GitContext, &PendingTrash)) -> Result<(), String> {
    let began: std::time::Instant = begin_work();
    let mut notices: Box<dyn Write + Send> = if pending.quiet { Box::new(std::io::sink()) } else { Box::new(std::io::stderr()) };
    let outcome: Result<(), String> = dispose_pending_trash(context, pending.destination.as_path(), pending.git_dir.as_path(), Some(&pending.manifest), &mut notices);
    end_work("disposeTrash", began);
    return outcome;
}

/// What:
///  Say what the disposal of a snapshot's trash left,
/// once,
/// with the reason it could not empty it.
/// Why:
///   The worktree is whole either way,
/// so a trash that stays is reported and never fails the add;
/// the work item
///  written for it means the commit service empties it later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportDisposal(outcome: ThreadResult<string | undefined>): void;
/// ```
#[cfg(target_os = "linux")]
fn report_disposal(outcome: std::thread::Result<Result<(), String>>) {
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(reason)) => warn(TAG, format!("monochromatic-git-policy-cli: {reason}").as_str()),
        // The work item the hand-over wrote means the service empties the trash when it next runs for that worktree; a control
        // that saw no record gets this sentence too, which names the worktree by the path it was made at.
        Err(_panic) => warn(TAG, "monochromatic-git-policy-cli: the directories left out of the new worktree were not emptied, because the disposal stopped unexpectedly; the trash stays inside the new worktree"),
    }
}

/// What:
///  The trash of a snapshot beside the package repair of the worktrees the command created,
/// with the stream the repair
///  writes to.
/// Why:
///   The two touch different directories and neither needs the other's result;
/// a `ScopeBody` names the borrows the
///  thread reads,
/// and the calling thread keeps the caller's writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TrashBesideRepair implements ScopeBody { run(scope) { ... } }
/// ```
#[cfg(target_os = "linux")]
pub(crate) struct TrashBesideRepair<'a> {
    /// The command whose worktree holds the trash and whose repair runs here.
    pub(crate) invocation: &'a Invocation,
    /// What the command created,
    /// with the trash to empty.
    pub(crate) after: &'a AfterLock,
    /// The trash to empty.
    pub(crate) pending: &'a PendingTrash,
    /// The stream the repair writes its findings to.
    pub(crate) stderr: &'a mut dyn Write,
    /// The job that empties the trash:
    /// `dispose_timed` in the shipped program,
    /// and a recording stand-in in a control.
    pub(crate) dispose: fn((&GitContext, &PendingTrash)) -> Result<(), String>,
}

#[cfg(target_os = "linux")]
impl<'env> ScopeBody<'env> for TrashBesideRepair<'env> {
    type Output = ();

    /// Start the disposal,
    /// repair the created worktrees here,
    /// then wait for the disposal,
    /// which is its own step:
    /// what it did
    ///  beside the repair is not time the person waited for.
    fn run<'scope>(self, scope: &'scope std::thread::Scope<'scope, 'env>) {
        let state: (&GitContext, &PendingTrash) = (&self.invocation.context, self.pending);
        let started: Started<'scope, Result<(), String>> = start_or_run_with(scope, state, self.dispose);
        let repair_entered: bool = begin_step("repair");
        repair_created(self.invocation, self.after, self.stderr);
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
///  alone when it left none.
/// Why:
///   The person waits for the slower of the two instead of for both,
/// and an add with nothing to repair still
///  empties its trash before the command ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pending ? await Promise.all([dispose(pending), repair(after)]) : repair(after);
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn finish_trash_and_repair(invocation: &Invocation, after: &AfterLock, stderr: &mut dyn Write) {
    let Some(pending) = after.trash.as_ref() else {
        let repair_entered: bool = begin_step("repair");
        repair_created(invocation, after, stderr);
        end_step(repair_entered);
        return;
    };
    in_thread_scope(TrashBesideRepair { invocation, after, pending, stderr, dispose: dispose_timed });
}
