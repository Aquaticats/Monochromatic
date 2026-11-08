//! What:
//! The commit service units of worktrees that no longer exist:
//! the note that says which worktree a unit belongs to,
//! the test of whether that worktree is gone,
//! and the stop of the units of a gone one through the user manager.
//! Why:
//! A unit instance is named by a hash of its worktree's own Git directory (`service_instance.rs`),
//! and a hash names nothing once the directory is gone:
//! removing a worktree left its socket and service unit running,
//! and nothing could say whose they were.
//! A socket unit has no idle exit,
//! and the service exits only three hours after its last client.
//! So provisioning writes the Git directory beside the socket
//! (`<runtime>/monochromatic-git-policy-cli/<instance>.git-dir`,
//! which lives as long as the units do),
//! and a unit whose note is missing is mapped from the `gitdir=` line of its service journal's records,
//! only when the text hashes to the instance.
//! A unit that maps to nothing is reported and never stopped.
//! The units are stopped through the manager,
//! never by killing a process by name,
//! and nothing here starts a unit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (worktreeIsGone(await worktreeOf(folders, instance))) await stopUnits(manager, instance);
//! ```

use crate::diagnostic_log::warn;
use crate::private_storage::create_private_directory_all;
use crate::service_instance::{INSTANCE_LENGTH, SOCKET_DIRECTORY, instance_of, service_unit_instance, socket_unit_instance};
use crate::service_provision_host_and_manager::Manager;
use crate::service_provision_unit_file_ownership_and_writes::write_atomically;
use crate::systemd_dbus::{CallError, EXCHANGE_WAIT, NO_SUCH_UNIT};
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceUnitsOfRemovedWorktrees";

/// The ending of the note file that names the Git directory of an instance's worktree.
pub const NOTE_ENDING: &str = ".git-dir";

/// Where the units of the user's instances are told apart from one another.
#[derive(Clone, Copy, Debug)]
pub struct UnitFolders<'a> {
    /// The user's runtime directory,
    /// which holds the sockets and the notes.
    pub runtime_dir: &'a Path,
    /// The user's state directory,
    /// whose `monochromatic-git-policy-cli/service/<instance>` folders hold the service journals.
    pub state_dir: Option<&'a Path>,
}

/// What a unit instance's worktree is,
/// as far as the notes and the journals say.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitWorktree {
    /// The worktree's Git directory,
    /// whose text hashes to the instance.
    Known(PathBuf),
    /// Nothing maps the instance to a worktree.
    Unmapped,
}

/// Which instances have a unit,
/// whose worktree is gone,
/// and which cannot be told.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// The instances whose worktree's Git directory no longer exists.
    pub gone: Vec<String>,
    /// The instances that map to nothing.
    pub unmapped: Vec<String>,
    /// The instances whose worktree is still there.
    pub present: usize,
}

/// What a stop of the units of gone worktrees did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SweepOutcome {
    /// The instances whose units were stopped.
    pub stopped: Vec<String>,
    /// The instances whose units could not be stopped,
    /// each with the reason.
    pub failed: Vec<(String, String)>,
    /// The survey the stop was made from.
    pub survey: Survey,
}

/// The note file of an instance.
pub fn note_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    return runtime_dir.join(SOCKET_DIRECTORY).join(format!("{instance}{NOTE_ENDING}"));
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
    let text: String = git_dir.to_string_lossy().into_owned();
    let path: PathBuf = note_path(runtime_dir, instance);
    if matches!(std::fs::read_to_string(&path), Ok(current) if current == text) {
        return Ok(());
    }
    create_private_directory_all(runtime_dir.join(SOCKET_DIRECTORY).as_path())?;
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
/// Whether the worktree whose own Git directory is `git_dir` is certainly gone:
/// the directory is missing while the place that holds it is there.
/// Why:
///   A repository on a drive that is not mounted has a missing Git directory too,
/// and its worktrees are not gone.
/// A linked worktree's directory is `<common>/worktrees/<id>`,
/// and Git removes `worktrees` itself with the last one,
/// so the common directory is the place that must be there;
/// a main checkout's Git directory is held by the checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeIsGone('/repo/.git/worktrees/topic') // true when /repo/.git is a directory and topic is not there
/// ```
pub fn worktree_is_gone(git_dir: &Path) -> bool {
    match std::fs::symlink_metadata(git_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return false,
    }
    let Some(parent) = git_dir.parent() else { return false };
    let anchor: Option<&Path> = if parent.file_name() == Some(std::ffi::OsStr::new("worktrees")) { parent.parent() } else { Some(parent) };
    return match anchor {
        Some(place) => place.is_dir(),
        None => false,
    };
}

/// The Git directory a text names for an instance,
/// when the text hashes to that instance.
fn named_for(instance: &str, text: &str) -> Option<PathBuf> {
    let trimmed: &str = text.trim_end_matches('\n');
    if trimmed.is_empty() {
        return None;
    }
    let path: PathBuf = PathBuf::from(trimmed);
    return (instance_of(path.as_path()) == instance).then_some(path);
}

/// The records of a service journal,
/// sorted by name.
fn journal_records(journal: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(journal) else { return Vec::new() };
    let mut records: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("done")) {
            records.push(path);
        }
    }
    records.sort();
    return records;
}

/// The `gitdir=` text of the first record of the instance's service journal that has one,
/// and names a directory that hashes to the instance.
fn from_journal(state_dir: &Path, instance: &str) -> Option<PathBuf> {
    let journal: PathBuf = state_dir.join(SERVICE_UNIT.current).join("service").join(instance);
    for record in journal_records(journal.as_path()) {
        let Ok(text) = std::fs::read_to_string(&record) else { continue };
        for line in text.lines() {
            if let Some(named) = line.strip_prefix("gitdir=")
                && let Some(found) = named_for(instance, named)
            {
                return Some(found);
            }
        }
    }
    return None;
}

