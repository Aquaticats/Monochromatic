//! What:
//!  The words and size bounds the toolchain repair reads,
//!  starts and prints with:
//!  the launcher directory and the record beside each launcher,
//!  the lock file's name and the key that names it in a record,
//!  the program an install runs and the variable that names its root,
//!  the worktree's own link to that program,
//!  the tag every line of this module's diagnostic log carries,
//!  the two byte bounds a lock and a record are read at,
//!  and the slots a control stands a concurrent writer or a kill in.
//! Why:
//!  `package_repair_toolchain.rs` would otherwise pass the file-size limit,
//!  and every other file of this module reads these names,
//!  so they live in the one file the rest may use without any file using it back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const LOCK_FILE = 'toolchain.lock.json';
//! ```

/// Import the one size every file this repair reads is bounded by.
use crate::plain_file::SMALL_FILE_LIMIT;
/// The bytes of a `toolchain.lock.json` this repair reads,
/// past which the lock is no lock it can read.
///  Measured on this repository on 2026-11-13:
/// its lock is 63,625 bytes,
/// so this bound is sixteen times it,
/// and a lock that large would declare some three thousand tools.
///  A longer lock is refused rather than read,
/// and a refusal is the answer an unreadable lock already gets:
/// every launcher record stays counted.
pub(super) const LONGEST_LOCK: u64 = SMALL_FILE_LIMIT;

/// The bytes of a launcher record this release reads,
/// past which the file at that name is no record it can read.
///  A record is one JSON object holding a tool's name,
/// its scope and two paths,
/// and every path the kernel accepts is at most 4,096 bytes (`getconf PATH_MAX /`,
/// measured 2026-11-25),
/// so a record this release writes stays some hundred bytes
/// and the longest one a kernel can hand a writer stays under ten kilobytes.
///  The bound is `SMALL_FILE_LIMIT`,
/// the one the lock and a registration's `gitdir` are read at,
/// because the record is read the way those are;
/// a file past it is refused rather than read into memory,
/// and a refusal leaves the record alone,
/// which is what a record this release cannot read already gets.
pub(super) const LONGEST_RECORD: u64 = SMALL_FILE_LIMIT;

/// Tag used for diagnostics from this module.
pub(super) const TAG: &str = "packageRepairToolchain";

// What a control writes inside the removal pass, taken out when it runs so it runs once, and absent in a release build;
// and a control's own kill at the one instruction a record is out of its place.
#[cfg(test)]
thread_local! {
    /// The writer a control stands in the classification-to-removal window with.
    pub(crate) static WHEN_THE_REMOVAL_BEGINS: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
    /// Whether this thread's removal stops once a record is aside,
    /// which no release build can do.
    pub(crate) static KILLED_AFTER_THE_RECORD_IS_ASIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The replacement a control makes in the window between a record's name being looked at and its open,
    /// which no release build has.
    pub(crate) static WHEN_THE_RECORD_IS_OPENED: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

/// Where mpm-shadow keeps the launchers and their records,
/// relative to a worktree.
pub(super) const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// The record that sits beside a launcher,
/// named after it.
pub(super) const RECORD_EXTENSION: &str = "launch";

/// The lock file a launcher record names,
/// relative to a worktree.
pub(super) const LOCK_FILE: &str = "toolchain.lock.json";

/// The key of a launcher record that names the lock file.
pub(super) const LOCK_KEY: &str = "lockFile";

/// The program that owns the toolchain,
/// and where a worktree keeps a link to it.
pub(super) const PROGRAM: &str = "mpm-shadow";

/// Where the worktree's own link to the program sits when the search path does not provide one.
pub(super) const OWN_LINK: &str = ".cache/toolchain-bin/mpm-shadow";

/// The variable that names the worktree whose toolchain a command acts on.
pub(super) const ROOT_VARIABLE: &str = "MONOCHROMATIC_WORKTREE_ROOT";
