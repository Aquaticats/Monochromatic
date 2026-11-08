//! What:
//! The probe that takes a Cargo profile directory's own build locks without waiting,
//! and tells whether anyone is queued behind them.
//! Why:
//! Cargo's builds hold `<profile>/.cargo-build-lock` exclusively with `flock` (the lock a container's cargo holds through a
//! bind mount too,
//! probed 2026-11-04 in `doc/planning/lean-build-dirs.md`),
//! so a sweep that holds the same lock excludes every
//! cooperating build of that directory.
//! The attempt never waits ("nothing should ever wait forever or indefinitely"):
//! a busy lock,
//! a file system that cannot lock and a queued waiter all mean the directory is skipped and reported.
//! The lock files are never renamed or unlinked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function lockProfileDirectory(profile: string): { held: Lock[] } | { busy: string } | { unlockable: string } | 'none'
//! ```

/// Files and paths.
use std::{fs, fs::File, path::{Path, PathBuf}};

/// The non-blocking exclusive `flock`,
/// and the device number split that `/proc/locks` prints.
use rustix::fs::{FlockOperation, flock, major, minor};

/// Device and inode of the lock files.
use crate::lean_identity::{Identity, identity_of};

/// The lock files Cargo keeps in a profile directory;
/// every one that exists is taken,
/// so the finer-grained locking of `-Zfine-grain-locking` is excluded as well.
pub const LOCK_FILE_NAMES: [&str; 3] = [".cargo-build-lock", ".cargo-artifact-lock", ".cargo-lock"];

/// The kernel's table of file locks,
/// where a queued waiter is a line with an arrow.
pub const KERNEL_LOCK_TABLE: &str = "/proc/locks";

/// The exclusive locks of one profile directory,
/// held until this value drops.
#[derive(Debug)]
pub struct HeldLocks {
    /// The open files that carry the locks.
    files: Vec<File>,
    /// Their paths,
    /// for the proof text.
    names: Vec<PathBuf>,
}

/// Reading the held locks.
impl HeldLocks {
    /// The paths of the locked files.
    pub fn names(&self) -> &[PathBuf] {
        return &self.names;
    }
}

/// What the probe found.
#[derive(Debug)]
pub enum LockOutcome {
    /// Every lock file was taken exclusively and no waiter is queued behind it.
    Held(
        /// The held locks.
        HeldLocks,
    ),
    /// Someone else holds this lock file:
    /// a build is running.
    Busy(
        /// The lock file that is held.
        PathBuf,
    ),
    /// A process is queued behind a lock this probe took,
    /// so a build is about to start.
    Waiting(
        /// The lock file with a waiter.
        PathBuf,
    ),
    /// The file system refuses `flock` here (Cargo itself skips locking on some network file systems).
    Unlockable {
        /// The file.
        file: PathBuf,
        /// The system error.
        reason: String,
    },
    /// The directory has none of the lock files,
    /// so nothing proves it is quiescent.
    NoLockFile,
}

/// What:
/// `pub fn waiter_lines_name(table: &str, device: u64, inode: u64) -> bool` says whether the kernel lock table shows a
/// queued waiter (a `->` line) on the file with this device and inode.
/// Why:
/// A build that already opened the lock file and waits for it would,
/// once this probe lets go,
/// build into paths that were
/// moved away;
/// seeing the queue first avoids that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const waiterLinesName = (table: string, device: bigint, inode: bigint): boolean => table.split('\n').some((l) => l.includes('->') && l.includes(`${hex(major(device))}:${hex(minor(device))}:${inode}`));
/// ```
pub fn waiter_lines_name(table: &str, device: u64, inode: u64) -> bool {
    let needle: String = format!("{:02x}:{:02x}:{}", major(device), minor(device), inode);
    for line in table.lines() {
        if !line.contains("->") {
            continue;
        }
        for field in line.split_whitespace() {
            if field == needle {
                return true;
            }
        }
    }
    return false;
}

/// What:
/// `pub fn lock_profile_directory(profile: &Path, lock_table: &Path) -> LockOutcome` takes every Cargo lock file of the
/// directory exclusively without waiting.
/// Why:
/// It is the first proof of "not in use":
/// while it holds,
/// no cooperating cargo builds in this profile directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lockProfileDirectory(profile: string, lockTable: string): LockOutcome
/// ```
pub fn lock_profile_directory(profile: &Path, lock_table: &Path) -> LockOutcome {
    let mut files: Vec<File> = Vec::new();
    let mut names: Vec<PathBuf> = Vec::new();
    for name in LOCK_FILE_NAMES {
        let path: PathBuf = profile.join(name);
        let file: File = match File::open(&path) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return LockOutcome::Unlockable { file: path, reason: error.to_string() },
        };
        match flock(&file, FlockOperation::NonBlockingLockExclusive) {
            Ok(()) => {}
            Err(code) if code == rustix::io::Errno::WOULDBLOCK => return LockOutcome::Busy(path),
            Err(code) => return LockOutcome::Unlockable { file: path, reason: code.to_string() },
        }
        files.push(file);
        names.push(path);
    }
    if names.is_empty() {
        return LockOutcome::NoLockFile;
    }
    let table: String = fs::read_to_string(lock_table).unwrap_or_default();
    for path in &names {
        if let Ok(Identity { device, inode }) = identity_of(path)
            && waiter_lines_name(&table, device, inode)
        {
            return LockOutcome::Waiting(path.clone());
        }
    }
    return LockOutcome::Held(HeldLocks { files, names });
}

/// What:
/// `pub fn describe_held(held: &HeldLocks) -> String` is the proof text of a taken lock.
/// Why:
/// Each removal's log line records why the directory counted as not in use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const describeHeld = (held: HeldLocks): string => `exclusive flock on ${held.names.join(', ')}, no waiter queued`;
/// ```
pub fn describe_held(held: &HeldLocks) -> String {
    let mut shown: Vec<String> = Vec::new();
    for name in held.names() {
        shown.push(name.display().to_string());
    }
    return format!("exclusive flock on {} ({} files, no waiter queued in {})", shown.join(", "), held.files.len(), KERNEL_LOCK_TABLE);
}
