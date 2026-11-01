//! Controls for the wrapper's own provisioning: what is written, when the manager is asked to reload, what is removed or
//!  refused, and that racing first commits agree through idempotence alone. The manager is a recorder.

use super::*;
use crate::service_messages::render;
use crate::test_support::{fixture, remove};
use std::sync::{Arc, Mutex};

/// A manager that records what it is asked and can be made to fail.
#[derive(Clone, Default)]
struct Recorder {
    calls: Arc<Mutex<Vec<String>>>,
    failure: Arc<Mutex<Option<CallError>>>,
}

impl Manager for Recorder {
    fn reload(&mut self) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(String::from("reload"));
        return self.failure.lock().expect("lock").clone().map_or(Ok(()), Err);
    }

    fn reset_failed(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("reset-failed {unit}"));
        return self.failure.lock().expect("lock").clone().map_or(Ok(()), Err);
    }

    fn start(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("start {unit}"));
        return self.failure.lock().expect("lock").clone().map_or(Ok(()), Err);
    }
}

impl Recorder {
    fn calls(&self) -> Vec<String> {
        return self.calls.lock().expect("lock").clone();
    }
}

struct Machine {
    root: PathBuf,
    host: ProvisionHost,
}

fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: root.join("run"),
        user_unit_dir: Some(root.join("config/systemd/user")),
        bus: root.join("run/bus"),
        service_binary: root.join("programs/cli-git-service"),
        git: PathBuf::from("/usr/bin/git"),
    };
    return Machine { root, host };
}

fn written(machine: &Machine, name: &str) -> String {
    return std::fs::read_to_string(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit");
}

/// The first provisioning writes both units, filled in and marked, and only starts the socket: the manager finds a new file
///  by itself, and a reload would interrupt its other clients.
#[test]
fn the_first_provisioning_writes_the_units_and_starts_without_a_reload() {
    let machine: Machine = machine("provision-first-lib");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    for (name, text) in unit_texts(&machine.host) {
        assert_eq!(written(&machine, name), text);
        assert!(!text.contains('@'), "no placeholder is left in {name}");
        assert!(OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }), "{name} carries the marker");
    }
    assert!(written(&machine, SERVICE_UNIT_NAME).contains(machine.host.service_binary.to_string_lossy().as_ref()));
    assert_eq!(recorder.calls(), vec![format!("start {SOCKET_UNIT_NAME}")]);
    remove(machine.root.as_path());
}

