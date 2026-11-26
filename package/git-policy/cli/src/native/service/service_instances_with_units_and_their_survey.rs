//! What:
//!  Which instances have a unit or a journal,
//!  and the survey that sorts them into those whose worktree is gone,
//!  those that map to nothing and could still have a unit,
//!  those that map to nothing and are a journal alone,
//!  and the count of those whose worktree is still there.
//! Why:
//!  A sweep that stopped an instance it could not finish kept its journal and removed the two files that named it,
//! so a later sweep found no unit and never looked at the journal again (issue 6322).
//!  The survey reads files only,
//! so `doctor` can report it without a manager,
//!  and the sweep asks the manager about exactly the instances it names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { gone, unmapped, journalsWithoutUnits, present } = survey(folders);
//! ```

use crate::service_files_of_a_stopped_instance::{
    NOTE_ENDING,
    NOTES_FOLDER,
    note_path,
};
use crate::service_instance::{
    INSTANCE_LENGTH,
    SOCKET_DIRECTORY,
    socket_path,
};
use crate::service_journal_folder_of_an_instance::JOURNAL_FOLDER;
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_unit_folders_and_the_worktree_of_an_instance::{
    UnitFolders,
    UnitWorktree,
    worktree_is_gone,
    worktree_of,
};
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::Path;

/// Which instances have a unit,
/// whose worktree is gone,
/// which cannot be told,
/// and which are a journal with nothing beside it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// The instances whose worktree's Git directory no longer exists.
    pub gone: Vec<String>,
    /// The instances that map to nothing and still have a socket file or a note,
    /// so a unit of theirs could be running.
    pub unmapped: Vec<String>,
    /// The instances that map to nothing and whose journal is the only thing left of them:
    /// no socket file and no note stands beside it,
    /// so no unit of theirs exists and only a person can clear the folder.
    pub journals_without_units: Vec<String>,
    /// The instances whose worktree is still there.
    pub present: usize,
}

/// Whether one character is a lower-case hexadecimal digit.
fn is_hexadecimal_digit(character: char) -> bool {
    return character.is_ascii_digit() || ('a'..='f').contains(&character);
}

/// Whether a file name's stem is an instance name:
/// the digits of a hash.
fn is_instance_name(name: &str) -> bool {
    return name.len() == INSTANCE_LENGTH && name.chars().all(is_hexadecimal_digit);
}

/// The instance a file name belongs to when it ends with `ending` after an instance name.
fn instance_before<'a>(
    name: &'a str,
    ending: &str,
) -> Option<&'a str> {
    let stem: &str = name.strip_suffix(ending)?;
    return is_instance_name(stem).then_some(stem);
}

/// Adds the instances that the files of `folder` ending with `ending` name.
fn add_instances_of(
    folder: &Path,
    ending: &str,
    found: &mut Vec<String>,
) {
    let Ok(listing) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if let Some(instance) = instance_before(name.as_str(), ending) {
            found.push(instance.to_owned());
        }
    }
}

/// Which instances have a socket file in the socket directory or a note in the notes folder,
/// sorted.
pub fn instances_with_units(runtime_dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    add_instances_of(
        runtime_dir.join(SOCKET_DIRECTORY).as_path(),
        ".sock",
        &mut found,
    );
    add_instances_of(
        runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).as_path(),
        NOTE_ENDING,
        &mut found,
    );
    found.sort();
    found.dedup();
    return found;
}

/// What:
///  Whether any unit of an instance could exist:
///  its socket file or its note is there.
/// Why:
///   A worktree that never provisioned a unit has neither,
/// so the removal need not connect to the manager at all,
/// and the sweep asks the manager about no instance that has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hasUnits(folders, instance) // socket file or note
/// ```
pub fn has_units(
    folders: &UnitFolders<'_>,
    instance: &str,
) -> bool {
    return socket_path(folders.runtime_dir, instance).exists()
        || note_path(folders.runtime_dir, instance).exists();
}

/// What:
///  Every instance the sweep and `doctor` look at:
///  the ones with a socket file or a note,
///  and the ones whose journal folder is in the state directory.
/// Why:
///   A sweep that stopped an instance it could not finish kept its journal and removed the two files that named it,
/// so a later sweep found no unit and never looked at the journal again:
/// the folders of removed worktrees stayed for ever and asked a person at every removal (issue 6322).
/// A journal folder is named after its instance,
/// so the state directory names the same instances the note did,
/// and `worktree_of` reads the worktree back from the journal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// instancesWithUnitsOrJournals(folders) // socket files, notes and journal folders, sorted and each once
/// ```
pub fn instances_with_units_or_journals(folders: &UnitFolders<'_>) -> Vec<String> {
    let mut found: Vec<String> = instances_with_units(folders.runtime_dir);
    if let Some(state) = folders.state_dir {
        add_instances_of(
            state
                .join(SERVICE_UNIT.current)
                .join(JOURNAL_FOLDER)
                .as_path(),
            "",
            &mut found,
        );
    }
    found.sort();
    found.dedup();
    return found;
}

/// What:
/// Sorts the instances that have a unit or a journal into those whose worktree is gone,
/// those that map to nothing,
/// those that map to nothing and have no unit left,
/// and the count of those whose worktree is still there.
/// Why:
///   The sort reads files only,
/// so `doctor` can report it without a manager,
/// and an instance whose journal outlived the note and the socket file that named it is looked at again.
///   An instance that maps to nothing is only a unit that could not be matched while a socket file or a note
///  stands beside it:
///  those two files are what make a unit possible,
///  and without them the journal is all that is left,
///  which a person can look at and remove
///  (M1 of the alpha.46 commit,
/// push and LFS release pass:
///  seven such journals on one machine were counted on every `doctor` as instances nothing could touch).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { gone, unmapped, journalsWithoutUnits, present } = survey(folders);
/// ```
pub fn survey(folders: &UnitFolders<'_>) -> Survey {
    let mut found: Survey = Survey::default();
    for instance in instances_with_units_or_journals(folders) {
        match worktree_of(folders, instance.as_str()) {
            UnitWorktree::Unmapped if has_units(folders, instance.as_str()) => {
                found.unmapped.push(instance)
            }
            UnitWorktree::Unmapped => found.journals_without_units.push(instance),
            UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => {
                found.gone.push(instance)
            }
            UnitWorktree::Known(_present) => found.present += 1,
        }
    }
    return found;
}
