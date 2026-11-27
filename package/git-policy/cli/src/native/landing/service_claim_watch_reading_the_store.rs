//! What:
//!  The reading of the claim watch's store:
//! which notes are kept,
//! which are removed as certainly of no use,
//! and what could not be read at all.
//! Why:
//!  A note is read the way every record of this wrapper is read (`private_storage::read_regular_file`),
//! so a name that holds no regular file is answered at once,
//! said once and left where it is,
//! and so is a file longer than a note can hold:
//! a pipe is never waited on,
//! a link is never read through,
//! and neither is removed,
//! because no note this store wrote is any of those.
//! Only a file that is certainly of no use is removed (it holds no path,
//! names another instance,
//! or its work directory is absent or not a directory).
//! A read that fails for any other reason leaves the file where it is and is tried again
//! after a growing pause,
//! so a refused read or a full disk delays the look and never ends it
//! (`service_claim_watch.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = scanTheStore(store, instance, known, unread, read) // the directories not known before
//! ```

use super::service_claim_watch_note_format::{
    NOTE_FILE_LIMIT,
    ReadFile,
    WATCH_EXTENSION,
    decode,
};
use super::service_claim_watch_whose_repository_it_is::of_another_instance;
use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::private_storage::ReadRefusal;
use crate::work_item::WORK_DIRECTORY;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Reads the store:
/// keeps the directories whose file is whole,
/// names this instance and has a work directory,
/// removes the files that are certainly of no use,
/// and notes a failure to read as `unread`.
/// Why:
///  A watch that starts again knows no repository until this reading,
/// and a look finds the records the notes name,
/// so what is kept here is what the service will look at.
/// The answer is the directories that were not known before.
/// A name that holds no regular file,
/// and a file longer than a note can hold,
/// is said once and left where it is:
/// no note this store wrote is either,
/// and it is not read again after a pause,
/// because a second read of it answers the same.
/// Nothing is kept from a store the process has no directory for,
/// and such a store is not called unread either,
/// because a watch kept in memory only knows its directories already.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = scanTheStore(store, instance, known, unread, read)
/// ```
pub(super) fn scan_the_store(
    store: Option<&Path>,
    instance: Option<&str>,
    known: &mut Vec<PathBuf>,
    unread: &mut bool,
    read: ReadFile<'_>,
) -> Vec<PathBuf> {
    let Some(directory) = store else {
        return Vec::new();
    };
    *unread = false;
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        // A journal that never held a note.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(error) => {
            crate::service_warn!(
                "the notes of the repositories to look at in {} could not be listed, so they are listed again after a pause: {error}",
                directory.display()
            );
            *unread = true;
            return Vec::new();
        }
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for listed in listing {
        let path: PathBuf = match listed {
            Ok(entry) => entry.path(),
            Err(error) => {
                crate::service_warn!(
                    "an entry of {} could not be listed, so the notes are listed again after a pause: {error}",
                    directory.display()
                );
                *unread = true;
                continue;
            }
        };
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(WATCH_EXTENSION) {
            continue;
        }
        let git_dir: PathBuf = match read(path.as_path()) {
            Ok(bytes) => match decode(bytes.as_slice()) {
                Ok(git_dir) => git_dir,
                Err(reason) => {
                    crate::service_warn!(
                        "the note {} holds no repository and is removed: {reason}",
                        path.display()
                    );
                    let _removed = std::fs::remove_file(path.as_path());
                    continue;
                }
            },
            // Removed between the listing and the read.
            Err(ReadRefusal::Missing) => continue,
            // Nothing this store writes: a pipe, a link or a directory.
            // It is said once and left where it is,
            // and it is not read again after a pause, because a second read of it answers the same.
            Err(ReadRefusal::NotRegular) => {
                crate::service_warn!(
                    "the note {} is not a regular file, so it holds no repository to look at and stays",
                    path.display()
                );
                continue;
            }
            // Nothing this store writes either: a note is the format words and one path's bytes in hexadecimal.
            Err(ReadRefusal::TooLarge) => {
                crate::service_warn!(
                    "the note {} holds more than the {NOTE_FILE_LIMIT} bytes a note can, so it holds no repository and stays",
                    path.display()
                );
                continue;
            }
            Err(ReadRefusal::Io(error)) => {
                crate::service_warn!(
                    "the note {} could not be read, so it is read again after a pause: {error}",
                    path.display()
                );
                *unread = true;
                continue;
            }
        };
        if of_another_instance(instance, git_dir.as_path()) {
            crate::service_warn!(
                "the note {} names {}, which is not this instance's repository, and is removed",
                path.display(),
                git_dir.display()
            );
            let _removed = std::fs::remove_file(path.as_path());
            continue;
        }
        match std::fs::symlink_metadata(path_for_reading(git_dir.as_path(), WORK_DIRECTORY)) {
            Ok(metadata) if metadata.is_dir() => {
                if !known.contains(&git_dir) {
                    known.push(git_dir.clone());
                    found.push(git_dir);
                }
            }
            Ok(_) => {
                crate::service_warn!(
                    "the work directory of {} is not a directory, so its note {} is removed",
                    git_dir.display(),
                    path.display()
                );
                let _removed = std::fs::remove_file(path.as_path());
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                let _removed = std::fs::remove_file(path.as_path());
            }
            Err(error) => {
                crate::service_warn!(
                    "the work directory of {} could not be looked at, so its note is kept and read again after a pause: {error}",
                    git_dir.display()
                );
                *unread = true;
            }
        }
    }
    return found;
}
