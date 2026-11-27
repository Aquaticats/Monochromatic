//! What:
//!  The one removal by name this folder keeps,
//! for the lock file this program created and still holds open,
//! and the publishing of an open file under a new name.
//! Why:
//!  A name can be taken between the look and the unlink,
//! and nothing in the kernel makes the two one step,
//! so everything else this program made is kept and reported rather than removed by its name.
//! A lock file is the exception because no other writer can have replaced it without breaking the lock protocol,
//! so it is removed only after the identity recorded when it was made matches the one the name holds.
//! Publishing an open file under a name is the same idea in the other direction:
//! `linkat` makes the object the descriptor holds into a new name and fails on an existing one,
//! so nothing is replaced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owned = await removeIfIdentical(dir, 'index.lock', recorded); // 'removed' | 'absent' | { kept: reason }
//! await linkOpenFileAt(file, dir, 'index');                          // fails when the name exists
//! ```

use super::fs_beneath_entries::{
    entry_metadata_at,
    remove_at,
};
use super::fs_beneath_identity::{
    ObjectId,
    identity_at,
};
use super::fs_beneath_names::{
    c_string_of,
    checked,
    handle_path,
    name_of,
};
use std::ffi::OsStr;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;

/// What a request to remove an entry that the caller made did.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Owned {
    /// The entry was removed.
    Removed,
    /// Nothing was at the name.
    Absent,
    /// The name held another object and was left alone,
    /// for this reason.
    Kept(String),
}

/// What:
///  Remove the entry `name` of `dir` if,
/// at the moment of the look,
/// it is the object with identity `expected`.
/// Why:
///   This is the one removal by name the ladder keeps,
/// for the lock file it created and still holds open:
/// it is what Git
///  itself does with its own lock.
/// A name can be taken between the look and the unlink,
/// and nothing in the kernel makes the
///  two one step,
/// so the function is used only for a lock that no other writer can have replaced without breaking the lock
///  protocol,
/// and never to remove a tree:
/// a directory is refused.
/// Everything else the ladder made stays and is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeIfIdentical(dir, 'index.lock', expected);
/// ```
pub fn remove_if_identical(
    dir: &File,
    name: &OsStr,
    expected: ObjectId,
) -> std::io::Result<Owned> {
    let found: ObjectId = match identity_at(dir, name) {
        Ok(known) => known,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Owned::Absent),
        Err(error) => return Err(error),
    };
    if found != expected {
        return Ok(Owned::Kept(format!(
            "{} is not the object that was made ({} against {})",
            name.to_string_lossy(),
            found.encode(),
            expected.encode()
        )));
    }
    if entry_metadata_at(dir, name)?.is_dir() {
        return Ok(Owned::Kept(format!(
            "{} is a directory and is not removed",
            name.to_string_lossy()
        )));
    }
    return match remove_at(dir, name, false) {
        Ok(()) => Ok(Owned::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Owned::Absent),
        Err(error) => Err(error),
    };
}

/// What:
///  Make the open file `source` another name `name` of `dir`,
/// failing if the name exists.
/// Why:
///   The object published is the one the descriptor holds,
/// not whatever a separately checked name refers to at the moment of a
///  rename,
/// and an existing entry is never replaced (`linkat` fails with `EEXIST`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await linkOpenFileAt(file, dir, 'index');
/// ```
pub fn link_open_file_at(
    source: &File,
    dir: &File,
    name: &OsStr,
) -> std::io::Result<()> {
    let c_name = name_of(name)?;
    let through = c_string_of(handle_path(source).as_os_str().as_bytes(), "a path")?;
    // SAFETY: both strings outlive the call, `dir` is an open descriptor, and `AT_SYMLINK_FOLLOW` makes the link to the file
    // the descriptor's magic link names.
    checked(unsafe {
        libc::linkat(
            libc::AT_FDCWD,
            through.as_ptr(),
            dir.as_raw_fd(),
            c_name.as_ptr(),
            libc::AT_SYMLINK_FOLLOW,
        )
    })?;
    return Ok(());
}
