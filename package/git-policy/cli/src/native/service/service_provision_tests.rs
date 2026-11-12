//! Controls for the wrapper's own provisioning:
//! what is written,
//! when the manager is asked to reload,
//! what is removed or
//!  refused,
//! and that racing first commits agree through idempotence alone.
//! The manager is a recorder.

use super::*;
use crate::service_provision_unit_file_ownership_and_writes::remove_settled;
use crate::service_messages::render;
use crate::test_support::{fixture, remove};
use std::sync::{Arc, Mutex};

/// A manager that records what it is asked and can be made to fail.
#[derive(Clone, Default)]
pub(crate) struct Recorder {
    calls: Arc<Mutex<Vec<String>>>,
    failure: Arc<Mutex<Option<CallError>>>,
    /// What a stop is answered with,
    /// when it is not the general failure.
    stop_failure: Arc<Mutex<Option<CallError>>>,
    /// What a stop that spares the units requiring the stopped one is answered with,
    /// when it fails.
    stop_alone_failure: Arc<Mutex<Option<CallError>>>,
    /// A socket file that the earlier wrapper's unit makes again after every answer about its socket and after every stop,
    /// as an
    ///  earlier-release wrapper that sets its unit up again would.
    recreate: Arc<Mutex<Option<PathBuf>>>,
    /// The states a unit reports in turn,
    /// the last one repeating;
    /// a unit without a script is not held by the manager.
    states: Arc<Mutex<std::collections::HashMap<String, Vec<String>>>>,
    /// What a state question is answered with instead of a state,
    /// when it is made to fail.
    state_failure: Arc<Mutex<Option<CallError>>>,
    /// The times the calls that followed were given,
    /// in order,
    /// kept apart from the calls.
    budgets: Arc<Mutex<Vec<std::time::Duration>>>,
    /// What a control reads at the moment the manager is asked to start a unit.
    at_start: Arc<Mutex<Option<Box<dyn FnMut() + Send>>>>,
    /// The addresses the manager answers when it is asked where a unit listens,
    /// keyed by unit;
    /// a unit without an entry is not held by this manager.
    addresses: Arc<Mutex<std::collections::HashMap<String, Vec<String>>>>,
    /// What the question about a unit's addresses is answered with instead,
    /// when it is made to fail.
    listen_failure: Arc<Mutex<Option<CallError>>>,
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
        if let Some(read) = self.at_start.lock().expect("lock").as_mut() {
            read();
        }
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

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.calls.lock().expect("lock").push(format!("stop-alone {unit}"));
        self.recreate_socket();
        return self.stop_alone_failure.lock().expect("lock").clone().map_or(Ok(()), Err);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        self.calls.lock().expect("lock").push(format!("state {unit}"));
        if unit == earlier_socket_unit_instance(INSTANCE) {
            self.recreate_socket();
        }
        if let Some(failure) = self.state_failure.lock().expect("lock").clone() {
            return Err(failure);
        }
        let mut scripts = self.states.lock().expect("lock");
        let Some(script) = scripts.get_mut(unit) else { return Ok(None) };
        return Ok(Some(if script.len() > 1 { script.remove(0) } else { script[0].clone() }));
    }

    fn listen(&mut self, unit: &str) -> Result<Option<Vec<String>>, CallError> {
        if let Some(failure) = self.listen_failure.lock().expect("lock").clone() {
            return Err(failure);
        }
        // The addresses a control set, keyed by unit: a unit without one is not held by this manager.
        return Ok(self.addresses.lock().expect("lock").get(unit).cloned());
    }
}

/// What provisioning asks last of a manager that holds no unit of the earlier release for the worktree:
/// whether its service
///  runs,
/// then whether its socket listens.
fn with_earlier_instance_questions(mut calls: Vec<String>) -> Vec<String> {
    calls.push(format!("state {}", earlier_service_unit_instance(INSTANCE)));
    calls.push(format!("state {}", earlier_socket_unit_instance(INSTANCE)));
    return calls;
}

/// The instance the fixture machine's worktree has.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// The units of the instance,
/// as the manager is asked about them.
fn instance_socket() -> String {
    return socket_unit_instance(INSTANCE);
}

fn instance_service() -> String {
    return service_unit_instance(INSTANCE);
}

impl Recorder {
    /// Makes the socket file again when a control asked for that.
    fn recreate_socket(&self) {
        if let Some(file) = self.recreate.lock().expect("lock").as_ref() {
            std::fs::create_dir_all(file.parent().expect("socket directory")).expect("directories");
            std::fs::write(file, b"live").expect("the socket made again");
        }
    }

    pub(crate) fn calls(&self) -> Vec<String> {
        return self.calls.lock().expect("lock").clone();
    }

    /// What a control reads each time the manager is asked to start a unit.
    pub(super) fn observe_start(&self, read: Box<dyn FnMut() + Send>) {
        *self.at_start.lock().expect("lock") = Some(read);
    }
}

pub(crate) struct Machine {
    pub(crate) root: PathBuf,
    pub(crate) host: ProvisionHost,
}

pub(crate) fn machine(name: &str) -> Machine {
    let root: PathBuf = fixture(name);
    let runtime: PathBuf = short_runtime_directory();
    let host: ProvisionHost = ProvisionHost {
        bus: runtime.join("bus"),
        runtime_dir: runtime,
        user_unit_dir: Some(root.join("config/systemd/user")),
        service_binary: root.join("programs/monochromatic-git-policy-cli"),
        git: PathBuf::from("/usr/bin/git"),
        instance: String::from(INSTANCE),
        git_dir: root.join("repository/.git/worktrees/topic"),
        retirement_wait: std::time::Duration::from_millis(400),
    };
    return Machine { root, host };
}

