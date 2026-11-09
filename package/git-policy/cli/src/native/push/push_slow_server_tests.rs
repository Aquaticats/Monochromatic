//! Controls for a push whose server runs a check after the pack is sent and says nothing for longer than the stall window,
//!  with real Git:
//! over HTTP to Git's own `http-backend` behind a loopback port,
//! and over a local path as the control.
//! Why:
//!  GitHub checks a push (its pre-receive hooks) after the pack arrived,
//! and a push that waits for that is not stuck.
//! The
//!  remote here sets `receive.keepAlive` to 0 ("set to 0 to disable keepalives entirely",
//! `git help config`) so the server is
//!  exactly silent,
//! and its `pre-receive` hook sleeps past the window:
//! the push must land on its first try over a connection that
//!  is open,
//! however long the server takes,
//! and be stopped as before where there is no connection.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const attempt = await runBoundedPush(pushTo(httpRemoteWithSilentHook), { kind: 'progress', stallMs });
//! expect(attempt.exitCode).toBe(0);
//! ```
#![cfg(unix)]

use crate::auto_push_single_flight::{PushAttempt, PushGit};
use crate::bounded_push::{PUSH_DEADLINE_VARIABLE, Patience, STOPPED_EXIT_CODE, WAIT_NOTE_PHRASE, run_bounded_push};
use crate::lfs_bounded_push::run_push;
use crate::smart_http_support::SmartHttp;
use crate::test_support::{REAL_GIT, fixture, git, git_text, remove, repository, scratch_environment};
use std::io::Read;
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::ffi::OsString;
use std::process::Command;
use std::time::{Duration, Instant};

/// The stall window of the controls,
/// a whole second as the environment variable of the push takes it.
const STALL: Duration = Duration::from_secs(1);

/// How many stall windows the server's hook stays silent.
const SILENCE_WINDOWS: u32 = 4;

/// The grace after the polite stop request.
const GRACE: Duration = Duration::from_secs(2);

/// How long a loaded host may take over a stop beyond its limit and the grace.
const STOP_SLACK: Duration = Duration::from_secs(30);

/// How long the fixture's server reads what the push sends between two looks at whether this control let the connection go.
const READ_SLICE: Duration = Duration::from_millis(50);

/// What the fixture's listener did with the push's connection.
enum Reached {
    /// The connection was taken and is held.
    Connected,
    /// The listener refused it, so no connection was ever held and the push has nothing to wait on.
    Failed(std::io::Error),
}

/// A bare remote whose `pre-receive` hook sleeps for `silence` and whose server sends no keepalive.
fn silent_remote(projects: &std::path::Path, silence: Duration) -> PathBuf {
    let remote: PathBuf = projects.join("remote.git");
    git(projects, &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(remote.as_path(), &["config", "http.receivepack", "true"]);
    git(remote.as_path(), &["config", "receive.keepAlive", "0"]);
    let hook: PathBuf = remote.join("hooks").join("pre-receive");
    std::fs::write(hook.as_path(), format!("#!/bin/sh\nexec sleep {}\n", silence.as_secs())).expect("hook");
    std::fs::set_permissions(hook.as_path(), std::fs::Permissions::from_mode(0o755)).expect("hook mode");
    return remote;
}

/// The push of `main` to `origin`,
/// with the environment the wrapper gives its pushes and nothing from the host.
fn push_command(repo: &std::path::Path) -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .args(["push", "origin", "main"])
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_TERMINAL_PROMPT", "0");
    return command;
}

/// Run the push under a patience,
/// with its time.
fn run(repo: &std::path::Path, patience: Patience) -> (PushAttempt, Duration) {
    let started: Instant = Instant::now();
    let attempt: PushAttempt = run_bounded_push(push_command(repo), patience, GRACE, scratch_environment().as_slice()).expect("runs");
    return (attempt, started.elapsed());
}

