//! What:
//!  Whether a stopped commit service is really gone,
//! which is what lets the journal it wrote be removed.
//! Why:
//!   A service that was asked to retire answers before it exits,
//! and a service that was killed ends without answering at all,
//! so the answer to a stop is not the answer that nothing writes any more.
//! The sweep of removed worktrees and the removal a person runs both ask this before they
//! remove a journal,
//! and a journal whose service may still run keeps the note and the socket file that name
//! its instance.
//! Without a service manager there is nobody to ask,
//! so the answer comes from the note the service writes when it listens
//! (`service_server_pid_note_of_an_instance.rs`):
//! the instance's service is gone when the note names no process,
//! or names a process number that no longer carries that birth identity.
//! A note that is not there says the same:
//! a service writes it before it accepts anything,
//! so a service that runs always has one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! serviceIsGone(socketDirectory, instance) // !stillTheProcess(readPidNote(instance))
//! ```

use crate::service_server_pid_note_of_an_instance::{is_still_the_process, path as pid_note, read as read_note};
use std::path::Path;

/// What:
///  Whether no process serves this instance any more.
/// Why:
///   The note is the only record of which process served it,
/// and the birth identity is what keeps a reused number from being read as it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceIsGone(dir, instance) // !readNote(instance) || !stillTheProcess(note)
/// ```
pub fn service_is_gone(socket_directory: &Path, instance: &str) -> bool {
    let note: std::path::PathBuf = pid_note(socket_directory, instance);
    return match read_note(note.as_path()) {
        Some(server) => !is_still_the_process(&server),
        None => true,
    };
}

/// Controls of the proof stay out of the release executable.
#[cfg(test)]
#[path = "service_is_gone_after_a_stop_tests.rs"]
mod tests;