/// How many runtime directories this process has made.
static RUNTIME_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// What:
///  A runtime directory short enough that a socket name fits inside one socket address.
/// Why:
///   The socket stands at `<runtime directory>/<folder>/<instance>.sock`,
///   which takes 40 bytes of the runtime directory's room at most,
///   and the fixture root under this checkout's build directory is far longer than that.
/// It stands in the agent scratch directory,
/// as a fixture of the built executable's controls does,
/// and this test thread removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const runtime = join(home, 'temp/agent', `pr-${process.pid}-${n}`); // removed with the thread's fixtures
/// ```
fn short_runtime_directory() -> PathBuf {
    let scratch: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    let number: usize = RUNTIME_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let runtime: PathBuf = scratch.join(format!("pr-{}-{number}", std::process::id()));
    std::fs::create_dir_all(&runtime).expect("the runtime directory");
    crate::test_support::track_for_this_thread(runtime.as_path());
    return runtime;
}

fn written(machine: &Machine, name: &str) -> String {
    return std::fs::read_to_string(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit");
}

/// A unit file that an earlier release wrote and one this release writes are both the wrapper's own,
///  and a file of someone else is not;
///  the units this release writes name only the new program,
/// folder and identifier.
#[test]
fn unit_files_of_both_spellings_are_the_wrappers_own_and_the_new_ones_name_the_new_program_only() {
    use crate::service_provision_unit_file_ownership_and_writes::carries_own_marker;
    assert!(carries_own_marker("# Written by the git wrapper for its commit service (cli-git-unit); the wrapper rewrites this file.\n"));
    assert!(carries_own_marker("# Written by the git wrapper for its commit service (monochromatic-git-policy-cli-unit); the wrapper rewrites this file.\n"));
    assert!(!carries_own_marker("# A unit someone else wrote for their own service.\n"));
    for text in [SOCKET_UNIT, SERVICE_UNIT] {
        assert!(!text.contains("cli-git"), "a unit this release writes names the new program only: {text}");
    }
}

/// The first provisioning writes both units,
/// filled in and marked,
/// and only starts the socket:
/// the manager finds a new file
///  by itself,
/// and a reload would interrupt its other clients.
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
    let program: PathBuf = crate::service_provision_program_pointer::program_of(machine.host.runtime_dir.as_path(), machine.host.service_binary.as_path());
    assert!(written(&machine, SERVICE_UNIT_NAME).contains(program.to_string_lossy().as_ref()), "the service unit starts the program through the link");
    assert!(!written(&machine, SERVICE_UNIT_NAME).contains(machine.host.service_binary.to_string_lossy().as_ref()), "and names no executable of its own");
    assert_eq!(recorder.calls(), with_earlier_instance_questions(vec![format!("start {}", instance_socket())]));
    remove(machine.root.as_path());
}

/// The service unit makes the manager signal only the service on a stop (`KillMode=mixed`),
/// so the service ends its own Gits and lets a branch write finish,
/// and it gives the stop a timeout of its own that is above the service's last-resort bound,
/// so the host's default (15 s here) never decides.
#[test]
fn the_service_unit_signals_only_the_service_and_waits_longer_than_the_last_resort() {
    let machine: Machine = machine("unit-stop-timeout-lib");
    let text: String = unit_texts(&machine.host).into_iter().find(|(name, _text)| { return *name == SERVICE_UNIT_NAME; }).expect("the service unit").1;
    assert!(text.lines().any(|line| { return line == "KillMode=mixed"; }), "{text}");
    let written: u64 = text
        .lines()
        .find_map(|line| { return line.strip_prefix("TimeoutStopSec="); })
        .expect("the unit sets its own stop timeout")
        .parse()
        .expect("whole seconds");
    assert_eq!(written, STOP_TIMEOUT_SECONDS);
    assert!(std::time::Duration::from_secs(written) > crate::stop_of_the_service_noticed_by_its_children::LAST_RESORT, "{written} s is not above the last resort");
    assert!(!text.contains("@STOP_TIMEOUT_SECONDS@"), "no placeholder is left: {text}");
    remove(machine.root.as_path());
}

/// Provisioning writes the worktree's Git directory beside the socket,
/// so the units can be told apart from a removed worktree's later,
/// and a second provisioning leaves the same note.
#[test]
fn provisioning_notes_which_worktree_the_units_are_for() {
    let machine: Machine = machine("provision-note-lib");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let note: PathBuf = crate::service_units_of_removed_worktrees::note_path(machine.host.runtime_dir.as_path(), INSTANCE);
    assert_eq!(noted(note.as_path()), machine.host.git_dir, "the note names the worktree of the units");
    let written: Vec<u8> = std::fs::read(&note).expect("the note");
    let mut again: Recorder = Recorder::default();
    provision_with(&machine.host, &mut again).expect("provisioned again");
    assert_eq!(std::fs::read(&note).expect("the note"), written, "the second provisioning writes no other note");
    assert_eq!(noted(note.as_path()), machine.host.git_dir);
    remove(machine.root.as_path());
}

/// The Git directory a note names,
/// read the way the sweep reads a note of this release:
/// the bytes it holds,
/// not its text,
/// which would name another place or none for a path that is not text.
fn noted(note: &Path) -> PathBuf {
    use crate::path_bytes_in_records::{GITDIR_HEX_PREFIX, path_of_hex};
    let text: String = std::fs::read_to_string(note).expect("the note");
    let hexadecimal: &str = text.strip_prefix(GITDIR_HEX_PREFIX).expect("the note names the bytes of the directory");
    return path_of_hex(hexadecimal.trim_end_matches('\n')).expect("the note holds the bytes of a path");
}

