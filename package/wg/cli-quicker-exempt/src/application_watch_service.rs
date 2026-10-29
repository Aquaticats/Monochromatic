//! Starts,
//!  validates,
//!  and stops detached application-cgroup watcher processes.

/// Watch loop preparation and execution.
use crate::application_watch::{prepare_application_watch, run_application_watch};
/// Production target roots and app slice path.
use crate::application_targets::{user_app_slice, ScanRoots};
/// Generic exact process validation and shutdown.
use crate::keeper_process::{
    command_process_is_live,
    process_start_time,
    stop_command_process,
};
/// Watcher state and log paths.
use crate::application_watch_state::{
    ensure_watch_state_directory,
    read_watch_state,
    remove_watch_state,
    watch_log_path,
    write_watch_state,
    WatchProcess,
};
/// Handshake and file errors.
use std::io::{self, BufRead, BufReader, Write};
/// Unix log creation mode.
use std::os::unix::fs::OpenOptionsExt;
/// Child process session setup hook.
use std::os::unix::process::CommandExt;
/// Watcher application slice path.
use std::path::Path;
/// Child process launch pipes.
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Child readiness after inotify installation and race-closing second scan.
const READY: &str = "READY";
/// Parent command after watcher state reaches durable storage.
const COMMIT: &str = "COMMIT";
/// Child acknowledgement before entering event loop.
const COMMITTED: &str = "COMMITTED";

/// Candidate watcher and handshake pipes.
struct WatchCandidate {
    /// Process handle used for startup failure collection.
    child: Child,
    /// Commit channel.
    input: ChildStdin,
    /// Readiness and commit acknowledgement channel.
    output: BufReader<ChildStdout>,
    /// Exact persisted process identity.
    process: WatchProcess,
}

/// Builds exact watcher command expected in procfs.
fn expected_watch_command(
    process: WatchProcess,
    key: &str,
) -> io::Result<Vec<Vec<u8>>> {
    let executable = std::env::current_exe()?;
    return Ok(vec![
        executable.as_os_str().as_encoded_bytes().to_vec(),
        b"__watch".to_vec(),
        key.as_bytes().to_vec(),
        process.mark.to_string().into_bytes(),
        process.uid.to_string().into_bytes(),
    ]);
}

/// Reads one newline-delimited watcher handshake message.
fn read_message(output: &mut BufReader<ChildStdout>) -> io::Result<String> {
    let mut line = String::new();
    let count = output.read_line(&mut line)?;
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "watcher closed handshake channel",
        ));
    }
    return Ok(line.trim_end().to_owned());
}

/// Stops persisted watcher only when complete command identity matches.
pub fn stop_application_watch(key: &str) -> io::Result<()> {
    let Some(process) = read_watch_state(key)? else {
        return Ok(());
    };
    let expected = expected_watch_command(process, key)?;
    if !command_process_is_live(
        process.pid,
        process.start_time,
        &expected,
        "application watcher",
    )? {
        return remove_watch_state(key);
    }
    stop_command_process(
        process.pid,
        process.start_time,
        &expected,
        "application watcher",
    )?;
    return remove_watch_state(key);
}

/// Reads watcher log after child failed before readiness.
fn read_watch_log(key: &str) -> io::Result<String> {
    let path = watch_log_path(key)?;
    match std::fs::read_to_string(path) {
        Ok(text) => return Ok(text),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(String::new()),
        Err(error) => return Err(error),
    }
}

/// Spawns detached watcher and waits for initial target coverage.
fn spawn_watch_candidate(
    key: &str,
    mark: u32,
    uid: u32,
) -> io::Result<WatchCandidate> {
    ensure_watch_state_directory(key)?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(watch_log_path(key)?)?;
    let executable = std::env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .arg("__watch")
        .arg(key)
        .arg(mark.to_string())
        .arg(uid.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log));
    // SAFETY: setsid is async-signal-safe and isolates watcher from invoking terminal.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    let mut child = command.spawn()?;
    let input = child.stdin.take().ok_or_else(|| return io::Error::other("watcher stdin absent"))?;
    let stdout = child.stdout.take().ok_or_else(|| return io::Error::other("watcher stdout absent"))?;
    let mut output = BufReader::new(stdout);
    let ready = read_message(&mut output);
    let ready_message = match ready {
        Ok(message) => message,
        Err(error) => {
            drop(input);
            let status = child.wait()?;
            let log_text = read_watch_log(key)?;
            return Err(io::Error::new(
                error.kind(),
                format!("application watcher failed before readiness ({status}): {}", log_text.trim()),
            ));
        }
    };
    if ready_message != READY {
        drop(input);
        let status = child.wait()?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected application watcher readiness ({status}): {ready_message}"),
        ));
    }
    let pid = child.id() as i32;
    return Ok(WatchCandidate {
        child,
        input,
        output,
        process: WatchProcess {
            pid,
            start_time: process_start_time(pid)?,
            mark,
            uid,
        },
    });
}

/// Starts fresh watcher after validated old watcher shutdown.
pub fn start_application_watch(
    key: &str,
    mark: u32,
    uid: u32,
) -> io::Result<()> {
    stop_application_watch(key)?;
    let mut candidate = spawn_watch_candidate(key, mark, uid)?;
    if let Err(error) = write_watch_state(key, candidate.process) {
        drop(candidate.input);
        let _wait_result = candidate.child.wait();
        return Err(error);
    }
    writeln!(candidate.input, "{COMMIT}")?;
    candidate.input.flush()?;
    let acknowledgement = read_message(&mut candidate.output)?;
    if acknowledgement != COMMITTED {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected application watcher commit: {acknowledgement}"),
        ));
    }
    return Ok(());
}

/// Runs hidden watcher child with production roots and commit-gated persistence.
pub fn run_application_watch_child(
    mark: u32,
    uid: u32,
    app_slice_override: Option<&Path>,
) -> io::Result<()> {
    let production_app_slice = user_app_slice(uid);
    let app_slice = app_slice_override.unwrap_or(&production_app_slice);
    let roots = ScanRoots {
        app_slice,
        proc_root: Path::new("/proc"),
        cgroup_root: Path::new("/sys/fs/cgroup"),
    };
    let mut watch = prepare_application_watch(roots, mark)?;
    println!("{READY}");
    io::stdout().flush()?;
    let mut commit = String::new();
    io::stdin().lock().read_line(&mut commit)?;
    if commit.trim_end() != COMMIT {
        return Ok(());
    }
    println!("{COMMITTED}");
    io::stdout().flush()?;
    return run_application_watch(&mut watch);
}
