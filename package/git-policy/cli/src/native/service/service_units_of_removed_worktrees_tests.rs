//! Controls for the commit service units of worktrees that no longer exist:
//! the note,
//! the test of a gone worktree,
//! the survey and the stop.
//! The manager is a recorder.

use super::*;
use crate::service_instance::{instance_of, socket_path};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::service_units_fixture_test_support::{Machine, age_a_journal, machine};
use crate::test_support::{git, remove, repository};

/// The units of a worktree that was removed are stopped by name through the manager,
/// the socket first so nothing starts the service again,
/// and a unit the manager does not hold is not an error.
#[test]
fn the_socket_and_then_the_service_of_an_instance_are_stopped_through_the_manager() {
    let mut recorder: Recorder = Recorder::default();
    stop_units(&mut recorder, "abc").expect("stopped");
    assert_eq!(recorder.stops(), vec![format!("stop {}", socket_unit_instance("abc")), format!("stop {}", service_unit_instance("abc"))]);
}

#[test]
fn a_unit_the_manager_does_not_hold_is_already_stopped_and_any_other_refusal_is_returned() {
    let mut missing: Recorder = Recorder::default();
    missing.refuse_stop_of(service_unit_instance("abc").as_str(), CallError::Manager { name: String::from(NO_SUCH_UNIT), text: String::new() });
    stop_units(&mut missing, "abc").expect("a unit that is not there is stopped");
    let mut refused: Recorder = Recorder::default();
    refused.refuse_stop_of(socket_unit_instance("abc").as_str(), CallError::TimedOut);
    assert_eq!(stop_units(&mut refused, "abc"), Err(CallError::TimedOut));
    assert_eq!(refused.stops().len(), 2, "a refused socket stop still stops the service: {:?}", refused.stops());
}

/// The state directory the service's journal sits in is read from the journal path the unit passes,
/// so the service can tell whose units these are from the records of its own journal too;
/// a journal of another shape names none.
#[test]
fn the_state_directory_is_read_from_the_journal_the_unit_passes() {
    let state: PathBuf = PathBuf::from("/state");
    let instance: &str = "3f9a3f9a3f9a3f9a3f9a3f9a3f9a3f9a";
    let journal: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current).join("service").join(instance);
    assert_eq!(state_dir_of_journal(journal.as_path(), instance), Some(state));
    assert_eq!(state_dir_of_journal(Path::new("/elsewhere/journal"), instance), None);
    assert_eq!(state_dir_of_journal(journal.as_path(), "ffffffffffffffffffffffffffffffff"), None, "a journal of another instance names no state directory");
}

/// A worktree is gone only when its Git directory is missing while the place that holds it is still there:
/// a repository on a drive that is not mounted proves nothing.
#[test]
fn a_linked_worktree_is_gone_when_its_directory_is_missing_under_a_common_directory_that_is_there() {
    let machine: Machine = machine("units-gone-linked");
    let present: PathBuf = machine.linked("kept", true);
    let removed: PathBuf = machine.linked("removed", false);
    assert!(!worktree_is_gone(present.as_path()));
    assert!(worktree_is_gone(removed.as_path()));
    remove(machine.root.as_path());
}

/// What:
///  A worktree whose whole repository was deleted is gone:
///  the Git directory is missing,
///  the directory that held it is missing too,
///  and the directory above that one holds other entries.
/// Why:
///   This is the state a throwaway repository is left in when a proof deletes it:
/// on 2026-11-11 a deleted `/tmp/lfs-login-proof-*/repository-a` kept its units listening and its service retrying pushes,
/// because a missing place read as one a drive may still fill.
#[test]
fn a_worktree_whose_whole_repository_was_deleted_is_gone() {
    let machine: Machine = machine("units-gone-repository-deleted");
    let holder: PathBuf = machine.root.join("work");
    let repository: PathBuf = holder.join("repository-a");
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory of a repository");
    std::fs::write(holder.join("kept.txt"), "another entry").expect("an entry beside it");
    assert!(!worktree_is_gone(git_dir.as_path()), "a repository that is there is not gone");
    std::fs::remove_dir_all(repository.as_path()).expect("the repository is deleted");
    assert!(
        worktree_is_gone(git_dir.as_path()),
        "the directory that held the repository holds other entries, so the repository was deleted and nothing is left to serve"
    );
    remove(machine.root.as_path());
}

