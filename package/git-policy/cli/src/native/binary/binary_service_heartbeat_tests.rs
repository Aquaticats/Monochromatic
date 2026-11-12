//! What:
//!  The client's wait for the commit service,
//! with a fake service that sends exactly what a control says:
//! heartbeats through
//!  a long batch,
//! heartbeats that stop,
//! a service that never starts and one that starts late.
//! Why:
//!  A wait that ended on nothing but the service's work would break the owner's rules
//!  "You should (almost) never be waiting."
//! and "Commits should never
//!  fail.";
//! so the service's silence,
//! the manager saying the start failed,
//! and the start and answer ceilings each end the wait.
//! A control that runs a real service
//!  cannot make it stall on cue,
//! so the service here is a listener that follows a script.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await submit(config, request); // while the fake service sends beats, however long
//! ```

use git_policy_cli::commit_timing::SubmitTrace;
use git_policy_cli::scratch_location::{SCRATCH_STALE_AFTER, sweep_stale};
use git_policy_cli::service_client::{Activation, ClientConfig, FreshScratch, ManagerWatch, Submitted, submit, submit_traced};
use git_policy_cli::service_messages::{ClientFailure, position_notice, silent_notice};

/// The words a commit the service named no place for is told,
/// as the person reads them,
/// without the seconds it waited.
const UNPLACED_SAID: &str = "has not said where this commit stands";
use git_policy_cli::service_protocol::{
    ALLOWED_MODES, Change, ClientMessage, CommitRequest, PROTOCOL_VERSION, RELEASE, ServiceMessage, decode_client, encode_service,
};
use git_policy_cli::service_server::Bind;
use std::io::{BufRead, BufReader, Write};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

/// How long the fake service may be silent before the client under test treats it as stopped:
/// short,
/// so a control is quick.
const QUICK_SILENCE: Duration = Duration::from_millis(300);

/// How often the fake service beats.
const FAKE_BEAT: Duration = Duration::from_millis(60);

/// How long a request waits before the client says where the commit stands,
/// short so a control is quick.
const QUICK_NOTICE: Duration = Duration::from_millis(250);

/// The longest the controls wait for anything:
/// far beyond what a healthy run needs,
/// so only a hang reaches it.
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

