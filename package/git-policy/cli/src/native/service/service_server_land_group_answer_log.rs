//! What:
//!  What one batch's answer says about that batch:
//! the times every request of it shares,
//! the figures a landed answer carries,
//! the paths its rules are warmed for,
//! and the log line of every answer.
//! Why:
//!  The landing step answers a whole batch of requests and logs each answer with the phases of its wait;
//!  a file of its own keeps that step under its line bound
//!  (`service_server_land_group.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const times = new BatchTimes({ groupBegan, settle, landing, landTotal, batch, size, queuedBehind });
//! logAnswer(times, request, arrival, position, outcome);
//! ```

use crate::journald_native::Priority;
use crate::service_protocol::{CommitRequest, LandingFigures, LandingSync};
use crate::service_timing::{Answered, Arrival, LandTiming, answered_line, log_entry};
use std::time::{Duration, Instant};

/// The times of one batch that every request of it shares,
/// and the moment the landing ended.
pub(crate) struct BatchTimes {
    /// The batch number,
    /// zero when no batch ran (a request answered as already landed).
    pub(crate) batch: u64,
    /// How many requests of the branch the batch held.
    pub(crate) size: usize,
    /// When the group began to be handled,
    /// which ends every request's wait in the queue.
    pub(crate) group_began: Instant,
    /// Time to settle the batches a killed service left open.
    pub(crate) settle: Duration,
    /// The phases of the landing.
    pub(crate) landing: LandTiming,
    /// Time the landing took,
    /// from its start to its return.
    pub(crate) land_total: Duration,
    /// When the landing returned,
    /// which starts the answers.
    pub(crate) landed_at: Instant,
    /// How many requests were still waiting for their answer when the landing finished,
    /// this batch's own taken out:
    /// how many commits queued behind it.
    pub(crate) queued_behind: usize,
}

impl BatchTimes {
    /// The times of a request that no batch landed.
    pub(crate) fn not_landed(group_began: Instant) -> Self {
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
pub(crate) fn landing_figures_of(times: &BatchTimes) -> LandingFigures {
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
pub(crate) fn path_of_change(change: &crate::service_protocol::Change) -> String {
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