/// A second provisioning changes no file and does not reload; it clears what a failed start may have left and asks again for
///  the start, which is a no-op for a socket that already listens.
#[test]
fn a_second_provisioning_changes_nothing_and_clears_failed_state_before_the_start() {
    let machine: Machine = machine("provision-second-lib");
    let mut first: Recorder = Recorder::default();
    provision_with(&machine.host, &mut first).expect("provisioned");
    let stamps: Vec<std::time::SystemTime> = [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]
        .map(|name| { return std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit").modified().expect("time"); })
        .to_vec();
    let mut second: Recorder = Recorder::default();
    provision_with(&machine.host, &mut second).expect("provisioned again");
    assert_eq!(
        second.calls(),
        vec![format!("reset-failed {SERVICE_UNIT_NAME}"), format!("reset-failed {SOCKET_UNIT_NAME}"), format!("start {SOCKET_UNIT_NAME}")]
    );
    let after: Vec<std::time::SystemTime> = [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]
        .map(|name| { return std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit").modified().expect("time"); })
        .to_vec();
    assert_eq!(stamps, after);
    remove(machine.root.as_path());
}

/// The socket the units activate, as the manager would have made it.
fn add_socket(machine: &Machine) {
    let socket: PathBuf = machine.host.runtime_dir.join(SOCKET_RELATIVE);
    std::fs::create_dir_all(socket.parent().expect("socket directory")).expect("directories");
    std::fs::write(&socket, b"").expect("socket stand-in");
}

/// The installed units count as in place only when both match what this wrapper would write and the socket is there:
///  two small reads and one `stat`, no bus.
#[test]
fn units_are_in_place_only_when_they_match_and_the_socket_exists() {
    let mut machine: Machine = machine("provision-in-place-lib");
    assert!(!in_place(&machine.host), "nothing installed");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    assert!(!in_place(&machine.host), "units without the socket");
    add_socket(&machine);
    assert!(in_place(&machine.host), "units that match and the socket");
    machine.host.git = PathBuf::from("/opt/other/git");
    assert!(!in_place(&machine.host), "a different real Git is a different service unit");
    machine.host.git = PathBuf::from("/usr/bin/git");
    assert!(in_place(&machine.host));
    remove(machine.root.as_path());
}

/// A provisioned unit that names a program an upgrade removed is not in place, and provisioning repairs it: the unit is
///  rewritten with the program that exists, the manager reloaded, failed state cleared, and the socket started.
#[test]
fn a_unit_naming_a_removed_program_is_repaired_with_a_reload_and_a_reset() {
    let machine: Machine = machine("provision-removed-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    let unit: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME);
    let removed: &str = "/home/a/.local/share/toolchain/cargo/cli/0.5.2/bin/cli-git-service";
    std::fs::write(&unit, written(&machine, SERVICE_UNIT_NAME).replace(machine.host.service_binary.to_string_lossy().as_ref(), removed)).expect("older unit");
    assert!(!in_place(&machine.host));
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("repaired");
    assert!(!written(&machine, SERVICE_UNIT_NAME).contains(removed));
    assert!(in_place(&machine.host));
    assert_eq!(
        recorder.calls(),
        vec![
            String::from("reload"),
            format!("reset-failed {SERVICE_UNIT_NAME}"),
            format!("reset-failed {SOCKET_UNIT_NAME}"),
            format!("start {SOCKET_UNIT_NAME}")
        ]
    );
    remove(machine.root.as_path());
}

/// A unit another release wrote, whose text differs from this release's template and not only in the program path, is
///  rewritten as well.
#[test]
fn a_unit_from_another_release_is_rewritten() {
    let machine: Machine = machine("provision-release-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    let dir: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    let other: String = written(&machine, SERVICE_UNIT_NAME).replace("Restart=on-failure", "Restart=always").replace("MemoryMax=1G", "MemoryMax=2G");
    std::fs::write(dir.join(SERVICE_UNIT_NAME), &other).expect("other release's unit");
    assert!(!in_place(&machine.host));
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("rewritten");
    assert_eq!(written(&machine, SERVICE_UNIT_NAME), unit_texts(&machine.host)[1].1);
    assert_eq!(recorder.calls().first().map(String::as_str), Some("reload"));
    remove(machine.root.as_path());
}

/// When the manager refuses after a unit was rewritten, the rewrite is withdrawn, so the next commit does not take a file the
///  manager never reloaded for an installed one.
#[test]
fn a_refused_reload_leaves_no_unit_that_looks_installed() {
    let machine: Machine = machine("provision-withdrawn-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    let unit: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME);
    std::fs::write(&unit, written(&machine, SERVICE_UNIT_NAME).replace("MemoryMax=1G", "MemoryMax=2G")).expect("older unit");
    let mut refusing: Recorder = Recorder::default();
    *refusing.failure.lock().expect("lock") = Some(CallError::Manager { name: String::from("org.freedesktop.systemd1.AccessDenied"), text: String::new() });
    provision_with(&machine.host, &mut refusing).expect_err("refused");
    assert!(!in_place(&machine.host), "the next commit provisions again");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned again");
    assert!(in_place(&machine.host));
    remove(machine.root.as_path());
}

/// A moved service program (an upgrade) rewrites the service unit only, and reloads.
#[test]
fn a_moved_program_rewrites_the_service_unit_and_reloads() {
    let mut machine: Machine = machine("provision-moved-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    let socket_before: String = written(&machine, SOCKET_UNIT_NAME);
    machine.host.service_binary = machine.root.join("upgraded/cli-git-service");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(written(&machine, SERVICE_UNIT_NAME).contains(machine.host.service_binary.to_string_lossy().as_ref()));
    assert_eq!(written(&machine, SOCKET_UNIT_NAME), socket_before);
    assert_eq!(recorder.calls().first().map(String::as_str), Some("reload"));
    remove(machine.root.as_path());
}

/// Units of an earlier release are recognized by their content and removed with their enabling link, then a reload follows.
#[test]
fn earlier_units_are_removed_with_their_link() {
    let machine: Machine = machine("provision-earlier-lib");
    let directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    std::fs::create_dir_all(directory.join("sockets.target.wants")).expect("directories");
    std::fs::write(directory.join(SOCKET_UNIT_NAME), "[Unit]\nDescription=Commit service socket for the git wrapper\n").expect("earlier socket");
    std::fs::write(directory.join(SERVICE_UNIT_NAME), "[Unit]\nDescription=Commit service for the git wrapper\n").expect("earlier service");
    std::os::unix::fs::symlink("../cli-git.socket", directory.join("sockets.target.wants").join(SOCKET_UNIT_NAME)).expect("link");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(!directory.join(SOCKET_UNIT_NAME).exists() && !directory.join(SERVICE_UNIT_NAME).exists());
    assert!(std::fs::symlink_metadata(directory.join("sockets.target.wants").join(SOCKET_UNIT_NAME)).is_err());
    assert_eq!(recorder.calls().first().map(String::as_str), Some("reload"));
    remove(machine.root.as_path());
}

/// A unit of the same name that something else wrote is never touched, nothing else is written, the manager is not asked,
///  and the failure names the file.
#[test]
fn a_foreign_unit_is_left_alone_and_named() {
    let machine: Machine = machine("provision-foreign-lib");
    let directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    std::fs::create_dir_all(&directory).expect("directory");
    let foreign: PathBuf = directory.join(SERVICE_UNIT_NAME);
    std::fs::write(&foreign, "[Service]\nExecStart=/usr/bin/true\n").expect("foreign unit");
    let mut recorder: Recorder = Recorder::default();
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("refused");
    assert_eq!(failure, ClientFailure::UnitInTheWay(foreign.display().to_string()));
    assert!(render(&failure).contains(foreign.to_string_lossy().as_ref()));
    assert_eq!(std::fs::read_to_string(&foreign).expect("still there"), "[Service]\nExecStart=/usr/bin/true\n");
    assert!(!runtime_unit_dir(machine.host.runtime_dir.as_path()).exists(), "nothing was written");
    assert!(recorder.calls().is_empty());
    remove(machine.root.as_path());
}

/// The manager's refusal reaches the person in plain words without the manager's internal error name, and a broken
///  connection is the no-manager failure.
#[test]
fn manager_failures_are_plain() {
    let machine: Machine = machine("provision-failures-lib");
    let mut recorder: Recorder = Recorder::default();
    *recorder.failure.lock().expect("lock") =
        Some(CallError::Manager { name: String::from("org.freedesktop.systemd1.AccessDenied"), text: String::from("permission denied") });
    let refused: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("refused");
    let text: String = render(&refused);
    assert!(text.contains("permission denied") && !text.contains("org.freedesktop"), "{text}");
    *recorder.failure.lock().expect("lock") = Some(CallError::Connection(String::from("broken pipe")));
    assert_eq!(provision_with(&machine.host, &mut recorder), Err(ClientFailure::NoActivation(platform_without_manager())));
    remove(machine.root.as_path());
}

/// Many first commits provisioning at once agree through idempotence alone: every one succeeds and the files are whole.
#[test]
fn racing_first_commits_agree_without_a_lock() {
    let machine: Machine = machine("provision-race-lib");
    let shared: Arc<ProvisionHost> = Arc::new(machine.host.clone());
    let recorder: Recorder = Recorder::default();
    let handles: Vec<std::thread::JoinHandle<Result<(), ClientFailure>>> = (0..8)
        .map(|_| {
            let for_thread: Arc<ProvisionHost> = Arc::clone(&shared);
            let mut manager: Recorder = recorder.clone();
            return std::thread::spawn(move || { return provision_with(&for_thread, &mut manager); });
        })
        .collect();
    for handle in handles {
        assert_eq!(handle.join().expect("thread"), Ok(()));
    }
    for (name, text) in unit_texts(&machine.host) {
        assert_eq!(written(&machine, name), text);
    }
    let leftovers: Vec<String> = std::fs::read_dir(runtime_unit_dir(machine.host.runtime_dir.as_path()))
        .expect("directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.contains(".tmp-"); })
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    remove(machine.root.as_path());
}

/// The wrapper's own program name and the unit names agree with what the units say.
#[test]
fn the_unit_names_agree_with_the_templates() {
    assert!(SERVICE_UNIT.contains(&format!("Requires={SOCKET_UNIT_NAME}")));
    assert!(SERVICE_UNIT.contains(BINARY_PLACEHOLDER) && SERVICE_UNIT.contains(GIT_PLACEHOLDER));
}
