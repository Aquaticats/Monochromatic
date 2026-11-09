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

/// The client, its settings and the manager watch it asks.
use super::*;
/// How the client asks the manager about a unit.
use crate::service_client_settings_and_watches::ManagerWatch;
/// The line a service greets with.
use crate::service_protocol::{PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_service};
use crate::service_server::Bind;
use std::io::Write;
use std::os::unix::net::UnixListener;
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
    /// The socket the manager's start makes the service listen on.
    socket: Option<PathBuf>,
    /// What `ActiveState` answers,
    /// until a start replaces it.
    state: Option<String>,
    /// What the client asked for,
    /// in order.
    asked: Vec<String>,
}

impl StandIn {
    /// A manager that holds nothing and answers nothing.
    const fn empty() -> Self {
        return Self { socket: None, state: None, asked: Vec::new() };
    }
}

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

/// The start the client asks for,
/// recorded and performed from the stand-in:
/// one service greets the next connection,
/// as the manager's socket activation would have it.
fn stand_in_start(_bus: &Path, unit: &str) -> bool {
    let socket: PathBuf = {
        let mut manager = MANAGER.lock().expect("lock");
        manager.asked.push(format!("start {unit}"));
        match manager.socket.clone() {
            Some(socket) => socket,
            None => return false,
        }
    };
    let Ok(listener) = UnixListener::bind(socket) else { return false };
    // The service greets the connection the client makes next, then the stand-in has done its part.
    let _served = std::thread::spawn(move || {
        let Ok((mut stream, _address)) = listener.accept() else { return };
        let mut line: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
        line.push('\n');
        let _sent = stream.write_all(line.as_bytes());
    });
    let mut manager = MANAGER.lock().expect("lock");
    manager.state = Some(String::from("active"));
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
fn config(socket: &Path, unit: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Path(socket.to_path_buf()),
        activation: Activation::Managed,
        start_wait: Duration::from_millis(200),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: Some(ManagerWatch { bus: PathBuf::from("/nonexistent/bus"), unit: String::from(unit), every: Duration::from_millis(40), state: stand_in_state, start: stand_in_start }),
        keep_fresh: None,
        answer_ceiling: Duration::from_secs(5),
        withdraw_ceiling: Duration::from_secs(5),
        extra_wait: Duration::from_secs(1),
        start_ceiling: Duration::from_secs(5),
    };
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
/// expect(asked).toContain('start monochromatic-git-policy-cli@…socket');
/// ```
#[test]
fn a_unit_the_manager_calls_failed_is_asked_to_start_again_and_the_service_is_reached() {
    let root: PathBuf = crate::test_support::fixture("client-reach-failed");
    let socket: PathBuf = root.join("service.sock");
    {
        let mut manager = MANAGER.lock().expect("lock");
        manager.socket = Some(socket.clone());
        manager.state = Some(String::from("failed"));
        manager.asked.clear();
    }
    let unit: String = crate::service_instance::socket_unit_instance(INSTANCE);
    let mut announced: bool = false;
    let reached = reach(&config(socket.as_path(), unit.as_str()), &mut std::io::sink(), &mut announced, &mut crate::commit_timing::ReachTrace::default(), false);
    let asked_of_the_manager: Vec<String> = asked();
    let (reader, version, release) = reached.unwrap_or_else(|failure| {
        panic!("the service the manager was asked to start must be reached: {failure:?} after {asked_of_the_manager:?}")
    });
    assert_eq!(version, PROTOCOL_VERSION);
    assert_eq!(release.as_deref(), Some(RELEASE));
    drop(reader);
    assert!(asked_of_the_manager.iter().any(|call| { return call == format!("start {unit}").as_str(); }), "{asked_of_the_manager:?}");
    crate::test_support::remove(root.as_path());
}
