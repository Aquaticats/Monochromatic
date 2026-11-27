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
//! because a file that stayed would count the instance again and the next sweep stop it again;
//! they stay only beside a journal that is kept for the clients that may still read it,
//! which a later sweep removes together with them.
//! The journal goes only when every entry in it is the record of a batch that landed
//! and no client of that batch can still read the record:
//! a client whose answer was lost looks for the commit its request became in the record of its batch
//! (`service_commit_looking_for_what_landed.rs`),
//! so a record written within `A_FINISHED_RECORD_IS_READ` keeps the journal for the clients that can still read it.
//! That age is judged on the reading of the clocks the record itself carries,
//! so a wall clock that steps,
//! or a host that resumed from sleep,
//! never makes a record of this boot look older than the wait its client is inside
//! (`service_journal_record_age.rs`);
//! a record an earlier release wrote carries no reading and is judged by its modification time.
//! A commit that has not landed,
//! a batch whose objects a scratch reference may hold,
//! a record that was written and not published,
//! a landing or withdrawal record,
//! a claim watch note naming a repository that still holds the work directory a look reads,
//! and any name this release does not explain keep the whole journal,
//! which is then named to the person,
//! because work they may still want is never discarded.
//! Two kinds of entry that asked for a person at every removal are not work:
//! the repository the service itself remembered for its rules warm (`rules-repository`,
//! `service_rules_precompile.rs`),
//! which names a repository and carries no record of a commit,
//! and a claim watch note that names the removed worktree's own Git directory,
//! or one whose work directory is gone:
//! a look at such a note finds `<Git directory>/monochromatic-git-policy-cli/work/pending`,
//! which went with the directory it was under (`service_claim_watch.rs`).
//! A batch's copy of the conclusion state (`<batch>.cleanup`) is not one of them:
//! it is a copy of what the landing compares,
//! made for a worktree whose units are stopped and whose files are gone,
//! so it goes with the journal.
//! A service that may still write keeps the journal and the files that name the instance,
//! so a later sweep finishes then.
//! Nothing here starts a unit,
//! and nothing here asks a manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! recordWorktree(runtimeDir, instance, gitDir); // the note
//! forgetStoppedInstance(runtimeDir, instance); // note and socket file
//! removeJournalOfAStoppedInstance(stateDir, instance, serviceIsGone); // 'removed', or what kept it
//! ```
//!
//! The note beside the socket is `service_files_of_a_stopped_instance_note_beside_socket.rs`,
//! and what one journal entry means is `service_files_of_a_stopped_instance_journal_entries.rs`;
//! this file decides what a stopped instance's journal comes to and removes the files that named it.

pub use super::service_files_of_a_stopped_instance_journal_entries::{
    JournalAfterStop,
    KeptJournal,
};
pub(crate) use super::service_files_of_a_stopped_instance_journal_entries::{
    Kept,
    kept_entry_of,
};
pub(crate) use super::service_files_of_a_stopped_instance_note_beside_socket::TAG;
pub use super::service_files_of_a_stopped_instance_note_beside_socket::{
    NOTE_ENDING,
    NOTES_FOLDER,
    note_path,
    note_worktree,
    record_worktree,
};
use crate::claim_time::ClaimTime;
use crate::diagnostic_log::warn;
use crate::service_instance::{
    service_unit_instance,
    socket_path,
};
use crate::service_journal_folder_of_an_instance::journal_of;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The journal folder of an instance,
///  and nothing when the caller has no state directory to look in.
/// Why:
///   The folder is named in the lines that keep it,
/// so the words and the removal name the same path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalOfTheInstance(stateDir, instance) // stateDir && join(stateDir, SERVICE_UNIT, 'service', instance)
/// ```
pub fn journal_of_the_instance(
    state_dir: Option<&Path>,
    instance: &str,
) -> Option<PathBuf> {
    let state: &Path = state_dir?;
    return Some(journal_of(
        state.join(SERVICE_UNIT.current).as_path(),
        instance,
    ));
}

