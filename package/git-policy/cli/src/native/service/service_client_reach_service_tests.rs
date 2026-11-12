//! What:
//!  Controls for reaching the commit service when the unit the manager holds is not running:
//! what the client asks the
//!  manager,
//! and what it does with the answer.
//! Why:
//!  Provisioning leaves an installed unit alone when its text is current and its socket file is there,
//! and systemd keeps
//!  a stopped or failed unit's socket file,
//! so the client is the only place that can repair either state.
//! Leaving it to the
//!  client also costs the healthy path nothing:
//! the manager is asked only after a connection found nothing listening
//!  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
//! findings 2 and 5).
//! Every control here hands the client a stand-in for the manager's two
//!  calls,
//! never the account's own manager.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const manager = { state: () => 'failed', start: () => service.listen() };
//! // expect(await reach(config)).toBeDefined();
//! ```
#![cfg(target_os = "linux")]

/// The client,
/// its settings and the manager watch it asks.
use super::*;
/// How the client asks the manager about a unit.
use crate::service_client_settings_and_watches::ManagerWatch;
/// The line a service greets with.
use crate::service_protocol::{PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_service};
use crate::service_server::Bind;
use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

/// What:
///  The stand-in for the account's manager that every control here shares.
/// Why:
///   `ManagerWatch` holds plain functions,
/// not a trait object,
/// so what a control configures has to outlive them in
///  one place of the test process.
/// A control sets it up under the lock before it starts reaching and reads it back after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const manager = { socket: undefined as string | undefined, state: 'failed', asked: [] as string[] };
/// ```
struct StandIn {
    /// The address the manager's start makes the service listen on,
    /// an abstract name so no path length can refuse it.
    socket: Option<String>,
    /// What `ActiveState` answers,
    /// until a start replaces it.
    state: Option<String>,
    /// The release the service greets with,
    /// and this release unless a control says otherwise.
    release: Option<String>,
    /// What the client asked for,
    /// in order.
    asked: Vec<String>,
    /// Whether the unit the manager holds names an executable an upgrade removed,
    /// so no start can bring it back until the manager reads the units again.
    unit_executable_gone: bool,
    /// Whether the service of the stand-in accepts connections and greets none of them,
    /// which is what a service stopped inside its own process looks like;
    /// a restart is what clears it.
    wedged: bool,
    /// Whether the thread of a stand-in service keeps accepting,
    /// which a control clears so no thread waits on a test that ended.
    keep_serving: bool,
}

impl StandIn {
    /// A manager that holds nothing and answers nothing.
    const fn empty() -> Self {
        return Self { socket: None, state: None, release: None, asked: Vec::new(), unit_executable_gone: false, wedged: false, keep_serving: true };
    }
}

/// How many connections the stand-in service greets before its thread ends:
/// a control reconnects a few times a second,
/// and this is a bound of its own so no thread waits on a test that ended.
const SERVED_CONNECTIONS: u32 = 400;

/// The one stand-in of this test process.
static MANAGER: Mutex<StandIn> = Mutex::new(StandIn::empty());

/// The `ActiveState` the client asks for,
/// recorded and answered from the stand-in.
fn stand_in_state(_bus: &Path, unit: &str) -> Option<String> {
    let mut manager = MANAGER.lock().expect("lock");
    let answer: Option<String> = manager.state.clone();
    manager.asked.push(format!("state {unit} {}", answer.as_deref().unwrap_or("nothing")));
    return answer;
}

/// The reset the client asks for,
/// recorded and taken from the stand-in.
fn stand_in_reset(_bus: &Path, unit: &str) -> bool {
    MANAGER.lock().expect("lock").asked.push(format!("reset-failed {unit}"));
    return true;
}

/// What:
///  The restart the client asks for,
/// recorded and performed from the stand-in:
/// the service stops and a fresh one greets the next connection.
/// Why:
///  A unit the manager reports as running that greets nothing is one whose program stopped inside its own process,
/// so the
///  stand-in answers the restart by making its service greet again,
/// which is what the manager's restart does.
fn stand_in_restart(_bus: &Path, unit: &str) -> bool {
    let mut manager = MANAGER.lock().expect("lock");
    manager.asked.push(format!("restart {unit}"));
    manager.wedged = false;
    manager.state = Some(String::from("active"));
    // A restart starts the program the unit names, which is this release's: the stand-in greets with this release from then
    //  on, as the manager's restart of a unit that names the program link does.
    manager.release = None;
    return true;
}

