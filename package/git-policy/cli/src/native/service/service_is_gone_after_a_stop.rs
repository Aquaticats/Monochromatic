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
//! so the answer comes from two facts read together:
//! the note the service writes when it listens
//! (`service_server_pid_note_of_an_instance.rs`),
//! and the address itself.
//!  A note that names a living process is a service that is there.
//!  A note that is missing or names no living process is not proof on its own:
//! a service writes its note after it binds,
//! so a service that has bound the address and has not yet written the note is a live writer with no note,
//! and reading that absence as an end let a sweep remove a live journal
//! (the review of `567ae37198`, finding 1).
//!  So the address answers second: the instance is gone when nothing accepts at it,
//! which a bound socket does whether or not it will greet,
//! and a socket nothing holds refuses or is not there.
//!  Anything else the kernel answers with is a fact this reading cannot see,
//! and an answer it cannot see is not an end:
//! the caller keeps the journal, the note and the address for the next sweep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! serviceIsGone(socketDirectory, instance) // !stillTheProcess(note) && addressState(...) === 'nothing'
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
    if matches!(read_note(note.as_path()), Some(server) if is_still_the_process(&server)) {
        return false;
    }
    let runtime_dir: &Path = socket_directory.parent().unwrap_or(socket_directory);
    let address: crate::service_server::Bind = crate::service_address_of_a_worktree::socket_address_of(runtime_dir, instance);
    return !matches!(address_state(&address), AddressState::Serves | AddressState::CannotTell);
}

/// What:
///  What the kernel answers about one address.
/// Why:
///   "Nothing holds it" and "this command cannot see what holds it" are different facts,
/// and only the first lets a journal be removed
/// (the review of `567ae37198`, finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addressState(address) // 'serves' | 'nothing' | 'cannot-tell'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddressState {
    /// A process holds the address: the connection was made, or its queue is full and it took none.
    Serves,
    /// No process holds it: the socket is not there, or the kernel refused the connection.
    Nothing,
    /// The kernel answered with something that says neither.
    CannotTell,
}

/// What:
///  Which of the three answers this address gives now.
/// Why:
///   The refusal and the missing socket are the two forms "nothing is bound" takes,
/// and every other error is a fact this reading cannot use as an end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const error = tryConnect(address); error?.code === 'ECONNREFUSED' || error?.code === 'ENOENT' ? 'nothing' : 'cannot-tell'
/// ```
pub fn address_state(address: &crate::service_server::Bind) -> AddressState {
    return match crate::service_client_connect_within_a_bound::connect_within(address) {
        Ok(_stream) => AddressState::Serves,
        // A full queue is a process that holds the address and takes no more this instant.
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => AddressState::Serves,
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => AddressState::Nothing,
        // A path address with no socket file, and an abstract name nothing holds.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AddressState::Nothing,
        Err(_) => AddressState::CannotTell,
    };
}

/// Controls of the proof stay out of the release executable.
#[cfg(test)]
#[path = "service_is_gone_after_a_stop_tests.rs"]
mod tests;
