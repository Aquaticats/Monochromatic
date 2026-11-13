//! What:
//!  The note that says which process is a worktree's commit service:
//! its process number and the birth identity that proves the number still
//! names that process,
//! written by the service once it listens and removed as it ends.
//! Why:
//!   Without a service manager nothing else knows which process serves a worktree.
//! A socket that a killed service left behind refuses a connection and is replaced by
//! the next bind,
//! but a service that stopped inside its own process still holds its socket and answers nothing,
//! and the command that needs an answer has to be able to end it and start a fresh one.
//! A process number alone is not enough:
//! the kernel reuses numbers,
//! and killing a stranger because it inherited the number would be a defect nobody would find.
//! The birth identity is the repository's own answer to that
//! (`process_identity.rs`: the start tick from `/proc/<pid>/stat`, field 22),
//! and every lock in this program already records an owner this way.
//! The note is written after the socket is bound,
//! so a note that exists always means "this process had the socket",
//! and it is written through a temporary file and a rename,
//! so a reader never sees half of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeAtomic(join(instanceDir, `${instance}.pid`), `${process.pid} ${currentBirthIdentity()}\n`);
//! ```

use crate::process_identity::{ProcessIdentityError, current_birth_identity, process_birth_identity};
use std::path::{Path, PathBuf};

/// The ending of the note that names an instance's process.
pub const PID_NOTE_ENDING: &str = ".pid";

/// One running commit service, as its note names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerProcess {
    /// The process number the service had.
    pub pid: i32,
    /// The birth identity that proves the number still names that process.
    pub identity: String,
}

/// The note of one instance, beside its socket.
pub fn path(socket_directory: &Path, instance: &str) -> PathBuf {
    return socket_directory.join(format!("{instance}{PID_NOTE_ENDING}"));
}

/// What:
///  Writes the note that names this process.
/// Why:
///   Only the service itself can say which process bound the socket,
/// and only its own birth identity is certainly its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(pidNotePath, `${process.pid} ${currentBirthIdentity()}\n`);
/// ```
pub fn write(note: &Path, pid: i32) -> std::io::Result<()> {
    let Ok(text) = text(pid) else {
        // A platform without an identity source cannot make a note a later command may trust, so none is made.
        return Ok(());
    };
    let temporary: PathBuf = note.with_extension("pid.writing");
    std::fs::write(&temporary, text)?;
    return std::fs::rename(&temporary, note);
}

/// What:
///  The exact text a note for `pid` holds:
///  the number,
///  one space,
///  the process's birth identity,
///  and one line feed.
/// Why:
///   A service removes its note only while the file still holds this text,
/// so the text it will write is also the text it compares later
/// (`service_server_files_of_a_running_instance.rs`).
/// A comparison of the two strings is the proof that the file is that service's and not a later server's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteText(pid) // `${pid} linux:123456\n`
/// ```
pub fn text(pid: i32) -> Result<String, ProcessIdentityError> {
    let identity: String = current_birth_identity()?;
    return Ok(format!("{pid} {identity}\n"));
}

/// The note as text, or nothing when it is not there.
pub fn read(note: &Path) -> Option<ServerProcess> {
    return parse(std::fs::read_to_string(note).ok()?.as_str());
}

/// What:
///  The server a note's text names,
/// or nothing when the text is not a note this program wrote.
/// Why:
///   A file someone else put at that path must never be a reason to signal a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parse(text) // { pid, identity } or undefined
/// ```
pub fn parse(text: &str) -> Option<ServerProcess> {
    let (pid, identity) = text.trim_end().split_once(' ')?;
    return Some(ServerProcess { pid: pid.parse().ok()?, identity: identity.to_string() });
}

/// What:
///  Whether the process a note names is still that process.
/// Why:
///   A note outlives the service when the service was killed,
/// and acting on it then would signal whatever now holds the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stillTheProcess(note) // processBirthIdentity(note.pid) === note.identity
/// ```
pub fn is_still_the_process(server: &ServerProcess) -> bool {
    return matches!(process_birth_identity(i64::from(server.pid)), Ok(Some(identity)) if identity == server.identity);
}

/// Removes the note;
/// a note that is not there is already removed.
pub fn remove(note: &Path) {
    let _removed = std::fs::remove_file(note);
}

/// Controls of the note stay out of the release executable.
#[cfg(test)]
#[path = "service_server_pid_note_of_an_instance_tests.rs"]
mod tests;