/// The start the client asks for,
/// recorded and performed from the stand-in:
/// one service greets the next connection,
/// as the manager's socket activation would have it.
fn stand_in_start(_bus: &Path, unit: &str) -> bool {
    let name: String = {
        let mut manager = MANAGER.lock().expect("lock");
        manager.asked.push(format!("start {unit}"));
        // A unit whose executable is gone starts nothing until the manager reads the units again.
        if manager.unit_executable_gone {
            return false;
        }
        match manager.socket.clone() {
            Some(name) => name,
            None => return false,
        }
    };
    let listener: UnixListener = match listen_at(name.as_str()) {
        Ok(listener) => listener,
        Err(error) => {
            MANAGER.lock().expect("lock").asked.push(format!("bind failed: {error}"));
            return false;
        }
    };
    // The service greets the connections the client makes next, one after another: a control of a service that will not
    //  retire reconnects until its wait ends.
    let _served = std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        for _attempt in 0..SERVED_CONNECTIONS {
            match listener.accept() {
                Ok((mut stream, _address)) => {
                    let greeting: String = manager_release();
                    let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(greeting) });
                    line.push('\n');
                    let _sent = stream.write_all(line.as_bytes());
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let mut manager = MANAGER.lock().expect("lock");
    manager.state = Some(String::from("active"));
    return true;
}

/// The release the stand-in service greets with:
/// this one unless a control named another.
fn manager_release() -> String {
    let named: Option<String> = MANAGER.lock().expect("lock").release.clone();
    return named.unwrap_or_else(|| { return String::from(RELEASE); });
}

/// The stand-in service's listening socket at an abstract name,
/// which no path length can refuse.
fn listen_at(name: &str) -> std::io::Result<UnixListener> {
    return UnixListener::bind_addr(&SocketAddr::from_abstract_name(name.as_bytes())?);
}

/// The reload the client asks for,
/// recorded and answered from the stand-in:
/// the manager reads its units again,
/// which replaces a unit whose executable is gone.
fn stand_in_reload(_bus: &Path) -> bool {
    let mut manager = MANAGER.lock().expect("lock");
    manager.asked.push(String::from("reload"));
    manager.unit_executable_gone = false;
    return true;
}

/// What the client asked of the stand-in,
/// in order.
fn asked() -> Vec<String> {
    return MANAGER.lock().expect("lock").asked.clone();
}

/// What:
///  The settings of a client that reaches a socket the stand-in's start makes,
/// asking the stand-in whenever nothing answers.
/// Why:
///   The waits are milliseconds,
/// so a control tells the failing case from the repaired one in well under a second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reachConfig({ socket, every: 40, startWait: 200, startCeiling: 5000 })
/// ```
fn config(socket: &str, unit: &str, socket_unit: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(String::from(socket)),
        activation: Activation::Managed,
        // A whole second, so the covered case cannot spend it unnoticed: the client must ask after the first answer.
        start_wait: Duration::from_secs(1),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: Some(ManagerWatch {
            bus: PathBuf::from("/nonexistent/bus"),
            unit: String::from(unit),
            socket_unit: String::from(socket_unit),
            every: Duration::from_millis(40),
            state: stand_in_state,
            start: stand_in_start,
            reset: stand_in_reset,
            restart: stand_in_restart,
            reload: stand_in_reload,
        }),
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        waiting_notice_after: Duration::from_secs(60),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_secs(5),
        units_record_dir: None,
    };
}

/// An abstract address of its own for one control,
/// so two controls of this process never share a socket.
fn address(name: &str) -> String {
    return format!("cli-git-test-reach-{}-{name}", std::process::id());
}

