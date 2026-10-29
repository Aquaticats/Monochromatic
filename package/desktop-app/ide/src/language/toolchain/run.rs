//! A run of a program that cannot outlast its time limit.
//!
//! Both output streams are read on their own threads so a full pipe cannot stop the child,
//!  and a grandchild that keeps a stream open cannot hold the result.

/// What:
///  `Command` describes a child process;
///  `Stdio` says where its streams go.
/// Why:
///  The program is started with no input and with both output streams captured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Most bytes read from one stream of the run.
///  Chosen,
///  not measured:
///  the largest output seen was a listing of about 12 KiB.
const CAPTURE_LIMIT: u64 = 1024 * 1024;

/// How long a finished run may take to hand over its streams
///  when a grandchild still holds them open.
const DRAIN_WAIT: Duration = Duration::from_millis(500);

/// What the run did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Completed = { success: boolean; status: string; stdout: string; stderr: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completed {
    /// Whether the exit status was zero.
    pub success: bool,
    /// The exit status as text,
    ///  for messages.
    pub status: String,
    /// Standard output,
    ///  decoded lossily.
    pub stdout: String,
    /// Standard error,
    ///  decoded lossily.
    pub stderr: String,
}

/// What:
///  Read one stream to its end on its own thread and send the bytes.
///  `impl Read + Send` is any readable stream that may move to another thread.
/// Why:
///  A stream nobody reads fills its pipe and stops the child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.stdout.on('end', () => send(isError, bytes));
/// ```
fn collect(
    stream: impl Read + Send + 'static,
    is_error: bool,
    sender: mpsc::Sender<(bool, Vec<u8>)>,
) {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        // A stream larger than the limit is cut; the child is not read further.
        if let Err(error) = stream.take(CAPTURE_LIMIT).read_to_end(&mut bytes) {
            tracing::debug!(?error, "cannot read a stream of the lookup run");
        }
        // The receiver is gone after a time-out; the bytes are then not needed.
        let _ = sender.send((is_error, bytes));
    });
}

/// What:
///  Run a command and wait at most `limit` for it.
///  `Duration` is a time span.
/// Why:
///  A lookup that hangs must never hold the language thread for good,
///  so the child is killed when the time is up.
///  Both streams are read on their own threads so a full pipe cannot stop it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runBounded(command: Command, limitMs: number): Promise<Completed> // rejects on timeout
/// ```
pub fn run_bounded(mut command: Command, limit: Duration) -> Result<Completed, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(started) => started,
        Err(error) => {
            return Err(format!("cannot start {:?}: {error}", command.get_program()));
        }
    };
    let (sender, receiver) = mpsc::channel::<(bool, Vec<u8>)>();
    if let Some(stream) = child.stdout.take() {
        collect(stream, false, sender.clone());
    }
    if let Some(stream) = child.stderr.take() {
        collect(stream, true, sender.clone());
    }
    drop(sender);
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {}
            Err(error) => return Err(format!("cannot wait for the lookup run: {error}")),
        }
        if started.elapsed() >= limit {
            if let Err(error) = child.kill() {
                tracing::debug!(?error, "cannot kill the timed-out lookup run");
            }
            if let Err(error) = child.wait() {
                tracing::debug!(?error, "cannot reap the timed-out lookup run");
            }
            return Err(format!(
                "did not finish within {} s and was stopped",
                limit.as_secs_f64().round()
            ));
        }
        thread::sleep(Duration::from_millis(5));
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    for _ in 0..2 {
        // A grandchild that keeps a stream open must not hold this thread: take what has arrived.
        let Ok((is_error, bytes)) = receiver.recv_timeout(DRAIN_WAIT) else {
            break;
        };
        if is_error {
            stderr = bytes;
        } else {
            stdout = bytes;
        }
    }
    return Ok(Completed {
        success: status.success(),
        status: status.to_string(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    });
}

/// The result of a finished run,
///  the time limit,
///  and a grandchild that holds the streams.
#[cfg(test)]
#[path = "run_tests.rs"]
mod tests;
