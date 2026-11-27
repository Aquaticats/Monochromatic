//! Starts,
//!  validates,
//!  and stops detached application-cgroup watcher processes.

/// The watcher child itself.
mod watch_child_run;
/// Starting and stopping the detached watcher.
mod watch_start;

/// The watcher child itself.
pub use watch_child_run::run_application_watch_child;
/// Starting and stopping the detached watcher.
pub use watch_start::{
    start_application_watch,
    stop_application_watch,
};

/// Watcher state and log paths.
use crate::application_watch_state::WatchProcess;
/// The user's toolchain roots.
use crate::install_roots::{
    InstallRoots,
    install_roots,
};
/// The account of the watched user.
use crate::user_account::{
    UserAccount,
    lookup_user,
};
/// Handshake and file errors.
use std::io::{
    self,
    BufReader,
};
/// Child process launch pipes.
use std::process::{
    Child,
    ChildStdin,
    ChildStdout,
};

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

/// Reads one handshake message,
/// but not for longer than `limit`:
/// the caller holds the lifecycle lock while it waits,
/// so a watcher that never speaks must not keep it.
fn read_message_within(
    output: &mut BufReader<ChildStdout>,
    limit: std::time::Duration,
    what: &str,
) -> io::Result<String> {
    return crate::line_ready::read_message_within(output, limit, what);
}

/// Waits for the candidate to end,
/// and ends it when it does not,
/// because a watcher whose parent is gone waits on a commit that never comes.
fn wait_within(
    child: &mut Child,
    limit: std::time::Duration,
) -> io::Result<std::process::ExitStatus> {
    return crate::line_ready::wait_within(child, limit);
}

/// Looks up the watched user and the toolchain install roots of that user.
fn user_install_roots(uid: u32) -> io::Result<(UserAccount, InstallRoots)> {
    let account = lookup_user(uid)?;
    let canonical_home = std::fs::canonicalize(&account.home).ok();
    let roots = install_roots(&account.home, canonical_home.as_deref());
    return Ok((account, roots));
}
