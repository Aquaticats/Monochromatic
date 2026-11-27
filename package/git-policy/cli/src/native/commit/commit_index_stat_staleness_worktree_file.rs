//! What:
//!  One worktree file's stat data as an index entry records it,
//!  and whether an entry's recorded data is still that file's.
//! Why:
//!  Git compares a fixed set of 32-bit fields,
//!  so a comparison here has to cut them the same way or every entry would look changed.
//!  A mode is not the raw permission bits either:
//!  Git records a regular file as `100644` or `100755` and a link as `120000`,
//!  so the raw mode is reduced to what an entry keeps.
//!  A directory,
//!  socket or device in a tracked path is a change only Git can judge.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const matchesRecorded = (entry, file) => entry.mode === file.mode && entry.dev === file.dev && ...;
//! ```

/// An entry of the whole index,
///  whose mode and stat data an entry records.
use crate::index_full::FullEntry;
/// The nine 32-bit stat fields an entry stores,
///  which the comparison below reads.
use crate::index_stat::StatData;
/// `MetadataExt` is the trait whose `dev` and `ino` the file's own data is read with.
use std::os::unix::fs::MetadataExt;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The stat data one worktree file has,
/// each field cut to the 32 bits an index entry stores.
/// Why not the full 64-bit fields:
/// Git compares the index's cut fields with the file's own,
/// so a comparison here has to cut them the same way,
/// or every entry would look changed.
#[cfg(unix)]
pub(crate) struct WorktreeFile {
    /// The device.
    pub(crate) dev: u32,
    /// The inode.
    pub(crate) ino: u32,
    /// The mode Git would record for the file.
    pub(crate) mode: u32,
    /// The owner.
    pub(crate) uid: u32,
    /// The group.
    pub(crate) gid: u32,
    /// The size.
    pub(crate) size: u32,
    /// The change time,
    /// seconds.
    pub(crate) ctime_sec: u32,
    /// The change time,
    /// nanoseconds.
    pub(crate) ctime_nsec: u32,
    /// The modification time,
    /// seconds.
    pub(crate) mtime_sec: u32,
    /// The modification time,
    /// nanoseconds.
    pub(crate) mtime_nsec: u32,
}

/// The file-type bits of an entry's mode.
#[cfg(unix)]
pub(crate) const TYPE_BITS: u32 = 0o170_000;

/// The file-type bits of an entry that names another repository:
/// a gitlink holds no file to stat.
#[cfg(unix)]
pub(crate) const GITLINK: u32 = 0o160_000;

/// The file-type bits of a symbolic link.
#[cfg(unix)]
const SYMBOLIC_LINK: u32 = 0o120_000;

/// The file-type bits of a regular file.
#[cfg(unix)]
const REGULAR_FILE: u32 = 0o100_000;

/// The one permission bit of a file's mode that an index entry records.
#[cfg(unix)]
const EXECUTABLE_BIT: u32 = 0o100;

/// The mode an index entry has for a regular file,
/// as Git's `create_ce_mode` makes it.
#[cfg(unix)]
const MODE_REGULAR_FILE: u32 = 0o100_644;

/// The mode an index entry has for an executable regular file.
#[cfg(unix)]
const MODE_EXECUTABLE_FILE: u32 = 0o100_755;

/// What:
///  The mode an index entry has for a file with `file_mode`,
/// or nothing when no entry can describe the file.
/// Why:
///  Git records a regular file as 100644 or 100755 and a link as 120000,
/// whatever the rest of the permission bits say,
/// so comparing the raw mode would report every file as changed.
/// A directory,
/// socket or device in a tracked path is a change only Git can judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordedMode = (fileMode) =>
///   fileMode & TYPE_BITS === SYMBOLIC_LINK ? 0o120000
///   : fileMode & TYPE_BITS === REGULAR_FILE ? (fileMode & EXECUTABLE_BIT ? 0o100755 : 0o100644)
///   : undefined;
/// ```
#[cfg(unix)]
fn recorded_mode(file_mode: u32) -> Option<u32> {
    if file_mode & TYPE_BITS == SYMBOLIC_LINK {
        return Some(SYMBOLIC_LINK);
    }
    if file_mode & TYPE_BITS != REGULAR_FILE {
        return None;
    }
    return if file_mode & EXECUTABLE_BIT != 0 {
        Some(MODE_EXECUTABLE_FILE)
    } else {
        Some(MODE_REGULAR_FILE)
    };
}

/// What:
///  The stat data the file at `path` has.
/// Why:
///  A file this cannot read is one Git has to judge,
///  and so is a link where the worktree expects none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktreeFile = (path) => {
///   const stat = lstatSync(path); // the link itself, not what it points at
///   const mode = recordedMode(stat.mode);
///   return mode === undefined ? undefined : { dev: stat.dev, ino: stat.ino, mode, ... };
/// };
/// ```
#[cfg(unix)]
pub(crate) fn worktree_file(path: &Path) -> Option<WorktreeFile> {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    let mode: u32 = recorded_mode(metadata.mode())?;
    return Some(WorktreeFile {
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        mode,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
        ctime_sec: metadata.ctime() as u32,
        ctime_nsec: metadata.ctime_nsec() as u32,
        mtime_sec: metadata.mtime() as u32,
        mtime_nsec: metadata.mtime_nsec() as u32,
    });
}

/// What:
///  Whether the stat data an entry records is the stat data its file has.
/// Why:
///  This is the question `git update-index --refresh` asks,
///  field for field,
///  and only an entry that answers it needs no refresh.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const matchesRecorded = (entry, file) => entry.mode === file.mode && entry.dev === file.dev && entry.ino === file.ino &&
///   entry.uid === file.uid && entry.gid === file.gid && entry.size === file.size &&
///   entry.ctimeSec === file.ctimeSec && entry.ctimeNsec === file.ctimeNsec &&
///   entry.mtimeSec === file.mtimeSec && entry.mtimeNsec === file.mtimeNsec;
/// ```
#[cfg(unix)]
pub(crate) fn matches_recorded(
    entry: &FullEntry,
    file: &WorktreeFile,
) -> bool {
    let recorded: StatData = entry.stat;
    return entry.mode == file.mode
        && recorded.dev == file.dev
        && recorded.ino == file.ino
        && recorded.uid == file.uid
        && recorded.gid == file.gid
        && recorded.size == file.size
        && recorded.ctime_sec == file.ctime_sec
        && recorded.ctime_nsec == file.ctime_nsec
        && recorded.mtime_sec == file.mtime_sec
        && recorded.mtime_nsec == file.mtime_nsec;
}
