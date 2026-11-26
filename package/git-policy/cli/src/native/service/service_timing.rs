//! What:
//!  The lines the resident commit service writes to its log (standard error,
//! which the service manager keeps in the journal)
//!  about its own life and about the time each request took:
//! when it started,
//! one line for each request it answered with the
//!  phases of the wait,
//! a retire request,
//! and its exit with the reason and the uptime.
//! Why:
//!  After a slow commit the journal held `ready` and nothing else,
//! so nothing said whether the time went to the queue,
//! to
//!  another program's hold on the index,
//! to the landing or to a service being replaced.
//! These lines cost one clock read per
//!  phase edge and one line per request;
//! they name no person (the commit's message and identities never appear) and are not
//!  shown to the person at the terminal.
//!
//! The lines' parts,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `service_timing_records.rs`:
//!   what one request's timing is made of.
//! - `service_timing_wait_line.rs`:
//!   the line about the wait for the index lock,
//!   and the read of a step's duration from a line already written.
//! - `service_timing_answered_line.rs`:
//!   the one line each answered request gets.
//! - `service_timing_line_text.rs`:
//!   the prefix,
//!   the wall clock,
//!   and the text of one line with its own time.
//! - `service_timing_life_lines.rs`:
//!   the lines about the service's own life.
//! - `service_timing_log_entry.rs`:
//!   writing one entry,
//!   and the three macros every call site uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.error(`cli-git-service: answered request=${id} queued_ms=${queued} lock_wait_ms=${lockWait} total_ms=${total}`);
//! ```

/// The one line an answered request gets,
/// with every figure of its wait and its landing.
pub use crate::service_timing_answered_line::answered_line;
/// The lines about the service's own life:
/// its start,
/// a retire request,
/// the repository a line is about,
/// and its exit.
pub use crate::service_timing_life_lines::{
    ExitReason,
    exit_line,
    in_the_repository_at,
    repository_label,
    retire_line,
    started_line,
};
/// The prefix of every line,
/// the wall clock it states,
/// and the text of one line.
pub use crate::service_timing_line_text::{
    LOG_PREFIX,
    stamped_line,
    wall_millis,
};
/// What a control sees of the entries the code under test writes.
#[cfg(test)]
pub use crate::service_timing_log_entry::capture;
/// Writing one entry:
/// the sink it goes to,
/// whether this process is the resident service,
/// the release and instance it carries,
/// and the macros every call site uses.
pub use crate::service_timing_log_entry::{
    in_service,
    log,
    log_entry,
    set_context,
    sink,
};
/// What one request's timing is made of.
pub use crate::service_timing_records::{
    Answered,
    Arrival,
    LandTiming,
    ReadSplit,
};
/// The line about the wait for the index lock,
/// when it is written at all,
/// and the duration a line already written states for the index step.
pub use crate::service_timing_wait_line::{
    SLOW_LOCK_STEP,
    index_update_ms_of,
    lock_notes_fields,
    wait_line,
};

/// The lines,
/// their fields,
/// and the escaping.
#[cfg(test)]
#[path = "service_timing_tests.rs"]
mod tests;
