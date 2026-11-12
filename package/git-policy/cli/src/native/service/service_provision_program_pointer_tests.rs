//! Controls for the link the units start the commit service's program through:
//! that two releases and two builds of one release write the very same unit text,
//! that a release change moves only the link and asks the manager nothing,
//! that an installation whose link names another program is not one this commit can use,
//! that a link an upgrade left naming a removed program is pointed again,
//! and that a replacement killed between its link and its rename leaves the old link.
//! The manager is a recorder,
//! so none of these reach the account's own manager.

use super::*;
use crate::service_provision::{SERVICE_UNIT_NAME, SOCKET_UNIT_NAME, in_place, provision_with, unit_texts};
use crate::service_provision_host_and_manager::runtime_unit_dir;
use crate::service_provision::tests::{Machine, Recorder, add_socket, machine};
use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  A program this machine's wrapper does not run,
/// standing where a release an upgrade replaced kept its executable.
/// Why:
///   The two states the link is asked about are "this executable" and "some other one",
/// and a file beside the fixture is what makes
///  the second one a program that exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const earlier = writeProgram(root, 'earlier-release/program');
/// ```
fn another_program(machine: &Machine, under: &str) -> PathBuf {
    let program: PathBuf = machine.root.join(under).join("monochromatic-git-policy-cli");
    std::fs::create_dir_all(program.parent().expect("a folder")).expect("the program's folder");
    std::fs::write(&program, b"a program of another release").expect("the program");
    return program;
}

/// What:
///  A link pointed at a program and read back,
/// and one pointed at another program,
/// under a runtime directory of the test's own.
/// Why:
///   The two readings are the one question provisioning asks before it touches anything,
/// so a control that points and reads it is the whole
///  contract of the link.
#[test]
fn a_link_names_the_program_it_was_pointed_at_and_another_afterwards() {
    let root: PathBuf = fixture("program-pointer-points");
    let first: PathBuf = root.join("programs/first");
    let second: PathBuf = root.join("programs/second");
    std::fs::create_dir_all(root.join("programs")).expect("fixture folders");
    std::fs::write(&first, b"first").expect("first program");
    std::fs::write(&second, b"second").expect("second program");
    assert!(!points_at(root.as_path(), first.as_path()), "a folder with no link names no program");
    point_at(root.as_path(), first.as_path()).expect("the link is written");
    assert!(points_at(root.as_path(), first.as_path()), "the link names the program it was pointed at");
    assert!(!points_at(root.as_path(), second.as_path()), "the link names one program, not two");
    point_at(root.as_path(), second.as_path()).expect("the link is moved");
    assert!(points_at(root.as_path(), second.as_path()), "the link names the program it was moved to");
    assert_eq!(std::fs::read_link(program_pointer_path(root.as_path())).expect("the link"), second, "the link names the program itself");
    remove(root.as_path());
}

/// What:
///  Two executables that differ in the release they were built with,
/// and two builds of one release,
/// write the same unit text,
/// and that text names the link rather than an executable.
/// Why:
///   The manager reads a unit file again only when it is asked to reload every unit,
/// and that reload took 1 to 13
///  seconds on this host:
/// a text that carried the executable of the release that wrote it made every release change and every
///  second build pay for one.
#[test]
fn two_releases_write_one_unit_text_naming_the_link() {
    let mut machine: Machine = machine("program-pointer-one-text");
    let installed: [(&'static str, String); 2] = unit_texts(&machine.host);
    let program: PathBuf = program_of(machine.host.runtime_dir.as_path(), machine.host.service_binary.as_path());
    assert!(installed.iter().any(|(name, text)| { return *name == SERVICE_UNIT_NAME && text.contains(program.to_string_lossy().as_ref()); }), "the service unit starts the program through the link");
    assert!(!installed.iter().any(|(_name, text)| text.contains(machine.host.service_binary.to_string_lossy().as_ref())), "no unit names an executable of its own");
    // The next release, and a second build of this one, are other executables under another folder.
    for under in ["next-release/program", "second-build-of-this-release/program"] {
        let program: PathBuf = another_program(&machine, under);
        machine.host.service_binary = program;
        assert_eq!(unit_texts(&machine.host), installed, "the unit text of {under} is the one the earlier release wrote");
    }
    // The real Git a release runs is not a release's own program, so a unit that names a different one is a different unit.
    machine.host.service_binary = machine.root.join("programs/monochromatic-git-policy-cli");
    machine.host.git = PathBuf::from("/opt/other/git");
    assert_ne!(unit_texts(&machine.host), installed, "another real Git is another service unit");
    remove(machine.root.as_path());
}

/// What:
///  A commit that finds the link naming another executable points it and asks the manager nothing.
/// Why:
///   This is the whole cost a first commit after a release is installed pays:
/// one read and one rename,
/// never a reload of every
///  unit, and never a round trip that queues behind another worktree's reload.
#[test]
fn a_release_change_moves_the_link_and_asks_the_manager_nothing() {
    let machine: Machine = machine("program-pointer-release-change");
    std::fs::create_dir_all(runtime_unit_dir(machine.host.runtime_dir.as_path())).expect("the unit folder");
    provision_with(&machine.host, &mut Recorder::default()).expect("a first provision");
    add_socket(&machine);
    assert!(in_place(&machine.host), "the release that provisioned is installed");
    let earlier: PathBuf = another_program(&machine, "earlier-release/program");
    point_at(machine.host.runtime_dir.as_path(), earlier.as_path()).expect("the link names the earlier release");
    let installed: String = std::fs::read_to_string(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME)).expect("the installed template");
    let written_at: std::time::SystemTime = std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME)).expect("the template's times").modified().expect("a modification time");
    assert!(!in_place(&machine.host), "the link names another release, so the installation is not this command's");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("the provisioning of a changed release");
    assert!(recorder.calls().is_empty(), "a release change asks the manager nothing, and it asked {:?}", recorder.calls());
    assert!(points_at(machine.host.runtime_dir.as_path(), machine.host.service_binary.as_path()), "the link names the program of this release");
    assert_eq!(std::fs::read_to_string(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME)).expect("the template"), installed, "the template is the one the earlier release wrote");
    assert_eq!(std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME)).expect("the template's times").modified().expect("a modification time"), written_at, "the template was not written again");
    assert!(in_place(&machine.host), "and the installation is this command's again");
    remove(machine.root.as_path());
}

