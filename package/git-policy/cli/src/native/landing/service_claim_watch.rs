//! What:
//!  The Git directories whose after-commit work this service instance has been woken for, kept on disk in the instance's own
//!  journal, so that a service that starts again looks at them by itself.
//! Why:
//!  The service learns which repository to drain only from a client's wake, and a client wakes it only after a commit. A service
//!  that was killed while it held a claim, or that ended with work still pending, was started again by the next connection and
//!  then had nothing to drain until the next commit, which might never come: the backup push of the last commit of the day waited
//!  without a bound. The journal survives a restart and has one writer, so it can name the directories to look at when the service
//!  starts. The name is written by the worker thread before it drains a directory for the first time in a process, never by the
//!  thread that lands commits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = restore(journalDir); watch.remember(gitDir); for (const dir of watch.directories()) await drain(dir);
//! ```

use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::push_failure_notice::fnv_hex;
use crate::work_item::WORK_DIRECTORY;
use std::path::{Path, PathBuf};

/// Directory, under the service instance's journal, that holds one small file per watched Git directory.
pub const WATCH_DIRECTORY: &str = "claim-watch";

/// The extension of a watch file.
const WATCH_EXTENSION: &str = "watch";

/// What:
///  The Git directories to look at when the service starts, and the ones this process has already written down.
/// Why:
///   One entry per directory; a directory that was written down is not written again, so the file system is touched once per
///  directory per process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ClaimWatch { store?: string; known: string[]; }
/// ```
#[derive(Debug, Default)]
pub struct ClaimWatch {
    /// The directory the watch files are kept in; nothing for a watch kept in memory only.
    store: Option<PathBuf>,
    /// The Git directories known to this process, from the files read at the start or remembered since.
    known: Vec<PathBuf>,
}

/// The file name of one watched Git directory.
fn watch_name(git_dir: &Path) -> String {
    return format!("{}.{WATCH_EXTENSION}", fnv_hex(git_dir.display().to_string().as_str()));
}

/// Methods over the watch.
impl ClaimWatch {
    /// A watch kept in memory only.
    pub fn in_memory() -> Self {
        return Self::default();
    }

    /// What:
    ///  A watch kept in `directory`, holding the Git directories the service before this one wrote down.
    /// Why:
    ///   A directory that has no work directory any more (the worktree was removed, the records were swept) has nothing to look
    ///  at; its file is removed, and the next wake for it writes it again. A file that holds no absolute path is removed as well,
    ///  so that it is not said at every start.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ClaimWatch(dir, (await readdir(dir)).map(read).filter(hasWorkDirectory))
    /// ```
    pub fn restored(directory: &Path) -> Self {
        let mut known: Vec<PathBuf> = Vec::new();
        if let Ok(listing) = std::fs::read_dir(directory) {
            for entry in listing.flatten() {
                let path: PathBuf = entry.path();
                if path.extension().and_then(|extension| { return extension.to_str(); }) != Some(WATCH_EXTENSION) {
                    continue;
                }
                let git_dir: PathBuf = PathBuf::from(std::fs::read_to_string(path.as_path()).unwrap_or_default().trim_end_matches('\n'));
                if git_dir.is_absolute() && git_dir.join(WORK_DIRECTORY).is_dir() {
                    if !known.contains(&git_dir) {
                        known.push(git_dir);
                    }
                } else {
                    let _removed = std::fs::remove_file(path.as_path());
                }
            }
        }
        return Self { store: Some(directory.to_path_buf()), known };
    }

    /// The Git directories to look at, each once.
    pub fn directories(&self) -> Vec<PathBuf> {
        return self.known.clone();
    }

    /// Writes `git_dir` down unless this process has done so; a write that fails is said and tried again at the next wake.
    pub fn remember(&mut self, git_dir: &Path) {
        if self.known.iter().any(|known| { return known == git_dir; }) {
            return;
        }
        if let Some(directory) = self.store.as_deref()
            && let Err(reason) = write_down(directory, git_dir)
        {
            crate::service_warn!("{} could not be kept for a restart, so a restart waits for the next wake to look at it: {reason}", git_dir.display());
            return;
        }
        self.known.push(git_dir.to_path_buf());
    }
}

/// Writes the file of `git_dir`; one that is already there is the same file.
fn write_down(directory: &Path, git_dir: &Path) -> Result<(), String> {
    create_private_directory_all(directory).map_err(|error| { return format!("creating the watch directory failed: {error}"); })?;
    let text: String = format!("{}\n", git_dir.display());
    return match write_private_file_unsynced(directory.join(watch_name(git_dir)).as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("writing the watch file failed: {error}")),
    };
}

/// Watch controls stay out of the release executable.
#[cfg(test)]
#[path = "service_claim_watch_tests.rs"]
mod tests;
