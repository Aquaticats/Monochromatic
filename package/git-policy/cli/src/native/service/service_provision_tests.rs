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
    /// What a stop is answered with, when it is not the general failure.
    stop_failure: Arc<Mutex<Option<CallError>>>,
    /// The states a unit reports in turn, the last one repeating; a unit without a script is not held by the manager.
    states: Arc<Mutex<std::collections::HashMap<String, Vec<String>>>>,
    /// What a state question is answered with instead of a state, when it is made to fail.
    state_failure: Arc<Mutex<Option<CallError>>>,
    /// The times the calls that followed were given, in order, kept apart from the calls.
    budgets: Arc<Mutex<Vec<std::time::Duration>>>,
}

impl Manager for Recorder {
    fn within(&mut self, wait: std::time::Duration) {
        self.budgets.lock().expect("lock").push(wait);
    }

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

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("stop {unit}"));
        if let Some(refusal) = self.stop_failure.lock().expect("lock").clone() {
            return Err(refusal);
        }
        return self.failure.lock().expect("lock").clone().map_or(Ok(()), Err);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        self.calls.lock().expect("lock").push(format!("state {unit}"));
        if let Some(failure) = self.state_failure.lock().expect("lock").clone() {
            return Err(failure);
        }
        let mut scripts = self.states.lock().expect("lock");
        let Some(script) = scripts.get_mut(unit) else { return Ok(None) };
        return Ok(Some(if script.len() > 1 { script.remove(0) } else { script[0].clone() }));
    }
}

/// The instance the fixture machine's worktree has.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// The units of the instance, as the manager is asked about them.
fn instance_socket() -> String {
    return socket_unit_instance(INSTANCE);
}

fn instance_service() -> String {
    return service_unit_instance(INSTANCE);
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
        instance: String::from(INSTANCE),
        retirement_wait: std::time::Duration::from_millis(400),
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
        assert!(!text.contains(BINARY_PLACEHOLDER) && !text.contains(GIT_PLACEHOLDER), "no placeholder is left in {name}");
        assert!(OWN_MARKERS.iter().any(|marker| { return text.contains(marker); }), "{name} carries the marker");
    }
    assert!(written(&machine, SERVICE_UNIT_NAME).contains(machine.host.service_binary.to_string_lossy().as_ref()));
    assert_eq!(recorder.calls(), vec![format!("start {}", instance_socket())]);
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
        vec![format!("reset-failed {}", instance_service()), format!("reset-failed {}", instance_socket()), format!("start {}", instance_socket())]
    );
    let after: Vec<std::time::SystemTime> = [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]
        .map(|name| { return std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit").modified().expect("time"); })
        .to_vec();
    assert_eq!(stamps, after);
    remove(machine.root.as_path());
}

/// The socket the units activate, as the manager would have made it.
fn add_socket(machine: &Machine) {
    let socket: PathBuf = socket_path(machine.host.runtime_dir.as_path(), machine.host.instance.as_str());
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

/// An earlier unit of the wrapper's own, in either unit directory, keeps the installation from counting as in place; a file of
///  that name that something else wrote is not the wrapper's to retire and does not.
#[test]
fn only_the_wrappers_own_earlier_units_keep_an_installation_from_being_in_place() {
    let machine: Machine = machine("provision-in-place-earlier-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    assert!(in_place(&machine.host));
    let runtime_directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::write(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME), FOREIGN_UNIT).expect("someone's unit");
    assert!(in_place(&machine.host), "a unit someone else wrote is not an earlier service of the wrapper's");
    std::fs::write(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME), OWN_UNIT).expect("an earlier unit of the wrapper's");
    assert!(!in_place(&machine.host), "an earlier unit of the wrapper's in the runtime directory");
    std::fs::remove_file(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME)).expect("removed");
    assert!(in_place(&machine.host));
    let user_directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    std::fs::create_dir_all(&user_directory).expect("user directory");
    std::fs::write(user_directory.join(LEGACY_SOCKET_UNIT_NAME), OWN_UNIT).expect("an earlier unit of the wrapper's");
    assert!(!in_place(&machine.host), "an earlier unit of the wrapper's in the user's own directory");
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
            format!("reset-failed {}", instance_service()),
            format!("reset-failed {}", instance_socket()),
            format!("start {}", instance_socket())
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
    let other: String = written(&machine, SERVICE_UNIT_NAME).replace("Restart=on-failure", "Restart=always");
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
    // What:     `format!("{}...", text)` builds a new owned `String`: the written unit followed by one more line.
    // Why:      an older release's unit differs from this release's template by a line this release never writes, so
    //           the difference does not depend on any one setting staying in the template.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(unit, `${written(machine, SERVICE_UNIT_NAME)}# a line an older release wrote\n`);
    // ```
    std::fs::write(&unit, format!("{}# a line an older release wrote\n", written(&machine, SERVICE_UNIT_NAME))).expect("older unit");
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
    std::fs::write(directory.join(LEGACY_SOCKET_UNIT_NAME), "[Unit]\nDescription=Commit service socket for the git wrapper\n").expect("earlier socket");
    std::fs::write(directory.join(LEGACY_SERVICE_UNIT_NAME), "[Unit]\nDescription=Commit service for the git wrapper\n").expect("earlier service");
    std::os::unix::fs::symlink("../cli-git.socket", directory.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME)).expect("link");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(!directory.join(LEGACY_SOCKET_UNIT_NAME).exists() && !directory.join(LEGACY_SERVICE_UNIT_NAME).exists());
    assert!(std::fs::symlink_metadata(directory.join("sockets.target.wants").join(LEGACY_SOCKET_UNIT_NAME)).is_err());
    // The service those units ran is stopped before the files go, then the manager reloads.
    let calls: Vec<String> = recorder.calls();
    assert!(position(&calls, format!("stop {LEGACY_SERVICE_UNIT_NAME}").as_str()) < position(&calls, "reload"), "{calls:?}");
    remove(machine.root.as_path());
}

