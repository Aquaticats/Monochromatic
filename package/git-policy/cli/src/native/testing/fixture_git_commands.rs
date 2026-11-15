//! What:
//!  The ways the controls start real Git in a fixture:
//! one that answers its output,
//! one that requires it to have succeeded,
//! and one that keeps a bound on the query and says who ended it.
//! Why:
//!  Every control that needs a repository,
//!  a commit or an answer from Git starts it the same way,
//! with no system or global
//!  configuration and a fixed identity,
//!  so nothing of the account's own configuration reaches a fixture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(REAL_GIT, args, { cwd, env: { PATH: '/usr/bin:/bin', GIT_CONFIG_NOSYSTEM: '1', ... } })
//! ```

/// The one path of the host's Git that the binary tests share.
use super::REAL_GIT;
/// `OsStr`/`OsString` are borrowed/owned operating-system text (siblings `&str`/`String` must be UTF-8).
use std::ffi::{OsStr, OsString};
/// `Read`/`Write` fill the standard input of a query and drain its pipes.
use std::io::{Read, Write};
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::Path;
/// The command builder, the child's pipes and the finished child every fixture Git run is.
use std::process::{Command, Output, Stdio};
/// `process_group` gives the query a process group of its own, which the bound ends as a whole.
use std::os::unix::process::CommandExt as _;

/// Run real Git in a fixture with no system or global configuration and a fixed identity.
pub(crate) fn git_output<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    return Command::new(REAL_GIT)
        .current_dir(directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .args(arguments)
        .output()
        .expect("native Git fixture command");
}

/// Run real Git in a fixture and require success.
pub(crate) fn git<S: AsRef<OsStr>>(directory: &Path, arguments: &[S]) -> Output {
    let output: Output = git_output(directory, arguments);
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        arguments.iter().map(AsRef::as_ref).collect::<Vec<&OsStr>>(),
        String::from_utf8_lossy(&output.stderr)
    );
    return output;
}

/// How long one real-Git query of a control may run before this helper ends it.
/// Why sixty:
///  The same bound the binary-level controls give one fixture Git run
/// (`NATIVE_GIT_BOUND_SECONDS`), and for the same measurement:
/// the longest disk stall one `fsync` of a trivial command showed on this host is 4.74 s
/// (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
/// so sixty is more than ten times it.
pub(crate) const FIXTURE_GIT_BOUND: std::time::Duration = std::time::Duration::from_secs(60);

/// How long the two pipes may take to reach their end once the query is gone
/// before this answer leaves them to a descendant.
/// Why 250 ms:
///  everything a gone query wrote sits in the pipe's buffer already,
/// so the window covers the scheduler and no more,
/// and a query whose descendant holds one end open never holds the control.
pub(crate) const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(250);

/// How long a query this helper has killed may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once, so this window only covers the kernel and the scheduler;
/// a query that still has not been reaped in it is a defect of this helper, not a slow host.
const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// Who ended one bounded query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryEnding {
    /// The query ended by itself, and its status and output are its own,
    /// whatever status that was: a code of 124 or 137 or a signal's death included.
    Itself,
    /// This helper ended it at the bound, by killing the process group the query led.
    AtTheBound,
}

/// What one bounded query of real Git answered.
#[derive(Debug)]
pub(crate) struct QueryAnswer {
    /// The query's own ending and what its pipes gave before it went.
    pub output: Output,
    /// Who ended the query, decided by this helper's own act and never by the query's status.
    pub ending: QueryEnding,
    /// Whether a pipe was still held when the grace passed,
    /// so a descendant of the query kept that end open and the output may be short of what the query wrote.
    pub held_open: bool,
}

/// What:
///  One pipe of a query: the bytes as they arrive,
///  and the news that its reader reached the end.
/// Why:
///  Reading the pipes on this thread would wait for the end of file,
/// which a descendant that inherited the pipe can postpone past every bound;
/// a reader of its own lets the answer keep what has arrived and leave the rest to whoever holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { bytes, ended } = drain(child.stdout); // ended: a message when the pipe reaches its end
/// ```
fn draining<R: Read + Send + 'static>(mut reader: R) -> (std::sync::Arc<std::sync::Mutex<Vec<u8>>>, std::sync::mpsc::Receiver<()>) {
    let kept: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let sink: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::clone(&kept);
    let _reader: std::thread::JoinHandle<()> = std::thread::Builder::new().spawn(move || {
        let mut buffer: [u8; 8192] = [0; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => sink.lock().expect("the pipe's bytes").extend_from_slice(&buffer[..read]),
                Err(_) => break,
            }
        }
        let _ended = sender.send(());
    }).expect("the reader of a query's pipe starts");
    return (kept, receiver);
}