/// What:
///  A unit the manager reports `inactive` while nothing listens on its socket,
/// which is what a person's `systemctl --user stop` leaves:
/// the socket file stays,
///  provisioning counts the installation as in place,
/// and nothing listens.
/// Why:
///  The client asked the manager to start such a unit only after the start wait had passed,
/// and that wait is ten seconds
///  (`START_WAIT`),
/// so the first commit after a stop took ten seconds longer than native Git,
/// against the owner's rule "commits vs native git should not take 5s longer"
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 5).
/// An `inactive` unit with the client's connection refused is one nothing is starting,
/// so the first such answer is
///  reason enough to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(stateAsksBeforeTheStart).toBeLessThanOrEqual(2);
/// ```
#[test]
fn the_first_inactive_answer_asks_the_manager_to_start_the_unit() {
    let socket: String = address("client-reach-inactive");
    {
        let mut manager = MANAGER.lock().expect("lock");
        manager.socket = Some(socket.clone());
        manager.state = Some(String::from("inactive"));
        manager.asked.clear();
    }
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut announced: bool = false;
    let reached = reach(
        &config(socket.as_str(), unit.as_str(), socket_unit.as_str()),
        &mut std::io::sink(),
        &mut announced,
        &mut crate::commit_timing::ReachTrace::default(),
        false,
    );
    let asked_of_the_manager: Vec<String> = asked();
    let (reader, version, _release) = reached.unwrap_or_else(|failure| {
        panic!("the service the manager was asked to start must be reached: {failure:?} after {asked_of_the_manager:?}")
    });
    assert_eq!(version, PROTOCOL_VERSION);
    drop(reader);
    // The start comes from the first answer, not from the start wait: the answers before it are counted, not timed.
    let before_the_start: usize = asked_of_the_manager
        .iter()
        .take_while(|call| { return !call.starts_with("start "); })
        .filter(|call| { return call.starts_with("state "); })
        .count();
    assert!(before_the_start <= 2, "{before_the_start} state answers came before the start request: {asked_of_the_manager:?}");
}


/// What:
///  A service of another release that will not stop ends the wait with the words that name it,
/// and leaves the record of
///  the unit texts the manager was told about as it was.
/// Why:
///  Another release answering is a service that was started before the program link moved,
/// and the ret
///  that follows ends it;
/// it is no longer proof that the manager's unit text and the files disagree,
/// because a unit text names the link and
///  not a release (`service_provision_program_pointer.rs`).
/// Dropping the record here made every commit after a release change read every unit again,
///  which took 3,521 to 20,002 ms of provisioning on 2026-11-11T20:01-20:12-05:00 and failed two commits
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// findings 2 of the round-2 review and 3 of the round-3
///  review,
/// superseded by the program link).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reachCurrent(configWithAnotherRelease)).toEqual({ otherRelease: true }); expect(recordExists()).toBe(true);
/// ```
#[test]
fn a_service_of_another_release_that_will_not_stop_leaves_the_record_of_the_units() {
    let socket: String = address("client-reach-other-release");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the service listens");
    // The service answers every connection as a later release of this wrapper, which reads no retire of this one.
    let done: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let serving_done: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&done);
    let serving = std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        let began: std::time::Instant = std::time::Instant::now();
        while !serving_done.load(std::sync::atomic::Ordering::Relaxed) && began.elapsed() < Duration::from_secs(30) {
            match listener.accept() {
                Ok((mut stream, _address)) => {
                    let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from("another-release")) });
                    line.push('\n');
                    let _sent = stream.write_all(line.as_bytes());
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let runtime: PathBuf = crate::test_support::fixture("client-reach-other-release-runtime");
    let texts: [(&'static str, String); 1] = [("a.service", String::from("what the manager was told"))];
    crate::service_provision_loaded_unit_texts::record(runtime.as_path(), texts.as_slice()).expect("the record is written");
    assert!(crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()), "the record stands before the answer");
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut settings: ClientConfig = config(socket.as_str(), unit.as_str(), socket_unit.as_str());
    // The socket answers every time, so no manager is asked; the wait for the retire is the whole of this control.
    settings.start_watch = None;
    settings.start_wait = Duration::from_millis(300);
    settings.units_record_dir = Some(runtime.clone());
    let mut announced: bool = false;
    let failure: ClientFailure = reach_current(&settings, &mut std::io::sink(), &mut announced, &mut crate::commit_timing::ReachTrace::default(), false)
        .expect_err("a service of another release that will not stop ends the command");
    assert_eq!(failure, ClientFailure::OtherRelease);
    assert!(
        crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()),
        "the record stands, so the next commit writes nothing and reads nothing of the manager"
    );
    done.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the service's thread");
    crate::test_support::remove(runtime.as_path());
}


