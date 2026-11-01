//! Runs a command line client as the watched user against that user's session bus.
//! A root watcher drops to the user's real and effective ids and clears supplementary groups through `setpriv`,
//! so the bus call is made by the user and not by root.
//! The client gets a cleared environment,
//! and is killed when it outlives its timeout,
//! so one hung bus cannot stall the watcher.

/// The search path every child process of the watcher gets.
use crate::bounded_command::SECURE_PATH;
/// Syscall and process failures.
use std::io;
/// Reader threads.
use std::thread::JoinHandle;
/// Child process control.
use std::process::{Child, Command, Stdio};
/// Bounded waits.
use std::time::{Duration, Instant};

/// How long a client may run before it is killed.
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(3);
/// Most bytes of standard output kept from a client.
const MAX_OUTPUT_BYTES: u64 = 65_536;
/// Pause between exit checks.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Reads at most `limit` bytes of a stream and drains the rest,
/// so a client is never left blocked on a full pipe:
/// a pipe holds 64 KiB,
/// and a client that writes more waits for the reader before it can exit.
fn read_bounded(mut stream: impl io::Read + Send, limit: u64) -> String {
    let mut kept: Vec<u8> = Vec::new();
    let _read = io::Read::read_to_end(&mut io::Read::take(io::Read::by_ref(&mut stream), limit), &mut kept);
    let _drained = io::copy(&mut stream, &mut io::sink());
    return String::from_utf8_lossy(&kept).into_owned();
}

/// Starts a thread that reads a client's stream while the client runs.
fn start_reader(stream: impl io::Read + Send + 'static, limit: u64) -> io::Result<JoinHandle<String>> {
    return std::thread::Builder::new().name("client-output".to_string()).spawn(move || return read_bounded(stream, limit));
}

/// Starts the reader of a client's stream,
/// the production choice of the starter that `run_with_readers` takes.
fn start_boxed_reader(stream: Box<dyn io::Read + Send>, limit: u64) -> io::Result<JoinHandle<String>> {
    return start_reader(stream, limit);
}

/// Kills a client and reaps it,
/// so none is left running or unreaped on a path that gives up.
fn reap(child: &mut Child) {
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// The text a reader thread collected,
/// empty when there was none.
fn collected(reader: Option<JoinHandle<String>>) -> String {
    return match reader {
        Some(handle) => handle.join().unwrap_or_default(),
        None => String::new(),
    };
}

/// Why a client run failed.
#[derive(Debug)]
pub enum ClientError {
    /// The client could not be started.
    NotStarted(
        /// The start failure.
        io::Error,
    ),
    /// The client ran and exited unsuccessfully.
    Failed(
        /// Its status and message.
        String,
    ),
    /// The client ran past its timeout and was killed.
    TimedOut(
        /// What was killed and when.
        String,
    ),
}

/// Wording for logs.
impl std::fmt::Display for ClientError {
    /// Writes the plain text of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            ClientError::NotStarted(error) => write!(formatter, "the client could not be started: {error}"),
            ClientError::Failed(text) | ClientError::TimedOut(text) => write!(formatter, "{text}"),
        };
    }
}

/// Builds the command that runs the client as the target user.
/// A root watcher drops through `setpriv`,
/// and an unprivileged watcher runs the client directly.
pub fn client_command(program: &str, uid: u32, gid: u32, arguments: &[String], as_root: bool) -> (String, Vec<String>) {
    if !as_root {
        return (program.to_string(), arguments.to_vec());
    }
    let mut full: Vec<String> = vec![
        "--reuid".to_string(),
        uid.to_string(),
        "--regid".to_string(),
        gid.to_string(),
        "--clear-groups".to_string(),
        "--no-new-privs".to_string(),
        program.to_string(),
    ];
    full.extend(arguments.iter().cloned());
    return ("setpriv".to_string(), full);
}

/// A command line client run as the watched user.
pub struct UserClient {
    /// Client executable.
    pub program: String,
    /// How long one run may take.
    pub timeout: Duration,
}

/// Construction and the run.
impl UserClient {
    /// Builds a client with the production timeout.
    pub fn new(program: &str) -> UserClient {
        return UserClient {
            program: program.to_string(),
            timeout: CLIENT_TIMEOUT,
        };
    }

    /// Runs the client as the target user against that user's manager bus,
    /// failing when it exits nonzero or runs past its timeout,
    /// and returns what it printed to standard output,
    /// at most `MAX_OUTPUT_BYTES` of it.
    /// The environment is cleared and holds only `PATH` and the user's runtime directory.
    pub fn run(&self, uid: u32, gid: u32, arguments: &[String]) -> Result<String, ClientError> {
        return self.run_limited(uid, gid, arguments, MAX_OUTPUT_BYTES);
    }

    /// Runs the client like `run` and keeps at most `max_output_bytes` of its standard output.
    /// The output is read while the client runs:
    /// reading it after the exit stalls a client that writes more than a pipe holds.
    pub fn run_limited(&self, uid: u32, gid: u32, arguments: &[String], max_output_bytes: u64) -> Result<String, ClientError> {
        return self.run_with_readers(uid, gid, arguments, max_output_bytes, &start_boxed_reader);
    }

    /// Runs the client like `run_limited` with the readers of its two streams started by `starter`,
    /// the seam where a test makes starting one fail.
    /// A client whose output or error stream has no reader would block on a full pipe until the timeout kills it,
    /// so a reader that cannot start kills and reaps the client at once,
    /// and so does a wait that fails.
    pub fn run_with_readers(
        &self,
        uid: u32,
        gid: u32,
        arguments: &[String],
        max_output_bytes: u64,
        starter: &dyn Fn(Box<dyn io::Read + Send>, u64) -> io::Result<JoinHandle<String>>,
    ) -> Result<String, ClientError> {
        // SAFETY: geteuid has no preconditions.
        let as_root = unsafe { libc::geteuid() } == 0;
        let (program, full_arguments) = client_command(&self.program, uid, gid, arguments, as_root);
        let mut command = Command::new(program);
        command
            .args(full_arguments)
            .env_clear()
            .env("PATH", SECURE_PATH)
            .env("XDG_RUNTIME_DIR", format!("/run/user/{uid}"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(ClientError::NotStarted)?;
        let out_reader = match child.stdout.take() {
            Some(stdout) => match starter(Box::new(stdout), max_output_bytes) {
                Ok(reader) => Some(reader),
                Err(error) => {
                    reap(&mut child);
                    return Err(ClientError::Failed(format!("reading the output of {} could not start: {error}", self.program)));
                }
            },
            None => None,
        };
        let err_reader = match child.stderr.take() {
            Some(stderr) => match starter(Box::new(stderr), MAX_OUTPUT_BYTES) {
                Ok(reader) => Some(reader),
                Err(error) => {
                    reap(&mut child);
                    return Err(ClientError::Failed(format!("reading the error output of {} could not start: {error}", self.program)));
                }
            },
            None => None,
        };
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = collected(out_reader);
                    let message = collected(err_reader);
                    if status.success() {
                        return Ok(output);
                    }
                    return Err(ClientError::Failed(format!("{} exited with {status}: {}", self.program, message.trim())));
                }
                Ok(None) => {}
                Err(error) => {
                    reap(&mut child);
                    return Err(ClientError::Failed(format!("waiting for {} failed: {error}", self.program)));
                }
            }
            if started.elapsed() > self.timeout {
                reap(&mut child);
                return Err(ClientError::TimedOut(format!("{} did not finish within {:?} and was killed", self.program, self.timeout)));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}
