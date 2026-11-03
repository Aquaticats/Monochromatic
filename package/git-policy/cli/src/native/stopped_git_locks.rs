//! What:
//!  The lock files a stopped Git child leaves behind, found and removed once the child and what it started are gone.
//! Why:
//!  Git removes its `.lock` files when it ends normally or on a catchable signal, but a child that is killed, or that is
//!  in an uninterruptible call when the stop arrives, cannot (measured on 2026-11-03: two pushes stopped at their deadline
//!  left `refs/remotes/origin/main.lock` and a second tracking-ref lock, both empty and born in the same instant, long
//!  after the stop; every later `git fetch` of that ref then failed with "Unable to create ... File exists").
//!  Two kinds of file are removed, and nothing else:
//!       a lock whose `~pid.lock` file (written by `core.lockfilePid`, which the wrapper turns on for every Git it starts)
//!       names a process of the stopped tree that has ended, with that pid file;
//!       an empty lock with no pid file, born after the child started, because the child can die between creating
//!       the lock and writing the pid file, and an empty file holds nothing.
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
/// The common directory a linked worktree's Git directory names.
use super::service_review::common_dir_of;
/// Owned OS strings of a command's environment.
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
const LOCK_SUFFIX: &str = ".lock";

/// What the pid file of a lock ends with: the lock's name without `.lock`, then this.
const PID_LOCK_SUFFIX: &str = "~pid.lock";

/// What a pid file's text starts with, before the number.
const PID_TEXT_PREFIX: &str = "pid ";

/// How far a file's birth may precede the clock reading taken before the child started.
///  The kernel stamps files from a coarse clock that lags the one `SystemTime` reads by up to one tick.
const COARSE_CLOCK_SLACK: Duration = Duration::from_millis(50);

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
}

/// Where the Git directory of a working directory is: `.git` as a directory, a `gitdir:` file, or a bare repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function gitDirectoryOf(start: string): string | undefined;
/// ```
fn git_directory_of(start: &Path) -> Option<PathBuf> {
    for directory in start.ancestors() {
        let dot_git: PathBuf = directory.join(".git");
        if let Ok(metadata) = std::fs::symlink_metadata(dot_git.as_path()) {
            if metadata.is_dir() {
                return Some(dot_git);
            }
            if metadata.is_file() {
                let text: String = std::fs::read_to_string(dot_git.as_path()).ok()?;
                let named: &str = text.lines().next()?.strip_prefix("gitdir:")?.trim();
                return Some(directory.join(named));
            }
        }
        if directory.join("HEAD").is_file() && directory.join("objects").is_dir() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

impl LockLedger {
    /// A ledger that finds nothing: for a child that holds no Git lock, such as a reader of objects.
    pub fn none() -> Self {
        return Self { roots: Vec::new(), since: SystemTime::now() };
    }

    /// The ledger of a command that has not started yet: its repository comes from `GIT_DIR` in its environment, or from
    ///  its working directory (the process's own when the command sets none).
    pub fn before_start(command: &Command) -> Self {
        let since: SystemTime = SystemTime::now();
        let working: PathBuf = command
            .get_current_dir()
            .map(Path::to_path_buf)
            .or_else(|| { return std::env::current_dir().ok(); })
            .unwrap_or_default();
        let named: Option<PathBuf> = command
            .get_envs()
            .filter(|(name, _)| { return *name == OsStr::new("GIT_DIR"); })
            .last()
            .and_then(|(_, value)| { return value.map(|path| { return working.join(path); }); });
        let git_dir: Option<PathBuf> = named.or_else(|| { return git_directory_of(working.as_path()); });
        let mut roots: Vec<PathBuf> = Vec::new();
        if let Some(directory) = git_dir {
            let common: PathBuf = common_dir_of(directory.as_path());
            roots.push(directory.clone());
            if common != directory {
                roots.push(common);
            }
        }
        return Self { roots, since };
    }

    /// Whether a file was born at or after the child started.
    fn born_since(&self, path: &Path) -> bool {
        let Ok(metadata) = std::fs::symlink_metadata(path) else {
            return false;
        };
        // A file system that keeps no birth time still stamps a never-written file with its creation time.
        let Ok(born) = metadata.created().or_else(|_| { return metadata.modified(); }) else {
            return false;
        };
        return born + COARSE_CLOCK_SLACK >= self.since;
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
                && std::fs::symlink_metadata(path.as_path()).is_ok_and(|metadata| { return metadata.is_file() && metadata.len() == 0; })
                && self.born_since(path.as_path())
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
fn pid_named_by(path: &Path) -> Option<i32> {
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
