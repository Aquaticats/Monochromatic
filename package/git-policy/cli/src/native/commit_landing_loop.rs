//! What:
//!  The landing loop: land, and after each lost race replay and revalidate outside both locks, until the commit
//!  lands, its replay conflicts, revalidation rejects it, or the target cannot take it.
//! Why:
//!  Each replay rebuilds the prepared commit from the preparation base onto the target that won, so every replay
//!  starts from the same prepared change. After `landing.reserveAfterLostRaces` lost races the transaction takes the
//!  landing reservation before it replays, and every landing attempt yields to another transaction's live
//!  reservation; the reservation is released when the loop ends (`src/policy-engine/commit-landing-loop.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await landWithReplay({ context, mode, prepared, settled, options, committedPaths, ... });
//! ```

use super::capture_invocation::InvocationCapture;
use super::commit_facts::PrivateIndexFacts;
use super::commit_landing_lock::{acquire_reserved_lock, landing_is_nested};
use super::commit_preparation::PreparedCommit;
use super::commit_replay_options::ReplayOptions;
use super::commit_revalidation::{Revalidation, RevalidationContext, revalidate_replay};
use super::commit_transaction::{CommitInvocation, TransactionStop, failed};
use super::commit_workspace::CommitWorkspace;
use super::commit_worktree_records::{HookChanges, settled_entry};
use super::diagnostic_log::debug;
use super::event_path::EventPath;
use super::landing_attempt::{LandingOutcome, LandingPayload, LandingRequest, land_transaction};
use super::landing_findings::{LANDING_FINDING_EXIT_CODE, landing_finding};
use super::landing_objects::migrate_shadow_objects;
use super::owner_lock::OwnerLock;
use super::policy_checks::ShippedChecks;
use super::policy_engine::StageRequest;
use super::policy_events::PolicyEvent;
use super::replay_commit::{MergeStep, ReplayError, ReplayRequest, merge_replay, rebuild_replay};
use super::replay_merge::ReplayedCommit;
use super::replay_reservation::{LandingReservation, ReservationError};
use super::repository_facts::RepositoryFacts;
use super::transaction_journal::{AddedPath, Base, TransactionMode};
use super::transaction_phase::{Phase, reach_transaction_phase};
use std::path::{Path, PathBuf};

/// Module tag for diagnostics.
const TAG: &str = "landWithReplay";

/// What:
///  What the next landing attempt lands.
/// Why:
///   The candidate is replaced after each replay and lives only inside the loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingCandidate = { newOid; treeOid; expectedOld; indexPath; exactPrivateIndex; approvedTree; committedPaths; ... };
/// ```
struct Candidate {
    /// The commit to land.
    new_oid: String,
    /// Its tree.
    tree: String,
    /// The target value its parent names.
    expected_old: Base,
    /// The private index holding its tree.
    index: PathBuf,
    /// Whether that index is the one native preparation committed.
    exact_private_index: bool,
    /// The tree `pre-commit` last approved.
    approved_tree: String,
    /// The paths an explicit-path landing resets.
    committed_paths: Vec<Vec<u8>>,
    /// The policy-added paths.
    added_paths: Vec<AddedPath>,
    /// The selected, corrected and hook-changed worktree completions.
    worktree_records: Vec<AddedPath>,
    /// Everything commit hooks staged so far.
    hook_changes: HookChanges,
    /// The events of lost races, replays and revalidations so far.
    events: Vec<PolicyEvent>,
    /// The landing attempt number.
    attempt: i64,
    /// The lost races so far.
    lost_races: u64,
}