/// What:
///  A service of another release that has not left at the bound is ended once,
///  said once,
///  and waited for again,
/// so the commit lands instead of ending with nothing committed.
/// Why:
///  The bound is the commit's and a service that finishes a batch outlasts it;
/// the commit that ended there left the
///  person's change uncommitted (5 of 6 changes of release in the two-build reproduction of 2026-11-12,
/// 10,032 to
///  10,052 ms of a wait that then failed).
/// Ending it is what lets the manager start the program this release installs,
/// and one ending only:
///  a unit the manager starts from a text an upgrade left behind comes back as the other release again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reachCurrent(anotherReleaseThatOutlastsTheBound)).toBeDefined(); expect(asked()).toEqual(['restart ' + unit]);
/// ```
#[test]
fn a_service_of_another_release_that_outlasts_the_bound_is_ended_once_and_said_once() {
    let socket: String = address("client-reach-other-release-ends");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the service listens");
    MANAGER.lock().expect("lock").release = Some(String::from("another-release"));
    let done: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let serving_done: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&done);
    let serving = std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        let began: std::time::Instant = std::time::Instant::now();
        while !serving_done.load(std::sync::atomic::Ordering::Relaxed) && began.elapsed() < Duration::from_secs(30) {
            match listener.accept() {
                Ok((mut stream, _address)) => {
                    // The release the stand-in answers with, which its restart changes to this release's.
                    let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(manager_release()) });
                    line.push('\n');
                    let _sent = stream.write_all(line.as_bytes());
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut settings: ClientConfig = config(socket.as_str(), unit.as_str(), socket_unit.as_str());
    // The bound the commit grants the retiring service, and the ceiling it then waits under, both short so a control reads
    //  them in well under a second.
    settings.start_wait = Duration::from_millis(300);
    settings.start_ceiling = Duration::from_millis(3000);
    let mut notices: Vec<u8> = Vec::new();
    let mut announced: bool = false;
    let reader = reach_current(&settings, &mut notices, &mut announced, &mut crate::commit_timing::ReachTrace::default(), false)
        .expect("the service of another release is ended and the program of this release answers");
    drop(reader);
    let said: String = String::from_utf8_lossy(&notices).into_owned();
    assert_eq!(said.matches("has not stopped after").count(), 1, "said once: {said}");
    assert!(said.contains(unit.as_str()) && said.contains("this commit goes on"), "{said}");
    assert_eq!(asked().iter().filter(|call| { return call.starts_with("restart "); }).count(), 1, "ended once: {:?}", asked());
    done.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the service's thread");
    MANAGER.lock().expect("lock").release = None;
}

/// What:
///  A service of another release that takes the retire and cannot come back leaves the record too,
/// and the wait ends
///  with the words that name what it found.
/// Why:
///  The record says the manager was told about the texts on disk,
/// which stays true however the other release's service
///  ended;
/// what protects the next commit from a manager that starts a program an upgrade removed is that the unit
///  names the program link rather than an executable (`service_provision_program_pointer.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reachCurrent(answeringOnceFromAnotherRelease)).toEqual({ didNotStart: true }); expect(recordExists()).toBe(true);
/// ```
#[test]
fn a_service_of_another_release_that_takes_the_retire_and_never_returns_leaves_the_record() {
    let socket: String = address("client-reach-other-release-once");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the service listens");
    // One answer from another release, then nothing: the service left and the manager could not start its own.
    let serving = std::thread::spawn(move || {
        if let Ok((mut stream, _address)) = listener.accept() {
            let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from("another-release")) });
            line.push('\n');
            let _sent = stream.write_all(line.as_bytes());
        }
    });
    let runtime: PathBuf = crate::test_support::fixture("client-reach-other-release-once-runtime");
    let texts: [(&'static str, String); 1] = [("a.service", String::from("what the manager was told"))];
    crate::service_provision_loaded_unit_texts::record(runtime.as_path(), texts.as_slice()).expect("the record is written");
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut settings: ClientConfig = config(socket.as_str(), unit.as_str(), socket_unit.as_str());
    settings.start_watch = None;
    settings.start_wait = Duration::from_millis(200);
    settings.units_record_dir = Some(runtime.clone());
    let mut announced: bool = false;
    let failure: ClientFailure = reach_current(&settings, &mut std::io::sink(), &mut announced, &mut crate::commit_timing::ReachTrace::default(), false)
        .expect_err("a service that left and cannot come back ends the wait");
    assert_eq!(failure, ClientFailure::DidNotStart);
    assert!(
        crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()),
        "the record stands when the other release answers, whatever the wait then finds"
    );
    serving.join().expect("the service's thread");
    crate::test_support::remove(runtime.as_path());
}


