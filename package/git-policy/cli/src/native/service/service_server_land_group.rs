//! What:
//!  Lands one group of queued requests as a batch and answers each client:
//! settles open batches first,
//! answers repeats and
//!  requests landed before,
//! numbers the batch,
//! lands it,
//! and logs every answer with its timings.
//! Why:
//!  The landing thread of `service_server.rs` calls it once per group;
//! it is the largest step of the service and has its own
//!  file so the loop that feeds it stays short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await landGroup(context, instance, seq, group, landed, captures);
//! ```

use crate::service_server_queue_types_and_send::Queued;
use crate::service_journal::{BatchClaim, Recovery, claim_batch};
use crate::service_land_follow::storage_failure;
use crate::service_index_lock::{Announce, Announcer};
use crate::service_captures::CaptureMemory;
use crate::service_land::{LandContext, Outcome, land_batch_timed, recover, recover_instance};
use crate::journald_native::Priority;
use crate::service_timing::{Answered, Arrival, LandTiming, answered_line, log_entry};
use crate::service_protocol::{
    CommitRequest, FailureCode, LandingFigures, LandingSync, ServiceMessage,
};
/// What a batch of landings owes,
/// named here so every caller of the group keeps one path.
pub(crate) use crate::service_server_land_group_owed_pushes::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};
/// Finishes the open batches this instance owns:
/// those of its worktree when it serves one,
/// all of them in a development run.
pub(crate) fn settle_open_batches(context: &LandContext, instance: Option<&str>) -> Vec<crate::service_land::RecoveryReport> {
    let reports: Vec<crate::service_land::RecoveryReport> = match instance {
        Some(mine) => recover_instance(context, mine),
        None => recover(context),
    };
    // An open batch means an earlier service stopped between its intent and its end; each one settled is said.
    for report in &reports {
        log_entry(
            Priority::Warning,
            format!("settled the batch an earlier service left open batch={} decision={}", report.seq, if report.decision == Recovery::Finish { "finish" } else { "drop" }).as_str(),
            &[("CLI_GIT_BATCH", report.seq.to_string())],
        );
    }
    return reports;
}

/// The times of one batch that every request of it shares,
/// and the moment the landing ended.
pub(crate) struct BatchTimes {
    /// The batch number,
    /// zero when no batch ran (a request answered as already landed).
    batch: u64,
    /// How many requests of the branch the batch held.
    size: usize,
    /// When the group began to be handled,
    /// which ends every request's wait in the queue.
    group_began: Instant,
    /// Time to settle the batches a killed service left open.
    settle: Duration,
    /// The phases of the landing.
    landing: LandTiming,
    /// Time the landing took,
    /// from its start to its return.
    land_total: Duration,
    /// When the landing returned,
    /// which starts the answers.
    landed_at: Instant,
    /// How many requests were still waiting for their answer when the landing finished,
    /// this batch's own taken out:
    /// how many commits queued behind it.
    queued_behind: usize,
}

impl BatchTimes {
    /// The times of a request that no batch landed.
    fn not_landed(group_began: Instant) -> Self {
        return Self { batch: 0, size: 0, group_began, settle: Duration::ZERO, landing: LandTiming::default(), land_total: Duration::ZERO, landed_at: Instant::now(), queued_behind: 0 };
    }
}

