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
//! (`<runtime>/systemd/user/monochromatic-git-policy-cli-worktree-notes/<instance>.git-dir`,
//! which lives as long as the units do),
//! and a unit whose note is missing is mapped from the `gitdir=` line of its service journal's records,
//! only when the text hashes to the instance.
//! A unit that maps to nothing is reported and never stopped.
//! The units are stopped through the manager,
//! never by killing a process by name,
//! and nothing here starts a unit.
//! A service that starts and finds its own worktree gone stops its own socket before it serves anything,
//! so a removal that Git ran with the wrapper replaced,
//! or a worktree deleted without Git,
//! takes its units with it at the first connection rather than at the next sweep a person remembers.
//! The journal of an instance whose units were stopped goes with them
//! (`remove_journal_of_a_stopped_instance`),
//! because a stopped instance is never served again and its records name a worktree that is gone;
//! a journal that holds a batch no branch shows,
//! a record that was written and not published,
//! or a file this release does not know is kept and named instead of discarded,
//! and so is a journal whose landed batches a client may still read,
//! or whose service the manager does not yet report gone:
//! those keep the note and the socket file too,
//! so a later sweep finishes.
//! A service that stops its own socket never removes its own journal,
//! because it is still one of that journal's writers:
//! its after-commit task and that task's jobs write the note of a repository to look at and the plan of a push to try again,
//! and a write that lands between the listing and the removal would be taken with the whole tree.
//! It keeps the note and the socket file instead,
//! so the sweep that asks the manager whether its service is gone removes the journal once that is over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (worktreeIsGone(await worktreeOf(folders, instance))) await stopUnits(manager, instance);
//! removeJournalOfAStoppedInstance(folders, instance);
//! ```

use crate::diagnostic_log::warn;
use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::{GITDIR_HEX_PREFIX, path_of_hex};
use crate::service_instance::{INSTANCE_LENGTH, SOCKET_DIRECTORY, instance_of, service_unit_instance, socket_unit_instance};
use crate::service_provision_host_and_manager::{Manager, runtime_unit_dir};
use crate::service_route::non_empty_path_variable;
use crate::service_is_gone_after_a_stop::service_is_gone;
use crate::systemd_dbus::{Bus, CallError, EXCHANGE_WAIT, NO_SUCH_UNIT, session_bus_path};
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceUnitsOfRemovedWorktrees";

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

/// The journal folder of an instance and the state folder that holds it,
/// which the unit's `--journal` names.
pub use crate::service_journal_folder_of_an_instance::{JOURNAL_FOLDER, journal_of, state_dir_of_journal};

/// What an instance keeps beside its socket and in its journal,
/// read and written by the code here and by provisioning
/// (`service_files_of_a_stopped_instance.rs`).
pub use crate::service_files_of_a_stopped_instance::{
    JournalAfterStop, KeptJournal, NOTE_ENDING, NOTES_FOLDER, forget_stopped_instance, journal_of_the_instance, note_path, note_worktree,
    record_worktree, remove_journal_of_a_stopped_instance, work_a_person_may_want,
};

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
    /// The journals that stayed after their instance's units were stopped,
    /// each with the instance and the entry that kept it.
    pub journals_kept: Vec<(String, KeptJournal)>,
    /// The survey the stop was made from.
    pub survey: Survey,
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

