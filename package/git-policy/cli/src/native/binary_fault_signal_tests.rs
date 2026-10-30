//! What:
//!  A closed terminal and the other signals a session sends: `SIGHUP`, `SIGINT` and `SIGQUIT` to the wrapper while
//!  a hook runs, and `SIGHUP` to the whole process group (what a terminal does when it closes) while the commit sits
//!  at each phase of its transaction.
//! Why:
//!  The README promises that a closed terminal does not strand the repository.
//!  A signal to the wrapper alone is relayed to Git and ends the commit with the signal's status
//!  (`binary_commit_tests.rs` has `SIGTERM`); a hangup of the group reaches the wrapper, Git and the hook at once,
//!  while the wrapper holds its locks and has written part of its records, and the next command must recover.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const phase of phases) { const group = startCommitPausedAt(phase); process.kill(-group.pid, 'SIGHUP'); await expectRecoveredRepository(); }
//! ```

use super::commit::{await_marker, remove_settled, wrapped_commit};
use super::fault::{staged_repository, verify_recovery};
use super::support::{Fixture, configure_pre_commit_hook, executable, fixture};
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

/// A repository whose `pre-commit` hook records that it started and then waits for a minute.
fn repository_with_waiting_hook(fixture: &Fixture, started: &Path) -> PathBuf {
    let repo: PathBuf = staged_repository(fixture, "repo");
    configure_pre_commit_hook(
        fixture,
        repo.as_path(),
        format!("#!/bin/sh\ntouch '{}'\nsleep 60\n", started.display()).as_bytes(),
    );
    return repo;
}

/// Send `signal` to a wrapper whose hook is running and check that the commit ends with `128 + signal` and the
///  repository recovers.
fn signal_during_a_hook(name: &str, signal: i32) {
    let fixture: Fixture = fixture(name);
    let started: PathBuf = fixture.root.join("hook-started");
    let repo: PathBuf = repository_with_waiting_hook(&fixture, started.as_path());
    let child: Child = wrapped_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=x", "--", "a.txt"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the commit starts");
    await_marker(started.as_path());
    send(i32::try_from(child.id()).expect("a process ID fits"), signal);
    let finished = child.wait_with_output().expect("the commit ends");
    assert_eq!(
        finished.status.code(),
        Some(128 + signal),
        "{}",
        String::from_utf8_lossy(finished.stderr.as_slice())
    );
    // The hook that was waiting must not make the further commit of the check wait too.
    executable(fixture.root.join("pre-commit-hook-script").as_path(), b"#!/bin/sh\nexit 0\n");
    let verdict: Result<(), String> = verify_recovery(&fixture, repo.as_path(), name, "");
    assert_eq!(verdict, Ok(()));
    remove_settled(&fixture);
}

/// A hangup sent to the wrapper alone while a hook runs ends the hook's Git and the commit with status 129.
#[test]
fn a_hangup_during_a_hook_ends_the_commit_and_leaves_nothing() {
    signal_during_a_hook("signal-hup-hook", libc::SIGHUP);
}

/// An interrupt sent to the wrapper alone while a hook runs ends the commit with status 130.
#[test]
fn an_interrupt_during_a_hook_ends_the_commit_and_leaves_nothing() {
    signal_during_a_hook("signal-int-hook", libc::SIGINT);
}

/// A quit sent to the wrapper alone while a hook runs ends the commit with status 131.
#[test]
fn a_quit_during_a_hook_ends_the_commit_and_leaves_nothing() {
    signal_during_a_hook("signal-quit-hook", libc::SIGQUIT);
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