/// A request that needs no repository,
/// because the fake service never looks at it.
fn request() -> CommitRequest {
    return CommitRequest {
        id: String::from("heartbeat-control"),
        repository: PathBuf::from("/nowhere"),
        common_dir: PathBuf::from("/nowhere/.git"),
        git_dir: PathBuf::from("/nowhere/.git"),
        reference: git_policy_cli::ref_name::RefName::from_text("refs/heads/main"),
        base: "a".repeat(40),
        changes: vec![Change { path: PathBuf::from("a.txt"), mode: ALLOWED_MODES[0], blob: Some("b".repeat(40)) }],
        message: String::from("m\n"),
        author: None,
        committer: String::from("T <t@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: PathBuf::from("/nowhere/.git/index"),
        scratch: PathBuf::from("/nowhere/.git/monochromatic-git-policy-cli-scratch/monochromatic-git-policy-cli-service-heartbeat-control-0"),
        captured: 0,
        holder: String::new(),
    };
}

/// A client that treats `QUICK_SILENCE` as the end of a service.
fn config(name: &str) -> ClientConfig {
    return ClientConfig {
        address: Bind::Abstract(name.to_string()),
        activation: Activation::Managed,
        start_wait: PATIENCE,
        heartbeat_silence: QUICK_SILENCE,
        start_watch: None, keep_fresh: None, answer_ceiling: git_policy_cli::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: git_policy_cli::wait_ceilings::WITHDRAW_CEILING, extra_wait: git_policy_cli::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: git_policy_cli::wait_ceilings::START_CEILING, waiting_notice_after: QUICK_NOTICE, units_record_dir: None,
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

/// A service that keeps sending beats through a wait many times longer than the silence bound is waited for,
/// and its answer
/// is taken;
/// the wait has no limit of the client's own.
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
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert!(!notices.contains(silent_notice(QUICK_SILENCE).as_str()), "a service that beats is never called silent: {notices:?}");
}

/// What:
///  A commit the service named no place for is told once that the service has not said where it stands,
/// and told nothing about
///  whether the landing has begun.
/// Why:
///   The client knows only that no answer has come and that no place was named:
/// the landing may be waiting behind
///  another program's `index.lock`,
/// so words that say the service has not started on the commit would be false.
#[test]
fn a_commit_the_service_named_no_place_for_is_told_the_service_has_not_said_where_it_stands() {
    let name: String = address("unplaced");
    let service: UnixListener = listen(name.as_str());
    let waiting = client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let waited: Instant = Instant::now();
    while waited.elapsed() < QUICK_NOTICE * 4 {
        (&stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    let commit: String = "d".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert_eq!(notices.matches(UNPLACED_SAID).count(), 1, "said once, and only once: {notices:?}");
    assert!(!notices.contains("has not started on this commit") && !notices.contains("still reading it"), "the client cannot know that the landing has not begun: {notices:?}");
    assert!(!notices.contains(silent_notice(QUICK_SILENCE).as_str()), "the service was never silent: {notices:?}");
}

/// How long the commit of the stalled-stream control may take to answer:
/// far beyond the service's own answer,
/// which is written before the control waits.
const ANSWERED_WHILE_THE_STREAM_IS_HELD: Duration = Duration::from_secs(10);

/// What:
///  A stream that takes one line and then stops,
/// as a pipe whose reader is still open and no longer draining does,
/// and keeps what it finally takes.
/// Why:
///   The review of this notice names the sequence:
/// a reader that stops draining must not hold the commit's own thread,
/// which is the
///  only thread that reads the answer,
/// withdraws a request at its ceiling and keeps the scratch directory fresh.
struct StalledStream {
    /// Says that a line reached the stream and is now held there.
    held: mpsc::Sender<()>,
    /// Where the stream waits until the control lets the line go.
    release: mpsc::Receiver<()>,
    /// What the stream took once it was let go.
    kept: Arc<Mutex<Vec<u8>>>,
}

impl Write for StalledStream {
    /// Announce that the line arrived,
    /// wait to be released,
    /// and keep it.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let _told = self.held.send(());
        let _let_go = self.release.recv();
        self.kept.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What the stream kept once it took it,
/// within the patience of the controls.
fn kept_within(kept: &Arc<Mutex<Vec<u8>>>) -> String {
    let began: Instant = Instant::now();
    loop {
        let taken: String = String::from_utf8_lossy(kept.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_slice()).into_owned();
        if !taken.is_empty() || began.elapsed() >= PATIENCE {
            return taken;
        }
        std::thread::sleep(FAKE_BEAT);
    }
}

/// What:
///  A commit whose notice stream stopped taking lines still lands,
/// and the line it said reaches that stream when the reader takes it.
/// Why:
///   The standing notice is written between two reads of the service's answer,
/// so a reader that stopped draining it must cost
///  the commit nothing:
/// the answer is still read,
/// the command ends with it,
/// and the line goes when the reader resumes.
#[test]
fn a_commit_whose_notice_stream_stopped_taking_lines_still_lands() {
    let name: String = address("stalled-notices");
    let service: UnixListener = listen(name.as_str());
    let kept: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let (held_sender, held) = mpsc::channel::<()>();
    let (release_sender, release) = mpsc::channel::<()>();
    let stalled: StalledStream = StalledStream { held: held_sender, release, kept: Arc::clone(&kept) };
    // The stream the command says its lines to, which is written by a thread of its own: this is the fix the control holds.
    let mut stream: git_policy_cli::foundation::notice_stream_that_does_not_hold_its_writer::NoticeStreamThatDoesNotHoldItsWriter =
        git_policy_cli::foundation::notice_stream_that_does_not_hold_its_writer::NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(stalled));
    let (answered_sender, answered) = mpsc::channel::<Result<Submitted, ClientFailure>>();
    let client_config: ClientConfig = config(name.as_str());
    std::thread::spawn(move || {
        let end: Result<Submitted, ClientFailure> = submit(&client_config, &request(), &mut stream);
        let _sent = answered_sender.send(end);
    });
    let (service_stream, _) = service.accept().expect("the client connects");
    greet(&service_stream);
    read_request(&service_stream);
    // The service beats past the notice threshold while its landing runs, so the client says where the commit stands into
    //  a stream that takes one line and stops.
    let while_waiting: Instant = Instant::now();
    while while_waiting.elapsed() < QUICK_NOTICE * 4 {
        (&service_stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    assert!(held.recv_timeout(PATIENCE).is_ok(), "the standing notice reached the stream, which then held it");
    let commit: String = "a".repeat(40);
    (&service_stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    // The answer is read while the line is still held: a stream that stopped taking lines never holds the commit.
    let end = answered.recv_timeout(ANSWERED_WHILE_THE_STREAM_IS_HELD).expect("the commit answered while its notice stream was held");
    assert_eq!(end, Ok(Submitted::Landed { commit, already: false }));
    drop(release_sender);
    let said: String = kept_within(&kept);
    assert!(said.contains(UNPLACED_SAID), "the line the client said reached the stream: {said:?}");
}

/// What:
///  A place named by a service whose connection then ended is not printed as the place of the wait that follows.
/// Why:
///   The place belongs to the batch of the service that named it,
/// and the request is sent again to a service that decides its own
///  batch;
/// printing the old place turns an earlier service's batch into a false statement about this wait.
#[test]
fn a_place_from_a_service_that_went_away_is_not_printed_as_the_current_one() {
    let name: String = address("stale-place");
    let service: UnixListener = listen(name.as_str());
    let waiting = client(config(name.as_str()));
    // The first service names a place well before the wait matters, and its connection ends without an answer.
    let (first, _) = service.accept().expect("the client connects");
    greet(&first);
    read_request(&first);
    (&first).write_all(&line(&ServiceMessage::Position { number: 2, of: 2 })).expect("the place of its batch");
    drop(first);
    // The service that takes the request again names no place, and answers after the wait has mattered.
    let (second, _) = service.accept().expect("the client sends its request again");
    greet(&second);
    read_request(&second);
    let waited: Instant = Instant::now();
    while waited.elapsed() < QUICK_NOTICE * 3 {
        (&second).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    let commit: String = "b".repeat(40);
    (&second).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert!(notices.contains(UNPLACED_SAID), "the second service named no place, and that is what the person is told: {notices:?}");
    assert!(!notices.contains("earlier commit") && !notices.contains("is next"), "the place of the service that went away is not said of this wait: {notices:?}");
}

/// A commit that lands at once prints nothing at all,
/// so the telling costs a person who commits quickly no line they did not have before.
#[test]
fn a_commit_that_lands_at_once_prints_nothing_new() {
    let name: String = address("at-once");
    let service: UnixListener = listen(name.as_str());
    let waiting = client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let commit: String = "e".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert_eq!(notices, "", "a commit the service lands at once prints nothing");
}

/// A place the service names before the commit has waited at all is kept and not printed,
/// so a commit that lands at once says nothing new however its place was said.
#[test]
fn a_place_the_service_names_before_the_wait_matters_prints_nothing() {
    let name: String = address("placed-at-once");
    let service: UnixListener = listen(name.as_str());
    let waiting = traced_client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let place: (usize, usize) = (2, 4);
    (&stream).write_all(&line(&ServiceMessage::Position { number: place.0, of: place.1 })).expect("the place");
    let commit: String = "c".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices, trace) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    assert_eq!(notices, "", "a commit the service lands at once prints nothing");
    assert_eq!(trace.positions, vec![place.0], "the record keeps the place the service named, printed or not");
}

/// What:
///  A commit the service places after it has waited says how many commits go before it,
/// and the record holds the place the service
///  named.
/// Why:
///   `positions` stayed empty in every record of the incident,
/// so the place a service names is what this control proves reaches both
///  the person and the timing record.
#[test]
fn a_commit_the_service_places_after_a_wait_says_how_many_go_before_it() {
    let name: String = address("placed");
    let service: UnixListener = listen(name.as_str());
    let waiting = traced_client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let waited: Instant = Instant::now();
    let place: (usize, usize) = (2, 4);
    while waited.elapsed() < QUICK_NOTICE * 3 {
        (&stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    (&stream).write_all(&line(&ServiceMessage::Position { number: place.0, of: place.1 })).expect("the place");
    // A landing is work that follows the place the service names, never the same instant as it: the two share no write.
    std::thread::sleep(FAKE_BEAT * 2);
    let commit: String = "f".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, notices, trace) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{notices}");
    let placed: String = position_notice(place.0, place.1);
    assert!(notices.contains(placed.as_str()), "the place the service named is said: {notices:?}");
    assert_eq!(notices.matches(placed.as_str()).count(), 1, "said once: {notices:?}");
    assert!(notices.contains(UNPLACED_SAID) && notices.find(UNPLACED_SAID) < notices.find(placed.as_str()), "the service not having said where it stands is said first: {notices:?}");
    assert_eq!(trace.positions, vec![place.0], "the record holds the place the service named");
}

/// A service that stops sending beats ends the wait:
/// the client says how long it has been quiet,
/// says that the same commit is being sent again,
/// asks again with the same request,
/// and meets a
/// service that accepts and greets nothing,
/// which is the same silence;
/// it ends with the plain failure that says whether the
/// commit may have been made and what to do.
/// Why the notice names the seconds:
///  a service that is busy and one that has stopped look the same from the client,
///  and the person decides whether to wait or to run something else,
///  so the older line that claimed a stop and named no duration sent them to a log about a service that was working
///  (the record of 2026-11-11 in
///  `doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
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
    assert!(notices.contains("1 second"), "the notice names the seconds the service was quiet: {notices:?}");
    assert!(notices.contains("nothing for you to do"), "the notice says that nothing is asked of the reader: {notices:?}");
    assert!(notices.contains(silent_notice(QUICK_SILENCE).as_str()), "{notices:?}");
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

/// The fake manager takes a request to forget a unit's failed state and counts it as a start's own.
fn fake_reset(_bus: &Path, _unit: &str) -> bool {
    return true;
}

/// The fake manager takes a request to read the units again,
/// and nothing comes of it:
/// these controls are about a service that never starts.
fn fake_reload(_bus: &Path) -> bool {
    return true;
}

/// How many times the fake manager was asked to restart the unit.
static RESTART_REQUESTS: AtomicUsize = AtomicUsize::new(0);

/// The fake manager takes a request to restart the unit and counts it.
/// Its answer never says `active` here,
/// so no control of this file reaches a restart by being wedged.
fn fake_restart(_bus: &Path, _unit: &str) -> bool {
    RESTART_REQUESTS.fetch_add(1, Ordering::SeqCst);
    return true;
}

/// A watch on the fake manager that asks quickly.
fn watched(name: &str) -> ClientConfig {
    let watch: ManagerWatch = ManagerWatch {
        bus: PathBuf::from("/nowhere/bus"),
        unit: String::from("cli-git@fake.service"),
        socket_unit: String::from("cli-git@fake.socket"),
        every: Duration::from_millis(50),
        state: fake_manager,
        start: fake_start,
        reset: fake_reset,
        restart: fake_restart,
        reload: fake_reload,
    };
    return ClientConfig { start_wait: OLD_START_WAIT, heartbeat_silence: PATIENCE, start_watch: Some(watch), ..config(name) };
}

/// A service that never starts is waited for until the manager says the start failed,
/// past the old start wait,
/// and the wait ends with the plain failure once the manager says it twice.
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

/// A service that starts a long while after the client asks,
/// against the old start wait,
/// is waited for and answers.
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
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
}

/// A unit the manager reports `inactive`,
/// while the socket refuses the client's connection and nothing starts
/// it,
/// is started by the client's request to the manager at the first such answer,
/// without waiting the start wait;
/// the client
/// goes on waiting and the service then answers.
/// A unit the manager reports `activating` is never asked to start.
#[test]
fn an_inactive_unit_is_asked_to_start_at_once_and_an_activating_one_is_not() {
    let name: String = address("inactive-unit");
    START_REQUESTS.store(0, Ordering::SeqCst);
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    MANAGER_SAYS_INACTIVE.store(false, Ordering::SeqCst);
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
    // The manager started the service: it listens, greets and answers.
    let service: UnixListener = listen(name.as_str());
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let commit: String = "e".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
    MANAGER_SAYS_INACTIVE.store(false, Ordering::SeqCst);
}

/// A manager that cannot be asked leaves the wait to the start wait,
/// as before.
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

/// A command that waits on the service keeps its scratch directory from aging out:
/// the sweep that removes entries older than
/// `SCRATCH_STALE_AFTER` finds a waiter's directory fresh however long the wait lasts,
/// and removes the same directory of a
/// command that is no longer waiting (the positive control).
#[test]
fn a_waiting_command_keeps_its_scratch_directory_from_being_swept() {
    let name: String = address("keeps-scratch");
    let fixture: super::support::Fixture = super::support::fixture("heartbeat-keeps-scratch");
    let root: PathBuf = fixture.root.join("scratch-root");
    let waiting_dir: PathBuf = root.join("waiting");
    let left_behind: PathBuf = root.join("left-behind");
    std::fs::create_dir_all(&waiting_dir).expect("waiting directory");
    std::fs::create_dir_all(&left_behind).expect("left-behind directory");
    // Both directories are two stale ages old, as a wait of more than a day would make them.
    let long_ago: std::time::SystemTime = std::time::SystemTime::now() - SCRATCH_STALE_AFTER * 2;
    for directory in [&waiting_dir, &left_behind] {
        std::fs::File::open(directory).expect("open").set_modified(long_ago).expect("age");
    }
    let service: UnixListener = listen(name.as_str());
    let mut fresh_config: ClientConfig = config(name.as_str());
    fresh_config.keep_fresh = Some(FreshScratch { path: waiting_dir.clone(), every: FAKE_BEAT });
    // This control does its own work between two beats (the aging of two directories, the sweep), so the silence bound that
    //  ends its wait is this control's patience and not the `QUICK_SILENCE` the other controls give a service they watch: a beat
    //  cadence is not what it checks, and the sweep once took longer than `QUICK_SILENCE` under host load, which made the client
    //  call the service stopped and close its connection, and the answer then failed with `Broken pipe` (2026-11-08).
    fresh_config.heartbeat_silence = PATIENCE;
    let waiting = client(fresh_config);
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    let began: Instant = Instant::now();
    while began.elapsed() < FAKE_BEAT * 8 {
        (&stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    // The sweep runs while the command still waits.
    let removed: usize = sweep_stale(root.as_path(), std::time::SystemTime::now(), SCRATCH_STALE_AFTER, git_policy_cli::scratch_entry_removal::SCRATCH_REMOVAL_CEILING);
    let commit: String = "f".repeat(40);
    if let Err(error) = (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })) {
        // The client is the one that closed the connection, so what it ended with says why the answer had nowhere to go.
        let (answered, notices) = waiting.join().expect("client thread");
        panic!("the answer could not be written ({error}); the client ended with {answered:?}: {notices}");
    }
    let (answered, _notices) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
    assert!(waiting_dir.exists(), "the waiting command's directory was swept");
    assert!(!left_behind.exists(), "the positive control: a directory nobody keeps fresh is swept");
    assert_eq!(removed, 1);
    super::commit::remove_settled(&fixture);
}

/// How long a control waits for a client that must end at its ceiling before it calls it hung.
const CEILING_PATIENCE: Duration = Duration::from_secs(15);

/// The ceilings the controls give the client:
/// short,
/// so a control ends in a moment.
const SHORT_CEILING: Duration = Duration::from_millis(500);

/// Runs the client on a thread and returns what it ended with and printed,
/// or nothing when it was still waiting after
///  `CEILING_PATIENCE`.
fn client_within_patience(config: ClientConfig) -> Option<(Result<Submitted, ClientFailure>, String)> {
    let (finished, ended) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let answered = submit(&config, &request(), &mut notices);
        let _sent = finished.send((answered, String::from_utf8(notices).expect("UTF-8")));
    });
    return ended.recv_timeout(CEILING_PATIENCE).ok();
}

/// A fake service for the ceiling controls:
/// the first connection is greeted,
/// reads the request and beats for ever;
/// the second is
///  greeted and reads the line that asks for the withdrawal,
/// which it returns with the answer `reply` gives (nothing:
/// no answer).
fn serve_a_wait_and_a_withdrawal(service: UnixListener, reply: Option<ServiceMessage>) -> std::thread::JoinHandle<Option<ClientMessage>> {
    return std::thread::spawn(move || {
        let (first, _) = service.accept().expect("the first connection");
        greet(&first);
        read_request(&first);
        let beating = std::thread::spawn(move || {
            while (&first).write_all(&line(&ServiceMessage::Heartbeat)).is_ok() {
                std::thread::sleep(FAKE_BEAT);
            }
        });
        let (second, _) = service.accept().expect("the withdrawal's connection");
        greet(&second);
        let mut asked: String = String::new();
        BufReader::new(&second).read_line(&mut asked).expect("the withdrawal");
        let message: Option<ClientMessage> = decode_client(asked.trim_end_matches('\n')).ok();
        if let Some(answer) = reply {
            (&second).write_all(&line(&answer)).expect("answer the withdrawal");
        } else {
            // The service never answers: the connection stays open until the control ends.
            std::thread::sleep(CEILING_PATIENCE);
        }
        drop(beating);
        return message;
    });
}

/// A wait that passes the answer ceiling,
/// with the service still sending heartbeats,
/// is withdrawn:
/// the client asks the service to
/// withdraw the request of its identity on a new connection,
/// and when the service says it did,
/// the command ends with the error
/// that says the commit will not land later and what to run.
#[test]
fn a_wait_past_the_answer_ceiling_withdraws_the_request_and_ends_with_the_error() {
    let name: String = address("ceiling-withdraws");
    let service: UnixListener = listen(name.as_str());
    let fake = serve_a_wait_and_a_withdrawal(service, Some(ServiceMessage::Withdrawn { id: String::from("heartbeat-control") }));
    let config: ClientConfig = ClientConfig { answer_ceiling: SHORT_CEILING, ..config(name.as_str()) };
    let (answered, _notices) = client_within_patience(config).expect("the client ended at its ceiling");
    assert!(matches!(answered, Err(ClientFailure::StillWithService { .. })), "{answered:?}");
    let asked: Option<ClientMessage> = fake.join().expect("fake service");
    assert_eq!(asked, Some(ClientMessage::Withdraw { id: String::from("heartbeat-control") }));
}

/// The race:
/// the commit landed just as the bound passed.
/// The service answers the withdrawal with the landed commit,
/// and the
/// command reports success,
/// not the error.
#[test]
fn a_commit_that_landed_as_the_bound_passed_is_reported_as_landed() {
    let name: String = address("ceiling-race-landed");
    let service: UnixListener = listen(name.as_str());
    let commit: String = "9".repeat(40);
    let landed: ServiceMessage = ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: true, landing: None };
    let fake = serve_a_wait_and_a_withdrawal(service, Some(landed));
    let config: ClientConfig = ClientConfig { answer_ceiling: SHORT_CEILING, ..config(name.as_str()) };
    let (answered, _notices) = client_within_patience(config).expect("the client ended at its ceiling");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: true }));
    fake.join().expect("fake service");
}

/// A service that does not answer the withdrawal ends the wait at the withdrawal's own ceiling,
/// with the error that says it is not
/// known whether the commit was made.
#[test]
fn a_withdrawal_the_service_does_not_answer_ends_at_its_own_ceiling() {
    let name: String = address("ceiling-unconfirmed");
    let service: UnixListener = listen(name.as_str());
    let _fake = serve_a_wait_and_a_withdrawal(service, None);
    let config: ClientConfig = ClientConfig { answer_ceiling: SHORT_CEILING, withdraw_ceiling: SHORT_CEILING, heartbeat_silence: PATIENCE, ..config(name.as_str()) };
    let (answered, _notices) = client_within_patience(config).expect("the client ended at its ceilings");
    assert_eq!(answered, Err(ClientFailure::WithdrawalUnconfirmed { blocker: None }));
}

/// A start the manager reports as under way (`activating`) is waited for only until the start ceiling,
/// then ends with the plain
/// failure that names the log.
#[test]
fn a_start_that_never_finishes_ends_at_the_start_ceiling() {
    let name: String = address("start-ceiling");
    MANAGER_SAYS_FAILED.store(false, Ordering::SeqCst);
    MANAGER_SAYS_INACTIVE.store(false, Ordering::SeqCst);
    let _service: UnixListener = listen(name.as_str());
    let config: ClientConfig = ClientConfig { start_ceiling: SHORT_CEILING, ..watched(name.as_str()) };
    let (answered, _notices) = client_within_patience(config).expect("the client ended at the start ceiling");
    assert_eq!(answered, Err(ClientFailure::DidNotStart));
}

/// A fake service for the late-notice control:
/// the first connection is greeted and beats;
/// the second,
/// the withdrawal's,
/// is greeted,
/// reads the line that asks for the withdrawal,
/// tells the client its request is inside the write,
/// and beats without ever answering.
fn serve_a_withdrawal_that_says_the_request_is_inside_the_write(service: UnixListener) -> std::thread::JoinHandle<()> {
    return std::thread::spawn(move || {
        let (first, _) = service.accept().expect("the first connection");
        greet(&first);
        read_request(&first);
        let beating = std::thread::spawn(move || {
            while (&first).write_all(&line(&ServiceMessage::Heartbeat)).is_ok() {
                std::thread::sleep(FAKE_BEAT);
            }
        });
        let (second, _) = service.accept().expect("the withdrawal's connection");
        greet(&second);
        let mut asked: String = String::new();
        BufReader::new(&second).read_line(&mut asked).expect("the withdrawal");
        (&second).write_all(&line(&ServiceMessage::InsideWrite { text: String::from("monochromatic-git-policy-cli: this commit is inside a write of the commit service") })).expect("the notice");
        // The connection stays open and beats until the client leaves.
        while (&second).write_all(&line(&ServiceMessage::Heartbeat)).is_ok() {
            std::thread::sleep(FAKE_BEAT);
        }
        drop(beating);
    });
}

/// The notice that the request is inside the write reaches the client only on the connection it opened to ask for its withdrawal,
/// because the write began within a minute of the answer ceiling (sol-1:
/// the seven-second path was missed):
/// the client does what a
/// client that heard it earlier does,
/// and ends at its answer ceiling plus the extra wait,
/// not at the withdrawal's ceiling.
#[test]
fn a_notice_that_arrives_on_the_withdrawal_connection_ends_the_wait_at_the_answer_ceiling_plus_the_extra_wait() {
    let name: String = address("late-inside-notice");
    let service: UnixListener = listen(name.as_str());
    let _fake = serve_a_withdrawal_that_says_the_request_is_inside_the_write(service);
    let ceiling: Duration = Duration::from_secs(1);
    let extra_wait: Duration = Duration::from_secs(1);
    let config: ClientConfig = ClientConfig { answer_ceiling: ceiling, extra_wait, withdraw_ceiling: Duration::from_secs(120), heartbeat_silence: PATIENCE, ..config(name.as_str()) };
    let began: Instant = Instant::now();
    let (answered, _notices) = client_within_patience(config).expect("the client ended at its answer ceiling plus the extra wait");
    let waited: Duration = began.elapsed();
    let Err(ClientFailure::WithdrawalUnconfirmed { blocker }) = answered else { panic!("{answered:?}") };
    assert!(blocker.is_some_and(|text| { return text.contains("inside a write"); }));
    assert!(waited >= ceiling + extra_wait && waited < ceiling + extra_wait + Duration::from_secs(6), "{waited:?}");
}

/// Runs the client on a thread and returns what it ended with,
/// what it printed and the trace it kept.
fn traced_client(config: ClientConfig) -> std::thread::JoinHandle<(Result<Submitted, ClientFailure>, String, SubmitTrace)> {
    return std::thread::spawn(move || {
        let mut notices: Vec<u8> = Vec::new();
        let mut trace: SubmitTrace = SubmitTrace::default();
        let answered = submit_traced(&config, &request(), &mut notices, &mut trace);
        return (answered, String::from_utf8(notices).expect("UTF-8"), trace);
    });
}

/// A fake service that sends `notices` in order,
/// then lands the commit,
/// and the trace the client kept.
fn trace_after(label: &str, notices: Vec<ServiceMessage>) -> git_policy_cli::commit_timing::SubmitTrace {
    let name: String = address(label);
    let service: UnixListener = listen(name.as_str());
    let waiting = traced_client(config(name.as_str()));
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    for notice in &notices {
        (&stream).write_all(&line(notice)).expect("a notice");
    }
    let commit: String = "c".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, _notices, trace) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
    return trace;
}

