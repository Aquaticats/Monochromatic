//! What:
//!  Run one child program to completion with a time bound and output caps:
//!  its
//!       input comes from a file,
//!  its two outputs go to files,
//!  and the parent polls it.
//! Why:
//!  The Markdown policy starts the linter once per candidate,
//!  and a child that hangs,
//!      floods its output or exits early must never hang or exhaust the wrapper.
//!  Files
//!      instead of pipes need no reader or writer threads:
//!  a child that stops reading its
//!      input cannot block the parent,
//!  a child that writes more than the cap is stopped
//!      when the parent sees the file grow past it,
//!  and a grandchild that keeps an output
//!      open cannot delay the parent once the child itself has exited.
//!  Every file is an
//!      anonymous temporary file that the operating system removes when it is closed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const run = await runBoundedChild({ executable, args, cwd, env, input, timeoutMs, maxStdout, maxStderr });
//! ```

/// When the wait prints its notices, and the printing.
use super::wait_progress::{Cadence, ChildWaitFacts, WaitReporter, child_wait_line};
/// `OsString` is owned operating-system text:
///  arguments and environment values need not be UTF-8.
use std::ffi::OsString;
/// What:
///  `File` is an open file;
///  `Read`,
///  `Seek` and `Write` add reading,
///  repositioning and
///       writing;
///  `SeekFrom::Start(0)` names the first byte.
/// Why:
///   The input file is written before the child starts,
///  and the output files are read
///       back after it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open } from 'node:fs/promises';
/// ```
use std::fs::File;
/// `Read`,
///  `Seek` and `Write` are the traits that add reading,
///  rewinding and writing to a file.
use std::io::{Read, Seek, SeekFrom, Write};
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// What:
///  `Command` describes a child before it starts;
///  `Child` is a running one;
///       `ExitStatus` is how it ended;
///  `Stdio` says where each stream goes.
/// Why:
///   The child is started directly,
///  never through a shell.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::process::{Child, Command, ExitStatus, Stdio};
/// `Duration` is a span of time;
///  `Instant` is a point on a clock that never goes backwards.
use std::time::{Duration, Instant};

/// What:
///  The first and the longest pause between two looks at a running child.
/// Why:
///   A short child,
///  such as the linter on one file (about 11 milliseconds measured on
///       the host),
///  is noticed within a few milliseconds of ending;
///  the pause doubles so a
///       long child costs a look every 16 milliseconds at most.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIRST_PAUSE_MS = 1; const LONGEST_PAUSE_MS = 16;
/// ```
const FIRST_PAUSE: Duration = Duration::from_millis(1);
/// See `FIRST_PAUSE`.
const LONGEST_PAUSE: Duration = Duration::from_millis(16);

/// What:
///  The bounds of one child.
///  `usize` is an unsigned size in bytes.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   The caller chooses them,
///  so tests can use bounds small enough to reach quickly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildLimits = { timeoutMs: number; maxStdout: number; maxStderr: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildLimits {
    /// How long the child may run before it is killed.
    pub timeout: Duration,
    /// The most bytes of standard output kept;
    ///  one more stops the child.
    pub max_stdout: usize,
    /// The most bytes of standard error kept;
    ///  one more stops the child.
    pub max_stderr: usize,
}

/// What:
///  Everything one child needs.
///  `'a` names how long the borrowed parts last;
///       `&[(OsString, OsString)]` is the complete environment,
///  as name and value pairs.
/// Why:
///   The child gets exactly this environment and nothing inherited by accident.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildRequest = { executable: string; args: string[]; cwd: string; env: [string, string][]; input: Uint8Array; scratch: string; limits: ChildLimits };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ChildRequest<'a> {
    /// The program,
    ///  by path.
    pub executable: &'a Path,
    /// Its arguments,
    ///  after the program name.
    pub arguments: &'a [OsString],
    /// Its working directory.
    pub directory: &'a Path,
    /// Its whole environment.
    pub environment: &'a [(OsString, OsString)],
    /// The bytes it reads as standard input.
    pub input: &'a [u8],
    /// The directory the anonymous stream files are made in,
    ///  normally the system's
    /// temporary directory.
    pub scratch: &'a Path,
    /// Its time bound and output caps.
    pub limits: ChildLimits,
    /// What the wait is for, in words, as the notice names it ("monochromatic-lint to check docs/a.md").
    pub waiting_for: &'a str,
    /// When the wait prints its notices.
    pub cadence: Cadence,
}

