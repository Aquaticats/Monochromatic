//! Provides crash-recoverable link-FD persistence when affected kernels cannot pin bpffs objects.

/// Creates unpinned links retained by holder process.
use crate::bpf::attach_marker_unpinned;
/// Validates and terminates only exact holder process identities.
use crate::keeper_process::{keeper_is_live, process_start_time, stop_process};
/// Transition state and commit-marker persistence.
use crate::keeper_state::{
    active_state,
    commit_marker_exists,
    read_state,
    remove_commit_marker,
    remove_state,
    replacing_state,
    write_commit_marker,
    write_state,
    KeeperProcess,
};
/// Canonical or stable lexical cgroup identity.
use crate::pin::cgroup_identity_path;
/// Buffered handshake input and output.
use std::io::{self, BufRead, BufReader, Read, Write};
/// Raw descriptor access for cgroup file.
use std::os::fd::AsRawFd;
/// Child setup hook for detached session.
use std::os::unix::process::CommandExt;
/// Cgroup filesystem paths.
use std::path::{Path, PathBuf};
/// Holder process and pipe handles during replacement handshake.
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Exact readiness line sent after all four unpinned links exist.
const READY: &str = "READY";
/// Parent commit command sent only after transition state is durable.
const COMMIT: &str = "COMMIT";
/// Exact acknowledgement sent after child commit marker is durable.
const COMMITTED: &str = "COMMITTED";
/// Debug-build test seam forcing candidate failure after all links attach.
#[cfg(debug_assertions)]
const FAIL_AFTER_ATTACH_ENV: &str = "WG_QUICKER_EXEMPT_TEST_FAIL_AFTER_ATTACH";

/// Candidate child plus pipes retained until two-phase commit completes.
struct Candidate {
    /// Process handle used for startup error collection.
    child: Child,
    /// Parent-to-child commit channel.
    input: ChildStdin,
    /// Child-to-parent readiness and commit acknowledgements.
    output: BufReader<ChildStdout>,
    /// Persisted identity used by recovery and validation.
    process: KeeperProcess,
}

/// Reads one newline-delimited child handshake message.
fn read_message(output: &mut BufReader<ChildStdout>) -> io::Result<String> {
    let mut line = String::new();
    let count = output.read_line(&mut line)?;
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "keeper closed handshake channel",
        ));
    }
    return Ok(line.trim_end().to_owned());
}

/// Starts candidate in a detached session and waits until all links exist.
fn spawn_candidate(mark: u32, cgroup_dir: &Path) -> io::Result<Candidate> {
    let executable = std::env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .arg("__hold")
        .arg(mark.to_string())
        .arg(cgroup_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // SAFETY: `setsid` is async-signal-safe and touches only child process session state.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        });
    }
    let mut child = command.spawn()?;
    let input = child.stdin.take().ok_or_else(|| return io::Error::other("keeper stdin pipe absent"))?;
    let stdout = child.stdout.take().ok_or_else(|| return io::Error::other("keeper stdout pipe absent"))?;
    let mut output = BufReader::new(stdout);
    let ready_result = read_message(&mut output);
    let ready = match ready_result {
        Ok(message) => message,
        Err(error) => {
            drop(input);
            let status = child.wait()?;
            let mut stderr = String::new();
            if let Some(mut stream) = child.stderr.take() {
                stream.read_to_string(&mut stderr)?;
            }
            return Err(io::Error::new(
                error.kind(),
                format!("keeper failed before readiness ({status}): {}", stderr.trim()),
            ));
        }
    };
    if ready != READY {
        drop(input);
        let status = child.wait()?;
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected keeper readiness message ({status}): {ready}"),
        ));
    }
    let pid = child.id() as i32;
    let process = KeeperProcess {
        pid,
        start_time: process_start_time(pid)?,
        mark,
    };
    return Ok(Candidate {
        child,
        input,
        output,
        process,
    });
}

