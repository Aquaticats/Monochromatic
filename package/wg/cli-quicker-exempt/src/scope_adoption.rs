//! Moves processes into the updates slice by asking the user's systemd manager to adopt them.
//! The manager creates a transient scope under the slice and moves the listed pids into it,
//! so its own records stay true.
//! Writing to `cgroup.procs` directly would put processes into a cgroup the manager never created.
//! The manager would not garbage-collect it,
//! and its pid-to-unit map would go stale.
//! That breaks the single-writer rule of systemd's cgroup delegation document.
//! The manager can only move processes inside its own subtree,
//! `user@<uid>.service`,
//! so other processes are reported and left where they are.

/// Syscall and process failures.
use std::io;
/// Fake procfs roots in tests.
use std::path::{Path, PathBuf};
/// Child process control.
use std::process::{Command, Stdio};
/// Bounded waits.
use std::time::{Duration, Instant};

/// How long `busctl` may run before it is killed.
const BUSCTL_TIMEOUT: Duration = Duration::from_secs(5);
/// How long the adopted pid may take to appear in the slice after the job is queued.
const MOVE_CONFIRMATION_TIMEOUT: Duration = Duration::from_secs(2);
/// Pause between confirmation reads.
const POLL_INTERVAL: Duration = Duration::from_millis(10);

/// One adoption of a process group into a transient scope.
pub struct AdoptRequest<'a> {
    /// Processes to move,
    /// the matched process first.
    pub pids: &'a [i32],
    /// User whose manager adopts them.
    pub uid: u32,
    /// Group id used when dropping privilege for the bus call.
    pub gid: u32,
    /// Slice that receives the scope.
    pub slice: &'a str,
}

/// Seam between the policy and the user's service manager.
pub trait Adopter {
    /// Creates the slice when it does not exist.
    fn ensure_slice(&self, uid: u32, gid: u32, slice: &str, slice_directory: &Path) -> io::Result<()>;
    /// Moves the requested processes into a new scope below the slice.
    fn adopt(&self, request: &AdoptRequest<'_>) -> io::Result<()>;
}

/// Builds the `busctl` arguments for `StartTransientUnit` of a slice.
pub fn ensure_slice_arguments(slice: &str) -> Vec<String> {
    return vec![
        "--user".to_string(),
        "call".to_string(),
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
        "org.freedesktop.systemd1.Manager".to_string(),
        "StartTransientUnit".to_string(),
        "ssa(sv)a(sa(sv))".to_string(),
        slice.to_string(),
        "fail".to_string(),
        "1".to_string(),
        "Description".to_string(),
        "s".to_string(),
        "wg-quicker update traffic that bypasses the tunnel".to_string(),
        "0".to_string(),
    ];
}

/// Builds the `busctl` arguments for `StartTransientUnit` of a scope that adopts `pids`.
pub fn adopt_arguments(scope_name: &str, slice: &str, pids: &[i32]) -> Vec<String> {
    let mut arguments: Vec<String> = vec![
        "--user".to_string(),
        "call".to_string(),
        "org.freedesktop.systemd1".to_string(),
        "/org/freedesktop/systemd1".to_string(),
        "org.freedesktop.systemd1.Manager".to_string(),
        "StartTransientUnit".to_string(),
        "ssa(sv)a(sa(sv))".to_string(),
        scope_name.to_string(),
        "fail".to_string(),
        "3".to_string(),
        "Slice".to_string(),
        "s".to_string(),
        slice.to_string(),
        "PIDs".to_string(),
        "au".to_string(),
        pids.len().to_string(),
    ];
    for pid in pids {
        arguments.push(pid.to_string());
    }
    arguments.push("Description".to_string());
    arguments.push("s".to_string());
    arguments.push("wg-quicker update traffic that bypasses the tunnel".to_string());
    arguments.push("0".to_string());
    return arguments;
}

/// Builds the command that runs the client as the target user.
/// A root watcher drops to the user's real and effective ids and clears supplementary groups through `setpriv`,
/// so the bus call is made by the user and not by root.
/// An unprivileged watcher runs the client directly.
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

/// Adopts through the `busctl` command line client.
/// The marshalling of D-Bus values is the client's,
/// not hand-written wire code in a root process.
pub struct BusctlAdopter {
    /// Client executable.
    pub program: String,
    /// Procfs root used to confirm the move.
    pub proc_root: PathBuf,
    /// How long the moved pid may take to appear in the slice.
    pub confirmation_timeout: Duration,
}

/// Construction and the two client steps.
impl BusctlAdopter {
    /// Builds the production adopter.
    pub fn new() -> BusctlAdopter {
        return BusctlAdopter {
            program: "busctl".to_string(),
            proc_root: PathBuf::from("/proc"),
            confirmation_timeout: MOVE_CONFIRMATION_TIMEOUT,
        };
    }

    /// Runs the client as the target user against that user's manager bus,
    /// failing when it exits nonzero or runs past its timeout.
    fn run_client(&self, uid: u32, gid: u32, arguments: &[String]) -> io::Result<()> {
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
        let mut child = command.spawn()?;
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if status.success() {
                    return Ok(());
                }
                let mut message = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _read = io::Read::read_to_string(&mut stderr, &mut message);
                }
                return Err(io::Error::other(format!("{} exited with {status}: {}", self.program, message.trim())));
            }
            if started.elapsed() > BUSCTL_TIMEOUT {
                let _killed = child.kill();
                let _reaped = child.wait();
                return Err(io::Error::other(format!("{} did not finish within {BUSCTL_TIMEOUT:?}", self.program)));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// Waits until the pid's cgroup path names the slice.
    fn confirm_moved(&self, pid: i32, slice: &str) -> io::Result<()> {
        let path = self.proc_root.join(pid.to_string()).join("cgroup");
        let started = Instant::now();
        loop {
            if let Ok(text) = std::fs::read_to_string(&path)
                && text.contains(slice)
            {
                return Ok(());
            }
            if started.elapsed() > self.confirmation_timeout {
                return Err(io::Error::other(format!(
                    "the manager accepted the job but pid {pid} is not in {slice} after {:?}",
                    self.confirmation_timeout
                )));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}

/// The production seam to the user's manager.
impl Adopter for BusctlAdopter {
    /// Requests the slice through the client and waits for its cgroup directory.
    fn ensure_slice(&self, uid: u32, gid: u32, slice: &str, slice_directory: &Path) -> io::Result<()> {
        if slice_directory.is_dir() {
            return Ok(());
        }
        self.run_client(uid, gid, &ensure_slice_arguments(slice))?;
        let started = Instant::now();
        while !slice_directory.is_dir() {
            if started.elapsed() > self.confirmation_timeout {
                return Err(io::Error::other(format!("{slice} was requested but {} never appeared", slice_directory.display())));
            }
            std::thread::sleep(POLL_INTERVAL);
        }
        return Ok(());
    }

    /// Requests a scope that adopts the pids and waits until the first pid is in the slice.
    fn adopt(&self, request: &AdoptRequest<'_>) -> io::Result<()> {
        let Some(first) = request.pids.first() else {
            return Err(io::Error::other("no pid to adopt"));
        };
        let scope_name = format!("wg-quicker-update-{first}.scope");
        let arguments = adopt_arguments(&scope_name, request.slice, request.pids);
        self.run_client(request.uid, request.gid, &arguments)?;
        return self.confirm_moved(*first, request.slice);
    }
}
