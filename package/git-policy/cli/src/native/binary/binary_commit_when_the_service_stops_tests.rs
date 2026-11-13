//! What:
//!  What a commit does when the commit service of its worktree stops reading,
//!  stops answering,
//!  or stops altogether:
//!  the bound each step of the exchange ends at,
//! the words a person reads,
//! and that nothing is left half done.
//! Why:
//!  Two audits of release 0.6.1-alpha.12 found a service that is running and has stopped reading hold a commit in the
//! write of its request with no bound at all,
//! and a stopped service cost a commit ten seconds and words that named no step to take.
//! The owner's rule is that nothing waits for ever,
//! and the decision of 2026-11-12 is that such a commit fails
//!  with a clear message (`doc/handover/cli-git-rust-implementation.md`,
//! item 1.10).
//! Every control here is bounded,
//! so a bound that is lost fails
//!  it instead of holding the run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = startFixtureService(fixture); process.kill(service.pid, 'SIGSTOP');
//! const ran = await observe(wrappedCommit(fixture, repo)); expect(ran.code).toBe(1);
//! ```

use super::commit::{COMMIT_BOUND_SECONDS, remove_settled};
use super::support::{
    Fixture, FixtureService, Observed, bounded_for, fixture, git, hermetic, observe, repository, service_runtime, start_fixture_service, wrapper_first_path,
};
use git_policy_cli::service_instance::socket_for;
use git_policy_cli::service_protocol::{HEARTBEAT_SILENCE, PROTOCOL_VERSION, RELEASE, ServiceMessage, encode_service};
use git_policy_cli::service_client::START_WAIT;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How long a control waits for something that must happen,
/// so a broken bound fails the control instead of holding the run.
const PATIENCE: Duration = Duration::from_secs(120);

/// How many files a control changes when it needs a request larger than one connection's buffers,
/// and how long each name is.
///  Measured on this host on 2026-11-12:
/// the send buffer of a local socket holds 212,992 bytes and one socket's two buffers took 233,152 bytes with nothing
///  reading,
/// and 1,600 names of this length carry a request of about 390 kB,
/// which no service that stopped reading can take.
const MANY_FILES: usize = 1_600;
const LONG_NAME: usize = 180;

/// What the stand-in service does after it has greeted a connection.
#[derive(Clone, Copy, Debug)]
enum Duty {
    /// Greets and reads nothing,
    /// so the request write has nowhere to go.
    ReadNothing,
    /// Reads the whole request and answers nothing.
    ReadsAndSilent,
    /// Reads the whole request and refuses it,
    /// so a control has a real request line and the commit ends at once.
    ReadsAndRefuses,
    /// Answers nothing at all,
    /// not even a greeting.
    SilentThroughout,
}

/// What a control knows about the stand-in service.
struct StandIn {
    /// The directory of the wrapper's own runtime,
    /// kept alive for the control.
    runtime: PathBuf,
    /// How many connections the stand-in has greeted.
    greeted: Arc<AtomicUsize>,
    /// How many connections it has taken at all,
    /// which is what a control knows when the stand-in greets nothing.
    accepted: Arc<AtomicUsize>,
    /// Whether a whole request line has been read.
    read_all: Arc<AtomicBool>,
    /// The whole request line the stand-in read,
    /// with its framing newline,
    /// for a control that sends it to a real service.
    recorded: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    /// Told to the thread that serves at the end of the control.
    stop: Arc<AtomicBool>,
    /// The fixture's services,
    /// which keep the runtime directory the stand-in listened in alive.
    _service: FixtureService,
}

