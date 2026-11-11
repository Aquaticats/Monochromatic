//! What:
//!  Standard streams that are closed,
//! whose reader is gone,
//! or whose device is full,
//! before and during a command.
//! Why:
//!  The README promises that a closed terminal or a broken pipe does not strand the repository or make the wrapper
//!  report something other than what Git would.
//!  Rust's runtime ignores `SIGPIPE`,
//! so a write to a broken pipe returns `EPIPE` to the wrapper instead of ending
//!  it,
//! and an ignored signal survives `exec`:
//! every control here checks that a Git process the wrapper becomes or
//!  starts still ends the way it does when started directly,
//! that a commit already in its transaction lands or is
//!  recovered whichever stream failed,
//! and that no write to a dead stream panics.
//!  The reader of a broken pipe is closed before the command starts,
//! so the first write fails and no timing decides it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { writeEnd } = brokenPipe(); const status = await run(wrappedGit, ['show', 'HEAD:big.txt'], { stdout: writeEnd });
//! expect(status).toBe(await run(directGit, ['show', 'HEAD:big.txt'], { stdout: brokenPipe().writeEnd }));
//! ```

use super::commit::{remove_settled, wrapped_commit};
use super::fault::{staged_repository, verify_recovery};
use super::support::{Fixture, bounded, close_on_exec_pipe, direct_held, fixture, git, repository, wrapper_first_path};
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// What is wrong with a stream.
#[derive(Clone, Copy, Debug)]
enum StreamFault {
    /// The descriptor is closed before the program starts.
    Closed,
    /// The descriptor is the write end of a pipe whose read end is closed:
    /// every write fails with `EPIPE`.
    BrokenPipe,
    /// The descriptor is `/dev/full`:
    /// every write fails with `ENOSPC`.
    FullDevice,
}

/// Every stream fault the controls that compare exit statuses apply.
const OUTPUT_FAULTS: [StreamFault; 3] = [StreamFault::Closed, StreamFault::BrokenPipe, StreamFault::FullDevice];

/// The write end of a pipe whose read end is already closed.
fn broken_pipe() -> OwnedFd {
    let ends: [libc::c_int; 2] = close_on_exec_pipe();
    // SAFETY: both descriptors were just created here and are owned by nothing else.
    let (read_end, write_end): (OwnedFd, OwnedFd) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    drop(read_end);
    return write_end;
}

/// A `Stdio` that carries the fault,
/// for an output stream.
fn faulty_output(fault: StreamFault) -> Stdio {
    match fault {
        StreamFault::Closed => return Stdio::null(),
        StreamFault::BrokenPipe => return Stdio::from(broken_pipe()),
        StreamFault::FullDevice => {
            return Stdio::from(std::fs::OpenOptions::new().write(true).open("/dev/full").expect("/dev/full"));
        }
    }
}

/// Close descriptors 0,
/// 1 and 2 of the child just before it runs,
/// so the program starts with no standard streams.
fn close_standard_streams(command: &mut Command) {
    // SAFETY: between fork and exec only `close`, an async-signal-safe call, runs.
    unsafe {
        command.pre_exec(|| {
            libc::close(0);
            libc::close(1);
            libc::close(2);
            return Ok(());
        });
    }
}

/// Give a command the fault on its output streams and,
/// for a closed fault,
/// on its input too.
fn apply(command: &mut Command, fault: StreamFault) {
    match fault {
        StreamFault::Closed => {
            command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
            close_standard_streams(command);
        }
        StreamFault::BrokenPipe | StreamFault::FullDevice => {
            command.stdin(Stdio::null()).stdout(faulty_output(fault)).stderr(faulty_output(fault));
        }
    }
}

/// A commit that prints its summary (no `--quiet`),
/// so the wrapper and Git both write to the faulty streams.
const SUMMARY_COMMIT: [&str; 4] = ["commit", "--message=x", "--", "a.txt"];

/// A commit whose streams are faulty lands,
/// or is recovered by the next command,
/// and the wrapper never panics.
fn commit_with_faulty_streams(name: &str, fault: StreamFault) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = staged_repository(&fixture, "repo");
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &SUMMARY_COMMIT);
    apply(&mut command, fault);
    let finished: ExitStatus = command.status().expect("the commit starts");
    assert_ne!(finished.signal(), Some(9), "{fault:?}: the commit hit its time bound");
    assert_ne!(finished.code(), Some(101), "{fault:?}: the wrapper panicked");
    let verdict: Result<(), String> = verify_recovery(&fixture, repo.as_path(), name, format!("{finished:?}").as_str());
    assert_eq!(verdict, Ok(()), "{fault:?}");
    remove_settled(&fixture);
}

/// A commit started with its standard streams closed behaves as one started with them open.
#[test]
fn a_commit_with_closed_standard_streams_lands() {
    commit_with_faulty_streams("stream-closed", StreamFault::Closed);
}

/// A commit whose output pipes have no reader lands:
/// the summary has nowhere to go,
/// and nothing waits for it.
#[test]
fn a_commit_with_broken_output_pipes_lands() {
    commit_with_faulty_streams("stream-pipe", StreamFault::BrokenPipe);
}