/// The notice that names a holder proven alive.
fn proven_notice() -> ServiceMessage {
    let end: git_policy_cli::index_lock_evidence::WaitEnd<'_> =
        git_policy_cli::index_lock_evidence::WaitEnd { within: git_policy_cli::wait_ceilings::LANDING_LOCK_CEILING, ending: git_policy_cli::service_index_attempts::COMMIT_WITHDRAWN };
    return ServiceMessage::Waiting { text: git_policy_cli::index_lock_evidence::proven_holder_line(4242, true, Some("git commit"), Path::new("/r/.git/index.lock"), end) };
}

/// What the service says when its landing is inside a write,
/// which says nothing of any holder.
fn inside_write_notice() -> ServiceMessage {
    return ServiceMessage::InsideWrite { text: String::from("monochromatic-git-policy-cli: this commit is inside a write of the commit service, `git update-ref --stdin`.\n") };
}

/// What:
///  A fake service that says `at_once`,
/// beats for `beating` while the command waits,
/// says `after` and then lands the commit,
/// and what the client printed and kept.
/// Why:
///   A control about what a wait is worth saying needs the wait to last past the moment it becomes worth saying,
/// and the beats of a fake service are what keeps a client waiting:
/// no clock of the wrapper's own ends a wait that is still hearing from the service.
fn said_while_beating(name: &str, config: ClientConfig, at_once: Vec<ServiceMessage>, beating: Duration, after: Vec<ServiceMessage>) -> (String, SubmitTrace) {
    let service: UnixListener = listen(name);
    let waiting = traced_client(config);
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    for notice in &at_once {
        (&stream).write_all(&line(notice)).expect("a notice");
    }
    let began: Instant = Instant::now();
    while began.elapsed() < beating {
        (&stream).write_all(&line(&ServiceMessage::Heartbeat)).expect("beat");
        std::thread::sleep(FAKE_BEAT);
    }
    for notice in &after {
        (&stream).write_all(&line(notice)).expect("a notice after the wait");
    }
    // A write that is still stuck says its newer words while it is stuck, and the landing comes only once the write returns:
    //  words sent from a timer and the answer to the request never share one write, as the words sent when the write begins do.
    std::thread::sleep(FAKE_BEAT * 2);
    let commit: String = "c".repeat(40);
    (&stream).write_all(&line(&ServiceMessage::Landed { id: String::from("heartbeat-control"), commit: commit.clone(), already: false, landing: None })).expect("answer");
    let (answered, said, trace) = waiting.join().expect("client thread");
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }));
    return (said, trace);
}

