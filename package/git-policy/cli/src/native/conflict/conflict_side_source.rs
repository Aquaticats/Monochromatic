//! What:
//!  Finding the worktree file a conflict-side command is about to replace,
//! and copying it.
//! Why:
//!  The copy is the owner's "save,
//! then allow":
//! whatever Git is about to overwrite must be on disk first.
//! Git writes
//!  `dir/file` by replacing whatever stands at `dir` (a symbolic link or a file) with a real directory,
//! and by replacing the
//!  file as it is when Git runs.
//! So the walk to the file never follows a link,
//! a link or file in front of the name is a
//!  failure that names it,
//! and a copy counts as saved only when the file it read is the file the name still holds and did
//!  not change while it was read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const located = locate(root, 'dir/a.rs'); // undefined when nothing is there to save; throws below a link
//! if (located) copyLocated(located, destination, () => {}); // throws when the file changed while it was copied
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{ObjectId, entry_metadata_at, identity_at, identity_of, open_directory_at, open_file_at, read_link_at};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "conflictSideSource";

/// Which advice goes with a failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureKind {
    /// The system refused a read,
    /// a write or a place:
    /// free the space or fix the permissions.
    Io,
    /// A link or a file stands where a directory in front of the name should be,
    /// and Git would replace it unsaved.
    Ancestor,
    /// The file changed while it was copied,
    /// so the copy could be stale or cut short.
    Changed,
}

/// Why a copy could not be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFailure {
    /// The repository-relative path being copied,
    /// the directory in front of it that is in the way,
    /// or the place the copies go
    ///  when none was reached.
    pub subject: String,
    /// The operating system's words,
    /// or what the subject is.
    pub reason: String,
    /// Which advice fits.
    pub kind: FailureKind,
}

impl SaveFailure {
    /// A failure of the system on `subject`,
    /// logged.
    pub fn io(subject: String, reason: String) -> SaveFailure {
        debug(TAG, &format!("fail closed: {subject}: {reason}"));
        return SaveFailure { subject, reason, kind: FailureKind::Io };
    }

    /// A failure of kind `kind` on `subject`,
    /// logged.
    fn of_kind(kind: FailureKind, subject: String, reason: String) -> SaveFailure {
        debug(TAG, &format!("fail closed ({kind:?}): {subject}: {reason}"));
        return SaveFailure { subject, reason, kind };
    }
}

/// A failure of the system on `subject`,
/// logged.
fn io_failure(subject: &str, error: &std::io::Error) -> SaveFailure {
    return SaveFailure::io(String::from(subject), error.to_string());
}

/// The names joined with `/`,
/// as the path they make is written.
fn joined(names: &[OsString]) -> String {
    let mut text: String = String::new();
    for (position, name) in names.iter().enumerate() {
        if position > 0 {
            text.push('/');
        }
        text.push_str(&name.to_string_lossy());
    }
    return text;
}

/// A worktree entry that exists and is to be copied.
#[derive(Debug)]
pub struct Located {
    /// The top of the worktree,
    /// which is trusted:
    /// the reader returns its real path.
    root: PathBuf,
    /// The path below the root,
    /// as typed.
    relative: Vec<u8>,
    /// The held directory that contains the entry.
    parent: File,
    /// The entry's own name.
    name: OsString,
}

/// What:
///  The held directory that contains `relative` below `root`,
/// reached without following a link,
/// or nothing when a directory
///  in front of the name is missing.
/// Why:
///   A path string is resolved again by every call that takes it,
/// and a link in front of the name sends the read somewhere
///  Git does not write.
/// A link or a file in front of the name is a failure that names it,
/// because Git replaces it with a
///  directory and nothing of it would be saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openParent(root: string, relative: Uint8Array): { parent: Dir; name: string } | undefined;
/// ```
fn open_parent(root: &Path, relative: &[u8]) -> Result<Option<(File, OsString)>, SaveFailure> {
    let shown: String = String::from_utf8_lossy(relative).into_owned();
    let names: Vec<OsString> = match crate::fs_beneath::plain_components(Path::new(OsStr::from_bytes(relative))) {
        Ok(found) => found,
        Err(error) => return Err(io_failure(&shown, &error)),
    };
    let Some((last, ancestors)) = names.split_last() else {
        return Err(SaveFailure::io(shown, String::from("it names no file")));
    };
    // The root is the worktree's real path as the layout reader found it; links among its own ancestors are the person's
    // arrangement and Git's own, not something a name below it can redirect.
    let mut current: File = match File::open(root) {
        Ok(opened) => opened,
        Err(error) => return Err(io_failure(&root.display().to_string(), &error)),
    };
    for (depth, name) in ancestors.iter().enumerate() {
        match open_directory_at(&current, name.as_os_str()) {
            Ok(next) => current = next,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                let subject: String = joined(&names[..=depth]);
                let reason: String = match error.raw_os_error() {
                    Some(libc::ELOOP) => String::from("is a symbolic link"),
                    Some(libc::ENOTDIR) => String::from("is a file, not a directory"),
                    Some(libc::EXDEV) => {
                        // Git writes through a mount; only this wrapper does not read across one.
                        return Err(SaveFailure::io(subject, String::from("is on another mount, which the wrapper does not read across")));
                    }
                    _ => format!("cannot be opened without following a link ({error})"),
                };
                return Err(SaveFailure::of_kind(FailureKind::Ancestor, subject, reason));
            }
        }
    }
    return Ok(Some((current, last.clone())));
}