/// What:
///  What one landing spent and held,
/// as the answer to its clients states it.
/// Why:
///   The landing runs in this service,
/// so the commit's own record knows nothing about the reads,
/// the hold or the syncs of its landing
/// unless the answer carries them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const figuresOf = (times) => ({ lockHoldMs: ms(times.landing.hold), queuedBehind: times.queuedBehind, readsMs: ms(times.landing.reads(times.landTotal)), reads: {...}, syncs: [...] });
/// ```
fn landing_figures_of(times: &BatchTimes) -> LandingFigures {
    let split: crate::service_timing::ReadSplit = times.landing.reads_split;
    let named: [(&'static str, Duration); 9] = [
        ("config", split.config),
        ("branch", split.branch),
        ("parents", split.parents),
        ("changed", split.changed),
        ("content", split.content),
        ("witness", split.witness),
        ("conclusion", split.conclusion),
        ("owed", split.owed),
        ("follow", split.follow),
    ];
    let mut reads: Vec<(String, u64)> = Vec::new();
    for (name, took) in named {
        reads.push((String::from(name), u64::try_from(took.as_millis()).unwrap_or(u64::MAX)));
    }
    let mut syncs: Vec<LandingSync> = Vec::new();
    for sync in &times.landing.syncs {
        syncs.push(LandingSync {
            what: String::from(sync.what),
            waited: sync.waited,
            ms: u64::try_from(sync.took.as_millis()).unwrap_or(u64::MAX),
        });
    }
    return LandingFigures {
        lock_hold_ms: u64::try_from(times.landing.hold.as_millis()).unwrap_or(u64::MAX),
        queued_behind: u64::try_from(times.queued_behind).unwrap_or(u64::MAX),
        reads_ms: u64::try_from(times.landing.reads(times.land_total).as_millis()).unwrap_or(u64::MAX),
        reads,
        syncs,
    };
}

/// The repository-relative path a change names,
/// as the text a rule is warmed for.
/// Why:
///   The rules are keyed by a path's text,
/// so a name that is not text is warmed under its readable spelling,
/// which only decides whether a rule is looked at early;
/// the entry that is written keeps the name's bytes.
fn path_of_change(change: &crate::service_protocol::Change) -> String {
    return crate::path_bytes::path_message_text(crate::path_bytes::path_bytes_of(change.path.as_path()));
}

/// Writes the log line for an answered request:
/// the phases of its wait,
/// from the acceptance of its connection to the answer.
pub(crate) fn log_answer(times: &BatchTimes, request: &CommitRequest, arrival: &Arrival, position: usize, outcome: &str) {
    let queued: Duration = times.group_began.saturating_duration_since(arrival.accepted + arrival.read);
    // What the Git steps and tools since the previous answer showed; the answer that finds them takes them.
    let quiet: Vec<(String, Duration)> = crate::quiet_ledger::drain();
    // The Git steps since the previous answer, with their offsets from this request's acceptance.
    let (steps, steps_dropped): (Vec<crate::transaction_step_durations::StepFigure>, usize) = crate::transaction_step_durations::drain_steps(arrival.accepted);
    let line: String = answered_line(&Answered {
        request: request.id.as_str(),
        reference: request.reference.as_str(),
        outcome,
        batch: times.batch,
        size: times.size,
        position,
        queued_behind: times.queued_behind,
        arrival: *arrival,
        queued,
        settle: times.settle,
        landing: &times.landing,
        land_total: times.land_total,
        answer: times.landed_at.elapsed(),
        total: arrival.accepted.elapsed(),
        quiet: quiet.as_slice(),
        steps: steps.as_slice(),
        steps_dropped,
    });
    // A request that was answered with a failure is a warning, so `journalctl --priority=warning` finds the commits that failed.
    let priority: Priority = if outcome.starts_with("failed") { Priority::Warning } else { Priority::Info };
    let fields: [(&str, String); 4] = [
        ("CLI_GIT_REQUEST_ID", request.id.clone()),
        ("CLI_GIT_BRANCH", request.reference.clone()),
        ("CLI_GIT_OUTCOME", outcome.to_string()),
        ("CLI_GIT_BATCH", times.batch.to_string()),
    ];
    log_entry(priority, line.as_str(), &fields);
}

/// What:
///  Tells every client of a batch what the landing waits for.
/// Why:
///   The landing takes its announcer as a value;
/// the streams of the batch are the state each announcement is sent through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AnnounceToStreams implements Announcer { announce(note) { for (const stream of streams) stream.send(note); } }
/// ```
struct AnnounceToStreams<'a> {
    /// The requests of the batch,
    /// each with the stream its client listens on.
    fresh: &'a [Queued],
}

impl Announcer for AnnounceToStreams<'_> {
    fn announce(&self, note: Announce) {
        for (_, stream, _) in self.fresh {
            match &note {
                Announce::Waiting(text) => stream.send(&ServiceMessage::Waiting { text: text.clone() }),
                Announce::Holder(who) => stream.send(&ServiceMessage::Holder { who: who.clone() }),
            }
        }
    }
}