/// What:
///  A commit whose own request the service says is inside its branch write says nothing of that write while the
///  person's wait has not mattered.
/// Why:
///   The service says it when the write begins,
/// because a client whose ceiling falls inside the write would miss it (sol-1),
///  and a write that takes what writes take is over in milliseconds:
/// a line about it on a commit that lands at once is noise,
///  and on 2026-11-11 it made controls that watch a healthy commit's standard error fail on a loaded disk.
#[test]
fn a_commit_inside_a_write_says_nothing_of_it_before_its_wait_has_mattered() {
    let name: String = address("inside-write-not-yet");
    let config: ClientConfig = ClientConfig { waiting_notice_after: PATIENCE, ..config(name.as_str()) };
    let (said, trace) = said_while_beating(name.as_str(), config, vec![inside_write_notice()], FAKE_BEAT * 6, Vec::new());
    assert_eq!(trace.waiting_notices, 1, "the client was told, and said nothing of it");
    assert_eq!(said, "", "a commit that lands at once says nothing: {said:?}");
}

/// A commit whose wait on that write has mattered says it once,
/// in the words the service sent.
#[test]
fn a_commit_inside_a_write_says_it_once_its_wait_has_mattered() {
    let name: String = address("inside-write-told");
    let (said, trace) = said_while_beating(name.as_str(), config(name.as_str()), vec![inside_write_notice()], QUICK_NOTICE * 2, Vec::new());
    assert_eq!(trace.waiting_notices, 1, "{said}");
    assert_eq!(said.matches("this commit is inside a write of the commit service").count(), 1, "{said}");
}

