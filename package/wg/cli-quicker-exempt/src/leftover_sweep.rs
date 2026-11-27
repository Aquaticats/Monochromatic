//! Sweeps,
//! once when the watcher starts,
//! the scopes the earlier design left behind.
//! That design made a scope per moved process,
//! named `wg-quicker-update-<pid>-<start time>.scope`,
//! and a scope that adopted only exiting processes stayed running with no process in it,
//! or failed with no cgroup,
//! until the user's manager held thousands of them.
//! Nothing else removes them:
//! the manager stops a scope when its last process exits,
//! and a scope that never held one gets no such event.
//! The sweep resets the failed ones and stops the running ones whose cgroup holds no process.
//! It never touches a scope that holds a process,
//! rechecks each one right before the stop,
//! and never takes a scope that holds a process for a leftover.
//! The scopes that hold the processes of one origin are swept too,
//! and so are the ones an earlier build named with a second `.scope`:
//! the manager stops a scope when its last process exits,
//! but a scope that was created for a process that was already exiting stays empty for ever,
//! and a scope whose name this build no longer writes would otherwise never be found again.
//! A running scope of that design is only ever stopped while its cgroup holds no process,
//! which the manager cannot report for an active scope that adopted a process,
//! because the start job attaches the processes before the scope becomes active.
//! An origin scope can also receive a process from another watcher of the same user,
//! so every origin stop holds the adoption gate across its last population read and the stop,
//! and a sweep that cannot take the gate leaves those scopes for the next watcher start.
//! It runs on its own thread,
//! so the watcher enforces its policy while the manager works through the stops,
//! and logs its counts once.

/// The names of the scopes the sweep may touch,
/// and the manager's listing of them.
#[path = "leftover_sweep_scope_names_and_listing.rs"]
mod scope_names_and_listing;
/// The runs of the sweep and the manager calls they make.
#[path = "leftover_sweep_runs.rs"]
mod sweep_runs;

/// The test for a unit name of the earlier design,
/// which only a test asks about on its own.
#[cfg(test)]
pub use scope_names_and_listing::is_old_style_scope_name;
/// The names of a scope,
/// the manager's listing of them,
/// and the split of that listing into the two designs.
pub use scope_names_and_listing::{
    ListedUnit,
    SweepTargets,
    targets,
};
/// The runs of the sweep,
/// and the manager calls they make.
pub use sweep_runs::{
    list_scopes,
    note_problem,
    read_populated,
    sweep_leftover_per_process_scopes,
    user_arguments,
};
/// The test-only runs of the sweep,
/// which a test reaches through the populated-state seam.
#[cfg(test)]
pub use sweep_runs::{
    sweep_leftover_scopes,
    sweep_with,
};

/// The lock an adoption and an origin stop share.
use crate::adoption_gate;
/// The slice the scopes live in.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// The log.
use crate::log_line::{
    log_line,
    sanitize,
};
/// The client run as the user.
use crate::user_client::UserClient;
/// Cgroup directories.
use std::path::PathBuf;
/// Bounded waits.
use std::time::Duration;

/// Start of the name of a scope of the earlier design.
const OLD_SCOPE_PREFIX: &str = "wg-quicker-update-";
/// End of the name of a scope.
const OLD_SCOPE_SUFFIX: &str = ".scope";
/// Scopes named in one `stop` call.
pub const STOP_BATCH: usize = 25;
/// Failed units named in one `reset-failed` call.
const RESET_BATCH: usize = 100;
/// Most bytes read from the manager's list:
/// about 110 bytes a unit,
/// so a list of 100000 units fits.
const LIST_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;
/// How long one `systemctl` call may run:
/// stopping 25 scopes takes the manager a moment.
const SWEEP_CLIENT_TIMEOUT: Duration = Duration::from_secs(60);
/// Most problems kept for the log line.
const MAX_PROBLEMS: usize = 3;
/// How long an origin stop waits for the adoption gate:
/// the holder is one move,
/// whose manager call and confirmation end well inside this,
/// and a scope left unswept costs nothing while the next watcher start sweeps it.
const ORIGIN_STOP_GATE_WAIT: Duration = Duration::from_secs(5);