/// Settles interrupted transition into one active process or no state.
fn recover(cgroup_dir: &Path) -> io::Result<Option<KeeperProcess>> {
    let Some(state) = read_state(cgroup_dir)? else {
        return Ok(None);
    };
    let active_live = if let Some(active) = state.active {
        keeper_is_live(active, cgroup_dir)?
    } else {
        false
    };
    let Some(candidate) = state.candidate else {
        if let Some(active) = state.active {
            remove_commit_marker(cgroup_dir, active)?;
        }
        if active_live {
            return Ok(state.active);
        }
        remove_state(cgroup_dir)?;
        return Ok(None);
    };
    let candidate_live = keeper_is_live(candidate, cgroup_dir)?;
    let committed = commit_marker_exists(cgroup_dir, candidate)?;
    if candidate_live && committed {
        if active_live {
            let active = state.active.ok_or_else(|| return io::Error::other("live active keeper absent"))?;
            stop_process(active, cgroup_dir)?;
        }
        write_state(cgroup_dir, &active_state(candidate))?;
        remove_commit_marker(cgroup_dir, candidate)?;
        return Ok(Some(candidate));
    }
    if candidate_live {
        stop_process(candidate, cgroup_dir)?;
    }
    remove_commit_marker(cgroup_dir, candidate)?;
    if active_live {
        let active = state.active.ok_or_else(|| return io::Error::other("live active keeper absent"))?;
        write_state(cgroup_dir, &active_state(active))?;
        return Ok(Some(active));
    }
    remove_state(cgroup_dir)?;
    return Ok(None);
}

/// Replaces holder only after new links and durable transition state exist.
pub fn replace_keeper(mark: u32, cgroup_dir: &Path) -> io::Result<()> {
    let canonical = cgroup_identity_path(cgroup_dir)?;
    let active = recover(&canonical)?;
    let mut candidate = spawn_candidate(mark, &canonical)?;
    // Candidate exits and drops links on stdin EOF until durable transition earns COMMIT.
    let transition = replacing_state(active, candidate.process);
    if let Err(error) = write_state(&canonical, &transition) {
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
            format!("unexpected keeper acknowledgement: {acknowledgement}"),
        ));
    }
    if let Some(previous) = active {
        stop_process(previous, &canonical)?;
    }
    write_state(&canonical, &active_state(candidate.process))?;
    remove_commit_marker(&canonical, candidate.process)?;
    return Ok(());
}

/// Stops recovered active holder and removes state only after disappearance.
pub fn detach_keeper(cgroup_dir: &Path) -> io::Result<()> {
    let identity = cgroup_identity_path(cgroup_dir)?;
    let Some(active) = recover(&identity)? else {
        return Ok(());
    };
    stop_process(active, &identity)?;
    remove_commit_marker(&identity, active)?;
    return remove_state(&identity);
}

/// Child entry loads all links,
///  waits for durable parent transition,
///  then holds descriptors.
pub fn run_holder(mark: u32, cgroup_dir: &Path) -> io::Result<()> {
    let identity: PathBuf = cgroup_identity_path(cgroup_dir)?;
    let cgroup = std::fs::File::open(&identity)?;
    let _links = attach_marker_unpinned(cgroup.as_raw_fd(), mark)?;
    #[cfg(debug_assertions)]
    if std::env::var_os(FAIL_AFTER_ATTACH_ENV).is_some() {
        return Err(io::Error::other("injected holder failure after complete attach"));
    }
    println!("{READY}");
    io::stdout().flush()?;
    let mut command = String::new();
    io::stdin().lock().read_line(&mut command)?;
    if command.trim_end() != COMMIT {
        return Ok(());
    }
    let process = KeeperProcess {
        pid: std::process::id() as i32,
        start_time: process_start_time(std::process::id() as i32)?,
        mark,
    };
    write_commit_marker(&identity, process)?;
    println!("{COMMITTED}");
    io::stdout().flush()?;
    loop {
        // SAFETY: pause blocks this descriptor owner until termination signal closes all links.
        unsafe { libc::pause() };
    }
}