/// A push over HTTP to a server that checks the push in silence for longer than the stall window lands on its first try,
/// and
///  its output names the wait.
#[test]
fn a_push_over_http_waits_for_a_silent_server_check_and_lands() {
    let root: PathBuf = fixture("slow-server-http");
    let projects: PathBuf = root.join("projects");
    std::fs::create_dir(&projects).expect("projects");
    let remote: PathBuf = silent_remote(projects.as_path(), STALL * SILENCE_WINDOWS);
    let server: SmartHttp = SmartHttp::start(projects.as_path());
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", format!("http://127.0.0.1:{}/remote.git", server.port()).as_str()]);
    let (attempt, elapsed): (PushAttempt, Duration) = run(repo.as_path(), Patience::Progress { stall: STALL });
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(elapsed >= STALL * SILENCE_WINDOWS, "the push waited out the server's silence: {elapsed:?}");
    assert!(attempt.output.contains(WAIT_NOTE_PHRASE), "the output names the wait: {}", attempt.output);
    assert_eq!(git_text(remote.as_path(), &["rev-parse", "main"]), git_text(repo.as_path(), &["rev-parse", "main"]), "the commit arrived");
    drop(server);
    remove(root.as_path());
}

/// The same silent check behind a local path has no connection to wait on (the hook is a child of the push),
/// so the push is
///  stopped on a stall,
/// as before.
#[test]
fn the_same_silence_behind_a_local_path_is_still_a_stall() {
    let root: PathBuf = fixture("slow-server-local");
    let remote: PathBuf = silent_remote(root.as_path(), STALL * SILENCE_WINDOWS);
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    let (attempt, elapsed): (PushAttempt, Duration) = run(repo.as_path(), Patience::Progress { stall: STALL });
    assert_eq!(attempt.exit_code, STOPPED_EXIT_CODE, "{}", attempt.output);
    assert!(attempt.output.contains("limit=stall"), "{}", attempt.output);
    assert!(!attempt.output.contains(WAIT_NOTE_PHRASE), "{}", attempt.output);
    assert!(elapsed >= STALL, "{elapsed:?}");
    assert!(elapsed < STALL * SILENCE_WINDOWS + GRACE + STOP_SLACK, "{elapsed:?}");
    remove(root.as_path());
}

/// A server that took the connection and says nothing for longer than the stall window is a connected remote,
/// not a stalled
///  push:
/// the wrapper does not stop the push for time,
/// and the push ends when the server lets the connection go.
///
/// The fixture lets the connection go only when this control says so, so the claim is an ordering and never a clock:
/// the push is still running after the server has kept the connection open and silent for several stall windows,
/// and it ends only once the server lets go.
/// A fixture that closed the connection first,
/// or that never took it,
/// is named as that,
/// which is what the fixture's own 4 s sleep could not tell from a wrapper that stopped the push.
#[test]
fn a_server_that_holds_the_connection_without_answering_is_waited_for() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("a local listener");
    let port: u16 = listener.local_addr().expect("its address").port();
    let silence: Duration = STALL * SILENCE_WINDOWS;
    let (reached, arriving) = std::sync::mpsc::channel::<Reached>();
    let (let_go, letting_go) = std::sync::mpsc::channel::<()>();
    let server = std::thread::spawn(move || {
        // What:
        //  Take every connection the push opens, read what it sends, answer nothing, and hold all of them until this
        //  control lets go.
        // Why:
        //  A fixture that holds one connection and refuses to take another leaves the second one to the kernel's backlog,
        //  where nobody answers it, and an accept that failed used to end the thread in silence,
        // which is a push that failed for the fixture's sake and read as a wrapper that stopped it.
        listener.set_nonblocking(true).expect("the fixture's listener can be looked at between two readers");
        let mut held: Vec<std::net::TcpStream> = Vec::new();
        let mut told: bool = false;
        let mut sink: [u8; 4096] = [0; 4096];
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let _blocking = stream.set_nonblocking(false);
                    let _timeout = stream.set_read_timeout(Some(READ_SLICE));
                    if !told && reached.send(Reached::Connected).is_err() {
                        return;
                    }
                    told = true;
                    held.push(stream);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    let _sent = reached.send(Reached::Failed(error));
                    return;
                }
            }
            for stream in &mut held {
                let _read = stream.read(&mut sink);
            }
            if letting_go.try_recv().is_ok() {
                return;
            }
            std::thread::sleep(READ_SLICE);
        }
    });
    let root: PathBuf = fixture("slow-server-holds-connection");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", format!("http://127.0.0.1:{port}/silent.git").as_str()]);
    std::thread::scope(|scope| {
        let push = scope.spawn(|| { return run(repo.as_path(), Patience::Progress { stall: STALL }); });
        // The arrival is waited for as long as the push runs and no longer: a push that ends first is reported with its own
        //  output, which is the difference between a fixture that lost the connection and a Git that gave up.
        let arrival: Reached = loop {
            match arriving.recv_timeout(READ_SLICE) {
                Ok(reached) => break reached,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let _let_go = let_go.send(());
                    let (attempt, elapsed): (PushAttempt, Duration) = push.join().expect("the push thread ends");
                    panic!(
                        "the fixture's server thread ended before this control let the connection go, and the push ended after {elapsed:?} with {}: {}",
                        attempt.exit_code, attempt.output
                    );
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) if push.is_finished() => {
                    let _let_go = let_go.send(());
                    let (attempt, elapsed): (PushAttempt, Duration) = push.join().expect("the push thread ends");
                    panic!(
                        "the push ended before the fixture's listener took its connection: after {elapsed:?} with {}: {}",
                        attempt.exit_code, attempt.output
                    );
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            }
        };
        if let Reached::Failed(error) = arrival {
            let _let_go = let_go.send(());
            let (attempt, elapsed): (PushAttempt, Duration) = push.join().expect("the push thread ends");
            panic!(
                "the fixture's listener refused the push's connection ({error}), and the push ended after {elapsed:?} with {}: {}",
                attempt.exit_code, attempt.output
            );
        }
        // The connection is open and silent for longer than several stall windows. A wrapper that stopped the push for time
        //  would have ended it by now, and so would a Git that gave up on its own.
        std::thread::sleep(silence);
        let ended_while_held: bool = push.is_finished();
        let _let_go = let_go.send(());
        let (attempt, elapsed): (PushAttempt, Duration) = push.join().expect("the push thread ends");
        assert!(
            !ended_while_held,
            "the push ended while the server still held the connection, so the wrapper stopped it or Git gave up on its own: after {elapsed:?} \
             with {}: {}",
            attempt.exit_code, attempt.output
        );
        assert_ne!(attempt.exit_code, STOPPED_EXIT_CODE, "the wrapper did not stop a connected push: {}", attempt.output);
        assert!(!attempt.output.contains("limit=stall"), "{}", attempt.output);
        assert!(attempt.output.contains(WAIT_NOTE_PHRASE), "the output names the wait: {}", attempt.output);
    });
    server.join().expect("the fixture's server thread ends");
    remove(root.as_path());
}