/// What:
///  A service that never greets because the manager's unit names an executable an upgrade removed is
///  repaired by one reload of the units.
/// Why:
///  Two kills during a version switch leave the files and the record of what the manager was told
/// agreeing while
///  the manager holds the other release's unit;
/// its executable is gone,
/// so no reset and no start brings a
///  greeting and the command ends with "The commit service did not start" however often it is run
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the
///  round-4 review).
/// Reading the units again is the one step that replaces the program the unit names,
/// so the client asks for
///  it once,
/// drops the record,
/// and gives the wait one more ceiling;
/// a service that still does not greet ends the command
///  in bounded time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reach(configWithAGoneUnit)).toBeDefined(); expect(asked).toContain('reload'); expect(recordExists()).toBe(false);
/// ```
#[test]
fn a_service_that_never_greets_because_the_units_program_is_gone_is_repaired_by_one_reload() {
    let socket: String = address("client-reach-gone-unit");
    {
        let mut manager = MANAGER.lock().expect("lock");
        manager.socket = Some(socket.clone());
        manager.state = Some(String::from("inactive"));
        manager.unit_executable_gone = true;
        manager.asked.clear();
    }
    let runtime: PathBuf = crate::test_support::fixture("client-reach-gone-unit-runtime");
    let texts: [(&'static str, String); 1] = [("a.service", String::from("what the manager was told"))];
    crate::service_provision_loaded_unit_texts::record(runtime.as_path(), texts.as_slice()).expect("the record is written");
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut settings: ClientConfig = config(socket.as_str(), unit.as_str(), socket_unit.as_str());
    settings.start_wait = Duration::from_millis(100);
    settings.start_ceiling = Duration::from_millis(600);
    settings.units_record_dir = Some(runtime.clone());
    let mut announced: bool = false;
    let reached = reach(&settings, &mut std::io::sink(), &mut announced, &mut crate::commit_timing::ReachTrace::default(), false);
    let asked_of_the_manager: Vec<String> = asked();
    let (_reader, version, _release) = reached.unwrap_or_else(|failure| {
        panic!("the reload of the units must make the service startable: {failure:?} after {asked_of_the_manager:?}")
    });
    assert_eq!(version, PROTOCOL_VERSION);
    assert_eq!(asked_of_the_manager.iter().filter(|call| { return *call == "reload"; }).count(), 1, "one reload, and one only: {asked_of_the_manager:?}");
    assert!(
        !crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()),
        "the record is dropped, so the next command provisions again"
    );
    MANAGER.lock().expect("lock").unit_executable_gone = false;
    crate::test_support::remove(runtime.as_path());
}

/// What:
///  The stand-in service that accepts every connection and greets none until the stand-in is restarted,
/// which is what a service stopped inside its own process looks like.
/// Why:
///   The client could not tell this from a service that is still starting before the restart,
/// and the commit it held failed with "the commit service did not start" where nothing had stopped the service.
/// A connection is taken and closes at once while `wedged` stands,
/// so the client meets silence and never a greeting,
/// exactly as it does against a frozen program that the manager's socket still holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = listen_at(address); serve(listener, () => MANAGER.wedged ? close() : greet());
/// ```
fn serve_silent_until_restarted(listener: UnixListener) -> std::thread::JoinHandle<()> {
    return std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        while MANAGER.lock().expect("lock").keep_serving {
            match listener.accept() {
                Ok((mut stream, _address)) => {
                    if MANAGER.lock().expect("lock").wedged {
                        continue;
                    }
                    let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
                    line.push('\n');
                    let _sent = stream.write_all(line.as_bytes());
                }
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
}

/// What:
///  A unit the manager reports `active` whose service greets nothing is asked to restart,
/// once,
/// and the commit is reached after it does.
/// Why:
///  A service greets a connection before it restores anything,
/// so a running unit that greets nothing is one whose program
///  stopped inside its own process:
/// a frozen control group,
/// a debugger,
/// a suspended machine,
/// a wait in the kernel nothing ends.
/// The client waited `START_WAIT` (ten seconds) and then reported "The commit service did not start",
/// which is not what happened and
///  sends the person to a log that holds nothing about it
/// (`~/.claude/../sonnet-audits/integration-tip-never-stuck/report.md`,
/// finding 3 of the after-alpha.9 audit).
/// A restart is the one repair,
/// and the service survives being ended by design.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reach(config)).toBeDefined(); expect(asked).toContain('restart monochromatic-git-policy-cli@….service');
/// ```
#[test]
fn a_running_unit_that_greets_nothing_is_asked_to_restart_once_and_the_service_is_reached() {
    let socket: String = address("client-reach-wedged");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the service's address is bound");
    {
        let mut manager = MANAGER.lock().expect("lock");
        manager.socket = Some(socket.clone());
        manager.state = Some(String::from("active"));
        manager.wedged = true;
        manager.asked.clear();
    }
    let serving: std::thread::JoinHandle<()> = serve_silent_until_restarted(listener);
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut settings: ClientConfig = config(socket.as_str(), unit.as_str(), socket_unit.as_str());
    settings.start_ceiling = Duration::from_secs(30);
    let mut announced: bool = false;
    let mut notices: Vec<u8> = Vec::new();
    let reached = reach(&settings, &mut notices, &mut announced, &mut crate::commit_timing::ReachTrace::default(), false);
    let asked_of_the_manager: Vec<String> = asked();
    let (_reader, version, _release) = reached.unwrap_or_else(|failure| {
        panic!("the restart must make the service answer: {failure:?} after {asked_of_the_manager:?}, notices {:?}", String::from_utf8_lossy(notices.as_slice()))
    });
    assert_eq!(version, PROTOCOL_VERSION);
    let restarts: Vec<&String> = asked_of_the_manager.iter().filter(|call| { return call.starts_with("restart "); }).collect();
    assert_eq!(restarts.len(), 1, "one restart, and one only: {asked_of_the_manager:?}");
    // The wait says what it waits for before it acts, so a commit held for a restart is not a commit held in silence.
    let said: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert!(said.contains("nothing has answered at"), "the wait names what it waits for and where: {said:?}");
    assert!(said.contains("asked to restart"), "the restart is said: {said:?}");
    MANAGER.lock().expect("lock").keep_serving = false;
    serving.join().expect("the stand-in service's thread");
    MANAGER.lock().expect("lock").wedged = false;
    MANAGER.lock().expect("lock").keep_serving = true;
}


/// What:
///  A service that accepts every connection and greets none,
/// for the length of a control.
/// Why:
///  A connection that is taken and never answered is what a service stopped inside its own process looks like,
/// and a
///  control must be able to make it without a manager and without a real service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = takeEveryConnectionAndSayNothing(listener);
/// ```
fn take_connections_and_greet_nothing(listener: UnixListener) -> (std::sync::Arc<std::sync::atomic::AtomicBool>, std::sync::Arc<std::sync::Mutex<Vec<UnixStream>>>, std::thread::JoinHandle<()>) {
    let finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let serving_finished: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&finished);
    let held: std::sync::Arc<std::sync::Mutex<Vec<UnixStream>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let serving_held: std::sync::Arc<std::sync::Mutex<Vec<UnixStream>>> = std::sync::Arc::clone(&held);
    let serving = std::thread::spawn(move || {
        let _set = listener.set_nonblocking(true);
        while !serving_finished.load(std::sync::atomic::Ordering::Relaxed) {
            match listener.accept() {
                // The connection is held open and nothing is written on it: silence, never a close.
                Ok((stream, _address)) => serving_held.lock().expect("held").push(stream),
                Err(_nothing_yet) => std::thread::sleep(Duration::from_millis(5)),
            }
        }
    });
    return (finished, held, serving);
}

/// What:
///  Where a manager could not be asked and nothing answered,
/// the failure says what the wait proved and never that the service did not start.
/// Why:
///  `DidNotStart` was printed in this state,
/// where the service had started and then stopped inside its own process,
/// and it sent the
///  person to a log that holds nothing about it
/// (`~/.claude/../sonnet-audits/integration-tip-never-stuck/report.md`,
/// finding 3 of the after-alpha.9 audit).
/// The wait says what it waits for and where within a second,
/// so a commit held here is never a commit held in
///  silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reach(config)).toEqual({ nothingAnswered: { address, waited } }); expect(notices).toContain(address);
/// ```
#[test]
fn an_address_that_answers_nothing_with_no_manager_to_ask_is_named_truly() {
    let socket: String = address("client-reach-silent-no-manager");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the address is bound");
    let (finished, held, serving) = take_connections_and_greet_nothing(listener);
    let settings: ClientConfig = ClientConfig {
        address: Bind::Abstract(socket.clone()),
        activation: Activation::Managed,
        // Past the notice grace, so this control also reads the line the wait prints.
        start_wait: Duration::from_millis(1_200),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: None,
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        waiting_notice_after: Duration::from_secs(60),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_secs(5),
        units_record_dir: None,
    };
    let mut announced: bool = false;
    let mut notices: Vec<u8> = Vec::new();
    let began: std::time::Instant = std::time::Instant::now();
    let failure: ClientFailure = reach(&settings, &mut notices, &mut announced, &mut crate::commit_timing::ReachTrace::default(), false).expect_err("nothing answered, so nothing is reached");
    assert!(began.elapsed() < Duration::from_secs(3), "the wait ends at its own bound: {:?}", began.elapsed());
    let ClientFailure::NothingAnswered { address: named, waited } = &failure else { panic!("the failure says what the wait proved: {failure:?}") };
    assert!(named.contains(socket.as_str()), "the address is named: {named}");
    // The wait that ended is the one the wait counted: it lasts the start wait and no longer than the pause after the last
    //  look, so a reader of the record is told what the wait came to and never the bound it was granted.
    assert!(*waited >= settings.start_wait, "the wait lasted its bound: {waited:?}");
    assert!(*waited < settings.start_wait + Duration::from_millis(200), "and no longer than that plus one pause: {waited:?}");
    let rendered: String = crate::service_messages::render(&failure);
    assert!(rendered.contains(socket.as_str()), "the person reads the address: {rendered}");
    assert!(rendered.contains("signal") || rendered.contains("stopped") || rendered.contains("frozen"), "the words name what a silent address means: {rendered}");
    assert!(!rendered.contains("did not start"), "the service did start: {rendered}");
    let said: String = String::from_utf8_lossy(notices.as_slice()).into_owned();
    assert!(said.contains(socket.as_str()), "the wait names the address before it ends: {said:?}");
    assert!(said.contains("no service manager"), "and says whom it cannot ask: {said:?}");
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the stand-in service's thread");
    drop(held);
}

/// What:
///  An address nothing listens on is still the service that did not start.
/// Why:
///   The two states have different next steps,
/// and a word that covers both sends the person to the wrong one:
/// nothing
///  listening means no service was started,
/// and a connection nothing answers means one was and stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reach(unboundAddress)).toEqual({ didNotStart: true });
/// ```
#[test]
fn an_address_nothing_listens_on_is_the_service_that_did_not_start() {
    let socket: String = address("client-reach-nothing-listening");
    let settings: ClientConfig = ClientConfig {
        address: Bind::Abstract(socket),
        activation: Activation::Managed,
        start_wait: Duration::from_millis(200),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: None,
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        waiting_notice_after: Duration::from_secs(60),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_secs(5),
        units_record_dir: None,
    };
    let mut announced: bool = false;
    let failure: ClientFailure = reach(&settings, &mut std::io::sink(), &mut announced, &mut crate::commit_timing::ReachTrace::default(), false).expect_err("nothing listens, so nothing is reached");
    assert_eq!(failure, ClientFailure::DidNotStart);
}


/// The instance name every control's unit carries,
/// in the shape systemd's instance units use.
const INSTANCE: &str = "0123456789abcdef0123456789abcdef";

/// What:
///  A unit the manager reports as `failed`,
/// with its socket file still there and nothing listening,
/// which is what a stopped or failed start leaves behind.
/// Why:
///  Provisioning reads the two unit texts and the socket file and finds everything in place,
/// so nothing clears
///  the failed state and nothing starts the unit:
/// the client heard `failed` twice and gave up with "run the commit again",
/// which
///  cannot work (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2).
/// Reaching the service must repair it instead,
/// by asking the manager to start the unit the
///  failed state belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reach(config)).toBeDefined();
/// expect(asked).toContain('start monochromatic-git-policy-cli@….service');
/// ```
#[test]
fn a_unit_the_manager_calls_failed_is_asked_to_start_again_and_the_service_is_reached() {
    let socket: String = address("client-reach-failed");
    {
        let mut manager = MANAGER.lock().expect("lock");
        manager.socket = Some(socket.clone());
        manager.state = Some(String::from("failed"));
        manager.asked.clear();
    }
    let unit: String = crate::service_instance::service_unit_instance(INSTANCE);
    let socket_unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut announced: bool = false;
    let reached = reach(
        &config(socket.as_str(), unit.as_str(), socket_unit.as_str()),
        &mut std::io::sink(),
        &mut announced,
        &mut crate::commit_timing::ReachTrace::default(),
        false,
    );
    let asked_of_the_manager: Vec<String> = asked();
    let (reader, version, release) = reached.unwrap_or_else(|failure| {
        panic!("the service the manager was asked to start must be reached: {failure:?} after {asked_of_the_manager:?}")
    });
    assert_eq!(version, PROTOCOL_VERSION);
    assert_eq!(release.as_deref(), Some(RELEASE));
    drop(reader);
    // The reset comes first: a unit that failed too often refuses a start as "repeated too quickly".
    let reset: String = format!("reset-failed {unit}");
    let start: String = format!("start {unit}");
    let reset_at: Option<usize> = asked_of_the_manager.iter().position(|call| { return *call == reset; });
    let start_at: Option<usize> = asked_of_the_manager.iter().position(|call| { return *call == start; });
    assert!(reset_at.is_some(), "the failed state is forgotten: {asked_of_the_manager:?}");
    assert!(start_at.is_some(), "the unit is asked to start: {asked_of_the_manager:?}");
    assert!(reset_at < start_at, "the reset comes before the start: {asked_of_the_manager:?}");
    // The socket unit is required by the service unit, so its failed state is forgotten in the same repair.
    let socket_reset: String = format!("reset-failed {socket_unit}");
    assert!(asked_of_the_manager.iter().any(|call| { return *call == socket_reset; }), "the socket unit's failed state is forgotten too: {asked_of_the_manager:?}");
}

/// What:
///  A sink of the notice lines that keeps the line and the moment it was written,
/// so a control reads when the person saw it and not only what it said.
/// Why:
///   The audit's finding was the silence of the wait and not only its words:
/// a line the person reads when the
///  command is already over says nothing about the ten seconds before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const times = timesOfLines(); reach(config, times, ...); expect(times.lines()[0][0]).toBeLessThan(1500);
/// ```
struct TimesOfLines {
    /// When the wait began.
    began: std::time::Instant,
    /// Every line,
    /// with the moment it was written.
    lines: std::sync::Arc<std::sync::Mutex<Vec<(Duration, String)>>>,
}

impl TimesOfLines {
    /// A sink that records every line from now on.
    fn start() -> Self {
        return Self { began: std::time::Instant::now(), lines: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())) };
    }

    /// The lines written so far,
    /// each with the moment of its writing.
    fn lines(&self) -> Vec<(Duration, String)> {
        return self.lines.lock().expect("lines").clone();
    }
}