/// What:
///  The inputs of the loop.
/// Why:
///   Replay and revalidation reuse the invocation's facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingLoopInput = { context; mode; prepared; settled; options; committedPaths; addedPaths; worktreeRecords; hookChanges; indexLockTimeoutMs; reserveAfterLostRaces };
/// ```
pub struct LoopInput<'a, F: RepositoryFacts> {
    /// The invocation's fixed facts.
    pub invocation: &'a CommitInvocation<'a>,
    /// The published workspace.
    pub workspace: &'a mut CommitWorkspace,
    /// The invocation capture.
    pub capture: &'a InvocationCapture,
    /// The commit selection mode.
    pub mode: TransactionMode,
    /// The request the policies run under.
    pub request: &'a StageRequest,
    /// The shipped checks over the private index.
    pub checks: &'a mut ShippedChecks<PrivateIndexFacts<F>>,
    /// The verified prepared commit.
    pub prepared: &'a PreparedCommit,
    /// The settled preparation pass's events with its fix summary.
    pub settled_events: Vec<PolicyEvent>,
    /// The replay-relevant commit options.
    pub options: &'a ReplayOptions,
    /// The caller's kept global options.
    pub global_args: &'a [String],
    /// Whether interactive or include selection forbids automatic patches.
    pub read_only_selection: bool,
    /// Whether the dispatch plan wrote hook entries, so a replay has a `pre-commit` entry to re-run.
    pub named_hooks_possible: bool,
    /// The paths an explicit-path commit carries.
    pub committed_paths: Vec<Vec<u8>>,
    /// The policy-added paths.
    pub added_paths: Vec<AddedPath>,
    /// The worktree completions.
    pub worktree_records: Vec<AddedPath>,
    /// What preparation hooks staged.
    pub hook_changes: HookChanges,
    /// The backoff budget for a foreign `index.lock`.
    pub index_lock_timeout_ms: u64,
    /// The lost landing races after which the transaction takes the landing reservation.
    pub reserve_after_lost_races: u64,
}

/// What:
///  How the loop ended.
/// Why:
///   A landed commit continues to post-landing completion; a failure carries the complete events and the status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LandingLoopOutcome = { kind: 'landed'; oid; events; worktreeRecords; addedPaths } | { kind: 'failed'; result };
/// ```
pub enum LoopOutcome {
    /// The commit landed.
    Landed {
        /// The landed commit.
        oid: String,
        /// The events to append after the settled preparation pass.
        events: Vec<PolicyEvent>,
        /// The worktree completions for the landed commit.
        worktree_records: Vec<AddedPath>,
        /// The policy-added paths of the landed commit.
        added_paths: Vec<AddedPath>,
    },
    /// Nothing landed.
    Failed {
        /// The complete blocking events.
        events: Vec<PolicyEvent>,
        /// The exit status.
        exit_code: i32,
    },
}

/// What:
///  Points worktree completions at the blobs a replayed tree holds, dropping paths it no longer holds as ordinary files.
/// Why:
///   A completion names the blob the commit now carries for its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await retargetRecords({ context, indexPath, records });
/// ```
fn retarget_records(
    input: &CommitInvocation<'_>,
    index: &Path,
    records: &[AddedPath],
) -> Result<Vec<AddedPath>, TransactionStop> {
    let mut retargeted: Vec<AddedPath> = Vec::new();
    for record in records {
        let Some((mode, oid)) = settled_entry(input.context, input.cwd, index, record.path.as_slice())? else {
            continue;
        };
        if mode != record.git_mode {
            continue;
        }
        retargeted.push(AddedPath {
            intended_oid: oid,
            ..record.clone()
        });
    }
    return Ok(retargeted);
}

