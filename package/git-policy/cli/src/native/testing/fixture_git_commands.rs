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
/// `signal` reads which signal ended the query, which is how the ending is known.
use std::os::unix::process::ExitStatusExt as _;

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

/// How often the wait looks whether the query ended.
/// Why one millisecond:
///  this helper owns the wait, so no second thread may collect the query while the bound's signal is on its way;
/// the standard library has no timed wait, so the wait looks,
/// and a query that ends in a millisecond costs about a millisecond.
///  The typed-value sweep, which makes thousands of these calls, measured 143 s with a ten-millisecond look and 25 s
/// with a wait in another thread that this replaces.
const LOOK_EVERY: std::time::Duration = std::time::Duration::from_millis(1);

/// How long a query this helper has killed may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once, so this window only covers the kernel and the scheduler;
/// a query that still has not been reaped in it is a defect of this helper, not a slow host.
const REAPING: std::time::Duration = std::time::Duration::from_secs(5);

/// What:
///  The seam a control of the ending uses:
///  it runs between the last look at the query and the signal that ends it.
/// Why:
///  That window is a few instructions wide,
/// so no query can arrange to end in it from outside,
/// and a control about which ending this helper reports needs a way to put an ending there.
///  Nothing of this seam exists in the release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeTheSignal?.(query); // absent unless a control armed it
/// ```
#[cfg(test)]
fn the_ending_seam(query: libc::pid_t) {
    if let Some(seam) = ENDING_SEAM.lock().expect("the ending seam").take() {
        seam(query);
    }
}

/// The one ending seam a control arms, for the one call it measures.
#[cfg(test)]
static ENDING_SEAM: std::sync::Mutex<Option<Box<dyn FnOnce(libc::pid_t) + Send>>> = std::sync::Mutex::new(None);

/// `the_ending_seam` in a build without controls: nothing runs.
#[cfg(not(test))]
fn the_ending_seam(_query: libc::pid_t) {}

/// Arm the ending seam for one call of `real_git_within`.
#[cfg(test)]
pub(crate) fn arm_the_ending_seam(seam: impl FnOnce(libc::pid_t) + Send + 'static) {
    *ENDING_SEAM.lock().expect("the ending seam") = Some(Box::new(seam));
}

/// Disarm the ending seam, answering whether it was still armed.
/// Why:
///  a control that did not reach the seam must not leave it armed for another test.
#[cfg(test)]
pub(crate) fn disarm_the_ending_seam() -> bool {
    return ENDING_SEAM.lock().expect("the ending seam").take().is_some();
}

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
    /// Whether the input was still being written when the grace passed,
    /// so a descendant of the query kept its read end open and the query never read all of it.
    pub input_held: bool,
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
/// `QueryEnding::AtTheBound` is true exactly when the status the wait returns shows that signal,
/// never when a signal could merely be sent,
/// and a query that ended by itself comes back with its own status and output.
/// Nothing waits without a bound, and no answer the query gave becomes a panic:
/// the write of the input runs on a thread of its own,
/// so a query that keeps its standard input open and reads none of it fills the pipe and is ended at the bound
/// like any other,
/// a write the ending broke is part of that ending rather than a failure,
/// and an input a descendant of the query still holds is said in `input_held`, as a held pipe is in `held_open`.
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
    // This helper owns the wait, so no other thread can collect the query before the bound's signal is sent:
    // the query stays unreaped, and the number it leads its group by stays its own until it is killed or reaped.
    let began: std::time::Instant = std::time::Instant::now();
    let (status, ending): (std::process::ExitStatus, QueryEnding) = loop {
        if let Some(status) = child.try_wait().expect("the query's state") {
            break (status, QueryEnding::Itself);
        }
        if began.elapsed() >= bound {
            // A control about the ending places the query's own exit here, before the signal.
            the_ending_seam(group);
            // SAFETY: `kill` reads the two numbers it is given and touches no memory of ours.
            let _killed = unsafe { libc::kill(-group, libc::SIGKILL) };
            // The killed query is reaped within its own window, so nothing waits without a bound here either.
            let reaping: std::time::Instant = std::time::Instant::now() + REAPING;
            let status: std::process::ExitStatus = loop {
                if let Some(status) = child.try_wait().expect("the killed query's state") {
                    break status;
                }
                if std::time::Instant::now() >= reaping {
                    panic!("the query {program:?} was killed at its bound of {bound:?} and not reaped within {REAPING:?}");
                }
                std::thread::sleep(LOOK_EVERY);
            };
            // The status the wait returns says who ended the query: the signal this helper sent at its bound,
            // or the query's own ending, which it reached first and which is not this helper's.
            // Whether a signal could be sent says nothing about that, so it is not read here.
            break (status, if status.signal() == Some(libc::SIGKILL) { QueryEnding::AtTheBound } else { QueryEnding::Itself });
        }
        std::thread::sleep(LOOK_EVERY);
    };
    // One grace for the three pipes, and never a wait past it.
    let grace_ends: std::time::Instant = std::time::Instant::now() + PIPE_GRACE;
    // The write ends with the query, and the ending is what judges it:
    // a write the query's death broke is part of that ending and no failure,
    // any other failure of it is said,
    // and a write a descendant of the query still holds is said in the answer, as a held pipe is.
    let mut input_held: bool = false;
    if let Some(writer) = input_writer {
        let written: Option<std::io::Result<()>> = input_receiver.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).ok();
        match written {
            // A descendant of the query kept the read end open and reads none of it: the write is left to that process.
            None => input_held = true,
            Some(Ok(())) => {}
            Some(Err(error)) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
            Some(Err(error)) => panic!("the input of the query {program:?} could not be written: {error}"),
        }
        if !input_held {
            let _writer_joined = writer.join();
        }
    }
    // What the pipes gave, under the same grace, and never a wait past it.
    let output_ended: bool = output_reached_its_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let error_ended: bool = error_reached_its_end.recv_timeout(grace_ends.saturating_duration_since(std::time::Instant::now())).is_ok();
    let stdout: Vec<u8> = standard_output.lock().expect("the query's standard output").clone();
    let stderr: Vec<u8> = standard_error.lock().expect("the query's standard error").clone();
    return QueryAnswer {
        output: Output { status, stdout, stderr },
        ending,
        held_open: !(output_ended && error_ended),
        input_held,
    };
}

/// The controls of the bounded query above.
#[cfg(test)]
#[path = "fixture_git_commands_tests.rs"]
mod tests;
