//! The runs of the sweep,
//! and the calls each one makes on the user's manager:
//! listing the units,
//! resetting the failed ones,
//! and stopping the empty ones.

/// The units split into the two designs,
/// which only the test-only stop reaches.
#[cfg(test)]
use super::scope_names_and_listing::SweepTargets;
/// The units the manager listed,
/// and the reading of one plain line of it.
use super::scope_names_and_listing::{
    ListedUnit,
    parse_listing,
    targets,
};
/// What the sweep was set up with,
/// what it reports,
/// and the limits each call obeys.
use super::{
    LIST_OUTPUT_BYTES,
    MAX_PROBLEMS,
    OLD_SCOPE_SUFFIX,
    RESET_BATCH,
    SweepReport,
    SweepSetup,
};
/// The batch of stops,
/// and whether it must hold the adoption gate.
use crate::origin_scope_sweep::{
    OriginGate,
    stop_batches,
};
/// Where a scope lives in the cgroup filesystem.
use crate::scope_naming::scope_cgroup_directory;
/// Cgroup directories.
use std::path::Path;

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
pub fn note_problem(
    report: &mut SweepReport,
    problem: String,
) {
    if report.problems.len() < MAX_PROBLEMS {
        report.problems.push(problem);
    }
}

/// Builds the arguments of a `systemctl` call on the user's manager.
pub fn user_arguments(
    verb: &str,
    names: &[&str],
) -> Vec<String> {
    let mut arguments: Vec<String> = vec!["--user".to_string(), verb.to_string()];
    for name in names {
        arguments.push((*name).to_string());
    }
    return arguments;
}

/// Resets the failed units.
fn reset_failed(
    setup: &SweepSetup,
    units: &[&ListedUnit],
    report: &mut SweepReport,
) {
    let mut failed: Vec<&str> = Vec::new();
    for unit in units {
        if unit.active == "failed" {
            failed.push(unit.name.as_str());
        }
    }
    for chunk in failed.chunks(RESET_BATCH) {
        match setup
            .client
            .run(setup.uid, setup.gid, &user_arguments("reset-failed", chunk))
        {
            Ok(_output) => report.reset += chunk.len(),
            Err(error) => note_problem(
                report,
                format!("resetting {} failed units failed: {error}", chunk.len()),
            ),
        }
    }
}

/// Stops the running scopes that hold no process.
/// Each is read again right before its batch is sent,
/// and a scope that holds a process by then is left alone.
/// An origin scope that an earlier build named with a second `.scope` is stopped by the same rule,
/// so the older spelling does not leave a scope nothing can ever use again.
#[cfg(test)]
fn stop_empty(
    setup: &SweepSetup,
    targets: &SweepTargets<'_>,
    populated: &dyn Fn(&Path) -> Option<bool>,
    report: &mut SweepReport,
) {
    let mut empty: Vec<&ListedUnit> = Vec::new();
    for unit in &targets.per_process {
        if unit.active != "active" {
            continue;
        }
        match populated(&scope_cgroup_directory(
            &setup.cgroup_root,
            setup.uid,
            &setup.slice,
            &unit.name,
        )) {
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
        match populated(&scope_cgroup_directory(
            &setup.cgroup_root,
            setup.uid,
            &setup.slice,
            &unit.name,
        )) {
            Some(false) => empty_origins.push(unit),
            Some(true) => report.held += 1,
            None => report.unreadable += 1,
        }
    }
    let stopped = stop_batches(
        setup,
        &empty,
        populated,
        report,
        "empty scopes",
        OriginGate::NotNeeded,
    );
    report.stopped = report.stopped.saturating_add(stopped);
    let stopped_origins = stop_batches(
        setup,
        &empty_origins,
        populated,
        report,
        "empty origin scopes",
        OriginGate::Needed,
    );
    report.origins_stopped = report.origins_stopped.saturating_add(stopped_origins);
}

/// Lists the manager's scopes of this watcher,
/// or notes why it could not.
pub fn list_scopes(
    setup: &SweepSetup,
    report: &mut SweepReport,
) -> Option<Vec<ListedUnit>> {
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
    let listing =
        match setup
            .client
            .run_limited(setup.uid, setup.gid, &arguments, LIST_OUTPUT_BYTES)
        {
            Ok(text) => text,
            Err(error) => {
                note_problem(report, format!("listing the scopes failed: {error}"));
                return None;
            }
        };
    return Some(parse_listing(&listing));
}

/// Sweeps the scopes of the earlier design on a thread of its own,
/// and stops no origin scope:
/// those are stopped before the first adoption,
/// by `stop_empty_origin_scopes`.
pub fn sweep_leftover_per_process_scopes(
    setup: &SweepSetup,
    populated: &dyn Fn(&Path) -> Option<bool>,
) -> SweepReport {
    let mut report = SweepReport::default();
    let units = match list_scopes(setup, &mut report) {
        Some(units) => units,
        None => return report,
    };
    let split = targets(setup, &units);
    report.listed = split.per_process.len();
    report.origins_listed = split.origins.len();
    reset_failed(setup, &split.per_process, &mut report);
    let mut empty: Vec<&ListedUnit> = Vec::new();
    for unit in &split.per_process {
        if unit.active != "active" {
            continue;
        }
        match populated(&scope_cgroup_directory(
            &setup.cgroup_root,
            setup.uid,
            &setup.slice,
            &unit.name,
        )) {
            Some(false) => empty.push(unit),
            Some(true) => report.held += 1,
            None => report.unreadable += 1,
        }
    }
    let stopped = stop_batches(
        setup,
        &empty,
        populated,
        &mut report,
        "empty scopes",
        OriginGate::NotNeeded,
    );
    report.stopped = report.stopped.saturating_add(stopped);
    return report;
}

/// Sweeps with a given reader of the populated state,
/// the seam where a test makes a scope gain a process between the two reads.
#[cfg(test)]
pub fn sweep_with(
    setup: &SweepSetup,
    populated: &dyn Fn(&Path) -> Option<bool>,
) -> SweepReport {
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
    let listing =
        match setup
            .client
            .run_limited(setup.uid, setup.gid, &arguments, LIST_OUTPUT_BYTES)
        {
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

/// Sweeps the real cgroup filesystem,
/// both designs,
/// which is what a watcher did before the origin scopes were ordered before adoption.
#[cfg(test)]
pub fn sweep_leftover_scopes(setup: &SweepSetup) -> SweepReport {
    return sweep_with(setup, &read_populated);
}