/// What:
///  The note that names the worktree is on disk before the manager is asked to start the units.
/// Why:
///  The service learns whose units it runs from that note,
/// and a service that finds none answers `Unmapped`
///  and never asks whether its worktree is still there,
/// so the sweep that stops the units of a removed worktree can never name it
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 4).
/// A kill between the
///  start of the units and the note left exactly that service behind,
/// for up to the idle exit of three hours.
/// The note names a directory
///  that exists and needs no manager,
/// so nothing is owed to writing it after the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(noteExistsAtTheMomentOfStart).toBe(true);
/// ```
#[test]
fn the_worktrees_note_is_written_before_the_manager_is_asked_to_start_the_units() {
    let machine: Machine = machine("provision-note-first-lib");
    let note: PathBuf = crate::service_units_of_removed_worktrees::note_path(machine.host.runtime_dir.as_path(), INSTANCE);
    let seen: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));
    let mut recorder: Recorder = Recorder::default();
    let watched: Arc<Mutex<Vec<bool>>> = Arc::clone(&seen);
    let named: PathBuf = note.clone();
    recorder.observe_start(Box::new(move || { watched.lock().expect("lock").push(named.exists()); }));
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert_eq!(seen.lock().expect("lock").as_slice(), [true], "the note names the worktree before the units are started");
    assert_eq!(noted(note.as_path()), machine.host.git_dir, "the note names the worktree of the units");
    remove(machine.root.as_path());
}

/// A second provisioning changes no file and does not reload;
/// it clears what a failed start may have left and asks again for
///  the start,
/// which is a no-op for a socket that already listens.
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
        with_earlier_instance_questions(vec![format!("reset-failed {}", instance_service()), format!("reset-failed {}", instance_socket()), format!("start {}", instance_socket())])
    );
    let after: Vec<std::time::SystemTime> = [SOCKET_UNIT_NAME, SERVICE_UNIT_NAME]
        .map(|name| { return std::fs::metadata(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name)).expect("unit").modified().expect("time"); })
        .to_vec();
    assert_eq!(stamps, after);
    remove(machine.root.as_path());
}

/// The socket the units activate,
/// as the manager would have made it.
pub(crate) fn add_socket(machine: &Machine) {
    let socket: PathBuf = socket_path(machine.host.runtime_dir.as_path(), machine.host.instance.as_str());
    std::fs::create_dir_all(socket.parent().expect("socket directory")).expect("directories");
    std::fs::write(&socket, b"").expect("socket stand-in");
}

/// The installed units count as in place only when both match what this wrapper would write and the socket is there:
///  two small reads and one `stat`,
/// no bus.
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

/// An earlier unit of the wrapper's own,
/// in either unit directory,
/// keeps the installation from counting as in place;
/// a file of
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