/// What:
///  Run one real-Git query under `bound` and answer what it printed and who ended it.
/// Why:
///  A control that ran Git directly would hold the suite for ever against a Git that never answers,
/// and report nothing at all;
///  and a bound asked of another program (`/usr/bin/timeout`) can only be read back from an exit status,
/// which an ordinary 124 or 137 or a signal's death does not tell apart from the bound.
/// So the bound is this helper's own promise and this helper keeps it:
/// it starts the query in a process group of its own,
/// watches the clock itself,
/// and at the bound ends that group and reaps the query;
/// `QueryEnding::AtTheBound` is true exactly when it did that,
/// and a query that ended by itself comes back with its own status and output.
/// Nothing waits without a bound, the input included:
/// the write of the input runs on a thread of its own,
/// so a query that keeps its standard input open and reads none of it fills the pipe and is ended at the bound
/// like any other,
/// and the write the ending broke is part of that ending rather than a failure.
/// The caller answers each ending with its own sentence,
/// because only it knows which query it made and what the query is for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runUnderBound('git', ['--version'], 60); // { output, ending, heldOpen }
/// ```
pub(crate) fn real_git_within(
    program: &Path,
    directory: Option<&Path>,
    environment: &[(OsString, OsString)],
    arguments: &[OsString],
    input: Option<&[u8]>,
    bound: std::time::Duration,
) -> QueryAnswer {
    let mut command: Command = Command::new(program);
    command.args(arguments);
    if let Some(at) = directory {
        command.current_dir(at);
    }
    command.env_clear();
    for (name, value) in environment {
        command.env(name, value);
    }
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() });
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    // The query leads a process group of its own, so the bound takes what it started in that group with it.
    command.process_group(0);
    let mut child: std::process::Child = command.spawn().expect("the query starts");
    let group: libc::pid_t = libc::pid_t::try_from(child.id()).expect("a pid fits the pid type");
    let (standard_output, output_reached_its_end) = draining(child.stdout.take().expect("piped standard output"));
    let (standard_error, error_reached_its_end) = draining(child.stderr.take().expect("piped standard error"));
    // The input is written on a thread of its own, under the same bound as the wait:
    // a query that keeps its standard input open and reads none of it fills the pipe,
    // and a write on this thread would wait there with nothing watching the clock.
    let (input_sender, input_receiver) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let input_writer: Option<std::thread::JoinHandle<()>> = match input {
        Some(bytes) => {
            let mut standard_input: std::process::ChildStdin = child.stdin.take().expect("piped standard input");
            let kept: Vec<u8> = bytes.to_vec();
            Some(std::thread::Builder::new().spawn(move || {
                let written: std::io::Result<()> = standard_input.write_all(kept.as_slice());
                // Dropping the handle in this thread is what gives the query its end of file.
                let _sent = input_sender.send(written);
            }).expect("the writer of a query's input starts"))
        }
        // Without an input the query's standard input is the null device, and nothing is written.
        None => None,
    };
    // The wait blocks on the query instead of looking at the clock, so a query that ends in a millisecond costs a
    // millisecond; the bound is the wait's own deadline, not a poll's.
    let (status_sender, status_receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::ExitStatus>>();
    let waiter: std::thread::JoinHandle<()> = std::thread::Builder::new().spawn(move || {
        let _status = status_sender.send(child.wait());
    }).expect("the waiter of a query starts");
    let (status, ending): (std::process::ExitStatus, QueryEnding) = match status_receiver.recv_timeout(bound) {
        Ok(Ok(status)) => (status, QueryEnding::Itself),
        Ok(Err(error)) => panic!("the ending of the query {program:?} cannot be read: {error}"),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // SAFETY: `kill` reads the two numbers it is given and touches no memory of ours.
            let killed: i32 = unsafe { libc::kill(-group, libc::SIGKILL) };
            let reaped: std::io::Result<std::process::ExitStatus> = status_receiver
                .recv_timeout(REAPING)
                .unwrap_or_else(|_| panic!("the query {program:?} was killed at its bound of {bound:?} and not reaped within {REAPING:?}"));
            let status: std::process::ExitStatus = reaped.unwrap_or_else(|error| panic!("the ending of the query {program:?} cannot be read: {error}"));
            // The helper's own act decides this and never the status the query came back with;
            // a group that was already gone was ended by itself, not by this call.
            (status, if killed == 0 { QueryEnding::AtTheBound } else { QueryEnding::Itself })
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => panic!("the waiter of the query {program:?} ended without its status"),
    };
    let _joined = waiter.join();
    // The write ends with the query, and the ending is what judges it:
    // a write the query's death broke is part of that ending and no failure,
    // and any other failure of it is said.
    if let Some(writer) = input_writer {
        let written: std::io::Result<()> = input_receiver
            .recv_timeout(REAPING)
            .unwrap_or_else(|_| panic!("the input of the query {program:?} was still being written {REAPING:?} after the query was gone"));
        let _writer_joined = writer.join();
        match written {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
            Err(error) => panic!("the input of the query {program:?} could not be written: {error}"),
        }
    }
    // What the pipes gave, under one grace for both, and never a wait past it.
    let grace_ends: std::time::Instant = std::time::Instant::now() + PIPE_GRACE;
    let output_ended: bool = output_reached_its_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let error_ended: bool = error_reached_its_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let stdout: Vec<u8> = standard_output.lock().expect("the query's standard output").clone();
    let stderr: Vec<u8> = standard_error.lock().expect("the query's standard error").clone();
    return QueryAnswer {
        output: Output { status, stdout, stderr },
        ending,
        held_open: !(output_ended && error_ended),
    };
}

/// The controls of the bounded query above.
#[cfg(test)]
#[path = "fixture_git_commands_tests.rs"]
mod tests;
