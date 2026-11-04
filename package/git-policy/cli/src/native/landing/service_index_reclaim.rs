//! What:
//!  The removal of a stale index lock by one reclaimer at a time: the reclaimer opens the lock it will judge, reads the marker
//!  from that open file, takes a non-blocking exclusive `flock` on it, checks that the name still refers to that file, and only
//!  then unlinks the name.
//! Why:
//!  A removal by name removes whatever the name holds by then. Between a look at the marker and the removal, the stale lock may
//!  be gone and a live lock in its place; that happens when a second reclaimer (another instance serving the same physical
//!  index, an alternate index shared by two worktrees) removed the stale lock and placed its own. The earlier cure took the
//!  file aside by a rename and put it back by a hard link, which left the live lock off its name for a while, in which Git could
//!  take the name and strand it. Here no file is moved: the `flock` is held on the very file that was judged, so a second
//!  reclaimer of that file backs off, and a reclaimer that opened it before it was removed finds the name no longer refers to
//!  it. A live landing cannot place a lock while the name is occupied. What stays open is a person's `rm index.lock` between
//!  the check and the unlink, because no system call unlinks a name only if it still names a given file
//!  (`doc/planning/cli-git-one-reclaimer-per-index.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reclaimIf(platform, lock, isStale) // open; judge; flock(NB) ?? busy; sameFile(lock, fd) ? unlink(lock) : changed
//! ```

use crate::service_index_lock::lock_path_of;
use crate::service_index_marker::{Marked, holder_is_gone, marker_of_file};
use crate::service_index_placement::{Platform, SYSTEM};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// What a reclaimer made of the file at a lock's name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reclaimed {
    /// No lock is left in the way: the stale file was removed, or was already gone.
    Removed,
    /// The name no longer refers to the file that was judged stale: the placement is worth another try.
    Changed,
    /// Another reclaimer holds the stale file and is removing it: the lock is not this reclaimer's to remove.
    Busy,
    /// The file is not a stale lock of the service (a live holder's, Git's, or not readable): it is left where it is.
    Kept,
    /// The file is stale and cannot be removed; the log says why.
    Stuck,
}

/// What:
///  Removes the index lock of `index` left by a service that was killed, when it still carries the service's marker for `seq`
///  and its holder is gone.
/// Why:
///   Only the marker proves the lock is ours; a foreign lock, one a Git holds, or one whose holder still runs is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    let for_this_batch = |marked: &Marked| -> bool { return marked.seq == seq && holder_is_gone(marked.holder.as_ref()); };
    let _outcome: Reclaimed = reclaim_if(&SYSTEM, lock.as_path(), &for_this_batch);
}

/// What:
///  Removes the file at `lock` if `is_stale` says the marker in it is a stale one, while no other reclaimer holds the file.
/// Why:
///   One reclaimer per file at a time is what keeps a live lock another instance placed after the marker was read from being
///  removed. The answer says what the caller does next: place again after `Removed` and `Changed`, wait as for any held lock
///  after `Busy` and `Kept`, and give up after `Stuck`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = reclaimIf(platform, lock, isStale) // 'removed' | 'changed' | 'busy' | 'kept' | 'stuck'
/// ```
pub fn reclaim_if(platform: &Platform, lock: &Path, is_stale: &dyn Fn(&Marked) -> bool) -> Reclaimed {
    let file: std::fs::File = match open_to_judge(lock) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Reclaimed::Removed,
        Err(_) => return Reclaimed::Kept,
    };
    // A control reaches here to act as another program would while the reclaimer looks at the file.
    crate::service_fault::reach("reclaim-opened");
    let Some(marked) = marker_of_file(&file) else {
        return Reclaimed::Kept;
    };
    if !is_stale(&marked) {
        return Reclaimed::Kept;
    }
    crate::service_fault::reach("reclaim-judged");
    match hold(platform, &file) {
        Hold::Taken => {}
        Hold::Busy => return Reclaimed::Busy,
        Hold::Unavailable(error) => {
            crate::service_error!("the index lock {} is removed without the reclaim hold ({error}): this file system gives no flock, so two services reclaiming one physical index at once are not kept apart", lock.display());
        }
    }
    crate::service_fault::reach("reclaim-held");
    match same_file(lock, &file) {
        Ok(true) => {}
        Ok(false) => return Reclaimed::Changed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Reclaimed::Removed,
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be looked at ({error})", lock.display());
            return Reclaimed::Stuck;
        }
    }
    return match (platform.unlink)(lock) {
        Ok(()) => Reclaimed::Removed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Reclaimed::Removed,
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be removed ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            Reclaimed::Stuck
        }
    };
}

/// Opens `lock` to read its marker without following a link or waiting on a pipe, and without a descriptor a child could inherit.
fn open_to_judge(lock: &Path) -> std::io::Result<std::fs::File> {
    return std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(lock);
}

/// What taking the reclaim hold on a file came to.
enum Hold {
    /// This reclaimer holds the file until its descriptor is closed, or the process ends.
    Taken,
    /// Another reclaimer holds it.
    Busy,
    /// The file system gives no `flock`, or the call failed: the reclaim goes on unguarded.
    Unavailable(std::io::Error),
}

/// What:
///  Takes the reclaim hold on `file` through the platform's `flock`, without waiting.
/// Why:
///   Two reclaimers of one stale lock must not both remove a name; the reason is written beside the call in `flock_exclusive`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hold = tryFlock(file) // 'taken' | 'busy' | { unavailable: error }
/// ```
fn hold(platform: &Platform, file: &std::fs::File) -> Hold {
    return match (platform.flock)(file) {
        Ok(()) => Hold::Taken,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Hold::Busy,
        Err(error) => Hold::Unavailable(error),
    };
}

/// What:
///  The system's exclusive `flock` on `file`, without waiting; `WouldBlock` when another descriptor holds it.
/// Why:
///   The one place the lock is taken, so a control can stand in a file system that gives none (`Platform::flock`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// flockExclusive(file) // throws WouldBlock when held
/// ```
pub fn flock_exclusive(file: &std::fs::File) -> std::io::Result<()> {
    // Lock kept: two reclaimers of one stale lock must not both remove a name, because the second removal would remove what a
    //  live landing placed after the first. A design without a lock needs an atomic conditional unlink, which Linux does not have
    //  (a rename aside leaves a live lock off its name for a while, in which Git can take the name). The hold is non-blocking,
    //  is on the one file being reclaimed (the smallest unit there is, never the repository or the directory), is kept for a few
    //  system calls, and is dropped by the kernel when the descriptor closes or the process dies, so no leftover can block a
    //  reclaim.
    // SAFETY: `flock` takes an open descriptor, which `file` keeps open for the call.
    let result: libc::c_int = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    return if result == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

/// Whether the name `lock` refers to the very file `file` has open: the same device and the same inode, so a replaced file whose
///  inode number was reused cannot pass while `file` keeps the old one alive.
fn same_file(lock: &Path, file: &std::fs::File) -> std::io::Result<bool> {
    let named = std::fs::symlink_metadata(lock)?;
    let open = file.metadata()?;
    return Ok(named.dev() == open.dev() && named.ino() == open.ino());
}
