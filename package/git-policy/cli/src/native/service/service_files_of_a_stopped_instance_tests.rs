//! Controls for the files an instance of the commit service keeps:
//! the note beside its socket,
//! the note and the socket file that say it has units,
//! and the journal that remembers its batches.
//! The manager is a recorder.

use super::*;
use crate::service_commit_looking_for_what_landed::{LookForWhatLanded, looking_for_what_landed};
use crate::service_instance::{instance_of, service_unit_instance};
use crate::service_journal::{CLAIM_EXTENSION, DONE_EXTENSION, Intent, OPEN_EXTENSION, SCRATCH_EXTENSION, START_EXTENSION, TEMPORARY_EXTENSION, intent_text};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::service_units_fixture_test_support::{Machine, age_a_journal, machine};
use crate::service_units_of_removed_worktrees::{stop_own_socket_of_a_gone_worktree, stop_units_of_gone_worktrees};
use crate::test_support::{fixture, git_context, remove};
use crate::transaction_git::GitContext;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use std::time::Duration;

/// The journal folder of an instance as the unit's `--journal` names it,
/// made for a machine of the controls.
fn journal_folder(state: &std::path::Path, instance: &str) -> PathBuf {
    let wrapper_state: PathBuf = state.join(SERVICE_UNIT.current);
    let journal: PathBuf = journal_of(wrapper_state.as_path(), instance);
    std::fs::create_dir_all(&journal).expect("journal folder");
    return journal;
}