/// What:
///  A provisioning of an installation that is already what this command would write asks the manager nothing.
/// Why:
///  The manager is asked only about a unit it may hold changed or a socket that is not there;
/// a
///  commit whose only change is the program link (a release change, or a second build of one release) reads one link and
///  returns,
///  which is what keeps a first commit after an install from queueing behind a manager that is reading every
///  unit.
#[test]
fn a_provisioning_of_an_installation_that_is_in_place_asks_the_manager_nothing() {
    let machine: Machine = machine("provision-in-place-asks-nothing-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    assert!(in_place(&machine.host));
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned again");
    assert!(recorder.calls().is_empty(), "nothing to write and nothing to start, so nothing is asked: {:?}", recorder.calls());
    remove(machine.root.as_path());
}

/// A unit another release wrote,
/// whose text differs from this release's template and not only in the program path,
/// is
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

/// When the manager refuses after a unit was rewritten,
/// the rewrite is withdrawn,
/// so the next commit does not take a file the
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


/// What:
///  An upgrade whose process wrote the new service template and was killed before the manager was reloaded
///  repairs itself at the next commit:
/// the files alone are not taken for an installation the manager was told about.
/// Why:
///  The files then hold exactly the text this wrapper would write,
/// and a fast path that reads only the files takes them for
///  installed and returns without the reload.
/// The manager keeps the unit it loaded,
/// so every later commit reaches the service
///  of the release that was replaced and ends with `ClientFailure::OtherRelease` however often it is run
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// the sequence of the interrupted
///  upgrade).
/// A commit must not fail repeatedly because of a kill it cannot see,
/// so the next one asks for the reload the killed one
///  never made,
/// and only then are the files an installation again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(inPlaceAfterAKilledUpgrade).toBe(false);
/// expect(reloadAskedAfterRepairingIt).toBe(true);
/// ```
#[test]
fn an_upgrade_killed_before_the_reload_is_repaired_by_the_next_commit() {
    let machine: Machine = machine("provision-interrupted-upgrade-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("the replaced release provisioned");
    add_socket(&machine);
    assert!(in_place(&machine.host), "the replaced release is installed and the manager was told");
    // The upgrading release changed the service template itself (a line the other release did not write) and was killed
    // before it asked the manager to reload. A release that moves only its program leaves the template alone, and the record
    // with it (`service_provision_program_pointer.rs`), so this is the state that repair is still for.
    let name: &'static str = SERVICE_UNIT_NAME;
    let text: String = format!("{}# a line the upgrading release added\n", written(&machine, SERVICE_UNIT_NAME));
    std::fs::write(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(name), text).expect("the upgrading release's template");
    assert!(!in_place(&machine.host), "the manager was never told about the template the kill left behind");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("repaired");
    assert_eq!(recorder.calls().first().map(String::as_str), Some("reload"), "the reload the killed upgrade never made: {:?}", recorder.calls());
    assert!(in_place(&machine.host), "and the installation is one the manager was told about");
    remove(machine.root.as_path());
}

/// Units of an earlier release are recognized by their content and removed with their enabling link,
/// then a reload follows.
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

/// A unit of the earlier single service's name that something else wrote is never touched,
/// nothing else is written,
/// the manager is not asked,
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

/// The manager's refusal reaches the person in plain words without the manager's internal error name,
/// and a broken
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

/// What:
///  A session bus that carries no user service manager is the plain remedy for an account without user services,
/// not the bus's own words about a name.
/// Why:
///   A login session's bus answers every manager call with `ServiceUnknown` when no `systemd --user` owns the name,
/// and the manager's text says a bus name was not provided by any unit,
/// which tells a person nothing they can act on.
/// The remedy that works is the one for an account without user services.
#[test]
fn a_bus_without_a_user_manager_gets_the_plain_remedy_for_an_account_without_user_services() {
    let machine: Machine = machine("provision-bus-without-manager");
    let mut recorder: Recorder = Recorder::default();
    *recorder.failure.lock().expect("lock") = Some(CallError::Manager {
        name: String::from("org.freedesktop.DBus.Error.ServiceUnknown"),
        text: String::from("The name org.freedesktop.systemd1 was not provided by any .service files"),
    });
    let refused: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("no manager to start the service");
    assert_eq!(refused, ClientFailure::NoActivation(platform_without_manager()));
    let text: String = render(&refused);
    assert!(text.contains(crate::service_messages::LINGER_COMMAND), "the remedy is the one for an account without user services: {text}");
    assert!(!text.contains("org.freedesktop"), "the bus's own words do not reach the person: {text}");
    remove(machine.root.as_path());
}

/// Many first commits provisioning at once agree through idempotence alone:
/// every one succeeds and the files are whole.
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

/// The single service of the earlier releases,
/// which served every worktree,
/// is retired by the first provisioning of the release
///  that gives each worktree its own:
/// its socket and service are stopped (socket first),
/// its unit files and socket file are
///  removed,
/// the manager is reloaded,
/// and the worktree's own instance is started.
/// Nothing of it counts as in place.
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
        with_earlier_instance_questions(vec![
            format!("stop {LEGACY_SOCKET_UNIT_NAME}"),
            // The service is asked to retire and given the wait to finish its batch before the manager is asked to stop it.
            format!("state {LEGACY_SERVICE_UNIT_NAME}"),
            format!("stop {LEGACY_SERVICE_UNIT_NAME}"),
            format!("state {LEGACY_SOCKET_UNIT_NAME}"),
            format!("state {LEGACY_SERVICE_UNIT_NAME}"),
            String::from("reload"),
            format!("start {}", instance_socket())
        ])
    );
    assert!(!directory.join(LEGACY_SOCKET_UNIT_NAME).exists() && !directory.join(LEGACY_SERVICE_UNIT_NAME).exists());
    assert!(!earlier_socket.exists(), "the earlier socket's file is gone");
    add_socket(&machine);
    assert!(in_place(&machine.host));
    remove(machine.root.as_path());
}

/// The wrapper's units of the earlier single service in `directory`,
/// and the socket file it listened on.
pub(super) fn earlier_service_files(machine: &Machine, directory: &Path) -> PathBuf {
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

/// A stop job that is only queued has not stopped the service:
/// the new instance starts after the manager reports the earlier
///  service and its socket inactive,
/// and not before.
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
    assert!(service_states.len() >= 3, "asked until it was inactive: {calls:?}");
    assert!(position(&calls, format!("start {}", instance_socket()).as_str()) > service_states[2], "{calls:?}");
    remove(machine.root.as_path());
}

/// A service that is still stopping when the wait ends fails the commit with nothing started and nothing removed,
/// so the
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

/// A question to the manager that runs out of the retirement budget is a service still stopping,
/// told apart from a manager
///  that is not there:
/// the commit says what is stopping and how to stop it,
/// and does not name a missing service manager.
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

/// What:
///  A manager that does not answer within the exchange's time never fails the commit:
/// the units stay written,
/// the record of what it
///  was told stays unwritten,
/// the person reads what the wait was for once,
/// and the next commit asks again.
/// Why:
///   The units being written is what the manager needs,
/// and a service of this release that is already running can answer the request
///  however slowly the manager reads its units;
/// a commit that ended here told the person this account has no user services, which was false, and
///  left the change uncommitted (two commits of 2026-11-11, twenty seconds of provisioning each).
/// The record must not be written: only the manager's own answer may say it was told.
#[test]
fn a_manager_that_does_not_answer_never_fails_the_commit() {
    let machine: Machine = machine("provision-timeout-lib");
    let mut recorder: Recorder = Recorder::default();
    *recorder.failure.lock().expect("lock") = Some(CallError::TimedOut);
    let entries: Vec<crate::service_timing::capture::Captured> = crate::service_timing::capture::entries_of(|| {
        provision_with(&machine.host, &mut recorder).expect("a manager that does not answer is not a failure");
    });
    let said: Vec<String> = entries.iter().map(|entry| { return entry.message.clone(); }).collect();
    assert_eq!(said.len(), 1, "one line says what the wait was for: {said:?}");
    assert!(said[0].contains(&format!("within {} seconds", EXCHANGE_WAIT.as_secs())), "{said:?}");
    assert!(said[0].contains(instance_socket().as_str()), "{said:?}");
    assert!(said[0].contains("the commit goes on") && said[0].contains("the next commit asks the manager again"), "{said:?}");
    assert!(runtime_unit_dir(machine.host.runtime_dir.as_path()).join(SERVICE_UNIT_NAME).exists(), "the units are written");
    assert!(!crate::service_provision_loaded_unit_texts::remembers(machine.host.runtime_dir.as_path(), unit_texts(&machine.host).as_slice()), "the record is unwritten, so the next commit asks again");
    let mut next: Recorder = Recorder::default();
    add_socket(&machine);
    provision_with(&machine.host, &mut next).expect("the next commit asks again");
    assert_eq!(next.calls().first().map(String::as_str), Some("reload"), "the reload the manager never answered: {:?}", next.calls());
    remove(machine.root.as_path());
}