/// What:
///  How a finished child ended:
///  its exit code,
///  or the signal that ended it.
/// Why:
///   A child ended by a signal has no exit code,
///  and the message must say which.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildExit = { kind: 'code'; code: number } | { kind: 'signal'; signal: number } | { kind: 'unknown' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildExit {
    /// The child exited with this code.
    Code(
        /// The exit code.
        i32,
    ),
    /// A signal ended the child.
    Signal(
        /// The signal number.
        i32,
    ),
    /// Neither is known;
    ///  a platform without signals reported no code.
    Unknown,
}

/// What:
///  A child that ran to its end within its bounds,
///  with what it wrote.
/// Why:
///   Interpreting the output is the caller's job;
///  this only reports it exactly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FinishedChild = { exit: ChildExit; stdout: Uint8Array; stderr: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinishedChild {
    /// How it ended.
    pub exit: ChildExit,
    /// Its exact standard output.
    pub stdout: Vec<u8>,
    /// Its exact standard error.
    pub stderr: Vec<u8>,
}

/// What:
///  Why a child produced no usable result.
///  `String` carries the operating system's
///       own words where it gave any.
/// Why:
///   Each cause gets its own sentence for the person who ran the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ChildFailure = { kind: 'files' | 'start' | 'wait'; reason: string } | { kind: 'timed-out' } | { kind: 'too-large'; stream: string; limit: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChildFailure {
    /// A temporary file for the child's streams could not be made or used.
    Files(
        /// The operating system's reason.
        String,
    ),
    /// The child could not be started.
    Start(
        /// The operating system's reason.
        String,
    ),
    /// The child could not be waited for.
    Wait(
        /// The operating system's reason.
        String,
    ),
    /// The child was still running at its time bound and was killed.
    TimedOut(
        /// The bound it exceeded.
        Duration,
    ),
    /// The child wrote more than its cap to one stream and was killed.
    TooLarge {
        /// `standard output` or `standard error`.
        stream: &'static str,
        /// The cap it exceeded,
        ///  in bytes.
        limit: usize,
    },
}

/// What:
///  The child's three stream files.
///  Each is an anonymous temporary file.
/// Why:
///   One record keeps them together for the child's whole run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StreamFiles = { input: FileHandle; stdout: FileHandle; stderr: FileHandle };
/// ```
struct StreamFiles {
    /// The child's standard input,
    ///  already written and rewound.
    input: File,
    /// The child's standard output.
    stdout: File,
    /// The child's standard error.
    stderr: File,
}

/// What:
///  The text of an operating-system failure.
///  `&std::io::Error` borrows the failure.
/// Why:
///   A named conversion for the `map_err` calls below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const files = (error: Error) => ({ kind: 'files', reason: error.message });
/// ```
fn files_failure(error: std::io::Error) -> ChildFailure {
    return ChildFailure::Files(error.to_string());
}

/// What:
///  Make the three stream files and write the input into the first.
///       The trailing `?` returns a failure to the caller at once.
/// Why:
///   The input is complete before the child starts,
///  so the child can read it at any
///       pace or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function streamFiles(input: Uint8Array, scratch: string): Promise<StreamFiles>;
/// ```
fn stream_files(input: &[u8], scratch: &Path) -> Result<StreamFiles, ChildFailure> {
    let mut written: File = tempfile::tempfile_in(scratch).map_err(files_failure)?;
    written.write_all(input).map_err(files_failure)?;
    written.seek(SeekFrom::Start(0)).map_err(files_failure)?;
    return Ok(StreamFiles {
        input: written,
        stdout: tempfile::tempfile_in(scratch).map_err(files_failure)?,
        stderr: tempfile::tempfile_in(scratch).map_err(files_failure)?,
    });
}

/// What:
///  Whether an output of `size` bytes is past a cap of `limit` bytes.
///  `u64` is an
///       unsigned 64-bit size.
/// Why:
///   A size equal to the cap is still within it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exceeds = (size: number, limit: number) => size > limit;
/// ```
fn exceeds(size: u64, limit: usize) -> bool {
    // `as u64` widens the cap to the size's type.
    return size > limit as u64;
}