/// What:
///  The replay request of one lost race.
/// Why:
///   Every replay step reads the same facts, so they are built once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { gitPath, shadowPath, objectDirectory, cwd, directory, replay, mergeBase, current, prepared }
/// ```
fn replay_request<'a, F: RepositoryFacts>(
    input: &'a LoopInput<'a, F>,
    replay: u64,
    onto: &'a str,
) -> ReplayRequest<'a> {
    return ReplayRequest {
        context: input.invocation.context,
        shadow: input.workspace.landing.shadow.as_path(),
        object_directory: input.workspace.object_directory.as_path(),
        worktree_root: input.capture.layout.repository_root.as_path(),
        git_dir: input.capture.layout.git_dir.as_path(),
        transaction_directory: input.workspace.landing.directory.as_path(),
        replay,
        base: match &input.capture.base {
            Base::Commit(oid) => Some(oid.as_str()),
            Base::Unborn => None,
        },
        empty_tree: input.capture.layout.empty_tree_oid.as_str(),
        current: onto,
        prepared: input.prepared.oid.as_str(),
        signing_key: input.options.signing_key.as_deref(),
        global_arguments: input.global_args,
    };
}

/// What:
///  The failure a replay that broke maps to.
/// Why:
///   A rejected re-run `pre-commit` is Git's own status; any other replay failure becomes a transaction failure
///   naming the target and the cause, with nothing landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// transactionFailure({ previous: input.settled, message: `${targetRef} moved while this commit was prepared, and replaying it onto ${onto} failed; nothing landed: ${error}` })
/// ```
fn replay_failure(capture: &InvocationCapture, onto: &str, cause: String) -> LoopOutcome {
    return LoopOutcome::Failed {
        events: vec![PolicyEvent::EngineFailure {
            code: super::diagnostics::EngineFailureCode::TransactionFailed,
            message: format!(
                "{} moved while this commit was prepared, and replaying it onto {onto} failed; nothing landed: {cause}",
                capture.layout.target_ref
            ),
            trigger: Some(super::policy_trigger::Trigger::PreForward),
            policy: None,
            path: None,
        }],
        exit_code: 2,
    };
}

/// What:
///  The events of a failure after some attempts: the settled events, then what the loop gathered, then the last ones.
/// Why:
///   Every failure ends the same list of events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendEvents({ events: settled.events, appended: [...candidate.events, finding] })
/// ```
fn failed_events(settled: &[PolicyEvent], raced: &[PolicyEvent], last: Vec<PolicyEvent>) -> Vec<PolicyEvent> {
    let mut events: Vec<PolicyEvent> = settled.to_vec();
    events.extend(raced.iter().cloned());
    events.extend(last);
    return events;
}

