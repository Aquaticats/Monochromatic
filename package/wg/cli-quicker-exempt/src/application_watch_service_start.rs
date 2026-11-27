//! Starting and stopping the detached watcher:
//! the candidate it spawns,
//! the handshake it readies,
//! and the stop that only a matching identity allows.

/// The candidate the start spawns,
/// its words,
/// the bounded helpers that read and wait,
/// and the account lookup both sides share.
use super::{
    COMMIT,
    COMMITTED,
    HANDSHAKE_LIMIT,
    READY,
    WatchCandidate,
    expected_watch_command,
    read_message_within,
    user_install_roots,
    wait_within,
};
/// Watcher state and log paths,
/// and the identity written to them.
use crate::application_watch_state::{
    WatchProcess,
    ensure_watch_state_directory,
    read_watch_state,
    remove_watch_state,
    watch_log_path,
    write_watch_state,
};
/// Generic exact process validation and shutdown,
/// including the group a killed watcher left.
use crate::keeper_process::{
    command_process_is_live,
    end_leftover_group,
    process_start_time,
    stop_command_process,
};
/// Non-panicking output and escaping.
use crate::log_line::{
    log_line,
    sanitize,
    write_unchecked,
};
/// The sentinel of the watcher,
/// started outside the watcher's cgroup.
use crate::sentinel_launch::{
    SentinelRequest,
    launch_sentinel,
};
/// The real steps of starting a sentinel.
use crate::sentinel_system_steps::SystemSteps;
/// The relay of the start's loud lines.
use crate::startup_relay::forward_startup_failures;
/// The start notice and the marker of a loud report.
use crate::update_report::{
    policy_failure,
    policy_notice,
};
/// Handshake and file errors.
use std::io::{
    self,
    BufReader,
    Write,
};
/// Unix log creation mode.
use std::os::unix::fs::OpenOptionsExt;
/// Child process session setup hook.
use std::os::unix::process::CommandExt;
/// Child process launch pipes.
use std::process::{
    Command,
    Stdio,
};

/// Stops persisted watcher only when complete command identity matches.
pub fn stop_application_watch(key: &str) -> io::Result<()> {
    let Some(process) = read_watch_state(key)? else {
        return Ok(());
    };
    let expected = expected_watch_command(process, key)?;
    if command_process_is_live(
        process.pid,
        process.start_time,
        &expected,
        "application watcher",
    )? {
        stop_command_process(
            process.pid,
            process.start_time,
            &expected,
            "application watcher",
        )?;
        return remove_watch_state(key);
    }
    // The leader is gone,
    // and the children it started in its group need not be:
    // a manager client left alive can still change the user's manager after this stop returned,
    // so the group is ended before the record is forgotten.
    // A group that survives keeps the record, so the next stop tries again.
    let group = end_leftover_group(process.pid, process.start_time, "application watcher")?;
    if group == crate::keeper_process::LeftoverGroup::Ended {
        log_line(&format!(
            "wg-quicker-exempt: the watcher of {key} was gone and its process group still held a member, so the group was ended before its record was removed"
        ));
    }
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
            let status = wait_within(&mut child, HANDSHAKE_LIMIT)?;
            let log_text = read_watch_log(key)?;
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "application watcher failed before readiness ({status}): {}",
                    log_text.trim()
                ),
            ));
        }
    };
    if ready_message != READY {
        drop(input);
        let status = wait_within(&mut child, HANDSHAKE_LIMIT)?;
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
        let _wait_result = wait_within(&mut candidate.child, HANDSHAKE_LIMIT);
        return Err(error);
    }
    writeln!(candidate.input, "{COMMIT}")?;
    candidate.input.flush()?;
    let acknowledgement =
        read_message_within(&mut candidate.output, HANDSHAKE_LIMIT, "its commit")?;
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
