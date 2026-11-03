//! What:
//!  How the in-process index update gets its result onto the landing's private index: the name of that index, a temporary file
//!  no other run can share, Git's file mode for an index, and the rename.
//! Why:
//!  The update replaces `index.cli-git-<batch>`, the copy of the live index the landing made under its `index.lock`; the
//!  lock's release then renames that copy over the live index (`service_index_hold.rs`, `IndexLock::release_with`). The update
//!  takes a borrow of the held `IndexLock`, so only the holder can call it, and it works on the one name `private_index_path`
//!  gives for that lock's index and batch. Two runs on one target must not share a temporary name: each creates its
//!  own file exclusively and removes only that one. Git creates an index through a lock file whose mode `adjust_shared_perm`
//!  widens by `core.sharedRepository` (`tempfile.c`, `path.c`), so the file gets the same mode before it is published.
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

/// A file name beside `target` that no other run uses: the process, a count and the clock.
fn temporary_beside(target: &Path) -> Option<PathBuf> {
    let count: u64 = NEXT_NAME.fetch_add(1, Ordering::Relaxed);
    let clock: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
    return Some(target.with_file_name(format!("{}.reset-{}-{count}-{clock:x}", target.file_name()?.to_string_lossy(), std::process::id())));
}

/// The steps of a publication that touch the file system, so a control can make any one of them fail or run short.
pub struct Steps<'a> {
    /// One write of some of the bytes; returns how many were taken.
    pub write: &'a dyn Fn(&mut std::fs::File, &[u8]) -> std::io::Result<usize>,
    /// The change of the open file's permission bits.
    pub chmod: &'a dyn Fn(&std::fs::File, u32) -> std::io::Result<()>,
    /// The flush of the file to the disk.
    pub sync: &'a dyn Fn(&std::fs::File) -> std::io::Result<()>,
    /// The rename of the finished file over the target.
    pub rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// The process's file creation mask, which Git's lock file is created under.
    pub umask: &'a dyn Fn() -> std::io::Result<u32>,
}

/// The process's umask, read from `/proc/self/status` (Linux): reading it never changes it, which `umask(2)` cannot promise
/// while other threads create files.
fn process_umask() -> std::io::Result<u32> {
    let status: String = std::fs::read_to_string("/proc/self/status")?;
    let line: &str = status.lines().find_map(|line| { return line.strip_prefix("Umask:"); }).ok_or_else(|| { return std::io::Error::other("no Umask line in /proc/self/status"); })?;
    return u32::from_str_radix(line.trim(), 8).map_err(std::io::Error::other);
}

/// One write to the file.
fn write_some(file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<usize> {
    return file.write(bytes);
}

/// The file's permission bits set with `fchmod`.
fn set_mode(file: &std::fs::File, mode: u32) -> std::io::Result<()> {
    return file.set_permissions(std::fs::Permissions::from_mode(mode));
}

/// The file flushed to the disk, data and metadata (`CSUM_FSYNC`).
fn sync_all(file: &std::fs::File) -> std::io::Result<()> {
    return file.sync_all();
}

/// The rename of the finished file over the target.
fn rename_file(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::rename(from, to);
}

/// The operating system's own calls.
pub const SYSTEM: Steps<'static> = Steps { write: &write_some, chmod: &set_mode, sync: &sync_all, rename: &rename_file, umask: &process_umask };

/// Create the temporary file with the permissions Git's index lock file has before a byte is written to it: Git creates it
/// under the umask and `adjust_shared_perm` (`tempfile.c:152`) changes it at once, so a `core.sharedRepository` value that is
/// narrower than the umask never lets the bytes be read more widely. With a shared repository the file is created with the owner's
/// bits only and then set to the mode `calc_shared_perm` makes of what Git's file would have had.
fn create_beside(steps: &Steps<'_>, target: &Path, shared: i32) -> std::io::Result<(std::fs::File, PathBuf)> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let open_mode: u32 = if shared != 0 { 0o600 } else { 0o666 };
    for _ in 0..NAME_TRIES {
        let Some(temporary) = temporary_beside(target) else {
            break;
        };
        let file: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).mode(open_mode).open(&temporary) {
            Ok(created) => created,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        if shared != 0 {
            let gits_file: u32 = 0o666 & !(steps.umask)().inspect_err(|_| { let _removed = std::fs::remove_file(&temporary); })?;
            if let Err(error) = (steps.chmod)(&file, shared_mode(shared, gits_file) & 0o7777) {
                let _removed = std::fs::remove_file(&temporary);
                return Err(error);
            }
        }
        return Ok((file, temporary));
    }
    return Err(std::io::Error::other("no unused temporary name beside the index"));
}

/// Write all of `bytes`, however the writes are cut.
fn write_all_with(steps: &Steps<'_>, file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<()> {
    let mut rest: &[u8] = bytes;
    while !rest.is_empty() {
        let taken: usize = (steps.write)(file, rest)?;
        if taken == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::WriteZero));
        }
        rest = &rest[taken..];
    }
    return Ok(());
}

/// What:
///  Write `bytes` to a new file of the mode Git gives an index, flush it, and rename it over `target`, through `steps`.
/// Why:
///   The file is created exclusively, so it is this run's and only this run removes it; its mode is set before any byte is
///  written; a failure at any step removes it and leaves `target` as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await publishWith(steps, target, bytes, shared); // throws; the target is the old index or the whole new one
/// ```
///
/// # Errors
///
/// The input and output error of the step that failed.
pub fn publish_with(steps: &Steps<'_>, target: &Path, bytes: &[u8], shared: i32) -> std::io::Result<()> {
    let (mut file, temporary) = create_beside(steps, target, shared)?;
    let written: std::io::Result<()> = write_all_with(steps, &mut file, bytes).and_then(|()| { return (steps.sync)(&file); }).and_then(|()| { return (steps.rename)(&temporary, target); });
    if written.is_err() {
        let _removed = std::fs::remove_file(&temporary);
    }
    return written;
}

/// `publish_with` the operating system's calls.
///
/// # Errors
///
/// As `publish_with`.
pub fn publish(target: &Path, bytes: &[u8], shared: i32) -> std::io::Result<()> {
    return publish_with(&SYSTEM, target, bytes, shared);
}

/// Controls for the publication step stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_publish_tests.rs"]
mod tests;
