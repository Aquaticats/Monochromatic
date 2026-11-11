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
//! It runs on its own thread,
//! so the watcher enforces its policy while the manager works through the stops,
//! and logs its counts once.

/// The slice the scopes live in.
use crate::application_targets::TOOLCHAIN_UPDATES_SLICE;
/// The log.
use crate::log_line::{log_line, sanitize};
/// Where a scope lives in the cgroup filesystem.
use crate::scope_naming::scope_cgroup_directory;
/// The client run as the user.
use crate::user_client::UserClient;
/// Fake cgroup roots in tests.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::Duration;

/// Start of the name of a scope of the earlier design.
const OLD_SCOPE_PREFIX: &str = "wg-quicker-update-";
/// End of the name of a scope.
const OLD_SCOPE_SUFFIX: &str = ".scope";
/// Scopes named in one `stop` call.
const STOP_BATCH: usize = 25;
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
    pub fn production(uid: u32, gid: u32) -> SweepSetup {
        let mut client = UserClient::new("systemctl");
        client.timeout = SWEEP_CLIENT_TIMEOUT;
        return SweepSetup { uid, gid, slice: TOOLCHAIN_UPDATES_SLICE.to_string(), cgroup_root: PathBuf::from("/sys/fs/cgroup"), client, prefix: OLD_SCOPE_PREFIX.to_string() };
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

/// Reports an ASCII digit byte.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Reports a nonempty run of digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_digit_byte);
}

/// Reports whether a unit name is one of the earlier design:
/// the prefix,
/// two runs of digits joined by a dash,
/// and `.scope`.
/// The scopes of the current design are named from a path and never match.
pub fn is_old_style_scope_name(name: &str, prefix: &str) -> bool {
    let Some(after_prefix) = name.strip_prefix(prefix) else {
        return false;
    };
    let Some(rest) = after_prefix.strip_suffix(OLD_SCOPE_SUFFIX) else {
        return false;
    };
    let Some((pid, start_time)) = rest.split_once('-') else {
        return false;
    };
    return is_digits(pid) && is_digits(start_time);
}

/// Reports whether a unit name is a scope that holds the processes of one origin:
/// the prefix, a name made from a cgroup path, and `.scope`.
/// The names of the earlier design are two runs of digits, which a path never is,
/// so the two kinds never overlap.
pub fn is_origin_scope_name(name: &str, prefix: &str) -> bool {
    if is_old_style_scope_name(name, prefix) {
        return false;
    }
    let Some(rest) = name.strip_prefix(prefix) else {
        return false;
    };
    let Some(stem) = rest.strip_suffix(OLD_SCOPE_SUFFIX) else {
        return false;
    };
    return !stem.is_empty();
}

/// One unit of the manager's list.
struct ListedUnit {
    /// Unit name.
    name: String,
    /// Active state,
    /// such as `active` or `failed`.
    active: String,
}

/// Reads the manager's plain list:
/// the name,
/// the load state and the active state are the first three words of each line.
fn parse_listing(text: &str) -> Vec<ListedUnit> {
    let mut units: Vec<ListedUnit> = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let (Some(name), Some(_load), Some(active)) = (words.next(), words.next(), words.next()) else {
            continue;
        };
        units.push(ListedUnit { name: name.to_string(), active: active.to_string() });
    }
    return units;
}

/// Reads the populated state from a `cgroup.events` file of a cgroup directory:
/// `Some(true)` when it holds a process,
/// `Some(false)` when it holds none,
/// and `None` when the file cannot be read.
pub fn read_populated(directory: &Path) -> Option<bool> {
    let text = std::fs::read_to_string(directory.join("cgroup.events")).ok()?;
    for line in text.lines() {
        if let Some(value) = line.trim().strip_prefix("populated ") {
            return Some(value.trim() == "1");
        }
    }
    return None;
}

/// Records a problem unless enough are kept.
fn note_problem(report: &mut SweepReport, problem: String) {
    if report.problems.len() < MAX_PROBLEMS {
        report.problems.push(problem);
    }
}

/// Builds the arguments of a `systemctl` call on the user's manager.
fn user_arguments(verb: &str, names: &[&str]) -> Vec<String> {
    let mut arguments: Vec<String> = vec!["--user".to_string(), verb.to_string()];
    for name in names {
        arguments.push((*name).to_string());
    }
    return arguments;
}

/// Resets the failed units.
fn reset_failed(setup: &SweepSetup, units: &[&ListedUnit], report: &mut SweepReport) {
    let mut failed: Vec<&str> = Vec::new();
    for unit in units {
        if unit.active == "failed" {
            failed.push(unit.name.as_str());
        }
    }
    for chunk in failed.chunks(RESET_BATCH) {
        match setup.client.run(setup.uid, setup.gid, &user_arguments("reset-failed", chunk)) {
            Ok(_output) => report.reset += chunk.len(),
            Err(error) => note_problem(report, format!("resetting {} failed units failed: {error}", chunk.len())),
        }
    }
}

/// The scopes of the earlier design that the sweep may reset and stop,
/// and the origin scopes it may stop while they hold nothing.
struct SweepTargets<'a> {
    /// The scopes of the earlier design.
    per_process: Vec<&'a ListedUnit>,
    /// The scopes that hold the processes of one origin.
    origins: Vec<&'a ListedUnit>,
}