/// What:
///  An installation whose link names another program is not in place,
/// even though both templates and the record of what the manager was told match.
/// Why:
///   The manager starts the program the link names when it replaces a service,
/// and a service of another release greets this commit only to be asked to
///  retire,
/// so the fast path would hand every commit of a freshly installed release to the release it replaced.
#[test]
fn an_installation_whose_link_names_another_program_is_not_in_place() {
    let machine: Machine = machine("program-pointer-not-in-place");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    assert!(in_place(&machine.host), "the program this command runs is the one the link names");
    let other: PathBuf = another_program(&machine, "another-release/program");
    point_at(machine.host.runtime_dir.as_path(), other.as_path()).expect("the link names another release");
    assert!(!in_place(&machine.host), "another program is not this command's");
    remove(machine.root.as_path());
}

/// What:
///  A link that names a program an upgrade removed is pointed at this release's program,
/// without the manager.
/// Why:
///   A start from such a link fails with a message about a path,
/// and the commit would wait out its start ceiling for a
///  service no start can bring.
#[test]
fn a_link_naming_a_removed_program_is_pointed_at_this_program() {
    let machine: Machine = machine("program-pointer-removed");
    provision_with(&machine.host, &mut Recorder::default()).expect("a first provision");
    add_socket(&machine);
    let removed: PathBuf = machine.root.join("upgrade-removed/program");
    point_at(machine.host.runtime_dir.as_path(), removed.as_path()).expect("the link names a program an upgrade removed");
    assert!(!removed.exists(), "the program the link names is gone");
    assert!(!in_place(&machine.host), "provisioning is needed");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("the provisioning of a commit that finds the link stale");
    assert!(points_at(machine.host.runtime_dir.as_path(), machine.host.service_binary.as_path()), "the link names a program that is there");
    assert!(recorder.calls().is_empty(), "and the manager was told nothing, and it was asked {:?}", recorder.calls());
    remove(machine.root.as_path());
}

/// What:
///  A replacement killed between its new link and its rename leaves the link that was there,
/// which still names a program that exists.
/// Why:
///   The two steps are the temporary link and the rename;
/// a reader that finds the old link is a provisioning that runs again and
///  costs one rename,
/// where a link removed before the new one was made would name nothing and no start could bring the
///  service back.
#[test]
fn a_replacement_that_stops_before_its_rename_leaves_the_old_link() {
    let root: PathBuf = fixture("program-pointer-half-written");
    let first: PathBuf = root.join("programs/first");
    let second: PathBuf = root.join("programs/second");
    std::fs::create_dir_all(root.join("programs")).expect("fixture folders");
    std::fs::write(&first, b"first").expect("first program");
    std::fs::write(&second, b"second").expect("second program");
    point_at(root.as_path(), first.as_path()).expect("the link is written");
    // The temporary link a killed replacement leaves behind, and the old link beside it.
    let temporary: PathBuf = program_pointer_path(root.as_path()).with_file_name(format!("{PROGRAM_POINTER}.tmp-{}-0", std::process::id()));
    std::os::unix::fs::symlink(&second, &temporary).expect("the temporary link");
    assert!(points_at(root.as_path(), first.as_path()), "a reader sees the link that was there, whole");
    assert!(!points_at(root.as_path(), second.as_path()), "and not the one the killed replacement was making");
    // The next provisioning renames its own temporary link over the old one, and touches neither program.
    point_at(root.as_path(), second.as_path()).expect("the link is moved");
    assert!(points_at(root.as_path(), second.as_path()), "the link names the program the next provisioning wrote");
    assert!(first.exists() && second.exists(), "neither program is touched");
    let _removed = std::fs::remove_file(&temporary);
    remove(root.as_path());
}

/// What:
///  Both templates name the link and no executable,
/// and the socket unit does not name it at all.
/// Why:
///   Only the service unit starts a program,
/// and the socket's text must not change when a release moves,
/// so a change of it can never be
///  the reason for a reload.
#[test]
fn only_the_service_template_names_the_program_and_it_names_it_through_the_link() {
    let mut machine: Machine = machine("program-pointer-templates");
    let program: PathBuf = program_of(machine.host.runtime_dir.as_path(), machine.host.service_binary.as_path());
    let texts: [(&'static str, String); 2] = unit_texts(&machine.host);
    let socket: &String = texts.iter().find_map(|(name, text)| { return (*name == SOCKET_UNIT_NAME).then_some(text); }).expect("the socket unit");
    assert!(!socket.contains(program.to_string_lossy().as_ref()), "the socket unit names no program");
    let service: &String = texts.iter().find_map(|(name, text)| { return (*name == SERVICE_UNIT_NAME).then_some(text); }).expect("the service unit");
    assert!(service.lines().any(|line| { return line.starts_with("ExecStart=") && line.contains(program.to_string_lossy().as_ref()); }), "the service unit starts the link");
    machine.host.service_binary = another_program(&machine, "another/program");
    assert_eq!(texts, unit_texts(&machine.host), "no text moves with the program");
    remove(machine.root.as_path());
}
