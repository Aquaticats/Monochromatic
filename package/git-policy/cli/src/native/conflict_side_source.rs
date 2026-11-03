//! What:
//!  Finding the worktree file a conflict-side command is about to replace, and copying it.
//! Why:
//!  The copy is the owner's "save, then allow": whatever Git is about to overwrite must be on disk first. Git writes
//!  `dir/file` by replacing whatever stands at `dir` (a symbolic link or a file) with a real directory, and by replacing the
//!  file as it is when Git runs. So the walk to the file never follows a link, a link or file in front of the name is a
//!  failure that names it, and a copy counts as saved only when the file it read is the file the name still holds and did
//!  not change while it was read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const located = locate(root, 'dir/a.rs'); // undefined when nothing is there to save; throws below a link
//! if (located) copyLocated(located, destination, () => {}); // throws when the file changed while it was copied
//! ```

use super::diagnostic_log::debug;
use super::fs_beneath::{identity_at, identity_of, open_directory_at, open_file_at, read_link_at, entry_metadata_at};
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
    /// The system refused a read, a write or a place: free the space or fix the permissions.
    Io,
    /// A link or a file stands where a directory in front of the name should be, and Git would replace it unsaved.
    Ancestor,
    /// The file changed while it was copied, so the copy could be stale or cut short.
    Changed,
}

/// Why a copy could not be written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFailure {
    /// The repository-relative path being copied, the directory in front of it that is in the way, or the place the copies go
    ///  when none was reached.
    pub subject: String,
    /// The operating system's words, or what the subject is.
    pub reason: String,
    /// Which advice fits.
    pub kind: FailureKind,
}

impl SaveFailure {
    /// A failure of the system on `subject`, logged.
    pub fn io(subject: String, reason: String) -> SaveFailure {
        debug(TAG, &format!("fail closed: {subject}: {reason}"));
        return SaveFailure { subject, reason, kind: FailureKind::Io };
    }

    /// A failure of kind `kind` on `subject`, logged.
    fn of_kind(kind: FailureKind, subject: String, reason: String) -> SaveFailure {
        debug(TAG, &format!("fail closed ({kind:?}): {subject}: {reason}"));
        return SaveFailure { subject, reason, kind };
    }
}

/// A worktree entry that exists and is to be copied.
#[derive(Debug)]
pub struct Located {
    /// The top of the worktree, which is trusted: the reader returns its real path.
    root: PathBuf,
    /// The path below the root, as typed.
    relative: Vec<u8>,
    /// The held directory that contains the entry.
    parent: File,
    /// The entry's own name.
    name: OsString,
}

/// What:
///  The held directory that contains `relative` below `root`, reached without following a link, or nothing when a directory
///  in front of the name is missing.
/// Why:
///   A path string is resolved again by every call that takes it, and a link in front of the name sends the read somewhere
///  Git does not write. A link or a file in front of the name is a failure that names it, because Git replaces it with a
///  directory and nothing of it would be saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openParent(root: string, relative: Uint8Array): { parent: Dir; name: string } | undefined;
/// ```
fn open_parent(root: &Path, relative: &[u8]) -> Result<Option<(File, OsString)>, SaveFailure> {
    let shown: String = String::from_utf8_lossy(relative).into_owned();
    let names: Vec<OsString> = super::fs_beneath::plain_components(Path::new(OsStr::from_bytes(relative)))
        .map_err(|error| { return SaveFailure::io(shown.clone(), error.to_string()); })?;
    let Some((last, ancestors)) = names.split_last() else {
        return Err(SaveFailure::io(shown, String::from("it names no file")));
    };
    // The root is the worktree's real path as the layout reader found it; links among its own ancestors are the person's
    // arrangement and Git's own, not something a name below it can redirect.
    let mut current: File = File::open(root).map_err(|error| { return SaveFailure::io(root.display().to_string(), error.to_string()); })?;
    for (depth, name) in ancestors.iter().enumerate() {
        match open_directory_at(&current, name.as_os_str()) {
            Ok(next) => current = next,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                let subject: String = names[..=depth].iter().map(|each| { return each.to_string_lossy().into_owned(); }).collect::<Vec<String>>().join("/");
                let reason: String = match error.raw_os_error() {
                    Some(libc::ELOOP) => String::from("is a symbolic link"),
                    Some(libc::ENOTDIR) => String::from("is a file, not a directory"),
                    Some(libc::EXDEV) => String::from("is on another mount"),
                    _ => format!("cannot be opened without following a link ({error})"),
                };
                return Err(SaveFailure::of_kind(FailureKind::Ancestor, subject, reason));
            }
        }
    }
    return Ok(Some((current, last.clone())));
}

/// What:
///  Find the entry at `relative` below `root`, or nothing when no entry is there.
/// Why:
///   A path with no file in the worktree has nothing to copy, and Git then creates it; every other lookup failure
///  stops the save, because Git would still write.
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
        Err(error) => Err(SaveFailure::io(String::from_utf8_lossy(relative).into_owned(), error.to_string())),
        Ok(_) => Ok(Some(Located { root: root.to_path_buf(), relative: relative.to_vec(), parent, name })),
    };
}

/// What a file looked like at one moment: which object, how big, when last written, when last changed, with what mode.
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

/// The failure for a source that changed while it was read; `what` is a verb phrase that says how.
fn changed(located: &Located, what: &str) -> SaveFailure {
    return SaveFailure::of_kind(FailureKind::Changed, String::from_utf8_lossy(located.relative.as_slice()).into_owned(), what.to_string());
}

