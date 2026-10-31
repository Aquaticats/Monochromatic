//! What:
//!  Remote trouble: a connection that is refused, a host name that does not resolve, a server that hangs up, a server
//!  that never answers and a `git-lfs` that never exits, met by the push after a landed commit and by an explicit
//!  `git push`.
//! Why:
//!  The README promises that DNS or mirror trouble does not wedge the wrapper: the commit stays saved, the failure
//!  is reported in words, no lock stays held and no process the push started stays running.
//!  The server side is a local socket (refused: nothing listens; hung up: it accepts and closes; silent: it accepts
//!  and reads), and the unresolvable name is under `.invalid`, which resolvers answer with "no such name" without
//!  asking anyone; no outside address is ever contacted.
//!  A pipe whose write end the child hands on to everything it starts proves that no descendant survives: its read
//!  end reports the hang-up only when the last holder has exited, which needs no sleep and no process listing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const server = await listen({ onConnection: (socket) => { /* never answer */ } });
//! const result = await run(wrappedGit, ['commit', '-m', 'x'], { env: { CLI_GIT_PUSH_DEADLINE_SECONDS: '1' } });
//! expect(result.stderr).toContain('auto-push to origin failed'); await expectNoDescendantsAlive();
//! ```

use super::commit::{assert_nothing_left, git_text, keep_maintenance_in_the_foreground, remove_settled, wrapped_commit};
use super::support::{Fixture, Observed, executable, fixture, git, observe, repository, stderr_of};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::JoinHandle;
use std::time::Duration;

/// The descriptor number the tripwire's write end has in the command and in everything it starts.
const TRIPWIRE_DESCRIPTOR: RawFd = 9;

/// How long a control waits for a descendant to exit or a server to see its client go, in milliseconds.
///  It is a deadline for a failure, not a pause: a healthy run returns as soon as the event happens.
///  It is below the 60 seconds after which Git ends a transfer that has stalled (`GIT_HTTP_LOW_SPEED_TIME`), so a
///  process that survives its push's stop is not mistaken for one that ended by itself.
const DEADLINE_MILLISECONDS: libc::c_int = 20_000;

/// A pipe that every process started under the command inherits, and the end that reports when all have exited.
struct Tripwire {
    /// The end the control polls.
    read_end: File,
    /// The end the command's tree holds; closed in this process once the command has started.
    write_end: Option<OwnedFd>,
}

impl Tripwire {
    /// A new pipe, both ends closed on `exec` so that only the copy handed to a command is inherited.
    fn new() -> Tripwire {
        let mut ends: [libc::c_int; 2] = [0, 0];
        // SAFETY: `pipe2` fills the two integers it is given.
        let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
        assert_eq!(made, 0, "pipe2 failed: {}", std::io::Error::last_os_error());
        // SAFETY: both descriptors were just created here and are owned by nothing else.
        let (read_end, write_end): (OwnedFd, OwnedFd) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
        return Tripwire { read_end: File::from(read_end), write_end: Some(write_end) };
    }

    /// Hand the write end to the command as descriptor nine.
    fn arm(&self, command: &mut Command) {
        let held: RawFd = self.write_end.as_ref().expect("the write end is open").as_raw_fd();
        // SAFETY: between fork and exec only `dup2`, an async-signal-safe call, runs; it clears the close-on-exec flag.
        unsafe {
            command.pre_exec(move || {
                if libc::dup2(held, TRIPWIRE_DESCRIPTOR) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                return Ok(());
            });
        }
    }

