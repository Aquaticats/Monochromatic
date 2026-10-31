//! What:
//!  Start the per-worktree server when none answers: the startup lock, stale-socket recovery and the detached spawn.
//! Why:
//!  Many clients may find no server at the same moment. An owner-only lock file, separate from the socket, elects one
//!  of them: only its holder probes the socket, removes a socket whose owner is dead (connection refused), and spawns
//!  the server; the others wait. The server runs detached from the client's terminal and standard streams.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function startServerIfNeeded(options: LaunchOptions): Promise<void>
//! ```

use crate::diagnostic::Diagnostic;
use crate::runtime_dir::lock_path;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What:
///  How to start a server: the program and its arguments.
///  `Vec<OsString>` is a list of operating-system strings (TS: `string[]`).
/// Why:
///  The library does not know the command line; the CLI crate supplies the hidden `serve` arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ServerLaunch = { program: string; args: string[] };
/// ```
#[derive(Clone, Debug)]
pub struct ServerLaunch {
    /// Executable to start, usually the current executable.
    pub program: PathBuf,
    /// Arguments that make it run the server.
    pub args: Vec<OsString>,
}

/// What:
///  The result of one attempt to start a server.
/// Why:
///  The caller loops: connect, start if needed, connect again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartOutcome = 'started' | 'someone-else-is-starting' | 'already-running';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartOutcome {
    /// This caller spawned a server that now accepts connections.
    Started,
    /// Another caller holds the lock; wait and connect again.
    Waiting,
    /// A server answered while the lock was held.
    AlreadyRunning,
}

/// What:
///  Make the child its own session leader, detached from the client's terminal.
///  `unsafe` marks code the compiler cannot check; here it is one call to `setsid`.
/// Why:
///  The server must outlive the terminal and must not receive its hangup or interrupt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(program, args, { detached: true, stdio: 'ignore' }).unref();
/// ```
fn detach_session(command: &mut Command) {
    // SAFETY: the closure runs in the forked child before `exec` and calls only `setsid`, which is async-signal-safe.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            return Ok(());
        });
    }
}

/// What:
///  Wait until the socket accepts a connection, failing if the server process exits first.
/// Why:
///  A server that dies on startup (for example a bind error) must be reported, not waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function waitForSocket(socket: string, child: ChildProcess, deadline: number): Promise<void>
/// ```
fn wait_for_socket(socket: &Path, child: &mut std::process::Child, deadline: Instant) -> Result<(), Diagnostic> {
    while Instant::now() < deadline {
        if UnixStream::connect(socket).is_ok() {
            return Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            let message = format!("the server process exited with {status} before it listened on {}", socket.display());
            return Err(Diagnostic::new("server-start-failed", &message));
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    let message = format!("the server did not listen on {} in time", socket.display());
    return Err(Diagnostic::new("server-start-failed", &message));
}

/// What:
///  Take the startup lock; start a server unless one answers.
/// Why:
///  Stale-socket recovery is verified: a socket file is removed only after a connection to it is refused, which
///  proves nothing is listening; any other failure is reported, not worked around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function startServerIfNeeded(socket: string, launch: ServerLaunch, deadline: number): Promise<StartOutcome>
/// ```
pub fn start_server_if_needed(socket: &Path, launch: &ServerLaunch, deadline: Instant) -> Result<StartOutcome, Diagnostic> {
    let lock = lock_path(socket);
    let file = match fs::OpenOptions::new().create(true).truncate(false).write(true).mode(0o600).open(&lock) {
        Ok(file) => file,
        Err(error) => {
            let message = format!("cannot open the startup lock {}: {error}", lock.display());
            return Err(Diagnostic::new("server-start-failed", &message));
        }
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(std::fs::TryLockError::WouldBlock) => return Ok(StartOutcome::Waiting),
        Err(std::fs::TryLockError::Error(error)) => {
            let message = format!("cannot take the startup lock {}: {error}", lock.display());
            return Err(Diagnostic::new("server-start-failed", &message));
        }
    }
    match UnixStream::connect(socket) {
        Ok(_) => return Ok(StartOutcome::AlreadyRunning),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            // Verified stale: the file exists but nothing listens behind it.
            let _ = fs::remove_file(socket);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            let message = format!("cannot probe the socket {}: {error}", socket.display());
            return Err(Diagnostic::new("server-start-failed", &message));
        }
    }
    let mut command = Command::new(&launch.program);
    command.args(&launch.args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    detach_session(&mut command);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let message = format!("cannot start the server {}: {error}", launch.program.display());
            return Err(Diagnostic::new("server-start-failed", &message));
        }
    };
    wait_for_socket(socket, &mut child, deadline)?;
    return Ok(StartOutcome::Started);
}
