//! What:
//!  Creating one file in the stage:
//! the private directories it lands in,
//! the clone request that copies its bytes when the filesystem can,
//! the exclusive creation that fills it,
//! and the change time that proves a clone's bytes.
//! Why:
//!  The stage must hold every byte before the final comparison,
//! and the stage is private whatever the source's modes say.
//! A clone is the machine's choice,
//! so the caller supplies the request and reads back the stamp that proves the bytes,
//! and a filesystem that refuses the clone gets a plain byte copy.
//! The stamp and the clock that settles it live here because they answer one question,
//! "are these bytes the source's bytes and nothing else?".
//! They live beside the leaf copying because that file was over the line limit before this change
//! (`worktree_copy_stage_leaf_copying.rs`,
//! which re-exports them).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stamp = copyFileExclusiveWith({ sourcePath, stagePath, clone }); // undefined when bytes were copied
//! ```

use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure_of,
};
use crate::worktree_copy_manifest::{
    FileStamp,
    stamp_of,
};
use std::path::Path;
/// What:
///  Creates one private directory with mode `0700`,
/// parents included.
/// Why:
///   Staged content is never readable by other users,
/// even before installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
#[cfg(unix)]
pub fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::DirBuilderExt;
    return std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(raw_failure_of);
}

/// What:
///  Creates one directory tree on a platform without Unix modes.
/// Why:
///   Keeps the staging code identical on every platform;
/// no mode is claimed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    return std::fs::create_dir_all(path).map_err(raw_failure_of);
}

/// What:
///  Asks the filesystem to clone the source's content into a new destination file,
/// when it can.
/// Why:
///   A clone shares storage on copy-on-write filesystems;
/// any refusal means the caller must copy bytes instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
#[cfg(target_os = "linux")]
pub fn try_clone(
    source: &std::fs::File,
    destination: &std::fs::File,
) -> bool {
    // The one `FICLONE` call lives with the other Linux filesystem primitives, which the cheap-worktree ladder shares.
    return crate::fs_clone_exchange::clone_file(source, destination, std::path::Path::new(""))
        .is_ok();
}