/// What:
///  Whether `elapsed` has reached the time bound `timeout`.
/// Why:
///   A child still running at exactly its bound has used all of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deadlineReached = (elapsed: number, timeout: number) => elapsed >= timeout;
/// ```
fn deadline_reached(elapsed: Duration, timeout: Duration) -> bool {
    return elapsed >= timeout;
}

/// What:
///  The pause after `pause`:
///  twice as long,
///  but never longer than `LONGEST_PAUSE`.
/// Why:
///   Looks come quickly at first and settle at one every 16 milliseconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nextPause = (pause: number) => Math.min(pause * 2, LONGEST_PAUSE_MS);
/// ```
fn next_pause(pause: Duration) -> Duration {
    return (pause * 2).min(LONGEST_PAUSE);
}

/// What:
///  How long to sleep now:
///  the current pause,
///  or what is left of the bound if less.
/// Why:
///   The parent never sleeps past the bound,
///  so a stopped child is reported on time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sleepFor = (pause: number, remaining: number) => Math.min(pause, remaining);
/// ```
fn sleep_for(pause: Duration, remaining: Duration) -> Duration {
    return pause.min(remaining);
}

/// What:
///  The stream that has grown past its cap,
///  if any.
///  `&File` borrows an output file.
/// Why:
///   The parent looks at both sizes on every poll;
///  a file whose size cannot be read is
///       treated as unbounded,
///  because nothing proves it stayed small.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function overflowed(files, limits): ChildFailure | undefined;
/// ```
fn overflowed(files: &StreamFiles, limits: ChildLimits) -> Option<ChildFailure> {
    for (file, stream, limit) in [
        (&files.stdout, "standard output", limits.max_stdout),
        (&files.stderr, "standard error", limits.max_stderr),
    ] {
        let size: u64 = match file.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => u64::MAX,
        };
        if exceeds(size, limit) {
            return Some(ChildFailure::TooLarge { stream, limit });
        }
    }
    return None;
}

/// What:
///  Kill a child and collect it.
///  `&mut Child` lends the running child.
/// Why:
///   A killed child must still be waited for,
///  or it stays behind as a zombie.
///  A child
///       that already ended makes `kill` fail harmlessly,
///  which is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGKILL'); await once(child, 'exit');
/// ```
fn stop(child: &mut Child) {
    // What: `let _ = ...` discards both results on purpose.
    // Why:  The child is being abandoned; its exit is not reported, only its cause.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // child.kill('SIGKILL');
    // ```
    let _ = child.kill();
    let _ = child.wait();
}

/// What:
///  Wait for a child within its bound,
///  stopping it at the first overflow or at the
///       bound.
///  `Result<ExitStatus, ChildFailure>` is how it ended or why it was stopped.
/// Why:
///   `std` has no timed wait,
///  so the parent looks with `try_wait`,
///  which never blocks,
///       and sleeps a growing pause between looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitBounded(child, files, limits): Promise<number>;
/// ```
fn wait_bounded(
    child: &mut Child,
    files: &StreamFiles,
    limits: ChildLimits,
    reporter: &mut WaitReporter<'_>,
    waiting_for: &str,
) -> Result<ExitStatus, ChildFailure> {
    let started: Instant = Instant::now();
    let pid: u32 = child.id();
    let mut pause: Duration = FIRST_PAUSE;
    loop {
        let waited: Option<ExitStatus> = match child.try_wait() {
            Ok(found) => found,
            Err(error) => {
                stop(child);
                return Err(ChildFailure::Wait(error.to_string()));
            }
        };
        if let Some(failure) = overflowed(files, limits) {
            stop(child);
            return Err(failure);
        }
        if let Some(status) = waited {
            return Ok(status);
        }
        // A notice that is due goes out before the bound is checked, so a child stopped at its bound was announced.
        reporter.poll(|lasted, repeat| {
            return child_wait_line(&ChildWaitFacts {
                waiting_for,
                pid,
                waited: lasted,
                limit: limits.timeout,
                limit_note: "",
                repeat,
            });
        });
        let elapsed: Duration = started.elapsed();
        if deadline_reached(elapsed, limits.timeout) {
            stop(child);
            return Err(ChildFailure::TimedOut(limits.timeout));
        }
        // `.saturating_sub` is what is left of the bound, never less than nothing.
        std::thread::sleep(sleep_for(pause, limits.timeout.saturating_sub(elapsed)));
        pause = next_pause(pause);
    }
}

