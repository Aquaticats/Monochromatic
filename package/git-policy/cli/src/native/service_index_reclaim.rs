//! What:
//!  The removal of a stale index lock: taken aside by one rename, looked at again there, and removed only if it is still the stale
//!  one.
//! Why:
//!  A removal by name removes whatever the name holds by then, so a lock another program placed after the marker was read would
//!  be lost. The module is split from `service_index_placement.rs` and `service_index_lock.rs`, which re-export its functions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reclaimIf(platform, lock, stillStale) // rename to aside; stillStale(aside) ? unlink(aside) : link(aside, lock)
//! ```

use super::service_index_lock::lock_path_of;
use super::service_index_marker::{holder_is_gone, marker_of};
use super::service_index_placement::{Platform, SYSTEM};
use std::path::{Path, PathBuf};

/// What:
///  Removes an index lock left by a service that was killed, when it still carries the service's marker for `seq` and its
///  holder is gone.
/// Why:
///   Only the marker proves the lock is ours; a foreign lock, one a Git holds, or one whose holder still runs is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    let stale = |path: &Path| -> bool { return marker_of(path).is_some_and(|marked| { return marked.seq == seq && holder_is_gone(marked.holder.as_ref()); }); };
    if stale(lock.as_path()) {
        crate::service_fault::reach("reclaim-checked");
        // The lock is taken aside by one rename and looked at again, so a lock another program placed after the look above is
        //  put back and never removed: removing by name would remove whatever the name holds by then.
        let _removed = reclaim_if(&SYSTEM, lock.as_path(), &stale);
    }
}

/// What:
///  Removes the file at `lock` only if `still_stale` says the file taken from that name is the stale one; true when no lock is
///  left in the way.
/// Why:
///   A removal by name removes whatever the name holds by then: between a look at the marker and the removal, the stale lock
///  may be gone and a foreign program's live lock in its place (a second instance serving the same physical index, or Git), and
///  the live lock would be removed. The file is taken aside by one rename, which takes exactly one file, looked at again
///  there, removed if it is the stale one, and otherwise put back by a hard link that fails rather than replace a lock
///  placed meanwhile. A lock that could not be put back stays at the name it was taken to, and the log says where.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reclaimIf(platform, lock, stillStale) // rename to aside; stillStale(aside) ? unlink(aside) : link(aside, lock)
/// ```
pub fn reclaim_if(platform: &Platform, lock: &Path, still_stale: &dyn Fn(&Path) -> bool) -> bool {
    let Some(aside) = aside_of(lock) else {
        crate::service_error!("the index lock {} cannot be taken aside: it has no file name", lock.display());
        return false;
    };
    match std::fs::rename(lock, &aside) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return true,
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be taken aside ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            return false;
        }
    }
    if !still_stale(aside.as_path()) {
        put_back(aside.as_path(), lock);
        return false;
    }
    return match (platform.unlink)(&aside) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be removed ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            put_back(aside.as_path(), lock);
            false
        }
    };
}

/// A name beside `lock` that no other run uses: the process and the clock.
fn aside_of(lock: &Path) -> Option<std::path::PathBuf> {
    let clock: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
    return Some(lock.with_file_name(format!("{}.aside-{}-{clock:x}", lock.file_name()?.to_string_lossy(), std::process::id())));
}

/// Puts the file taken aside back under the lock's name, without replacing a file that has the name by now.
fn put_back(aside: &Path, lock: &Path) {
    match std::fs::hard_link(aside, lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(aside);
        }
        Err(error) => {
            crate::service_error!("the index lock {} was taken aside to {} and could not be put back ({error}); the file at the second name is that lock, and it is not removed",
                lock.display(),
                aside.display()
            );
        }
    }
}