/// What:
///  Replay after a lost race: merge, conflict handling, revalidation, and the rebuilt commit.
/// Why:
///   The next candidate is built from the prepared commit again, with the lost race's events kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const next = await replayCandidate({ input, candidate, onto });
/// ```
fn replay_candidate<F: RepositoryFacts>(
    input: &mut LoopInput<'_, F>,
    candidate: &Candidate,
    raced: Vec<PolicyEvent>,
    onto: &str,
) -> Result<Result<Candidate, LoopOutcome>, TransactionStop> {
    let replay: u64 = lost_races_after(candidate.lost_races);
    let merge: Result<MergeStep, ReplayError> = merge_replay(&replay_request(input, replay, onto));
    let merged_tree: String = match merge {
        Err(error) => return Ok(Err(replay_failure(input.capture, onto, replay_error_text(&error)))),
        Ok(MergeStep::Conflict(conflict)) => {
            // Unkept: the prepared commit survives the shadow's removal until gc expires unreachable objects.
            migrate_shadow_objects(
                input.invocation.context,
                input.invocation.cwd,
                input.workspace.landing.shadow.as_path(),
                input.prepared.oid.as_str(),
                &input.capture.base,
                None,
            )?;
            let last: Vec<PolicyEvent> = vec![PolicyEvent::ReplayConflict {
                target_ref: input.capture.layout.target_ref.clone(),
                paths: conflict
                    .paths
                    .iter()
                    .map(|path| { return EventPath::from_git_bytes(path.as_slice()); })
                    .collect(),
                winning_oid: conflict.winning_oid,
                prepared_oid: conflict.prepared_oid,
            }];
            return Ok(Err(LoopOutcome::Failed {
                events: failed_events(input.settled_events.as_slice(), raced.as_slice(), last),
                exit_code: 1,
            }));
        }
        Ok(MergeStep::Merged { tree, subsumed_paths, kept_landed_paths }) => {
            debug(
                TAG,
                format!("replay onto {onto} keeps the prepared entries of {subsumed_paths:?} and the landed entries of {kept_landed_paths:?}").as_str(),
            );
            tree
        }
    };
    let revalidation: Revalidation = {
        let context: RevalidationContext<'_> = RevalidationContext {
            invocation: input.invocation,
            capture: input.capture,
            read_only_selection: input.read_only_selection,
            global_args: input.global_args,
            no_verify: input.options.no_verify,
            named_hooks_possible: input.named_hooks_possible,
        };
        revalidate_replay(
            &context,
            input.workspace,
            input.request,
            input.checks,
            replay,
            merged_tree.as_str(),
            onto,
            candidate.approved_tree.as_str(),
        )?
    };
    let (tree, index, revalidation_events, revalidated_paths, revalidated_hooks): (String, PathBuf, Vec<PolicyEvent>, Vec<Vec<u8>>, HookChanges) =
        match revalidation {
            Revalidation::Blocked { events, exit_code } => {
                return Ok(Err(LoopOutcome::Failed {
                    events: failed_events(input.settled_events.as_slice(), raced.as_slice(), events),
                    exit_code,
                }));
            }
            Revalidation::Revalidated {
                tree,
                index,
                events,
                committed_paths,
                hook_changes,
            } => (tree, index, events, committed_paths, hook_changes),
        };
    let rebuilt = match rebuild_replay(&replay_request(input, replay, onto), tree.as_str()) {
        Ok(found) => found,
        Err(error) => return Ok(Err(replay_failure(input.capture, onto, replay_error_text(&error)))),
    };
    let committed_paths: Vec<Vec<u8>> = union_paths(input.committed_paths.as_slice(), revalidated_paths);
    let mut worktree_records: Vec<AddedPath> = retarget_records(input.invocation, index.as_path(), input.worktree_records.as_slice())?;
    worktree_records.extend(revalidated_hooks.worktree_records.iter().cloned());
    let mut events: Vec<PolicyEvent> = replayed_events(
        raced,
        input.prepared.oid.as_str(),
        &input.capture.base,
        onto,
        &rebuilt,
    );
    events.extend(revalidation_events);
    return Ok(Ok(Candidate {
        new_oid: rebuilt.oid,
        tree: tree.clone(),
        expected_old: Base::Commit(String::from(onto)),
        index: index.clone(),
        exact_private_index: false,
        approved_tree: tree,
        committed_paths,
        added_paths: retarget_records(input.invocation, index.as_path(), input.added_paths.as_slice())?,
        worktree_records,
        hook_changes: HookChanges::combine(&input.hook_changes, &revalidated_hooks),
        events,
        attempt: next_attempt_number(candidate.attempt),
        lost_races: lost_races_after(candidate.lost_races),
    }));
}

/// What:
///  The paths of a replay: those already committed, then the ones revalidation added that are new.
/// Why:
///   A path both lists name is one path of the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...new Set([...committedPaths, ...revalidatedPaths])]
/// ```
pub fn union_paths(committed: &[Vec<u8>], revalidated: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut paths: Vec<Vec<u8>> = committed.to_vec();
    // Membership is a set lookup, so a commit of thousands of paths costs linear work per replay, not quadratic.
    let known: std::collections::HashSet<&[u8]> = committed.iter().map(Vec::as_slice).collect();
    let mut added: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
    for path in revalidated {
        if known.contains(path.as_slice()) || !added.insert(path.clone()) {
            continue;
        }
        paths.push(path);
    }
    return paths;
}