/// A unit of the earlier single service's name that something else wrote is never touched, nothing else is written, the manager is not asked,
///  and the failure names the file.
#[test]
fn a_foreign_unit_is_left_alone_and_named() {
    let machine: Machine = machine("provision-foreign-lib");
    let directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    std::fs::create_dir_all(&directory).expect("directory");
    let foreign: PathBuf = directory.join(LEGACY_SERVICE_UNIT_NAME);
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
    assert!(SERVICE_UNIT.contains(&format!("Requires={}", SOCKET_UNIT_NAME.replace('@', "@%i"))));
    assert!(SERVICE_UNIT.contains(BINARY_PLACEHOLDER) && SERVICE_UNIT.contains(GIT_PLACEHOLDER));
}

/// The single service of the earlier releases, which served every worktree, is retired by the first provisioning of the release
///  that gives each worktree its own: its socket and service are stopped (socket first), its unit files and socket file are
///  removed, the manager is reloaded, and the worktree's own instance is started. Nothing of it counts as in place.
#[test]
fn the_earlier_single_service_is_stopped_and_its_units_removed() {
    let machine: Machine = machine("provision-retire-earlier-lib");
    let directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&directory).expect("unit directory");
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        std::fs::write(directory.join(name), "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n").expect("earlier unit");
    }
    let earlier_socket: PathBuf = machine.host.runtime_dir.join(LEGACY_SOCKET_RELATIVE);
    std::fs::create_dir_all(earlier_socket.parent().expect("socket directory")).expect("directories");
    std::fs::write(&earlier_socket, b"").expect("the earlier socket's file");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert_eq!(
        recorder.calls(),
        vec![
            format!("stop {LEGACY_SOCKET_UNIT_NAME}"),
            format!("stop {LEGACY_SERVICE_UNIT_NAME}"),
            format!("state {LEGACY_SOCKET_UNIT_NAME}"),
            format!("state {LEGACY_SERVICE_UNIT_NAME}"),
            String::from("reload"),
            format!("start {}", instance_socket())
        ]
    );
    assert!(!directory.join(LEGACY_SOCKET_UNIT_NAME).exists() && !directory.join(LEGACY_SERVICE_UNIT_NAME).exists());
    assert!(!earlier_socket.exists(), "the earlier socket's file is gone");
    add_socket(&machine);
    assert!(in_place(&machine.host));
    remove(machine.root.as_path());
}

