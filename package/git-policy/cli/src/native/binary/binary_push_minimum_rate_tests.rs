//! What:
//!  Controls for the least traffic a push's connection must carry and still count as progress,
//!  through the built wrapper and a local smart-HTTP server of this file's own.
//! Why:
//!  A server that keeps the connection alive and moves a byte now and then used to hold a typed
//!  push for ever,
//!  because every byte reset both silence windows
//! (the audit's F14 for a dripped answer,
//! and F3 for the keepalive a server sends while its `pre-receive` hook never answers).
//!  The owner's answer of 2026-11-12 is a minimum rate:
//! at least a stated number of bytes within a stated window,
// and under it
//!  the push stops with a message
//! (`SPEC.md`,
//! "The least traffic that counts as progress").
//!  Each control here runs the push after a commit through the wrapper's own service,
//! with the floor and the connected window a
//!  control can shorten,
//! against a loopback server that serves Git's own `git http-backend` and relays its answer at a
//!  chosen pace:
//!       a hung hook whose only traffic is Git's keepalive,
//!       a dripped answer,
//!       an upload the server reads a few bytes at a time,
//!       an honest push on a slow line above the floor,
//!       and a hook that says nothing at all,
//! which the floor must not cut sooner than the connected window does.
//!  Every stop is checked for what it left:
//! a pipe every process of the command inherits reports when the last holder ended,
//! so a
//!  stopped upload that left a helper behind fails the control.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const relay = relayOf(projects, { writeChunk: 1, writePauseMs: 200 });
//! const landed = await observe(wrappedCommit({ env: { CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS: '3' } }));
//! expect(pushReport(repo, landed.stderr)).toContain('limit=connected'); expect(tripwire.released()).toBe(true);
//! ```

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::fault_remote::Tripwire;
use super::support::{
    Fixture, Observed, REAL_GIT, fixture, git, give_service_environment, observe, push_report, push_unfinished_words, repository, stderr_of,
};
use git_policy_cli::bounded_push::PUSH_MINIMUM_RATE_VARIABLE;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// The connected window the controls give the push,
/// in whole seconds:
/// short enough to wait for,
/// and the window whose silence rule is what a sub-floor transfer is judged by.
const WINDOW_SECONDS: u64 = 3;

/// The least traffic a control's connection must carry in that window,
/// where the control means the transfer to be under it:
/// above any handshake and below the keepalive of five bytes per five seconds.
const FLOOR_ABOVE_A_TRICKLE: u64 = 4096;

/// The least traffic a control's connection must carry where the control means an honest push to pass it:
/// the payload of the control's own commit is far above it.
const FLOOR_BELOW_AN_HONEST_PUSH: u64 = 512;

/// A floor no traffic of these controls reaches,
/// so every window is judged on a connection that carried too little:
/// what a hook that says nothing at all must not be cut by.
const FLOOR_NOTHING_REACHES: u64 = 10_000_000;

/// How much longer than its window and the stop grace a control lets a stop take on a loaded host,
/// before it fails the run.
const STOP_SLACK: Duration = Duration::from_secs(30);

/// The pause the fixture's service is told to wait before it tries a stopped push again:
/// long enough that a control sees only the push it started.
const NO_RETRY_SECONDS: &str = "3600";

/// How long the hook of the control that hangs sleeps,
/// far past every window it is judged by.
const HANGING_HOOK_SECONDS: u64 = 1000;

/// How long the hook of the control that says nothing and answers sleeps,
/// longer than the floor's judgement and shorter than the connected window.
const SILENT_HOOK_SECONDS: u64 = 2;

/// The payload of a control's commit, in bytes,
/// incompressible so the pack that is sent is about its size.
const PAYLOAD_BYTES: usize = 64 * 1024;

/// The pace a control's server moves bytes at, in each direction.
#[derive(Clone, Copy, Debug)]
struct Pace {
    /// How many bytes are read from the client between two pauses.
    read_chunk: usize,
    /// How long the pause between two reads lasts.
    read_pause: Duration,
    /// How many bytes of an answer are written between two pauses.
    write_chunk: usize,
    /// How long the pause between two writes lasts.
    write_pause: Duration,
    /// Whether the ref advertisement is written whole:
    /// it is what the push's handshake reads before any transfer,
    /// and a handshake dripped a byte at a time
    ///  would hold a control for minutes before the phase it means to measure.
    whole_advertisement: bool,
}