/// Each phase has a time of its own:
/// the questions of the retirement wait never get more than the wait allows,
/// and the start
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
    // The last budget is the earlier socket of the worktree's, and the one before it the start's.
    assert_eq!(budgets[budgets.len() - 2], EXCHANGE_WAIT, "the start gets a whole exchange's time again: {budgets:?}");
    let questions: &[std::time::Duration] = &budgets[1..budgets.len() - 2];
    assert!(!questions.is_empty(), "{budgets:?}");
    assert!(questions.iter().all(|budget| { return *budget <= machine.host.retirement_wait; }), "{budgets:?}");
    remove(machine.root.as_path());
}

/// A stop the manager refuses is a failure of the commit,
/// not a retirement.
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
///  sends the stop,
/// and a unit the manager does not hold counts as stopped.
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

/// The text of a unit the wrapper wrote,
/// and of one something else wrote.
const OWN_UNIT: &str = "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n";
const FOREIGN_UNIT: &str = "[Service]\nExecStart=/usr/bin/true\n";

/// An owned unit in the user's own directory is the unit that runs,
/// so it is stopped and removed;
/// an unrelated file of the
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
///  a file that is someone else's:
/// the manager is not asked to stop it,
/// and the commit names the file in the way.
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

/// A unit file that is gone after the ownership check is a removal someone else finished,
/// and any other refusal to remove
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

