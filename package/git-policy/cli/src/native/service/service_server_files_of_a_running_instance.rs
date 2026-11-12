//! What:
//!  The files a commit service owns while it runs:
//!  the folder its address sits in,
//!  the address itself,
//!  and the note that names its process,
//!  made when it starts and removed when it ends.
//! Why:
//!   The wrapper starts the server itself, so nothing else creates the folder, sets the socket's mode,
//!  writes the note or removes them.
//! A service manager used to do all four through the unit.
//! The folder is 0700 and the socket 0600 because the socket carries a standards-review credential
//! (`service_socket_directory_only_this_account_may_open.rs`).
//! The note is written after the bind,
//! so a note that exists always means "this process had the address",
//! and removed as the service ends,
//! so a worktree whose service exited leaves neither a note nor an address to look at;
//! a service that was killed leaves both,
//! and the next command recognises them as stale by the note's birth identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const files = ownFiles(socketPath, instance); // mkdir 0700, chmod 0600, write pid note
//! files.remove(); // unlink socket, unlink note
//! ```

use crate::service_server_pid_note_of_an_instance::{path as pid_note_path, remove as remove_pid_note, write as write_pid_note};
use crate::service_socket_directory_only_this_account_may_open::{ensure_private, keep_only_its_owner};
use std::path::{Path, PathBuf};

/// The files one running service owns.
#[derive(Clone, Debug)]
pub struct OwnFiles {
    /// The address the service listens at,
    /// when it was given one.
    pub socket: Option<PathBuf>,
    /// The note that names the service's process,
    /// beside the address.
    pub pid_note: Option<PathBuf>,
}

impl OwnFiles {
    /// What:
    ///  Removes the address and the note,
    ///  which is what an instance that ended leaves.
    /// Why:
    ///   A file that stayed would count the instance again in the survey of removed worktrees,
    /// and a person looking for a service that exited would find an address to it.
    ///  A file that cannot be removed is noise for the next sweep and never a failure of the end.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { unlink(socket) } catch {} try { unlink(note) } catch {}
    /// ```
    pub fn remove(&self) {
        if let Some(socket) = self.socket.as_ref() {
            let _removed = std::fs::remove_file(socket);
        }
        if let Some(note) = self.pid_note.as_ref() {
            remove_pid_note(note.as_path());
        }
    }
}

/// What:
///  Makes the folder private,
///  writes the note once the address is bound,
///  and gives the files the service owns.
/// Why:
///   The folder must exist before a bind and before a log beside it,
/// and the note must be written only once the address answers,
/// so no command reads a note and finds nothing listening.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensurePrivate(dirname(socket)); keepOnlyItsOwner(socket); writePidNote(pidNote); return { socket, pidNote };
/// ```
pub fn take(socket: &Path, instance: Option<&str>) -> std::io::Result<OwnFiles> {
    let directory: &Path = socket.parent().unwrap_or_else(|| Path::new("."));
    ensure_private(directory)?;
    keep_only_its_owner(socket)?;
    let note: Option<PathBuf> = instance.map(|name| pid_note_path(directory, name));
    if let Some(path) = note.as_ref() {
        write_pid_note(path.as_path(), i32::try_from(std::process::id()).unwrap_or(0))?;
    }
    return Ok(OwnFiles { socket: Some(socket.to_path_buf()), pid_note: note });
}

/// Controls of the files stay out of the release executable.
#[cfg(test)]
#[path = "service_server_files_of_a_running_instance_tests.rs"]
mod tests;
