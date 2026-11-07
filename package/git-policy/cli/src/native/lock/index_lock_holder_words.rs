//! What:
//!  The words about a lock's holder that the wait lines are built from:
//! the program of a live holder with its start time and the way to end it,
//! and the short summary of the processes a scan could not look at.
//! Why:
//!  The evidence file holds the proof and the verdict;
//! the sentences a person reads about the holder are kept apart from them so each file stays under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! holderWayOut({ pid, command }) // 'The holder is PID 5 (git), started 2026-11-07T18:04:20.000Z; ...'
//! ```

/// Process start times.
use crate::process_start::{ProcessStart, resolve_process_start};

/// What:
///  The program name of a process,
/// from its `comm`,
/// when it can be read.
/// Why:
///   A holder proven by Git's PID file has no command in the evidence,
/// and the person needs the program to know what to end.
///  Only the name is read,
/// never the arguments or the environment,
/// which can hold a token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(`/proc/${pid}/comm`, 'utf8').trim()
/// ```
fn program_of(pid: i64) -> Option<String> {
    let text: String = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let name: &str = text.trim();
    return if name.is_empty() { None } else { Some(name.to_string()) };
}

/// What:
///  The sentence that names a live holder by its number,
/// program and start time,
/// and the way out if it is stuck.
/// Why:
///   On 2026-11-07 four commits waited a combined hour for a lock held by a live `git commit` and were told only that the owner
///  "cannot be proven alive".
/// A person who reads PID,
/// program,
/// start time and the command that ends the process can decide in
///  seconds;
/// the wrapper itself never ends a process or removes a lock whose holder runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holderWayOut({ pid, command }): string; // 'The holder is PID 5 (git), started 2026-11-07T18:04:20.000Z; ...'
/// ```
pub fn holder_way_out(pid: i64, command: Option<&str>) -> String {
    let named: Option<String> = match command {
        Some(given) => Some(given.to_string()),
        None => program_of(pid),
    };
    let program: String = match named {
        Some(name) => name,
        None => String::from("program unknown"),
    };
    let started: String = match resolve_process_start(pid) {
        ProcessStart::Running { started_at_ms, .. } => format!("started {}", crate::iso_time::format_iso_milliseconds(started_at_ms as i64)),
        ProcessStart::Missing => String::from("start time unknown, it has just ended"),
        ProcessStart::Unknown(_) => String::from("start time unknown"),
    };
    return format!(
        "The holder is PID {pid} ({program}), {started}; if it is stuck, `kill {pid}` ends it and Git removes its lock, and a commit still waiting then lands.\n"
    );
}

/// Whether a reason says the scan ran out of time.
fn is_time_bound(reason: &&String) -> bool {
    return reason.contains("time bound");
}

/// What:
///  The reasons a scan could not see everything,
/// as one short text:
/// every reason when there are few,
/// otherwise their number,
/// the
///  first of them and the one that says the scan ran out of time.
/// Why:
///   On 2026-11-07 the line named every process the account could not look at (hundreds of rootless-container processes the
///  account's descriptors do not reach),
/// which buried the one fact a person needs:
/// the holder may be among them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// partialSummary(['PID 4: Permission denied', ...]) // '412 reasons, the first: PID 4: Permission denied'
/// ```
pub fn partial_summary(reasons: &[String]) -> String {
    const SHOWN_IN_FULL: usize = 3;
    if reasons.len() <= SHOWN_IN_FULL {
        return reasons.join("; ");
    }
    let mut text: String = format!("{} processes could not be looked at, the first of them: {}", reasons.len(), reasons[0]);
    if let Some(cut) = reasons.iter().find(is_time_bound) {
        text.push_str("; ");
        text.push_str(cut.as_str());
    }
    return text;
}