/// A service that does not leave by itself when it is asked to retire may be in the middle of a landing,
/// and the manager's stop
/// ends the whole unit,
/// Git children included (sol-1 round 2,
/// upgrade):
/// the stop is sent only to a service that has left,
/// and a
/// service that is still there when the wait ends fails the commit with a sentence that names the stop for the person to choose.
#[test]
fn an_earlier_service_that_is_still_working_is_asked_to_retire_and_never_stopped_by_the_wrapper() {
    let machine: Machine = machine("provision-retire-polite-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["active"]);
    let failure: ClientFailure = provision_with(&machine.host, &mut recorder).expect_err("the service kept working");
    assert!(matches!(&failure, ClientFailure::CouldNotStart(reason) if reason.contains(LEGACY_SERVICE_UNIT_NAME)), "{failure:?}");
    let calls: Vec<String> = recorder.calls();
    assert!(!calls.contains(&format!("stop {LEGACY_SERVICE_UNIT_NAME}")), "a service that may be landing is not stopped: {calls:?}");
    remove(machine.root.as_path());
}

/// A service that leaves between the question and the stop is stopped after it left,
/// never before.
#[test]
fn the_earlier_service_is_stopped_only_after_it_has_left() {
    let machine: Machine = machine("provision-retire-after-leaving-lib");
    earlier_service_files(&machine, runtime_unit_dir(machine.host.runtime_dir.as_path()).as_path());
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, LEGACY_SERVICE_UNIT_NAME, &["active", "deactivating", "inactive"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    let stop: usize = position(&calls, format!("stop {LEGACY_SERVICE_UNIT_NAME}").as_str());
    let service_states: Vec<usize> =
        calls.iter().enumerate().filter(|(_, call)| { return call.as_str() == format!("state {LEGACY_SERVICE_UNIT_NAME}"); }).map(|(index, _)| { return index; }).collect();
    assert!(service_states.iter().filter(|index| { return **index < stop; }).count() >= 3, "it was asked until it was inactive before the stop: {calls:?}");
    remove(machine.root.as_path());
}

//region the earlier release's socket of the same worktree
//  The earlier release set up `cli-git@<instance>.socket` for each worktree; the owner's pick of 2026-11-07 ("Stop the old
//  socket") has the new provisioning stop it, so that the batches its helper left open can be finished at once.

/// The instance of another worktree of the same account.
const OTHER_INSTANCE: &str = "fedcba9876543210fedcba9876543210";

/// The earlier release's units of an instance,
/// as the manager is asked about them.
fn earlier_socket() -> String {
    return earlier_socket_unit_instance(INSTANCE);
}

fn earlier_service() -> String {
    return earlier_service_unit_instance(INSTANCE);
}

/// What the earlier release wrote:
/// its two templates,
/// which every worktree's instance is made from.
fn earlier_templates(machine: &Machine) -> Vec<PathBuf> {
    let directory: PathBuf = runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&directory).expect("unit directory");
    let mut files: Vec<PathBuf> = Vec::new();
    for name in ["cli-git@.socket", "cli-git@.service"] {
        let file: PathBuf = directory.join(name);
        std::fs::write(&file, "# Written by the git wrapper for its commit service (cli-git-unit)\n[Unit]\n").expect("earlier template");
        files.push(file);
    }
    return files;
}

/// The socket file the manager made for the earlier unit of an instance.
fn earlier_socket_file(machine: &Machine, instance: &str) -> PathBuf {
    let file: PathBuf = earlier_socket_path(machine.host.runtime_dir.as_path(), instance);
    std::fs::create_dir_all(file.parent().expect("socket directory")).expect("directories");
    std::fs::write(&file, b"").expect("the earlier socket's file");
    return file;
}

/// The earlier socket of the same worktree is stopped,
/// after the instance of this release has started;
/// the units it was made
///  from are shared with every other worktree and stay.
#[test]
fn the_same_worktrees_earlier_socket_is_stopped_after_the_new_instance_started() {
    let machine: Machine = machine("provision-earlier-socket-stopped-lib");
    let templates: Vec<PathBuf> = earlier_templates(&machine);
    let socket_file: PathBuf = earlier_socket_file(&machine, INSTANCE);
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, earlier_socket().as_str(), &["active", "inactive"]);
    script(&recorder, earlier_service().as_str(), &["inactive"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    // The manager leaves the file of a stopped socket behind (measured with a throwaway socket unit). An earlier-release wrapper
    //  can make its unit listen again at that path at any moment, so the file is never unlinked here.
    assert!(socket_file.exists(), "the file is the manager's and an earlier wrapper's, not this program's to unlink");
    let calls: Vec<String> = recorder.calls();
    assert!(
        position(&calls, format!("stop-alone {}", earlier_socket()).as_str()) > position(&calls, format!("start {}", instance_socket()).as_str()),
        "{calls:?}"
    );
    assert!(templates.iter().all(|file| { return file.exists(); }), "the templates of the other worktrees' instances stay");
    remove(machine.root.as_path());
}

/// The stop spares the earlier service:
/// the manager stops a service that requires the stopped socket,
/// so the stop goes out in
///  the mode that names the one unit,
/// and no stop of the earlier service is ever sent.
#[test]
fn the_earlier_socket_is_stopped_without_stopping_the_unit_that_requires_it() {
    let machine: Machine = machine("provision-earlier-socket-alone-lib");
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, earlier_socket().as_str(), &["active"]);
    let socket_file: PathBuf = earlier_socket_file(&machine, INSTANCE);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    assert!(socket_file.exists(), "a socket the manager still reports active keeps its file; the next commit looks again");
    assert!(calls.contains(&format!("stop-alone {}", earlier_socket())), "{calls:?}");
    assert!(!calls.contains(&format!("stop {}", earlier_socket())), "a plain stop would stop the service too: {calls:?}");
    assert!(!calls.iter().any(|call| { return call.starts_with("stop") && call.ends_with(earlier_service().as_str()); }), "{calls:?}");
    remove(machine.root.as_path());
}

/// The earlier socket of another worktree is not this worktree's to stop:
/// no call names it.
#[test]
fn another_worktrees_earlier_socket_is_left_alone() {
    let machine: Machine = machine("provision-earlier-socket-other-lib");
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, earlier_socket_unit_instance(OTHER_INSTANCE).as_str(), &["active"]);
    script(&recorder, earlier_service_unit_instance(OTHER_INSTANCE).as_str(), &["active"]);
    let other_file: PathBuf = earlier_socket_file(&machine, OTHER_INSTANCE);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    assert!(!calls.iter().any(|call| { return call.contains(OTHER_INSTANCE); }), "{calls:?}");
    assert!(other_file.exists());
    assert!(!calls.iter().any(|call| { return call.starts_with("stop"); }), "nothing else is stopped: {calls:?}");
    remove(machine.root.as_path());
}

/// A worktree that never had an earlier socket has nothing to stop,
/// and a unit the manager has just unloaded is no error.
#[test]
fn a_missing_earlier_unit_is_not_an_error() {
    let machine: Machine = machine("provision-earlier-socket-missing-lib");
    let mut recorder: Recorder = Recorder::default();
    let entries = crate::service_timing::capture::entries_of(|| { provision_with(&machine.host, &mut recorder).expect("provisioned"); });
    assert!(recorder.calls().contains(&format!("state {}", earlier_socket())), "the manager was asked: {:?}", recorder.calls());
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("stop"); }), "{:?}", recorder.calls());
    assert!(entries.is_empty(), "nothing to say: {entries:?}");
    remove(machine.root.as_path());
    let vanished: Machine = self::machine("provision-earlier-socket-vanished-lib");
    let mut vanishing: Recorder = Recorder::default();
    script(&vanishing, earlier_socket().as_str(), &["active"]);
    *vanishing.stop_alone_failure.lock().expect("lock") = Some(CallError::Manager { name: NO_SUCH_UNIT.to_string(), text: String::from("not loaded") });
    let vanished_entries = crate::service_timing::capture::entries_of(|| { provision_with(&vanished.host, &mut vanishing).expect("provisioned"); });
    assert!(vanished_entries.is_empty(), "a unit that is gone is what the stop was for: {vanished_entries:?}");
    remove(vanished.root.as_path());
}