impl std::io::Write for TimesOfLines {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.lines.lock().expect("lines").push((self.began.elapsed(), String::from_utf8_lossy(bytes).into_owned()));
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A wait with no service manager to ask says what it waits for while it is still waiting,
/// never only when it ends.
/// Why:
///   The ten seconds of silence the audit measured were one socket read:
/// the attempt waited the whole start wait,
///  so the loop that says the line only reached it at the bound that ended the wait.
/// The wait still says the same thing within the grace,
/// and still
///  ends at the bound it was granted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(times.lines()).toHaveLength(1); expect(times.lines()[0][0]).toBeLessThan(startWait / 2);
/// ```
#[test]
fn the_notice_of_a_wait_with_no_manager_reaches_the_person_before_the_wait_ends() {
    let socket: String = address("client-reach-notice-while-waiting");
    let listener: UnixListener = listen_at(socket.as_str()).expect("the address is bound");
    let (finished, held, serving) = take_connections_and_greet_nothing(listener);
    let settings: ClientConfig = ClientConfig {
        address: Bind::Abstract(socket.clone()),
        activation: Activation::Managed,
        // Twice the notice grace, so the line and the end of the wait are a second apart at the least.
        start_wait: Duration::from_millis(2_400),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: None,
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        waiting_notice_after: Duration::from_secs(60),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_secs(5),
        units_record_dir: None,
    };
    let mut notices: TimesOfLines = TimesOfLines::start();
    let mut announced: bool = false;
    let began: std::time::Instant = std::time::Instant::now();
    let failure: ClientFailure = reach(&settings, &mut notices, &mut announced, &mut crate::commit_timing::ReachTrace::default(), false)
        .expect_err("nothing answered, so nothing is reached");
    let waited: Duration = began.elapsed();
    // A formatting call may hand the sink the text and its line feed in two writes, so the text is read as one and the
    //  moment is the first write's.
    let said: Vec<(Duration, String)> = notices.lines();
    let text: String = said.iter().map(|(_when, line)| { return line.as_str(); }).collect();
    let telling: Vec<&(Duration, String)> = said.iter().filter(|(_when, line)| { return line.contains("nothing has answered at"); }).collect();
    assert_eq!(telling.len(), 1, "one line, and one only: {text:?}");
    let (when, _line): &(Duration, String) = telling[0];
    assert!(text.contains(socket.as_str()), "the line names the address: {text}");
    assert!(*when < Duration::from_millis(1_500), "the line is read while the wait goes on ({when:?} of {waited:?}): {text}");
    assert!(waited >= settings.start_wait, "and the wait still ends at its own bound: {waited:?}");
    assert!(waited < settings.start_wait + Duration::from_millis(200), "and no later than that plus one pause: {waited:?}");
    assert!(matches!(failure, ClientFailure::NothingAnswered { .. }), "the end says what the wait proved: {failure:?}");
    finished.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the stand-in service's thread");
    drop(held);
}
