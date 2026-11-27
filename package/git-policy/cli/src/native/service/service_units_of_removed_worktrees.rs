//! What:
//! The commit service units of worktrees that no longer exist:
//! the note that says which worktree a unit belongs to,
//! the test of whether that worktree or its repository is gone,
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
//! An instance whose journal is the only thing left of it,
//! with no socket file and no note beside it,
//! has no unit that could be running:
//! the survey counts it apart,
//! and the command says the folder a person can look at and clear by hand.
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
//! a claim watch note naming a repository that still holds the work directory a look reads,
//! or a file this release does not know is kept and named instead of discarded,
//! and so is a journal whose landed batches a client may still read,
//! or whose service the manager does not yet report gone:
//! those keep the note and the socket file too,
//! so a later sweep finishes.
//! Two entries that a removal used to keep the journal for carry no work,
//! because their only reader is the removed worktree's own service:
//! the repository the service remembered for its rules warm,
//! and a claim watch note naming the removed worktree or a repository whose work directory is gone
//! (`service_files_of_a_stopped_instance.rs`).
//! A service that stops its own socket never removes its own journal,
//! because it is still one of that journal's writers:
//! its after-commit task and that task's jobs write the note of a repository to look at and the plan of a push to try again,
//! and a write that lands between the listing and the removal would be taken with the whole tree.
//! It keeps the note and the socket file instead,
//! so the sweep that asks the manager whether its service is gone removes the journal once that is over.
//!
//! A service that asks about its own worktree,
//! and stops its own socket when that worktree is gone,
//! is `service_own_worktree_gone_and_the_stop_of_its_socket.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (worktreeIsGone(await worktreeOf(folders, instance))) await stopUnits(manager, instance);
//! removeJournalOfAStoppedInstance(folders, instance);
//! ```

pub use super::service_own_worktree_gone_and_the_stop_of_its_socket::{
    OwnWorktree,
    journals_folder_in_words,
    own_worktree_gone,
    own_worktree_state,
    stop_own_socket_of_a_gone_worktree,
    stop_own_socket_when_the_worktree_is_gone,
};
use crate::diagnostic_log::warn;
use crate::service_instance::{
    service_unit_instance,
    socket_unit_instance,
};
use crate::service_is_gone_after_a_stop::service_is_gone;
use crate::service_provision_host_and_manager::Manager;
use crate::systemd_dbus::{
    CallError,
    EXCHANGE_WAIT,
    NO_SUCH_UNIT,
};
use std::path::PathBuf;

//region The pieces the survey and the sweep read from their own modules

/// What:
///  `pub use` names each piece of this module under the one path its callers already write,
///  `crate::service_units_of_removed_worktrees::name`.
/// Why:
///   The module was split into sibling files so each one stays a size a reader can hold,
/// and the re-export keeps every caller and the sidecar's `use super::*` unchanged.
/// Gotcha:
///   A private helper of a sibling stays there;
///  only the names written outside that sibling are listed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { UnitFolders, UnitWorktree, worktreeOf, worktreeIsGone } from './service_unit_folders_and_the_worktree_of_an_instance';
/// export { Survey, survey, hasUnits } from './service_instances_with_units_and_their_survey';
/// ```
pub use super::service_instances_with_units_and_their_survey::{
    Survey,
    has_units,
    instances_with_units,
    instances_with_units_or_journals,
    survey,
};
pub use super::service_unit_folders_and_the_worktree_of_an_instance::{
    UnitFolders,
    UnitWorktree,
    worktree_is_gone,
    worktree_of,
};

//endregion

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceUnitsOfRemovedWorktrees";

/// The journal folder of an instance and the state folder that holds it,
/// which the unit's `--journal` names.
pub use crate::service_journal_folder_of_an_instance::{
    JOURNAL_FOLDER,
    journal_of,
    state_dir_of_journal,
};

/// What an instance keeps beside its socket and in its journal,
/// read and written by the code here and by provisioning
/// (`service_files_of_a_stopped_instance.rs`).
pub use crate::service_files_of_a_stopped_instance::{
    JournalAfterStop,
    KeptJournal,
    NOTE_ENDING,
    NOTES_FOLDER,
    forget_stopped_instance,
    journal_of_the_instance,
    note_path,
    note_worktree,
    record_worktree,
    remove_journal_of_a_stopped_instance,
    work_a_person_may_want,
};

/// The record of the unit texts the manager was told about for one instance,
/// which goes with the instance once its units are stopped
/// (`service_provision_loaded_unit_texts.rs`).
use crate::service_provision_loaded_unit_texts::forget as forget_loaded_unit_texts;

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
    /// The journals that went with the units of a gone worktree,
    /// each with the instance and the folder that was removed.
    pub journals_removed: Vec<(String, PathBuf)>,
    /// The survey the stop was made from.
    pub survey: Survey,
}