/// The place each request of a batch is told,
/// as `(number, of)`,
/// in the order of the batch.
/// Why:
///   A request that is not the batch's first goes behind the ones before it,
/// so it is told how many commits go first however
///  short its wait;
/// the first is next,
/// and is told so only once its own wait has mattered,
/// so a batch that begins at once says
///  nothing new to anybody (and a client of an earlier release,
/// which prints every place it is told,
/// sees no new line for a quick commit).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placeNotices(3, 12_000, 10_000) // [[1,3],[2,3],[3,3]]
/// ```
fn place_notices(size: usize, waited: Duration, threshold: Duration) -> Vec<(usize, usize)> {
    let mut places: Vec<(usize, usize)> = Vec::new();
    for index in 0..size {
        if index == 0 && waited < threshold {
            continue;
        }
        places.push((index + 1, size));
    }
    return places;
}

/// How long the first request of a batch has waited since its connection was accepted,
/// which is the wait the telling answers.
/// Nothing when the batch holds no request,
/// which lands nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitedSinceAccepted(group[0]) // milliseconds
/// ```
fn waited_since_accepted(queued: &Queued) -> Duration {
    return queued.2.accepted.elapsed();
}

/// Lands one group of jobs that share a branch,
/// and answers each client.
/// `place_after` is the wait a batch's first request must have borne before it is told it is next
/// (`wait_ceilings.rs`,
/// `WAITING_NOTICE_AFTER`).
/// Returns the identities of the requests whose scratch tree the landing did not read for good,
/// because the answer sent their client back to read the private index of the attempt it captured
/// (`service_commit_tree_ownership.rs`):
/// those trees stay where they are until the client that owns them removes them.
pub(crate) fn land_group(
    context: &LandContext,
    instance: Option<&str>,
    seq: &mut u64,
    group: Vec<Queued>,
    landed: &mut HashMap<String, String>,
    captures: &mut CaptureMemory,
    place_after: Duration,
) -> Vec<String> {
    let group_began: Instant = Instant::now();
    // Requests this service already landed are answered from memory; the rest are still to land.
    let mut again: Vec<Queued> = Vec::new();
    let mut pending: Vec<Queued> = Vec::new();
    for queued in group {
        if landed.contains_key(&queued.0.id) {
            again.push(queued);
        } else {
            pending.push(queued);
        }
    }
    let unlanded: BatchTimes = BatchTimes::not_landed(group_began);
    for (request, stream, arrival) in again {
        stream.send(&ServiceMessage::Landed { id: request.id.clone(), commit: landed[&request.id].clone(), already: true, landing: None });
        log_answer(&unlanded, &request, &arrival, 1, "already");
    }
    // The same request sent twice into one batch (a client that reconnected) is landed once and answered twice.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    // The first request of an identity is fresh; each later one with the same identity is a repeat.
    let mut fresh: Vec<Queued> = Vec::new();
    let mut repeats: Vec<Queued> = Vec::new();
    for queued in pending {
        if seen.insert(queued.0.id.clone()) {
            fresh.push(queued);
        } else {
            repeats.push(queued);
        }
    }
    if fresh.is_empty() {
        return Vec::new();
    }
    // A batch whose branch moved but whose index update had to give up (another program held Git's index lock the whole
    //  time) left its intent open; settling it first puts the index right before anything else changes it.
    let settling: Instant = Instant::now();
    let _settled = settle_open_batches(context, instance);
    let settle: Duration = settling.elapsed();
    // The number is taken from the journal, not from this service's count alone: another service of the same journal may count
    //  from the same clock reading, and two landings with one number would overwrite each other's records.
    let claim: BatchClaim = match claim_batch(context.journal.as_path(), *seq + 1) {
        Ok(claim) => claim,
        Err(error) => {
            let code: FailureCode = storage_failure(&error);
            crate::service_error!("no batch number could be taken in {}: {error}", context.journal.display());
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed { id: request.id.clone(), code });
                log_answer(&unlanded, request, arrival, position + 1, format!("failed:{}", code.word()).as_str());
            }
            return Vec::new();
        }
    };
    *seq = claim.seq();
    // The first request of a batch has waited since its connection was accepted; the wait that matters for being told
    //  it is next is the one it has borne when the batch begins to land.
    let waited: Duration = fresh.first().map_or(Duration::ZERO, waited_since_accepted);
    for (number, of) in place_notices(fresh.len(), waited, place_after) {
        let Some((_, stream, _)) = fresh.get(number - 1) else {
            continue;
        };
        stream.send(&ServiceMessage::Position { number, of });
    }
    let mut requests: Vec<CommitRequest> = Vec::new();
    for (request, _, _) in &fresh {
        requests.push((**request).clone());
    }
    let announce: AnnounceToStreams<'_> = AnnounceToStreams { fresh: fresh.as_slice() };
    let mut landing: LandTiming = LandTiming::default();
    let landing_began: Instant = Instant::now();
    let result = land_batch_timed(context, *seq, &requests, &announce, &mut landing);
    // The intent or the finished batch keeps the number now; a landing that wrote none gives the number back.
    drop(claim);
    // How many requests were still waiting when this landing finished: the batch's own are still unanswered here, so they come
    //  out of the count, and what is left is how many commits queued behind this landing.
    let waiting: usize = crate::service_reply::Reply::waiting();
    let queued_behind: usize = waiting.saturating_sub(fresh.len());
    let times: BatchTimes = BatchTimes { batch: *seq, size: fresh.len(), group_began, settle, landing, land_total: landing_began.elapsed(), landed_at: Instant::now(), queued_behind };
    // The trees this answer leaves to their clients: the ones whose request was told to prepare again, or that the branch
    //  moved under, because each client reads the private index of that attempt to capture its change for the rebuild.
    let mut kept: Vec<String> = Vec::new();
    match result {
        Ok(outcomes) => {
            // What this batch landed is remembered before any answer, so a commit sent back in the same batch is told about it.
            for ((request, _, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    captures.record_landed(request.common_dir.as_path(), request.repository.as_path(), commit.as_str(), request.captured);
                }
            }
            crate::service_fault::reach("before-ack");
            for ((request, _, _), outcome) in fresh.iter().zip(&outcomes) {
                if let Outcome::Landed { commit } = outcome {
                    for (repeat, stream, arrival) in &repeats {
                        if repeat.id != request.id {
                            continue;
                        }
                        stream.send(&ServiceMessage::Landed { id: repeat.id.clone(), commit: commit.clone(), already: true, landing: None });
                        log_answer(&times, repeat, arrival, 1, "already");
                    }
                }
            }
            for (position, ((request, stream, arrival), outcome)) in fresh.iter().zip(outcomes).enumerate() {
                let word: &str = match outcome {
                    Outcome::Landed { commit } => {
                        // The compiled rules of the repository are warmed on the blocking pool, off this thread and off the answer's path.
                        let paths: Vec<String> = request.changes.iter().map(path_of_change).collect();
                        crate::service_rules_precompile::after_landing(context.journal.as_path(), request.repository.as_path(), paths.as_slice());
                        // The push is held before the answer, so a client killed between the two cannot lose it, and the post-commit
                        //  gate is what promotes it.
                        hold_the_push(request, commit.as_str());
                        // The repository is written down for the next service in the same instant, so a wrapper killed between
                        //  promoting that push and waking the service is still found when a service starts
                        //  (`service_claim_watch.rs`, `note_for_the_next_service`).
                        crate::service_claim_watch::note_for_the_next_service(
                            context.journal.join(crate::service_claim_watch::WATCH_DIRECTORY).as_path(),
                            instance,
                            request.git_dir.as_path(),
                        );
                        landed.insert(request.id.clone(), commit.clone());
                        stream.send(&ServiceMessage::Landed { id: request.id.clone(), commit, already: false, landing: Some(landing_figures_of(&times)) });
                        "landed"
                    }
                    Outcome::RetryPrepare { base } => {
                        let known = captures.remembered(request.common_dir.as_path(), request.repository.as_path());
                        stream.send(&ServiceMessage::RetryPrepare { id: request.id.clone(), base, landed: known });
                        kept.push(request.id.clone());
                        "retry-prepare"
                    }
                    Outcome::BranchMoved => {
                        stream.send(&ServiceMessage::Failed { id: request.id.clone(), code: FailureCode::BranchMoved });
                        kept.push(request.id.clone());
                        "branch-moved"
                    }
                };
                log_answer(&times, request, arrival, position + 1, word);
            }
        }
        Err(code) => {
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed { id: request.id.clone(), code });
                log_answer(&times, request, arrival, position + 1, format!("failed:{}", code.word()).as_str());
            }
        }
    }
    return kept;
}

/// What a batch tells its requests is controlled in the test module.
#[cfg(test)]
#[path = "service_server_land_group_tests.rs"]
mod tests;
