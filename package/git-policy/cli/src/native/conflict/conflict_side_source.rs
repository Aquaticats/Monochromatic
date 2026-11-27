//! What:
//!  Copying a worktree file a conflict-side command is about to replace:
//!  its bytes,
//!  its mode and kind,
//!  a symbolic link's target,
//!  and the proof that the source held still while it was read.
//! Why:
//!  The copy is the owner's "save,
//! then allow":
//! whatever Git is about to overwrite must be on disk first.
//! A copy counts as saved only when the file it read is the file the name still holds and did
//! not change while it was read,
//! because a stale or torn copy would pass for the file Git then replaces.
//! Where the file is found is `conflict_side_source_finding_the_file.rs`,
//! and this file copies what it found.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! copyLocated(located, destination, () => {}); // throws when the file changed while it was copied
//! ```

/// A file that was found,
/// the failure the save reports,
/// and where to find another one.
pub use crate::conflict_side_source_finding_the_file::{
    FailureKind,
    Located,
    SaveFailure,
    locate,
};
/// The pieces of that finding this copy is built on.
use crate::conflict_side_source_finding_the_file::{
    TAG,
    changed,
    io_failure,
    open_parent,
};
/// The debug log that says what a copy refused.
use crate::diagnostic_log::debug;
/// The reads and identities below an open directory.
use crate::fs_beneath::{
    ObjectId,
    entry_metadata_at,
    identity_at,
    identity_of,
    open_file_at,
    read_link_at,
};
/// `File` is the held source,
/// the held directory and the copy being written.
use std::fs::File;
/// `Read`/`Write` are the byte streams a copy moves between them.
use std::io::{
    Read,
    Write,
};
/// `MetadataExt` reads the device,
/// inode,
/// times,
/// size and mode of a file's metadata.
use std::os::unix::fs::MetadataExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// What a file looked like at one moment:
/// which object,
/// how big,
/// when last written,
/// when last changed,
/// with what mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Stamp {
    /// Device and inode.
    object: (u64, u64),
    /// Size in bytes.
    size: u64,
    /// Modification time as seconds and nanoseconds.
    modified: (i64, i64),
    /// Status-change time as seconds and nanoseconds.
    changed: (i64, i64),
    /// File type and permission bits.
    mode: u32,
}

