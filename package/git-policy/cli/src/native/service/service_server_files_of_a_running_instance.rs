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

/// The files one running service owns,
/// and what proves each is still its own when it removes it.
#[derive(Clone, Debug)]
pub struct OwnFiles {
    /// The address the service listens at,
    /// when it was given one.
    pub socket: Option<PathBuf>,
    /// What:
    ///  The device and inode of the socket this service bound.
    /// Why:
    ///  A path can name another file by the time the service ends:
    ///  a second server that bound the same name,
    /// or a person's file put there.
    /// The inode is the object this process bound,
    /// and the kernel never hands one inode to two files.
    pub socket_identity: Option<(u64, u64)>,
    /// The note that names the service's process,
    /// beside the address.
    pub pid_note: Option<PathBuf>,
    /// What:
    ///  The exact text this service wrote into its note.
    /// Why:
    ///  A note is removed only while it still names this process;
    /// one that names another server is that server's to remove.
    pub note_text: Option<String>,
}

impl OwnFiles {
    /// What:
    ///  Removes the address and the note this service owns,
    ///  and leaves whatever a path names that is not its own.
    /// Why:
    ///   A file that stayed would count the instance again in the survey of removed worktrees,
    /// and a person looking for a service that exited would find an address to it.
    /// What must not happen is the review's finding 1:
    ///  a service that ends unlinking a second server's address and note by their paths,
    /// which left that server unreachable and the note missing,
    ///  so the proof that guards a journal's removal read true of a live service.
    /// So each removal is a comparison, not a path:
    /// the socket goes only while the path still names the inode this process bound,
    /// and the note only while it still holds the text this process wrote.
    /// A file that cannot be removed or compared is noise for the next sweep and never a failure of the end.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (inodeOf(socket) === mine) unlink(socket); if (readFile(note) === myText) unlink(note)
    /// ```
    pub fn remove(&self) {
        use std::os::unix::fs::MetadataExt;
        let address_was_ours: bool = match (self.socket.as_ref(), self.socket_identity) {
            (Some(socket), Some((device, inode))) => match std::fs::symlink_metadata(socket) {
                Ok(metadata) if metadata.dev() == device && metadata.ino() == inode => {
                    let _removed = std::fs::remove_file(socket);
                    true
                }
                _ => false,
            },
            _ => false,
        };
        // What: A claim file a killed command left goes once this service's own address is gone.
        // Why:  The address is unlinked first (`address_was_ours`), so no takeover can start now: whoever arrives binds the
        //       free address, and nothing claims anything. A file another process holds is left where it is
        //       (`service_address_takeover_claim.rs`, `remove_a_leftover_claim`).
        if address_was_ours
            && let Some(socket) = self.socket.as_ref()
        {
            let _no_leftover: bool = crate::service_address_takeover_claim::remove_a_leftover_claim(socket.as_path());
        }
        if let (Some(note), Some(text)) = (self.pid_note.as_ref(), self.note_text.as_ref())
            && matches!(std::fs::read_to_string(note), Ok(current) if current == *text)
        {
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
    use std::os::unix::fs::MetadataExt;
    let directory: &Path = socket.parent().unwrap_or_else(|| {
        return Path::new(".");
    });
    ensure_private(directory)?;
    keep_only_its_owner(socket)?;
    // What: The socket's identity is read from the file this process just bound.
    // Why:  It is the only proof available at the end that the path still names this service's own socket
    //       (`remove`).
    let identity: Option<(u64, u64)> = std::fs::symlink_metadata(socket).ok().map(|metadata| {
        return (metadata.dev(), metadata.ino());
    });
    let note: Option<PathBuf> = instance.map(|name| {
        return pid_note_path(directory, name);
    });
    let mut note_text: Option<String> = None;
    if let Some(path) = note.as_ref() {
        let pid: i32 = i32::try_from(std::process::id()).unwrap_or(0);
        write_pid_note(path.as_path(), pid)?;
        // A note that was not written (a platform without a birth identity) leaves nothing to compare, and nothing is removed.
        note_text = crate::service_server_pid_note_of_an_instance::text(pid).ok();
    }
    return Ok(OwnFiles { socket: Some(socket.to_path_buf()), socket_identity: identity, pid_note: note, note_text });
}

/// Controls of the files stay out of the release executable.
#[cfg(test)]
#[path = "service_server_files_of_a_running_instance_tests.rs"]
mod tests;