    /// Close this process's copy and report whether every other holder has exited within the deadline.
    fn released(mut self) -> bool {
        self.write_end = None;
        let mut watched: libc::pollfd = libc::pollfd { fd: self.read_end.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        // SAFETY: `poll` reads and writes the one `pollfd` it is given.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut watched, 1, DEADLINE_MILLISECONDS) };
        let mut byte: [u8; 1] = [0];
        return ready == 1 && self.read_end.read(&mut byte).is_ok_and(|count| { return count == 0; });
    }
}

/// What a local server does with the first connection it gets.
#[derive(Clone, Copy, Debug)]
enum Behavior {
    /// Accept and close at once: the server hangs up.
    HangUp,
    /// Accept and read without ever answering, until the client goes away.
    Silent,
}

/// A local server on an ephemeral port that handles one connection.
struct Server {
    /// The port it listens on.
    port: u16,
    /// Ends with whether the client went away (an end of stream), or `None` when no client ever connected.
    handled: JoinHandle<Option<bool>>,
}

/// Start a server that treats its first connection as `behavior` says.
fn serve(behavior: Behavior) -> Server {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port: u16 = listener.local_addr().expect("local address").port();
    let handled: JoinHandle<Option<bool>> = std::thread::spawn(move || {
        let mut watched: libc::pollfd = libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        // SAFETY: `poll` reads and writes the one `pollfd` it is given.
        let ready: libc::c_int = unsafe { libc::poll(&raw mut watched, 1, DEADLINE_MILLISECONDS) };
        if ready != 1 {
            return None;
        }
        let (mut connection, _peer): (TcpStream, _) = listener.accept().expect("accept");
        if matches!(behavior, Behavior::HangUp) {
            return Some(true);
        }
        connection.set_read_timeout(Some(Duration::from_millis(u64::try_from(DEADLINE_MILLISECONDS).expect("positive")))).expect("read timeout");
        let mut chunk: [u8; 4096] = [0; 4096];
        loop {
            match connection.read(&mut chunk) {
                Ok(0) => return Some(true),
                Ok(_) => {}
                Err(_) => return Some(false),
            }
        }
    });
    return Server { port, handled };
}

/// A repository whose `origin` is `url`, with `a.txt` staged, and a `git-lfs` that never exits when `hanging_lfs`.
fn world(fixture: &Fixture, url: &str, hanging_lfs: bool) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, repo.as_path(), &["remote", "add", "origin", url]);
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    if hanging_lfs {
        git(fixture, repo.as_path(), &["config", "lfs.url", "http://127.0.0.1:9/never-contacted"]);
        executable(fixture.root.join("bin/git-lfs").as_path(), b"#!/bin/sh\nexec sleep 600\n");
    }
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a.txt");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// Every path under the wrapper's state directories whose name ends in `.lock`, or holds a lock directory.
fn locks_left(repo: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = std::fs::read_dir(repo.join(".git"))
        .expect("git directory")
        .flatten()
        .map(|entry| { return entry.path(); })
        .filter(|path| { return path.file_name().is_some_and(|name| { return name.to_string_lossy().starts_with("cli-git"); }); })
        .collect();
    while let Some(path) = pending.pop() {
        if path.file_name().is_some_and(|name| { return name.to_string_lossy().contains(".lock"); }) {
            found.push(path.display().to_string());
        }
        if path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&path)
        {
            pending.extend(entries.flatten().map(|entry| { return entry.path(); }));
        }
    }
    return found;
}

/// Commit through the wrapper with the push deadline set and the tripwire armed.
fn commit_with_tripwire(fixture: &Fixture, repo: &Path, deadline: &str) -> (Observed, Tripwire) {
    let tripwire: Tripwire = Tripwire::new();
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    command.env("CLI_GIT_PUSH_DEADLINE_SECONDS", deadline);
    tripwire.arm(&mut command);
    let landed: Observed = observe(&mut command, b"");
    return (landed, tripwire);
}

/// What every remote failure of the push after a commit must leave:
///  the commit succeeded and is saved, the failure is reported in words, no process the push started is alive,
///  no lock is held, and the next commit lands.
fn assert_push_failure_is_contained(fixture: &Fixture, repo: &Path, landed: &Observed, tripwire: Tripwire, expected: &[&str]) {
    let stderr: String = stderr_of(landed);
    assert_eq!(landed.code, Some(0), "the commit itself succeeds: {stderr}");
    assert!(stderr.contains("auto-push to origin failed"), "the failure is reported: {stderr}");
    assert!(
        expected.iter().any(|words| { return stderr.contains(words); }),
        "the report names the cause (one of {expected:?}): {stderr}"
    );
    assert_eq!(git_text(fixture, repo, &["rev-list", "--count", "HEAD"]).trim(), "2", "the commit is saved locally");
    assert!(tripwire.released(), "a process the push started was still running after the command returned");
    assert_eq!(locks_left(repo), Vec::<String>::new(), "no lock stays held");
    assert_nothing_left(fixture, repo);
    std::fs::write(repo.join("b.txt"), b"two\n").expect("b.txt");
    git(fixture, repo, &["add", "--", "b.txt"]);
    let mut next: Command = wrapped_commit(fixture, repo, &["commit", "--quiet", "--message=two", "--", "b.txt"]);
    // The remote is still unreachable, so this commit's push fails too; the short deadline ends one that hangs.
    next.env("CLI_GIT_PUSH_DEADLINE_SECONDS", "1");
    let again: Observed = observe(&mut next, b"");
    assert_eq!(again.code, Some(0), "the next commit lands: {}", stderr_of(&again));
    assert!(!stderr_of(&again).contains("waiting for"), "the next commit waited for a lock: {}", stderr_of(&again));
}

/// A connection that is refused fails the auto-push at once; the commit stays.
#[test]
fn an_auto_push_to_a_refused_connection_keeps_the_commit_and_holds_nothing() {
    let fixture: Fixture = fixture("remote-refused");
    let repo: PathBuf = world(&fixture, "http://127.0.0.1:9/never.git", false);
    let (landed, tripwire) = commit_with_tripwire(&fixture, repo.as_path(), "30");
    assert_push_failure_is_contained(&fixture, repo.as_path(), &landed, tripwire, &["Failed to connect", "Connection refused", "couldn't connect"]);
    remove_settled(&fixture);
}

