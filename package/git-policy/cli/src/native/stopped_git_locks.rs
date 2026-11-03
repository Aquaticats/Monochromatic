//! What:
//!  The lock files a stopped Git child leaves behind, found and removed once the child and what it started are gone.
//! Why:
//!  Git removes its `.lock` files when it ends normally or on a catchable signal, but a child that is killed, or that is
//!  in an uninterruptible call when the stop arrives, cannot. On 2026-11-03 two pushes were stopped at their deadline;
//!  about 37 seconds later, in the same instant, empty `refs/remotes/origin/*.lock` files appeared for exactly their two
//!  tracking refs, with no pid file beside them, and every later `git fetch` of those refs failed with "Unable to create
//!  ... File exists". The service had started those pushes without `core.lockfilePid`, so no pid file was written. That
//!  the creators were the stopped pushes, released by the kernel late, is the inference that fits these facts; nothing
//!  observed the processes themselves.
//!  Two kinds of file are removed, and nothing else:
//!       a lock whose `~pid.lock` file (written by `core.lockfilePid`, which the wrapper turns on for every Git it starts)
//!       names a process of the stopped tree that has ended, with that pid file;
//!       an empty lock with no pid file, born after the child started and still empty after a settle time, because the
//!       child can die between creating the lock and writing the pid file, and an empty file holds nothing.
//!  A lock another Git process holds is never matched: its pid file names a process outside the stopped tree, or it
//!  has content, or it is older than the child. No lock of the wrapper's own is taken to do this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ledger = LockLedger.beforeStart(command); const tree = await stop(child);
//! for (const lock of ledger.sweep(tree)) log(`removed ${lock}`);
//! ```

/// The diagnostic log of the wrapper.
use super::diagnostic_log::warn;
/// The processes of the stopped tree and whether each still runs.
use super::process_tree::{TreeMember, still_running};
/// The Git directories a command works in.
use super::git_directory_roots::roots_of_command;
/// Owned OS strings, for the names of directories the walk skips.
use std::ffi::OsStr;
/// Paths of the repository.
use std::path::{Path, PathBuf};
/// The command whose children are stopped.
use std::process::Command;
/// Time spans and the wall clock the kernel stamps files with.
use std::time::{Duration, SystemTime};

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// What every Git lock file's name ends with.
pub(crate) const LOCK_SUFFIX: &str = ".lock";

/// What the pid file of a lock ends with: the lock's name without `.lock`, then this.
pub(crate) const PID_LOCK_SUFFIX: &str = "~pid.lock";

/// What a pid file's text starts with, before the number.
const PID_TEXT_PREFIX: &str = "pid ";

/// How far a file's birth may precede the clock reading taken before the child started.
///  The kernel stamps files from a coarse clock that lags the one `SystemTime` reads by up to one tick.
///  Measured on this host (`CONFIG_HZ=1000`, so a tick is 1 ms): a new file's birth time lagged a clock read taken just before
///  the create by at most 1.684 ms and 1.729 ms in two runs of 10,000 creates each, so 50 ms is about thirty times the
///  longest lag. Kept; recorded in `doc/planning/cli-git-shipped-limits-audit.md`, "Margins for removing left-behind locks"
///  (the coordinator's pick under the owner's delegation, open to the owner's veto).
pub(crate) const COARSE_CLOCK_SLACK: Duration = Duration::from_millis(50);

/// How long an empty lock without a pid file must have stayed empty before it counts as left behind.
///  Git creates a ref lock empty and fills it a moment later, so a lock that a live process just made is empty for
///  milliseconds; one that is still empty after this long has no writer. The sweep waits out what remains of it.
///  Measured with real Git 2.56.0 on a throwaway repository, traced with `strace` (which slows each call, so these are upper
///  bounds): a reference lock was empty for at most 2.244 ms between its creation and its first write over 7,700 updates in
///  four alternating arms (median 0.38 ms), at load averages 3 to 14. One second is about four hundred times that longest
///  gap; the load average of 80 to 145 that `doc/planning/cli-git-commit-timeouts.md` records was not present, so the factor is
///  not a proof for it. Kept; the coordinator's pick under the owner's delegation, open to the owner's veto.
const EMPTY_LOCK_SETTLE: Duration = Duration::from_secs(1);

/// The directory inside `objects` that Git locks files in; the other directories there hold only loose objects.
const SEARCHED_OBJECT_DIRECTORIES: [&str; 2] = ["info", "pack"];

/// Directories that never hold a Git lock file and can hold very many files.
const SKIPPED_DIRECTORIES: [&str; 1] = ["hooks"];

/// What:
///  Where a command's Git child can leave lock files, and when the child started.
/// Why:
///   Taken before the child starts, so a lock born earlier is never taken for the child's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockLedger = { roots: string[]; since: Date };
/// ```
#[derive(Clone, Debug)]
pub struct LockLedger {
    /// The Git directory and the common Git directory of the repository the command runs in; empty when none is found.
    roots: Vec<PathBuf>,
    /// The wall-clock time read before the child started.
    since: SystemTime,
    /// How long an empty lock without a pid file must have stayed empty before it is removed.
    settle: Duration,
}

impl LockLedger {
    /// A ledger that finds nothing: for a child that holds no Git lock, such as a reader of objects.
    pub fn none() -> Self {
        return Self { roots: Vec::new(), since: SystemTime::now(), settle: EMPTY_LOCK_SETTLE };
    }