/// What the sweep needs.
pub struct SweepSetup {
    /// User whose manager holds the scopes.
    pub uid: u32,
    /// Group id used when dropping privilege.
    pub gid: u32,
    /// Slice that holds the scopes.
    pub slice: String,
    /// Cgroup filesystem root,
    /// where the populated state of a scope is read.
    pub cgroup_root: PathBuf,
    /// The `systemctl` client run as the user.
    pub client: UserClient,
    /// Directory the adoption gate file lives in.
    pub gate_root: PathBuf,
    /// How long one batch of origin stops waits for the adoption gate.
    pub gate_wait: Duration,
    /// Start of the names the sweep may touch.
    /// It is the earlier design's prefix in production,
    /// and another one in a test that runs against the real manager,
    /// which must never see a real scope name.
    pub prefix: String,
}

/// Construction.
impl SweepSetup {
    /// The production setup:
    /// the updates slice,
    /// the real cgroup filesystem and `systemctl` with a timeout that fits a batch of stops.
    pub fn production(
        uid: u32,
        gid: u32,
    ) -> SweepSetup {
        let mut client = UserClient::new("systemctl");
        client.timeout = SWEEP_CLIENT_TIMEOUT;
        return SweepSetup {
            uid,
            gid,
            slice: TOOLCHAIN_UPDATES_SLICE.to_string(),
            cgroup_root: PathBuf::from("/sys/fs/cgroup"),
            gate_root: PathBuf::from(adoption_gate::PRODUCTION_GATE_ROOT),
            gate_wait: ORIGIN_STOP_GATE_WAIT,
            client,
            prefix: OLD_SCOPE_PREFIX.to_string(),
        };
    }
}

/// What the sweep did.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct SweepReport {
    /// Scopes of the earlier design the manager listed.
    pub listed: usize,
    /// Scopes that hold the processes of one origin,
    /// current spelling or an earlier one,
    /// that the manager listed.
    pub origins_listed: usize,
    /// Failed units reset.
    pub reset: usize,
    /// Running scopes with no process that were stopped.
    pub stopped: usize,
    /// Running origin scopes with no process that were stopped.
    pub origins_stopped: usize,
    /// Empty origin scopes left alone because another watcher was moving a process just then.
    pub origins_deferred: usize,
    /// Running scopes that hold a process,
    /// left alone.
    pub held: usize,
    /// Running scopes whose populated state could not be read,
    /// left alone.
    pub unreadable: usize,
    /// What went wrong,
    /// the first few.
    pub problems: Vec<String>,
}

/// The line for the log,
/// or `None` when the manager held no scope of the earlier design and nothing went wrong.
pub fn describe_sweep(report: &SweepReport) -> Option<String> {
    if report.listed == 0
        && report.origins_listed == 0
        && report.stopped == 0
        && report.origins_stopped == 0
        && report.reset == 0
        && report.problems.is_empty()
    {
        return None;
    }
    let mut line = format!(
        "wg-quicker-exempt: swept the leftover scopes: {} of the earlier design listed, {} failed units reset, {} empty and stopped; \
{} scopes of one origin listed, {} of them empty and stopped; \
{} scopes that hold a process left alone (they end with their processes), {} left alone because their state could not be read",
        report.listed,
        report.reset,
        report.stopped,
        report.origins_listed,
        report.origins_stopped,
        report.held,
        report.unreadable
    );
    for problem in &report.problems {
        line.push_str("; ");
        line.push_str(&sanitize(problem));
    }
    return Some(line);
}

/// Starts the sweep on a thread of its own and logs its counts once when it ends.
/// The thread is created after the shutdown signals are blocked,
/// so it inherits the block like the rest of the watcher.
pub fn spawn_sweep(setup: SweepSetup) {
    let started = std::thread::Builder::new()
        .name("leftover-sweep".to_string())
        .spawn(move || {
            if let Some(line) =
                describe_sweep(&sweep_leftover_per_process_scopes(&setup, &read_populated))
            {
                log_line(&line);
            }
        });
    if let Err(error) = started {
        log_line(&format!(
            "wg-quicker-exempt: the sweep of the leftover scopes could not start: {error}"
        ));
    }
}
