//! Controls for the stop of a removed worktree's units:
//! the instance is read from the registration before the removal,
//! the units are stopped only once the Git directory is gone,
//! and a refused removal leaves them alone.

use super::*;
use crate::service_instance::{SOCKET_DIRECTORY, instance_of, service_unit_instance, socket_unit_instance};
use crate::service_unit_manager_recorder_test_support::Recorder;
use crate::test_support::{fixture, git, remove, repository};

/// A repository with one linked worktree whose instance has a socket,
/// and the runtime directory that holds it.
struct Fixture {
    root: PathBuf,
    runtime: PathBuf,
    common: PathBuf,
    worktree: PathBuf,
}

fn fixture_with_units(name: &str) -> Fixture {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "topic", "../wt"]);
    let runtime: PathBuf = root.join("run");
    std::fs::create_dir_all(runtime.join(SOCKET_DIRECTORY)).expect("socket directory");
    let worktree: PathBuf = std::fs::canonicalize(root.join("wt")).expect("canonical worktree");
    let common: PathBuf = std::fs::canonicalize(main.join(".git")).expect("canonical common directory");
    return Fixture { root, runtime, common, worktree };
}

impl Fixture {
    fn plan(&self) -> RemovePlan {
        return RemovePlan { target: self.worktree.clone(), operand: self.worktree.clone().into_os_string(), force: false };
    }

    fn folders(&self) -> UnitFolders<'_> {
        return UnitFolders { runtime_dir: self.runtime.as_path(), state_dir: None };
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