impl Drop for StandIn {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// What:
///  A stand-in for the commit service at the address the wrapper looks for,
///  with no service manager and no real service,
///  so a control decides exactly what the service does.
/// Why:
///   The one state a real service cannot be put in on demand is "greeted you and then stopped reading",
/// and that is the state the audit found a commit waiting in for ever.
/// The stand-in speaks the service's own greeting,
/// so the wrapper treats it as
///  the service of this worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = standIn(fixture, repo, 'read-nothing'); // greets, reads nothing
/// ```
fn stand_in(fixture: &Fixture, repo: &Path, duty: Duty) -> StandIn {
    let service: FixtureService = start_fixture_service(fixture);
    let git_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the worktree's Git directory");
    let socket: PathBuf = socket_for(service.runtime.as_path(), git_dir.as_path());
    let listener: UnixListener = UnixListener::bind(socket.as_path()).expect("bind the stand-in");
    listener.set_nonblocking(true).expect("a listener that does not block");
    let greeted: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let accepted: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    let read_all: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let recorded: Arc<std::sync::Mutex<Option<Vec<u8>>>> = Arc::new(std::sync::Mutex::new(None));
    let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let serving_greeted: Arc<AtomicUsize> = Arc::clone(&greeted);
    let serving_accepted: Arc<AtomicUsize> = Arc::clone(&accepted);
    let serving_read: Arc<AtomicBool> = Arc::clone(&read_all);
    let serving_recorded: Arc<std::sync::Mutex<Option<Vec<u8>>>> = Arc::clone(&recorded);
    let serving_stop: Arc<AtomicBool> = Arc::clone(&stop);
    std::thread::spawn(move || {
        let greeting: String = format!("{}\n", encode_service(&ServiceMessage::Welcome { version: PROTOCOL_VERSION, release: Some(RELEASE.to_string()) }));
        while !serving_stop.load(Ordering::SeqCst) {
            let Ok((stream, _address)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            _ = serving_accepted.fetch_add(1, Ordering::SeqCst);
            // Each connection is served by a thread of its own, because a connection the stand-in keeps open is what the
            //  wrapper waits on, and the next connection of the same commit must still be taken.
            let one_greeting: String = greeting.clone();
            let one_greeted: Arc<AtomicUsize> = Arc::clone(&serving_greeted);
            let one_read: Arc<AtomicBool> = Arc::clone(&serving_read);
            let one_recorded: Arc<std::sync::Mutex<Option<Vec<u8>>>> = Arc::clone(&serving_recorded);
            let one_stop: Arc<AtomicBool> = Arc::clone(&serving_stop);
            std::thread::spawn(move || {
                serve_one(stream, one_greeting.as_str(), duty, &one_greeted, &one_read, &one_recorded, &one_stop);
            });
        }
    });
    return StandIn { runtime: service.runtime.clone(), greeted, accepted, read_all, recorded, stop, _service: service };
}

/// What the stand-in does for one connection,
/// as the duty of the control says,
/// and it keeps the connection open until the control ends:
/// a connection the wrapper reads is a connection that must answer nothing rather than close.
fn serve_one(
    stream: UnixStream,
    greeting: &str,
    duty: Duty,
    greeted: &AtomicUsize,
    read_all: &AtomicBool,
    recorded: &std::sync::Mutex<Option<Vec<u8>>>,
    stop: &AtomicBool,
) {
    if !matches!(duty, Duty::SilentThroughout) {
        let _timeout = stream.set_write_timeout(Some(PATIENCE));
        if stream.try_clone().is_ok_and(|mut welcome| { return welcome.write_all(greeting.as_bytes()).is_ok() }) {
            _ = greeted.fetch_add(1, Ordering::SeqCst);
        }
    }
    if matches!(duty, Duty::ReadsAndSilent | Duty::ReadsAndRefuses) {
        let mut reader: BufReader<UnixStream> = BufReader::new(stream.try_clone().expect("a reader of the connection"));
        let _held = reader.get_ref().set_read_timeout(Some(PATIENCE));
        let mut line: Vec<u8> = Vec::new();
        if reader.read_until(b'\n', &mut line).is_ok_and(|read| { return read > 0 }) {
            read_all.store(true, Ordering::SeqCst);
            *recorded.lock().expect("the stand-in's record of the request") = Some(line.clone());
        }
        if matches!(duty, Duty::ReadsAndRefuses) {
            // A refusal the client reads as "the service answered and did not take it", so the commit ends at once and the
            //  request line this control needs is the one it just recorded.
            let refused: String = format!("{}\n", encode_service(&ServiceMessage::Failed { id: String::new(), code: code_of("unavailable") }));
            let _sent = stream.try_clone().is_ok_and(|mut answer| { return answer.write_all(refused.as_bytes()).is_ok() });
        }
    }
    // Nothing is read of a connection whose duty is to read nothing, so the wrapper's write fills its buffers and waits in the
    //  kernel; and nothing is answered on the other kinds, so the wrapper waits for the answer its bound ends.
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The refusal code a stand-in answers with,
/// by the name the protocol spells it.
fn code_of(name: &str) -> git_policy_cli::service_protocol::FailureCode {
    use git_policy_cli::service_protocol::FailureCode;
    return match name {
        "unavailable" => FailureCode::Unavailable,
        other => panic!("a stand-in asks for a refusal code this control does not know: {other}"),
    };
}

/// A command for the fixture's `bin/git` against the stand-in's runtime,
/// which is the address the wrapper looks for,
/// killed after `seconds`.
fn wrapped_against_stand_in(fixture: &Fixture, stand_in: &StandIn, directory: &Path, arguments: &[&str], seconds: &str) -> Command {
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(seconds).arg(fixture.root.join("bin/git"));
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    command.env("XDG_RUNTIME_DIR", stand_in.runtime.as_path());
    command.env(git_policy_cli::wait_ceilings::WAITING_NOTICE_VARIABLE, "1");
    command.current_dir(directory).args(arguments);
    return command;
}

/// The commit of a worktree whose service the control stopped:
/// the fixture's own service, bounded like every commit.
fn wrapped_commit(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = bounded_for(fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(fixture).as_os_str(), COMMIT_BOUND_SECONDS);
    command.current_dir(directory).args(arguments);
    return command;
}

/// A file's name,
/// padded so a request of many of them is larger than one connection's buffers.
fn long_name(index: usize) -> String {
    let base: String = format!("file-{index}");
    let padding: String = "p".repeat(LONG_NAME.saturating_sub(base.len()));
    return format!("{base}-{padding}.txt");
}

/// A repository with a base commit,
/// `changed` files edited and staged.
fn staged_repository(fixture: &Fixture, name: &str, changed: usize) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    for index in 0..changed {
        std::fs::write(repo.join(long_name(index)), format!("line {index}\n")).expect("a file");
    }
    let paths: PathBuf = fixture.root.join(format!("{name}-paths.txt"));
    let mut listed: String = String::new();
    for index in 0..changed {
        listed.push_str(long_name(index).as_str());
        listed.push('\n');
    }
    std::fs::write(paths.as_path(), listed).expect("the list of paths");
    git(fixture, repo.as_path(), &["add", "--all"]);
    for index in 0..changed {
        std::fs::write(repo.join(long_name(index)), format!("line {index}, edited\n")).expect("an edit");
    }
    git(fixture, repo.as_path(), &["add", "--all"]);
    return repo;
}

/// The commit line of a control: every named path,
/// which is the route whose request carries the changed paths.
fn commit_arguments(fixture: &Fixture, name: &str) -> Vec<String> {
    let paths: PathBuf = fixture.root.join(format!("{name}-paths.txt"));
    return vec![
        String::from("commit"),
        String::from("--quiet"),
        format!("--pathspec-from-file={}", paths.display()),
        String::from("--message=a commit of many files"),
    ];
}

/// The commit a worktree is at.
fn head_of(fixture: &Fixture, repo: &Path) -> String {
    let output = git(fixture, repo, &["rev-parse", "HEAD"]);
    return String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
}

/// Whether the worktree holds Git's own index lock,
/// which a commit must never leave behind.
fn index_locked(repo: &Path) -> bool {
    return repo.join(".git/index.lock").exists();
}

/// Waits until `condition` holds,
/// and fails the control naming it when it does not.
fn wait_for(what: &str, condition: &dyn Fn() -> bool) {
    let began: Instant = Instant::now();
    while !condition() {
        assert!(began.elapsed() < PATIENCE, "{what} never happened within {PATIENCE:?}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The service a fixture started for its one worktree,
/// checked to be a service before anything signals it.
fn service_process(fixture: &Fixture) -> u32 {
    let service = fixture.service.lock().expect("the fixture's service lock");
    let started = service.as_ref().expect("the fixture started a service");
    let running: Vec<u32> = started.process_ids();
    assert_eq!(running.len(), 1, "one worktree has one service");
    let pid: u32 = running[0];
    let command_line: String = std::fs::read_to_string(format!("/proc/{pid}/cmdline")).expect("the process's own command line");
    assert!(command_line.contains("service"), "the process {pid} is the fixture's commit service: {command_line}");
    return pid;
}

/// What:
///  A commit whose service stopped before it greeted fails at the wait the greeting is given,
/// says the commit was not made and that the staged work is untouched,
/// and leaves nothing that stops the next command.
/// Why:
///  The audit of release 0.6.1-alpha.12 stopped the service during a commit and read the failure this control reads;
/// the person is owed the step that ends it,
/// which is what this checks the words for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(service.pid, 'SIGSTOP'); const ran = await observe(wrappedCommit(...));
/// expect(ran.code).toBe(1); expect(ran.stderr).toMatch(/was not made/);
/// ```
#[test]
fn a_commit_whose_service_stopped_names_the_wait_and_leaves_the_staged_work() {
    let fixture: Fixture = fixture("service-stopped-before-the-greeting");
    let repo: PathBuf = staged_repository(&fixture, "repo", 1);
    let before: String = head_of(&fixture, repo.as_path());
    let _runtime: PathBuf = service_runtime(&fixture).expect("the fixture serves its worktree");
    let pid: u32 = service_process(&fixture);
    // SAFETY: `kill` takes a process identifier and a signal, and this one is the fixture's own service, which the check
    //  above named.
    assert_eq!(unsafe { libc::kill(pid.cast_signed(), libc::SIGSTOP) }, 0, "the fixture's service is stopped");
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let began: Instant = Instant::now();
    let ran: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    let took: Duration = began.elapsed();
    // SAFETY: as above, with the signal that lets the fixture's service run again.
    assert_eq!(unsafe { libc::kill(pid.cast_signed(), libc::SIGCONT) }, 0, "the fixture's service runs again");
    let words: String = String::from_utf8_lossy(ran.stderr.as_slice()).into_owned();
    assert_eq!(ran.code, Some(1), "a commit whose service stopped fails: {words}");
    assert!(took >= Duration::from_secs(10), "the greeting is waited for no less than the start wait: {took:?}");
    assert!(took < PATIENCE, "and the wait ends: {took:?}");
    assert!(words.contains("was not made"), "the commit says it was not made: {words}");
    assert!(words.contains("staged changes and your working files are as they were"), "it says the staged work is untouched: {words}");
    assert!(words.contains("service status"), "it names the step that shows the service's state: {words}");
    assert!(words.contains("restart"), "it names the step that restarts the service: {words}");
    assert!(words.contains("Run the same commit again"), "it says running it again is safe: {words}");
    assert!(!index_locked(repo.as_path()), "no index lock is left: {}", index_locked(repo.as_path()));
    assert_eq!(head_of(&fixture, repo.as_path()), before, "no commit was made");
    // The same command lands once the service answers again, so nothing of the failure outlives it.
    let again: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    assert_eq!(again.code, Some(0), "the retry lands: {}", String::from_utf8_lossy(again.stderr.as_slice()));
    remove_settled(&fixture);
}

/// What:
///  A commit whose service greeted it and then read nothing fails at the bound on the write of its request,
/// says the request never reached the service,
/// and prints that within seconds rather than waiting in the kernel for ever.
/// Why:
///  This is the case the audit of release 0.6.1-alpha.12 found:
/// a request of 652,332 bytes,
/// a service stopped with `SIGSTOP`,
/// and a wrapper in `sendto` that had made no progress after twelve seconds and printed nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = standIn(fixture, repo, 'read-nothing'); const ran = await observe(wrappedCommit(...));
/// expect(ran.code).toBe(1); expect(ran.stderr).toMatch(/stopped reading/);
/// ```
#[test]
fn a_commit_whose_service_stopped_reading_fails_at_the_write_bound() {
    let fixture: Fixture = fixture("service-stopped-reading");
    let repo: PathBuf = staged_repository(&fixture, "repo", MANY_FILES);
    let before: String = head_of(&fixture, repo.as_path());
    let stand_in: StandIn = stand_in(&fixture, repo.as_path(), Duty::ReadNothing);
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let began: Instant = Instant::now();
    let ran: Observed = observe(&mut wrapped_against_stand_in(&fixture, &stand_in, repo.as_path(), borrowed.as_slice(), COMMIT_BOUND_SECONDS), b"");
    let took: Duration = began.elapsed();
    let words: String = String::from_utf8_lossy(ran.stderr.as_slice()).into_owned();
    assert_eq!(stand_in.greeted.load(Ordering::SeqCst), 1, "the stand-in greeted the commit once");
    assert_eq!(ran.code, Some(1), "a commit whose service stopped reading fails: {words}");
    // The write waits on progress and not on a clock: its silence window is the one a service that works may keep, and the
    //  whole request has the wait a start is granted.
    assert!(took >= HEARTBEAT_SILENCE, "the write is waited for no less than its silence bound: {took:?}");
    assert!(took < HEARTBEAT_SILENCE + START_WAIT, "and it ends at the write's own bound: {took:?}");
    assert!(words.contains("stopped reading"), "the words name what stopped: {words}");
    assert!(words.contains("took no part of this commit's request"), "they name what the service did not take: {words}");
    assert!(words.contains("was not made"), "the commit says it was not made: {words}");
    assert!(words.contains("staged changes and your working files are as they were"), "it says the staged work is untouched: {words}");
    assert!(!index_locked(repo.as_path()), "no index lock is left");
    assert_eq!(head_of(&fixture, repo.as_path()), before, "no commit was made");
    remove_settled(&fixture);
}

/// What:
///  A commit whose service read the whole request and answered nothing still lands nothing,
/// and the command ends at the silence bound with the words for a result nobody knows yet.
/// Why:
///  The stand-in greets again on the connection the client makes after a silence,
/// which is what tells a service that stopped from one whose beats were late,
/// and the request is sent again under the same identity,
/// so nothing lands twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = standIn(fixture, repo, 'read-and-say-nothing'); const ran = await observe(wrappedCommit(...));
/// expect(ran.code).toBe(1); expect(ran.stderr).toMatch(/stopped answering/);
/// ```
#[test]
fn a_commit_whose_service_reads_and_says_nothing_ends_at_the_silence_bound() {
    let fixture: Fixture = fixture("service-says-nothing");
    let repo: PathBuf = staged_repository(&fixture, "repo", 1);
    let before: String = head_of(&fixture, repo.as_path());
    let stand_in: StandIn = stand_in(&fixture, repo.as_path(), Duty::ReadsAndSilent);
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let began: Instant = Instant::now();
    let ran: Observed = observe(&mut wrapped_against_stand_in(&fixture, &stand_in, repo.as_path(), borrowed.as_slice(), COMMIT_BOUND_SECONDS), b"");
    let took: Duration = began.elapsed();
    let words: String = String::from_utf8_lossy(ran.stderr.as_slice()).into_owned();
    assert!(stand_in.read_all.load(Ordering::SeqCst), "the stand-in read the whole request");
    assert_eq!(ran.code, Some(1), "a commit nothing answers fails: {words}");
    assert!(took >= HEARTBEAT_SILENCE, "the answer is waited for no less than the silence bound: {took:?}");
    assert!(took < PATIENCE, "and the wait ends: {took:?}");
    assert!(words.contains("stopped answering"), "the words name what stopped: {words}");
    assert!(words.contains("service status"), "it names the step that shows the service's state: {words}");
    assert!(!index_locked(repo.as_path()), "no index lock is left");
    assert_eq!(head_of(&fixture, repo.as_path()), before, "no commit was made");
    remove_settled(&fixture);
}

impl StandIn {
    /// The whole request line the stand-in read,
    /// with its framing newline.
    fn recording(&self) -> Option<Vec<u8>> {
        return self.recorded.lock().expect("the stand-in's record of the request").clone();
    }
}

/// What:
///  A connection the real service greeted,
///  tried again while a connection is taken and answered nothing.
/// Why:
///   A service that has just started takes a connection before it greets,
/// and a control that speaks first would be talking to a connection the
///  service is about to drop;
/// the retry keeps the control about framing rather than about a start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const connection = connectAndGreet(socket); // a stream whose first line was the welcome
/// ```
fn connect_and_greet(socket: &Path) -> (UnixStream, String) {
    let began: Instant = Instant::now();
    loop {
        let mut connection: UnixStream = UnixStream::connect(socket).expect("the real service listens");
        let greeting: String = read_for(&mut connection, Duration::from_secs(2)).0;
        if greeting.contains("welcome") {
            return (connection, greeting);
        }
        assert!(began.elapsed() < PATIENCE, "the service at {} never greeted a connection", socket.display());
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// What a connection to the service said within `within`,
/// and whether the service closed it.
fn read_for(connection: &mut UnixStream, within: Duration) -> (String, bool) {
    let until: Instant = Instant::now() + within;
    let mut said: String = String::new();
    let mut chunk: [u8; 4096] = [0; 4096];
    loop {
        let left: Duration = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return (said, false);
        }
        let _timeout = connection.set_read_timeout(Some(left));
        match std::io::Read::read(connection, &mut chunk) {
            Ok(0) => return (said, true),
            Ok(read) => said.push_str(String::from_utf8_lossy(&chunk[..read]).as_ref()),
            Err(error) if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => return (said, false),
            Err(_ended) => return (said, true),
        }
    }
}

/// What:
///  A commit whose landing takes longer than the silence bound still lands, because the service says it is alive while it
///  works.
/// Why:
///  The owner's bound is five seconds over native Git,
/// and a working service must never be taken for one that stopped:
/// the silence this client ends at is a service that says nothing,
/// and a service that is landing says a beat every second
/// (`HEARTBEAT_EVERY`).
/// Measured on this host on 2026-11-12: the whole commit of 600 changed files took 4.7 s and a landing of 2,500 changed
/// files 52.8 s,
/// so a commit of 1,600 of them waits longer than the silence bound for its landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = await observe(wrappedCommit(fixture, repo, 600 paths)); expect(ran.code).toBe(0)
/// ```
#[test]
fn a_commit_that_waits_longer_than_the_silence_still_lands() {
    let fixture: Fixture = fixture("service-slow-but-working");
    let repo: PathBuf = staged_repository(&fixture, "repo", MANY_FILES);
    let before: String = head_of(&fixture, repo.as_path());
    let _runtime: PathBuf = service_runtime(&fixture).expect("the fixture serves its worktree");
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let began: Instant = Instant::now();
    let ran: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    let took: Duration = began.elapsed();
    let words: String = String::from_utf8_lossy(ran.stderr.as_slice()).into_owned();
    assert_eq!(ran.code, Some(0), "a commit whose landing is slower than the silence bound lands: {words}");
    assert!(took > HEARTBEAT_SILENCE, "the landing it waited for is longer than the silence bound: {took:?}");
    assert!(!words.contains("said nothing"), "a working service is never taken for one that stopped: {words}");
    assert_ne!(head_of(&fixture, repo.as_path()), before, "and the branch is at a new commit");
    remove_settled(&fixture);
}

/// What:
///  A service that wakes after the commit failed makes no commit,
///  and the staged work is where it was,
///  so the next commit is the one that makes it.
/// Why:
///  A commit a person was told had failed must not appear afterwards.
/// Nothing of the request is left for a service that wakes:
/// the whole line never reached it,
/// or the request was dropped because the command on its connection is gone
///  (`service_land_requests_whose_client_is_gone.rs`).
/// This control stops the service of the worktree before the commit asks it anything,
/// watches the commit fail,
/// and lets the service run again before looking at the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.kill(service.pid, 'SIGSTOP'); await fails(); process.kill(service.pid, 'SIGCONT'); await settled; expect(head).toBe(before);
/// ```
#[test]
fn a_service_that_wakes_after_the_failure_makes_no_commit() {
    let fixture: Fixture = fixture("service-wakes-after-the-failure");
    let repo: PathBuf = staged_repository(&fixture, "repo", 1);
    let before: String = head_of(&fixture, repo.as_path());
    let _runtime: PathBuf = service_runtime(&fixture).expect("the fixture serves its worktree");
    let pid: u32 = service_process(&fixture);
    // SAFETY: `kill` takes a process identifier and a signal, and this one is the fixture's own service, which the check above
    //  named.
    assert_eq!(unsafe { libc::kill(pid.cast_signed(), libc::SIGSTOP) }, 0, "the fixture's service is stopped");
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let failed: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    assert_eq!(failed.code, Some(1), "the commit whose service is stopped fails: {}", String::from_utf8_lossy(failed.stderr.as_slice()));
    // SAFETY: as above, with the signal that lets the fixture's service run again.
    assert_eq!(unsafe { libc::kill(pid.cast_signed(), libc::SIGCONT) }, 0, "the fixture's service runs again");
    // What a service that woke with a request to land would do within a second of running again.
    std::thread::sleep(Duration::from_secs(6));
    assert_eq!(head_of(&fixture, repo.as_path()), before, "the waking service made no commit");
    let staged: String = String::from_utf8_lossy(git(&fixture, repo.as_path(), &["status", "--porcelain=v1"]).stdout.as_slice()).into_owned();
    assert!(staged.contains(long_name(0).as_str()), "the staged edit is still staged: {staged}");
    assert!(!index_locked(repo.as_path()), "no index lock is left behind");
    // The next command of the worktree is not stopped by anything the failure left.
    let again: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    assert_eq!(again.code, Some(0), "the retry lands: {}", String::from_utf8_lossy(again.stderr.as_slice()));
    assert_ne!(head_of(&fixture, repo.as_path()), before, "and it is the commit the person asked for once");
    remove_settled(&fixture);
}

/// What:
///  A commit that finds an instance holding its socket and answering nothing ends within the wait a start is granted,
///  and the same commit reaches a live service when it is run again.
/// Why:
///  A service that was told to stop keeps its socket until it ends,
/// and the manager starts the next instance only when it has gone:
/// measured on this machine on 2026-11-12,
/// a client waited 37,327 ms for a socket a dying instance still held,
/// and 34 of its 35 connection tries were refused.
/// The wait is bounded, and the connect to a socket nothing accepts on no longer waits in the kernel
/// (`CONNECT_CEILING`),
/// so a client that finds such an instance ends at the start wait with words that name it,
/// and its next run reaches the service that is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const standIn = standIn(fixture, repo, 'say-nothing'); await fails(); drop(standIn); await wrappedCommit(fixture); expect(landed)
/// ```
#[test]
fn a_commit_that_finds_a_dying_instance_ends_within_the_start_wait() {
    let fixture: Fixture = fixture("dying-instance");
    let repo: PathBuf = staged_repository(&fixture, "repo", 1);
    let before: String = head_of(&fixture, repo.as_path());
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let stand_in: StandIn = stand_in(&fixture, repo.as_path(), Duty::SilentThroughout);
    let began: Instant = Instant::now();
    let ran: Observed = observe(&mut wrapped_against_stand_in(&fixture, &stand_in, repo.as_path(), borrowed.as_slice(), COMMIT_BOUND_SECONDS), b"");
    let took: Duration = began.elapsed();
    let words: String = String::from_utf8_lossy(ran.stderr.as_slice()).into_owned();
    assert!(stand_in.accepted.load(Ordering::SeqCst) > 0, "the stand-in took the connection of the commit");
    assert_eq!(ran.code, Some(1), "a commit whose service answers nothing fails: {words}");
    assert!(took >= START_WAIT, "the wait is the one a start is granted: {took:?}");
    assert!(took < START_WAIT + HEARTBEAT_SILENCE + HEARTBEAT_SILENCE, "and it ends there rather than waiting for the instance: {took:?}");
    assert!(words.contains("Nothing has answered"), "the words name what did not answer: {words}");
    assert!(words.contains("was not made"), "and that the commit was not made: {words}");
    // The instance that held the socket is gone, which is what the manager waits for before it starts the next one.
    drop(stand_in);
    let again: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), borrowed.as_slice()), b"");
    assert_eq!(again.code, Some(0), "the same commit reaches a live service and lands: {}", String::from_utf8_lossy(again.stderr.as_slice()));
    assert_ne!(head_of(&fixture, repo.as_path()), before, "and the branch is at a new commit");
    remove_settled(&fixture);
}

/// What:
///  A commit killed at each step of the exchange with its service leaves no commit,
/// no lock and nothing that stops the next command.
/// Why:
///  A command can be ended at any instruction on this machine.
/// The three steps are the greeting,
/// the write of the request and the wait for the answer,
/// and each is reached here through a stand-in that says when it is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const duty of ['say-nothing', 'read-nothing', 'read-and-say-nothing']) { const child = spawn(...); await at(duty); process.kill(child.pid, 'SIGKILL'); await look(); }
/// ```
#[test]
fn a_commit_killed_at_each_step_leaves_nothing_behind() {
    for (duty, at) in [(Duty::SilentThroughout, "the greeting"), (Duty::ReadNothing, "the write of the request"), (Duty::ReadsAndSilent, "the wait for the answer")] {
        let fixture: Fixture = fixture(format!("killed-at-{at}").replace(' ', "-").as_str());
        let changed: usize = if matches!(duty, Duty::ReadNothing) { MANY_FILES } else { 1 };
        let repo: PathBuf = staged_repository(&fixture, "repo", changed);
        let before: String = head_of(&fixture, repo.as_path());
        let stand_in: StandIn = stand_in(&fixture, repo.as_path(), duty);
        let arguments: Vec<String> = commit_arguments(&fixture, "repo");
        let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
        let mut child: Child = wrapped_against_stand_in(&fixture, &stand_in, repo.as_path(), borrowed.as_slice(), COMMIT_BOUND_SECONDS)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the commit starts");
        // A stand-in that greets nothing is known by the connection it took; every other one by the greeting it sent.
        let marker: &Arc<AtomicUsize> = if matches!(duty, Duty::SilentThroughout) { &stand_in.accepted } else { &stand_in.greeted };
        let reached: &dyn Fn() -> bool = &|| { return marker.load(Ordering::SeqCst) > 0; };
        wait_for(format!("the stand-in to answer the commit ({at})").as_str(), reached);
        if matches!(duty, Duty::ReadsAndSilent) {
            // The kill lands while the wrapper waits for an answer to a request the service has read.
            let read: &dyn Fn() -> bool = &|| { return stand_in.read_all.load(Ordering::SeqCst); };
            wait_for("the stand-in to read the whole request", read);
        } else {
            std::thread::sleep(Duration::from_millis(300));
        }
        let _killed = child.kill();
        let _reaped = child.wait();
        assert!(!index_locked(repo.as_path()), "a commit killed at {at} leaves no index lock");
        assert_eq!(head_of(&fixture, repo.as_path()), before, "a commit killed at {at} makes no commit");
        let staged: String = String::from_utf8_lossy(git(&fixture, repo.as_path(), &["status", "--porcelain=v1"]).stdout.as_slice()).into_owned();
        assert!(staged.contains(long_name(0).as_str()), "the staged edit survives a kill at {at}: {staged}");
        remove_settled(&fixture);
    }
}

/// What:
///  A request line whose framing newline never arrived is refused by the real service:
///  nothing of it is read as a message, nothing is queued, the branch does not move,
///  and the service says so in its log; the same bytes with their newline are taken.
/// Why:
///  A client that gives up writing says the service never took the request,
///  and that is true only if the service refuses a line the client could not finish.
///  The review of this change sent a real 3,262-byte request without its newline to the real service binary
///  and the commit landed, twice, once after a full close;
///  and the client's chunked write can leave exactly that stream, because a socket's buffers hold the whole
///  JSON object and not the last newline.
///  The control first has the client send a real request to a stand-in that records it,
///  then puts the real service at the same address and sends the recorded line without its newline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = await capturedRequest(fixture, repo); // from a stand-in that refuses it
/// await theRealService.serve(fixture, gitDir);
/// expect(await sendAndClose(socket, line.slice(0, -1)).answers).toBe(''); expect(head()).toBe(before);
/// ```
#[test]
fn a_request_line_whose_newline_never_arrived_is_refused_and_lands_nothing() {
    let fixture: Fixture = fixture("request-line-never-framed");
    let repo: PathBuf = staged_repository(&fixture, "repo", 1);
    let git_dir: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the worktree's Git directory");
    let before: String = head_of(&fixture, repo.as_path());
    // The client's own request line, recorded by a stand-in that refuses it, so the control sends the bytes a commit sends.
    let stand_in: StandIn = stand_in(&fixture, repo.as_path(), Duty::ReadsAndRefuses);
    let arguments: Vec<String> = commit_arguments(&fixture, "repo");
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let refused: Observed = observe(&mut wrapped_against_stand_in(&fixture, &stand_in, repo.as_path(), borrowed.as_slice(), COMMIT_BOUND_SECONDS), b"");
    assert_eq!(refused.code, Some(1), "the commit the stand-in refuses fails: {}", String::from_utf8_lossy(refused.stderr.as_slice()));
    let request: Vec<u8> = stand_in.recording().expect("the stand-in recorded the client's request line");
    assert!(request.ends_with(b"\n"), "a request line ends with its framing newline: {:?}", String::from_utf8_lossy(request.as_slice()));
    // The stand-in is dropped whole, and the real service of this worktree starts in a runtime directory of its own: the
    //  socket's name is the worktree's instance and only the directory differs, so a control still talks about this worktree,
    //  and no straggling connection of the stand-in can answer it.
    drop(stand_in);
    let service: FixtureService = start_fixture_service(&fixture);
    service.serve(&fixture, git_dir.as_path());
    let socket: PathBuf = socket_for(service.runtime.as_path(), git_dir.as_path());
    let (mut connection, greeting): (UnixStream, String) = connect_and_greet(socket.as_path());
    assert!(greeting.contains("welcome"), "the real service greets the connection: {greeting}");
    // The exact line the client sent, without its framing newline, and then the client's half of the connection closed.
    if let Err(error) = std::io::Write::write_all(&mut connection, &request[..request.len() - 1]) {
        let log: String = std::fs::read_to_string(fixture.root.join("service.log")).unwrap_or_default();
        panic!("the request line goes out without its newline: {error}; service log: {log}");
    }
    connection.shutdown(std::net::Shutdown::Write).expect("the client's half of the connection is closed");
    let (said, _closed): (String, bool) = read_for(&mut connection, Duration::from_secs(3));
    drop(connection);
    assert!(said.is_empty(), "a line that ended with the connection is refused, so no answer comes back: {said}");
    assert_eq!(head_of(&fixture, repo.as_path()), before, "no commit was made");
    let journal: PathBuf = fixture.root.join("journal");
    let queued: Vec<(PathBuf, Option<git_policy_cli::service_journal::Intent>)> =
        git_policy_cli::service_journal::list_intents_read_only(journal.as_path(), git_policy_cli::service_journal::OPEN_EXTENSION);
    assert!(queued.is_empty(), "nothing was queued: {queued:?}");
    let log: String = std::fs::read_to_string(fixture.root.join("service.log")).unwrap_or_default();
    assert!(log.contains("framing newline"), "the service says in its log that it refused the line: {log}");
    // The same bytes with their framing newline are taken: the refusal is the missing newline and nothing else.
    let (mut framed, _greeting): (UnixStream, String) = connect_and_greet(socket.as_path());
    std::io::Write::write_all(&mut framed, request.as_slice()).expect("the line goes out whole");
    let (answered, _closed): (String, bool) = read_for(&mut framed, PATIENCE);
    assert!(answered.contains("\"landed\""), "the same request with its newline is taken and lands: {answered}");
    assert_ne!(head_of(&fixture, repo.as_path()), before, "and the branch is at a new commit");
    remove_settled(&fixture);
}
