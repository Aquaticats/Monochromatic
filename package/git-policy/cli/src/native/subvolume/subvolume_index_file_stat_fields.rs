//! What:
//!  The stat fields a Git index entry stores for one file:
//!  read from what `lstat` reports now,
//!  and compared with the fields an entry recorded when Git wrote it.
//! Why:
//!  A reflink copy gives every file a new inode number and change time,
//!  so the rewrite refreshes an entry only where the file on disk is provably the one Git recorded.
//!  The recorded comparison reads a zero nanosecond count as any present value,
//!  which is Git's own reading of an index written by a Git built without nanosecond support,
//!  and the exact comparison is the proof a caller needs before it skips reading the file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const now = statFields(tree, path);             // the file's own lstat, in index units
//! const same = matchesStored(now, entry.stored);  // Git's own comparison
//! ```

use crate::subvolume_index::{
    StatFields,
    StoredStat,
};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  What `lstat` of `tree/path` reports now,
/// in index units,
/// or nothing for a file that cannot be examined.
/// Why:
///   The rewrite leaves an entry as Git recorded it when its file is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFields(tree: string, path: Uint8Array): StatFields | undefined;
/// ```
pub(crate) fn stat_fields(
    tree: &Path,
    path: &[u8],
) -> Option<StatFields> {
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(tree.join(OsStr::from_bytes(path))).ok()?;
    return Some(fields_of(&metadata));
}

/// What:
///  The stat fields of metadata in hand,
/// in index units.
/// Why:
///   A caller that holds the object it made records the fields of that object,
/// not of whatever its path names later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fieldsOf(stats: Stats): StatFields;
/// ```
pub(crate) fn fields_of(metadata: &std::fs::Metadata) -> StatFields {
    return StatFields {
        ctime_seconds: metadata.ctime() as u32,
        ctime_nanos: metadata.ctime_nsec() as u32,
        mtime_seconds: metadata.mtime() as u32,
        mtime_nanos: metadata.mtime_nsec() as u32,
        dev: metadata.dev() as u32,
        ino: metadata.ino() as u32,
        uid: metadata.uid(),
        gid: metadata.gid(),
        size: metadata.size() as u32,
    };
}

/// What:
///  Whether a file's present stat equals what an entry recorded (the device is not compared:
/// a subvolume has its own).
/// Why:
///   A recorded nanosecond count of zero is what a Git built without nanosecond support wrote for a time it could not
///  express,
/// so it matches any present count;
/// this is Git's own reading of such an entry,
/// and it is not a proof
///  that the file holds the bytes the entry names (`matches_stored_exactly` is that proof).
pub(crate) fn matches_stored(
    now: &StatFields,
    stored: &StoredStat,
) -> bool {
    return now.size == stored.size
        && now.ino == stored.ino
        && now.uid == stored.uid
        && now.gid == stored.gid
        && now.ctime_seconds == stored.ctime_seconds
        && (stored.ctime_nanos == 0 || now.ctime_nanos == stored.ctime_nanos)
        && now.mtime_seconds == stored.mtime_seconds
        && (stored.mtime_nanos == 0 || now.mtime_nanos == stored.mtime_nanos);
}

/// What:
///  Whether a file's present stat equals what an entry recorded,
/// with the change and modification times
///  agreeing to the nanosecond,
/// so no recorded field was matched as a wildcard and the file is provably the
///  one the entry described.
/// Why:
///   A recorded count of zero is what a Git built without nanosecond support wrote for a time it could not
///  express,
/// and the recorded comparison reads zero as any present value;
/// such an entry equals a file of the
///  same size,
/// inode and second even after an edit of the same length,
/// so a caller that would skip reading the
///  file must require the counts themselves to agree.
/// A file whose time is exactly on a second records zero as
///  well,
/// and its zero equals the record's,
/// which is an exact match of what the index holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// matchesStoredExactly(now, stored); // every recorded field, the nanosecond counts included, is equal
/// ```
pub(crate) fn matches_stored_exactly(
    now: &StatFields,
    stored: &StoredStat,
) -> bool {
    return now.ctime_nanos == stored.ctime_nanos
        && now.mtime_nanos == stored.mtime_nanos
        && matches_stored(now, stored);
}