/// What:
///  Find the entry at `relative` below `root`,
/// or nothing when no entry is there.
/// Why:
///   A path with no file in the worktree has nothing to copy,
/// and Git then creates it;
/// every other lookup failure
///  stops the save,
/// because Git would still write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(root: string, relative: Uint8Array): Located | undefined;
/// ```
pub fn locate(root: &Path, relative: &[u8]) -> Result<Option<Located>, SaveFailure> {
    let Some((parent, name)) = open_parent(root, relative)? else {
        return Ok(None);
    };
    return match entry_metadata_at(&parent, name.as_os_str()) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(io_failure(&String::from_utf8_lossy(relative), &error)),
        Ok(_) => Ok(Some(Located { root: root.to_path_buf(), relative: relative.to_vec(), parent, name })),
    };
}

/// What a file looked like at one moment:
/// which object,
/// how big,
/// when last written,
/// when last changed,
/// with what mode.
///  A copy proves the source held still against this,
///  and a later comparison asks whether it still does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Stamp {
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
    pub fn of(metadata: &std::fs::Metadata) -> Stamp {
        return Stamp {
            object: (metadata.dev(), metadata.ino()),
            size: metadata.size(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
            mode: metadata.mode(),
        };
    }
}

/// The failure for a source that changed while it was read;
/// `what` is a verb phrase that says how.
fn changed(located: &Located, what: &str) -> SaveFailure {
    return SaveFailure::of_kind(FailureKind::Changed, String::from_utf8_lossy(located.relative.as_slice()).into_owned(), what.to_string());
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
fn copy_link(located: &Located, before: &std::fs::Metadata, destination: &Path) -> Result<Stamp, SaveFailure> {
    let target: PathBuf = match read_link_at(&located.parent, located.name.as_os_str()) {
        Ok(read) => read,
        Err(error) => return Err(changed(located, format!("could not be read as a link ({error})").as_str())),
    };
    if let Err(error) = std::os::unix::fs::symlink(&target, destination) {
        return Err(io_failure(&String::from_utf8_lossy(located.relative.as_slice()), &error));
    }
    if !link_unchanged(located, before, &target) {
        let _removed = std::fs::remove_file(destination);
        return Err(changed(located, "was replaced while its link target was copied"));
    }
    return Ok(Stamp::of(before));
}

/// Whether the entry still has the stamp it had and still points at `target`.
fn link_unchanged(located: &Located, before: &std::fs::Metadata, target: &Path) -> bool {
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
fn copy_stream(located: &Located, from: &mut File, to: &mut File, before: Stamp, between: &mut dyn FnMut()) -> Result<(), CopyError> {
    let mut buffer: Vec<u8> = vec![0; 1 << 16];
    let mut copied: u64 = 0;
    while copied < before.size {
        let left: u64 = before.size - copied;
        let wanted: usize = usize::try_from(left).unwrap_or(usize::MAX).min(buffer.len());
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
        debug(TAG, &format!("{}: stamp before the copy {before:?}, after {after:?}", String::from_utf8_lossy(located.relative.as_slice())));
        return Err(changed(located, "changed while it was copied").into());
    }
    if copied != before.size {
        return Err(changed(located, format!("gave {copied} bytes where it had {}", before.size).as_str()).into());
    }
    // The name must still lead, below the same directories and without a link, to the object that was read.
    let named_now: Option<ObjectId> = match open_parent(located.root.as_path(), located.relative.as_slice())? {
        Some((parent, name)) => identity_at(&parent, name.as_os_str()).ok(),
        None => None,
    };
    if named_now != identity_of(from).ok() {
        return Err(changed(located, "no longer holds the file that was copied").into());
    }
    to.set_permissions(<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(before.mode & 0o777))?;
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
fn copy_entry(located: &Located, metadata: &std::fs::Metadata, destination: &Path, between: &mut dyn FnMut()) -> Result<Stamp, CopyError> {
    let shown: String = String::from_utf8_lossy(located.relative.as_slice()).into_owned();
    let mut from: File = open_file_at(&located.parent, located.name.as_os_str())?;
    let opened: std::fs::Metadata = from.metadata()?;
    if !opened.file_type().is_file() {
        return Err(SaveFailure::io(shown, String::from("it is neither a file nor a symbolic link")).into());
    }
    let before: Stamp = Stamp::of(&opened);
    if before != Stamp::of(metadata) {
        return Err(changed(located, "changed between the look at its name and the open").into());
    }
    let mut to: File = std::fs::OpenOptions::new().write(true).create_new(true).mode_600().open(destination)?;
    let streamed: Result<(), CopyError> = copy_stream(located, &mut from, &mut to, before, between);
    if streamed.is_err() {
        let _removed = std::fs::remove_file(destination);
        return Err(streamed.err().unwrap_or(CopyError::Failed(changed(located, "could not be copied"))));
    }
    return Ok(before);
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
/// function copyLocated(located: Located, destination: string, between: () => void): Stamp;
/// ```
pub fn copy_located(located: &Located, destination: &Path, between: &mut dyn FnMut()) -> Result<Stamp, SaveFailure> {
    let shown: String = String::from_utf8_lossy(located.relative.as_slice()).into_owned();
    let metadata: std::fs::Metadata = match entry_metadata_at(&located.parent, located.name.as_os_str()) {
        Ok(found) => found,
        Err(error) => return Err(io_failure(&shown, &error)),
    };
    if metadata.file_type().is_symlink() {
        return copy_link(located, &metadata, destination);
    }
    if !metadata.file_type().is_file() {
        return Err(SaveFailure::io(shown, String::from("it is neither a file nor a symbolic link")));
    }
    return match copy_entry(located, &metadata, destination, between) {
        Ok(stamp) => Ok(stamp),
        Err(CopyError::System(error)) => Err(io_failure(&shown, &error)),
        Err(CopyError::Failed(failure)) => Err(failure),
    };
}

/// What:
///  Whether the entry at `relative` below `root` is still the one the copy proved it read,
///  or why that cannot be asked.
/// Why:
///   A command the wrapper hands to Git discards what stands at that path now,
///  not what stood when the copy was made,
///  so the same proof the copy made is asked once more immediately before Git starts.
///  The stamp carries the object,
///  its size,
///  both its times and its mode,
///  so any write,
///  replacement or removal answers "changed".
///  A path the worktree no longer holds,
///  or holds as another kind of entry,
///  answers "changed" too,
///  and the caller saves it again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourceStillHolds(root: string, relative: Uint8Array, stamp: Stamp): boolean;
/// ```
/// What:
///  Whether the worktree still holds no entry at `relative`,
///  which is what a path with nothing to copy answered when it was read.
/// Why:
///   A path the listing named and the worktree did not hold had nothing to copy;
///  a file that appears there before Git discards that path is work that would go with no copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stillNothingThere(root: string, relative: Uint8Array): boolean;
/// ```
pub fn still_nothing_there(root: &Path, relative: &[u8]) -> bool {
    return matches!(locate(root, relative), Ok(None));
}

/// What:
///  Whether the entry at `relative` below `root` is still the one the copy proved it read.
/// Why:
///   A command the wrapper hands to Git discards what stands at that path now,
///  not what stood when the copy was made,
///  so the same proof the copy made is asked once more immediately before Git starts.
///  The stamp carries the object,
///  its size,
///  both its times and its mode,
///  so any write,
///  replacement or removal answers "changed".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourceStillHolds(root: string, relative: Uint8Array, stamp: Stamp): boolean;
/// ```
pub fn source_still_holds(root: &Path, relative: &[u8], stamp: Stamp) -> bool {
    let Ok(Some(located)) = locate(root, relative) else {
        return false;
    };
    let Ok(now) = entry_metadata_at(&located.parent, located.name.as_os_str()) else {
        return false;
    };
    return Stamp::of(&now) == stamp;
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
