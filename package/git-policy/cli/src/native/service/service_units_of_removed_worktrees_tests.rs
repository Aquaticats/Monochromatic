//! Controls for the commit service units of worktrees that no longer exist:
//! the note,
//! the test of a gone worktree,
//! the survey and the stop.
//! The manager is a recorder.

use super::*;
use crate::service_instance::{instance_of, socket_path};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::service_units_fixture_test_support::{Machine, machine};
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

#[test]
fn a_worktree_in_a_repository_that_cannot_be_reached_is_not_gone() {
    let machine: Machine = machine("units-gone-unreachable");
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

/// A service that starts for a worktree that is gone stops its own socket and serves nothing:
/// the socket goes first so no connection starts it again,
/// and the service unit is never asked to stop,
/// because the service's own end is what stops it.
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
    assert!(!note_path(machine.runtime.as_path(), instance.as_str()).exists(), "the note goes with the socket, so the doctor counts no leftover");
    assert!(!socket_path(machine.runtime.as_path(), instance.as_str()).exists(), "the stopped socket's file is removed");
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