/// What:
/// The worktree of an instance:
/// its note,
/// else the Git directory in a record of its service journal,
/// each only when its text hashes to the instance.
/// Why:
///   A text that does not hash to the instance names some other directory,
/// and stopping the units of a worktree on a wrong guess would cut a live one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeOf(folders, instance) // { known: gitDir } or unmapped
/// ```
pub fn worktree_of(folders: &UnitFolders<'_>, instance: &str) -> UnitWorktree {
    if let Ok(text) = std::fs::read_to_string(note_path(folders.runtime_dir, instance))
        && let Some(found) = named_for(instance, text.as_str())
    {
        return UnitWorktree::Known(found);
    }
    if let Some(state) = folders.state_dir
        && let Some(found) = from_journal(state, instance)
    {
        return UnitWorktree::Known(found);
    }
    return UnitWorktree::Unmapped;
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

/// The instance a file name of the socket directory belongs to,
/// when it is a socket file or a note.
fn instance_of_file_name(name: &str) -> Option<&str> {
    let stem: &str = match name.strip_suffix(".sock") {
        Some(socket) => socket,
        None => name.strip_suffix(NOTE_ENDING)?,
    };
    return is_instance_name(stem).then_some(stem);
}

/// Which instances have a socket file or a note in the runtime directory,
/// sorted.
pub fn instances_with_units(runtime_dir: &Path) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(runtime_dir.join(SOCKET_DIRECTORY)) else { return Vec::new() };
    let mut found: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if let Some(instance) = instance_of_file_name(name.as_str()) {
            found.push(instance.to_owned());
        }
    }
    found.sort();
    found.dedup();
    return found;
}

/// What:
/// Sorts the instances that have a unit into those whose worktree is gone,
/// those that map to nothing,
/// and the count of those whose worktree is still there.
/// Why:
///   The sort reads files only,
/// so `doctor` can report it without a manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { gone, unmapped, present } = survey(folders);
/// ```
pub fn survey(folders: &UnitFolders<'_>) -> Survey {
    let mut found: Survey = Survey::default();
    for instance in instances_with_units(folders.runtime_dir) {
        match worktree_of(folders, instance.as_str()) {
            UnitWorktree::Unmapped => found.unmapped.push(instance),
            UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => found.gone.push(instance),
            UnitWorktree::Known(_present) => found.present += 1,
        }
    }
    return found;
}

/// One stop,
/// where a unit the manager does not hold is stopped already.
fn stop_one(manager: &mut dyn Manager, unit: &str) -> Result<(), CallError> {
    return match manager.stop(unit) {
        Err(CallError::Manager { name, .. }) if name == NO_SUCH_UNIT => Ok(()),
        other => other,
    };
}

/// What:
/// Stops the socket unit and the service unit of one instance through the manager.
/// Why:
///   The socket goes first so a connection cannot start the service again.
/// A refusal of one stop does not keep the other from being asked,
/// and the first refusal is the answer.
/// Nothing is killed by name:
/// the manager stops its own units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopUnits(manager, instance) // throws the first refusal
/// ```
pub fn stop_units(manager: &mut dyn Manager, instance: &str) -> Result<(), CallError> {
    let socket: Result<(), CallError> = stop_one(manager, socket_unit_instance(instance).as_str());
    let service: Result<(), CallError> = stop_one(manager, service_unit_instance(instance).as_str());
    return socket.and(service);
}

/// The manager's failure in words.
fn reason_of(error: &CallError) -> String {
    return match error {
        CallError::Connection(reason) => format!("the session bus could not be reached ({reason})"),
        CallError::Authentication(reason) => format!("the session bus refused the connection ({reason})"),
        CallError::TimedOut => String::from("the session bus did not answer in time"),
        CallError::Manager { text, name } if text.is_empty() => format!("the service manager refused the request ({name})"),
        CallError::Manager { text, .. } => format!("the service manager refused the request ({text})"),
        CallError::Malformed(reason) => format!("the service manager's answer could not be read ({reason})"),
    };
}

/// What:
/// Stops the units of every instance whose worktree is gone,
/// and removes the note of each one that was stopped.
/// Why:
///   An instance that maps to nothing,
/// or whose worktree is still there,
/// is never asked about.
/// A refused stop keeps its note,
/// so the next sweep tries the instance again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stopped, failed, survey } = await stopUnitsOfGoneWorktrees(manager, folders);
/// ```
pub fn stop_units_of_gone_worktrees(manager: &mut dyn Manager, folders: &UnitFolders<'_>) -> SweepOutcome {
    let surveyed: Survey = survey(folders);
    let mut outcome: SweepOutcome = SweepOutcome { survey: surveyed.clone(), ..SweepOutcome::default() };
    manager.within(EXCHANGE_WAIT);
    for instance in &surveyed.gone {
        match stop_units(manager, instance.as_str()) {
            Ok(()) => {
                let _removed = std::fs::remove_file(note_path(folders.runtime_dir, instance.as_str()));
                outcome.stopped.push(instance.clone());
            }
            Err(error) => {
                warn(TAG, format!("the units of {instance} were not stopped: {}", reason_of(&error)).as_str());
                outcome.failed.push((instance.clone(), reason_of(&error)));
            }
        }
    }
    return outcome;
}

/// Unit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_units_of_removed_worktrees_tests.rs"]
mod tests;
