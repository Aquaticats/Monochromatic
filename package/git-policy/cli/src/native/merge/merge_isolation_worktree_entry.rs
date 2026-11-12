//! What:
//!  What stands at one worktree path,
//! read once for the two answers a merge needs of it:
//!  the bytes a capture holds,
//!  and the entry a gate must stop the merge for.
//! Why:
//!   The capture and the read of the marks that hide a path from status need different answers about a name that holds no bytes,
//!  so one read says what stands there and each caller decides,
//!  and neither ever opens a named pipe to find out
//! (`crate::plain_file`,
//!  owner's rule "nothing should ever wait forever or indefinitely").
//!  This lives beside the capture it serves,
//! in a file of its own because the capture's file is at the package's line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entry = worktreeEntryAt(location); // { kind: 'holds'; bytes; mode } | { kind: 'absent' } | { kind: 'no-bytes'; kind }
//! ```

/// The words for a failed read,
/// and the mode Git records,
/// both owned by the capture.
use crate::merge_isolation_carry::{could_not_be_read, mode_of};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What stands at one worktree path,
/// as a merge reads it.
/// Why:
///   Two readers need different answers about a name that holds no bytes:
/// the capture cannot carry a directory,
///  a named pipe,
/// a socket or a device and refuses one,
/// while the read of the marks that hide a path from status must
///  leave such an entry to the gates that stop a merge for it,
/// and neither may wait on a pipe to learn its type.
///  So one read answers what stands there,
/// and each caller decides what that means for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeEntry = { kind: 'holds'; bytes: Uint8Array; mode: string } | { kind: 'absent' } | { kind: 'no-bytes'; kind: NoBytes };
/// ```
pub(crate) enum WorktreeEntry {
    /// A plain file's content,
    /// or a link's target text,
    /// and the mode Git records for it.
    Holds {
        /// The bytes,
        /// exact and never through a filter.
        bytes: Vec<u8>,
        /// The Git mode of the entry the bytes came from.
        mode: &'static str,
    },
    /// Nothing stands at the path.
    Absent,
    /// A name that holds no bytes a merge can carry.
    NoBytes(NoBytes),
}

/// What:
///  The kinds of worktree entry that hold no bytes a merge can carry.
/// Why:
///   The merges of this package answer a directory and a named pipe with two different gates,
/// so the read
///  names which of the two it met,
/// and neither is left for the other to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NoBytes = 'directory' | 'special';
/// ```
pub(crate) enum NoBytes {
    /// A directory,
    /// which the merge stops for when it holds anything the merge does not delete.
    Directory,
    /// A named pipe,
    /// a socket or a device.
    Special,
}

/// What:
///  The words for a capture that met an entry it cannot hold.
/// Why:
///   The person reads which path it is and what stands there,
/// and a capture is never the place to guess
///  which files of a directory are theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notOneEdit = (location, kind) => `${location} is ...`;
/// ```
pub(crate) fn not_one_edit_a_merge_can_carry(location: &Path, kind: &NoBytes) -> String {
    return match kind {
        NoBytes::Directory => format!("{} is a directory, which is not one edit a merge can carry", location.display()),
        NoBytes::Special => {
            format!("{} is not a plain file (a named pipe, a socket or a device), which is not one edit a merge can carry", location.display())
        }
    };
}

/// What:
///  What the worktree holds at one path,
/// told from the entry's type before anything is opened.
/// Why:
///   Opening a named pipe for reading waits for a writer that may never come,
/// so the type decides first,
///  and the bytes of a plain file are read from a handle that can only be a plain file
/// (`crate::plain_file`,
///  owner's rule "nothing should ever wait forever or indefinitely").
///  A name swapped for a pipe between the type check and the open is refused by the handle.
///  A read that fails is not an answer about the entry,
/// so it stays an error the caller words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeEntryAt(location: string): { kind: 'holds'; bytes; mode } | { kind: 'absent' } | { kind: 'no-bytes'; kind };
/// ```
pub(crate) fn worktree_entry_at(location: &Path) -> Result<WorktreeEntry, String> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(location) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(WorktreeEntry::Absent),
        Err(error) => return Err(could_not_be_read(location, error)),
    };
    if metadata.is_dir() {
        return Ok(WorktreeEntry::NoBytes(NoBytes::Directory));
    }
    let mode: &'static str = mode_of(&metadata);
    if metadata.file_type().is_symlink() {
        let target: std::path::PathBuf = match std::fs::read_link(location) {
            Ok(found) => found,
            Err(error) => return Err(could_not_be_read(location, error)),
        };
        return Ok(WorktreeEntry::Holds { bytes: target.as_os_str().as_encoded_bytes().to_vec(), mode });
    }
    // The type is settled before the open and again on the handle the open returns, so a name swapped for a pipe or a device
    //  between the two is refused instead of waited on.
    if !metadata.file_type().is_file() {
        return Ok(WorktreeEntry::NoBytes(NoBytes::Special));
    }
    let mut plain: std::fs::File = match crate::plain_file::open_plain_file(location) {
        Ok(found) => found,
        Err(error) => return Err(could_not_be_read(location, error)),
    };
    let mut bytes: Vec<u8> = Vec::new();
    if let Err(error) = std::io::Read::read_to_end(&mut plain, &mut bytes) {
        return Err(could_not_be_read(location, error));
    }
    return Ok(WorktreeEntry::Holds { bytes, mode });
}
