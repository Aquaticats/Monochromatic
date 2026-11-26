//! What:
//!  The lines a service writes about its own life:
//!  its start,
//!  a retire request,
//!  the words that name the repository a line is about,
//!  and its exit with the reason and the uptime.
//! Why:
//!   A journal read across every worktree of the machine must say which repository a line is about,
//!   and it must say so exactly:
//!   two worktrees can differ only in bytes that cannot be written as text,
//!   so a path that is text is written as it reads and one that is not is written in Rust's quoted form,
//!   which escapes each byte that is no text.
//!  Why the service ended is the last thing a reader needs:
//!  an idle exit,
//! a retire request,
//! a dead acceptor or a worktree that is gone.
//!  The time each line states for itself is `service_timing_line_text.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! exitLine(ExitReason.Idle, uptime) // 'exiting reason=idle uptime_ms=1800000'
//! ```
use std::path::Path;
use std::time::Duration;

/// The line a service writes as it starts,
/// before `ready`:
/// the release it announces,
/// the protocol,
/// its process and the worktree
///  instance it serves.
pub fn started_line(
    release: &str,
    protocol: u32,
    instance: Option<&str>,
) -> String {
    return format!(
        "started release={release:?} protocol={protocol} pid={} instance={}",
        std::process::id(),
        instance.unwrap_or("any")
    );
}

/// The line for a retire message:
/// a client of another release found this service and asked it to finish its batch and exit.
pub fn retire_line(release: &str) -> String {
    return format!(
        "retire requested by a client; this service is release {release:?} and exits after its batch"
    );
}

/// What:
///  The words a line uses to name the repository it is about:
///  `in the repository at <worktree root>`.
/// Why:
///   One service per worktree writes into a journal read across the whole machine,
///  and a control's fixture runs the same service:
///  a line that says only `batch 41` can be read as one about the repository its reader is in,
///  which is how the host run of 0.6.1-alpha.10 had a held fixture commit read as a failed commit on this repository's own `main`.
///  Every line about a batch,
/// a landing,
/// a push or a held commit names the repository it is about,
///  and names it beside the batch or the branch,
///  before anything about the outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// inTheRepositoryAt('/home/x/repo') // 'in the repository at /home/x/repo'
/// ```
pub fn in_the_repository_at(repository: &Path) -> String {
    return format!("in the repository at {}", repository_label(repository));
}

/// What:
///  The text a prose journal line names one repository by,
/// distinct for every path:
/// the path as it reads when it is text,
/// and Rust's quoted form when it is not,
/// which escapes each byte that is no text.
/// Why:
///  One journal covers every worktree of the machine,
/// and two worktrees can differ only in bytes that cannot be written as text:
///  the readable spelling of both is the same string with U+FFFD where the bytes were,
/// so a label built that way names neither,
/// and a line cannot be attributed to the repository that wrote it
/// (`path_bytes::path_message_text` keeps the readable spelling for prose,
///  and the exact bytes travel in the field beside it).
///  A line of fields writes the path with Rust's quoted form instead,
///  so a path that holds a space stays one field and reaches the journal whole
///  (`answered_line`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryLabel(path) // '/repo/worktree' | '"/repo/worktree-\\xFF"'
/// ```
pub fn repository_label(repository: &Path) -> String {
    return match repository.to_str() {
        Some(text) => text.to_string(),
        None => format!("{repository:?}"),
    };
}

/// Why a service ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
    /// Nothing to do for the whole idle period.
    Idle,
    /// A client asked it to retire.
    Retire,
    /// The queue its connections feed closed,
    /// which only happens when the acceptor thread died.
    Error,
    /// The worktree this service serves is gone,
    /// so it stopped its own socket instead of serving it.
    WorktreeGone,
}

impl ExitReason {
    /// The word written in the line.
    pub fn word(self) -> &'static str {
        return match self {
            ExitReason::Idle => "idle",
            ExitReason::Retire => "retire",
            ExitReason::Error => "error",
            ExitReason::WorktreeGone => "worktree-gone",
        };
    }
}

/// The line for the service's own exit,
/// with the time it ran since it began serving.
pub fn exit_line(
    reason: ExitReason,
    uptime: Duration,
) -> String {
    return format!(
        "exiting reason={} uptime_ms={}",
        reason.word(),
        uptime.as_millis()
    );
}