/// A server check that outlasts several stall windows does not stop the push,
/// and no total time does:
/// it lands.
#[test]
fn a_push_over_http_is_not_stopped_for_time_while_the_server_checks() {
    let root: PathBuf = fixture("slow-server-no-ceiling");
    let projects: PathBuf = root.join("projects");
    std::fs::create_dir(&projects).expect("projects");
    let remote: PathBuf = silent_remote(projects.as_path(), STALL * SILENCE_WINDOWS);
    let server: SmartHttp = SmartHttp::start(projects.as_path());
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", format!("http://127.0.0.1:{}/remote.git", server.port()).as_str()]);
    let (attempt, elapsed): (PushAttempt, Duration) = run(repo.as_path(), Patience::Progress { stall: STALL });
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(elapsed >= STALL * SILENCE_WINDOWS, "the push waited out the server's silence: {elapsed:?}");
    assert_eq!(git_text(remote.as_path(), &["rev-parse", "main"]), git_text(repo.as_path(), &["rev-parse", "main"]), "the commit arrived");
    drop(server);
    remove(root.as_path());
}

/// The low-speed time in seconds that the wrapper used to give a push (`GIT_HTTP_LOW_SPEED_TIME`),
/// kept here only to put the
///  silence past it.
const FORMER_LOW_SPEED_SECONDS: u64 = 60;

/// The low-speed limit in bytes per second that the wrapper used to give a push (`GIT_HTTP_LOW_SPEED_LIMIT`).
const FORMER_LOW_SPEED_BYTES: &str = "1000";

/// How much longer than the wrapper's former low-speed cutoff the server's check stays silent.
const PAST_THE_CUTOFF: Duration = Duration::from_secs(5);

/// The low-speed time a person sets in the control that proves the person's own value still applies,
/// in whole seconds.
const PERSON_LOW_SPEED_SECONDS: u64 = 2;

/// How many times the person's low-speed time the server's check stays silent in that control.
const PERSON_SILENCE_FACTOR: u32 = 6;