impl Pace {
    /// An answer written whole, and a body read whole.
    const FAST: Pace =
        Pace { read_chunk: 65536, read_pause: Duration::ZERO, write_chunk: 65536, write_pause: Duration::ZERO, whole_advertisement: true };
}

/// What a relay's clients moved through it.
#[derive(Debug, Default)]
struct Counts {
    /// Bytes read from clients.
    read: AtomicU64,
    /// Bytes written to clients.
    written: AtomicU64,
    /// Requests answered.
    requests: AtomicU64,
}

impl Counts {
    /// The bytes read so far.
    fn read(&self) -> u64 {
        return self.read.load(Ordering::SeqCst);
    }

    /// The bytes written to clients so far.
    fn written(&self) -> u64 {
        return self.written.load(Ordering::SeqCst);
    }
}

/// A loopback smart-HTTP relay in front of `git http-backend`,
/// whose two directions move at a pace.
struct Relay {
    /// The loopback port.
    port: u16,
    /// What its clients moved.
    counts: Arc<Counts>,
    /// Set when the relay is being stopped.
    stopping: Arc<AtomicBool>,
    /// The thread that accepts, and joins the threads that serve.
    acceptor: Option<JoinHandle<()>>,
}

impl Relay {
    /// Start serving the repositories under `projects` at `pace`.
    fn start(projects: &Path, pace: Pace) -> Relay {
        let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a loopback listener");
        let port: u16 = listener.local_addr().expect("its address").port();
        let counts: Arc<Counts> = Arc::new(Counts::default());
        let stopping: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let flag: Arc<AtomicBool> = Arc::clone(&stopping);
        let served: Arc<Counts> = Arc::clone(&counts);
        let root: PathBuf = projects.to_path_buf();
        let acceptor: JoinHandle<()> = std::thread::spawn(move || {
            let mut serving: Vec<JoinHandle<()>> = Vec::new();
            for incoming in listener.incoming() {
                let Ok(stream) = incoming else { continue };
                if flag.load(Ordering::SeqCst) {
                    break;
                }
                let connection_root: PathBuf = root.clone();
                let connection_counts: Arc<Counts> = Arc::clone(&served);
                serving.push(std::thread::spawn(move || {
                    serve_connection(stream, connection_root.as_path(), pace, connection_counts.as_ref());
                }));
            }
            for thread in serving {
                let _ended = thread.join();
            }
        });
        return Relay { port, counts, stopping, acceptor: Some(acceptor) };
    }

    /// The URL of the remote named `name` under the relay.
    fn url(&self, name: &str) -> String {
        return format!("http://127.0.0.1:{}/{name}", self.port);
    }
}

impl Drop for Relay {
    /// Stop accepting,
    /// and wait for the threads that serve:
    /// one connection is made so the blocked `accept` returns.
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        let _woken = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(acceptor) = self.acceptor.take() {
            let _ended = acceptor.join();
        }
    }
}

/// Read one line of a request,
/// a byte at a time,
/// so the paced reads of a body are the socket's own.
fn read_line(stream: &mut TcpStream) -> String {
    let mut line: Vec<u8> = Vec::new();
    let mut byte: [u8; 1] = [0];
    while let Ok(1) = stream.read(&mut byte) {
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
        // A request line or a header is never longer than this, and a client that sent more is not one this test serves.
        if line.len() > 8192 {
            break;
        }
    }
    return String::from_utf8_lossy(line.as_slice()).trim_end_matches('\r').to_owned();
}