/// A host name that does not resolve fails the auto-push in words; the commit stays.
#[test]
fn an_auto_push_to_an_unresolvable_host_keeps_the_commit_and_holds_nothing() {
    let fixture: Fixture = fixture("remote-dns");
    let repo: PathBuf = world(&fixture, "https://cli-git-fault.invalid/never.git", false);
    let (landed, tripwire) = commit_with_tripwire(&fixture, repo.as_path(), "30");
    assert_push_failure_is_contained(
        &fixture,
        repo.as_path(),
        &landed,
        tripwire,
        &["Could not resolve host", "resolve", "did not finish within"],
    );
    remove_settled(&fixture);
}

/// A server that hangs up fails the auto-push; the commit stays.
#[test]
fn an_auto_push_to_a_server_that_hangs_up_keeps_the_commit_and_holds_nothing() {
    let fixture: Fixture = fixture("remote-hangup");
    let server: Server = serve(Behavior::HangUp);
    let repo: PathBuf = world(&fixture, format!("http://127.0.0.1:{}/never.git", server.port).as_str(), false);
    let (landed, tripwire) = commit_with_tripwire(&fixture, repo.as_path(), "30");
    assert_push_failure_is_contained(
        &fixture,
        repo.as_path(),
        &landed,
        tripwire,
        &["Empty reply", "RPC failed", "Connection reset", "unable to access", "did not finish within"],
    );
    remove_settled(&fixture);
}

/// A server that never answers is waited for until the push deadline and no longer: the push is stopped, the commit
///  stays, and the connection is closed by the stopped client (so no descendant survives).
#[test]
fn an_auto_push_to_a_server_that_never_answers_is_stopped_at_the_deadline() {
    let fixture: Fixture = fixture("remote-silent");
    let server: Server = serve(Behavior::Silent);
    let repo: PathBuf = world(&fixture, format!("http://127.0.0.1:{}/never.git", server.port).as_str(), false);
    let (landed, tripwire) = commit_with_tripwire(&fixture, repo.as_path(), "1");
    assert_push_failure_is_contained(&fixture, repo.as_path(), &landed, tripwire, &["did not finish within 1 seconds and was stopped"]);
    assert_eq!(server.handled.join().expect("server thread"), Some(true), "the client closed the connection when it was stopped");
    remove_settled(&fixture);
}

/// A `git-lfs` that never exits is stopped with the push that started it: the hook is a descendant of the push.
#[test]
fn an_auto_push_with_a_git_lfs_that_never_exits_is_stopped_with_everything_it_started() {
    let fixture: Fixture = fixture("remote-lfs-silent");
    let bare: PathBuf = fixture.root.join("origin.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    let repo: PathBuf = world(&fixture, bare.to_str().expect("UTF-8"), true);
    let (landed, tripwire) = commit_with_tripwire(&fixture, repo.as_path(), "1");
    assert_push_failure_is_contained(&fixture, repo.as_path(), &landed, tripwire, &["did not finish within 1 seconds and was stopped"]);
    remove_settled(&fixture);
}

/// An explicit `git push` to a remote that fails ends with Git's failure and a message, in time, and holds nothing.
fn explicit_push_fails_cleanly(name: &str, url: &str) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = world(&fixture, url, false);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    let tripwire: Tripwire = Tripwire::new();
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["push", "origin", "main"]);
    tripwire.arm(&mut command);
    let pushed: Observed = observe(&mut command, b"");
    assert_ne!(pushed.code, Some(0), "an unreachable remote cannot be pushed to");
    assert!(pushed.code.is_some(), "the push was ended by a signal: {}", stderr_of(&pushed));
    assert!(!stderr_of(&pushed).trim().is_empty(), "a failed push says why");
    assert!(tripwire.released(), "a process the push started was still running after the command returned");
    assert_eq!(locks_left(repo.as_path()), Vec::<String>::new(), "no lock stays held");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An explicit push to a refused connection fails in words and holds nothing.
#[test]
fn an_explicit_push_to_a_refused_connection_fails_cleanly() {
    explicit_push_fails_cleanly("remote-push-refused", "http://127.0.0.1:9/never.git");
}

/// An explicit push to an unresolvable host fails in words and holds nothing.
#[test]
fn an_explicit_push_to_an_unresolvable_host_fails_cleanly() {
    explicit_push_fails_cleanly("remote-push-dns", "https://cli-git-fault.invalid/never.git");
}

/// An explicit push to a server that hangs up fails in words and holds nothing.
#[test]
fn an_explicit_push_to_a_server_that_hangs_up_fails_cleanly() {
    let server: Server = serve(Behavior::HangUp);
    explicit_push_fails_cleanly("remote-push-hangup", format!("http://127.0.0.1:{}/never.git", server.port).as_str());
}