/// What:
///  A worktree in a repository that cannot be reached is not gone:
///  its mount point is there and holds nothing,
///  as a drive that is not mounted leaves it.
/// Why:
///   The proof that a repository is gone rests on the nearest directory above the Git directory that exists
///  (`unreachable_git_dir_proven_gone.rs`):
/// an empty one may be waiting for a drive,
/// and a directory holding other entries is one a repository was deleted from.
#[test]
fn a_worktree_in_a_repository_that_cannot_be_reached_is_not_gone() {
    let machine: Machine = machine("units-gone-unreachable");
    // The mount point is there and empty, as it is when a drive was not mounted: every path below it is missing.
    std::fs::create_dir_all(machine.root.join("drive-not-mounted")).expect("mount point");
    let unreachable: PathBuf = machine.root.join("drive-not-mounted/repository/.git/worktrees/topic");
    assert!(!worktree_is_gone(unreachable.as_path()));
    remove(machine.root.as_path());
}

#[test]
fn a_main_checkout_is_gone_when_its_git_directory_is_missing_beside_a_checkout_that_is_there() {
    let machine: Machine = machine("units-gone-main");
    std::fs::create_dir_all(machine.root.join("checkout")).expect("checkout");
    assert!(worktree_is_gone(machine.root.join("checkout/.git").as_path()));
    std::fs::create_dir_all(machine.root.join("checkout/.git")).expect("git directory");
    assert!(!worktree_is_gone(machine.root.join("checkout/.git").as_path()));
    remove(machine.root.as_path());
}

/// The note makes an instance's worktree known,
/// and a note whose text does not hash to the instance names nothing.
#[test]
fn the_note_written_at_provisioning_maps_the_instance_to_its_worktree() {
    let machine: Machine = machine("units-note");
    let removed: PathBuf = machine.linked("removed", false);
    let instance: String = instance_of(removed.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note written");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed.clone()));
    // Written again with the same text, it stays.
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note written again");
    let other: String = instance_of(machine.linked("another", false).as_path());
    std::fs::write(note_path(machine.runtime.as_path(), other.as_str()), removed.to_string_lossy().as_bytes()).expect("a note of another worktree");
    assert_eq!(worktree_of(&machine.folders(), other.as_str()), UnitWorktree::Unmapped);
    remove(machine.root.as_path());
}

/// What:
///  The note of a worktree whose Git directory is not text still maps its instance to that directory.
/// Why:
///   The note is read back and its path is hashed to find the instance it belongs to,
/// so a note that lost the path's bytes names no instance,
/// and the units of a removed worktree are never stopped (the survey says "unmapped").
#[test]
fn the_note_of_a_directory_that_is_not_text_maps_its_instance() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let machine: Machine = machine("units-note-bytes");
    let removed: PathBuf = machine.linked("removed", false).join(OsStr::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(removed.as_path()).expect("Git directory");
    let instance: String = instance_of(removed.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note written");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed));
    remove(machine.root.as_path());
}

/// What:
///  A note an earlier release wrote for a worktree whose Git directory ends with a line break names that worktree.
/// Why:
///   That release wrote the text and nothing after it,
/// so trimming one line break from the note names the sibling directory,
/// whose text hashes to another instance,
/// and the units of the removed worktree are never found again.
#[test]
fn a_note_of_an_earlier_release_whose_path_ends_with_a_line_break_names_that_worktree() {
    let machine: Machine = machine("units-note-line-break");
    let removed: PathBuf = machine.linked("topic\n", false);
    let sibling: PathBuf = machine.linked("topic", false);
    std::fs::create_dir_all(sibling.as_path()).expect("the sibling's Git directory");
    let instance: String = instance_of(removed.as_path());
    let note: PathBuf = note_path(machine.runtime.as_path(), instance.as_str());
    std::fs::create_dir_all(note.parent().expect("parent")).expect("the note folder");
    std::fs::write(note.as_path(), removed.to_string_lossy().as_bytes()).expect("an earlier note");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed), "the note names the worktree it was written for");
    remove(machine.root.as_path());
}

