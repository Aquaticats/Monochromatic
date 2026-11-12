//! What:
//! The identity of a file system entry (device and inode,
//! read without following links),
//! the clock the lean-folder tools measure age against,
//! and the deadline every walk of theirs obeys.
//! Why:
//! A removal is judged on an identity taken when the entry was classified and checked again just before the unlink,
//! so a path that was swapped for another entry in between is never removed.
//! Nothing here may wait without end:
//! a walk that passes its deadline stops and says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const identityOf = (path: string): { device: bigint; inode: bigint } => { const s = lstatSync(path, { bigint: true }); return { device: s.dev, inode: s.ino }; };
//! ```

/// The metadata fields that carry device and inode.
use std::os::unix::fs::MetadataExt;
/// Paths and the file system error type.
use std::{fs, io, path::Path};
/// Instants for deadlines and the system clock for ages.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The device and inode of one entry.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Identity {
    /// The device number (`st_dev`).
    pub device: u64,
    /// The inode number (`st_ino`).
    pub inode: u64,
}

/// What:
/// `pub fn identity_of(path: &Path) -> io::Result<Identity>` reads the identity of the entry itself,
/// never of a link's target.
/// Why:
/// A removal must name what it removes,
/// and a link is removed as a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function identityOf(path: string): Identity  // lstat
/// ```
pub fn identity_of(path: &Path) -> io::Result<Identity> {
    let metadata: fs::Metadata = fs::symlink_metadata(path)?;
    return Ok(Identity { device: metadata.dev(), inode: metadata.ino() });
}

/// What:
/// `pub fn seconds_since_epoch(time: SystemTime) -> i64` is a file time as whole seconds from the Unix epoch (negative before it).
/// Why:
/// Ages and thresholds are compared as plain integers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const secondsSinceEpoch = (time: Date): number => Math.floor(time.getTime() / 1000);
/// ```
pub fn seconds_since_epoch(time: SystemTime) -> i64 {
    return match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_secs()).unwrap_or(i64::MAX),
    };
}

/// What:
/// `pub fn missing(path: &Path) -> bool` says whether the entry at a path is confirmed absent.
/// Why:
/// `Path::exists()` answers the same for a path that is not there and one whose metadata could not be read
/// (a directory this user may not search,
/// an input and output error,
/// a link that loops),
/// and a caller that removes what a path stands for must not read a refusal as absence:
/// only a `NotFound` answer is absence,
/// and every other answer keeps whatever the caller would remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const missing = (path: string): boolean => { try { lstatSync(path); return false; } catch (error) { return error.code === 'ENOENT'; } };
/// ```
pub fn missing(path: &Path) -> bool {
    return matches!(fs::symlink_metadata(path), Err(error) if error.kind() == io::ErrorKind::NotFound);
}

/// What:
/// `pub fn modified_seconds(path: &Path) -> io::Result<i64>` is the modification time of the entry itself in seconds.
/// Why:
/// Idle and age rules read file times and nothing else of the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const modifiedSeconds = (path: string): number => Math.floor(lstatSync(path).mtimeMs / 1000);
/// ```
pub fn modified_seconds(path: &Path) -> io::Result<i64> {
    return Ok(fs::symlink_metadata(path)?.mtime());
}

/// The wall-clock time now in seconds,
/// the single place the tools read the clock so tests can pass their own.
pub fn now_seconds() -> i64 {
    return seconds_since_epoch(SystemTime::now());
}

/// A point in time after which a walk stops.
#[derive(Clone, Copy, Debug)]
pub struct Deadline {
    /// The instant the walk must be over by.
    at: Instant,
}

/// Construction and tests of a deadline.
impl Deadline {
    /// A deadline `limit` from now.
    pub fn after(limit: Duration) -> Deadline {
        return Deadline { at: Instant::now() + limit };
    }

    /// Whether the deadline has passed.
    pub fn passed(&self) -> bool {
        return Instant::now() >= self.at;
    }

    /// The time left,
    /// zero when it has passed.
    pub fn remaining(&self) -> Duration {
        return self.at.saturating_duration_since(Instant::now());
    }
}