/// A write that has stayed stuck a new minute is said at once,
/// in the words that came with the minute:
///  they differ from the ones already said,
/// and the wait has already passed the threshold that held the first line,
/// so they are said in the loop pass that reads them.
#[test]
fn the_new_words_of_a_write_that_stayed_stuck_are_said_at_once() {
    let name: String = address("inside-write-stuck");
    let stuck: ServiceMessage = ServiceMessage::InsideWrite {
        text: String::from(
            "monochromatic-git-policy-cli: the commit service has been inside a write for 1 minutes and has not heard back: \
             `git update-ref --stdin`. The service never ends a write in the middle.\n",
        ),
    };
    let (said, _trace) = said_while_beating(name.as_str(), config(name.as_str()), vec![inside_write_notice()], QUICK_NOTICE * 3, vec![stuck]);
    let later: usize = said.find("has been inside a write for 1 minutes").expect("the newer words are said");
    // The first words are said once the wait passes the threshold; a host that stalls past them leaves them out, and the
    // newer words are what this control is about.
    if let Some(first) = said.find("this commit is inside a write") {
        assert!(first < later, "the newer words come after the older ones: {said}");
    }
    assert_eq!(said.matches("has been inside a write for 1 minutes").count(), 1, "{said}");
}

/// What:
///  A fake service that says nothing while the wait matters,
/// then sends `queued` in one write,
/// and what the client printed.
/// Why:
///   One write is what a service that lands a commit at once leaves on the connection:
/// both lines are here when the client reads either of them,
/// as they are for a client the machine held up after its request was written.
fn said_after_a_quiet_wait_then_the_queued_lines(name: &str, quiet: Duration, queued: Vec<ServiceMessage>) -> String {
    let service: UnixListener = listen(name);
    // The quiet is longer than the notice's wait and shorter than the silence bound that would end the wait.
    let config: ClientConfig = ClientConfig { heartbeat_silence: PATIENCE, ..config(name) };
    let waiting = client(config);
    let (stream, _) = service.accept().expect("the client connects");
    greet(&stream);
    read_request(&stream);
    std::thread::sleep(quiet);
    let mut burst: Vec<u8> = Vec::new();
    for message in &queued {
        burst.extend_from_slice(&line(message));
    }
    (&stream).write_all(&burst).expect("the queued lines reach the client");
    let (answered, said) = waiting.join().expect("client thread");
    let commit: String = "c".repeat(40);
    assert_eq!(answered, Ok(Submitted::Landed { commit, already: false }), "{said}");
    return said;
}

