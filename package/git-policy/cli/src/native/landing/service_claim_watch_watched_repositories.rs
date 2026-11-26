//! What:
//!  What the notes of one claim watch say about the repositories they name,
//! read without keeping anything:
//!  whether one of them still leads a look to work,
//! and which repositories they name at all.
//! Why:
//!  A journal outlives the worktrees its notes were written for,
//! so the cleanup of a stopped instance asks these notes before it takes the watch with the journal,
//! and the sweep of a removed worktree's units asks them once the worktree is gone.
//! A note whose repository,
//! or whose work directory,
//! is gone leads a service to nothing.
//! The store that writes and reads notes is `service_claim_watch.rs`,
//! which re-exports what this file holds,
//!  so a caller names that file alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! watchedRepositories(directory) // 'all gone' | { there, note } | { unexplained, entry }
//! namedRepositories(directory) // the decoded paths of the notes, each once
//! ```

use super::service_claim_watch_note_format::{
    WATCH_EXTENSION,
    decode,
    read_file,
};
use crate::private_storage::ReadRefusal;
use crate::service_work::work_directory;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What the notes of one instance's claim watch say about the repositories they name.
/// Why:
///   A note is a pointer, and the work it leads a look to is the named repository's own:
/// a look lists `<Git directory>/monochromatic-git-policy-cli/work/pending` (`a_record_waits`),
/// so a note whose repository, or whose work directory, is gone leads a service to nothing.
/// The cleanup of a stopped instance's journal asks this before it takes the claim watch with the journal,
/// because after a worktree is removed the only service that read its notes is gone too.
/// A note that names a repository that still holds a work directory is kept and named:
/// the store keeps such a note itself and a service of that repository still looks at what waits there.
/// A name the store does not explain,
/// and a note it could not read,
/// is kept as well: nothing is discarded on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchedRepositories(directory) // 'all gone' | { there, note } | { unexplained, entry }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WatchedRepositories {
    /// Every note names a repository whose work directory is gone,
    /// or holds no path at all,
    /// so no look at them finds a record.
    AllGone,
    /// One note names a repository that still holds the work directory a look reads,
    /// with that repository and the note's own name.
    StillThere { git_dir: PathBuf, note: String },
    /// An entry of the folder this release cannot read as a note it wrote,
    /// with the entry and what stands in the way.
    Unexplained { entry: String, reason: String },
}

/// The name of a folder in a line that names it,
/// as the folder's path says it.
fn name_of(folder: &Path) -> String {
    return match folder.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => folder.display().to_string(),
    };
}

/// Whether a repository still holds the work directory a look reads:
/// the same question the store asks before it keeps a note,
/// and one it answers by removing a note whose work directory is absent or not a directory.
fn holds_work_a_look_would_read(git_dir: &Path) -> bool {
    return match std::fs::symlink_metadata(work_directory(git_dir).as_path()) {
        Ok(metadata) => metadata.is_dir(),
        Err(_unreachable) => false,
    };
}

/// What:
///  The notes of `directory`,
///  sorted by name,
///  and nothing when the folder is not there.
/// Why:
///   The answer about a claim watch is read in one listing of its notes,
/// and the folder sorts so that two readings of it name the same note.
fn notes_of(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("it could not be listed ({error})")),
    };
    let mut notes: Vec<PathBuf> = Vec::new();
    for listed in listing {
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => return Err(format!("an entry of it could not be listed ({error})")),
        };
        notes.push(entry.path());
    }
    notes.sort();
    return Ok(notes);
}

/// What:
///  What the notes of `directory` say about the repositories they name,
///  read in the way `scan` reads them.
/// Why:
///   The cleanup of a journal reads this before it removes the watch with it;
/// see `WatchedRepositories` for what each answer means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchedRepositories(directory) // list, read and decode every note, test each work directory
/// ```
pub fn watched_repositories(directory: &Path) -> WatchedRepositories {
    let notes: Vec<PathBuf> = match notes_of(directory) {
        Ok(notes) => notes,
        Err(reason) => {
            return WatchedRepositories::Unexplained {
                entry: name_of(directory),
                reason,
            };
        }
    };
    let mut still_there: Option<(PathBuf, String)> = None;
    for note in notes {
        let entry: String = name_of(note.as_path());
        if note.extension().and_then(std::ffi::OsStr::to_str) != Some(WATCH_EXTENSION) {
            return WatchedRepositories::Unexplained {
                entry,
                reason: String::from(
                    "a file in the claim watch that this release does not explain",
                ),
            };
        }
        let bytes: Vec<u8> = match read_file(note.as_path()) {
            Ok(bytes) => bytes,
            // Removed between the listing and the read.
            Err(ReadRefusal::Missing) => continue,
            Err(ReadRefusal::NotRegular) => {
                return WatchedRepositories::Unexplained {
                    entry,
                    reason: String::from(
                        "a note that is not a regular file, which this release leaves where it is",
                    ),
                };
            }
            Err(ReadRefusal::TooLarge) => {
                return WatchedRepositories::Unexplained {
                    entry,
                    reason: String::from(
                        "a note that holds more than a note can, which this release leaves where it is",
                    ),
                };
            }
            Err(ReadRefusal::Io(error)) => {
                return WatchedRepositories::Unexplained {
                    entry,
                    reason: format!("it could not be read ({error})"),
                };
            }
        };
        // A note that holds no whole path is one the store removes itself (`scan`), so it points at nothing.
        let Ok(git_dir) = decode(bytes.as_slice()) else {
            continue;
        };
        if still_there.is_none() && holds_work_a_look_would_read(git_dir.as_path()) {
            still_there = Some((git_dir, entry));
        }
    }
    return match still_there {
        Some((git_dir, note)) => WatchedRepositories::StillThere {
            git_dir,
            note,
        },
        None => WatchedRepositories::AllGone,
    };
}

/// What:
///  The Git directories the notes of `directory` name,
///  each once,
///  in the order of the notes' names.
/// Why:
///   A note names the Git directory of the repository whose work a look would drain,
/// and this instance's journal holds notes only for its own repository
/// (`of_another_instance` refuses every other),
/// so a note is one more record of whose units these are once the worktree is gone:
/// the sweep reads it when neither the note beside the socket nor a record of the journal names one.
/// A note that holds no whole path names nothing and is left out,
/// because the store removes such a note itself (`scan`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namedRepositories(directory) // the decoded paths of the notes, each once
/// ```
pub fn named_repositories(directory: &Path) -> Vec<PathBuf> {
    let notes: Vec<PathBuf> = match notes_of(directory) {
        Ok(notes) => notes,
        Err(_unreadable) => return Vec::new(),
    };
    let mut named: Vec<PathBuf> = Vec::new();
    for note in notes {
        if note.extension().and_then(std::ffi::OsStr::to_str) != Some(WATCH_EXTENSION) {
            continue;
        }
        let Ok(bytes) = read_file(note.as_path()) else {
            continue;
        };
        let Ok(git_dir) = decode(bytes.as_slice()) else {
            continue;
        };
        if !named.contains(&git_dir) {
            named.push(git_dir);
        }
    }
    return named;
}
