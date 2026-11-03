//! What:
//!  The client's wait for the commit service, with a fake service that sends exactly what a control says: heartbeats through
//!  a long batch, heartbeats that stop, a service that never starts and one that starts late.
//! Why:
//!  The wait has no clock of its own (the owner's rule "You should (almost) never be waiting." and "Commits should never
//!  fail."): only the service's silence, or the manager saying the start failed, ends it. A control that runs a real service
//!  cannot make it stall on cue, so the service here is a listener that follows a script.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await submit(config, request); // while the fake service sends beats, however long
//! ```

use git_policy_cli::service_client::{Activation, ClientConfig, ManagerWatch, Submitted, submit};
use git_policy_cli::service_messages::{ClientFailure, SILENT_NOTICE};
use git_policy_cli::service_protocol::{
    ALLOWED_MODES, Change, CommitRequest, PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_service,
};
use git_policy_cli::service_server::Bind;
use std::io::{BufRead, BufReader, Write};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How long the fake service may be silent before the client under test treats it as stopped: short, so a control is quick.
const QUICK_SILENCE: Duration = Duration::from_millis(300);

/// How often the fake service beats.
const FAKE_BEAT: Duration = Duration::from_millis(60);

/// The longest the controls wait for anything: far beyond what a healthy run needs, so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(60);

/// A start wait the controls would exceed many times over if it still bounded a start the manager can be asked about.
const OLD_START_WAIT: Duration = Duration::from_millis(200);

/// An address no other control uses.
fn address(label: &str) -> String {
    return format!("cli-git-heartbeat-test-{}-{label}", std::process::id());
}

/// A listener on `name`.
fn listen(name: &str) -> UnixListener {
    let socket = SocketAddr::from_abstract_name(name.as_bytes()).expect("name");
    return UnixListener::bind_addr(&socket).expect("bind the fake service");
}

/// The line a service sends for `message`.
fn line(message: &ServiceMessage) -> Vec<u8> {
    let mut text: String = encode_service(message);
    text.push('\n');
    return text.into_bytes();
}

/// Greets as a service of this release does.
fn greet(stream: &UnixStream) {
    (&*stream).write_all(&line(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(String::from(RELEASE)) })).expect("greet");
}

/// Reads the client's request line.
fn read_request(stream: &UnixStream) {
    let mut request: String = String::new();
    BufReader::new(stream).read_line(&mut request).expect("the client sends its request");
}

/// A request that needs no repository, because the fake service never looks at it.
fn request() -> CommitRequest {
    return CommitRequest {
        id: String::from("heartbeat-control"),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/.git"),
        reference: String::from("refs/heads/main"),
        base: "a".repeat(40),
        changes: vec![Change { path: String::from("a.txt"), mode: ALLOWED_MODES[0], blob: Some("b".repeat(40)) }],
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/.git/index"),
        captured: 0,
    };
}

/// A client that treats `QUICK_SILENCE` as the end of a service.
fn config(name: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(name.to_string()),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: QUICK_SILENCE,
        start_watch: None,
    };
}

/// Runs the client on a thread and returns what it ended with and what it printed.
fn client(config: ClientConfig) -> std::thread::JoinHandle<(Result<Submitted, ClientFailure>, String)> {
    return std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let answered = submit(&config, &request(), &mut notices);
        return (answered, String::from_utf8(notices).expect("UTF-8"));
    });
}

/// A service that keeps sending beats through a wait many times longer than the silence bound is waited for, and its answer
/// is taken; the wait has no limit of the client's own.
#[test]
fn a_service_that_keeps_sending_heartbeats_through_a_long_batch_is_waited_for() {
    let name: String = address("long-batch");
    let service: UnixListener = listen(name.as_str());
    let waiting = client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let batch: Duration = QUICK_SILENCE * 8;
    let began: Instant = Instant::now();
    while began.elapsed() < batch {
        (&stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    let commit: String = "c".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false })).expect("answer");
    let (answered, notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert!(!notices.contains(SILENT_NOTICE), "a service that beats is never called silent: {notices:?}");
}

/// A service that stops sending beats ends the wait: the client says so, asks again with the same request, and meets a
/// service that accepts and greets nothing, which is the same silence; it ends with the plain failure that says whether the
/// commit may have been made and what to do.
#[test]
fn a_service_that_stops_sending_heartbeats_ends_the_wait_and_the_request_is_sent_again() {
    let name: String = address("goes-silent");
    let service: UnixListener = listen(name.as_str());
    let sends: std::sync::Arc<AtomicUsize> = std::sync::Arc::new(AtomicUsize::new(0));
    let counted: std::sync::Arc<AtomicUsize> = std::sync::Arc::clone(&sends);
    let hung = std::thread::spawn(move || {
        // The first try meets a service that beats twice and then stops; the next meets a hung one that never greets.
        let mut held: Vec<UnixStream> = Vec::new();
        let (first, _) = service.accept().expect("the first try");
        greet(&first);
        read_request(&first);
        counted.fetch_add(1, Ordering::SeqCst);
        for _ in 0..2 {
            (&first).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        }
        held.push(first);
        let (later, _) = service.accept().expect("the next try");
        held.push(later);
        return held;
    });
    let (answered, notices) = client(config(name.as_str())).join().expect("client thread");
    let held = hung.join().expect("fake service");
    assert_eq!(answered, Err(ClientFailure::ServiceSilent), "{notices}");
    assert_eq!(held.len(), 2, "the client asked again once, and was met by the same silence");
    assert_eq!(sends.load(Ordering::SeqCst), 1, "only the first connection was greeted, so only it got the request");
    assert!(notices.contains(SILENT_NOTICE), "{notices:?}");
}

