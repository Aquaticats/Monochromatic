//! What:
//!  How the in-process index update gets its result onto the landing's private index:
//! the name of that index,
//! a temporary file
//!  no other run can share,
//! Git's file mode for an index,
//! and the rename.
//! Why:
//!  The update replaces `index.cli-git-<batch>`,
//! the copy of the live index the landing made under its `index.lock`;
//! the
//!  lock's release then renames that copy over the live index (`service_index_hold.rs`,
//! `IndexLock::release_with`).
//! The update
//!  takes a borrow of the held `IndexLock`,
//! so only the holder can call it,
//! and it works on the one name `private_index_path`
//!  gives for that lock's index and batch.
//! Two runs on one target must not share a temporary name:
//! each creates its
//!  own file exclusively and removes only that one.
//! Git creates an index through a lock file whose mode `adjust_shared_perm`
//!  widens by `core.sharedRepository` (`tempfile.c`,
//! `path.c`),
//! so the file gets the same mode before it is published.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await publish(privateIndexPath(lock.index(), seq), bytes, sharedRepository); // takes the held lock as a witness
//! ```

use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// How many times a temporary name is tried before the update gives up.
const NAME_TRIES: usize = 16;

/// A count that makes the temporary names of one process differ.
static NEXT_NAME: AtomicU64 = AtomicU64::new(0);

/// The mode `calc_shared_perm` makes of `mode` for the `core.sharedRepository` value `shared` (`path.c`).
pub fn shared_mode(shared: i32, mode: u32) -> u32 {
    let mut tweak: u32 = shared.unsigned_abs();
    if mode & 0o200 == 0 {
        tweak &= !0o222;
    }
    if mode & 0o100 != 0 {
        tweak |= (tweak & 0o444) >> 2;
    }
    return if shared < 0 { (mode & !0o777) | tweak } else { mode | tweak };
}

/// A file name beside `target` that no other run uses:
/// the process,
/// a count and the clock.
fn temporary_beside(target: &Path) -> Option<PathBuf> {
    let count: u64 = NEXT_NAME.fetch_add(1, Ordering::Relaxed);
    let clock: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
    return Some(target.with_file_name(format!("{}.reset-{}-{count}-{clock:x}", target.file_name()?.to_string_lossy(), std::process::id())));
}

/// The steps of a publication that touch the file system,
/// so a control can make any one of them fail or run short.
pub struct Steps<'a> {
    /// One write of some of the bytes;
    /// returns how many were taken.
    pub write: &'a dyn Fn(&mut std::fs::File, &[u8]) -> std::io::Result<usize>,
    /// The change of the open file's permission bits.
    pub chmod: &'a dyn Fn(&std::fs::File, u32) -> std::io::Result<()>,
    /// The flush of the file to the disk.
    pub sync: &'a dyn Fn(&std::fs::File) -> std::io::Result<()>,
    /// The rename of the finished file over the target.
    pub rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// The mode bits the new file has,
    /// as the file system gave them.
    pub mode: &'a dyn Fn(&std::fs::File) -> std::io::Result<u32>,
}

/// The permission bits the file system gave the new file,
/// which already include the mask and any inherited default ACL.
fn mode_of_file(file: &std::fs::File) -> std::io::Result<u32> {
    return Ok(file.metadata()?.permissions().mode());
}

