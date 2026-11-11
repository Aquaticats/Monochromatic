//! What:
//! The files an instance of the commit service keeps:
//! the note that names its worktree beside the socket,
//! the note and the socket file that say it has units,
//! and the journal that remembers its batches.
//! Why:
//! A unit instance is named by a hash of its worktree's own Git directory (`service_instance.rs`),
//! so the note is the one record of whose units these are once the worktree is gone:
//! provisioning writes it,
//! and a stop reads it back.
//! A stopped instance of a worktree that is gone is never served again,
//! so its note,
//! its socket file and the records of its batches are leftovers nothing reads,
//! and one state directory per removed worktree stayed for ever.
//! The note and the socket file always go,
//! because a file that stayed would count the instance again and the next sweep stop it again.
//! The journal goes only when every entry in it is the record of a batch that landed:
//! a commit that has not landed,
//! a batch whose objects a scratch reference may hold,
//! a record that was written and not published,
//! a landing or withdrawal record,
//! and any name this release does not explain keep the whole journal,
//! which is then named to the person,
//! because work they may still want is never discarded.
//! Nothing here starts a unit,
//! and nothing here asks a manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordWorktree(runtimeDir, instance, gitDir); // the note
//! forgetStoppedInstance(runtimeDir, instance); // note and socket file
//! removeJournalOfAStoppedInstance(stateDir, instance); // { journal, entry, reason } when it is kept
//! ```

use crate::diagnostic_log::warn;
use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::GITDIR_HEX_PREFIX;
use crate::private_storage::create_private_directory_all;
use crate::service_instance::socket_path;
use crate::service_journal::{CLAIM_EXTENSION, DONE_EXTENSION, INDEXED_EXTENSION, OPEN_EXTENSION, SCRATCH_EXTENSION, START_EXTENSION, TEMPORARY_EXTENSION};
use crate::service_journal_folder_of_an_instance::journal_of;
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_provision_unit_file_ownership_and_writes::write_atomically;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceFilesOfAnInstance";

/// The ending of the note file that names the Git directory of an instance's worktree.
pub const NOTE_ENDING: &str = ".git-dir";

/// The folder of the notes,
/// inside the directory the manager reads the wrapper's unit files from,
/// where provisioning already writes and nothing else of the wrapper does.
/// The manager looks only at unit names in that directory,
/// so a folder of this name is no unit and starts nothing.
pub const NOTES_FOLDER: &str = "monochromatic-git-policy-cli-worktree-notes";

/// The directory of the record a landing keeps inside the journal (`service_busy.rs`),
/// which is empty once no landing is in flight.
const LANDING_RECORD_DIRECTORY: &str = "busy";

/// The note file of an instance.
pub fn note_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    return runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).join(format!("{instance}{NOTE_ENDING}"));
}

/// What:
/// Writes the note that names the worktree of an instance,
/// unless it already says so.
/// Why:
///   The note is the one record of whose units these are once the worktree is gone;
/// it is read before it is written so that a commit in a worktree whose units are in place writes nothing.
/// It lives in the socket's directory under the runtime directory,
/// which is removed with the units at logout or shutdown,
/// so no note outlives the units it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if ((await readText(notePath)) !== gitDir) await writeAtomically(notePath, gitDir);
/// ```
pub fn record_worktree(runtime_dir: &Path, instance: &str, git_dir: &Path) -> std::io::Result<()> {
    // The note holds the bytes of the directory: its text would name another place, or none, for a path that is not text.
    let text: String = format!("{GITDIR_HEX_PREFIX}{}\n", path_bytes::hex_of_path(git_dir));
    let path: PathBuf = note_path(runtime_dir, instance);
    if matches!(std::fs::read_to_string(&path), Ok(current) if current == text) {
        return Ok(());
    }
    create_private_directory_all(runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).as_path())?;
    return write_atomically(path.as_path(), text.as_str());
}

/// What:
/// Writes the note of an instance for the real Git directory of its worktree,
/// and logs a failure.
/// Why:
///   A note that cannot be written only means the units cannot be told apart later;
/// it never fails a commit.
/// The real path is written because the instance is a hash of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await noteWorktree(runtimeDir, instance, await realpath(gitDir)).catch(log);
/// ```
pub fn note_worktree(runtime_dir: &Path, instance: &str, git_dir: &Path) {
    let real: PathBuf = match std::fs::canonicalize(git_dir) {
        Ok(resolved) => resolved,
        Err(_unresolvable) => git_dir.to_path_buf(),
    };
    if let Err(error) = record_worktree(runtime_dir, instance, real.as_path()) {
        warn(TAG, format!("the note of the worktree of {instance} could not be written ({error}); its units cannot be told apart from a removed worktree's later").as_str());
    }
}

/// What:
///  A journal that stayed after its instance's units were stopped,
///  the entry that kept it,
///  and what that entry is.
/// Why:
///   The caller decides where the line goes:
/// the command a person runs reports it on standard error,
/// and the service logs it to its own journal,
/// so a notice a person needs never depends on a warning being enabled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptJournal {
    /// The journal folder that stays.
    pub journal: PathBuf,
    /// The entry that keeps it.
    pub entry: String,
    /// What that entry is,
    /// in words.
    pub reason: String,
}

