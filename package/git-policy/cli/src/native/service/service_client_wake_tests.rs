//! What:
//!  Controls of the two one-way messages the client writes:
//!  what a running service is told, and that giving a record's programs up starts no service.
//! Why:
//!  The message that names the records which can no longer run is best effort,
//!  and it is the only step of a command that has nothing to wait for;
//!  a connection to the socket of a socket-activated unit is what asks the manager to start the service,
//!  so the controls read the manager's own answers rather than a service's.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! forgetPrograms(configWithInactiveUnit, gitDir, ids); expect(startsAsked).toBe(0);
//! ```

use super::forget_programs;
use crate::service_client_settings_and_watches::{Activation, ClientConfig, ManagerAnswer, ManagerWatch};
use crate::service_protocol::{ClientMessage, PROTOCOL_VERSION, RELEASE, ServiceMessage, decode_client, encode_service};
use crate::service_server::Bind;
use std::io::{BufRead, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// How many times the stand-in manager of these controls was asked to start the service,
/// which is the one act a command that only settles a record must never cause.
static STARTS_ASKED: AtomicUsize = AtomicUsize::new(0);

/// How long a control's stand-in service may take to answer:
/// a bound of its own, so no control waits on a thread that never ends.
const STAND_IN_PATIENCE: Duration = Duration::from_secs(30);

/// The manager's word for a service that is running now.
fn running(_bus: &Path, _unit: &str) -> ManagerAnswer {
    return ManagerAnswer::State(String::from("active"));
}

/// The manager's word for a service that is not running:
/// the state a person's own stop leaves, which a command may not repair by connecting.
fn stopped(_bus: &Path, _unit: &str) -> ManagerAnswer {
    return ManagerAnswer::State(String::from("inactive"));
}

/// A start the manager takes, counted, because a start is what these controls prove does not happen.
fn a_start_is_asked_for(_bus: &Path, _unit: &str) -> bool {
    let _counted = STARTS_ASKED.fetch_add(1, Ordering::SeqCst);
    return true;
}

/// A reset and a restart the manager takes, which no message of this file asks for.
fn a_repair_is_asked_for(_bus: &Path, _unit: &str) -> bool {
    return true;
}

/// A watch on a manager that answers `answer`, counts every start it is asked for, and asks it nothing else.
fn watching(answer: fn(&Path, &str) -> ManagerAnswer) -> ManagerWatch {
    return ManagerWatch {
        bus: std::path::PathBuf::from("/nowhere/the-session-bus"),
        unit: String::from("cli-git-test.service"),
        socket_unit: String::from("cli-git-test.socket"),
        every: Duration::from_millis(25),
        state: answer,
        start: a_start_is_asked_for,
        reset: a_repair_is_asked_for,
        restart: a_repair_is_asked_for,
    };
}

/// The settings of a client that reaches `address` and asks the manager in `watch`:
/// the waits are milliseconds, so a control ends in well under a second.
fn a_config(address: &str, watch: Option<ManagerWatch>) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(String::from(address)),
        activation: Activation::Managed,
        start_wait: Duration::from_millis(400),
        heartbeat_silence: Duration::from_millis(200),
        start_watch: watch,
        keep_fresh: None,
        answer_ceiling: Duration::from_millis(400),
        withdraw_ceiling: Duration::from_millis(400),
        waiting_notice_after: crate::wait_ceilings::WAITING_NOTICE_AFTER,
        extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT,
        start_ceiling: Duration::from_millis(400),
    };
}

/// An address no other control of this process uses.
fn an_address(label: &str) -> String {
    return format!("cli-git-wake-test-{}-{label}", std::process::id());
}

/// A listener on the abstract name `name`.
fn a_listener_on(name: &str) -> UnixListener {
    use std::os::linux::net::SocketAddrExt;
    let socket: std::os::unix::net::SocketAddr = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    return UnixListener::bind_addr(&socket).expect("bind the stand-in service");
}

/// What:
///  A command that only settles a push record asks the manager nothing and starts no service,
///  and nothing answers it, which is no failure:
///  a service that is not running holds none of the values this message would give up.
/// Why:
///   The message is best effort and the command has nothing to wait for,
///  so a connection made to a stopped service's socket would start a commit service for a command that only
///  wants to tell the memory of a service that ran before it,
///  and the person's command would wait for that start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = await forget(stoppedUnit, gitDir, ids); expect(asked).toBe(0); expect(result).toBe('done');
/// ```
#[test]
fn a_record_settled_where_no_service_runs_starts_none_to_tell_it() {
    STARTS_ASKED.store(0, Ordering::SeqCst);
    let config: ClientConfig = a_config(an_address("cold-service").as_str(), Some(watching(stopped)));
    let records: Vec<String> = vec![String::from("push-0123456789abcdef")];
    let told: Result<(), crate::service_messages::ClientFailure> = forget_programs(&config, Path::new("/repository/.git"), records.as_slice());
    assert!(told.is_ok(), "nothing to tell a service that does not run is no failure: {told:?}");
    assert_eq!(STARTS_ASKED.load(Ordering::SeqCst), 0, "the manager was asked to start a service a command only wanted to tell nothing");
}

/// What:
///  A service that is running is told exactly the records the command named.
/// Why:
///   The values live in that process's memory and the message is the only way they go,
///  so a message that named other records, or none, would leave them for the life of the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await forget(runningUnit, gitDir, ids); expect(capturedLine).toBe({ type: 'forget-programs', gitDir, records: ids });
/// ```
#[test]
fn a_running_service_is_told_which_records_can_no_longer_run() {
    STARTS_ASKED.store(0, Ordering::SeqCst);
    let name: String = an_address("running-service");
    let service: UnixListener = a_listener_on(name.as_str());
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    let serving = std::thread::spawn(move || {
        let (stream, _address) = service.accept().expect("the client connects to a service that is running");
        let mut greeting: String = encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) });
        greeting.push('\n');
        (&stream).write_all(greeting.as_bytes()).expect("greet");
        let mut line: String = String::new();
        std::io::BufReader::new(&stream).read_line(&mut line).expect("the client sends its message");
        sender.send(line).expect("the captured line");
        drop(stream);
    });
    let config: ClientConfig = a_config(name.as_str(), Some(watching(running)));
    let records: Vec<String> = vec![String::from("push-0123456789abcdef"), String::from("push-456789abcdef0123")];
    let told: Result<(), crate::service_messages::ClientFailure> = forget_programs(&config, Path::new("/repository/.git"), records.as_slice());
    assert!(told.is_ok(), "the service read the line: {told:?}");
    let line: String = receiver.recv_timeout(STAND_IN_PATIENCE).expect("the client wrote its message");
    let _joined = serving.join();
    assert_eq!(
        decode_client(line.as_str()),
        Ok(ClientMessage::ForgetPrograms { git_dir: std::path::PathBuf::from("/repository/.git"), records }),
        "the service is told the records the command settled and no others"
    );
    assert_eq!(STARTS_ASKED.load(Ordering::SeqCst), 0, "a service that answered needed no start");
}