/// Splits the listed units into the two designs.
fn targets<'a>(setup: &SweepSetup, units: &'a [ListedUnit]) -> SweepTargets<'a> {
    let mut split = SweepTargets { per_process: Vec::new(), origins: Vec::new() };
    for unit in units {
        if is_old_style_scope_name(&unit.name, &setup.prefix) {
            split.per_process.push(unit);
        } else if is_origin_scope_name(&unit.name, &setup.prefix) {
            split.origins.push(unit);
        }
    }
    return split;
}

/// Stops the running scopes that hold no process.
/// Each is read again right before its batch is sent,
/// and a scope that holds a process by then is left alone.
/// An origin scope that an earlier build named with a second `.scope` is stopped by the same rule,
/// so the older spelling does not leave a scope nothing can ever use again.
fn stop_empty(setup: &SweepSetup, targets: &SweepTargets<'_>, populated: &dyn Fn(&Path) -> Option<bool>, report: &mut SweepReport) {
    let mut empty: Vec<&ListedUnit> = Vec::new();
    for unit in &targets.per_process {
        if unit.active != "active" {
            continue;
        }
        match populated(&scope_cgroup_directory(&setup.cgroup_root, setup.uid, &setup.slice, &unit.name)) {
            Some(false) => empty.push(unit),
            Some(true) => report.held += 1,
            None => report.unreadable += 1,
        }
    }
    let mut empty_origins: Vec<&ListedUnit> = Vec::new();
    for unit in &targets.origins {
        if unit.active != "active" {
            continue;
        }
        match populated(&scope_cgroup_directory(&setup.cgroup_root, setup.uid, &setup.slice, &unit.name)) {
            Some(false) => empty_origins.push(unit),
            Some(true) => report.held += 1,
            None => report.unreadable += 1,
        }
    }
    let stopped = stop_batches(setup, &empty, populated, report, "empty scopes");
    report.stopped = report.stopped.saturating_add(stopped);
    let stopped_origins = stop_batches(setup, &empty_origins, populated, report, "empty origin scopes");
    report.origins_stopped = report.origins_stopped.saturating_add(stopped_origins);
}

/// Sends the stops of one design in batches,
/// reading each scope again right before its batch goes out.
fn stop_batches(setup: &SweepSetup, empty: &[&ListedUnit], populated: &dyn Fn(&Path) -> Option<bool>, report: &mut SweepReport, what: &str) -> usize {
    let mut stopped: usize = 0;
    for batch in empty.chunks(STOP_BATCH) {
        let mut names: Vec<&str> = Vec::new();
        for unit in batch {
            match populated(&scope_cgroup_directory(&setup.cgroup_root, setup.uid, &setup.slice, &unit.name)) {
                Some(false) => names.push(unit.name.as_str()),
                Some(true) => report.held += 1,
                None => report.unreadable += 1,
            }
        }
        if names.is_empty() {
            continue;
        }
        match setup.client.run(setup.uid, setup.gid, &user_arguments("stop", &names)) {
            Ok(_output) => stopped = stopped.saturating_add(names.len()),
            Err(error) => note_problem(report, format!("stopping {} {what} failed: {error}", names.len())),
        }
    }
    return stopped;
}

/// Sweeps with a given reader of the populated state,
/// the seam where a test makes a scope gain a process between the two reads.
pub fn sweep_with(setup: &SweepSetup, populated: &dyn Fn(&Path) -> Option<bool>) -> SweepReport {
    let mut report = SweepReport::default();
    let arguments = vec![
        "--user".to_string(),
        "list-units".to_string(),
        "--all".to_string(),
        "--plain".to_string(),
        "--no-legend".to_string(),
        "--no-pager".to_string(),
        // The pattern narrows the manager's list; every name it returns is classified again below.
        format!("{}*{OLD_SCOPE_SUFFIX}", setup.prefix),
    ];
    let listing = match setup.client.run_limited(setup.uid, setup.gid, &arguments, LIST_OUTPUT_BYTES) {
        Ok(text) => text,
        Err(error) => {
            note_problem(&mut report, format!("listing the scopes failed: {error}"));
            return report;
        }
    };
    let units: Vec<ListedUnit> = parse_listing(&listing);
    let split = targets(setup, &units);
    report.listed = split.per_process.len();
    report.origins_listed = split.origins.len();
    reset_failed(setup, &split.per_process, &mut report);
    stop_empty(setup, &split, populated, &mut report);
    return report;
}

/// Sweeps the real cgroup filesystem.
pub fn sweep_leftover_scopes(setup: &SweepSetup) -> SweepReport {
    return sweep_with(setup, &read_populated);
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
        report.listed, report.reset, report.stopped, report.origins_listed, report.origins_stopped, report.held, report.unreadable
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
    let started = std::thread::Builder::new().name("leftover-sweep".to_string()).spawn(move || {
        if let Some(line) = describe_sweep(&sweep_leftover_scopes(&setup)) {
            log_line(&line);
        }
    });
    if let Err(error) = started {
        log_line(&format!("wg-quicker-exempt: the sweep of the leftover scopes could not start: {error}"));
    }
}