/// What:
///  A note written by a release that spelled the directory as plain text is still read.
/// Why:
///   An upgrade must not lose the units it can already name,
/// so the older form has to keep mapping.
#[test]
fn a_note_of_plain_text_is_still_read() {
    let machine: Machine = machine("units-note-plain");
    let removed: PathBuf = machine.linked("removed", false);
    let instance: String = instance_of(removed.as_path());
    let note: PathBuf = note_path(machine.runtime.as_path(), instance.as_str());
    std::fs::create_dir_all(note.parent().expect("parent")).expect("the note folder");
    std::fs::write(note.as_path(), removed.to_string_lossy().as_bytes()).expect("an earlier note");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed));
    remove(machine.root.as_path());
}

/// What:
///  An instance with no note is mapped by the `gitdirHex=` line of a record in its service journal.
/// Why:
///   The service writes the bytes beside the plain text in its records,
/// and that line is what names a directory whose path is not text.
#[test]
fn an_instance_is_mapped_by_the_bytes_in_its_journal_records() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let machine: Machine = machine("units-journal-bytes");
    let removed: PathBuf = machine.linked("removed", false).join(OsStr::from_bytes(b"c\xffmmon"));
    std::fs::create_dir_all(removed.as_path()).expect("Git directory");
    let instance: String = instance_of(removed.as_path());
    let journal: PathBuf = machine.state.join("monochromatic-git-policy-cli/service").join(instance.as_str());
    std::fs::create_dir_all(&journal).expect("journal");
    let bytes: String = crate::path_bytes_in_records::hex_of_path(removed.as_path());
    std::fs::write(journal.join("1.done"), format!("seq=1\ngitdirHex={bytes}\n")).expect("record");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed));
    remove(machine.root.as_path());
}

/// An instance with no note is mapped by the `gitdir=` line of a record in its service journal,
/// when that line's text hashes to the instance.
#[test]
fn an_instance_without_a_note_is_mapped_by_its_journal_records() {
    let machine: Machine = machine("units-journal");
    let removed: PathBuf = machine.linked("removed", false);
    let instance: String = instance_of(removed.as_path());
    let journal: PathBuf = machine.state.join("monochromatic-git-policy-cli/service").join(instance.as_str());
    std::fs::create_dir_all(&journal).expect("journal");
    std::fs::write(journal.join("1.done"), format!("seq=1\ngitdir={}\n", removed.display())).expect("record");
    assert_eq!(worktree_of(&machine.folders(), instance.as_str()), UnitWorktree::Known(removed));
    let stranger: String = String::from("ffffffffffffffffffffffffffffffff");
    let stranger_journal: PathBuf = machine.state.join("monochromatic-git-policy-cli/service").join(stranger.as_str());
    std::fs::create_dir_all(&stranger_journal).expect("journal");
    std::fs::write(stranger_journal.join("1.done"), format!("gitdir={}\n", machine.root.join("somewhere").display())).expect("record");
    assert_eq!(worktree_of(&machine.folders(), stranger.as_str()), UnitWorktree::Unmapped);
    remove(machine.root.as_path());
}

