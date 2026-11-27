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
//! it is the largest step of the service,
//! so its parts each stand in a file of this folder and the loop that feeds it stays short.
//!
//! The parts of one landing,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `service_server_land_group_the_batches_left_open.rs`:
//!   the batches an earlier service left open,
//!   settled before this one lands.
//! - `service_server_land_group_the_words_a_client_waits_with.rs`:
//!   what the landing waits for and the place each request of the batch has.
//! - `service_server_land_group_the_answers_to_a_batch.rs`:
//!   what each request of a landed batch came to,
//!   the repeats answered from it,
//!   and the note that names the repository to the next service.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await landGroup(context, instance, seq, group, landed, captures);
//! ```

/// What a batch of landings owes,
/// named here so every caller of the group keeps one path.
pub(crate) use crate::service_server_land_group_owed_pushes::*;
/// What one landed batch tells its requests.
use crate::service_server_land_group_the_answers_to_a_batch::TheAnswersToABatch;
/// The batches an earlier service left open,
/// and the word a settled batch is logged with.
pub(crate) use crate::service_server_land_group_the_batches_left_open::{
    decision_word,
    settle_open_batches,
};
/// The words a client of one batch waits with.
use crate::service_server_land_group_the_words_a_client_waits_with::{
    AnnounceToStreams,
    place_notices,
    waited_since_accepted,
};

use crate::service_captures::CaptureMemory;
use crate::service_journal::{
    BatchClaim,
    claim_batch,
};
use crate::service_land::{
    LandContext,
    land_batch_timed,
};
use crate::service_land_follow::storage_failure;
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
    ServiceMessage,
};
use crate::service_server_land_group_answer_log::{
    BatchTimes,
    log_answer,
};
use crate::service_server_queue_types_and_send::Queued;
use crate::service_timing::LandTiming;
use std::collections::HashMap;
use std::time::{
    Duration,
    Instant,
};