/// What:
///  Read one output file back from its start.
///  `&mut File` lends it.
/// Why:
///   The overflow check already proved the file is within its cap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readBack(file: FileHandle): Promise<Uint8Array>;
/// ```
fn read_back(file: &mut File) -> Result<Vec<u8>, ChildFailure> {
    let mut bytes: Vec<u8> = Vec::new();
    file.seek(SeekFrom::Start(0)).map_err(files_failure)?;
    file.read_to_end(&mut bytes).map_err(files_failure)?;
    return Ok(bytes);
}

/// What:
///  How an exit status reads:
///  its code,
///  else its signal.
///  `#[cfg(unix)]` compiles the
///       signal lookup only where signals exist.
/// Why:
///   The message names a signal when there is one instead of "no exit code".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const childExit = (code, signal) => code !== null ? { kind: 'code', code } : signal !== null ? { kind: 'signal', signal } : { kind: 'unknown' };
/// ```
fn child_exit(status: ExitStatus) -> ChildExit {
    if let Some(code) = status.code() {
        return ChildExit::Code(code);
    }
    #[cfg(unix)]
    {
        /// The trait adds `.signal()`;
        ///  `use` inside a block scopes it to this block.
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return ChildExit::Signal(signal);
        }
    }
    return ChildExit::Unknown;
}

/// What:
///  Run the child of `request` to its end,
///  or stop it at its bound or its cap.
/// Why:
///   See the module comment.
///  The environment is cleared first,
///  so the child sees
///       exactly the pairs the caller passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedChild(request: ChildRequest): Promise<FinishedChild>; // throws ChildFailure
/// ```
pub fn run_bounded_child(request: &ChildRequest<'_>) -> Result<FinishedChild, ChildFailure> {
    return run_bounded_child_noticing(request, &mut std::io::stderr());
}

/// What:
///  `run_bounded_child` with the notices of a long wait going to `notices`.
/// Why:
///   The child's output goes to files, so a child that runs long is silent;
///  the wait says what it waits for, the child's process and the bound after the grace and at every interval.
///  Tests read the notices from a buffer instead of standard error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBoundedChild(request: ChildRequest, notices: Writable): Promise<FinishedChild>; // throws ChildFailure
/// ```
pub fn run_bounded_child_noticing(
    request: &ChildRequest<'_>,
    notices: &mut dyn Write,
) -> Result<FinishedChild, ChildFailure> {
    let mut files: StreamFiles = stream_files(request.input, request.scratch)?;
    // What: `.try_clone()` makes a second handle on the same open file.
    // Why:  The child gets one handle of each output; the parent keeps the other to watch
    //       its size and read it back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const childStdout = await dup(stdout);
    // ```
    let child_input: File = files.input.try_clone().map_err(files_failure)?;
    let child_stdout: File = files.stdout.try_clone().map_err(files_failure)?;
    let child_stderr: File = files.stderr.try_clone().map_err(files_failure)?;
    let mut command: Command = Command::new(request.executable);
    command
        .args(request.arguments)
        .current_dir(request.directory)
        .env_clear()
        .stdin(Stdio::from(child_input))
        .stdout(Stdio::from(child_stdout))
        .stderr(Stdio::from(child_stderr));
    for (name, value) in request.environment {
        command.env(name, value);
    }
    let mut child: Child = match super::start_retry::RetryStart::spawn_retrying(&mut command) {
        Ok(started) => started,
        Err(error) => return Err(ChildFailure::Start(error.to_string())),
    };
    let mut reporter: WaitReporter<'_> = WaitReporter::start(request.cadence, notices);
    let status: ExitStatus = wait_bounded(&mut child, &files, request.limits, &mut reporter, request.waiting_for)?;
    return Ok(FinishedChild {
        exit: child_exit(status),
        stdout: read_back(&mut files.stdout)?,
        stderr: read_back(&mut files.stderr)?,
    });
}

/// Bound,
///  cap and stream controls stay out of the release executable.
#[cfg(test)]
#[path = "bounded_child_tests.rs"]
mod tests;
