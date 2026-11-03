//! What:
//! Runs a child process and collects its output within a time bound.
//! Why:
//! The owner's rule of 2026-11-03 is "nothing should ever wait forever or indefinitely".
//! At the bound the child is stopped and the caller gets an error that names what was running and the command to run again;
//! every file stays where it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run.ts
//! export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
//! ```

use std::io::{self, Read, Write};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// How often the child is asked whether it has finished.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How long output readers are given to finish after the child exited.
/// A grandchild that inherited the pipes (cargo's compiler processes) can keep them open after the child is gone;
/// what was read by then is the output.
/// A bound chosen for the owner's rule above, not measured.
const READER_GRACE: Duration = Duration::from_millis(1_000);

/// The bound of one run and the words an error uses.
#[derive(Clone, Debug)]
pub struct Bound {
    /// The longest the child may run.
    pub limit: Duration,
    /// What is running, for the error.
    pub what: String,
    /// The command to run again, for the error.
    pub rerun: String,
}

/// Why a bounded run did not produce output.
#[derive(Debug)]
pub enum RunError {
    /// The child could not be started or read.
    Io(
        /// The system error.
        io::Error,
    ),
    /// The child outlived its bound and was killed.
    TimedOut {
        /// What was running.
        what: String,
        /// The bound it exceeded.
        limit: Duration,
        /// The command to run again.
        rerun: String,
    },
}

impl std::fmt::Display for RunError {
    /// What:
    /// `fn fmt(&self, formatter) -> fmt::Result` words the error for the user.
    /// Why:
    /// A timeout names what blocked, says nothing was lost and names the command to run again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            RunError::Io(error) => write!(formatter, "{error}"),
            RunError::TimedOut { what, limit, rerun } => write!(
                formatter,
                "{what} did not finish within {limit:?} and was stopped; every file is kept as it was. Run `{rerun}` again when it can finish"
            ),
        };
    }
}

/// What:
/// `fn collect(mut stream: impl Read + Send + 'static) -> Arc<Mutex<Vec<u8>>>` reads a pipe on its own thread into a shared buffer.
/// Why:
/// A child that fills a pipe nobody reads blocks, and a reader that never finishes must not block the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(stream: Readable): { bytes: Buffer }  // appended to by a background reader
/// ```
fn collect(mut stream: impl Read + Send + 'static) -> Arc<Mutex<Vec<u8>>> {
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let shared: Arc<Mutex<Vec<u8>>> = Arc::clone(&buffer);
    thread::spawn(move || {
        let mut chunk: [u8; 8192] = [0; 8192];
        loop {
            match stream.read(&mut chunk) {
                Ok(0) | Err(_) => return,
                Ok(count) => {
                    if let Ok(mut held) = shared.lock() {
                        held.extend_from_slice(&chunk[..count]);
                    }
                }
            }
        }
    });
    return buffer;
}

/// What:
/// `fn take(buffer: &Arc<Mutex<Vec<u8>>>, until: Instant) -> Vec<u8>` returns what a reader has gathered once it has stopped
/// growing or the time is up.
/// Why:
/// The child exited, so a reader that is still going is held open by a grandchild;
/// the bytes read so far are the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function take(buffer: { bytes: Buffer }, until: number): Buffer
/// ```
fn take(buffer: &Arc<Mutex<Vec<u8>>>, until: Instant) -> Vec<u8> {
    let mut last: usize = usize::MAX;
    loop {
        let length: usize = buffer.lock().map(|held| { return held.len(); }).unwrap_or(0);
        if Instant::now() >= until || (length == last && Arc::strong_count(buffer) == 1) {
            break;
        }
        last = length;
        thread::sleep(POLL_INTERVAL);
    }
    return buffer.lock().map(|held| { return held.clone(); }).unwrap_or_default();
}

/// What:
/// `pub fn output_within(command, input, bound) -> Result<Output, RunError>` runs the command, feeds it `input` on standard input
/// and returns its output, or stops it at the bound.
/// Why:
/// Every wait for a child process in cargo-shadow goes through here, so each has a bound and a named blocker.
/// The child is always waited for, killed at the bound and then waited for again, so none is left unreaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function outputWithin(command: Command, input: Buffer | undefined, bound: Bound): Output  // throws RunError
/// ```
pub fn output_within(command: &mut Command, input: Option<&[u8]>, bound: &Bound) -> Result<Output, RunError> {
    command.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(RunError::Io)?;
    if let (Some(bytes), Some(mut stdin)) = (input, child.stdin.take()) {
        let owned: Vec<u8> = bytes.to_vec();
        thread::spawn(move || {
            // A child that exits early closes the pipe; the failed write is the child's answer, and it is waited for below.
            let _ = stdin.write_all(&owned);
        });
    }
    let stdout = collect(child.stdout.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard output pipe")); })?);
    let stderr = collect(child.stderr.take().ok_or_else(|| { return RunError::Io(io::Error::other("no standard error pipe")); })?);
    let deadline: Instant = Instant::now() + bound.limit;
    let status = loop {
        match child.try_wait().map_err(RunError::Io)? {
            Some(found) => break found,
            None => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(RunError::TimedOut { what: bound.what.clone(), limit: bound.limit, rerun: bound.rerun.clone() });
                }
                thread::sleep(POLL_INTERVAL);
            }
        }
    };
    let until: Instant = Instant::now() + READER_GRACE;
    return Ok(Output { status, stdout: take(&stdout, until), stderr: take(&stderr, until) });
}