/// What:
///  The first entry of a stopped instance's journal that is work a person may still want,
///  with the words that say what it is,
///  and nothing when every entry is a landed batch's record or a folder the service left empty.
/// Why:
///   A service that stops its own socket never removes its own journal,
/// because it is still one of its writers (`service_units_of_removed_worktrees.rs`),
/// and work in that journal is still the person's business:
/// it is named in the service's own log so they can look,
/// and the sweep names it again when it finally removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// workAPersonMayWant(stateDir, instance) // { journal, entry, reason } or undefined
/// ```
pub fn work_a_person_may_want(
    state_dir: Option<&Path>,
    instance: &str,
) -> Option<KeptJournal> {
    let journal: PathBuf = journal_of_the_instance(state_dir, instance)?;
    return match kept_entry_of(journal.as_path(), &ClaimTime::read()) {
        Ok(Some(Kept::Work(entry, reason))) => Some(KeptJournal {
            journal,
            entry,
            reason,
        }),
        Err(reason) => Some(KeptJournal {
            journal,
            entry: String::from("something this command could not read"),
            reason,
        }),
        _ => None,
    };
}

/// What:
///  Removes the service journal of an instance whose units were stopped,
///  when every entry in it is the record of a batch that landed that no client can still read,
///  given that the service which writes it is gone,
///  and says what kept it otherwise.
/// Why:
///   Keeping those records left one state directory per removed worktree with up to `DONE_RETAINED` files in it,
/// and nothing swept them;
/// the note and the socket file go with the journal,
/// so no later survey asks this instance about its worktree.
/// A journal that holds anything else is kept whole and reported,
/// a journal a client may still read is kept quietly for it,
/// and a service that may still write keeps the journal and the files that name it,
/// which a later sweep tries again.
/// A state directory that is not known removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeJournalOfAStoppedInstance(stateDir, instance, serviceIsGone) // 'removed' | { journal, entry, reason } | { keptForAReader: journal }
/// ```
pub fn remove_journal_of_a_stopped_instance(
    state_dir: Option<&Path>,
    instance: &str,
    service_is_gone: bool,
) -> Option<JournalAfterStop> {
    let state: &Path = state_dir?;
    let journal: PathBuf = journal_of_the_instance(Some(state), instance)?;
    // A service that may still run writes into the journal, so nothing about it is decided here.
    if !service_is_gone {
        return Some(JournalAfterStop::StillRead(KeptJournal {
            journal,
            entry: service_unit_instance(instance),
            reason: String::from("the service that writes it may still be running"),
        }));
    }
    return Some(match kept_entry_of(journal.as_path(), &ClaimTime::read()) {
        Ok(None) => {
            if let Err(error) = std::fs::remove_dir_all(journal.as_path())
                && error.kind() != std::io::ErrorKind::NotFound
            {
                warn(TAG, format!("the journal of {instance} could not be removed ({error}), so it is left at {}", journal.display()).as_str());
                JournalAfterStop::StillRead(KeptJournal {
                    journal,
                    entry: String::from("something this command could not remove"),
                    reason: String::from("it could not be removed"),
                })
            } else {
                JournalAfterStop::Removed
            }
        }
        Ok(Some(Kept::Work(entry, reason))) => JournalAfterStop::HoldsWork(KeptJournal {
            journal,
            entry,
            reason,
        }),
        Ok(Some(Kept::Read(entry, reason))) => JournalAfterStop::StillRead(KeptJournal {
            journal,
            entry,
            reason: String::from(reason),
        }),
        Err(reason) => JournalAfterStop::HoldsWork(KeptJournal {
            journal,
            entry: String::from("something this command could not read"),
            reason,
        }),
    });
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
pub fn forget_stopped_instance(
    runtime_dir: &Path,
    instance: &str,
) {
    for path in [
        note_path(runtime_dir, instance),
        socket_path(runtime_dir, instance),
    ] {
        if let Err(error) = std::fs::remove_file(path.as_path())
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn(
                TAG,
                format!(
                    "{} could not be removed after the units of {instance} were stopped ({error})",
                    path.display()
                )
                .as_str(),
            );
        }
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_files_of_a_stopped_instance_tests.rs"]
mod tests;