/// What:
///  The events of one replay: the lost race, the replayed commit, and the dropped signature headers when there are some.
/// Why:
///   The incumbent prints them in this order, and the headers event only when the rebuild dropped a header.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [raceLost, commitReplayed, ...(dropped.length > 0 ? [replayHeadersDropped] : [])]
/// ```
pub fn replayed_events(
    raced: Vec<PolicyEvent>,
    prepared_oid: &str,
    base: &Base,
    onto: &str,
    rebuilt: &ReplayedCommit,
) -> Vec<PolicyEvent> {
    let mut events: Vec<PolicyEvent> = raced;
    events.push(PolicyEvent::CommitReplayed {
        prepared_oid: String::from(prepared_oid),
        from_base: match base {
            Base::Commit(oid) => Some(oid.clone()),
            Base::Unborn => None,
        },
        onto: String::from(onto),
        oid: rebuilt.oid.clone(),
    });
    if !rebuilt.dropped_headers.is_empty() {
        events.push(PolicyEvent::ReplayHeadersDropped {
            prepared_oid: String::from(prepared_oid),
            oid: rebuilt.oid.clone(),
            headers: rebuilt.dropped_headers.clone(),
        });
    }
    return events;
}

/// What:
///  The landing attempt number that follows a lost race.
/// Why:
///   The attempt number names the attempt's landing files, so two attempts of one commit never share a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// attempt + 1
/// ```
pub fn next_attempt_number(attempt: i64) -> i64 {
    return attempt + 1;
}

/// What:
///  The count of lost races once one more race is lost.
/// Why:
///   The reservation threshold and the `race-lost` marker occurrence both read this count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lostRaces + 1
/// ```
pub fn lost_races_after(lost_races: u64) -> u64 {
    return lost_races + 1;
}

/// What:
///  The wording of a replay step failure.
/// Why:
///   The transaction failure names the cause after the target and the parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caughtValueText(error)
/// ```
fn replay_error_text(error: &ReplayError) -> String {
    return match error {
        ReplayError::Shared(inner) => format!("{inner:?}"),
        ReplayError::Capture(inner) => format!("{inner:?}"),
        ReplayError::Subsumption(inner) => format!("{inner:?}"),
        ReplayError::Merge(inner) => format!("{inner:?}"),
        ReplayError::Git(inner) => inner.0.clone(),
        ReplayError::Object(text) => text.clone(),
    };
}

/// What:
///  Map a reservation failure to the transaction failure it is.
/// Why:
///   A reservation failure stops the landing loop with nothing landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw error
/// ```
fn reservation_stop(error: ReservationError) -> TransactionStop {
    return failed(format!("the landing reservation failed: {error}"));
}

/// What:
///  Land a prepared commit, replaying and revalidating after every lost race.
/// Why:
///   The landing lock is taken for one attempt only and released before any replay or revalidation work, so a loser
///   never blocks the others while it rebuilds its commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landWithReplay({ ... });
/// ```
pub fn land_with_replay<F: RepositoryFacts>(mut input: LoopInput<'_, F>) -> Result<LoopOutcome, TransactionStop> {
    let layout = &input.capture.layout;
    let nested: bool = landing_is_nested(layout, input.invocation.environment);
    let mut reservation: LandingReservation = LandingReservation::open(
        layout.registry_root.as_path(),
        input.workspace.landing.directory.as_path(),
        input.workspace.landing.transaction_id.as_str(),
        nested,
    );
    let result: Result<LoopOutcome, TransactionStop> = run_loop(&mut input, &mut reservation);
    // Released when the loop ends, landed or failed.
    if let Err(error) = reservation.release() {
        debug(TAG, format!("releasing the landing reservation failed: {error:?}").as_str());
    }
    return result;
}

