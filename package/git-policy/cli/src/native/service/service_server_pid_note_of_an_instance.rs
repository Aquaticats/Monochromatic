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
//! (`process_identity.rs`:
//! the start tick from `/proc/<pid>/stat`,
//! field 22),
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

use crate::process_identity::{current_birth_identity, process_birth_identity};
use std::path::{Path, PathBuf};

/// The ending of the note that names an instance's process.
pub const PID_NOTE_ENDING: &str = ".pid";

/// One running commit service,
/// as its note names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerProcess {
    /// The process number the service had.
    pub pid: i32,
    /// The birth identity that proves the number still names that process.
    pub identity: String,
}

/// The note of one instance,
/// beside its socket.
pub fn path(socket_directory: &Path, instance: &str) -> PathBuf {
    return socket_directory.join(format!("{instance}{PID_NOTE_ENDING}"));
}

/// What:
///  Writes the note that names this process.
/// Why:
///   Only the service itself can say which process is its own,
/// and only its own birth identity is certainly its own.
/// The text is written beside the note and renamed onto it,
/// so a reader never sees half of a note,
/// and the temporary file carries this process's number,
/// so two processes writing at once never write into one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write(pidNotePath, `${process.pid} ${currentBirthIdentity()}\n`);
/// ```
pub fn write(note: &Path, pid: i32) -> std::io::Result<()> {
    let identity: String = match current_birth_identity() {
        Ok(identity) => identity,
        // A platform without an identity source cannot make a note a later command may trust, so none is made.
        Err(_unsupported) => return Ok(()),
    };
    let temporary: PathBuf = note.with_extension(format!("{pid}.pid.writing"));
    std::fs::write(&temporary, format!("{pid} {identity}\n"))?;
    return std::fs::rename(&temporary, note);
}

/// What:
///  Claims one instance for this process:
/// writes the note that names it,
/// unless another process that is still that process holds the instance.
/// Why:
///   The note is what a starter,
/// a second service and the sweep all read to learn whose an address is,
/// so it is written before the address is bound,
/// and a service that is alive keeps its instance from the moment it claims it,
/// whether or not it has greeted yet.
/// A note that names a process which is gone is the leftovers of a killed service,
/// and the birth identity is what tells the two apart,
/// so the claim is taken over then and never otherwise.
/// Two services claiming at the same moment both write and both read the note back:
/// the one that does not read its own writing has lost the instance and does not bind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claim(note, pid) // throws when another live process holds it
/// ```
pub fn claim(note: &Path, pid: i32) -> std::io::Result<()> {
    for _round in 0..2 {
        if let Some(held) = held_by_another_live_process(note) {
            return Err(std::io::Error::other(format!(
                "the commit service of this instance is process {} and is still that process, so this one does not claim it",
                held.pid
            )));
        }
        write(note, pid)?;
        // The note is believed only once it reads back as this process's: another claimer may have written it after this one.
        if matches!(read(note), Some(written) if written.pid == pid && is_still_the_process(&written)) {
            return Ok(());
        }
    }
    return Err(std::io::Error::other("another process claimed this instance at the same moment, so this one does not claim it"));
}

/// What:
///  The process that holds this instance and is still that process,
/// when it is not this one.
/// Why:
///   A note whose process is gone is a killed service's leftovers,
/// which the next service claims;
/// a note whose process is alive is the service of this instance,
/// and neither a second service nor a bind may take its address away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldByAnotherLiveProcess(note) // the note's server when its pid is not this process's and it is still that process
/// ```
/// Whether another process that is still that process holds the instance the note names,
/// for the one question a bind asks.
pub fn another_live_process_holds(note: &Path) -> bool {
    return held_by_another_live_process(note).is_some();
}

pub fn held_by_another_live_process(note: &Path) -> Option<ServerProcess> {
    let held: ServerProcess = read(note)?;
    if held.pid == i32::try_from(std::process::id()).unwrap_or(0) || !is_still_the_process(&held) {
        return None;
    }
    return Some(held);
}

/// The note as text,
/// or nothing when it is not there.
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
