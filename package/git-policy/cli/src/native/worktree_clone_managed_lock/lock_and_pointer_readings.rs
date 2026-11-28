//! What:
//!  The two reads the managed Git lock's states rest on:
//!  a registration's `locked` file,
//!  and a slot's `.git` pointer.
//! Why:
//!  The lock's own bound is the length of the sentence the add writes plus Git's newline,
//!  computed from the sentence's parts rather than measured
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  `L7` and "What proves ownership"),
//!  and the slot's pointer is read with the bound the clone mount route already uses,
//!  so one file's bound is stated once in this package.
//!  Both names are Git's own,
//!  and both reads answer absence as absence,
//!  which is what tells the states S4 and S5 from a refused read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lock = readTheLock(registration); // the `locked` file
//! const pointer = readTheSlotPointer(slot); // the slot's `.git`
//! ```

use super::reading_a_bounded_file::{
    Reading,
    read_a_bounded_file,
};
use crate::worktree_clone_binding_record::lock_words::LOCK_WORDS_MAXIMUM_BYTES;
use crate::worktree_clone_mount::clone_slot_git_pointer::POINTER_BOUND;
use std::path::{
    Path,
    PathBuf,
};

/// Git's name for a registration's lock file,
/// which is the whole of Git's lock state (`F1` of the design's facts).
pub const LOCK_FILE_NAME: &str = "locked";

/// Git's name for the file inside a worktree that names its registration.
pub const SLOT_POINTER_NAME: &str = ".git";

/// The most bytes this route reads at a slot's `.git`,
/// which is the clone mount route's own bound.
pub const SLOT_POINTER_MAXIMUM_BYTES: u64 = POINTER_BOUND as u64;

/// The path of one registration's lock file.
pub fn lock_file_path(registration: &Path) -> PathBuf {
    return registration.join(LOCK_FILE_NAME);
}

/// The path of one slot's `.git` file.
pub fn slot_pointer_path(slot: &Path) -> PathBuf {
    return slot.join(SLOT_POINTER_NAME);
}

/// What:
///  Read one registration's lock file,
///  as the bytes Git's own write left or as why the read refused them.
/// Why:
///  The bytes are compared as they stand on disk and never as words Git printed
///  (`L5`),
/// and a file wider than the sentence the add writes is no lock this add wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readABoundedFile(join(registration, 'locked'), LOCK_WORDS_MAXIMUM_BYTES);
/// ```
pub fn read_the_lock(registration: &Path) -> Reading {
    return read_a_bounded_file(
        lock_file_path(registration).as_path(),
        LOCK_WORDS_MAXIMUM_BYTES,
    );
}

/// What:
///  Read one slot's `.git`,
///  as the bytes Git wrote or as why the read refused them.
/// Why:
///  The pointer's presence is the fact that decides whether `git worktree prune` falls through to the
///  registration's index and the expiry (`F5`),
/// and its absence is the state S5,
///  which stands beside the lock's own state rather than instead of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = readABoundedFile(join(slot, '.git'), SLOT_POINTER_MAXIMUM_BYTES);
/// ```
pub fn read_the_slot_pointer(slot: &Path) -> Reading {
    return read_a_bounded_file(
        slot_pointer_path(slot).as_path(),
        SLOT_POINTER_MAXIMUM_BYTES,
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_and_pointer_readings_tests.rs"]
mod tests;
