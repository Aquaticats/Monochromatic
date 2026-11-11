//! Starts,
//!  validates,
//!  and stops detached application-cgroup watcher processes.

/// Watch loop preparation and execution.
use crate::application_watch::{prepare_application_watch, run_application_watch, ReporterIdentity};
/// The sweep of the scopes of the earlier design.
use crate::leftover_sweep::SweepSetup;
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
/// The relay of the start's loud lines.
use crate::startup_relay::forward_startup_failures;
/// The user's toolchain roots.
use crate::install_roots::{install_roots, InstallRoots};
/// Moves matched processes into the updates slice.
use crate::scope_adoption::PrivateSocketAdopter;
/// The watched user's account.
use crate::user_account::lookup_user;
/// Non-panicking output and escaping.
use crate::log_line::{log_line, sanitize, write_unchecked};
/// The sentinel of the watcher,
/// started outside the watcher's cgroup.
use crate::sentinel_launch::{launch_sentinel, SentinelRequest};
/// The real steps of starting a sentinel.
use crate::sentinel_system_steps::SystemSteps;
/// The policy engine and its setup.
use crate::update_routing::{RoutingSetup, UpdateRouting};
/// The start notice and the marker of a loud report.
use crate::update_report::{policy_failure, policy_notice, refused_root_failures};
/// The account of the watched user.
use crate::user_account::UserAccount;
/// Handshake and file errors.
use std::io::{self, BufRead, BufReader, Write};
/// The descriptor of the handshake channel.
use std::os::fd::AsRawFd;
/// Unix log creation mode.
use std::os::unix::fs::OpenOptionsExt;
/// Child process session setup hook.
use std::os::unix::process::CommandExt;
/// Watcher application slice path.
use std::path::{Path, PathBuf};
/// Child process launch pipes.
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Child readiness after inotify installation and race-closing second scan.
const READY: &str = "READY";
/// How long the watcher may take to report readiness or to commit.
/// The wait is bounded because `watch-start` holds the lifecycle lock while it waits:
/// a watcher that never says one of them would keep every other lifecycle command out for good.
const HANDSHAKE_LIMIT: std::time::Duration = std::time::Duration::from_secs(60);
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

/// Waits until `reader` can be read or `limit` passes,
/// and reports whether a line is there.
fn line_is_ready(reader: &BufReader<ChildStdout>, limit: std::time::Duration) -> io::Result<bool> {
    return crate::line_ready::line_is_ready(reader.get_ref().as_raw_fd(), limit);
}

/// Reads one handshake message,
/// but not for longer than `limit`:
/// the caller holds the lifecycle lock while it waits,
/// so a watcher that never speaks must not keep it.
fn read_message_within(output: &mut BufReader<ChildStdout>, limit: std::time::Duration, what: &str) -> io::Result<String> {
    if !line_is_ready(output, limit)? {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            format!("the watcher did not report {what} within {limit:?}"),
        ));
    }
    return read_message(output);
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

/// Size of the watcher log now,
/// zero when it does not exist yet.
fn watch_log_length(key: &str) -> u64 {
    return match watch_log_path(key).and_then(std::fs::metadata) {
        Ok(metadata) => metadata.len(),
        Err(_error) => 0,
    };
}

/// Looks up the watched user and the toolchain install roots of that user.
fn user_install_roots(uid: u32) -> io::Result<(UserAccount, InstallRoots)> {
    let account = lookup_user(uid)?;
    let canonical_home = std::fs::canonicalize(&account.home).ok();
    let roots = install_roots(&account.home, canonical_home.as_deref());
    return Ok((account, roots));
}

/// Starts a new session in the forked child so the watcher is isolated from the invoking terminal.
fn start_new_session() -> io::Result<()> {
    // SAFETY: `setsid` is async-signal-safe and touches only the child's session state.
    if unsafe { libc::setsid() } < 0 {
        return Err(io::Error::last_os_error());
    }
    return Ok(());
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
        command.pre_exec(start_new_session);
    }
    let mut child = command.spawn()?;
    let Some(input) = child.stdin.take() else {
        return Err(io::Error::other("watcher stdin absent"));
    };
    let Some(stdout) = child.stdout.take() else {
        return Err(io::Error::other("watcher stdout absent"));
    };
    let mut output = BufReader::new(stdout);
    let ready = read_message_within(&mut output, HANDSHAKE_LIMIT, "readiness");
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
    let (account, roots) = user_install_roots(uid)?;
    write_unchecked(&policy_notice(&roots.roots, &account.home));
    let log_offset = watch_log_length(key);
    let mut candidate = spawn_watch_candidate(key, mark, uid)?;
    if let Err(error) = write_watch_state(key, candidate.process) {
        drop(candidate.input);
        let _wait_result = candidate.child.wait();
        return Err(error);
    }
    writeln!(candidate.input, "{COMMIT}")?;
    candidate.input.flush()?;
    let acknowledgement = read_message_within(&mut candidate.output, HANDSHAKE_LIMIT, "its commit")?;
    if acknowledgement != COMMITTED {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected application watcher commit: {acknowledgement}"),
        ));
    }
    // The watcher is running and committed;
    // failing to relay its startup reports must not undo that,
    // but it must be said.
    if let Err(error) = forward_startup_failures(key, log_offset) {
        write_unchecked(&format!(
            "wg-quicker-exempt: the watcher started, but its startup reports could not be read from its log: {}",
            sanitize(&error.to_string())
        ));
    }
    match std::env::current_exe() {
        Ok(executable) => {
            let outcome = launch_sentinel(
                &SystemSteps,
                &SentinelRequest {
                    key,
                    watcher_pid: candidate.process.pid,
                    watcher_start_time: candidate.process.start_time,
                    uid: candidate.process.uid,
                    gid: account.gid,
                    executable: &executable,
                },
            );
            for report in &outcome.reports {
                write_unchecked(report);
            }
            write_unchecked(&outcome.summary());
        }
        Err(error) => write_unchecked(&policy_failure(&format!(
            "the sentinel that reports the watcher's own death could not start, because the path of this program is unknown ({error}); \
if the watcher is killed or runs out of memory nothing will say so"
        ))),
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
    let (account, install_root_set) = user_install_roots(uid)?;
    log_line(&policy_notice(&install_root_set.roots, &account.home));
    for line in refused_root_failures(&install_root_set.refused) {
        log_line(&line);
    }
    let routing = UpdateRouting::new(RoutingSetup {
        uid,
        install_roots: install_root_set.roots,
        proc_root: PathBuf::from("/proc"),
        cgroup_root: PathBuf::from("/sys/fs/cgroup"),
        adopter: Box::new(PrivateSocketAdopter::new()),
    });
    let mut watch = prepare_application_watch(roots, mark, routing, ReporterIdentity { uid, gid: account.gid }, Some(SweepSetup::production(uid, account.gid)))?;
    writeln!(io::stdout(), "{READY}")?;
    io::stdout().flush()?;
    let mut commit = String::new();
    io::stdin().lock().read_line(&mut commit)?;
    if commit.trim_end() != COMMIT {
        return Ok(());
    }
    writeln!(io::stdout(), "{COMMITTED}")?;
    io::stdout().flush()?;
    return run_application_watch(&mut watch);
}