/// What:
///  The note an instance keeps maps its worktree to it,
///  and a note whose text hashes to no instance names nothing.
/// Why:
///   The note is what a stop reads to name the units of a worktree that is gone.
#[test]
fn the_note_written_at_provisioning_names_the_worktree_of_its_instance() {
    let machine: Machine = machine("files-note");
    let removed: PathBuf = machine.linked("removed", false);
    let instance: String = instance_of(removed.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note written");
    assert!(note_path(machine.runtime.as_path(), instance.as_str()).exists());
    // Written again with the same text, it stays.
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note written again");
    remove(machine.root.as_path());
}

/// A note that cannot be written is a notice and never a failure:
/// the units then cannot be told apart later,
/// and nothing else changes.
#[test]
fn a_note_that_cannot_be_written_is_no_failure() {
    let root: PathBuf = fixture("files-note-refused");
    // The note lives under `<runtime>/systemd/user`, so a file where that folder belongs refuses it.
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    std::fs::write(runtime.join("systemd"), b"not a folder").expect("a file where the folder belongs");
    note_worktree(runtime.as_path(), "3f9a3f9a3f9a3f9a3f9a3f9a3f9a3f9a", root.join("repository/.git").as_path());
    assert!(!note_path(runtime.as_path(), "3f9a3f9a3f9a3f9a3f9a3f9a3f9a3f9a").exists());
    remove(root.as_path());
}

/// What:
///  A stopped instance's note and socket file go,
///  and so does its journal once every entry in it is the record of a batch that landed.
/// Why:
///   Nothing serves a worktree that is gone again,
/// and one state directory per removed worktree stayed for ever before this.
#[test]
fn the_sweep_takes_the_note_the_socket_file_and_the_journal_of_a_stopped_instance() {
    let machine: Machine = machine("files-sweep");
    let gone: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = machine.aged_settled_journal_of(instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    let outcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![instance.clone()]);
    assert!(!note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note goes with the units");
    assert!(!crate::service_instance::socket_path(machine.runtime.as_path(), instance.as_str()).exists(), "the socket file goes with the units");
    assert!(!journal.exists(), "the journal of a stopped instance is removed with its units");
    assert!(outcome.journals_kept.is_empty(), "{:?}", outcome.journals_kept);
    remove(machine.root.as_path());
}

/// What:
///  A service that finds its own worktree gone takes its own journal when nothing is unresolved,
///  so a removal Git ran with the wrapper replaced leaves no state directory behind.
/// Why:
///   The service's own stop removes the note and the socket file,
/// so no later sweep asks about that instance at all.
#[test]
fn the_service_takes_its_own_journal_when_nothing_is_unresolved() {
    let machine: Machine = machine("files-own");
    let gone: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = machine.aged_settled_journal_of(instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    let stopped: Option<PathBuf> =
        stop_own_socket_of_a_gone_worktree(&machine.folders(), instance.as_str(), &mut recorder).expect("stopped");
    assert_eq!(stopped, Some(gone));
    assert!(!journal.exists(), "the journal of the service's own gone worktree is removed with its socket");
    remove(machine.root.as_path());
}

/// What:
///  A journal that holds the record of a commit that has not landed stays,
///  with every file in it,
///  and the command is told which file kept it.
/// Why:
///   That record is the only place the request and the commit a person asked for are written together.
#[test]
fn a_journal_of_a_commit_that_has_not_landed_stays_and_is_named() {
    let machine: Machine = machine("files-unlanded");
    let gone: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = machine.aged_settled_journal_of(instance.as_str());
    let intent: PathBuf = journal.join(format!("7.{OPEN_EXTENSION}"));
    std::fs::write(&intent, b"seq=7\n").expect("a batch that did not land");
    let mut recorder: Recorder = Recorder::default();
    let outcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![instance.clone()]);
    assert!(intent.exists(), "the record of a commit that has not landed is kept");
    assert!(journal.join(format!("1.{DONE_EXTENSION}")).exists(), "a kept journal is kept whole");
    assert_eq!(outcome.journals_kept.len(), 1, "the command is told which journal stayed");
    assert_eq!(outcome.journals_kept[0].0, instance);
    assert_eq!(outcome.journals_kept[0].1.entry, format!("7.{OPEN_EXTENSION}"));
    assert_eq!(outcome.journals_kept[0].1.reason, "the record of a commit that has not landed");
    remove(machine.root.as_path());
}

/// What:
///  A journal of a landing the service was inside stays:
///  the step it holds is what tells a person the landing never ended.
#[test]
fn a_journal_of_a_landing_that_was_in_flight_stays() {
    let root: PathBuf = fixture("files-landing");
    let state: PathBuf = root.join("state");
    let journal: PathBuf = journal_folder(state.as_path(), "0000000000000000000000000000000c");
    let step: PathBuf = journal.join(LANDING_RECORD_DIRECTORY).join("step");
    std::fs::create_dir_all(step.parent().expect("the landing's record")).expect("landing record");
    std::fs::write(&step, b"promote").expect("the step of a landing");
    let kept: KeptJournal = match remove_journal_of_a_stopped_instance(Some(state.as_path()), "0000000000000000000000000000000c", true) {
        Some(JournalAfterStop::HoldsWork(kept)) => kept,
        other => panic!("a landing that was in flight keeps the journal and is named: {other:?}"),
    };
    assert_eq!(kept.entry, LANDING_RECORD_DIRECTORY);
    assert!(step.exists(), "the step of a landing that did not end is kept");
    remove(root.as_path());
}

/// What:
///  A journal that holds a record which was written whole and never published stays,
///  because that file can name a commit no branch shows;
///  a scratch record and a batch being landed keep it too.
/// Why:
///   Publishing renames a record into place,
/// so a kill between the write and the rename leaves the only copy under the temporary name.
#[test]
fn a_journal_of_a_record_no_branch_shows_stays() {
    let root: PathBuf = fixture("files-unpublished");
    let state: PathBuf = root.join("state");
    for (name, reason) in [
        (format!("3.{TEMPORARY_EXTENSION}"), "a record that was written and not published"),
        (format!("4.{SCRATCH_EXTENSION}"), "the record of a batch whose objects a scratch reference may hold"),
        (format!("5.{CLAIM_EXTENSION}"), "the record of a batch that was being landed"),
        (format!("6.{START_EXTENSION}"), "the record of a batch that was being landed"),
        (String::from("notes-from-a-later-release.txt"), "a file this release does not know"),
    ] {
        let journal: PathBuf = journal_folder(state.as_path(), "0000000000000000000000000000000d");
        std::fs::write(journal.join(name.as_str()), b"seq=3\n").expect("a record");
        let kept: KeptJournal = match remove_journal_of_a_stopped_instance(Some(state.as_path()), "0000000000000000000000000000000d", true) {
            Some(JournalAfterStop::HoldsWork(kept)) => kept,
            other => panic!("{name} keeps the journal and is named: {other:?}"),
        };
        assert_eq!(kept.entry, name);
        assert_eq!(kept.reason, reason);
        assert!(journal.exists(), "{name} keeps the journal");
        std::fs::remove_dir_all(journal.as_path()).expect("the control's own journal");
    }
    remove(root.as_path());
}

/// The record of a batch that landed,
/// naming one request and the commit it became.
fn a_landed_batch(commit: &str, request: &str) -> Intent {
    return Intent {
        seq: 7,
        repository: PathBuf::from("/nowhere/repository"),
        common_dir: PathBuf::from("/nowhere/repository/.git"),
        reference: String::from("refs/heads/main"),
        old: "2".repeat(40),
        new: String::from(commit),
        ids: vec![String::from(request)],
        commits: vec![String::from(commit)],
        paths: Vec::new(),
        derived: true,
        cleanup: None,
        index: PathBuf::from("/nowhere/repository/.git/index"),
        git_dir: PathBuf::from("/nowhere/repository/.git"),
    };
}

/// The commit the record of `request` names,
/// as the client of a commit whose answer was lost looks for it.
fn looking_up(journal: &std::path::Path, request: &str) -> Option<String> {
    let context: GitContext = git_context();
    return looking_for_what_landed(LookForWhatLanded { context: &context, cwd: journal, journal: Some(journal), id: request, wait: Duration::from_millis(200) });
}

/// What:
///  A journal whose only entries are the records of landed batches keeps those records while a client whose answer was lost may still read them,
///  and the look that turns such a client's failure into its commit finds it.
/// Why:
///   The finished record is the only place the identity of a request and the commit it became are written together,
/// so a cleanup that removes it first makes the same look report a commit that landed as a failure.
#[test]
fn a_landed_batch_a_lost_answer_may_still_read_keeps_the_journal() {
    let root: PathBuf = fixture("files-lost-answer");
    let state: PathBuf = root.join("state");
    let instance: &str = "0000000000000000000000000000000a";
    let request: &str = "0d1e2f3a-4b5c-6d7e-8f90-a1b2c3d4e5f6";
    let commit: String = "1".repeat(40);
    let journal: PathBuf = journal_folder(state.as_path(), instance);
    std::fs::write(journal.join(format!("7.{DONE_EXTENSION}")), intent_text(&a_landed_batch(commit.as_str(), request))).expect("a finished batch");
    assert_eq!(looking_up(journal.as_path(), request), Some(commit.clone()), "the record is what a client whose answer was lost reads");
    assert_eq!(
        remove_journal_of_a_stopped_instance(Some(state.as_path()), instance, true),
        Some(JournalAfterStop::StillRead(KeptJournal {
            journal: journal.clone(),
            entry: String::from("7.done"),
            reason: String::from(A_LOST_ANSWER_MAY_READ_IT),
        })),
        "a landed record a client may still read keeps the journal, quietly",
    );
    assert!(journal.exists(), "the records of its landed batches stay while a client of the instance may still read them");
    assert_eq!(looking_up(journal.as_path(), request), Some(commit), "the cleanup took the only record that tells a lost answer what its commit became");
    remove(root.as_path());
}

/// What:
///  A journal is not removed while the service of its instance may still be running,
///  and the files that name the instance stay with it,
///  so a later sweep finishes the removal.
/// Why:
///   The manager stops a unit by queueing a job and answering,
/// so a listing that says the journal holds nothing unresolved and the deletion of it are two steps
///  with the service's own writes between them:
/// the deletion then takes a record the service published in between,
/// or makes its next write fail.
#[test]
fn a_service_that_may_still_run_keeps_the_journal_and_the_files_that_name_it() {
    let machine: Machine = machine("files-still-running");
    let gone: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = machine.aged_settled_journal_of(instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    recorder.report_state_of(service_unit_instance(instance.as_str()).as_str(), Some("active"));
    let outcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![instance.clone()]);
    assert!(journal.exists(), "the journal stays while its service may still write to it");
    assert!(note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note stays, so a later sweep finds the instance again");
    assert_eq!(outcome.journals_kept.len(), 0, "nothing in the journal needs a person: only its service has to be gone");
    remove(machine.root.as_path());
}

/// What:
///  The journal is removed once the manager reports the service gone,
///  and the files that name the instance go with it.
/// Why:
///   A wait that ends with the unit gone is what makes the deletion safe,
/// and one that ends without it must not delete anything.
#[test]
fn the_journal_goes_once_the_manager_reports_the_service_gone() {
    let machine: Machine = machine("files-service-gone");
    let gone: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = machine.aged_settled_journal_of(instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    recorder.report_states_of(service_unit_instance(instance.as_str()).as_str(), &[Some("deactivating"), Some("inactive")]);
    let outcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![instance.clone()]);
    assert!(!journal.exists(), "a journal whose records are old and whose service is gone is removed");
    assert!(!note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note goes with it");
    remove(machine.root.as_path());
}

/// A journal that is not there is nothing left of it,
/// and a state directory the caller does not know removes nothing.
#[test]
fn a_missing_journal_and_an_unknown_state_directory_remove_nothing() {
    let root: PathBuf = fixture("files-missing");
    let state: PathBuf = root.join("state");
    std::fs::create_dir_all(&state).expect("state directory");
    assert_eq!(remove_journal_of_a_stopped_instance(Some(state.as_path()), "0000000000000000000000000000000e", true), Some(JournalAfterStop::Removed));
    let journal: PathBuf = journal_folder(state.as_path(), "0000000000000000000000000000000f");
    std::fs::write(journal.join(format!("1.{DONE_EXTENSION}")), b"seq=1\n").expect("a finished batch");
    age_a_journal(journal.as_path());
    assert_eq!(remove_journal_of_a_stopped_instance(None, "0000000000000000000000000000000f", true), None);
    assert!(journal.exists(), "an unknown state directory keeps the journal");
    remove(root.as_path());
}