/// The wrapper's units of the earlier single service in `directory`, and the socket file it listened on.
fn earlier_service_files(machine: &Machine, directory: &Path) -> PathBuf {
    std::fs::create_dir_all(directory).expect("unit directory");
    for name in [LEGACY_SOCKET_UNIT_NAME, LEGACY_SERVICE_UNIT_NAME] {
        std::fs::write(directory.join(name), "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n").expect("earlier unit");
    }
    let earlier_socket: PathBuf = machine.host.runtime_dir.join(LEGACY_SOCKET_RELATIVE);
    std::fs::create_dir_all(earlier_socket.parent().expect("socket directory")).expect("directories");
    std::fs::write(&earlier_socket, b"").expect("the earlier socket's file");
    return earlier_socket;
}

fn script(recorder: &Recorder, unit: &str, states: &[&str]) {
    recorder.states.lock().expect("lock").insert(unit.to_string(), states.iter().map(|state| { return (*state).to_string(); }).collect());
}

fn position(calls: &[String], wanted: &str) -> usize {
    return calls.iter().position(|call| { return call == wanted; }).unwrap_or_else(|| { panic!("{wanted} was not asked: {calls:?}") });
}

/// A stop job that is only queued has not stopped the service: the new instance starts after the manager reports the earlier
///  service and its socket inactive, and not before.
#[test]
fn an_instance_starts_only_after_the_earlier_service_has_stopped() {
    let machine: Machine = machine("provision-retire-wait-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["active", "deactivating", "inactive"]);
    script(&recorder, LEGACY_SOCKET_UNIT_NAME, &["inactive"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    let service_states: Vec<usize> =
        calls.iter().enumerate().filter(|(_, call)| { return call.as_str() == format!("state {LEGACY_SERVICE_UNIT_NAME}"); }).map(|(index, _)| { return index; }).collect();
    assert_eq!(service_states.len(), 3, "asked until it was inactive: {calls:?}");
    assert!(position(&calls, format!("start {}", instance_socket()).as_str()) > service_states[2], "{calls:?}");
    remove(machine.root.as_path());
}

/// A service that is still stopping when the wait ends fails the commit with nothing started and nothing removed, so the
///  next commit finds the earlier service again.
#[test]
fn an_earlier_service_that_does_not_stop_in_time_fails_the_commit() {
    let machine: Machine = machine("provision-retire-late-lib");
    let directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    let earlier_socket: PathBuf = earlier_service_files(&machine, directory.as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["deactivating"]);
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("the stop did not finish");
    assert!(matches!(&failure, ClientFailure::CouldNotStart(reason) if reason.contains(LEGACY_SERVICE_UNIT_NAME)), "{failure:?}");
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("start ") || call == "reload"; }), "{:?}", recorder.calls());
    assert!(directory.join(LEGACY_SERVICE_UNIT_NAME).exists() && earlier_socket.exists(), "the evidence stays for the next commit");
    remove(machine.root.as_path());
}