/// What:
///  Copy a symbolic link's target to `destination` and check that the link is the same one afterwards.
/// Why:
///   The link is read by name below the held directory; a link replaced between the read and the end is a copy of the old one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLink(located: Located, destination: string): void;
/// ```
fn copy_link(located: &Located, before: &std::fs::Metadata, destination: &Path) -> Result<(), SaveFailure> {
    let target: PathBuf = read_link_at(&located.parent, located.name.as_os_str()).map_err(|error| { return changed(located, format!("could not be read as a link ({error})").as_str()); })?;
    std::os::unix::fs::symlink(&target, destination).map_err(|error| { return SaveFailure::io(String::from_utf8_lossy(located.relative.as_slice()).into_owned(), error.to_string()); })?;
    let still: bool = entry_metadata_at(&located.parent, located.name.as_os_str()).is_ok_and(|now| { return Stamp::of(&now) == Stamp::of(before); })
        && read_link_at(&located.parent, located.name.as_os_str()).is_ok_and(|now| { return now == target; });
    if !still {
        let _removed = std::fs::remove_file(destination);
        return Err(changed(located, "was replaced while its link target was copied"));
    }
    return Ok(());
}

/// What:
///  Copy one file's content, mode and kind to `destination`, which must not exist, sync it, and prove the source held still.
/// Why:
///   `between` runs after each chunk is written, so a control can change the source mid-copy. The stamp of the open
///  descriptor is taken before the first read and after the last, the byte count must equal the size, and the name must
///  still lead, without a link, to the object that was read; any difference removes the copy and fails, because a stale or
///  torn copy would pass for the file Git then replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function copyLocated(located: Located, destination: string, between: () => void): void;
/// ```
pub fn copy_located(located: &Located, destination: &Path, between: &mut dyn FnMut()) -> Result<(), SaveFailure> {
    let shown: String = String::from_utf8_lossy(located.relative.as_slice()).into_owned();
    let io_failure = |error: std::io::Error| -> SaveFailure { return SaveFailure::io(shown.clone(), error.to_string()); };
    let metadata: std::fs::Metadata = entry_metadata_at(&located.parent, located.name.as_os_str()).map_err(&io_failure)?;
    if metadata.file_type().is_symlink() {
        return copy_link(located, &metadata, destination);
    }
    if !metadata.file_type().is_file() {
        return Err(SaveFailure::io(shown.clone(), String::from("it is neither a file nor a symbolic link")));
    }
    let mut from: File = open_file_at(&located.parent, located.name.as_os_str()).map_err(&io_failure)?;
    let opened: std::fs::Metadata = from.metadata().map_err(&io_failure)?;
    if !opened.file_type().is_file() {
        return Err(SaveFailure::io(shown.clone(), String::from("it is neither a file nor a symbolic link")));
    }
    let before: Stamp = Stamp::of(&opened);
    if before != Stamp::of(&metadata) {
        return Err(changed(located, "changed between the look at its name and the open"));
    }
    let mut to: File = std::fs::OpenOptions::new().write(true).create_new(true).mode_600().open(destination).map_err(&io_failure)?;
    let written: Result<(), SaveFailure> = (|| -> Result<(), SaveFailure> {
        let mut buffer: Vec<u8> = vec![0; 1 << 16];
        let mut copied: u64 = 0;
        loop {
            let count: usize = from.read(buffer.as_mut_slice()).map_err(&io_failure)?;
            if count == 0 {
                break;
            }
            to.write_all(&buffer[..count]).map_err(&io_failure)?;
            copied += u64::try_from(count).unwrap_or(u64::MAX);
            between();
        }
        let after: Stamp = Stamp::of(&from.metadata().map_err(&io_failure)?);
        if after != before {
            debug(TAG, &format!("{}: stamp before the copy {before:?}, after {after:?}", String::from_utf8_lossy(located.relative.as_slice())));
            return Err(changed(located, "changed while it was copied"));
        }
        if copied != before.size {
            return Err(changed(located, format!("gave {copied} bytes where it had {}", before.size).as_str()));
        }
        // The name must still lead, below the same directories and without a link, to the object that was read.
        let named_now = open_parent(located.root.as_path(), located.relative.as_slice())?
            .and_then(|(parent, name)| { return identity_at(&parent, name.as_os_str()).ok(); });
        if named_now != identity_of(&from).ok() {
            return Err(changed(located, "no longer holds the file that was copied"));
        }
        to.set_permissions(<std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(before.mode & 0o777)).map_err(&io_failure)?;
        return to.sync_all().map_err(&io_failure);
    })();
    if let Err(error) = written {
        // A cut-short file named like a copy would pass for the whole file; the command is rejected, so none is left.
        let _removed = std::fs::remove_file(destination);
        return Err(error);
    }
    return Ok(());
}

/// `OpenOptions` with the private creation mode, named for what the call does.
trait PrivateCreation {
    /// The options with creation mode `0600`.
    fn mode_600(&mut self) -> &mut Self;
}

impl PrivateCreation for std::fs::OpenOptions {
    fn mode_600(&mut self) -> &mut Self {
        return std::os::unix::fs::OpenOptionsExt::mode(self, 0o600);
    }
}

/// Where a file is found and how it is copied, including a file that changes while it is read.
#[cfg(test)]
#[path = "conflict_side_source_tests.rs"]
mod tests;