/// What:
///  The loop proper, with the reservation owned by the caller.
/// Why:
///   The caller releases the reservation whatever the loop returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let candidate = ...;;) { ... }
/// ```
fn run_loop<F: RepositoryFacts>(
    input: &mut LoopInput<'_, F>,
    reservation: &mut LandingReservation,
) -> Result<LoopOutcome, TransactionStop> {
    let mut candidate: Candidate = Candidate {
        new_oid: input.prepared.oid.clone(),
        tree: input.prepared.tree_oid.clone(),
        expected_old: input.capture.base.clone(),
        index: input.workspace.landing.commit_index.clone(),
        exact_private_index: true,
        approved_tree: input.prepared.tree_oid.clone(),
        committed_paths: input.committed_paths.clone(),
        added_paths: input.added_paths.clone(),
        worktree_records: input.worktree_records.clone(),
        hook_changes: input.hook_changes.clone(),
        events: Vec::new(),
        attempt: 1,
        lost_races: 0,
    };
    loop {
        let payload: LandingPayload = LandingPayload::Commit {
            new_oid: candidate.new_oid.clone(),
            landed_tree_oid: candidate.tree.clone(),
            expected_old: candidate.expected_old.clone(),
            landed_index: candidate.index.clone(),
            exact_private_index: candidate.exact_private_index,
            guarded_paths: candidate.hook_changes.paths.clone(),
        };
        let request: LandingRequest<'_> = LandingRequest {
            capture: input.capture,
            mode: input.mode,
            payload: &payload,
            committed_paths: candidate.committed_paths.as_slice(),
            added_paths: candidate.added_paths.as_slice(),
            selected_worktree_paths: candidate.worktree_records.as_slice(),
            index_lock_timeout_ms: input.index_lock_timeout_ms,
            attempt: candidate.attempt,
        };
        let outcome: LandingOutcome = {
            // The lock is released when this block ends, before any replay work.
            let lock: OwnerLock = acquire_reserved_lock(
                input.invocation.context,
                input.invocation.cwd,
                &input.capture.layout,
                input.invocation.environment,
                reservation,
            )
            .map_err(reservation_stop)?;
            land_transaction(
                input.invocation.context,
                input.invocation.cwd,
                &lock,
                &mut input.workspace.landing,
                &request,
            )?
        };
        let onto: String = match &outcome {
            LandingOutcome::Landed { oid } => {
                return Ok(LoopOutcome::Landed {
                    oid: oid.clone(),
                    events: candidate.events,
                    worktree_records: candidate.worktree_records,
                    added_paths: candidate.added_paths,
                });
            }
            LandingOutcome::HeadMoved {
                current: Base::Commit(oid),
                replayable: true,
            } => oid.clone(),
            _ => {
                let mut last: Vec<PolicyEvent> = Vec::new();
                if let Some(event) = landing_finding(&outcome, input.capture, Some(input.prepared.oid.as_str()), false) {
                    last.push(event);
                }
                return Ok(LoopOutcome::Failed {
                    events: failed_events(input.settled_events.as_slice(), candidate.events.as_slice(), last),
                    exit_code: LANDING_FINDING_EXIT_CODE,
                });
            }
        };
        let lost_races: u64 = lost_races_after(candidate.lost_races);
        debug(TAG, format!("lost landing race {lost_races} to {onto}").as_str());
        let reserved: bool = lost_races >= input.reserve_after_lost_races
            && reservation.reserve(lost_races).map_err(reservation_stop)?;
        reach_transaction_phase(Phase::RaceLost, Some(lost_races))?;
        let mut raced: Vec<PolicyEvent> = candidate.events.clone();
        raced.push(PolicyEvent::LandingRaceLost {
            attempt: lost_races,
            winning_oid: onto.clone(),
        });
        if reserved {
            raced.push(PolicyEvent::LandingReserved { lost_races });
        }
        match replay_candidate(input, &candidate, raced, onto.as_str())? {
            Ok(next) => candidate = next,
            Err(ended) => return Ok(ended),
        }
    }
}

/// Controls for this module stay out of the release executable.
#[cfg(test)]
#[path = "commit_landing_loop_tests.rs"]
mod tests;