/// A manager that cannot be asked,
/// refuses,
/// or runs out of time for the earlier socket never fails the commit ("Commits should
///  never fail."):
/// the failure is written to the log and the provisioning is done.
#[test]
fn a_failure_to_stop_the_earlier_socket_is_logged_and_never_fails_provisioning() {
    let failures: [CallError; 4] = [
        CallError::Connection(String::from("broken pipe")),
        CallError::TimedOut,
        CallError::Manager { name: String::from("org.freedesktop.DBus.Error.AccessDenied"), text: String::from("permission denied") },
        CallError::Malformed(String::from("odd reply")),
    ];
    for failure in failures {
        let machine: Machine = machine("provision-earlier-socket-failure-lib");
        let mut recorder: Recorder = Recorder::default();
        script(&recorder, earlier_socket().as_str(), &["active"]);
        *recorder.stop_alone_failure.lock().expect("lock") = Some(failure.clone());
        let entries = crate::service_timing::capture::entries_of(|| { provision_with(&machine.host, &mut recorder).expect("provisioned"); });
        assert!(entries.iter().any(|entry| { return entry.message.contains(earlier_socket().as_str()); }), "{failure:?}: logged plainly, naming the unit: {entries:?}");
        assert!(recorder.calls().contains(&format!("start {}", instance_socket())), "the instance of this release was started");
        remove(machine.root.as_path());
    }
    let machine: Machine = machine("provision-earlier-socket-question-lib");
    let mut recorder: Recorder = Recorder::default();
    *recorder.state_failure.lock().expect("lock") = Some(CallError::TimedOut);
    let entries = crate::service_timing::capture::entries_of(|| { provision_with(&machine.host, &mut recorder).expect("provisioned"); });
    assert!(!entries.is_empty(), "a question that cannot be answered is logged too");
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("stop"); }), "nothing is stopped on a guess");
    remove(machine.root.as_path());
}

/// Every call about the earlier socket has a time of its own,
/// as the reload and the start do,
/// so that a bus that stalls delays
///  the commit by no more than one exchange.
#[test]
fn the_questions_about_the_earlier_socket_have_a_budget_of_their_own() {
    let machine: Machine = machine("provision-earlier-socket-budget-lib");
    let mut recorder: Recorder = Recorder::default();
    script(&recorder, earlier_socket().as_str(), &["active"]);
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let budgets = recorder.budgets.lock().expect("lock").clone();
    assert_eq!(budgets.len(), 2, "one for the start and one for the earlier socket: {budgets:?}");
    assert_eq!(budgets.last().copied(), Some(EXCHANGE_WAIT));
    remove(machine.root.as_path());
}

/// A helper of the earlier release that is running (or starting,
/// or stopping) finishes its batch:
/// nothing is stopped,
/// the log
///  stays quiet,
/// and the next provisioning looks again.
#[test]
fn a_running_earlier_helper_is_not_stopped_and_its_socket_is_kept() {
    for state in ["active", "activating", "deactivating", "reloading"] {
        let machine: Machine = machine("provision-earlier-helper-running-lib");
        let mut recorder: Recorder = Recorder::default();
        script(&recorder, earlier_socket().as_str(), &["active"]);
        script(&recorder, earlier_service().as_str(), &[state]);
        let socket_file: PathBuf = earlier_socket_file(&machine, INSTANCE);
        let entries = crate::service_timing::capture::entries_of(|| { provision_with(&machine.host, &mut recorder).expect("provisioned"); });
        let calls: Vec<String> = recorder.calls();
        assert!(calls.contains(&format!("state {}", earlier_service())), "{state}: the helper was looked for before anything was stopped: {calls:?}");
        assert!(!calls.iter().any(|call| { return call.starts_with("stop"); }), "{state}: {calls:?}");
        assert!(entries.is_empty(), "{state}: {entries:?}");
        assert!(socket_file.exists(), "{state}: the helper's socket file stays");
        remove(machine.root.as_path());
    }
}

/// A socket file whose unit the manager no longer holds,
/// with no helper running,
/// is left as it is:
/// an earlier-release wrapper
///  may be making the unit listen again at that very path,
/// and an unlink would take the live socket from it.
#[test]
fn a_socket_file_left_by_an_unloaded_earlier_unit_is_not_unlinked() {
    let machine: Machine = machine("provision-earlier-socket-stale-lib");
    let own: PathBuf = earlier_socket_file(&machine, INSTANCE);
    let other: PathBuf = earlier_socket_file(&machine, OTHER_INSTANCE);
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    assert!(own.exists() && other.exists());
    assert!(!recorder.calls().iter().any(|call| { return call.starts_with("stop"); }), "{:?}", recorder.calls());
    remove(machine.root.as_path());
}

/// Review finding of 2026-11-07 (cross-vendor,
/// round 6):
/// the socket-state answer and an unlink are separate steps,
/// and an
///  earlier-release wrapper can make its socket unit listen again in between,
/// so the unlink would take the live socket from
///  it and its next connect would fail.
/// The earlier wrapper here makes the file again after every answer and every stop;
///  provisioning finishes with that file as the wrapper made it.
#[test]
fn a_socket_the_earlier_wrapper_makes_again_during_provisioning_is_left_whole() {
    let machine: Machine = machine("provision-earlier-socket-recreated-lib");
    let file: PathBuf = earlier_socket_path(machine.host.runtime_dir.as_path(), INSTANCE);
    for states in [&["active", "inactive"][..], &["active"][..], &["inactive"][..]] {
        let mut recorder: Recorder = Recorder::default();
        *recorder.recreate.lock().expect("lock") = Some(file.clone());
        script(&recorder, earlier_socket().as_str(), states);
        provision_with(&machine.host, &mut recorder).expect("provisioned");
        assert_eq!(std::fs::read(&file).expect("the earlier wrapper's socket is still there"), b"live", "{states:?}");
        std::fs::remove_file(&file).expect("reset");
    }
    remove(machine.root.as_path());
}

