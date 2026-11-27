//! What:
//!  What one attempt's batch owes,
//!  taken with the index lock held and before the branch moves,
//!  and the index file the landing saw,
//!  recorded so that a restart never replays the index over what another program staged.
//! Why:
//!   A kill between the branch move and the index install leaves the record of the index update
//!  and the held push of every commit the batch lands,
//!  and a batch that never moves its branch owes nothing:
//!  taken any earlier,
//! a wait withdrawn at its ceiling,
//!  a lock never given up or a write Git never starts
//!  would leave records of a landing that did not happen,
//!  which every later command of the worktree would read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! takeWhatTheBatchOwes(the, built);
//! ```

use crate::service_journal::write_start;
use crate::service_land_attempt_batch_built::WhatTheBatchBuilt;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_start::witness_of;
use std::time::Instant;

/// What:
///  Takes everything the batch owes while the index lock is held,
///  and records the index file the landing saw.
/// Why:
///  The owed records are what a kill after the move is recovered from,
///  and the witness is what a replay of this batch must still find before it resets a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeWhatTheBatchOwes(the: TheAttempt, built: WhatTheBatchBuilt): void
/// ```
pub(crate) fn take_what_the_batch_owes(
    the: &mut TheAttempt<'_>,
    built: &WhatTheBatchBuilt,
) {
    let owed_began: Instant = Instant::now();
    // The programs each request's shell finds its programs through, and whether it chose the transport they reach the
    //  remote through, are held against the push record its commit becomes; nothing of them is written.
    let programs: Vec<Option<crate::service_push_programs_held_in_memory::PushPrograms>> = the
        .batch
        .iter()
        .map(|request| {
            return request.programs.clone();
        })
        .collect();
    crate::service_server_land_group::what_a_batch_owes(&built.intent, programs.as_slice());
    the.timing.reads_split.owed += owed_began.elapsed();
    // The index file now, under the lock, is what a replay of this batch must still find before it resets a path: a replay is
    //  made only over the file the landing saw. It is recorded for a commit Git built as for any other, since the witness
    //  needs no list of paths. A record that cannot be written leaves a batch no replay is made for, and this landing
    //  still updates the index itself while it holds the lock.
    let witness_began: Instant = Instant::now();
    let seq: u64 = the.seq;
    let repository: &std::path::Path = the.repository;
    match witness_of(the.first.index.as_path()) {
        Ok(witness) => {
            if let Err(error) = write_start(the.context.journal.as_path(), seq, &witness) {
                crate::service_error!(
                    "batch {seq} {}: the index file the landing saw could not be recorded ({error}); a restart will not replay the index update",
                    crate::service_timing::in_the_repository_at(repository)
                );
            }
        }
        Err(error) => crate::service_error!(
            "batch {seq} {}: the index file cannot be looked at ({error}); a restart will not replay the index update",
            crate::service_timing::in_the_repository_at(repository)
        ),
    }
    the.timing.reads_split.witness += witness_began.elapsed();
}