/// Lands one group of jobs that share a branch,
/// and answers each client.
/// `place_after` is the wait a batch's first request must have borne before it is told it is next
/// (`wait_ceilings.rs`,
/// `WAITING_NOTICE_AFTER`),
/// and `note_ceiling` the wait its note to the next service is granted
/// (`wait_ceilings.rs`,
/// `LANDING_WATCH_NOTE_CEILING`).
/// Records the identities of the requests whose scratch tree the landing did not read for good,
/// because the answer sent their client back to read the private index of the attempt it captured
/// (`service_prepare_ready_shapes.rs`,
/// `HandedBackTree`):
/// those trees stay where they are until the client that owns them removes them.
pub(crate) fn land_group(
    context: &LandContext,
    instance: Option<&str>,
    seq: &mut u64,
    group: Vec<Queued>,
    landed: &mut HashMap<String, String>,
    captures: &mut CaptureMemory,
    place_after: Duration,
    note_ceiling: Duration,
    handed_back: &mut std::collections::HashSet<String>,
    drawn: &std::sync::Arc<crate::service_shadow_draw::DrawnShadows>,
) {
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
        stream.send(&ServiceMessage::Landed {
            id: request.id.clone(),
            commit: landed[&request.id].clone(),
            already: true,
            landing: None,
        });
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
        return;
    }
    // What:
    //  A request whose client is gone before its commit lands is not made.
    // Why:
    //  A commit exists for the person who asked for it,
    //  and nothing reads the answer of a client that went away,
    //  while native Git makes no commit for a command that was ended.
    // This is the first of the two places the landing asks:
    //  the body asks again while it waits for another program's index lock,
    //  which is where a person's interrupt lands.
    //  The tree stays for a client that reconnects and sends the same request again,
    //  and the sweep takes it by age otherwise (`service_prepare_ready_shapes.rs`).
    let mut present: Vec<Queued> = Vec::with_capacity(fresh.len());
    for queued in fresh {
        if queued.1.client_is_gone() {
            crate::service_log!(
                "dropped the request {} whose client is gone before its commit landed",
                queued.0.id
            );
            handed_back.insert(queued.0.id.clone());
        } else {
            present.push(queued);
        }
    }
    fresh = present;
    if fresh.is_empty() {
        return;
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
            crate::service_error!(
                "no batch number could be taken in {}: {error}",
                context.journal.display()
            );
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed {
                    id: request.id.clone(),
                    code,
                });
                log_answer(
                    &unlanded,
                    request,
                    arrival,
                    position + 1,
                    format!("failed:{}", code.word()).as_str(),
                );
            }
            return;
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
        stream.send(&ServiceMessage::Position {
            number,
            of,
        });
    }
    let mut requests: Vec<CommitRequest> = Vec::new();
    for (request, _, _) in &fresh {
        requests.push((**request).clone());
    }
    let announce: AnnounceToStreams<'_> = AnnounceToStreams {
        fresh: fresh.as_slice(),
    };
    // The next commit in this worktree takes a finished private repository instead of making one. The fill runs beside this
    //  landing, on the service's own workers, so the commit that waits for the answer below pays nothing for it.
    if let Some(first) = requests.first() {
        crate::service_shadow_pool_fill::start(&context.git, first);
        // What: The scratch trees a killed command left are collected while this landing runs.
        // Why:  The collection lists the repository scratch directory and asks the file system about every entry, which was
        //       the largest single cause of a commit's own time in the records of 2026-11-11 (13,482 ms of 48). The commit
        //       that makes its tree no longer collects, so the work is here, beside the loop, on a thread of its own.
        crate::service_repository_scratch_sweep::start(first.common_dir.as_path());
    }
    let mut landing: LandTiming = LandTiming::default();
    let landing_began: Instant = Instant::now();
    let result = land_batch_timed(context, *seq, &requests, &announce, &mut landing);
    // What: The next commit in this worktree is handed a finished shadow the service renamed out of the pool before it asked.
    // Why:  The rename is the call that waited seconds on this volume, and doing it here — on a thread of its own, after this
    //       landing — takes it off every commit's wait while leaving every check the claim makes with the commit. It runs
    //       after the landing rather than before it so that the fill this landing started has made its member by then,
    //       and it makes one itself when the pool is still empty (`draw_one`).
    if let Some(first) = requests.first() {
        crate::service_shadow_draw::start(&context.git, first, drawn);
    }
    // The intent or the finished batch keeps the number now; a landing that wrote none gives the number back.
    drop(claim);
    // How many requests were still waiting when this landing finished: the batch's own are still unanswered here, so they come
    //  out of the count, and what is left is how many commits queued behind this landing.
    let waiting: usize = crate::service_reply::Reply::waiting();
    let queued_behind: usize = waiting.saturating_sub(fresh.len());
    let times: BatchTimes = BatchTimes {
        batch: *seq,
        size: fresh.len(),
        group_began,
        settle,
        landing,
        land_total: landing_began.elapsed(),
        landed_at: Instant::now(),
        queued_behind,
    };
    let mut answers: TheAnswersToABatch<'_> = TheAnswersToABatch::of(
        context,
        instance,
        note_ceiling,
        &times,
        landed,
        handed_back,
        captures,
    );
    match result {
        Ok(outcomes) => {
            // What this batch landed is remembered before any answer, so a commit sent back in the same batch is told about it.
            answers.remember_what_landed(fresh.as_slice(), outcomes.as_slice());
            crate::service_fault::reach("before-ack");
            answers.answer_the_repeats(fresh.as_slice(), repeats.as_slice(), outcomes.as_slice());
            for (position, ((request, stream, arrival), outcome)) in
                fresh.iter().zip(outcomes).enumerate()
            {
                let word: &str = answers.answer(request, stream, outcome);
                log_answer(&times, request, arrival, position + 1, word);
            }
        }
        Err(code) => {
            for (position, (request, stream, arrival)) in fresh.iter().enumerate() {
                stream.send(&ServiceMessage::Failed {
                    id: request.id.clone(),
                    code,
                });
                log_answer(
                    &times,
                    request,
                    arrival,
                    position + 1,
                    format!("failed:{}", code.word()).as_str(),
                );
            }
        }
    }
}

/// What a batch tells its requests is controlled in the test module.
#[cfg(test)]
#[path = "service_server_land_group_tests.rs"]
mod tests;
