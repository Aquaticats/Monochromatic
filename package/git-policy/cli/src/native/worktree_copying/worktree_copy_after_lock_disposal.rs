//! What:
//!  The job that empties the trash a snapshot of the main checkout left in a new worktree,
//! and the step that runs
//!  it beside the package repair,
//! or here when the repair has nothing to do.
//! Why:
//!  A trash is either handed to the commit service or emptied by this command,
//! and the notices of either way
//!  belong on the stream the command was given,
//! which the disposal reaches through the one lock it shares with the
//!  repair (`shared_notice_stream.rs`).
//! What the emptying left is a warning and never the add's failure:
//!  the worktree is whole either way,
//! and the work item written for the trash means the service empties it later.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await disposeTimed([git, pending, stream]);
//! reportDisposal(outcome);
//! ```

use crate::add_timing::{begin_work, end_work};
use crate::diagnostic_log::warn;
use crate::shared_notice_stream::{NoticeStream, SharedNoticeStream};
use crate::transaction_git::GitContext;
use crate::worktree_copy::Invocation;
use crate::worktree_copy_after_lock::AfterLock;
use crate::worktree_trash::{PendingTrash, dispose_pending_trash, dispose_pending_trash_awaiting};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeCopyAfterLock";

/// What:
///  Where the last disposal ran,
/// so a control can tell a wait of the runtime from a thread of its own.
/// Why:
///  Whether the disposal runs on a thread of its own is not visible in what it does,
/// only in where it runs,
///  and a timing threshold would say nothing on a busy host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhereTheDisposalRan = { thread: ThreadId; insideTheRuntime: boolean };
/// ```
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct WhereTheDisposalRan {
    /// The thread the disposal's own frame ran on.
    pub(crate) thread: std::thread::ThreadId,
    /// Whether it ran inside the process's runtime.
    pub(crate) inside_the_runtime: bool,
}

/// The place the last disposal ran,
/// for the controls that tell a wait of the runtime from a thread of its own.
#[cfg(test)]
pub(crate) static DISPOSED_ON: std::sync::Mutex<Option<WhereTheDisposalRan>> = std::sync::Mutex::new(None);

/// Note where this disposal runs,
/// for the controls of the lifecycle;
/// nothing outside a control reads it.
fn note_where_the_disposal_runs() {
    #[cfg(test)]
    if let Ok(mut last) = DISPOSED_ON.lock() {
        *last = Some(WhereTheDisposalRan { thread: std::thread::current().id(), inside_the_runtime: tokio::runtime::Handle::try_current().is_ok() });
    }
}

/// What:
///  Empty the trash a snapshot left,
/// or hand it to the commit service,
/// timed as the work `disposeTrash` because
///  it is work of its own;
/// the reason is what the person is told when it cannot be emptied.
/// Why:
///   The work is timed per disposal,
/// as every repair tool is,
/// so the record still says how long the emptying took
///  beside the step that waited for it.
/// The notices go to the stream the command was given,
/// which is where the add's own notices go,
/// and `--quiet`
///  silences them as it does the rest.
/// The callers of this face are the ones with no runtime to wait on:
/// an add that has nothing to repair,
///  and an add whose runtime could not be built;
/// a caller that is itself a task of the runtime awaits `dispose_timed_awaiting`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function disposeTimed([git, pending, stream]: [GitContext, PendingTrash, NoticeStream]): string | undefined;
/// ```
pub(crate) fn dispose_timed((context, pending, stream): (&GitContext, &PendingTrash, &NoticeStream<'_>)) -> Result<(), String> {
    let began: std::time::Instant = begin_work();
    note_where_the_disposal_runs();
    let mut notices = SharedNoticeStream { held: stream, quiet: pending.quiet };
    let outcome: Result<(), String> = dispose_pending_trash(context, pending.destination.as_path(), pending.git_dir.as_path(), Some(&pending.manifest), &mut notices);
    end_work("disposeTrash", began);
    return outcome;
}

/// What:
///  `dispose_timed` on the process's runtime:
/// the emptying is awaited,
/// so a caller that is itself a task of the runtime holds no storage wait.
/// Why:
///   The add empties a snapshot's trash beside the package repair,
/// and the two are waits of one task of the runtime
/// (`worktree_copy_after_lock.rs`);
/// the emptying's lookups and removals are jobs of the blocking pool,
///  so the repair's own waits keep running while it works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function disposeTimedAwaiting([git, pending, stream]): Promise<string | undefined>;
/// ```
pub(crate) async fn dispose_timed_awaiting((context, pending, stream): (&GitContext, &PendingTrash, &NoticeStream<'_>)) -> Result<(), String> {
    let began: std::time::Instant = begin_work();
    note_where_the_disposal_runs();
    let outcome: Result<(), String> = {
        let mut notices = SharedNoticeStream { held: stream, quiet: pending.quiet };
        dispose_pending_trash_awaiting(context, pending.destination.as_path(), pending.git_dir.as_path(), Some(&pending.manifest), &mut notices).await
    };
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
///  written for it means the commit service tries again later,
/// and so does the next `git worktree add` for that
///  worktree,
/// which also names a trash that no record accounts for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportDisposal(outcome: ThreadResult<string | undefined>): void;
/// ```
pub(crate) fn report_disposal(outcome: std::thread::Result<Result<(), String>>) {
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(reason)) => warn(TAG, format!("monochromatic-git-policy-cli: {reason}").as_str()),
        // The work item the hand-over wrote means the service empties the trash when it next runs for that worktree; a control
        // that saw no record gets this sentence too, which names the worktree by the path it was made at.
        Err(_panic) => warn(TAG, "monochromatic-git-policy-cli: the directories left out of the new worktree were not emptied, because the disposal stopped unexpectedly; the trash stays inside the new worktree"),
    }
}

/// What:
///  Whether the package repair of the worktrees the command created has anything to do.
/// Why:
///   The repair is the work the disposal runs beside;
/// with nothing to repair there is nothing to overlap,
/// so a
///  thread would empty the trash and be waited for with nothing done in the meantime.
/// A bare repository copies nothing and
///  so has no source to repair from,
/// a copy that failed leaves no whole worktree to repair,
/// and `--no-checkout` asked for
///  no checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// after.sourceRoot === undefined || after.created.length === 0 || switch.noCheckout
/// ```
pub(crate) fn nothing_to_repair(invocation: &Invocation, after: &AfterLock) -> bool {
    return after.source_root.is_none() || after.created.is_empty() || invocation.repair.no_checkout;
}