/// A fake process table whose unix socket table holds `lines`.
fn proc_with_sockets(machine: &Machine, lines: &[String]) -> PathBuf {
    let proc_root: PathBuf = machine.root.join("proc");
    std::fs::create_dir_all(proc_root.join("net")).expect("net folder");
    std::fs::write(proc_root.join("net/unix"), format!("Num       RefCount Protocol Flags    Type St Inode Path\n{}", lines.concat())).expect("table");
    return proc_root;
}

/// One line of the table for a socket bound at `path`.
fn socket_line(flags: &str, path: &str) -> String {
    return format!("000000002982a48d: 00000002 00000000 {flags} 0001 01 2269401 {path}\n");
}

/// The earlier socket of a worktree decides whether the installation is in place,
/// by whether anything listens on it and not by
///  whether its file is there:
/// a stopped socket leaves the file,
/// and that must not send every later commit to the bus.
#[test]
fn an_earlier_socket_that_listens_keeps_the_installation_from_being_in_place_and_a_leftover_file_does_not() {
    let machine: Machine = machine("provision-earlier-socket-in-place-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    let runtime: String = machine.host.runtime_dir.display().to_string();
    let own_path: String = format!("{runtime}/cli-git/{INSTANCE}.sock");
    let other_path: String = format!("{runtime}/cli-git/{OTHER_INSTANCE}.sock");
    let own_file: PathBuf = earlier_socket_file(&machine, INSTANCE);
    let quiet: PathBuf = proc_with_sockets(&machine, &[]);
    assert!(in_place_with(&machine.host, quiet.as_path()), "a leftover file of a stopped socket is no reason to ask the manager");
    let listening: PathBuf = proc_with_sockets(&machine, &[socket_line("00010000", own_path.as_str())]);
    assert!(!in_place_with(&machine.host, listening.as_path()), "this worktree's earlier socket listens");
    let connected: PathBuf = proc_with_sockets(&machine, &[socket_line("00000000", own_path.as_str())]);
    assert!(in_place_with(&machine.host, connected.as_path()), "a socket that does not listen starts no helper");
    let other: PathBuf = proc_with_sockets(&machine, &[socket_line("00010000", other_path.as_str())]);
    assert!(in_place_with(&machine.host, other.as_path()), "another worktree's earlier socket says nothing about this one");
    std::fs::remove_file(machine.root.join("proc/net/unix")).expect("table removed");
    assert!(!in_place_with(&machine.host, machine.root.join("proc").as_path()), "a table that cannot be read leaves the question open");
    assert!(own_file.exists());
    remove(machine.root.as_path());
}
//endregion

//region the shared socket of the first releases
//  Found by the audit of the names the rename changed: the single service of the first releases listened on
//  `<runtime>/cli-git/service.sock` (tag `monochromatic-git-policy-cli-v0.5.34`, `service_instance.rs` line 26), a path of the
//  earlier program, which the rename had turned into this release's folder name.

/// The socket file of the single service of the first releases,
/// spelled out.
fn shared_earlier_socket(machine: &Machine) -> PathBuf {
    return machine.host.runtime_dir.join("cli-git").join("service.sock");
}

/// The constant names that file,
/// and not a file of this release's folder.
#[test]
fn the_shared_earlier_socket_is_in_the_earlier_programs_folder() {
    assert_eq!(LEGACY_SOCKET_RELATIVE, "cli-git/service.sock");
}

/// A socket file of the single service of the first releases keeps the installation from counting as in place,
/// and a file of
///  that name in this release's folder (which nothing makes) does not.
#[test]
fn the_shared_earlier_socket_file_keeps_the_installation_from_being_in_place() {
    let machine: Machine = machine("provision-shared-earlier-in-place-lib");
    provision_with(&machine.host, &mut Recorder::default()).expect("provisioned");
    add_socket(&machine);
    assert!(in_place(&machine.host));
    let elsewhere: PathBuf = machine.host.runtime_dir.join("monochromatic-git-policy-cli").join("service.sock");
    std::fs::write(&elsewhere, b"").expect("a file of this release's folder");
    assert!(in_place(&machine.host), "this release's folder is not the single service's");
    std::fs::remove_file(&elsewhere).expect("removed");
    let shared: PathBuf = shared_earlier_socket(&machine);
    std::fs::create_dir_all(shared.parent().expect("socket directory")).expect("directories");
    std::fs::write(&shared, b"").expect("the single service's socket file");
    assert!(!in_place(&machine.host));
    remove(machine.root.as_path());
}

/// The single service's socket file with no unit file anywhere is what a release that removed the files but not the service left
///  behind:
/// both unit names are stopped,
/// and the file is removed afterwards.
#[test]
fn a_shared_earlier_socket_file_without_unit_files_retires_the_service_at_its_own_path() {
    let machine: Machine = machine("provision-shared-earlier-retire-lib");
    let shared: PathBuf = shared_earlier_socket(&machine);
    std::fs::create_dir_all(shared.parent().expect("socket directory")).expect("directories");
    std::fs::write(&shared, b"").expect("the single service's socket file");
    let mut recorder: Recorder = Recorder::default();
    provision_with(&machine.host, &mut recorder).expect("provisioned");
    let calls: Vec<String> = recorder.calls();
    assert!(calls.contains(&format!("stop {LEGACY_SOCKET_UNIT_NAME}")) && calls.contains(&format!("stop {LEGACY_SERVICE_UNIT_NAME}")), "{calls:?}");
    assert!(!shared.exists(), "its socket file is gone");
    remove(machine.root.as_path());
}
//endregion