/// A question to the manager that runs out of the retirement budget is a service still stopping, told apart from a manager
///  that is not there: the commit says what is stopping and how to stop it, and does not name a missing service manager.
#[test]
fn a_state_question_that_outlasts_the_retirement_wait_is_a_slow_stop_and_not_a_missing_manager() {
    let machine: Machine = machine("provision-retire-question-late-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    *recorder.state_failure.lock().expect("lock") = Some(CallError::TimedOut);
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("the question ran out of time");
    assert!(matches!(&failure, ClientFailure::CouldNotStart(reason) if reason.contains(LEGACY_SERVICE_UNIT_NAME) || reason.contains(LEGACY_SOCKET_UNIT_NAME)), "{failure:?}");
    assert_ne!(failure, ClientFailure::NoActivation(platform_without_manager()));
    remove(machine.root.as_path());
}

/// A timeout outside the retirement wait is still the manager not answering.
#[test]
fn a_call_that_times_out_outside_the_retirement_wait_is_a_manager_that_does_not_answer() {
    let machine: Machine = machine("provision-timeout-lib");
    let mut recorder: Recorder = Recorder::default();
    *recorder.failure.lock().expect("lock") = Some(CallError::TimedOut);
    assert_eq!(provision_with(&machine.host, &mut recorder), Err(ClientFailure::NoActivation(platform_without_manager())));
    remove(machine.root.as_path());
}

/// Each phase has a time of its own: the questions of the retirement wait never get more than the wait allows, and the start
///  that follows gets the whole exchange's time again however long the retirement took.
#[test]
fn the_retirement_questions_and_the_start_have_budgets_of_their_own() {
    let machine: Machine = machine("provision-budgets-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["deactivating", "inactive"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let budgets: Vec<std::time::Duration> = recorder.budgets.lock().expect("lock").clone();
    assert_eq!(budgets.first(), Some(&EXCHANGE_WAIT), "the stop calls get a whole exchange's time: {budgets:?}");
    assert_eq!(budgets.last(), Some(&EXCHANGE_WAIT), "the start gets a whole exchange's time again: {budgets:?}");
    let questions: &[std::time::Duration] = &budgets[1..budgets.len() - 1];
    assert!(!questions.is_empty(), "{budgets:?}");
    assert!(questions.iter().all(|budget| { return *budget <= machine.host.retirement_wait; }), "{budgets:?}");
    remove(machine.root.as_path());
}

/// A stop the manager refuses is a failure of the commit, not a retirement.
#[test]
fn a_refused_stop_of_the_earlier_service_fails_the_commit() {
    let machine: Machine = machine("provision-retire-refused-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    *recorder.stop_failure.lock().expect("lock") =
        Some(CallError::Manager { name: String::from("org.freedesktop.systemd1.AccessDenied"), text: String::from("permission denied") });
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("the stop was refused");
    assert!(matches!(&failure, ClientFailure::CouldNotStart(reason) if reason.contains("permission denied")), "{failure:?}");
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("start "); }), "{:?}", recorder.calls());
    remove(machine.root.as_path());
}

/// The earlier release that wrote its units into the user's own directory ran a service that the retirement must stop too,
///  before the files go.
#[test]
fn an_earlier_service_installed_in_the_persistent_unit_directory_is_stopped() {
    let machine: Machine = machine("provision-retire-persistent-lib");
    let directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    earlier_service_files(&machine, directory.as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["active", "inactive"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    assert!(position(&calls, format!("stop {LEGACY_SOCKET_UNIT_NAME}").as_str()) < position(&calls, format!("stop {LEGACY_SERVICE_UNIT_NAME}").as_str()));
    assert!(position(&calls, format!("stop {LEGACY_SERVICE_UNIT_NAME}").as_str()) < position(&calls, "reload"), "{calls:?}");
    assert!(!directory.join(LEGACY_SERVICE_UNIT_NAME).exists());
    remove(machine.root.as_path());
}

/// The earlier service's socket file with no unit file left (a release removed the files and left the service running) still
///  sends the stop, and a unit the manager does not hold counts as stopped.
#[test]
fn an_earlier_socket_file_without_unit_files_still_retires_the_service() {
    let machine: Machine = machine("provision-retire-socket-only-lib");
    let earlier_socket: PathBuf = machine.host.runtime_dir.join(LEGACY_SOCKET_RELATIVE);
    std::fs::create_dir_all(earlier_socket.parent().expect("socket directory")).expect("directories");
    std::fs::write(&earlier_socket, b"").expect("the earlier socket's file");
    let mut recorder: Recorder = Recorder::default();
    *recorder.stop_failure.lock().expect("lock") =
        Some(CallError::Manager { name: String::from("org.freedesktop.systemd1.NoSuchUnit"), text: String::from("not loaded") });
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(recorder.calls().contains(&format!("stop {LEGACY_SERVICE_UNIT_NAME}")), "{:?}", recorder.calls());
    assert!(!earlier_socket.exists(), "the earlier socket's file is gone");
    remove(machine.root.as_path());
}

/// The text of a unit the wrapper wrote, and of one something else wrote.
const OWN_UNIT: &str = "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n";
const FOREIGN_UNIT: &str = "[Service]\nExecStart=/usr/bin/true\n";

/// An owned unit in the user's own directory is the unit that runs, so it is stopped and removed; an unrelated file of the
///  same name in the runtime directory is shadowed by it and is neither stopped for nor removed.
#[test]
fn an_owned_persistent_unit_does_not_authorize_removing_a_foreign_runtime_file() {
    let machine: Machine = machine("provision-retire-shadow-lib");
    let user_directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    let runtime_directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&user_directory).expect("user directory");
    std::fs::create_dir_all(&runtime_directory).expect("runtime directory");
    std::fs::write(user_directory.join(LEGACY_SERVICE_UNIT_NAME), OWN_UNIT).expect("owned unit");
    std::fs::write(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME), FOREIGN_UNIT).expect("foreign unit");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(recorder.calls().contains(&format!("stop {LEGACY_SERVICE_UNIT_NAME}")), "the unit that runs is stopped: {:?}", recorder.calls());
    assert_eq!(std::fs::read_to_string(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME)).expect("still there"), FOREIGN_UNIT);
    assert!(!user_directory.join(LEGACY_SERVICE_UNIT_NAME).exists(), "the owned file is removed");
    remove(machine.root.as_path());
}