/// One write to the file.
fn write_some(file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<usize> {
    return file.write(bytes);
}

/// The file's permission bits set with `fchmod`.
fn set_mode(file: &std::fs::File, mode: u32) -> std::io::Result<()> {
    return file.set_permissions(std::fs::Permissions::from_mode(mode));
}

/// The file flushed to the disk,
/// data and metadata (`CSUM_FSYNC`).
fn sync_all(file: &std::fs::File) -> std::io::Result<()> {
    return crate::service_durable::flush_index(file, "index-publish");
}

/// The rename of the finished file over the target.
fn rename_file(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::rename(from, to);
}

/// The operating system's own calls.
pub const SYSTEM: Steps<'static> = Steps { write: &write_some, chmod: &set_mode, sync: &sync_all, rename: &rename_file, mode: &mode_of_file };

/// Create the temporary file as Git creates its index lock file and give it Git's mode before a byte is written to it:
/// Git
/// creates the file with the mode 0666 and lets the file system narrow it by the mask and any default ACL of the directory,
/// and
/// `adjust_shared_perm` (`tempfile.c:152`) then widens or replaces the bits it finds,
/// so the calculation starts from the mode the
/// new file really has.
/// Nothing is written before that,
/// so a `core.sharedRepository` value narrower than the mask or the ACL gives
/// never lets the bytes be read more widely.
fn create_beside(steps: &Steps<'_>, target: &Path, shared: i32) -> std::io::Result<(std::fs::File, PathBuf)> {
    use std::os::unix::fs::OpenOptionsExt as _;
    for _ in 0..NAME_TRIES {
        let Some(temporary) = temporary_beside(target) else {
            break;
        };
        let file: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o666).open(&temporary) {
            Ok(created) => created,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        if shared != 0 {
            let changed: std::io::Result<()> = match (steps.mode)(&file) {
                Ok(gits_file) => (steps.chmod)(&file, shared_mode(shared, gits_file) & 0o7777),
                Err(error) => Err(error),
            };
            if let Err(error) = changed {
                let _removed = std::fs::remove_file(&temporary);
                return Err(error);
            }
        }
        return Ok((file, temporary));
    }
    return Err(std::io::Error::other("no unused temporary name beside the index"));
}

/// Write all of `bytes`,
/// however the writes are cut.
fn write_all_with(steps: &Steps<'_>, file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<()> {
    let mut rest: &[u8] = bytes;
    while !rest.is_empty() {
        let taken: usize = match (steps.write)(file, rest) {
            Ok(count) => count,
            // A signal that arrives during a write is not a failure; `write_all` tries again.
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        if taken == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::WriteZero));
        }
        rest = &rest[taken..];
    }
    return Ok(());
}

/// What:
///  Write `bytes` to a new file of the mode Git gives an index,
/// give it `times`,
/// flush it,
/// and rename it over `target`,
/// through `steps`.
/// Why:
///   The file is created exclusively,
/// so it is this run's and only this run removes it;
/// its mode is set before any byte is
///  written;
/// a failure at any step removes it and leaves `target` as it was.
///  The times are set before the one flush,
/// so they are on the disk with the bytes:
/// an index installed as the live index must be as old as the stat data its entries describe,
///  and a second flush spent on that metadata would be the landing's cost for nothing
/// (`service_land_copy.rs`,
/// `keep_the_live_times`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await publishWith(steps, target, bytes, shared, times); // throws; the target is the old index or the whole new one
/// ```
///
/// # Errors
///
/// The input and output error of the step that failed.
pub fn publish_with(steps: &Steps<'_>, target: &Path, bytes: &[u8], shared: i32, times: Option<(std::time::SystemTime, std::time::SystemTime)>) -> std::io::Result<()> {
    let (mut file, temporary) = create_beside(steps, target, shared)?;
    // Each step runs only when the one before it succeeded, as `and_then` did.
    let mut written: std::io::Result<()> = write_all_with(steps, &mut file, bytes);
    if written.is_ok() {
        written = set_times(&file, times);
    }
    if written.is_ok() {
        written = (steps.sync)(&file);
    }
    if written.is_ok() {
        written = (steps.rename)(&temporary, target);
    }
    if written.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return written;
}

/// What:
///  Give one written file the times the caller named,
///  and nothing for a caller that named none.
/// Why:
///  A form that needs no times is one call site's `None`,
/// and the one that does is the landing's copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const touched = (file, times) => times === undefined ? undefined : file.utimes(times.atime, times.mtime)
/// ```
fn set_times(file: &std::fs::File, times: Option<(std::time::SystemTime, std::time::SystemTime)>) -> std::io::Result<()> {
    let Some((accessed, modified)) = times else { return Ok(()) };
    return file.set_times(std::fs::FileTimes::new().set_accessed(accessed).set_modified(modified));
}

/// `publish_with` the operating system's calls.
///
/// # Errors
///
/// As `publish_with`.
pub fn publish(target: &Path, bytes: &[u8], shared: i32, times: Option<(std::time::SystemTime, std::time::SystemTime)>) -> std::io::Result<()> {
    return publish_with(&SYSTEM, target, bytes, shared, times);
}

/// Controls for the publication step stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_publish_tests.rs"]
mod tests;
