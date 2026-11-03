//! What:
//!  How the in-process index update gets its result onto the landing's private index: proof that the landing holds the live
//!  index lock, a temporary file no other run can share, Git's file mode for an index, and the rename.
//! Why:
//!  The update replaces `index.cli-git-<batch>`, the copy of the live index the landing made under its `index.lock`; the
//!  lock's release then renames that copy over the live index (`service_index_hold.rs`, `IndexLock::release_with`). A run
//!  that is not that landing could replace an index a foreign `git add` is staging into, so the update refuses to run unless
//!  the live lock names this process and this batch. Two runs on one target must not share a temporary name: each creates its
//!  own file exclusively and removes only that one. Git creates an index through a lock file whose mode `adjust_shared_perm`
//!  widens by `core.sharedRepository` (`tempfile.c`, `path.c`), so the file gets the same mode before it is published.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!holdsLock(landing)) return false; await publish(landing.built, bytes, sharedRepository);
//! ```

use super::service_index_lock::lock_path_of;
use super::service_index_marker::{marker_of, own_identity};
use std::io::Write as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The private index of one landing and the live index it was copied from.
#[derive(Clone, Copy, Debug)]
pub struct Landing<'a> {
    /// The live index, whose `.lock` the landing holds.
    pub live: &'a Path,
    /// The landing's private copy, in the live index's directory.
    pub built: &'a Path,
    /// The batch number the lock's marker names.
    pub seq: u64,
}

/// How many times a temporary name is tried before the update gives up.
const NAME_TRIES: usize = 16;

/// A count that makes the temporary names of one process differ.
static NEXT_NAME: AtomicU64 = AtomicU64::new(0);

/// What:
///  Whether the live index lock is held by this process for this batch and the private index is a different file in the
///  live index's directory.
/// Why:
///   The marker is written by the landing when it takes the lock (`service_index_prestage.rs`) and names the batch, the process
///  and its birth identity, so a lock of another program, of another batch, or of a dead service does not match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours: boolean = landing.built !== landing.live && sameDirectory && lockMarker(landing.live) === ourMarker(landing.seq);
/// ```
pub fn holds_the_lock(landing: &Landing<'_>) -> bool {
    if landing.built == landing.live || landing.built.parent() != landing.live.parent() {
        return false;
    }
    let Some(marked) = marker_of(lock_path_of(landing.live).as_path()) else {
        return false;
    };
    let Ok(birth) = own_identity() else {
        return false;
    };
    return marked.seq == landing.seq && marked.holder == Some((i64::from(std::process::id()), birth));
}

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

/// Write `bytes` into the new file, give it the mode Git gives an index, and flush it (`CSUM_FSYNC`).
fn fill_and_flush(file: &mut std::fs::File, bytes: &[u8], shared: i32) -> std::io::Result<()> {
    file.write_all(bytes)?;
    if shared != 0 {
        let old: u32 = file.metadata()?.permissions().mode();
        let new: u32 = shared_mode(shared, old);
        if (new ^ old) & 0o7777 != 0 {
            file.set_permissions(std::fs::Permissions::from_mode(new & 0o7777))?;
        }
    }
    return file.sync_all();
}

/// What:
///  Write `bytes` to a new file of the mode Git gives an index, flush it, and rename it over `target`.
/// Why:
///   The file is created exclusively, so it is this run's and only this run removes it; a failure at any step removes it and
///  leaves `target` as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await publish(target, bytes, shared); // throws; the target is the old index or the whole new one
/// ```
///
/// # Errors
///
/// The input and output error of the step that failed.
pub fn publish(target: &Path, bytes: &[u8], shared: i32) -> std::io::Result<()> {
    for _ in 0..NAME_TRIES {
        let Some(temporary) = temporary_beside(target) else {
            break;
        };
        let mut file: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary) {
            Ok(created) => created,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        };
        let written: std::io::Result<()> = fill_and_flush(&mut file, bytes, shared).and_then(|()| { return std::fs::rename(&temporary, target); });
        if written.is_err() {
            let _removed = std::fs::remove_file(&temporary);
        }
        return written;
    }
    return Err(std::io::Error::other("no unused temporary name beside the index"));
}

/// Controls for the publication step stay out of the release executable.
#[cfg(test)]
#[path = "index_reset_publish_tests.rs"]
mod tests;