/// What:
///  Read `count` bytes of a body at a pace,
/// counting them.
/// Why:
///   The pacing has to be on the socket itself:
/// buffered reads would pull the whole body into this process at once,
/// and a
///  server that reads slowly is exactly what a slow line looks like to a push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (left > 0) { const chunk = stream.read(min(readChunk, left)); left -= chunk.length; await sleep(readPause); }
/// ```
fn read_paced(stream: &mut TcpStream, count: usize, pace: Pace, counts: &Counts) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::with_capacity(count);
    let mut buffer: Vec<u8> = vec![0; pace.read_chunk.min(count.max(1))];
    while body.len() < count {
        let want: usize = (count - body.len()).min(buffer.len());
        let Ok(got) = stream.read(&mut buffer[..want]) else { break };
        if got == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..got]);
        let _counted = counts.read.fetch_add(u64::try_from(got).unwrap_or(0), Ordering::SeqCst);
        if body.len() < count && !pace.read_pause.is_zero() {
            std::thread::sleep(pace.read_pause);
        }
    }
    return body;
}

/// Write an answer at a pace,
/// counting it.
fn write_paced(stream: &mut TcpStream, bytes: &[u8], chunk: usize, pause: Duration, counts: &Counts) -> std::io::Result<()> {
    for piece in bytes.chunks(chunk.max(1)) {
        stream.write_all(piece)?;
        let _counted = counts.written.fetch_add(u64::try_from(piece.len()).unwrap_or(0), Ordering::SeqCst);
        if !pause.is_zero() {
            let _slept = std::thread::sleep(pause);
        }
    }
    return Ok(());
}