/// What:
///  The landing of the commit already queued behind the words about a write is read before they are said,
/// so the command that landed says nothing of that write.
/// Why:
///   The words wait for the wait that matters,
/// but that wait can be over before the client reads them:
/// a client the machine held stopped read its
///  answer only after it had spoken,
/// and a commit that landed printed "this commit is inside a write of the commit service".
#[test]
fn a_landing_already_queued_behind_the_words_of_a_write_is_not_called_inside_a_write() {
    let name: String = address("inside-write-already-landed");
    let commit: String = "c".repeat(40);
    let said: String = said_after_a_quiet_wait_then_the_queued_lines(
        name.as_str(),
        QUICK_NOTICE * 2,
        vec![inside_write_notice(), ServiceMessage::Landed { id: String::from("heartbeat-control"), commit, already: false, landing: None }],
    );
    assert!(!said.contains("this commit is inside a write of the commit service"), "a commit the service had landed printed the words of its write: {said:?}");
}

/// What:
///  The landing already queued behind the place the service named is read before the place is printed,
/// so a commit that landed in the meantime says nothing of where it stood.
/// Why:
///   A place belongs to a wait that is still on,
/// and a client that was held up reads the place and the landing together:
/// the place of a commit that is already on the branch is news of a wait that is over.
#[test]
fn a_place_the_client_reads_after_the_landing_is_already_here_is_not_printed() {
    let name: String = address("place-already-landed");
    let commit: String = "c".repeat(40);
    let said: String = said_after_a_quiet_wait_then_the_queued_lines(
        name.as_str(),
        QUICK_NOTICE * 2,
        vec![ServiceMessage::Position { number: 1, of: 1 }, ServiceMessage::Landed { id: String::from("heartbeat-control"), commit, already: false, landing: None }],
    );
    assert!(!said.contains(position_notice(1, 1).as_str()), "a commit the service had landed printed its place: {said:?}");
}

