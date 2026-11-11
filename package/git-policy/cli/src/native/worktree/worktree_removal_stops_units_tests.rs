//! Controls for the stop of a removed worktree's units:
//! the instance is read from the registration before the removal,
//! the units are stopped only once the Git directory is gone,
//! and a refused removal leaves them alone.

use super::*;
use crate::service_instance::{SOCKET_DIRECTORY, instance_of, service_unit_instance, socket_unit_instance};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::test_support::{fixture, git, remove, repository};

/// A repository with one linked worktree whose instance has a socket,
/// and the runtime and state directories that hold it.
struct Fixture {
    root: PathBuf,
    runtime: PathBuf,
    state: PathBuf,
    common: PathBuf,
    worktree: PathBuf,
}

fn fixture_with_units(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    let state: PathBuf = root.join("state");
    std::fs::create_dir_all(&state).expect("state directory");
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("canonical common directory");
    return Fixture { root, runtime, state, common, worktree };
}

impl Fixture {
    fn plan(&self) -> RemovePlan {
        return RemovePlan { target: self.worktree.clone(), operand: self.worktree.clone().into_os_string(), force: false };
    }

    fn folders(&self) -> UnitFolders<'_> {
        return UnitFolders { runtime_dir: self.runtime.as_path(), state_dir: Some(self.state.as_path()) };
    }

    /// The service journal of an instance,
    /// as the unit's `--journal` names it,
    /// holding the records a service that committed leaves.
    fn journal_of(&self, instance: &str) -> PathBuf {
        let wrapper_state: PathBuf = self.state.join(crate::wrapper_names_old_and_new::SERVICE_UNIT.current);
        let journal: PathBuf = crate::service_journal_folder_of_an_instance::journal_of(wrapper_state.as_path(), instance);
        std::fs::create_dir_all(journal.join("busy")).expect("journal folder");
        std::fs::write(journal.join("1.done"), b"seq=1\n").expect("a finished batch");
        return journal;
    }
}

/// The instance comes from the registration the removal deletes,
/// the same text `instance_of` gives that Git directory.
#[test]
fn the_instance_is_read_from_the_registration_before_the_removal() {
    let fixture: Fixture = fixture_with_units("removal-units-before");
    let about: WorktreeAboutToGo = before_removal(fixture.common.as_path(), &fixture.plan()).expect("registered");
    assert_eq!(about.instance, instance_of(about.git_dir.as_path()));
    assert!(about.git_dir.starts_with(fixture.common.join("worktrees")));
    remove(fixture.root.as_path());
}

/// A directory that is no registration of the repository names no instance.
#[test]
fn a_directory_that_is_no_registration_names_no_instance() {
    let fixture: Fixture = fixture_with_units("removal-units-unregistered");
    let stranger: PathBuf = fixture.root.join("main");
    let plan: RemovePlan = RemovePlan { target: stranger.clone(), operand: stranger.into_os_string(), force: false };
    assert_eq!(before_removal(fixture.common.as_path(), &plan), None);
    remove(fixture.root.as_path());
}

/// After a removal that took the Git directory,
/// the socket and the service of that instance are stopped.
#[test]
fn a_removed_worktrees_units_are_stopped_after_the_removal() {
    let fixture: Fixture = fixture_with_units("removal-units-removed");
    let about: WorktreeAboutToGo = before_removal(fixture.common.as_path(), &fixture.plan()).expect("registered");
    std::fs::write(socket_path(fixture.runtime.as_path(), about.instance.as_str()), b"listening").expect("socket file");
    git(fixture.common.as_path(), &["worktree", "remove", "--force", fixture.worktree.to_str().expect("text")]);
    let mut recorder: Recorder = Recorder::default();
    let stopped: bool = stop_units_after_removal(&about, &fixture.folders(), &mut recorder).expect("stopped");
    assert!(stopped);
    assert_eq!(recorder.stops(), vec![format!("stop {}", socket_unit_instance(about.instance.as_str())), format!("stop {}", service_unit_instance(about.instance.as_str()))]);
    assert!(!socket_path(fixture.runtime.as_path(), about.instance.as_str()).exists(), "the stopped socket's file is removed");
    remove(fixture.root.as_path());
}

