//! Runs a command line client as the watched user against that user's session bus.
//! A root watcher drops to the user's real and effective ids and clears supplementary groups through `setpriv`,
//! so the bus call is made by the user and not by root.
//! The client gets a cleared environment,
//! and is killed when it outlives its timeout,
//! so one hung bus cannot stall the watcher.

/// Syscall and process failures.
use std::io;
/// Child process control.
use std::process::{Command, Stdio};
/// Bounded waits.
use std::time::{Duration, Instant};

/// How long a client may run before it is killed.
pub const CLIENT_TIMEOUT: Duration = Duration::from_secs(3);
/// Pause between exit checks.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

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
    /// failing when it exits nonzero or runs past its timeout.
    /// The environment is cleared and holds only `PATH` and the user's runtime directory.
    pub fn run(&self, uid: u32, gid: u32, arguments: &[String]) -> Result<(), ClientError> {
        // SAFETY: geteuid has no preconditions.
        let as_root = unsafe { libc::geteuid() } == 0;
        let (program, full_arguments) = client_command(&self.program, uid, gid, arguments, as_root);
        let mut command = Command::new(program);
        command
            .args(full_arguments)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("XDG_RUNTIME_DIR", format!("/run/user/{uid}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        let mut child = command.spawn().map_err(ClientError::NotStarted)?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if status.success() {
                        return Ok(());
                    }
                    let mut message = String::new();
                    if let Some(mut stderr) = child.stderr.take() {
                        let _read = io::Read::read_to_string(&mut stderr, &mut message);
                    }
                    return Err(ClientError::Failed(format!("{} exited with {status}: {}", self.program, message.trim())));
                }
                Ok(None) => {}
                Err(error) => return Err(ClientError::Failed(format!("waiting for {} failed: {error}", self.program))),
            }
            if started.elapsed() > self.timeout {
                let _killed = child.kill();
                let _reaped = child.wait();
                return Err(ClientError::TimedOut(format!("{} did not finish within {:?} and was killed", self.program, self.timeout)));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}
