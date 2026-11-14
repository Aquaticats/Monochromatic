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
//! The note claims the instance before the address is bound,
//! so a service that is alive holds its instance whether or not it has greeted yet,
//! and a second service that finds a live note does not bind at all.
//! The files are removed as the service ends,
//! so a worktree whose service exited leaves neither a note nor an address to look at,
//! but only by identity:
//! a service removes the note and the address only while the note still names it,
//! so an instance that ended never deletes the address of the service that took the instance over
//! (the fourth review of issue 6293, finding 1:
//! an ending service removed its successor's address and note by path).
//! A service that was killed leaves both,
//! and the next service recognises them as stale by the note's birth identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const files = claimTheInstance(socketPath, instance); // mkdir 0700, claim pid note
//! files.keepTheAddressToThisAccount(); // chmod 0600, after the bind
//! files.remove(); // unlink socket and note only while the note still names this process
//! ```

use crate::service_server_pid_note_of_an_instance::{claim as claim_the_note, path as pid_note_path, read as read_note, remove as remove_pid_note};
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
    /// The note that names this instance's service,
    /// when the service was given an instance.
    pub fn note(&self) -> Option<&Path> {
        return self.pid_note.as_deref();
    }

    /// What:
    ///  Keeps the address to this account alone,
    ///  once it is bound.
    /// Why:
    ///   The socket carries a standards-review credential,
    /// so only this account may connect to it;
    /// the mode is set after the bind because the file exists only then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// chmod(socket, 0o600)
    /// ```
    pub fn keep_the_address_to_this_account(&self) -> std::io::Result<()> {
        let Some(socket) = self.socket.as_ref() else { return Ok(()) };
        return keep_only_its_owner(socket.as_path());
    }

    /// What:
    ///  Removes the address and the note,
    ///  which is what an instance that ended leaves,
    ///  but only while the note still names this process.
    /// Why:
    ///   A file that stayed would count the instance again in the survey of removed worktrees,
    /// and a person looking for a service that exited would find an address to it.
    /// The note is the identity of the service that owns the two files:
    /// when it names another process,
    /// that process's service took the instance over and its address is not this one's to delete
    /// (the fourth review of issue 6293, finding 1).
    /// A file that cannot be removed is noise for the next sweep and never a failure of the end.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const mine = readNote(note)?.pid === process.pid;
    /// if (!mine) return; // the address and the note belong to the service that took the instance over
    /// try { unlink(socket) } catch {} try { unlink(note) } catch {}
    /// ```
    pub fn remove(&self) {
        if !self.names_this_process() {
            crate::service_log!("the address and the note of this instance now name another process, so this service leaves them to it");
            return;
        }
        if let Some(socket) = self.socket.as_ref() {
            let _removed = std::fs::remove_file(socket);
        }
        if let Some(note) = self.pid_note.as_ref() {
            remove_pid_note(note.as_path());
        }
    }

    /// Whether the note still names this process.
    /// A note that is not there names no one,
    /// so nothing it stood beside is certainly this process's,
    /// and a file left behind is the next sweep's to remove rather than another service's to lose.
    fn names_this_process(&self) -> bool {
        let Some(note) = self.pid_note.as_ref() else {
            // Without an instance there is no note and no other service can have claimed the address.
            return true;
        };
        let ours: i32 = i32::try_from(std::process::id()).unwrap_or(0);
        return matches!(read_note(note.as_path()), Some(named) if named.pid == ours);
    }
}

/// What:
///  Makes the folder private and claims the instance for this process,
///  before the address is bound,
///  and gives the files the service owns.
/// Why:
///   The folder must exist before a bind and before a log beside it,
/// and the note must name this process before the address can be contended,
/// so a second service that finds a live note never binds and never takes the address over.
/// A note written only after the bind would leave the moment of the bind unnamed,
/// which is exactly when a second service looks (the fourth review of issue 6293, finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ensurePrivate(dirname(socket)); claimPidNote(pidNote); return { socket, pidNote };
/// ```
pub fn claim(socket: &Path, instance: &str) -> std::io::Result<OwnFiles> {
    let directory: &Path = socket.parent().unwrap_or_else(|| {
        return Path::new(".");
    });
    ensure_private(directory)?;
    let note: PathBuf = pid_note_path(directory, instance);
    claim_the_note(note.as_path(), i32::try_from(std::process::id()).unwrap_or(0))?;
    return Ok(OwnFiles { socket: Some(socket.to_path_buf()), pid_note: Some(note) });
}

/// Controls of the files stay out of the release executable.
#[cfg(test)]
#[path = "service_server_files_of_a_running_instance_tests.rs"]
mod tests;