/// What:
///  The removal the wrapper runs takes the journal of the removed worktree with its units,
///  once the journal holds nothing but the records of the batches that landed.
/// Why:
///   A removal through the wrapper is the everyday one,
/// and its state directory stayed behind just as a foreign removal's did;
/// nothing serves an instance whose worktree is gone again.
#[test]
fn a_removed_worktrees_journal_goes_with_its_units() {
    let fixture: Fixture = fixture_with_units("removal-units-journal");
    let about: WorktreeAboutToGo = before_removal(fixture.common.as_path(), &fixture.plan()).expect("registered");
    std::fs::write(socket_path(fixture.runtime.as_path(), about.instance.as_str()), b"listening").expect("socket file");
    let journal: PathBuf = fixture.journal_of(about.instance.as_str());
    git(fixture.common.as_path(), &["worktree", "remove", "--force", fixture.worktree.to_str().expect("text")]);
    let mut recorder: Recorder = Recorder::default();
    assert!(stop_units_after_removal(&about, &fixture.folders(), &mut recorder).expect("stopped"));
    assert!(!journal.exists(), "the journal of the removed worktree is removed with its units");
    remove(fixture.root.as_path());
}

/// A removal that was refused leaves the Git directory,
/// and the units of the worktree stay.
#[test]
fn a_refused_removal_leaves_the_units_alone() {
    let fixture: Fixture = fixture_with_units("removal-units-refused");
    let about: WorktreeAboutToGo = before_removal(fixture.common.as_path(), &fixture.plan()).expect("registered");
    std::fs::write(socket_path(fixture.runtime.as_path(), about.instance.as_str()), b"listening").expect("socket file");
    let mut recorder: Recorder = Recorder::default();
    let stopped: bool = stop_units_after_removal(&about, &fixture.folders(), &mut recorder).expect("nothing to stop");
    assert!(!stopped);
    assert!(recorder.stops().is_empty(), "{:?}", recorder.stops());
    remove(fixture.root.as_path());
}

/// A removal the name guard leaves to Git still deletes the registration the plan names,
/// so that worktree's units are stopped once Git has removed it.
/// Why:
///  Reading the registration only through the name guard lost the instance whenever a second registration ended in the
///  name,
///  and the socket unit,
///  which has no idle exit,
///  kept listening after Git's removal.
#[test]
fn a_removal_left_to_git_still_stops_the_units_of_the_removed_worktree() {
    let root: PathBuf = fixture("removal-units-left-to-git");
    let main: PathBuf = repository(root.as_path(), "main");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../topic"]);
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "TOPIC", "../TOPIC"]);
    // The second worktree moves and a link is left at its old name, so two registrations name the topic directory.
    std::fs::rename(root.join("TOPIC"), root.join("upper-kept")).expect("move the second worktree");
    std::os::unix::fs::symlink(root.join("topic"), root.join("TOPIC")).expect("leave a link at the old name");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("common");
    let topic: PathBuf = std::fs::canonicalize(root.join("topic")).expect("canonical");
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    let state: PathBuf = root.join("state");
    std::fs::create_dir_all(&state).expect("state directory");
    let plan: RemovePlan = RemovePlan { target: topic.clone(), operand: topic.clone().into_os_string(), force: false };
    // Git's suffix match finds two registrations for this name, so the wrapper leaves the removal to Git.
    assert!(crate::worktree_remove_registry::registration_of(common.as_path(), topic.as_path(), plan.operand.as_os_str()).is_err());
    let about: WorktreeAboutToGo = before_removal(common.as_path(), &plan).expect("the registration names the units");
    let socket: PathBuf = socket_path(runtime.as_path(), about.instance.as_str());
    std::fs::write(socket.as_path(), b"listening").expect("socket file");
    git(main.as_path(), &["worktree", "remove", topic.to_str().expect("text")]);
    let mut recorder: Recorder = Recorder::default();
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime.as_path(), state_dir: Some(state.as_path()) };
    assert!(stop_units_after_removal(&about, &folders, &mut recorder).expect("stopped"));
    assert_eq!(recorder.stops(), vec![format!("stop {}", socket_unit_instance(about.instance.as_str())), format!("stop {}", service_unit_instance(about.instance.as_str()))]);
    assert!(!socket.exists(), "the stopped socket's file is removed");
    remove(root.as_path());
}

/// A worktree that never had a unit costs the removal no question to the manager.
#[test]
fn a_worktree_that_never_had_units_costs_the_manager_nothing() {
    let fixture: Fixture = fixture_with_units("removal-units-none");
    let about: WorktreeAboutToGo = before_removal(fixture.common.as_path(), &fixture.plan()).expect("registered");
    git(fixture.common.as_path(), &["worktree", "remove", "--force", fixture.worktree.to_str().expect("text")]);
    let mut recorder: Recorder = Recorder::default();
    assert!(!stop_units_after_removal(&about, &fixture.folders(), &mut recorder).expect("nothing to stop"));
    assert!(recorder.stops().is_empty());
    remove(fixture.root.as_path());
}