/// The coordinator of the wrapper's own push for the repository,
/// as the service builds it:
/// the stall window of the controls in
///  its environment,
/// and whatever the person set besides.
fn push_git(repo: &std::path::Path, person: &[(&str, &str)]) -> PushGit {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_TERMINAL_PROMPT", "0"),
        (PUSH_DEADLINE_VARIABLE, "1"),
    ]
    .iter()
    .chain(person.iter())
    .map(|(name, value)| { return (OsString::from(name), OsString::from(value)); })
    .collect();
    environment.extend(scratch_environment());
    return PushGit { real_git: PathBuf::from(REAL_GIT), cwd: repo.to_path_buf(), environment, lfs: false };
}

/// A silent HTTP server and a repository that pushes to it,
/// for a silence of `silence`.
fn http_stage(name: &str, silence: Duration) -> (PathBuf, PathBuf, PathBuf, SmartHttp) {
    let root: PathBuf = fixture(name);
    let projects: PathBuf = root.join("projects");
    std::fs::create_dir(&projects).expect("projects");
    let remote: PathBuf = silent_remote(projects.as_path(), silence);
    let server: SmartHttp = SmartHttp::start(projects.as_path());
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["remote", "add", "origin", format!("http://127.0.0.1:{}/remote.git", server.port()).as_str()]);
    return (root, repo, remote, server);
}

/// The wrapper's own push,
/// as the service runs it.
fn run_wrapper_push(git: &PushGit) -> (PushAttempt, Duration) {
    let started: Instant = Instant::now();
    let arguments: Vec<OsString> = ["push", "origin", "main"].iter().map(OsString::from).collect();
    let attempt: PushAttempt = run_push(git, arguments.as_slice()).expect("runs");
    return (attempt, started.elapsed());
}

/// A server check that stays silent past the 60 seconds after which the wrapper used to have Git end an HTTP transfer lands on
///  the first try:
/// the wrapper's push sets no low-speed limit of its own,
/// so nothing but the server or TCP ends the wait.
#[test]
fn a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands() {
    let cutoff: Duration = Duration::from_secs(FORMER_LOW_SPEED_SECONDS);
    let (root, repo, remote, server) = http_stage("slow-server-past-cutoff", cutoff + PAST_THE_CUTOFF);
    let (attempt, elapsed): (PushAttempt, Duration) = run_wrapper_push(&push_git(repo.as_path(), &[]));
    assert_eq!(attempt.exit_code, 0, "{}", attempt.output);
    assert!(elapsed >= cutoff, "the push waited past the cutoff: {elapsed:?}");
    assert!(attempt.output.contains(WAIT_NOTE_PHRASE), "{}", attempt.output);
    assert_eq!(git_text(remote.as_path(), &["rev-parse", "main"]), git_text(repo.as_path(), &["rev-parse", "main"]), "the commit arrived");
    drop(server);
    remove(root.as_path());
}

/// A low-speed limit and time the person set reach Git unchanged,
/// so they still end a silent HTTP wait:
/// a person who wants a
///  shorter cutoff has it,
/// and the wrapper's push neither adds nor lengthens one.
#[test]
fn a_low_speed_setting_the_person_made_still_ends_a_silent_http_wait() {
    let person_time: Duration = Duration::from_secs(PERSON_LOW_SPEED_SECONDS);
    let silence: Duration = person_time * PERSON_SILENCE_FACTOR;
    let (root, repo, _remote, server) = http_stage("slow-server-person-cutoff", silence);
    let seconds: String = PERSON_LOW_SPEED_SECONDS.to_string();
    let git: PushGit = push_git(repo.as_path(), &[("GIT_HTTP_LOW_SPEED_LIMIT", FORMER_LOW_SPEED_BYTES), ("GIT_HTTP_LOW_SPEED_TIME", seconds.as_str())]);
    let (attempt, elapsed): (PushAttempt, Duration) = run_wrapper_push(&git);
    assert_ne!(attempt.exit_code, 0, "{}", attempt.output);
    assert_ne!(attempt.exit_code, STOPPED_EXIT_CODE, "Git ended it, not the wrapper: {}", attempt.output);
    assert!(attempt.output.to_lowercase().contains("too slow"), "{}", attempt.output);
    assert!(elapsed < silence, "the person's cutoff ended the wait before the server's check did: {elapsed:?}");
    drop(server);
    remove(root.as_path());
}
