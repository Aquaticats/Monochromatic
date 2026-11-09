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
}

impl StandIn {
    /// A manager that holds nothing and answers nothing.
    const fn empty() -> Self {
        return Self { socket: None, state: None, release: None, asked: Vec::new(), unit_executable_gone: false };
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
///  A service of another release that will not retire is proof that the unit the manager loaded and the files
///  disagree,
/// so the record of the texts the manager was told about is dropped.
/// Why:
///  The record can certify the files while the manager runs the release they replaced:
/// an interrupted downgrade
///  writes and reloads the older release's files and is killed before it records them,
/// and an interrupted upgrade
///  then writes the newer files without a reload,
/// which leaves the files and the record agreeing and the manager running
///  the older release.
/// Repeating the commit took the fast path into the same answer,
/// `ClientFailure::OtherRelease`,
/// for ever
///  (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 2 of the round-2 review).
/// The next commit now writes the units and reloads,
/// which is the one repair of that state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reachCurrent(configWithAnotherRelease)).toEqual({ otherRelease: true }); expect(recordExists()).toBe(false);
/// ```
#[test]
fn a_service_of_another_release_that_will_not_stop_drops_the_record_of_the_units() {
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
        !crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()),
        "the record is dropped, so the next commit sets the units up again"
    );
    done.store(true, std::sync::atomic::Ordering::Relaxed);
    serving.join().expect("the service's thread");
    crate::test_support::remove(runtime.as_path());
}


/// What:
///  A service of another release that takes the retire and then cannot come back still drops the record.
/// Why:
///  Seeing another release answer at all proves that the unit the manager loaded is not the one the
/// files describe,
///  whatever happens next:
/// the service that took the retire exits,
/// the manager tries its cached older executable,
///  which an upgrade may have removed,
/// and the next connection finds nothing,
/// so the wait ends with `DidNotStart`
///  and never reaches the retirement deadline that dropped the record
/// (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`,
/// finding 3 of the
///  round-3 review).
/// The record has to be dropped when the other release is seen,
/// or every later commit keeps the fast
///  path into the manager's unchanged unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await reachCurrent(answeringOnceFromAnotherRelease)).toEqual({ didNotStart: true }); expect(recordExists()).toBe(false);
/// ```
#[test]
fn a_service_of_another_release_that_takes_the_retire_and_never_returns_drops_the_record() {
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
        !crate::service_provision_loaded_unit_texts::remembers(runtime.as_path(), texts.as_slice()),
        "the record is dropped when the other release answers, whatever the wait then finds"
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
