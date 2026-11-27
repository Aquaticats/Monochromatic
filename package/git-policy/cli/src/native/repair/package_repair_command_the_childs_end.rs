//! What:
//!  The end of a started tool's child:
//! the wait that reads whether it came within the poll interval,
//! the seam a control fails that wait through,
//! and the reaping of a leader that a stop left behind.
//! Why:
//!  The run reads the child's end many times and each read must not wait longer than one
//! poll interval,
//! while the stop of a group cannot promise that a process in uninterruptible input waits for
//! nothing,
//! so the leader is reaped within a budget and handed to the runtime when the budget ends.
//! Keeping these apart from what a run asks a tool and what it says of it leaves one file that
//! reads a child's ending and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const waitStatus = (child) => Promise<ExitStatus | undefined>; // waits one poll interval at most
//! const reapWithin = (child, budgetMs) => Promise<boolean>; // false: a background task reaps it later
//! ```

/// Import a child owned until it is reaped,
/// and awaiting it on the runtime through its pidfd.
use crate::child_awaited_through_a_pidfd::{
    OwnedChild,
    adopt,
};
/// Import the runtime entry the late reaper of a stopped leader runs on.
use crate::lazy_runtime::spawn_on_runtime;
/// Import the captured output a stopped leader's reaper is handed.
use crate::output_capture::Capture;
/// Import the owned filesystem path a control's seam is named by.
#[cfg(test)]
use std::path::PathBuf;
/// Import the child process handle the standard library starts.
use std::process::Child;
/// Import the span of time a reaping budget is written as.
use std::time::Duration;
/// What:
///  Reap a stopped leader within the push stop grace;
/// whether it was reaped.
/// Why:
///   `run_tool` stops a leader the usual way,
/// and a named function stands where a closure that ignored the captured
///  output stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reapWithinStopGrace = (leader: Child, _captured: Capture) => reapWithin(leader, STOP_GRACE);
/// ```
pub(super) async fn reap_within_stop_grace(
    leader: Child,
    _captured: &Capture,
) -> bool {
    return reap_within(leader, crate::bounded_push::STOP_GRACE).await;
}

/// How often a running program is looked at,
/// which is the longest a single read of its end waits.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// The child's end if it comes within `POLL_INTERVAL`,
/// awaited on its pidfd,
/// and in a control an error once the file the seam
///  names exists.
pub(super) async fn wait_status(
    child: &mut OwnedChild
) -> std::io::Result<Option<std::process::ExitStatus>> {
    #[cfg(test)]
    if WAIT_FAILS_ONCE_THIS_FILE_EXISTS.with(|seam| {
        return seam.borrow().as_ref().is_some_and(|path| {
            return path.exists();
        });
    }) {
        return Err(std::io::Error::other("the wait failed"));
    }
    return match tokio::time::timeout(POLL_INTERVAL, child.wait()).await {
        Ok(ended) => ended.map(Some),
        Err(_not_yet) => Ok(None),
    };
}

#[cfg(test)]
thread_local! {
    /// A control makes the wait for the tool fail once the file it names exists,
    /// which the tool makes after it has written.
    pub(super) static WAIT_FAILS_ONCE_THIS_FILE_EXISTS: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Reap `child` within `budget`;
/// when it has not ended by then,
/// hand it to a task that reaps it whenever it does.
/// Why:
///   The stop is a signal,
/// and a process whose kernel call cannot be interrupted ends only when the call does.
/// The add returns
///  without it,
/// and the task keeps the leader from becoming a zombie for good.
/// True when the leader ended within the budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ended = await reapWithin(child, budgetMs); // false: a background task reaps it later
/// ```
pub(super) async fn reap_within(
    child: Child,
    budget: Duration,
) -> bool {
    let mut owned: OwnedChild = adopt(child);
    if let Ok(Ok(_status)) = tokio::time::timeout(budget, owned.wait()).await {
        return true;
    }
    // A runtime with no worker leaves the leader to the system's init when this process ends; the add goes on either way.
    if let Some(late) = owned.into_std_when_unreaped() {
        let _handed = spawn_on_runtime(async move {
            let mut reaper: OwnedChild = adopt(late);
            let _reaped = reaper.wait().await;
        });
    }
    return false;
}