/// Whether every byte of the text is a decimal digit.
fn are_digits(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// Whether a file name is the record of a batch that landed,
/// which a stopped instance's journal may lose.
fn is_finished_memory(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else { return false };
    if !are_digits(stem) {
        return false;
    }
    return extension == DONE_EXTENSION;
}

/// The extension of a file name,
/// and nothing when it has none.
fn extension_of(name: &str) -> &str {
    return match name.rsplit_once('.') {
        Some((_stem, extension)) => extension,
        None => "",
    };
}

/// Why a file of a journal that is not a finished batch keeps the journal.
fn reason_of_a_kept_file(name: &str) -> &'static str {
    let extension: &str = extension_of(name);
    if extension == OPEN_EXTENSION {
        return "the record of a commit that has not landed";
    }
    if extension == SCRATCH_EXTENSION {
        return "the record of a batch whose objects a scratch reference may hold";
    }
    if extension == CLAIM_EXTENSION || extension == INDEXED_EXTENSION || extension == START_EXTENSION {
        return "the record of a batch that was being landed";
    }
    // A `.tmp` file can hold a record that was written whole and never published, so it is no remains.
    if extension == TEMPORARY_EXTENSION {
        return "a record that was written and not published";
    }
    return "a file this release does not know";
}

/// Why a folder inside a journal keeps it.
fn reason_of_a_kept_folder(name: &str) -> &'static str {
    if name == LANDING_RECORD_DIRECTORY {
        return "a record of a landing or of a withdrawal";
    }
    return "a folder this release does not know";
}

/// The folder's entries,
/// and the words that say why they could not be read.
fn entries_of(folder: &Path) -> Result<std::fs::ReadDir, String> {
    return match std::fs::read_dir(folder) {
        Ok(listing) => Ok(listing),
        Err(error) => Err(format!("{} could not be listed ({error})", folder.display())),
    };
}

/// Whether a folder holds no entry at all.
fn holds_nothing(folder: &Path) -> Result<bool, String> {
    let mut listing = entries_of(folder)?;
    return Ok(listing.next().is_none());
}

/// What:
///  The first entry of a stopped instance's journal that keeps the journal,
/// with the words that say why,
/// and nothing when every entry may go.
/// Why:
///   Only the record of a batch that landed,
/// and a folder the service made and left empty (`busy`,
/// the landing's record),
/// are provably of no use once the instance is stopped;
/// every other name either holds a commit a person may still want
/// or is a name this release does not explain,
/// and guessing about one of those would discard work.
/// The answer comes from one listing,
/// so a journal that grows while it is read is kept,
/// not half-removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptEntryOf(journal) // { name, reason } or undefined
/// ```
fn kept_entry_of(journal: &Path) -> Result<Option<(String, &'static str)>, String> {
    let listing = match entries_of(journal) {
        Ok(listing) => listing,
        Err(reason) => {
            // A journal that is not there has nothing to keep.
            if !journal.exists() {
                return Ok(None);
            }
            return Err(reason);
        }
    };
    for listed in listing {
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => return Err(format!("{} could not be listed ({error})", journal.display())),
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let path: PathBuf = entry.path();
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(error) => return Err(format!("{} could not be read ({error})", path.display())),
        };
        if kind.is_dir() {
            if !holds_nothing(path.as_path())? {
                return Ok(Some((name.clone(), reason_of_a_kept_folder(name.as_str()))));
            }
            continue;
        }
        if is_finished_memory(name.as_str()) {
            continue;
        }
        return Ok(Some((name.clone(), reason_of_a_kept_file(name.as_str()))));
    }
    return Ok(None);
}

/// What:
///  Removes the service journal of an instance whose units were stopped,
///  when every entry in it is the record of a batch that landed,
///  and says which entry kept it otherwise.
/// Why:
///   Keeping those records left one state directory per removed worktree with up to `DONE_RETAINED` files in it,
/// and nothing swept them;
/// the note and the socket file are removed in the same step,
/// so no later survey asks this instance about its worktree.
/// A journal that holds anything else is kept whole and reported,
/// and a state directory that is not known removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeJournalOfAStoppedInstance(stateDir, instance); // { journal, entry, reason } when it is kept
/// ```
pub fn remove_journal_of_a_stopped_instance(state_dir: Option<&Path>, instance: &str) -> Option<KeptJournal> {
    let state: &Path = state_dir?;
    let journal: PathBuf = journal_of(state.join(SERVICE_UNIT.current).as_path(), instance);
    let kept: KeptJournal = match kept_entry_of(journal.as_path()) {
        Ok(None) => {
            if let Err(error) = std::fs::remove_dir_all(journal.as_path())
                && error.kind() != std::io::ErrorKind::NotFound
            {
                warn(TAG, format!("the journal of {instance} could not be removed ({error}), so it is left at {}", journal.display()).as_str());
            }
            return None;
        }
        Ok(Some((entry, reason))) => KeptJournal { journal, entry, reason: String::from(reason) },
        Err(reason) => KeptJournal { journal, entry: String::from("something this command could not read"), reason },
    };
    return Some(kept);
}

/// What:
///  Removes the files that say an instance has units once its units are stopped:
/// its note and its socket file.
/// Why:
///   The manager leaves the socket file of a stopped socket unit where it was (`RemoveOnStop=` is off),
/// and a file that stays makes the survey count the instance again and the next sweep stop it again.
/// The worktree is gone,
/// so nothing will make that socket listen at that path again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of [notePath(runtimeDir, instance), socketPath(runtimeDir, instance)]) await rm(path);
/// ```
pub fn forget_stopped_instance(runtime_dir: &Path, instance: &str) {
    for path in [note_path(runtime_dir, instance), socket_path(runtime_dir, instance)] {
        if let Err(error) = std::fs::remove_file(path.as_path())
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn(TAG, format!("{} could not be removed after the units of {instance} were stopped ({error})", path.display()).as_str());
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_files_of_a_stopped_instance_tests.rs"]
mod tests;