/// A commit whose output device is full lands.
#[test]
fn a_commit_with_full_output_devices_lands() {
    commit_with_faulty_streams("stream-full", StreamFault::FullDevice);
}

/// A commit whose streams are faulty ends as the same commit started directly with the same fault:
/// Git's own status
///  is the oracle,
/// because a summary that cannot be written ends Git by `SIGPIPE` or a write error whichever program
///  starts it,
/// and the wrapper must neither add a failure nor hide one.
#[test]
fn a_commit_with_faulty_streams_ends_as_git_does() {
    let fixture: Fixture = fixture("stream-exit");
    for fault in OUTPUT_FAULTS {
        let wrapped_repo: PathBuf = staged_repository(&fixture, &format!("wrapped-{fault:?}"));
        let mut through_wrapper: Command = wrapped_commit(&fixture, wrapped_repo.as_path(), &SUMMARY_COMMIT);
        apply(&mut through_wrapper, fault);
        let wrapped: ExitStatus = through_wrapper.status().expect("the wrapped commit starts");
        let direct_repo: PathBuf = staged_repository(&fixture, &format!("direct-{fault:?}"));
        // No `timeout` in front: this control compares the ending signal of the Git it starts with the wrapper's.
        let mut straight: Command = direct_held(&fixture);
        straight.current_dir(direct_repo.as_path()).args(SUMMARY_COMMIT);
        apply(&mut straight, fault);
        let oracle: ExitStatus = straight.status().expect("the direct commit starts");
        assert_eq!(
            shell_status(wrapped),
            shell_status(oracle),
            "{fault:?}: the wrapped commit ended with {wrapped:?}, Git started directly with {oracle:?}"
        );
    }
    remove_settled(&fixture);
}

/// How a process ended as one number:
/// its exit code,
/// or 128 plus the signal that ended it,
/// as a shell reports it.
fn shell_status(status: ExitStatus) -> i32 {
    return status.code().unwrap_or_else(|| { return 128 + status.signal().unwrap_or(0); });
}

/// A repository with one tracked file of a megabyte,
/// so that `git show` writes more than a pipe holds.
fn repository_with_a_big_file(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("big"));
    std::fs::write(repo.join("big.txt"), "x".repeat(1 << 20)).expect("big file");
    git(fixture, repo.as_path(), &["add", "--", "big.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=big"]);
    return repo;
}

/// How a Git command that writes a lot ended when started directly and through the wrapper,
/// with the same fault.
fn show_both_ways(fixture: &Fixture, repo: &Path, fault: StreamFault) -> (ExitStatus, ExitStatus) {
    let mut through_wrapper: Command = bounded(fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(fixture).as_os_str());
    through_wrapper.current_dir(repo).args(["show", "HEAD:big.txt"]);
    apply(&mut through_wrapper, fault);
    // No `timeout` in front: this control compares the ending signal of the Git it starts with the wrapper's.
    let mut straight: Command = direct_held(fixture);
    straight.current_dir(repo).args(["show", "HEAD:big.txt"]);
    apply(&mut straight, fault);
    return (
        through_wrapper.status().expect("the wrapped command starts"),
        straight.status().expect("the direct command starts"),
    );
}

/// A forwarded command that writes into a broken pipe ends as Git does when started directly:
/// by `SIGPIPE`,
///  because the wrapper ignores that signal and `exec` must not hand the ignored disposition on to Git.
#[test]
fn a_forwarded_command_into_a_broken_pipe_ends_as_git_does() {
    let fixture: Fixture = fixture("stream-forward-pipe");
    let repo: PathBuf = repository_with_a_big_file(&fixture);
    let (wrapped, straight) = show_both_ways(&fixture, repo.as_path(), StreamFault::BrokenPipe);
    assert_eq!(straight.signal(), Some(libc::SIGPIPE), "the control: Git started directly ends by SIGPIPE");
    assert_eq!(wrapped, straight);
    remove_settled(&fixture);
}

/// A forwarded command that writes into a full device ends as Git does when started directly.
#[test]
fn a_forwarded_command_into_a_full_device_ends_as_git_does() {
    let fixture: Fixture = fixture("stream-forward-full");
    let repo: PathBuf = repository_with_a_big_file(&fixture);
    let (wrapped, straight) = show_both_ways(&fixture, repo.as_path(), StreamFault::FullDevice);
    assert_ne!(straight.code(), Some(0), "the control: Git started directly reports a write error");
    assert_eq!(wrapped, straight);
    remove_settled(&fixture);
}

/// A forwarded command with closed standard streams ends as Git does when started directly.
#[test]
fn a_forwarded_command_with_closed_streams_ends_as_git_does() {
    let fixture: Fixture = fixture("stream-forward-closed");
    let repo: PathBuf = repository_with_a_big_file(&fixture);
    let (wrapped, straight) = show_both_ways(&fixture, repo.as_path(), StreamFault::Closed);
    assert_eq!(wrapped, straight);
    remove_settled(&fixture);
}