impl Stamp {
    /// The stamp of metadata read from an open descriptor or from a name.
    fn of(metadata: &std::fs::Metadata) -> Stamp {
        return Stamp {
            object: (metadata.dev(), metadata.ino()),
            size: metadata.size(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
            mode: metadata.mode(),
        };
    }
}

/// What:
///  Copy a symbolic link's target to `destination` and check that the link is the same one afterwards.
/// Why:
///   The link is read by name below the held directory;
/// a link replaced between the read and the end is a copy of the old one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLink(located: Located, destination: string): void;
/// ```
fn copy_link(
    located: &Located,
    before: &std::fs::Metadata,
    destination: &Path,
) -> Result<(), SaveFailure> {
    let target: PathBuf = match read_link_at(&located.parent, located.name.as_os_str()) {
        Ok(read) => read,
        Err(error) => {
            return Err(changed(
                located,
                format!("could not be read as a link ({error})").as_str(),
            ));
        }
    };
    if let Err(error) = std::os::unix::fs::symlink(&target, destination) {
        return Err(io_failure(
            &String::from_utf8_lossy(located.relative.as_slice()),
            &error,
        ));
    }
    if !link_unchanged(located, before, &target) {
        let _removed = std::fs::remove_file(destination);
        return Err(changed(
            located,
            "was replaced while its link target was copied",
        ));
    }
    return Ok(());
}

/// Whether the entry still has the stamp it had and still points at `target`.
fn link_unchanged(
    located: &Located,
    before: &std::fs::Metadata,
    target: &Path,
) -> bool {
    let Ok(now) = entry_metadata_at(&located.parent, located.name.as_os_str()) else {
        return false;
    };
    if Stamp::of(&now) != Stamp::of(before) {
        return false;
    }
    let Ok(read) = read_link_at(&located.parent, located.name.as_os_str()) else {
        return false;
    };
    return read == target;
}

/// A failure inside the copy:
/// the system's own,
/// or one that already says what happened.
enum CopyError {
    /// A read,
    /// a write or a sync the system refused.
    System(std::io::Error),
    /// A failure with its subject,
    /// reason and advice.
    Failed(SaveFailure),
}

impl From<std::io::Error> for CopyError {
    fn from(error: std::io::Error) -> CopyError {
        return CopyError::System(error);
    }
}

impl From<SaveFailure> for CopyError {
    fn from(failure: SaveFailure) -> CopyError {
        return CopyError::Failed(failure);
    }
}

/// What:
///  Read `from` up to the size it had when it was opened into `to`,
/// then prove the source held still.
/// Why:
///   The stamp of the open descriptor must be the one taken before the first read,
/// the byte count must equal the size,
/// and
///  the name must still lead,
/// without a link,
/// to the object that was read;
/// `between` runs after each chunk.
///  The read stops at that size,
///  so a writer that keeps appending cannot hold the copy open for good:
///  the extra bytes are never read,
///  and the stamp comparison after the read rejects the changed source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyStream(located: Located, from: File, to: File, before: Stamp, between: () => void): void;
/// ```
fn copy_stream(
    located: &Located,
    from: &mut File,
    to: &mut File,
    before: Stamp,
    between: &mut dyn FnMut(),
) -> Result<(), CopyError> {
    let mut buffer: Vec<u8> = vec![0; 1 << 16];
    let mut copied: u64 = 0;
    while copied < before.size {
        let left: u64 = before.size - copied;
        let wanted: usize = usize::try_from(left)
            .unwrap_or(usize::MAX)
            .min(buffer.len());
        let count: usize = from.read(&mut buffer[..wanted])?;
        if count == 0 {
            break;
        }
        to.write_all(&buffer[..count])?;
        copied += u64::try_from(count).unwrap_or(u64::MAX);
        between();
    }
    let after: Stamp = Stamp::of(&from.metadata()?);
    if after != before {
        debug(
            TAG,
            &format!(
                "{}: stamp before the copy {before:?}, after {after:?}",
                String::from_utf8_lossy(located.relative.as_slice())
            ),
        );
        return Err(changed(located, "changed while it was copied").into());
    }
    if copied != before.size {
        return Err(changed(
            located,
            format!("gave {copied} bytes where it had {}", before.size).as_str(),
        )
        .into());
    }
    // The name must still lead, below the same directories and without a link, to the object that was read.
    let named_now: Option<ObjectId> =
        match open_parent(located.root.as_path(), located.relative.as_slice())? {
            Some((parent, name)) => identity_at(&parent, name.as_os_str()).ok(),
            None => None,
        };
    if named_now != identity_of(from).ok() {
        return Err(changed(located, "no longer holds the file that was copied").into());
    }
    to.set_permissions(
        <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(before.mode & 0o777),
    )?;
    to.sync_all()?;
    return Ok(());
}

/// What:
///  Open the source and the destination of a regular file,
/// copy it,
/// and remove the destination on any failure.
/// Why:
///   A cut-short or stale file named like a copy would pass for the whole file;
/// the command is rejected,
/// so none is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyEntry(located: Located, metadata: Metadata, destination: string, between: () => void): void;
/// ```
fn copy_entry(
    located: &Located,
    metadata: &std::fs::Metadata,
    destination: &Path,
    between: &mut dyn FnMut(),
) -> Result<(), CopyError> {
    let shown: String = String::from_utf8_lossy(located.relative.as_slice()).into_owned();
    let mut from: File = open_file_at(&located.parent, located.name.as_os_str())?;
    let opened: std::fs::Metadata = from.metadata()?;
    if !opened.file_type().is_file() {
        return Err(SaveFailure::io(
            shown,
            String::from("it is neither a file nor a symbolic link"),
        )
        .into());
    }
    let before: Stamp = Stamp::of(&opened);
    if before != Stamp::of(metadata) {
        return Err(changed(located, "changed between the look at its name and the open").into());
    }
    let mut to: File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode_600()
        .open(destination)?;
    let streamed: Result<(), CopyError> = copy_stream(located, &mut from, &mut to, before, between);
    if streamed.is_err() {
        let _removed = std::fs::remove_file(destination);
    }
    return streamed;
}

/// What:
///  Copy one file's content,
/// mode and kind to `destination`,
/// which must not exist,
/// sync it,
/// and prove the source held still.
/// Why:
///   `between` runs after each chunk is written,
/// so a control can change the source mid-copy.
/// The stamp of the open
///  descriptor is taken before the first read and after the last,
/// the byte count must equal the size,
/// and the name must
///  still lead,
/// without a link,
/// to the object that was read;
/// any difference removes the copy and fails,
/// because a stale or
///  torn copy would pass for the file Git then replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLocated(located: Located, destination: string, between: () => void): void;
/// ```
pub fn copy_located(
    located: &Located,
    destination: &Path,
    between: &mut dyn FnMut(),
) -> Result<(), SaveFailure> {
    let shown: String = String::from_utf8_lossy(located.relative.as_slice()).into_owned();
    let metadata: std::fs::Metadata =
        match entry_metadata_at(&located.parent, located.name.as_os_str()) {
            Ok(found) => found,
            Err(error) => return Err(io_failure(&shown, &error)),
        };
    if metadata.file_type().is_symlink() {
        return copy_link(located, &metadata, destination);
    }
    if !metadata.file_type().is_file() {
        return Err(SaveFailure::io(
            shown,
            String::from("it is neither a file nor a symbolic link"),
        ));
    }
    return match copy_entry(located, &metadata, destination, between) {
        Ok(()) => Ok(()),
        Err(CopyError::System(error)) => Err(io_failure(&shown, &error)),
        Err(CopyError::Failed(failure)) => Err(failure),
    };
}

/// `OpenOptions` with the private creation mode,
/// named for what the call does.
trait PrivateCreation {
    /// The options with creation mode `0600`.
    fn mode_600(&mut self) -> &mut Self;
}

impl PrivateCreation for std::fs::OpenOptions {
    fn mode_600(&mut self) -> &mut Self {
        return std::os::unix::fs::OpenOptionsExt::mode(self, 0o600);
    }
}

/// Where a file is found and how it is copied,
/// including a file that changes while it is read.
#[cfg(test)]
#[path = "conflict_side_source_tests.rs"]
mod tests;