/// One stop,
/// where a unit the manager does not hold is stopped already.
fn stop_one(
    manager: &mut dyn Manager,
    unit: &str,
) -> Result<(), CallError> {
    return match manager.stop(unit) {
        Err(CallError::Manager {
            name,
            ..
        }) if name == NO_SUCH_UNIT => Ok(()),
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
pub fn stop_units(
    manager: &mut dyn Manager,
    instance: &str,
) -> Result<(), CallError> {
    let socket: Result<(), CallError> = stop_one(manager, socket_unit_instance(instance).as_str());
    let service: Result<(), CallError> =
        stop_one(manager, service_unit_instance(instance).as_str());
    return socket.and(service);
}

/// The manager's failure in words.
fn reason_of(error: &CallError) -> String {
    return match error {
        CallError::Connection(reason) => format!("the session bus could not be reached ({reason})"),
        CallError::Authentication(reason) => {
            format!("the session bus refused the connection ({reason})")
        }
        CallError::TimedOut => String::from("the session bus did not answer in time"),
        CallError::Manager {
            text,
            name,
        } if text.is_empty() => format!("the service manager refused the request ({name})"),
        CallError::Manager {
            text,
            ..
        } => format!("the service manager refused the request ({text})"),
        CallError::Malformed(reason) => {
            format!("the service manager's answer could not be read ({reason})")
        }
    };
}

/// What:
///  Forgets every file that says an instance has units,
/// once the units of a worktree that is gone are stopped:
/// its note,
///  its socket file and the record of the unit texts the manager was told about.
/// Why:
///   The record says what the manager read for an instance whose units no longer exist,
/// and a worktree that is gone never
///  comes back at that Git directory,
/// so nothing will ask the manager about it again
/// (`service_provision_loaded_unit_texts.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheInstance(folders, instance); // note, socket file and record
/// ```
fn forget_the_instance(
    folders: &UnitFolders<'_>,
    instance: &str,
) {
    forget_stopped_instance(folders.runtime_dir, instance);
    forget_loaded_unit_texts(folders.runtime_dir, instance);
}

/// What:
/// Stops the units of every instance whose worktree is gone,
/// and removes the note and the socket file of each one that was stopped,
/// and its journal when it holds nothing a person may still want and the manager reports its service gone.
/// Why:
///   An instance that maps to nothing,
/// or whose worktree is still there,
/// is never asked about.
/// An instance whose socket file and note are both gone has no unit the manager could start again,
/// so the manager is not asked about it and only its journal is decided:
/// that is how the journal of a removal which kept one,
/// and left no file that named the instance but the journal
/// itself,
/// is cleared here.
/// A refused stop keeps its note,
/// so the next sweep tries the instance again,
/// and so does a journal kept for a client that may still read it,
/// or for a service that outlasted `SERVICE_GONE_AFTER_A_STOP`.
/// A journal that was removed is named with its folder in the outcome,
/// so the command a person ran says where it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stopped, failed, survey } = await stopUnitsOfGoneWorktrees(manager, folders);
/// ```
pub fn stop_units_of_gone_worktrees(
    manager: &mut dyn Manager,
    folders: &UnitFolders<'_>,
) -> SweepOutcome {
    let surveyed: Survey = survey(folders);
    let mut outcome: SweepOutcome = SweepOutcome {
        survey: surveyed.clone(),
        ..SweepOutcome::default()
    };
    manager.within(EXCHANGE_WAIT);
    for instance in &surveyed.gone {
        let had_units: bool = has_units(folders, instance.as_str());
        if had_units {
            match stop_units(manager, instance.as_str()) {
                Ok(()) => outcome.stopped.push(instance.clone()),
                Err(error) => {
                    warn(
                        TAG,
                        format!(
                            "the units of {instance} were not stopped: {}",
                            reason_of(&error)
                        )
                        .as_str(),
                    );
                    outcome.failed.push((instance.clone(), reason_of(&error)));
                    continue;
                }
            }
        }
        // A stop answers when the manager queued the job, so one question more says the service's processes are gone,
        //  and only then may the journal it writes be removed. A journal with neither a socket file nor a note beside it
        //  belongs to no unit of this manager: the two files are what make one possible.
        let quiet: bool = if had_units {
            service_is_gone(manager, instance.as_str())
        } else {
            true
        };
        match remove_journal_of_a_stopped_instance(folders.state_dir, instance.as_str(), quiet) {
            Some(JournalAfterStop::HoldsWork(kept)) => {
                forget_the_instance(folders, instance.as_str());
                outcome.journals_kept.push((instance.clone(), kept));
            }
            // The journal is kept for the clients that may still read it, or for a service that is still going:
            //  the note and the socket file stay, so the next sweep finds the instance and finishes.
            Some(JournalAfterStop::StillRead(_)) => {}
            Some(JournalAfterStop::Removed) => {
                // Named before the folder goes, so the line a person reads says where it was.
                if let Some(journal) = journal_of_the_instance(folders.state_dir, instance.as_str())
                {
                    outcome.journals_removed.push((instance.clone(), journal));
                }
                forget_the_instance(folders, instance.as_str());
            }
            None => forget_the_instance(folders, instance.as_str()),
        }
    }
    return outcome;
}

/// Unit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_units_of_removed_worktrees_tests.rs"]
mod tests;
