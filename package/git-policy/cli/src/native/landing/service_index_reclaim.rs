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
use crate::service_index_claim::{Claim, ClaimHold, take_claim};
use crate::service_index_marker::{LOCK_MARKER, Marked, holder_is_gone, marker_of_file};
use crate::service_index_placement::{Platform, SYSTEM};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// What a reclaimer made of the file at a lock's name.
#[derive(Clone, Debug, Eq, PartialEq)]
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
    /// The file is stale, but no exclusion for its removal could be had, neither the `flock` nor a claim file (the file system
    ///  gives no `flock` and refuses to place the claim): the file is left where it is, and the person is told how to remove it.
    ///  It carries the marker read from the very file that was judged, so the notice never describes a file that replaced it.
    Unguarded(Marked),
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
    // The exclusion this reclaimer is under until it is done: the `flock` on the file (held by the descriptor), or, where the
    //  file system gives none, the claim chain beside the lock.
    let mut claim: Option<ClaimHold> = None;
    match hold(platform, &file) {
        Hold::Taken => {}
        Hold::Busy => return Reclaimed::Busy,
        Hold::Unavailable(error) => match take_claim(platform, lock, &file) {
            Claim::Held(held) => {
                crate::service_log!("the index lock {} is reclaimed under the claim chain, because the file system gives no flock on it ({error})", lock.display());
                claim = Some(held);
            }
            Claim::Busy => return Reclaimed::Busy,
            Claim::Unavailable(why) => {
                crate::service_error!("the index lock {} was left by a service that is gone, and it is not removed: neither the flock ({error}) nor a claim file ({why}) \
                     could be had to keep two reclaimers apart, so a removal could take a live lock; remove it by hand once no git command runs in this repository",
                    lock.display()
                );
                return Reclaimed::Unguarded(marked);
            }
        },
    }
    crate::service_fault::reach("reclaim-held");
    match same_file(lock, &file) {
        Ok(true) => {}
        Ok(false) => return cleared(claim, Reclaimed::Changed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return cleared(claim, Reclaimed::Removed),
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be looked at ({error})", lock.display());
            return Reclaimed::Stuck;
        }
    }
    return match (platform.unlink)(lock) {
        Ok(()) => cleared(claim, Reclaimed::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => cleared(claim, Reclaimed::Removed),
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be removed ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            Reclaimed::Stuck
        }
    };
}

/// `answer`, after the claims of a reclaimer that saw the stale file leave its name are removed (a reclaimer under the `flock`
///  has none).
fn cleared(claim: Option<ClaimHold>, answer: Reclaimed) -> Reclaimed {
    if let Some(held) = claim {
        held.clear_all();
    }
    return answer;
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
    /// The file system gives no `flock`, or the call failed for a reason that waiting does not change: the claim chain takes
    ///  the `flock`'s place.
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
        // An interrupted call, and one that found no memory for the kernel's lock record (`ENOMEM` from `fs/locks.c`), are asked
        //  again by the wait, like a hold another reclaimer has taken: neither says the file system has no `flock`, and a
        //  reclaimer that took the claim path beside one that holds the `flock` would not be kept apart from it.
        Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted | std::io::ErrorKind::OutOfMemory) => Hold::Busy,
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

/// What:
///  The words for a stale lock that was not removed: which lock, which batch and process left it, how to tell it is still that
///  lock, and how to remove it.
/// Why:
///   Native Git leaves a stale `index.lock` to the person, and so does a service that cannot take the hold a safe removal needs;
///  the person must be able to act from the notice alone. `marked` is the marker read from the file that was judged, never a
///  second read of the name, which another landing may have taken since; the first line to look for lets the person see that
///  the file is still the one described.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The index lock ${lock} was left by cli-git-service batch ${seq} (process ${pid}, gone). ... remove it yourself: rm ${lock}`
/// ```
pub fn stale_lock_words(lock: &Path, marked: &Marked) -> String {
    let (who, first_line): (String, String) = match marked.holder.as_ref() {
        Some((pid, birth)) => (
            format!("cli-git-service batch {} (process {pid}, which is gone)", marked.seq),
            format!("{LOCK_MARKER}{} {pid} {birth}", marked.seq),
        ),
        None => (format!("cli-git-service batch {}", marked.seq), format!("{LOCK_MARKER}{}", marked.seq)),
    };
    return format!(
        "The index lock {} was left by {who} and is not removed here, because neither the file lock that makes its removal safe nor a claim file beside it could be had on this file system. \
         The commit waits as for any held lock. Once no git command and no commit of this service is running in this repository, and the first line of the file \
         still reads `{first_line}`, remove it yourself (`rm {}`).",
        lock.display(),
        lock.display()
    );
}