/// An owned file in the runtime directory does not make the unit ours when the user's own directory defines the unit with
///  a file that is someone else's: the manager is not asked to stop it, and the commit names the file in the way.
#[test]
fn an_owned_runtime_file_does_not_authorize_stopping_a_foreign_persistent_unit() {
    let machine: Machine = machine("provision-retire-foreign-effective-lib");
    let user_directory: PathBuf = machine.host.user_unit_dir.clone().expect("unit directory");
    let runtime_directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&user_directory).expect("user directory");
    std::fs::create_dir_all(&runtime_directory).expect("runtime directory");
    let foreign: PathBuf = user_directory.join(LEGACY_SERVICE_UNIT_NAME);
    std::fs::write(&foreign, FOREIGN_UNIT).expect("foreign unit");
    std::fs::write(runtime_directory.join(LEGACY_SERVICE_UNIT_NAME), OWN_UNIT).expect("owned unit");
    let mut recorder: Recorder = Recorder::default();
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("refused");
    assert_eq!(failure, ClientFailure::UnitInTheWay(foreign.display().to_string()));
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("stop "); }), "{:?}", recorder.calls());
    assert_eq!(std::fs::read_to_string(&foreign).expect("still there"), FOREIGN_UNIT);
    remove(machine.root.as_path());
}

/// A unit file that is gone after the ownership check is a removal someone else finished, and any other refusal to remove
///  one fails the commit with the file named.
#[test]
fn a_unit_file_that_vanished_after_the_ownership_check_is_a_finished_removal() {
    let machine: Machine = machine("provision-vanished-lib");
    std::fs::create_dir_all(&machine.root).expect("root");
    assert_eq!(remove_settled(machine.root.join("gone.service").as_path()), Ok(()));
    let blocked: PathBuf = machine.root.join("directory.service");
    std::fs::create_dir_all(&blocked).expect("a directory where a file was expected");
    assert!(matches!(remove_settled(blocked.as_path()), Err(ClientFailure::CouldNotStart(reason)) if reason.contains(blocked.to_string_lossy().as_ref())));
    remove(machine.root.as_path());
}

/// Units of the earlier name that something else wrote are not the wrapper's to stop or remove.
#[test]
fn an_earlier_unit_the_wrapper_did_not_write_is_not_stopped() {
    let machine: Machine = machine("provision-retire-foreign-lib");
    let directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&directory).expect("unit directory");
    std::fs::write(directory.join(LEGACY_SERVICE_UNIT_NAME), "[Service]\nExecStart=/usr/bin/true\n").expect("someone's unit");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("stop "); }), "{:?}", recorder.calls());
    assert!(directory.join(LEGACY_SERVICE_UNIT_NAME).exists());
    remove(machine.root.as_path());
}