/// What:
///  The Git directory a note or a record line names for an instance,
/// when the path it names hashes to that instance.
/// Why:
///   A note of this release holds the bytes of the directory and one line break after them;
/// a note of an earlier release holds its text and nothing after it,
/// so the last byte of that text is the path's even when it is a line break.
/// A note whose bytes do not decode names nothing,
/// and its units stay where they are rather than being stopped on a guess.
fn named_for(instance: &str, text: &str) -> Option<PathBuf> {
    let (bytes, plain): (Option<&str>, Option<&str>) = match text.strip_prefix(GITDIR_HEX_PREFIX) {
        Some(hexadecimal) => (Some(hexadecimal.strip_suffix('\n').unwrap_or(hexadecimal)), None),
        None if text.is_empty() => (None, None),
        None => (None, Some(text)),
    };
    let path: PathBuf = path_bytes::path_of_the_two_fields(bytes, plain).ok().flatten()?;
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
        // A line of this release names the bytes of the directory, which is the only spelling that keeps a path that is not
        //  text; a line of an earlier release names its plain text.
        let mut plain: Option<PathBuf> = None;
        for line in text.lines() {
            if let Some(hexadecimal) = line.strip_prefix(GITDIR_HEX_PREFIX) {
                if let Some(found) = path_of_hex(hexadecimal)
                    && instance_of(found.as_path()) == instance
                {
                    return Some(found);
                }
                continue;
            }
            if plain.is_none()
                && let Some(named) = line.strip_prefix("gitdir=")
            {
                plain = named_for(instance, named);
            }
        }
        if plain.is_some() {
            return plain;
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

/// The instance a file name belongs to when it ends with `ending` after an instance name.
fn instance_before<'a>(name: &'a str, ending: &str) -> Option<&'a str> {
    let stem: &str = name.strip_suffix(ending)?;
    return is_instance_name(stem).then_some(stem);
}

/// Adds the instances that the files of `folder` ending with `ending` name.
fn add_instances_of(folder: &Path, ending: &str, found: &mut Vec<String>) {
    let Ok(listing) = std::fs::read_dir(folder) else { return };
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
    add_instances_of(runtime_dir.join(SOCKET_DIRECTORY).as_path(), ".sock", &mut found);
    add_instances_of(runtime_unit_dir(runtime_dir).join(NOTES_FOLDER).as_path(), NOTE_ENDING, &mut found);
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

/// What:
/// The Git directory of an instance whose worktree is gone,
/// and nothing when it is still there,
/// when it cannot be told whose the units are,
/// or when the place that held the directory is not there either.
/// Why:
///   This is the question a service asks about itself before it serves:
/// a service of a worktree that no longer exists has nothing to commit to,
/// and the files of its socket and its note are the only record of whose they are.
/// The answer is the same one the survey and the sweep give,
/// so a service and the sweep never disagree about a gone worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeGone(folders, instance) // '/repo/.git/worktrees/topic' or undefined
/// ```
pub fn own_worktree_gone(folders: &UnitFolders<'_>, instance: &str) -> Option<PathBuf> {
    let UnitWorktree::Known(git_dir) = worktree_of(folders, instance) else { return None };
    return worktree_is_gone(git_dir.as_path()).then_some(git_dir);
}

/// What a service's own units are to it,
/// once it asks about the worktree they serve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnWorktree {
    /// The worktree's Git directory is certainly gone:
    /// the service has nothing to commit to and stops its own socket.
    Gone(PathBuf),
    /// The worktree is there,
    /// or it is on a device that cannot be reached right now,
    /// so the service serves it.
    Served(PathBuf),
    /// Nothing maps the instance to a worktree,
    /// so the service serves and nothing is stopped.
    Unmapped,
}

/// What:
/// What this service's own units are,
/// read from the runtime directory of the process and the journal it was started with.
/// Why:
///   This is the question a serving service asks of itself:
/// the note provisioning wrote beside the socket,
/// else the `gitdir=` records of its own journal,
/// name a directory that hashes to the instance,
/// and the same rule the sweep and `doctor` use decides whether it is gone.
/// A worktree on a device that is simply not mounted is served,
/// and so is one neither the note nor the journal names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeState(instance, journal) // { gone: gitDir } | { served: gitDir } | 'unmapped'
/// ```
pub fn own_worktree_state(instance: &str, journal: &Path) -> OwnWorktree {
    let Some(runtime_dir) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return OwnWorktree::Unmapped };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() };
    return match worktree_of(&folders, instance) {
        UnitWorktree::Unmapped => OwnWorktree::Unmapped,
        UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => OwnWorktree::Gone(git_dir),
        UnitWorktree::Known(git_dir) => OwnWorktree::Served(git_dir),
    };
}