/// Whether the fake manager says the unit failed.
static MANAGER_SAYS_FAILED: AtomicBool = AtomicBool::new(false);

/// Whether the fake manager says the unit is inactive (it is `activating` otherwise).
static MANAGER_SAYS_INACTIVE: AtomicBool = AtomicBool::new(false);

/// How many times the fake manager was asked to start the unit.
static START_REQUESTS: AtomicUsize = AtomicUsize::new(0);

/// The fake manager's answer.
fn fake_manager(_bus: &Path, _unit: &str) -> Option<String> {
    let word: &str = if MANAGER_SAYS_FAILED.load(Ordering::SeqCst) {
        "failed"
    } else if MANAGER_SAYS_INACTIVE.load(Ordering::SeqCst) {
        "inactive"
    } else {
        "activating"
    };
    return Some(String::from(word));
}

/// The fake manager takes a request to start the unit and counts it.
fn fake_start(_bus: &Path, _unit: &str) -> bool {
    START_REQUESTS.fetch_add(1, Ordering::SeqCst);
    return true;
}

/// A watch on the fake manager that asks quickly.
fn watched(name: &str) -> ClientConfig {
    let watch: ManagerWatch = ManagerWatch { bus: PathBuf::from("/nowhere/bus"), unit: String::from("cli-git@fake.service"), every: Duration::from_millis(50), state: fake_manager, start: fake_start };
    return ClientConfig { start_wait: OLD_START_WAIT, heartbeat_silence: PATIENCE, start_watch: Some(watch), ..config(name) };
}

/// A service that never starts is waited for as long as the manager does not say the start failed, however long that is
/// against the old start wait, and the wait ends with the plain failure once the manager says it twice.
#[test]
fn a_service_that_never_starts_ends_the_wait_when_the_manager_says_the_start_failed() {
    let name: String = address("never-starts");
    // Connects, and the service never greets: the operating system holds the connection for a program that does not come up.
    let service: UnixListener = listen(name.as_str());
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    let waiting = client(watched(name.as_str()));
    let held = std::thread::spawn(move || {
        let mut streams: Vec<UnixStream> = Vec::new();
        while let Ok((stream, _)) = service.accept() {
            streams.push(stream);
            if streams.len() >= 64 {
                break;
            }
        }
        return streams;
    });
    std::thread::sleep(OLD_START_WAIT * 6);
    assert!(!waiting.is_finished(), "the wait ended on a clock although the manager had not said the start failed");
    MANAGER_SAYS_FAILED.store(true, Ordering::SeqCst);
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Err(ClientFailure::DidNotStart));
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    drop(UnixStream::connect_addr(&SocketAddr::from_abstract_name(name.as_bytes()).expect("name")));
    drop(held);
}

/// A service that starts a long while after the client asks, against the old start wait, is waited for and answers.
#[test]
fn a_service_that_starts_late_is_waited_for_while_the_manager_does_not_say_failed() {
    let name: String = address("starts-late");
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    let waiting = client(watched(name.as_str()));
    std::thread::sleep(OLD_START_WAIT * 6);
    assert!(!waiting.is_finished(), "the wait ended before the service started");
    let service: UnixListener = listen(name.as_str());
    let (stream, _) = service.accept().expect("the client connects once the service listens");
    greet(&stream);
    read_request(&stream);
    let commit: String = "d".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false })).expect("answer");
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
}

/// A unit the manager reports `inactive` after the start wait, while the socket holds the client's connection and nothing starts
/// it, is started by the client's request to the manager; the client goes on waiting and the service then answers. A unit the
/// manager reports `activating` is never asked to start.
#[test]
fn an_inactive_unit_is_asked_to_start_after_the_start_wait_and_an_activating_one_is_not() {
    let name: String = address("inactive-unit");
    START_REQUESTS.store(0, Ordering::SeqCst);
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    MANAGER_SAYS_INACTIVE.store(false, Ordering::SeqCst);
    let service: UnixListener = listen(name.as_str());
    let waiting = client(watched(name.as_str()));
    // Activating for several start waits: the manager is starting the unit itself, so the client does not ask.
    std::thread::sleep(OLD_START_WAIT * 4);
    assert_eq!(START_REQUESTS.load(Ordering::SeqCst), 0, "an activating unit is not asked to start");
    MANAGER_SAYS_INACTIVE.store(true, Ordering::SeqCst);
    let began: Instant = Instant::now();
    while START_REQUESTS.load(Ordering::SeqCst) == 0 {
        assert!(began.elapsed() < PATIENCE, "an inactive unit was never asked to start");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!waiting.is_finished(), "asking the manager to start the unit ends no wait");
    // The manager started the service: it greets and answers.
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let commit: String = "e".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false })).expect("answer");
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
    MANAGER_SAYS_INACTIVE.store(false, Ordering::SeqCst);
}

/// A manager that cannot be asked leaves the wait to the start wait, as before.
#[test]
fn a_manager_that_cannot_be_asked_leaves_the_wait_to_the_start_wait() {
    let name: String = address("unaskable");
    fn unreachable_manager(_bus: &Path, _unit: &str) -> Option<String> {
        return None;
    }
    let mut unaskable: ClientConfig = watched(name.as_str());
    unaskable.start_watch = unaskable.start_watch.map(|watch| { return ManagerWatch { state: unreachable_manager, ..watch }; });
    let service: UnixListener = listen(name.as_str());
    let held = std::thread::spawn(move || { return service.accept().map(|(stream, _)| { return stream; }); });
    let (answered, _notices) = client(unaskable).join().expect("client thread");
    assert_eq!(answered, Err(ClientFailure::DidNotStart));
    drop(UnixStream::connect_addr(&SocketAddr::from_abstract_name(name.as_bytes()).expect("name")));
    drop(held);
}