/// A wait for a holder proven alive is recorded as one,
/// whatever the service says after it:
/// the landing's own notice that it is inside a write comes the moment the holder lets go,
/// and says nothing of the holder.
#[test]
fn a_holder_proven_alive_stays_proven_when_the_landing_says_it_is_inside_a_write() {
    let trace = trace_after("kind-after-inside-write", vec![proven_notice(), inside_write_notice()]);
    assert_eq!(trace.holder_kind, Some("proven-alive"));
    assert_eq!(trace.waiting_notices, 2, "both notices are counted");
}

/// A later notice that names no proof (the way out of a wait that reached its ceiling) does not undo the proof either.
#[test]
fn a_holder_proven_alive_stays_proven_when_a_later_notice_names_no_proof() {
    let way_out: ServiceMessage = ServiceMessage::Waiting { text: String::from("monochromatic-git-policy-cli: to end it, stop PID 4242 or wait for it to finish.\n") };
    let trace = trace_after("kind-after-way-out", vec![proven_notice(), way_out]);
    assert_eq!(trace.holder_kind, Some("proven-alive"));
}

/// An unproven wait that a later notice proves is recorded as proven:
/// the proof counts when it comes.
#[test]
fn an_unproven_wait_that_a_later_notice_proves_is_recorded_as_proven() {
    let end: git_policy_cli::index_lock_evidence::WaitEnd<'_> =
        git_policy_cli::index_lock_evidence::WaitEnd { within: git_policy_cli::wait_ceilings::LANDING_LOCK_CEILING, ending: git_policy_cli::service_index_attempts::COMMIT_WITHDRAWN };
    let unproven: ServiceMessage = ServiceMessage::Waiting { text: git_policy_cli::index_lock_evidence::unproven_wait_line(Path::new("/r/.git/index.lock"), None, end) };
    let trace = trace_after("kind-unproven-then-proven", vec![unproven, proven_notice()]);
    assert_eq!(trace.holder_kind, Some("proven-alive"));
}

/// A notice that the landing is inside a write is not about a holder:
/// alone it leaves the kind of holder unrecorded.
#[test]
fn a_notice_that_the_landing_is_inside_a_write_records_no_holder_kind() {
    let trace = trace_after("kind-inside-write-alone", vec![inside_write_notice()]);
    assert_eq!(trace.holder_kind, None);
}