/// One stop,
/// where a unit the manager does not hold is stopped already.
fn stop_one(manager: &mut dyn Manager, unit: &str) -> Result<(), CallError> {
    return match manager.stop(unit) {
        Err(CallError::Manager { name, .. }) if name == NO_SUCH_UNIT => Ok(()),
        other => other,
    };
}

/// One stop of the named unit and of nothing that requires it,
/// where a unit the manager does not hold is stopped already.
fn stop_one_alone(manager: &mut dyn Manager, unit: &str) -> Result<(), CallError> {
    return match manager.stop_alone(unit) {
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

/// One stop of an instance's socket through the manager.
fn stop_own_socket(instance: &str, manager: &mut dyn Manager) -> Result<(), CallError> {
    manager.within(EXCHANGE_WAIT);
    return stop_one_alone(manager, socket_unit_instance(instance).as_str());
}

/// What:
/// Stops the socket unit of one instance through the manager
/// and keeps the journal,
/// the note and the socket file for the sweep that follows this process's end,
/// when the worktree of that instance is gone,
/// and nothing when it is not.
/// Why:
///   This is what a service of a gone worktree does about itself:
/// it serves nothing,
/// and no connection starts it again.
/// `doctor` counts the instance until the sweep removes it,
/// which is the price of never taking a record this process may still publish.
/// Only the socket is stopped,
/// and with the mode that leaves what requires it alone:
/// the service's own end is what stops its service unit,
/// and a stop of that unit from inside the service would signal this process mid-sentence.
/// The socket goes first so no connection starts the service again.
/// The journal is the one file this process may still write,
/// so its removal waits for a sweep that proves the service gone,
/// and the files that name the instance stay until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopOwnSocketOfAGoneWorktree(folders, instance, manager) // the Git directory, or undefined
/// ```
pub fn stop_own_socket_of_a_gone_worktree(folders: &UnitFolders<'_>, instance: &str, manager: &mut dyn Manager) -> Result<Option<PathBuf>, CallError> {
    let Some(gone) = own_worktree_gone(folders, instance) else { return Ok(None) };
    stop_own_socket(instance, manager)?;
    // Nothing of the journal is removed here, because this process is one of its writers and still runs:
    //  the task that runs the after-commit work writes the note that names a repository to look at and the plan of a push
    //  to try again, on a thread of its own and from jobs of the blocking pool, and both may write between the listing that
    //  says the journal holds nothing unresolved and the removal of the whole tree.
    // Removing it there took a record published in that window: the durable record of a pending push, of a repository still
    //  to look at, or of a batch a client whose answer was lost still reads.
    // So the journal, the note and the socket file stay for the sweep that runs after this process has ended and the manager
    //  reports its service gone (`stop_units_of_gone_worktrees`), and what the journal holds is still named here.
    match work_a_person_may_want(folders.state_dir, instance) {
        Some(kept) => crate::service_warn!(
            "the journal of this commit service is kept at {}, because it holds {}, {}; look at it and remove it yourself if nothing in it is wanted",
            kept.journal.display(),
            kept.entry,
            kept.reason
        ),
        // Landed records a client may still read, and the landing record of nothing in flight, ask nothing of a person:
        //  the sweep removes them together with the files that name this instance once this process is gone.
        None => crate::service_log!(
            "the journal of this commit service is kept at {} for the sweep that follows this process's end, because this service is still one of its writers",
            journal_folder_in_words(folders.state_dir, instance)
        ),
    }
    return Ok(Some(gone));
}

/// The journal folder of an instance in words,
/// for a line that names where the journal a service leaves is kept.
fn journal_folder_in_words(state_dir: Option<&Path>, instance: &str) -> String {
    let Some(journal) = journal_of_the_instance(state_dir, instance) else {
        return String::from("a folder this process was given no state directory for");
    };
    return journal.display().to_string();
}

/// What:
/// Stops this service's own socket when the worktree it serves is gone,
/// with the line that says why,
/// and says whether the worktree is gone.
/// Why:
///   A removal Git ran with the wrapper replaced (`worktree remove --force`,
/// `worktree prune`),
/// and a worktree deleted without Git,
/// leave the socket unit listening and the service unit running for a directory that is gone,
/// and a service has nothing to commit to there.
/// Nothing else asks,
/// so the service asks itself:
/// when it starts,
/// and while it serves (`service_server.rs`).
/// It stops its own socket and nothing that requires it,
/// because its own end is what stops its service unit,
/// and a stop of that unit from inside the service would signal it before it removed the note and the socket file.
/// A manager that cannot be reached only logs:
/// the files stay,
/// so `doctor` counts the leftover and the next start tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopOwnSocketWhenTheWorktreeIsGone(instance, journal) // true: gone, so serve nothing
/// ```
pub fn stop_own_socket_when_the_worktree_is_gone(instance: &str, journal: &Path) -> bool {
    let Some(runtime_dir) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return false };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() };
    // A worktree that is there, or one nothing names, costs no bus connection and no question to the manager: the files answer it.
    let OwnWorktree::Gone(gone) = own_worktree_state(instance, journal) else { return false };
    let address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    let Some(socket) = session_bus_path(address.as_deref(), Some(runtime_dir.as_path())) else {
        crate::service_warn!(
            "the worktree of this commit service is gone ({}), so it serves nothing, but no session bus was found to stop its socket; `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
            gone.display()
        );
        return true;
    };
    let mut bus: Bus = match Bus::connect(socket.as_path(), EXCHANGE_WAIT) {
        Ok(connected) => connected,
        Err(error) => {
            crate::service_warn!(
                "the worktree of this commit service is gone ({}), so it serves nothing, but the session bus could not be reached ({error:?}) to stop its socket; `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
                gone.display()
            );
            return true;
        }
    };
    match stop_own_socket_of_a_gone_worktree(&folders, instance, &mut bus) {
        Ok(Some(stopped)) => {
            crate::service_log!("the worktree of this commit service is gone ({}), so it served nothing and stopped its own socket", stopped.display());
        }
        // The directory came back between the two looks, so this service is a live worktree's again.
        Ok(None) => {}
        Err(error) => {
            crate::service_warn!(
                "the worktree of this commit service is gone ({}), so it serves nothing, but its socket was not stopped ({error:?}); `git monochromatic-git-policy-cli doctor` counts it and the sweep stops it",
                gone.display()
            );
        }
    }
    return true;
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
/// and removes the note and the socket file of each one that was stopped,
/// and its journal when it holds nothing a person may still want and the manager reports its service gone.
/// Why:
///   An instance that maps to nothing,
/// or whose worktree is still there,
/// is never asked about.
/// A refused stop keeps its note,
/// so the next sweep tries the instance again,
/// and so does a journal kept for a client that may still read it,
/// or for a service that outlasted `SERVICE_GONE_AFTER_A_STOP`.
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
                // A stop answers when the manager queued the job, so one question more says the service's processes are gone,
                //  and only then may the journal it writes be removed.
                let quiet: bool = service_is_gone(manager, instance.as_str());
                match remove_journal_of_a_stopped_instance(folders.state_dir, instance.as_str(), quiet) {
                    Some(JournalAfterStop::HoldsWork(kept)) => {
                        forget_stopped_instance(folders.runtime_dir, instance.as_str());
                        outcome.journals_kept.push((instance.clone(), kept));
                    }
                    // The journal is kept for the clients that may still read it, or for a service that is still going:
                    //  the note and the socket file stay, so the next sweep finds the instance and finishes.
                    Some(JournalAfterStop::StillRead(_)) => {}
                    Some(JournalAfterStop::Removed) | None => forget_stopped_instance(folders.runtime_dir, instance.as_str()),
                }
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
