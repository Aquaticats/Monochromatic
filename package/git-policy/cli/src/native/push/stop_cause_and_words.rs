//! What:
//!  The causes of a stop the wrapper gives a run,
//!  the report of one,
//!  which quiet window ended it,
//!  and the line a stalled push prints.
//! Why:
//!  `bounded_push.rs` runs the push and stops it,
//!  and this file holds what a stop means and says,
//!  so the runner stays inside the package's length limit
//!  (`rust/max-lines`).
//!  A stop names the window that fired because only one of the two is a setting:
//!  the stall window is `CLI_GIT_PUSH_DEADLINE_SECONDS`,
//!  and the connected window is fixed for every run,
//!  so a stop by the connected window must not offer that variable as its remedy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // stop-cause-and-words.ts: StopCause, StopReport, StopWindow, stalledReason, PUSH_QUEUED_LINE
//! ```

use crate::push_deadline_advice::extend_clause;
use crate::push_deadline_and_patience_settings::PUSH_DEADLINE_VARIABLE;
use std::time::Duration;

/// What a person is told when a push is ended for showing no progress:
/// the owner's answer of 2026-11-03,
/// "Bound it,
/// keep retrying
///  (Recommended)",
/// is that the service's retry plan keeps the push and tries it again,
/// and the line says so.
pub const PUSH_QUEUED_LINE: &str = "The push is queued: the commit service keeps it in its retry plan and tries it again.";

/// What a person is told about the remote when a push was stopped:
/// that the wrapper does not know,
/// and that the retry is what settles it.
///  A stopped push proves nothing about the remote:
/// the wrapper cannot know which object Git really sent,
/// which of a
///  remote's push URLs went first or which configuration shaped the attempt,
/// so the report says what is known and the retry
///  answers the question,
/// because a push of something the remote already has succeeds.
pub const STOPPED_DELIVERY_IS_NOT_KNOWN: &str = "the remote may or may not have taken the push, and only the next push settles which";

/// Exit code recorded for a push that this wrapper stopped,
///  the convention of `timeout(1)`.
pub const STOPPED_EXIT_CODE: i64 = 124;

/// What a stop reports:
/// which quiet window ended the run,
/// the limit that was hit,
/// what the push had moved and how long it had run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopReport {
    /// The window that was hit:
    /// the stall window of a tree with no open connection,
    /// or the connected window of one that holds a connection.
    pub window: StopWindow,
    /// The limit that was hit:
    /// the window the run was stopped by.
    pub limit: Duration,
    /// The bytes the push and everything it started had read and written.
    pub moved: u64,
    /// How long the push had run.
    pub elapsed: Duration,
}

/// Which quiet window a stopped run was ended by.
///  The two are told apart in the words of a stop because only one of them is a setting:
/// the stall window is `CLI_GIT_PUSH_DEADLINE_SECONDS`,
/// and the connected window is fixed for every run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StopWindow = 'stall' | 'connected';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopWindow {
    /// The run showed no sign of work for the stall window with no connection to a remote open.
    Stall,
    /// The run held a connection to the remote open and showed no sign of work for the connected window.
    Connected,
}

/// Why a push or query was stopped.
pub enum StopCause {
    /// The fixed deadline passed.
    Deadline,
    /// The push moved no data for the quiet window its report names.
    Stalled(StopReport),
    /// The output grew past the cap.
    Flood,
    /// The service was stopped,
    /// and its shutdown told every waiter to end its child.
    ServiceStopped,
}

/// What:
///  The line that ends a push the clock stopped as stalled,
/// in the words of the window that was hit,
/// naming the server it waited for when the run knew it.
/// Why:
///  A person reading a stop must be able to change the cause it names.
/// The stall window is
///  `CLI_GIT_PUSH_DEADLINE_SECONDS`,
/// and the connected window is fixed for every run,
/// so advice to set the variable after a connected stop
///  would name a limit that setting cannot move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `held its connection to ${server ? `the server \`${server}\`` : 'the remote'} open ... fixed at ${limit} seconds`
/// ```
pub(crate) fn stalled_reason(report: &StopReport, named: Option<&str>) -> String {
    // What the line calls the other end: the server the run named, or the remote a destination says nothing about.
    let other_end: String = match named {
        Some(host) => format!("the server `{host}`"),
        None => String::from("the remote"),
    };
    return match report.window {
        StopWindow::Stall => format!(
            "monochromatic-git-policy-cli: git push made no progress for {} seconds and was stopped (limit=stall, bytes_moved={}, \
             elapsed_seconds={}); {}. {PUSH_QUEUED_LINE} {STOPPED_DELIVERY_IS_NOT_KNOWN}. Hint: if you were pushing over a slow or \
             offline connection, your commit is already saved; push again when the connection works.",
            report.limit.as_secs(),
            report.moved,
            report.elapsed.as_secs(),
            extend_clause()
        ),
        StopWindow::Connected => format!(
            "monochromatic-git-policy-cli: git push held its connection to {other_end} open with nothing moving for {} seconds and was \
             stopped (limit=connected, bytes_moved={}, elapsed_seconds={}); {} does not change this window, which is fixed at {} \
             seconds. {PUSH_QUEUED_LINE} {STOPPED_DELIVERY_IS_NOT_KNOWN}. Hint: a remote that keeps the connection open for longer \
             than this stops every attempt, and the retry settles whether the push landed; your commit is already saved.",
            report.limit.as_secs(),
            report.moved,
            report.elapsed.as_secs(),
            PUSH_DEADLINE_VARIABLE,
            report.limit.as_secs()
        ),
    };
}
