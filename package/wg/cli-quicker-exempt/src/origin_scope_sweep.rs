//! The pass that stops the origin scopes no process holds,
//! which the watcher runs before its first adoption.
//! The scopes of one origin are shared by every watcher of the same user,
//! and the manager ends every process in a scope it stops,
//! so a batch of these stops holds the adoption gate across its last population read and its stop,
//! while every move into an origin scope holds that gate too.
//! A batch whose gate stays held is left for the next watcher start,
//! and the line says how many scopes that was and what was waited for.

/// The setup of a sweep and the listing of the manager's scopes.
use crate::leftover_sweep::{list_scopes, targets, ListedUnit, SweepSetup};
/// The two kinds of scope a listing is split into.
use crate::leftover_sweep::SweepTargets;
/// What a sweep did,
/// and the manager calls it makes.
use crate::leftover_sweep::{note_problem, user_arguments, SweepReport, STOP_BATCH};
/// The reader of a scope's population.
use crate::leftover_sweep::read_populated;
/// The lock an origin stop and a move share.
use crate::adoption_gate;
/// Where a scope lives in the cgroup filesystem.
use crate::scope_naming::scope_cgroup_directory;
/// The scopes of the real cgroup filesystem.
use std::path::Path;

/// Whether a batch of stops can meet a process another watcher is moving in.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum OriginGate {
    /// The scopes of the earlier design,
    /// which nothing fills any more,
    /// so the read and the stop need no exclusion.
    NotNeeded,
    /// The scopes of one origin,
    /// which every watcher of this user moves processes into.
    Needed,
}

/// Sends the stops of one design in batches,
/// reading each scope again right before its batch goes out.
/// A batch of origin scopes is read and stopped while this process holds the adoption gate,
/// so no move of any watcher of this user can land between the read and the stop.
/// A batch whose gate cannot be taken inside the wait is left for the next watcher start,
/// and the log line says how many scopes that was and what was waited for.
pub fn stop_batches(
    setup: &SweepSetup,
    empty: &[&ListedUnit],
    populated: &dyn Fn(&Path) -> Option<bool>,
    report: &mut SweepReport,
    what: &str,
    gate: OriginGate,
) -> usize {
    let mut stopped: usize = 0;
    for batch in empty.chunks(STOP_BATCH) {
        let _held = match gate {
            OriginGate::NotNeeded => None,
            OriginGate::Needed => match adoption_gate::acquire_within(&setup.gate_root, setup.uid, setup.gate_wait) {
                Ok(Some(held)) => Some(held),
                Ok(None) => {
                    report.origins_deferred = report.origins_deferred.saturating_add(batch.len());
                    note_problem(
                        report,
                        format!(
                            "{} {what} were left for the next watcher start because the adoption gate was still held after {:?}, so another watcher was moving a process into an origin scope",
                            batch.len(),
                            setup.gate_wait
                        ),
                    );
                    continue;
                }
                Err(error) => {
                    report.origins_deferred = report.origins_deferred.saturating_add(batch.len());
                    note_problem(report, format!("{} {what} were left alone because the adoption gate could not be taken: {error}", batch.len()));
                    continue;
                }
            },
        };
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

/// Stops the origin scopes that hold no process,
/// and only ever runs before the watcher adopts anything.
///
/// An origin scope is the scope the watcher itself fills:
/// a process joining one is a process the watcher just moved there,
/// and the manager stops a scope by ending every process in it.
/// A check of `populated 0` therefore says nothing once the adoption path is running,
/// because a process can join between that read and the stop request,
/// and the stop would end a live process.
/// Ordered before the first adoption,
/// no process of this watcher can be in a stopped scope,
/// and this pass is the only one that stops them.
pub fn stop_empty_origin_scopes(setup: &SweepSetup, populated: &dyn Fn(&Path) -> Option<bool>) -> SweepReport {
    let mut report = SweepReport::default();
    let units = match list_scopes(setup, &mut report) {
        Some(units) => units,
        None => return report,
    };
    let split = targets(setup, &units);
    report.origins_listed = split.origins.len();
    let empty_origins = collect_empty_origins(setup, &split, populated, &mut report);
    let stopped = stop_batches(setup, &empty_origins, populated, &mut report, "empty origin scopes", OriginGate::Needed);
    report.origins_stopped = report.origins_stopped.saturating_add(stopped);
    return report;
}

/// The origin scopes that are active and hold nothing.
fn collect_empty_origins<'a>(
    setup: &SweepSetup,
    split: &SweepTargets<'a>,
    populated: &dyn Fn(&Path) -> Option<bool>,
    report: &mut SweepReport,
) -> Vec<&'a ListedUnit> {
    let mut empty: Vec<&ListedUnit> = Vec::new();
    for unit in &split.origins {
        if unit.active != "active" {
            continue;
        }
        match populated(&scope_cgroup_directory(&setup.cgroup_root, setup.uid, &setup.slice, &unit.name)) {
            Some(false) => empty.push(unit),
            Some(true) => report.held += 1,
            None => report.unreadable += 1,
        }
    }
    return empty;
}

/// Stops the empty origin scopes of the real cgroup filesystem,
/// before the first adoption.
pub fn stop_empty_origin_scopes_now(setup: &SweepSetup) -> SweepReport {
    return stop_empty_origin_scopes(setup, &read_populated);
}
