//! What:
//!  A closed terminal and the other signals a session sends: `SIGHUP`, `SIGINT` and `SIGQUIT` to the wrapper while
//!  the commit sits in the editor, and `SIGHUP` to the whole process group (what a terminal does when it closes) while the commit sits
//!  at each phase of its transaction.
//! Why:
//!  The README promises that a closed terminal does not strand the repository.
//!  A signal to the wrapper alone is relayed to Git and ends the commit with the signal's status
//!  (`binary_commit_tests.rs` has `SIGTERM`); a hangup of the group reaches the wrapper, Git and the editor at once,
//!  while the wrapper holds its locks and has written part of its records, and the next command must recover.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const phase of phases) { const group = startCommitPausedAt(phase); process.kill(-group.pid, 'SIGHUP'); await expectRecoveredRepository(); }
//! ```

use super::commit::{await_marker, remove_settled, wrapped_commit};
use super::fault::{staged_repository, verify_recovery};
use super::support::{Fixture, fixture, stalling_editor};
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// The environment variable that arms a phase marker (`transaction_phase.rs`).
const PHASE_MARKER: &str = "CLI_GIT_TEST_ONLY_PHASE_SIGNAL";

/// Every phase a commit reaches, in order; a hangup at each is a terminal closing at that point.
const PHASES: [&str; 6] = [
    "capture-locked",
    "preparation-done",
    "landing-locked",
    "objects-migrated",
    "ref-updated",
    "index-installed",
];

/// Send a signal to one process or, with a negative target, to a process group.
fn send(target: i32, signal: i32) {
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    let status: libc::c_int = unsafe { libc::kill(target, signal) };
    assert_eq!(status, 0, "kill({target}, {signal}) failed: {}", std::io::Error::last_os_error());
}

/// The longest the announcement of the editor is waited for, in milliseconds; a hang guard, never part of a verdict.
const ANNOUNCEMENT_BOUND_MILLISECONDS: libc::c_int = 90_000;

/// Make the named pipe the editor announces itself through and open its read end without blocking.
///  The editor's own open for writing then never waits for a reader, and the read end is held from before the commit
///  starts, so the announcement reaches the test at the instant the editor makes it.
fn announcement_pipe(path: &Path) -> std::fs::File {
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a path has no NUL");
    // SAFETY: `mkfifo` reads the NUL-terminated name this function owns and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "mkfifo failed: {}", std::io::Error::last_os_error());
    return std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .expect("the read end of the announcement pipe opens");
}

/// Block until the editor has opened the announcement pipe for writing, without sleeping or polling on a timer.
///  The wait ends on the pipe's own readiness, so the signal that follows leaves within microseconds of the
///  announcement whatever the load: the worst moment for an editor that is still starting.
fn await_announcement(pipe: &std::fs::File, path: &Path) {
    let mut watched: libc::pollfd = libc::pollfd { fd: pipe.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    // SAFETY: `poll` reads and writes the one `pollfd` this function owns, whose count is 1.
    let ready: libc::c_int = unsafe { libc::poll(&raw mut watched, 1, ANNOUNCEMENT_BOUND_MILLISECONDS) };
    assert!(ready > 0, "the editor never announced itself on {}", path.display());
}

/// Send `signal` to a wrapper whose commit sits in the editor and check that the commit ends with `128 + signal` and
///  the repository recovers. The signal leaves at the instant the editor announces itself, the moment at which an
///  editor that is still starting is most likely to swallow it.
fn signal_during_the_editor(name: &str, signal: i32) {
    let fixture: Fixture = fixture(name);
    let started: PathBuf = fixture.root.join("editor-started");
    let announcement: std::fs::File = announcement_pipe(started.as_path());
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let editor: PathBuf = stalling_editor(&fixture, started.as_path());
    let child: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--", "a.txt"])
        .env("GIT_EDITOR", &editor)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the commit starts");
    await_announcement(&announcement, started.as_path());
    send(i32::try_from(child.id()).expect("a process ID fits"), signal);
    let finished = child.wait_with_output().expect("the commit ends");
    assert_eq!(
        finished.status.code(),
        Some(128 + signal),
        "{}",
        String::from_utf8_lossy(finished.stderr.as_slice())
    );
    let verdict: Result<(), String> = verify_recovery(&fixture, repo.as_path(), name, "");
    assert_eq!(verdict, Ok(()));
    remove_settled(&fixture);
}

/// A hangup sent to the wrapper alone while the editor waits ends the commit's Git and the commit with status 129.
#[test]
fn a_hangup_during_the_editor_ends_the_commit_and_leaves_nothing() {
    signal_during_the_editor("signal-hup-editor", libc::SIGHUP);
}

/// An interrupt sent to the wrapper alone while the editor waits ends the commit with status 130.
#[test]
fn an_interrupt_during_the_editor_ends_the_commit_and_leaves_nothing() {
    signal_during_the_editor("signal-int-editor", libc::SIGINT);
}

/// A quit sent to the wrapper alone while the editor waits ends the commit with status 131.
#[test]
fn a_quit_during_the_editor_ends_the_commit_and_leaves_nothing() {
    signal_during_the_editor("signal-quit-editor", libc::SIGQUIT);
}

/// A hangup of the whole process group, as a closing terminal sends, while the commit waits at each phase of its
///  transaction: the next command completes or discards the landing and nothing stays behind.
#[test]
fn a_hangup_of_the_process_group_at_each_phase_leaves_a_recoverable_repository() {
    let fixture: Fixture = fixture("signal-group-hup");
    for phase in PHASES {
        let repo: PathBuf = staged_repository(&fixture, &format!("repo-{phase}"));
        let markers: PathBuf = fixture.root.join(format!("markers-{phase}"));
        std::fs::create_dir(&markers).expect("marker directory");
        let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"]);
        command
            .env(PHASE_MARKER, format!("{phase}:pause:{}", markers.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        // SAFETY: between fork and exec only `setsid`, an async-signal-safe call, runs; the child leads a new session
        // and so a new process group, which the hangup below addresses.
        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                return Ok(());
            });
        }
        let child: Child = command.spawn().expect("the commit starts");
        await_marker(markers.join(format!("{phase}.reached")).as_path());
        send(-i32::try_from(child.id()).expect("a process ID fits"), libc::SIGHUP);
        let finished = child.wait_with_output().expect("the commit ends");
        let said: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
        let verdict: Result<(), String> = verify_recovery(&fixture, repo.as_path(), phase, said.as_str());
        assert_eq!(verdict, Ok(()), "hangup at {phase}");
    }
    remove_settled(&fixture);
}
