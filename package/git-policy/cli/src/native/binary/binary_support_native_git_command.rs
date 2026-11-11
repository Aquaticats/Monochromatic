//! What:
//!  The native Git command every control starts, and its bound:
//! the command line,
//! the read that keeps its two streams in files,
//! and the words a failure prints.
//! Why:
//!  A native Git waiting on something a control cannot see
//! (a lock another program holds,
//! a hook that never answers,
//! a mount that stopped answering)
//! held the conclusion control until the runner's five-minute stop ended the run,
//! naming nothing (2026-11-11).
//!  A bound alone is not enough:
//! `Command::output` returns only when the captured pipes reach end of file,
//! and `timeout` ends Git alone,
//! so a program that a hook of Git started keeps the pipe open past the bound
//!  and the wait outlives it.
//! Both are decided here so every read of native Git in this harness is bounded and says what it waited for.

use super::support::hermetic;
use crate::support_fixture_service::Fixture;
use crate::support_observation::TIMED_OUT_SIGNAL;
use crate::support_program_names_and_time_bound::{NATIVE_GIT_BOUND_SECONDS, REAL_GIT};
use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};

/// What:
///  The same invocation sent straight to real Git,
/// as the comparison oracle,
/// with a bound of its own.
/// Why a bound:
///  A native Git waiting on something a control cannot see
/// (a lock another program holds,
/// a hook that never answers,
/// a mount that stopped answering)
/// held the control until the runner's five-minute stop ended the run,
/// naming nothing (2026-11-11,
/// the conclusion control of the landing states).
/// A control that must hold the child itself,
/// or read how Git itself ended,
/// starts `direct_held` instead:
/// `timeout` stands between that control and Git
/// and reports its own ending.
pub fn direct(fixture: &Fixture) -> Command {
    return native_git_bound(fixture, NATIVE_GIT_BOUND_SECONDS);
}

/// Real Git with the fixture's hermetic environment,
/// and no `timeout` in front of it,
/// for a control that holds the child (a foreign holder's commit)
/// or reads the signal Git itself ended of (a `SIGPIPE`,
/// a `SIGKILL` of its own).
/// Every other command of this harness carries a bound:
/// see `direct`.
pub fn direct_held(fixture: &Fixture) -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    hermetic(&mut command, fixture, OsStr::new("/usr/bin:/bin"));
    return command;
}

/// A `timeout` that ends the real Git it starts at `seconds`,
/// under the fixture's hermetic environment.
pub fn native_git_bound(fixture: &Fixture, seconds: &str) -> Command {
    let mut command: Command = Command::new("/usr/bin/timeout");
    command.arg("--signal=KILL").arg(seconds).arg(REAL_GIT);
    hermetic(&mut command, fixture, OsStr::new("/usr/bin:/bin"));
    return command;
}

/// What:
///  Runs one prepared native Git command in `directory` to its end,
/// bounded at `bound` seconds,
/// with its two streams kept in files under the fixture.
/// Why files and not `Command::output`:
///  A captured pipe reaches end of file only when every process holding its write end is gone,
/// and `timeout` ends Git alone:
/// a hook's descendant that inherited the pipe outlives it,
/// so the wait for the output outlives the bound and the diagnostic naming the command never arrives.
/// A regular file has no such reader.
/// # Panics
///  When `timeout` ended the command at `bound`:
///  the failure names the arguments,
///  the directory and the bound,
///  so a control that met a Git waiting on something it cannot see says which one it was.
pub fn native_git_output(fixture: &Fixture, command: &mut Command, directory: &Path, bound: &str) -> Output {
    let line: String = spelled(command);
    let bounded: String = format!("/usr/bin/timeout --signal=KILL {bound} {REAL_GIT}");
    assert!(
        line == bounded || line.starts_with(format!("{bounded} ").as_str()),
        "this control asks for the failure of a command bounded at {bound} s and starts {line} instead: every read of native Git here keeps its stream in a file, so it must carry the bound in its own command line"
    );
    let mut stdout: std::fs::File = tempfile::tempfile_in(&fixture.root).expect("a file for standard output");
    let mut stderr: std::fs::File = tempfile::tempfile_in(&fixture.root).expect("a file for standard error");
    let status: ExitStatus = command
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().expect("standard output file")))
        .stderr(Stdio::from(stderr.try_clone().expect("standard error file")))
        .status()
        .expect("native Git starts");
    assert_ne!(
        status.signal(),
        Some(TIMED_OUT_SIGNAL),
        "native Git {line} in {} was ended at its {bound} s bound: the control was waiting for Git to answer, so no field of what it would have answered can be compared, and the run fails here naming it instead of holding the run",
        directory.display()
    );
    return Output { status, stdout: read_from_start(&mut stdout), stderr: read_from_start(&mut stderr) };
}

/// Real Git run in `directory` under the bound of one fixture command,
/// with its two streams kept in files.
pub fn native_git<S: AsRef<OsStr>>(fixture: &Fixture, directory: &Path, arguments: &[S]) -> Output {
    return native_git_within(fixture, directory, arguments, NATIVE_GIT_BOUND_SECONDS);
}

/// Real Git run in `directory` under `bound` seconds,
/// with its two streams kept in files.
/// A control that means to meet a bound it reaches in a moment states its own.
pub fn native_git_within<S: AsRef<OsStr>>(fixture: &Fixture, directory: &Path, arguments: &[S], bound: &str) -> Output {
    let mut command: Command = native_git_bound(fixture, bound);
    command.args(arguments);
    return native_git_output(fixture, &mut command, directory, bound);
}

/// Everything a file holds,
/// read from its start.
pub fn read_from_start(file: &mut std::fs::File) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    if file.seek(std::io::SeekFrom::Start(0)).is_ok() {
        let _read = file.read_to_end(&mut bytes);
    }
    return bytes;
}

/// A command line as a person reads it:
///  the program and then each argument,
///  separated by spaces.
pub fn spelled(command: &Command) -> String {
    let mut words: Vec<String> = vec![command.get_program().to_string_lossy().into_owned()];
    words.extend(command.get_args().map(|argument| { return argument.to_string_lossy().into_owned(); }));
    return words.join(" ");
}