    /// The ledger of a command that has not started yet: its repository comes from `GIT_DIR` in its environment, or from
    ///  its working directory (the process's own when the command sets none).
    pub fn before_start(command: &Command) -> Self {
        let since: SystemTime = SystemTime::now();
        return Self { roots: roots_of_command(command), since, settle: EMPTY_LOCK_SETTLE };
    }

    /// When a file was born; a file system that keeps no birth time still stamps a never-written file with its creation time.
    fn born_at(path: &Path) -> Option<SystemTime> {
        let metadata: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
        return metadata.created().or_else(|_| { return metadata.modified(); }).ok();
    }

    /// Whether a file was born at or after the child started.
    fn born_since(&self, path: &Path) -> bool {
        return Self::born_at(path).is_some_and(|born| { return born + COARSE_CLOCK_SLACK >= self.since; });
    }

    /// Whether `path` is a regular file with nothing in it.
    fn is_empty_file(path: &Path) -> bool {
        return std::fs::symlink_metadata(path).is_ok_and(|metadata| { return metadata.is_file() && metadata.len() == 0; });
    }

    /// The same ledger with another settle time, so a test does not wait out the real one.
    #[cfg(test)]
    pub fn with_settle(mut self, settle: Duration) -> Self {
        self.settle = settle;
        return self;
    }

    /// Whether an empty lock without a pid file is the stopped child's: born after it started, and still empty after the
    ///  settle time, which this waits out.
    fn left_empty(&self, path: &Path) -> bool {
        if !self.born_since(path) || !Self::is_empty_file(path) {
            return false;
        }
        let age: Duration = Self::born_at(path).and_then(|born| { return born.elapsed().ok(); }).unwrap_or_default();
        if age < self.settle {
            std::thread::sleep(self.settle - age);
        }
        return Self::is_empty_file(path);
    }

    /// What:
    ///  Remove the locks the stopped tree left, and return the paths removed.
    /// Why:
    ///   Called once the child is gone. A pid file counts only when the process it names belongs to `tree` and has ended;
    ///  an empty lock without a pid file counts only when the whole tree has ended.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function sweep(tree: TreeMember[]): string[];
    /// ```
    pub fn sweep(&self, tree: &[TreeMember]) -> Vec<PathBuf> {
        let tree_ended: bool = !tree.is_empty() && tree.iter().all(|member| { return !still_running(member); });
        let mut removed: Vec<PathBuf> = Vec::new();
        for path in lock_files(self.roots.as_slice()) {
            let name: String = path.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default();
            if let Some(stem) = name.strip_suffix(PID_LOCK_SUFFIX) {
                let Some(pid) = pid_named_by(path.as_path()) else {
                    continue;
                };
                let Some(member) = tree.iter().find(|member| { return member.pid == pid; }) else {
                    continue;
                };
                if still_running(member) || !self.born_since(path.as_path()) {
                    continue;
                }
                let lock: PathBuf = path.with_file_name(format!("{stem}{LOCK_SUFFIX}"));
                remove_recording(lock.as_path(), &mut removed);
                remove_recording(path.as_path(), &mut removed);
            } else if tree_ended
                && !path.with_file_name(format!("{}{PID_LOCK_SUFFIX}", name.trim_end_matches(LOCK_SUFFIX))).exists()
                && self.left_empty(path.as_path())
            {
                remove_recording(path.as_path(), &mut removed);
            }
        }
        return removed;
    }
}

/// The process number a pid file names, when its text is `pid <number>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(readFileSync(path, 'utf8').trim().replace(/^pid /, ''))
/// ```
pub(crate) fn pid_named_by(path: &Path) -> Option<i32> {
    let text: String = std::fs::read_to_string(path).ok()?;
    return text.trim().strip_prefix(PID_TEXT_PREFIX)?.trim().parse::<i32>().ok();
}

/// Remove one file and record it when it was removed; a file that is already gone is not an error.
fn remove_recording(path: &Path, removed: &mut Vec<PathBuf>) {
    match std::fs::remove_file(path) {
        Ok(()) => {
            warn(TAG, &format!("removed {}, a lock file the stopped Git process left behind", path.display()));
            removed.push(path.to_path_buf());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            warn(TAG, &format!("could not remove {}, a lock file the stopped Git process left behind: {error}", path.display()));
        }
    }
}

/// Every regular file below `roots` whose name ends in `.lock`, in one pass over the directories that can hold one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// walk(roots, { skip: ['hooks', 'objects/xx'] }).filter((path) => path.endsWith('.lock'))
/// ```
fn lock_files(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = roots.to_vec();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        let in_objects: bool =
            directory.file_name() == Some(OsStr::new("objects")) && directory.parent().is_some_and(|parent| { return roots.iter().any(|root| { return root == parent; }); });
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name: String = entry.file_name().to_string_lossy().into_owned();
            if kind.is_dir() {
                let searched: bool = if in_objects { SEARCHED_OBJECT_DIRECTORIES.contains(&name.as_str()) } else { !SKIPPED_DIRECTORIES.contains(&name.as_str()) };
                if searched {
                    pending.push(entry.path());
                }
            } else if kind.is_file() && name.ends_with(LOCK_SUFFIX) {
                found.push(entry.path());
            }
        }
    }
    found.sort();
    found.dedup();
    return found;
}

/// Lock-sweep behavior is controlled in the test module.
#[cfg(test)]
#[path = "stopped_git_locks_tests.rs"]
mod tests;
