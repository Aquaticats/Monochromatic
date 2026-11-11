//! Controls for `service stop-units-of-removed-worktrees` and the note `doctor` adds about it:
//! what each answer says,
//! and that a worktree that is there,
//! or an instance that maps to nothing,
//! is never stopped.

use super::*;
use crate::service_instance::{SOCKET_DIRECTORY, instance_of, service_unit_instance, socket_path, socket_unit_instance};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::service_units_of_removed_worktrees::record_worktree;
use crate::systemd_dbus::CallError;
use crate::test_support::{fixture, remove};

/// The command word the notes name.
const NAMED_COMMAND: &str = "git monochromatic-git-policy-cli service stop-units-of-removed-worktrees";

/// One disposable world:
/// a runtime directory with a socket and a note for a present worktree,
/// a removed worktree,
/// and an instance that maps to nothing.
struct World {
    root: PathBuf,
    runtime: PathBuf,
    gone_instance: String,
    kept_instance: String,
    unknown_instance: String,
}

fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    let common: PathBuf = root.join("repository/.git");
    let kept: PathBuf = common.join("worktrees/kept");
    std::fs::create_dir_all(&kept).expect("kept");
    let gone: PathBuf = common.join("worktrees/gone");
    let unknown_instance: String = String::from("00000000000000000000000000000001");
    let world: World = World { gone_instance: instance_of(gone.as_path()), kept_instance: instance_of(kept.as_path()), unknown_instance, root, runtime };
    for (instance, git_dir) in [(&world.kept_instance, &kept), (&world.gone_instance, &gone)] {
        record_worktree(world.runtime.as_path(), instance.as_str(), git_dir.as_path()).expect("note");
        std::fs::write(socket_path(world.runtime.as_path(), instance.as_str()), b"listening").expect("socket");
    }
    std::fs::write(socket_path(world.runtime.as_path(), world.unknown_instance.as_str()), b"listening").expect("socket");
    return world;
}

impl World {
    fn folders(&self) -> UnitFolders<'_> {
        return UnitFolders { runtime_dir: self.runtime.as_path(), state_dir: None };
    }

    fn environment(&self) -> Vec<(OsString, OsString)> {
        return vec![(OsString::from("XDG_RUNTIME_DIR"), self.runtime.clone().into_os_string())];
    }
}

fn exit_parts(action: Action) -> (i32, String, String) {
    let Action::Exit { code, stdout, stderr } = action else { panic!("the command exits") };
    return (code, stdout, stderr);
}

/// The command stops the removed worktree's two units and names the worktree,
/// and says that it left the unknown instance running.
#[test]
fn the_command_stops_the_units_of_the_removed_worktree_and_names_it() {
    let world: World = world("stop-units-command");
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(manager.stops(), vec![format!("stop {}", socket_unit_instance(world.gone_instance.as_str())), format!("stop {}", service_unit_instance(world.gone_instance.as_str()))]);
    assert!(stdout.contains("repository/.git/worktrees/gone") && stdout.contains(world.gone_instance.as_str()), "{stdout}");
    assert!(stderr.contains("1 commit service instance cannot be matched") && stderr.contains(NAMED_COMMAND), "{stderr}");
    assert!(!stdout.contains(world.kept_instance.as_str()));
    remove(world.root.as_path());
}

/// What:
///  A journal that holds a commit no branch shows is kept and named on standard error,
///  with where it is and what kept it,
///  and the command still ends with status 0 because the units it was asked about were stopped.
/// Why:
///   The person needs that line even when warnings are turned off,
/// and it must not read as a refusal:
/// nothing here failed.
#[test]
fn a_kept_journal_is_named_on_standard_error_and_is_no_refusal() {
    let world: World = world("stop-units-kept-journal");
    let state: PathBuf = world.root.join("state");
    let wrapper_state: PathBuf = state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
    let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(wrapper_state.as_path(), world.gone_instance.as_str());
    std::fs::create_dir_all(&journal).expect("journal folder");
    std::fs::write(journal.join("7.intent"), b"seq=7\n").expect("a batch that did not land");
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: world.runtime.as_path(), state_dir: Some(state.as_path()) };
    let mut manager: Recorder = Recorder::default();
    let (code, _stdout, stderr) = exit_parts(stop_units_action(&folders, &mut manager));
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.contains(journal.display().to_string().as_str()), "the line names the journal: {stderr}");
    assert!(stderr.contains("7.intent") && stderr.contains("has not landed"), "the line names what kept it: {stderr}");
    assert!(journal.exists(), "the journal is kept");
    remove(world.root.as_path());
}

/// A manager that refuses a stop makes status 1 with the instance and the reason,
/// and the command can be run again.
#[test]
fn a_refused_stop_is_status_one_with_the_instance_and_the_reason() {
    let world: World = world("stop-units-refused");
    let mut manager: Recorder = Recorder::default();
    manager.refuse_stop_of(socket_unit_instance(world.gone_instance.as_str()).as_str(), CallError::TimedOut);
    let (code, _stdout, stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 1);
    assert!(stderr.contains(world.gone_instance.as_str()) && stderr.contains("did not answer in time") && stderr.contains("run the command again"), "{stderr}");
    remove(world.root.as_path());
}

/// With no gone worktree the command says so and asks the manager nothing.
#[test]
fn nothing_to_stop_is_said_and_asks_the_manager_nothing() {
    let world: World = world("stop-units-nothing");
    std::fs::remove_file(socket_path(world.runtime.as_path(), world.gone_instance.as_str())).expect("socket removed");
    std::fs::remove_file(crate::service_units_of_removed_worktrees::note_path(world.runtime.as_path(), world.gone_instance.as_str())).expect("note removed");
    let mut manager: Recorder = Recorder::default();
    let (code, stdout, _stderr) = exit_parts(stop_units_action(&world.folders(), &mut manager));
    assert_eq!(code, 0);
    assert!(stdout.contains("no commit service unit belongs to a removed worktree"), "{stdout}");
    assert!(manager.stops().is_empty());
    remove(world.root.as_path());
}

/// `doctor` counts the units of removed worktrees without changing a file,
/// and says nothing when none is left.
#[test]
fn doctor_counts_the_units_of_removed_worktrees_and_names_the_command() {
    let world: World = world("stop-units-note");
    let note: String = units_note(world.environment().as_slice());
    assert!(note.contains("1 removed worktree are still set up") && note.contains(NAMED_COMMAND) && note.contains("nothing was changed"), "{note}");
    assert!(socket_path(world.runtime.as_path(), world.gone_instance.as_str()).exists(), "the note is read only");
    std::fs::remove_file(socket_path(world.runtime.as_path(), world.gone_instance.as_str())).expect("socket removed");
    std::fs::remove_file(crate::service_units_of_removed_worktrees::note_path(world.runtime.as_path(), world.gone_instance.as_str())).expect("note removed");
    std::fs::remove_file(socket_path(world.runtime.as_path(), world.unknown_instance.as_str())).expect("socket removed");
    assert_eq!(units_note(world.environment().as_slice()), "");
    remove(world.root.as_path());
}

/// Without a user runtime directory the note is empty and the command refuses with the reason.
#[test]
fn without_a_runtime_directory_the_note_is_empty_and_the_command_refuses() {
    assert_eq!(units_note(&[]), "");
    let (code, _stdout, stderr) = exit_parts(plan_stop_units_of_removed_worktrees(&[]));
    assert_eq!(code, 1);
    assert!(stderr.contains("XDG_RUNTIME_DIR is not set") && stderr.contains(NAMED_COMMAND), "{stderr}");
}