/// The survey sorts the instances that have a socket or a note.
#[test]
fn the_survey_names_the_gone_the_unmapped_and_counts_the_present() {
    let machine: Machine = machine("units-survey");
    let kept: PathBuf = machine.linked("kept", true);
    let gone: PathBuf = machine.linked("gone", false);
    let kept_instance: String = instance_of(kept.as_path());
    let gone_instance: String = instance_of(gone.as_path());
    let unknown: String = String::from("00000000000000000000000000000001");
    for (instance, git_dir) in [(&kept_instance, &kept), (&gone_instance, &gone)] {
        record_worktree(machine.runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("note");
        machine.socket_of(instance.as_str());
    }
    machine.socket_of(unknown.as_str());
    let found: Survey = survey(&machine.folders());
    assert_eq!(found, Survey { gone: vec![gone_instance], unmapped: vec![unknown], present: 1 });
    remove(machine.root.as_path());
}

/// Only the units of a gone worktree are stopped,
/// through the manager,
/// and its note goes with them;
/// a present worktree's and an unmapped instance's units are never asked about.
#[test]
fn the_sweep_stops_the_units_of_gone_worktrees_only() {
    let machine: Machine = machine("units-sweep");
    let kept: PathBuf = machine.linked("kept", true);
    let gone: PathBuf = machine.linked("gone", false);
    let kept_instance: String = instance_of(kept.as_path());
    let gone_instance: String = instance_of(gone.as_path());
    let unknown: String = String::from("00000000000000000000000000000001");
    for (instance, git_dir) in [(&kept_instance, &kept), (&gone_instance, &gone)] {
        record_worktree(machine.runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("note");
        machine.socket_of(instance.as_str());
    }
    machine.socket_of(unknown.as_str());
    let mut recorder: Recorder = Recorder::default();
    let outcome: SweepOutcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![gone_instance.clone()]);
    assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
    assert_eq!(outcome.survey.unmapped, vec![unknown.clone()]);
    assert_eq!(recorder.stops(), vec![format!("stop {}", socket_unit_instance(gone_instance.as_str())), format!("stop {}", service_unit_instance(gone_instance.as_str()))]);
    assert!(!note_path(machine.runtime.as_path(), gone_instance.as_str()).exists(), "the stopped instance's note is removed");
    assert!(!socket_path(machine.runtime.as_path(), gone_instance.as_str()).exists(), "the stopped instance's socket file is removed, so it is not counted or stopped again");
    assert!(socket_path(machine.runtime.as_path(), kept_instance.as_str()).exists(), "a present worktree's socket file stays");
    assert_eq!(survey(&machine.folders()), Survey { gone: Vec::new(), unmapped: vec![unknown], present: 1 });
    assert!(note_path(machine.runtime.as_path(), kept_instance.as_str()).exists(), "a present worktree's note stays");
    remove(machine.root.as_path());
}

/// What:
///  A service that starts for a worktree that is gone stops its own socket and serves nothing:
///  the socket goes first so no connection starts it again,
///  and the service unit is never asked to stop,
///  because the service's own end is what stops it.
/// Why:
///   The journal is not removed here,
/// because this process is still one of its writers;
/// the note and the socket file stay so the sweep that asks the manager whether the service is gone
/// finds the instance and removes the journal then.
#[test]
fn a_service_whose_worktree_is_gone_stops_its_own_socket_and_serves_nothing() {
    let machine: Machine = machine("units-own-gone");
    let removed: PathBuf = machine.linked("gone", false);
    let instance: String = instance_of(removed.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), removed.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    let gone: Option<PathBuf> = stop_own_socket_of_a_gone_worktree(&machine.folders(), instance.as_str(), &mut recorder).expect("stopped");
    assert_eq!(gone, Some(removed));
    assert_eq!(recorder.stops(), vec![format!("stop-alone {}", socket_unit_instance(instance.as_str()))]);
    assert!(note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note stays, so the sweep that follows this process's exit finds the instance");
    assert!(socket_path(machine.runtime.as_path(), instance.as_str()).exists(), "the socket file stays with the note, and a later sweep removes both");
    remove(machine.root.as_path());
}

/// A worktree on a device that is simply not mounted is not gone,
/// so the service that starts for it serves instead of stopping:
/// the missing administrative directory proves nothing while the repository that held it is missing too.
#[test]
fn a_service_whose_worktree_is_on_a_device_that_is_not_mounted_serves() {
    let machine: Machine = machine("units-own-unmounted");
    // The mount point is there and empty, as it is when a drive was not mounted: every path below it is missing.
    std::fs::create_dir_all(machine.root.join("drive-not-mounted")).expect("mount point");
    let unmounted: PathBuf = machine.root.join("drive-not-mounted/repository/.git/worktrees/topic");
    let instance: String = instance_of(unmounted.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), unmounted.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    assert_eq!(own_worktree_gone(&machine.folders(), instance.as_str()), None);
    let mut recorder: Recorder = Recorder::default();
    let gone: Option<PathBuf> = stop_own_socket_of_a_gone_worktree(&machine.folders(), instance.as_str(), &mut recorder).expect("nothing stopped");
    assert_eq!(gone, None);
    assert!(recorder.stops().is_empty(), "{:?}", recorder.stops());
    assert!(note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note of a worktree on an unmounted device stays");
    remove(machine.root.as_path());
}

/// An instance that maps to nothing and one whose worktree is still there are both served:
/// a wrong guess would cut the units of a live worktree.
#[test]
fn a_service_whose_worktree_cannot_be_told_serves() {
    let machine: Machine = machine("units-own-unknown");
    let kept: PathBuf = machine.linked("kept", true);
    let kept_instance: String = instance_of(kept.as_path());
    record_worktree(machine.runtime.as_path(), kept_instance.as_str(), kept.as_path()).expect("note");
    machine.socket_of(kept_instance.as_str());
    let unknown: String = String::from("00000000000000000000000000000001");
    machine.socket_of(unknown.as_str());
    assert_eq!(own_worktree_gone(&machine.folders(), kept_instance.as_str()), None);
    assert_eq!(own_worktree_gone(&machine.folders(), unknown.as_str()), None);
    let mut recorder: Recorder = Recorder::default();
    assert_eq!(stop_own_socket_of_a_gone_worktree(&machine.folders(), kept_instance.as_str(), &mut recorder), Ok(None));
    assert_eq!(stop_own_socket_of_a_gone_worktree(&machine.folders(), unknown.as_str(), &mut recorder), Ok(None));
    assert!(recorder.stops().is_empty(), "{:?}", recorder.stops());
    remove(machine.root.as_path());
}

/// A journal as a service of a worktree that committed long ago leaves it,
/// with one plan of a push waiting for a new try.
/// Why:
///   A journal younger than `A_FINISHED_RECORD_IS_READ` is kept for the client that may still read its landed batches,
/// so a control that judges what happens to a journal once those records may go needs one older than that,
/// and the plan is then the only entry that keeps it at all.
/// The journal is aged after the plan is written into it,
/// because writing that entry is a write to the journal's own folder.
fn a_journal_with_a_plan(machine: &Machine, instance: &str, git_dir: &Path, repository: &Path, oid: &str) -> PathBuf {
    let journal: PathBuf = machine.settled_journal_of(instance);
    let plan: crate::push_retry_store::StoredPlan = crate::push_retry_store::StoredPlan {
        git_dir: git_dir.to_path_buf(),
        item: crate::work_item::WorkItem {
            kind: crate::work_item::WorkKind::Push,
            repository: repository.to_path_buf(),
            git_dir: git_dir.to_path_buf(),
            branch_ref: String::from("refs/heads/main"),
            oid: String::from(oid),
            parent: String::from("fedcba9876543210fedcba9876543210fedcba98"),
            created: String::from("2026-11-11T08:55:00.000-05:00"),
            tries: 148,
        },
        attempt: 149,
    };
    crate::push_retry_store::save(journal.join(crate::push_retry::PLAN_DIRECTORY).as_path(), &plan).expect("the plan is kept");
    age_a_journal(journal.as_path());
    return journal;
}

/// One repository that is then deleted,
/// beside an entry that keeps the directory that held it holding something.
fn a_deleted_repository_under(machine: &Machine, name: &str) -> (PathBuf, PathBuf) {
    let holder: PathBuf = machine.root.join("work");
    let repository: PathBuf = holder.join(name);
    let git_dir: PathBuf = repository.join(".git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the Git directory of a repository");
    std::fs::write(holder.join("kept.txt"), "another entry").expect("an entry beside it");
    std::fs::remove_dir_all(repository.as_path()).expect("the repository is deleted");
    return (git_dir, repository);
}

/// What:
///  The pushes a service kept for a repository that is gone end with it,
///  and the journal stays for the sweep that follows the service's own end.
/// Why:
///   A plan that stayed would be read by a service of this instance started later,
/// which would spend a try on a directory nothing can reach.
/// The journal is not removed here,
/// because the service that stops its own socket is still one of its writers:
/// its after-commit work publishes records on a thread of its own,
/// and a record published between a listing and a removal goes with the tree
/// (`service_files_of_a_stopped_instance.rs`).
/// The sweep proves the service gone before it removes the journal,
/// so the records of landed batches no client may still read are removed later and never discarded now.
#[test]
fn a_service_of_a_gone_worktree_ends_the_plan_of_that_repository_and_keeps_the_journal_for_the_sweep() {
    let machine: Machine = machine("units-own-gone-plans");
    let (git_dir, repository) = a_deleted_repository_under(&machine, "repository-a");
    let instance: String = instance_of(git_dir.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = a_journal_with_a_plan(&machine, instance.as_str(), git_dir.as_path(), repository.as_path(), "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let mut recorder: Recorder = Recorder::default();
    let gone: Option<PathBuf> = stop_own_socket_of_a_gone_worktree(&machine.folders(), instance.as_str(), &mut recorder).expect("stopped");
    assert_eq!(gone, Some(git_dir.clone()));
    assert!(crate::push_retry_store::load(journal.join(crate::push_retry::PLAN_DIRECTORY).as_path()).is_empty(), "the plan of the gone repository ended");
    assert!(journal.exists(), "the journal stays, because this service is still one of its writers");
    assert!(note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note stays, so the sweep that follows this process's end finds the instance");
    assert!(socket_path(machine.runtime.as_path(), instance.as_str()).exists(), "the socket file stays with the note, and the sweep removes both");
    // The sweep is a later process, and the manager there reports the service gone, which is what lets the journal go.
    let swept: SweepOutcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(swept.stopped, vec![instance.clone()]);
    assert!(swept.journals_kept.is_empty(), "{:?}", swept.journals_kept);
    assert!(!journal.exists(), "the sweep removed the journal once the manager reported the service gone");
    remove(machine.root.as_path());
}

/// What:
///  A plan of a repository that is still there keeps the journal,
///  and the plan of the gone repository beside it ends.
/// Why:
///   A commit a person made is never discarded on a guess:
/// the journal of an instance may hold the work of a repository that exists,
/// and that work stays and is named.
#[test]
fn a_journal_that_keeps_a_plan_of_a_repository_that_is_there_stays() {
    let machine: Machine = machine("units-own-gone-plans-kept");
    let (git_dir, repository) = a_deleted_repository_under(&machine, "repository-a");
    let alive_repository: PathBuf = machine.root.join("alive");
    let alive_git_dir: PathBuf = alive_repository.join(".git");
    std::fs::create_dir_all(alive_git_dir.as_path()).expect("a repository that is there");
    let instance: String = instance_of(git_dir.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("note");
    machine.socket_of(instance.as_str());
    let journal: PathBuf = a_journal_with_a_plan(&machine, instance.as_str(), git_dir.as_path(), repository.as_path(), "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let kept_plan: crate::push_retry_store::StoredPlan = crate::push_retry_store::StoredPlan {
        git_dir: alive_git_dir.clone(),
        item: crate::work_item::WorkItem {
            kind: crate::work_item::WorkKind::Push,
            repository: alive_repository.clone(),
            git_dir: alive_git_dir.clone(),
            branch_ref: String::from("refs/heads/main"),
            oid: String::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            parent: String::new(),
            created: String::from("2026-11-11T08:55:00.000-05:00"),
            tries: 0,
        },
        attempt: 1,
    };
    crate::push_retry_store::save(journal.join(crate::push_retry::PLAN_DIRECTORY).as_path(), &kept_plan).expect("the plan of a repository that is there");
    let mut recorder: Recorder = Recorder::default();
    let gone: Option<PathBuf> = stop_own_socket_of_a_gone_worktree(&machine.folders(), instance.as_str(), &mut recorder).expect("stopped");
    assert_eq!(gone, Some(git_dir));
    assert!(journal.exists(), "the plan of a repository that is there keeps the journal");
    let plans: Vec<crate::push_retry_store::StoredPlan> = crate::push_retry_store::load(journal.join(crate::push_retry::PLAN_DIRECTORY).as_path());
    assert_eq!(plans.len(), 1, "only the plan of the gone repository ended: {plans:?}");
    assert_eq!(plans[0].git_dir, alive_git_dir, "the plan that stays is the one of the repository that is there");
    remove(machine.root.as_path());
}

/// What a removal Git runs with the wrapper replaced leaves behind (`git worktree remove --force` through a foreign Git):
/// the note and the socket of that worktree,
/// and the sweep stops them once the administrative directory is gone.
#[test]
fn a_foreign_git_removal_leaves_units_that_the_sweep_stops() {
    let machine: Machine = machine("units-foreign-removal");
    let main: PathBuf = repository(machine.root.as_path(), "repository");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../worktree"]);
    let admin: PathBuf = std::fs::canonicalize(main.join(".git/worktrees/worktree")).expect("canonical administrative directory");
    let instance: String = instance_of(admin.as_path());
    record_worktree(machine.runtime.as_path(), instance.as_str(), admin.as_path()).expect("note written by provisioning");
    machine.socket_of(instance.as_str());
    // Real Git, with no wrapper in front of it: the removal's own stop of the units never runs.
    git(main.as_path(), &["worktree", "remove", "--force", "../worktree"]);
    assert!(!admin.exists(), "the removal takes the administrative directory");
    let mut recorder: Recorder = Recorder::default();
    let outcome: SweepOutcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert_eq!(outcome.stopped, vec![instance.clone()]);
    assert_eq!(recorder.stops(), vec![format!("stop {}", socket_unit_instance(instance.as_str())), format!("stop {}", service_unit_instance(instance.as_str()))]);
    assert_eq!(survey(&machine.folders()), Survey::default(), "nothing is left for the doctor to count");
    remove(machine.root.as_path());
}

/// A refused stop is reported with its instance and keeps the note,
/// so that the next sweep tries again.
#[test]
fn a_refused_stop_is_reported_and_its_note_kept_for_the_next_sweep() {
    let machine: Machine = machine("units-sweep-refused");
    let gone: PathBuf = machine.linked("gone", false);
    let gone_instance: String = instance_of(gone.as_path());
    record_worktree(machine.runtime.as_path(), gone_instance.as_str(), gone.as_path()).expect("note");
    machine.socket_of(gone_instance.as_str());
    let mut recorder: Recorder = Recorder::default();
    recorder.refuse_stop_of(socket_unit_instance(gone_instance.as_str()).as_str(), CallError::TimedOut);
    let outcome: SweepOutcome = stop_units_of_gone_worktrees(&mut recorder, &machine.folders());
    assert!(outcome.stopped.is_empty());
    assert_eq!(outcome.failed.len(), 1);
    assert_eq!(outcome.failed[0].0, gone_instance);
    assert!(note_path(machine.runtime.as_path(), gone_instance.as_str()).exists());
    remove(machine.root.as_path());
}
