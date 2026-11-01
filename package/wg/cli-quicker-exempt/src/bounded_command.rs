//! Runs a command with a cleared environment,
//! a fixed search path,
//! and a time limit.
//! The commands this program runs as root (`systemd-run`,
//! `systemctl`)
//! must neither hang the start of a tunnel
//! nor look their program up in a search path the caller chose.

/// Reading what the command printed.
use std::io::Read;
/// Arguments that need not be text.
use std::ffi::OsString;
/// Starting the command.
use std::process::{Child, Command, Stdio};
/// The time limit.
use std::time::{Duration, Instant};

/// The search path the command gets,
/// and the one its program is looked up in.
pub(crate) const SECURE_PATH: &str = "/usr/bin:/bin";
/// Time between two checks whether the command ended.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// What a command that ended in time printed and how it ended.
#[derive(Debug)]
pub struct BoundedOutput {
    /// Whether it exited with status zero.
    pub success: bool,
    /// How it ended,
    /// in words.
    pub status: String,
    /// What it wrote to standard output.
    pub stdout: String,
    /// What it wrote to standard error.
    pub stderr: String,
}

/// Reads what is left in a pipe of a command that ended.
fn read_pipe<R: Read>(pipe: Option<R>) -> String {
    let mut text = String::new();
    if let Some(mut stream) = pipe {
        let _read = stream.read_to_string(&mut text);
    }
    return text;
}

/// The words of an input or output error.
fn io_message(error: std::io::Error) -> String {
    return error.to_string();
}

/// Ends a command that ran past its limit.
fn kill_and_reap(child: &mut Child) {
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// Runs `program` with `arguments`,
/// with a cleared environment that holds only a fixed search path,
/// and ends it when it has not finished within `limit`.
pub fn run_bounded(program: &str, arguments: &[OsString], limit: Duration) -> Result<BoundedOutput, String> {
    let mut child = Command::new(program)
        .env_clear()
        .env("PATH", SECURE_PATH)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(io_message)?;
    let deadline = Instant::now() + limit;
    loop {
        match child.try_wait().map_err(io_message)? {
            Some(status) => {
                return Ok(BoundedOutput {
                    success: status.success(),
                    status: status.to_string(),
                    stdout: read_pipe(child.stdout.take()),
                    stderr: read_pipe(child.stderr.take()),
                });
            }
            None => {
                if Instant::now() >= deadline {
                    kill_and_reap(&mut child);
                    return Err(format!("{program} did not finish within {limit:?} and was ended"));
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        }
    }
}