/// What:
///  Answer one connection: one request, its body read at a pace, and the answer `git http-backend` gives relayed at a
///  pace.
/// Why:
///   The answer is streamed and not buffered,
/// so the keepalives a `receive-pack` sends while its hook runs reach the client as they
///  do from a real server,
/// and a client that goes away ends the backend with everything the hook started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { method, target, headers, body } = readRequest(stream, pace); const backend = spawn(httpBackend(env)); relay(backend.stdout, pace);
/// ```
fn serve_connection(mut stream: TcpStream, projects: &Path, pace: Pace, counts: &Counts) {
    let request_line: String = read_line(&mut stream);
    let mut words = request_line.split_whitespace();
    let (Some(method), Some(target)) = (words.next(), words.next()) else { return };
    let mut headers: HashMap<String, String> = HashMap::new();
    loop {
        let line: String = read_line(&mut stream);
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let mut body: Vec<u8> = Vec::new();
    if headers.get("transfer-encoding").is_some_and(|value| value.contains("chunked")) {
        loop {
            let size_line: String = read_line(&mut stream);
            let size: usize = usize::from_str_radix(size_line.split(';').next().unwrap_or("0"), 16).unwrap_or(0);
            if size == 0 {
                let _trailer: String = read_line(&mut stream);
                break;
            }
            body.extend_from_slice(read_paced(&mut stream, size, pace, counts).as_slice());
            let _blank: String = read_line(&mut stream);
        }
    } else {
        let length: usize = headers.get("content-length").and_then(|value| value.parse::<usize>().ok()).unwrap_or(0);
        if length > 0 {
            body = read_paced(&mut stream, length, pace, counts);
        }
    }
    let (path, query): (&str, &str) = target.split_once('?').unwrap_or((target, ""));
    let mut environment: Vec<(&str, String)> = vec![
        ("PATH", String::from("/usr/bin:/bin")),
        ("GIT_CONFIG_NOSYSTEM", String::from("1")),
        ("GIT_CONFIG_GLOBAL", String::from("/nonexistent-global-config")),
        ("GIT_PROJECT_ROOT", projects.display().to_string()),
        ("GIT_HTTP_EXPORT_ALL", String::from("1")),
        ("REQUEST_METHOD", method.to_owned()),
        ("PATH_INFO", path.to_owned()),
        ("QUERY_STRING", query.to_owned()),
        ("CONTENT_LENGTH", body.len().to_string()),
        ("REMOTE_ADDR", String::from("127.0.0.1")),
    ];
    if let Some(kind) = headers.get("content-type") {
        environment.push(("CONTENT_TYPE", kind.clone()));
    }
    if let Some(protocol) = headers.get("git-protocol") {
        environment.push(("GIT_PROTOCOL", protocol.clone()));
    }
    let mut cgi: Command = Command::new(REAL_GIT);
    cgi.arg("http-backend").env_clear().stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    for (name, value) in &environment {
        cgi.env(name, value);
    }
    // A group of its own, so a client that goes away ends the backend and the hook it runs.
    cgi.process_group(0);
    let Ok(mut child) = cgi.spawn() else { return };
    let Some(mut input) = child.stdin.take() else { return };
    let _written = input.write_all(body.as_slice());
    drop(input);
    let _counted = counts.requests.fetch_add(1, Ordering::SeqCst);
    let Some(mut output) = child.stdout.take() else { return };
    let mut head: Vec<u8> = Vec::new();
    let mut chunk: Vec<u8> = vec![0; 4096];
    let mut relaying: std::io::Result<()> = Ok(());
    // The CGI head, and then the body as it comes.
    let mut wrote_head: bool = false;
    let mut advertisement: bool = false;
    loop {
        let got: std::io::Result<usize> = output.read(&mut chunk);
        let read: usize = match got {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        if !wrote_head {
            head.extend_from_slice(&chunk[..read]);
            let Some(split) = head.windows(4).position(|window| window == b"\r\n\r\n") else { continue };
            let text: String = String::from_utf8_lossy(&head[..split]).into_owned();
            advertisement = text.to_ascii_lowercase().contains("advertisement");
            let mut status: String = String::from("200 OK");
            let mut kept: String = String::new();
            for line in text.lines() {
                if let Some(value) = line.strip_prefix("Status:") {
                    status = value.trim().to_owned();
                } else if !line.is_empty() {
                    kept.push_str(line);
                    kept.push_str("\r\n");
                }
            }
            let answer: String = format!("HTTP/1.1 {status}\r\n{kept}Connection: close\r\n\r\n");
            relaying = write_paced(&mut stream, answer.as_bytes(), 65536, Duration::ZERO, counts);
            wrote_head = true;
            let rest: &[u8] = &head[split + 4..];
            if relaying.is_ok() && !rest.is_empty() {
                relaying = relay_body(&mut stream, rest, pace, advertisement, counts);
            }
            continue;
        }
        if relaying.is_err() {
            break;
        }
        relaying = relay_body(&mut stream, &chunk[..read], pace, advertisement, counts);
    }
    // The client went away, or the answer is done: the backend ends either way, and what it started goes with it.
    if relaying.is_err() {
        let group: String = format!("-{}", child.id());
        let _ended = Command::new("kill").args(["--signal=KILL", group.as_str()]).stdout(Stdio::null()).stderr(Stdio::null()).status();
    }
    let _reaped: Option<std::process::ExitStatus> = child.wait().ok();
    let _closed = stream.shutdown(std::net::Shutdown::Both);
}

/// Relay one piece of a body at a control's pace.
fn relay_body(stream: &mut TcpStream, bytes: &[u8], pace: Pace, advertisement: bool, counts: &Counts) -> std::io::Result<()> {
    if advertisement && pace.whole_advertisement {
        return write_paced(stream, bytes, 65536, Duration::ZERO, counts);
    }
    return write_paced(stream, bytes, pace.write_chunk, pace.write_pause, counts);
}

/// A bare remote the relay serves:
/// `http.receivepack` on,
/// the keepalive a control asks for,
/// and a `pre-receive` hook that sleeps when a control asks for one.
fn remote(fixture: &Fixture, projects: &Path, keep_alive: Option<&str>, hook_seconds: Option<u64>) -> PathBuf {
    let remote: PathBuf = projects.join("remote.git");
    git(fixture, projects, &["init", "--bare", "--quiet", remote.to_str().expect("UTF-8")]);
    git(fixture, remote.as_path(), &["config", "http.receivepack", "true"]);
    if let Some(value) = keep_alive {
        git(fixture, remote.as_path(), &["config", "receive.keepAlive", value]);
    }
    if let Some(seconds) = hook_seconds {
        let hook: PathBuf = remote.join("hooks").join("pre-receive");
        std::fs::write(hook.as_path(), format!("#!/bin/sh\nexec sleep {seconds}\n")).expect("the hook");
        std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("the hook's mode");
    }
    return remote;
}

/// A repository whose `origin` is served by `relay`,
/// with one staged file whose content is incompressible,
/// so the pack the push sends is about its size.
fn world(fixture: &Fixture, relay: &Relay) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, repo.as_path(), &["remote", "add", "origin", relay.url("remote.git").as_str()]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    // A file that does not compress, so the size the push sends is the size this control chose: a linear congruential
    //  sequence is incompressible enough for this and needs no dependency.
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let payload: Vec<u8> = (0..PAYLOAD_BYTES)
        .map(|_| {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            return u8::try_from(state >> 33).unwrap_or(0);
        })
        .collect();
    std::fs::write(repo.join("payload.bin"), payload).expect("the payload");
    git(fixture, repo.as_path(), &["add", "--", "payload.bin"]);
    return repo;
}

/// Commit `payload.bin` through the wrapper,
/// with the fixture's service told the floor and the window first,
/// and the tripwire armed.
fn commit_and_push(fixture: &Fixture, repo: &Path, floor: u64) -> (Observed, Tripwire) {
    give_service_environment(fixture, "CLI_GIT_TEST_ONLY_CONNECTED_WINDOW_SECONDS", WINDOW_SECONDS.to_string().as_str());
    // A stopped push is planned for another try; a control that measured a stop does not need the service to try again, and the
    //  tries would talk to the relay long after the control's own push ended.
    give_service_environment(fixture, "CLI_GIT_PUSH_RETRY_SECONDS", NO_RETRY_SECONDS);
    give_service_environment(fixture, PUSH_MINIMUM_RATE_VARIABLE, floor.to_string().as_str());
    let tripwire: Tripwire = Tripwire::new();
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=payload", "--", "payload.bin"]);
    tripwire.arm(&mut command);
    let landed: Observed = observe(&mut command, b"");
    return (landed, tripwire);
}

/// What:
///  What a stop of a control must leave:
/// the commit saved,
/// the report saying the push did not finish and naming the window and the bytes,
/// the floor as the setting that moves it,
/// no process of the push left,
/// no lock held,
/// and the next commit's push going ahead.
/// Why:
///   The owner's rule is that a stopped push changes nothing locally and never claims a delivery,
/// and a stop
///  that left a helper running or a lock held would hold the pushes queued behind it,
/// which is what the
///  audit's F3 observed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(landed.code).toBe(0); expect(report).toContain('limit=connected'); expect(tripwire.released()).toBe(true);
/// ```
fn assert_stopped_and_contained(fixture: &Fixture, repo: &Path, relay: &Relay, landed: &Observed, tripwire: Tripwire, floor: u64) {
    let report: String = push_report(repo, &stderr_of(landed));
    assert_eq!(landed.code, Some(0), "the commit itself succeeds: {report}");
    assert!(report.contains(push_unfinished_words()), "the report says the push did not finish: {report}");
    assert!(report.contains("limit=connected"), "the stop names the window that ended it: {report}");
    assert!(report.contains("carried_bytes="), "the stop says what the connection carried: {report}");
    assert!(report.contains(&format!("minimum_bytes_per_window={floor}")), "the stop names the floor it was judged on: {report}");
    assert!(report.contains(PUSH_MINIMUM_RATE_VARIABLE), "the stop names the setting that lets a slower line through: {report}");
    assert_eq!(git_text(fixture, repo, &["rev-list", "--count", "HEAD"]).trim(), "2", "the commit is saved locally");
    assert!(tripwire.released(), "a process the push started was still running after the command returned");
    assert_nothing_left(fixture, repo);
    // The pushes queued behind a stopped one go on: the next commit's push runs and is stopped by the same server, which is
    //  what says that the branch is not held by the one before it.
    std::fs::write(repo.join("second.txt"), b"second\n").expect("the second file");
    git(fixture, repo, &["add", "--", "second.txt"]);
    let mut next: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=second", "--", "second.txt"]);
    let again: Observed = observe(&mut next, b"");
    assert_eq!(again.code, Some(0), "the next commit lands: {}", stderr_of(&again));
    assert!(relay.counts.read() > 0, "the server saw the push it stopped");
}

/// A server whose `pre-receive` hook never answers sends nothing but Git's own keepalive,
/// five bytes every five seconds,
/// and the push is ended at the connected window instead of being held for ever.
/// This is the audit's F3.
#[test]
fn a_server_whose_hook_never_answers_is_ended_at_the_connected_window() {
    let root: Fixture = fixture("push-minimum-rate-hung-hook");
    let projects: PathBuf = root.root.join("projects");
    std::fs::create_dir(&projects).expect("the projects");
    // The keepalive is Git's own default (`receive.keepAlive` unset), which is what a real server sends.
    let _remote: PathBuf = remote(&root, projects.as_path(), None, Some(HANGING_HOOK_SECONDS));
    let relay: Relay = Relay::start(projects.as_path(), Pace::FAST);
    let repo: PathBuf = world(&root, &relay);
    let began: Instant = Instant::now();
    let (landed, tripwire): (Observed, Tripwire) = commit_and_push(&root, repo.as_path(), FLOOR_ABOVE_A_TRICKLE);
    let elapsed: Duration = began.elapsed();
    assert_stopped_and_contained(&root, repo.as_path(), &relay, &landed, tripwire, FLOOR_ABOVE_A_TRICKLE);
    assert!(
        elapsed < Duration::from_secs(WINDOW_SECONDS) + STOP_SLACK,
        "the push ended at its window and not later: {elapsed:?}"
    );
    drop(relay);
    remove_settled(&root);
}

/// A server that drips its answer a byte at a time,
/// as the audit's F14 server did,
/// is ended at the connected window.
#[test]
fn a_server_that_drips_its_answer_is_ended_at_the_connected_window() {
    let root: Fixture = fixture("push-minimum-rate-dripped-answer");
    let projects: PathBuf = root.root.join("projects");
    std::fs::create_dir(&projects).expect("the projects");
    // A hook that says nothing keeps the connection open with no payload at all, and the answer is then the only traffic
    //  there is: it is dripped a byte every fifth of a second, which is what the audit's server did every three seconds.
    let _remote: PathBuf = remote(&root, projects.as_path(), Some("0"), Some(HANGING_HOOK_SECONDS));
    let pace: Pace = Pace {
        read_chunk: 65536,
        read_pause: Duration::ZERO,
        write_chunk: 1,
        write_pause: Duration::from_millis(200),
        whole_advertisement: true,
    };
    let relay: Relay = Relay::start(projects.as_path(), pace);
    let repo: PathBuf = world(&root, &relay);
    let (landed, tripwire): (Observed, Tripwire) = commit_and_push(&root, repo.as_path(), FLOOR_ABOVE_A_TRICKLE);
    assert_stopped_and_contained(&root, repo.as_path(), &relay, &landed, tripwire, FLOOR_ABOVE_A_TRICKLE);
    assert!(relay.counts.written() > 0, "the answer was relayed a byte at a time");
    drop(relay);
    remove_settled(&root);
}

/// A server that reads the upload a few bytes at a time is a line that carries too little,
/// and the push is ended at the connected window.
#[test]
fn a_server_that_reads_the_upload_slowly_is_ended_at_the_connected_window() {
    let root: Fixture = fixture("push-minimum-rate-slow-reader");
    let projects: PathBuf = root.root.join("projects");
    std::fs::create_dir(&projects).expect("the projects");
    let _remote: PathBuf = remote(&root, projects.as_path(), Some("0"), Some(HANGING_HOOK_SECONDS));
    // Eight bytes every tenth of a second is 80 bytes per second, which is 240 bytes in the control's window: far under
    //  the floor of 4096.
    let pace: Pace = Pace {
        read_chunk: 8,
        read_pause: Duration::from_millis(100),
        write_chunk: 65536,
        write_pause: Duration::ZERO,
        whole_advertisement: true,
    };
    let relay: Relay = Relay::start(projects.as_path(), pace);
    let repo: PathBuf = world(&root, &relay);
    let (landed, tripwire): (Observed, Tripwire) = commit_and_push(&root, repo.as_path(), FLOOR_ABOVE_A_TRICKLE);
    assert_stopped_and_contained(&root, repo.as_path(), &relay, &landed, tripwire, FLOOR_ABOVE_A_TRICKLE);
    drop(relay);
    remove_settled(&root);
}

/// An honest push on a slow line above the floor is not stopped:
/// it lands,
/// and its pack reaches the remote.
#[test]
fn an_honest_slow_push_above_the_floor_lands() {
    let root: Fixture = fixture("push-minimum-rate-honest-slow");
    let projects: PathBuf = root.root.join("projects");
    std::fs::create_dir(&projects).expect("the projects");
    let remote_path: PathBuf = remote(&root, projects.as_path(), None, None);
    // Eight kilobytes every fifty milliseconds is about 160 KB/s: a slow line, far above a floor of 512 bytes in the window.
    let pace: Pace = Pace {
        read_chunk: 8192,
        read_pause: Duration::from_millis(50),
        write_chunk: 65536,
        write_pause: Duration::ZERO,
        whole_advertisement: true,
    };
    let relay: Relay = Relay::start(projects.as_path(), pace);
    let repo: PathBuf = world(&root, &relay);
    let (landed, tripwire): (Observed, Tripwire) = commit_and_push(&root, repo.as_path(), FLOOR_BELOW_AN_HONEST_PUSH);
    let report: String = push_report(repo.as_path(), &stderr_of(&landed));
    assert!(tripwire.released(), "no process of the push is left: {report}");
    assert_eq!(landed.code, Some(0), "{report}");
    assert!(!report.contains("limit=connected"), "the push was not stopped: {report}");
    assert!(!report.contains(push_unfinished_words()), "the push landed: {report}");
    assert_eq!(
        git_text(&root, remote_path.as_path(), &["rev-parse", "refs/heads/main"]).trim(),
        git_text(&root, repo.as_path(), &["rev-parse", "HEAD"]).trim(),
        "the commit reached the remote: {report}"
    );
    drop(relay);
    remove_settled(&root);
}

/// A server that answers the upload and then says nothing while its hook runs carried no byte at all,
/// so the floor never judges it:
/// it is bounded by the connected window, as it was before this rule,
/// and a hook shorter than that window lands.
#[test]
fn a_silent_phase_after_the_upload_is_not_cut_by_the_floor() {
    let root: Fixture = fixture("push-minimum-rate-silent-phase");
    let projects: PathBuf = root.root.join("projects");
    std::fs::create_dir(&projects).expect("the projects");
    // No keepalive at all (`receive.keepAlive` 0), so the hook phase is exactly silent, and the floor is one no traffic of
    //  this control reaches.
    let remote_path: PathBuf = remote(&root, projects.as_path(), Some("0"), Some(SILENT_HOOK_SECONDS));
    let relay: Relay = Relay::start(projects.as_path(), Pace::FAST);
    let repo: PathBuf = world(&root, &relay);
    let began: Instant = Instant::now();
    let (landed, tripwire): (Observed, Tripwire) = commit_and_push(&root, repo.as_path(), FLOOR_NOTHING_REACHES);
    let elapsed: Duration = began.elapsed();
    let report: String = push_report(repo.as_path(), &stderr_of(&landed));
    assert!(tripwire.released(), "no process of the push is left: {report}");
    assert_eq!(landed.code, Some(0), "{report}");
    assert!(!report.contains("limit="), "the silent phase was not cut by a limit: {report}");
    assert!(elapsed >= Duration::from_secs(SILENT_HOOK_SECONDS), "the push waited out the hook: {elapsed:?}");
    assert_eq!(
        git_text(&root, remote_path.as_path(), &["rev-parse", "refs/heads/main"]).trim(),
        git_text(&root, repo.as_path(), &["rev-parse", "HEAD"]).trim(),
        "the commit reached the remote after the silent phase: {report}"
    );
    drop(relay);
    remove_settled(&root);
}