/// What:
///  Declines cloning on platforms where this lane has no clone request,
/// so every copy takes the byte path.
/// Why:
///   The incumbent's fallback is a full copy,
/// which is always correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
pub fn try_clone(
    _source: &std::fs::File,
    _destination: &std::fs::File,
) -> bool {
    return false;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  cloning when the filesystem allows.
/// Why:
///   Exclusive creation never overwrites an existing staged file.
///  The failure is the operating system's own,
/// so a caller that is not this copy reads the cause from its kind
///  instead of guessing it from the message,
/// and the copy stage words it for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFile(sourcePath, stagePath, COPYFILE_EXCL | COPYFILE_FICLONE);
/// ```
pub fn copy_file_exclusive(
    source: &Path,
    destination: &Path,
) -> Result<(), std::io::Error> {
    return copy_file_exclusive_with(source, destination, try_clone).map(drop);
}

/// What:
///  How recent a change time may be and still be too close to the clock to prove "unchanged".
/// Why:
///   Filesystem timestamps have a granularity;
/// a write that lands in the same tick as the stamp would leave it equal,
///  so a file changed this recently is compared byte for byte instead of trusted.
///  A control states it as well,
/// so it can ask for a settled stamp without waiting for the clock to move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RECENT_CHANGE_NANOS = 20_000_000n;
/// ```
pub(crate) const RECENT_CHANGE_NANOS: i128 = 20_000_000;

/// A time of the file system's stamping in nanoseconds since the epoch,
/// from its seconds and nanoseconds.
/// One conversion for the clock and the stamp,
/// so a control can compare the same two figures.
pub(crate) fn stamp_nanos(parts: (i64, i64)) -> i128 {
    return i128::from(parts.0) * 1_000_000_000 + i128::from(parts.1);
}

/// What:
///  The clock that says whether a change time is old enough to prove that no write follows it unseen,
/// in
///  nanoseconds since the epoch.
/// Why:
///   A control has to ask the question about a stated instant,
/// not about how long the host took between
///  the write and the look:
/// a control of this repository failed once in a whole-suite run because a 7-byte write
///  and the look after its copy were more than the 20 ms granularity apart
/// (`doc/troubleshooting/git-policy-cli-stop-lock-controls-wait-on-order-not-time.md`).
/// Nothing of
///  the pinning is in the release executable.
fn clock_nanos() -> Option<i128> {
    #[cfg(test)]
    if let Some(pinned) = PINNED_CLOCK_NANOS.with(std::cell::Cell::get) {
        return Some(pinned);
    }
    let now: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    return Some(i128::from(now.as_secs()) * 1_000_000_000 + i128::from(now.subsec_nanos()));
}

#[cfg(test)]
thread_local! {
    /// The reading a control on this thread gave the clock;
    /// nothing outside a control.
    static PINNED_CLOCK_NANOS: std::cell::Cell<Option<i128>> = const { std::cell::Cell::new(None) };
}

/// What:
///  Make the clock read `nanos` on this thread until the returned guard drops.
/// Why:
///   The control decides the question by the file's own change time against a stated instant,
/// so it never waits
///  for a clock to move and never fails because the host was slow.
#[cfg(test)]
pub(crate) fn clock_reads(nanos: i128) -> ClockPin {
    PINNED_CLOCK_NANOS.with(|slot| {
        slot.set(Some(nanos));
    });
    return ClockPin;
}

/// Ends the pinned reading of this thread's clock when dropped.
#[cfg(test)]
pub(crate) struct ClockPin;

#[cfg(test)]
impl Drop for ClockPin {
    fn drop(&mut self) {
        PINNED_CLOCK_NANOS.with(|slot| {
            slot.set(None);
        });
    }
}

/// What:
///  Whether a stamp is old enough,
/// against the clock now,
/// to prove that no write follows it unseen.
/// Why:
///   A change time in the future or within the timestamp granularity of now proves nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// now - ctime >= RECENT_CHANGE_NANOS
/// ```
pub(crate) fn stamp_is_settled(changed: (i64, i64)) -> bool {
    let Some(now_nanos) = clock_nanos() else {
        return false;
    };
    return now_nanos - stamp_nanos(changed) >= RECENT_CHANGE_NANOS;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  using the clone request the caller supplies,
/// and returns the source's stamp when a clone proved the bytes.
/// Why:
///   Whether the filesystem clones is the machine's choice,
/// so a test supplies a clone that accepts or refuses
///  and sees each path taken on every machine.
/// A clone shares the source's blocks as of one instant;
/// a source stamp that is
///  equal before and after it and old enough to be settled proves no write interleaved,
/// and the final check compares it again.
/// The source is opened by the plain-file reader,
/// which waits for no writer and follows no link,
///  so a name the walk listed as a file and a dev server replaced with a named pipe between the two passes
///  is refused at once instead of waiting for a writer that may never come,
/// with the checkout held and nothing
///  said ("nothing should ever wait forever or indefinitely").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stamp = await copyFileWith({ sourcePath, stagePath, clone }); // undefined when bytes were copied
/// ```
pub fn copy_file_exclusive_with(
    source: &Path,
    destination: &Path,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<Option<FileStamp>, std::io::Error> {
    let mut input: std::fs::File = crate::plain_file::open_plain_file(source)?;
    let mut output: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let before: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    // A successful clone already holds every byte, so only a refused clone copies them.
    if !clone(&input, &output) {
        std::io::copy(&mut input, &mut output)?;
        return Ok(None);
    }
    let after: Option<FileStamp> = input.metadata().ok().as_ref().and_then(stamp_of);
    return Ok(match (before, after) {
        (Some(first), Some(second)) if first == second && stamp_is_settled(second.changed) => {
            Some(second)
        }
        _ => None,
    });
}
