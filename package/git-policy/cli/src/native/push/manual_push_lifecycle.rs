//! What:
//!  The manual-push candidate scope of one `git push`:
//! the commits and trees the destination lacks.
//! Why:
//!  The manual-push policies (`final-newline`,
//! `forbidden-strings`) scan the content a push publishes.
//!  The probe discovers the updates,
//! bounds each by what its
//!  destination already has,
//! and hands the scan to the policy engine;
//! a failure stops the push with exit `2`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const updates = await probeManualPushUpdates({ gitPath, cwd, args });
//! ```

/// The probe,
/// the published set and the range,
/// and the candidate request they feed.
use crate::auto_push_single_flight::PushGit;
use crate::candidate_object::parse_object_id;
use crate::candidate_prediction::PushedScope;
use crate::manual_push_cause::ManualPushCause;
use crate::manual_push_probe::{GitRefusal, MOST_CONCURRENT_LOCAL_QUERIES, PushUpdate, QUERY_MEMORY, probe_manual_push_updates};
use crate::parallel_budget::{BudgetRequest, HostResources, worker_budget};
use crate::line_sink::LineSink;
use crate::parallel_runner::{Event, OrderedEvents, OrderedRun, OrderedWork, run_ordered};
use crate::manual_push_range::{
    ManualPushRangeError, UpdateScope, peel_commits, published_tips, tracking_commits, update_scope,
};
/// Maps keyed by remote name and by object name,
/// and the sets that answer whether a commit or tree is already listed.
use std::collections::{BTreeMap, HashSet};
/// Owned OS strings for the replayed global options and the post-subcommand arguments.
use std::ffi::OsString;

/// What:
///  Why a push's scope could not be established.
/// Why:
///   The lifecycle reports every such failure as a content-unavailable engine failure and stops the push.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ManualPushProbeError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualPushFailure {
    /// Diagnostic naming the failed step.
    pub message: String,
    /// Git's refusal when Git itself ended the dry run:
    /// the push then ends as Git ends it.
    pub refusal: Option<GitRefusal>,
    /// What the person can do about it,
    /// which the engine failure's hint is chosen by.
    pub cause: ManualPushCause,
}

impl ManualPushFailure {
    /// A failure of the wrapper's own step,
    /// which is no refusal by Git.
    pub fn step(cause: ManualPushCause, message: String) -> Self {
        return Self { message, refusal: None, cause };
    }
}

impl std::fmt::Display for ManualPushFailure {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

impl std::error::Error for ManualPushFailure {}

/// Converts a probe failure into the lifecycle failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caughtValueText(error)
/// ```
fn from_probe(error: &crate::manual_push_probe::ManualPushProbeError) -> ManualPushFailure {
    return ManualPushFailure {
        message: error.message.clone(),
        refusal: error.refusal.clone(),
        cause: error.cause,
    };
}

/// Converts a range failure into the lifecycle failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caughtValueText(error)
/// ```
fn from_range(error: &ManualPushRangeError) -> ManualPushFailure {
    return ManualPushFailure::step(error.cause, error.message.clone());
}

/// What:
///  The scope of every update of one push,
/// in the order Git negotiated the updates.
/// Why:
///   Each update is bounded by the published tips of its own destination remote only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function manualPushScopes({ gitPath, cwd, args }): Promise<UpdateScope[]>;
/// ```
pub fn scopes_of_updates(
    git: &PushGit,
    updates: &[PushUpdate],
) -> Result<Vec<UpdateScope>, ManualPushFailure> {
    return scopes_of_updates_on(&HostResources::read(), git, updates);
}

/// What:
///  The scope of one update,
/// run by a worker of the ordered runner.
/// Why:
///   The Git context and the published tips are what each update reads;
/// the runner shares them among its worker threads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScopeOfEachUpdate implements OrderedWork<PushUpdate, Result<UpdateScope>> { run(i, update) { return updateScope(git, update, tips.get(update.remoteName) ?? []); } }
/// ```
struct ScopeOfEachUpdate<'a> {
    /// The coordinator's Git,
    /// whose queries are bounded.
    git: &'a PushGit,
    /// The tips already published at each destination,
    /// by remote name.
    tips: &'a BTreeMap<String, Vec<String>>,
}

impl OrderedWork<PushUpdate, Result<UpdateScope, ManualPushRangeError>> for ScopeOfEachUpdate<'_> {
    fn run(&self, _index: usize, update: &PushUpdate, _say: &mut dyn LineSink) -> Result<UpdateScope, ManualPushRangeError> {
        let published: Vec<String> = self.tips.get(&update.remote_name).cloned().unwrap_or_default();
        return update_scope(self.git, update, &published);
    }
}

/// What:
///  Keeps the scopes in update order and the first failure;
/// once an update has failed,
/// later results are dropped.
/// Why:
///   The runner reports on the calling thread only and in update order,
/// so the first update that fails decides the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class KeepScopesUntilTheFirstFailure implements OrderedEvents<Result<UpdateScope>> { scopes: UpdateScope[]; failure?: Error }
/// ```
struct KeepScopesUntilTheFirstFailure<'a> {
    /// The scopes so far.
    scopes: &'a mut Vec<UpdateScope>,
    /// The first failure,
    /// when an update has failed.
    failure: &'a mut Option<ManualPushRangeError>,
}

impl OrderedEvents<Result<UpdateScope, ManualPushRangeError>> for KeepScopesUntilTheFirstFailure<'_> {
    fn on_event(&mut self, event: Event<'_, Result<UpdateScope, ManualPushRangeError>>) {
        if let Event::End(_index, outcome) = event {
            match outcome {
                Ok(scope) if self.failure.is_none() => self.scopes.push(scope),
                Err(error) if self.failure.is_none() => *self.failure = Some(error),
                Ok(_) | Err(_) => {}
            }
        }
    }
}

/// `scopes_of_updates` with the host's resources chosen by the caller,
/// so a control states a host of any size.
pub fn scopes_of_updates_on(
    host: &HostResources,
    git: &PushGit,
    updates: &[PushUpdate],
) -> Result<Vec<UpdateScope>, ManualPushFailure> {
    let mut remote_names: Vec<String> = Vec::new();
    for update in updates {
        if !remote_names.contains(&update.remote_name) {
            remote_names.push(update.remote_name.clone());
        }
    }
    let tracking: BTreeMap<String, Vec<String>> = match tracking_commits(git, &remote_names) {
        Ok(found) => found,
        Err(error) => return Err(from_range(&error)),
    };
    let mut prior_oids: Vec<String> = Vec::new();
    for update in updates {
        if let Some(oid) = update.remote_oid.clone() {
            prior_oids.push(oid);
        }
    }
    let prior: BTreeMap<String, String> = match peel_commits(git, &prior_oids) {
        Ok(found) => found,
        Err(error) => return Err(from_range(&error)),
    };
    let tips: BTreeMap<String, Vec<String>> = published_tips(updates, &prior, &tracking);
    // Each update is bounded by its own destination's tips and read by its own `cat-file` and `rev-list`, so the updates are
    //  independent reads and run side by side, at most `MOST_CONCURRENT_LOCAL_QUERIES` at once; the scopes come back in the order Git
    //  negotiated the updates, and the first update that fails decides the error.
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for index in 0..updates.len() {
        lanes.push(vec![index]);
    }
    let workers: usize = worker_budget(&BudgetRequest { items: updates.len(), per_item_memory: QUERY_MEMORY, ceiling: Some(MOST_CONCURRENT_LOCAL_QUERIES), cap: None, host });
    let mut scopes: Vec<UpdateScope> = Vec::new();
    let mut failure: Option<ManualPushRangeError> = None;
    run_ordered(OrderedRun {
        items: updates,
        lanes: lanes.as_slice(),
        workers,
        work: &ScopeOfEachUpdate { git, tips: &tips },
        on_event: &mut KeepScopesUntilTheFirstFailure { scopes: &mut scopes, failure: &mut failure },
    });
    return match failure {
        Some(error) => Err(from_range(&error)),
        None => Ok(scopes),
    };
}

/// What:
///  Collect the commits and trees the candidate layer must list for a set of scopes.
/// Why:
///   Each commit is listed once even when several updates publish it,
/// so the scan never repeats a delta.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateTargets(scopes: UpdateScope[]): { commits: string[]; trees: string[] };
/// ```
pub fn pushed_scope(scopes: &[UpdateScope]) -> Result<PushedScope, ManualPushFailure> {
    let mut commits: Vec<crate::candidate_object::ObjectId> = Vec::new();
    // A set answers "is this one already listed" by naming the one collected name the
    //  candidate is compared with, so a push that publishes many commits stays one pass
    //  (`AGENTS.md` `RG2`); the list beside it keeps first-seen order.
    let mut listed_commits: HashSet<crate::candidate_object::ObjectId> = HashSet::new();
    let mut trees: Vec<crate::candidate_object::ObjectId> = Vec::new();
    let mut listed_trees: HashSet<crate::candidate_object::ObjectId> = HashSet::new();
    for scope in scopes {
        match scope {
            UpdateScope::None => {}
            UpdateScope::Commits(names) => {
                for name in names {
                    let Some(oid) = parse_object_id(name.as_bytes()) else {
                        return Err(ManualPushFailure::step(ManualPushCause::UnexpectedOutput, format!(
                            "Git named a commit that is not an object name: {name}"
                        )));
                    };
                    crate::collected_elements_looked_at::note_looked_at(1);
                    if listed_commits.insert(oid.clone()) {
                        commits.push(oid);
                    }
                }
            }
            UpdateScope::FinalTree(tip) => {
                let Some(oid) = parse_object_id(tip.as_bytes()) else {
                    return Err(ManualPushFailure::step(ManualPushCause::UnexpectedOutput, format!("Git named a commit that is not an object name: {tip}")));
                };
                crate::collected_elements_looked_at::note_looked_at(1);
                if listed_trees.insert(oid.clone()) {
                    trees.push(oid);
                }
            }
        }
    }
    return Ok(PushedScope { commits, trees });
}

/// What:
///  Discover the updates of one push and the candidate scope they publish.
/// Why:
///   The whole chain runs before any policy reads content,
/// so a failure in any step refuses the push
///  with its diagnostic and no partial scan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const updates = await probeManualPushUpdates({ gitPath, cwd, args });
/// const scopes = await manualPushScopes(updates);
/// ```
pub fn manual_push_scope(
    git: &PushGit,
    global_prefix: &[OsString],
    post_subcommand: &[OsString],
) -> Result<PushedScope, ManualPushFailure> {
    let updates = match probe_manual_push_updates(git, global_prefix, post_subcommand) {
        Ok(found) => found,
        Err(error) => return Err(from_probe(&error)),
    };
    let scopes = scopes_of_updates(git, &updates)?;
    return pushed_scope(&scopes);
}

/// The lifecycle's scope is controlled in the test module.
#[cfg(test)]
#[path = "manual_push_lifecycle_tests.rs"]
mod tests;

/// The controls for a push's destinations and updates read side by side stay out of the release executable.
#[cfg(test)]
#[path = "manual_push_parallel_tests.rs"]
mod parallel_tests;
