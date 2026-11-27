//! What:
//!  What one attempt of a landing does when its branch write did not happen:
//!  it waits out a reference lock that refused it and tries the write again,
//!  or holds the commit behind the lock,
//!  or ends the batch because the branch moved under it.
//! Why:
//!  A branch that is no longer where the batch found it is the race the loop is built for,
//!  and a branch that still names it did not move,
//!  so something else refused the write
//!  (`service_land_hold_a_batch.rs`,
//! `service_land_reference_lock.rs`).
//!  The index lock is given up while a wait runs,
//!  so another program's commit is not held behind this one,
//!  and the one bound every wait shares lives in the values of the attempt.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! switch (judgeTheRefusedMove(the, built, moved, held)) {
//!   case 'writeAgain': continue;
//!   case 'again': continue;
//!   case 'finish': return answer;
//! }
//! ```

use crate::service_index_lock::IndexLock;
use crate::service_journal::{
    OPEN_EXTENSION,
    batch_path,
    remove_start,
};
use crate::service_land_attempt_batch_built::WhatTheBatchBuilt;
use crate::service_land_attempt_index_lock::Held;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_git_steps_and_import_stream::checked;
use crate::service_land_hold_a_batch::{
    ARefusedBatch,
    GivenUp,
    give_up,
    hold_the_batch,
    say_it_waits,
};
use crate::service_land_outcomes::Outcome;
use crate::service_land_reference_lock::{
    Waiting,
    wait_out_the_reference_locks,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::GitOutput;
use std::time::Instant;

/// What a refused branch write of one attempt came to.
pub(crate) enum WhatTheRefusalSays {
    /// Nothing stands in the way any more and the bound for waiting has not passed,
    /// so the branch is where it was and the write is tried again.
    WriteAgain,
    /// The branch moved under the batch,
    /// which is built again with the index lock this attempt still holds.
    Again(IndexLock),
    /// The attempt is over,
    /// and this is what the landing answers.
    Finish(Result<Vec<Outcome>, FailureCode>),
}

/// What:
///  Judges a branch write that did not happen:
///  a lock still standing holds the commit,
///  a lock waited out past its bound gives the refusal back to the person,
///  and a branch that moved builds the batch again.
/// Why:
///  Neither a lock nor a wait for one is a branch that changed,
///  and the person is told which of them the attempt met
///  (`service_land_hold_a_batch.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judgeTheRefusedMove(the: TheAttempt, built: WhatTheBatchBuilt, moved: GitOutput, held: Held): WhatTheRefusalSays
/// ```
///
/// # Errors
///
/// The failure of asking Git what the branch names now.
pub(crate) fn judge_the_refused_move(
    the: &mut TheAttempt<'_>,
    built: &WhatTheBatchBuilt,
    moved: &GitOutput,
    held: Held,
) -> Result<WhatTheRefusalSays, FailureCode> {
    let Held {
        lock,
        since,
    } = held;
    // The branch is asked again in the same spelling the reads of this attempt asked it.
    let branch_value: [std::ffi::OsString; 4] =
        crate::service_land_branch_move_transaction::branch_commit_question(&the.first.reference);
    let standing: String =
        checked(&the.git, the.repository, &branch_value, None).unwrap_or_default();
    if standing == the.old {
        // The index lock is given up while the wait runs, so another program's commit is not held behind this one.
        drop(lock);
        let deadline: Instant = match the.lock_deadline {
            Some(known) => known,
            None => {
                let first_refusal: Instant =
                    Instant::now() + crate::wait_ceilings::REFERENCE_LOCK_WAIT;
                the.lock_deadline = Some(first_refusal);
                first_refusal
            }
        };
        let within_bound: bool = Instant::now() < deadline;
        say_it_waits(
            the.seq,
            the.repository,
            &the.first.reference,
            the.old.as_str(),
            within_bound,
        );
        match wait_out_the_reference_locks(
            the.lock_roots.as_slice(),
            &crate::stale_ref_locks::Judge::here(),
            deadline,
        ) {
            // Nothing stands any more and the bound for waiting on one has not passed: the branch is where it was,
            //  so the write may be tried again.
            Waiting::Gone if within_bound => {
                the.timing.hold += since.elapsed();
                let _intent_removed = std::fs::remove_file(batch_path(
                    the.context.journal.as_path(),
                    the.seq,
                    OPEN_EXTENSION,
                ));
                crate::service_server_land_group::forget_what_a_batch_owes(&built.intent);
                remove_start(the.context.journal.as_path(), the.seq);
                let _let_go = crate::service_land_side_reference_write::let_go_of_a_reference_only_while_it_names(&the.git, the.repository, the.scratch.as_str(), built.new.as_str());
                return Ok(WhatTheRefusalSays::WriteAgain);
            }
            // The bound has passed and nothing stands now: the refusal this batch keeps meeting is Git's own, and
            //  its words are what the person is told (`service_land_hold_a_batch.rs`).
            Waiting::Gone => {
                return Ok(WhatTheRefusalSays::Finish(Err(give_up(&GivenUp {
                    git: &the.git,
                    context: the.context,
                    repository: the.repository,
                    seq: the.seq,
                    intent: &built.intent,
                    scratch: the.scratch.as_str(),
                    reference: &the.first.reference,
                    stderr: moved.stderr.as_slice(),
                    announce: the.announce,
                }))));
            }
            // The owner chose "Make it, hold until lock goes" (2026-11-09): the commit is written and kept, the command
            //  exits 0, and the next command that finds the lock gone moves the branch (`service_land_hold_a_batch.rs`).
            Waiting::Blocked {
                words,
                locks,
            } => {
                let refused: ARefusedBatch<'_> = ARefusedBatch {
                    git: &the.git,
                    context: the.context,
                    repository: the.repository,
                    first: the.first,
                    seq: the.seq,
                    intent: &built.intent,
                    scratch: the.scratch.as_str(),
                    new: built.new.as_str(),
                    reflog: &built.reflog,
                    locks,
                    words,
                    announce: the.announce,
                    timing: &mut *the.timing,
                    held_at: since,
                    requests: the.requests.len(),
                };
                return Ok(WhatTheRefusalSays::Finish(hold_the_batch(refused)));
            }
        }
    }
    // The branch moved under the batch: forget the intent, what the batch owes and the scratch branch, and build it again.
    let _intent_removed = std::fs::remove_file(batch_path(
        the.context.journal.as_path(),
        the.seq,
        OPEN_EXTENSION,
    ));
    crate::service_server_land_group::forget_what_a_batch_owes(&built.intent);
    remove_start(the.context.journal.as_path(), the.seq);
    let _let_go =
        crate::service_land_side_reference_write::let_go_of_a_reference_only_while_it_names(
            &the.git,
            the.repository,
            the.scratch.as_str(),
            built.new.as_str(),
        );
    // For an amend, the next round finds the tip away from its base and answers that the branch moved.
    the.attempts_left -= 1;
    the.timing.hold += since.elapsed();
    if the.attempts_left == 0 {
        return Ok(WhatTheRefusalSays::Finish(Err(
            FailureCode::BranchKeepsChanging,
        )));
    }
    return Ok(WhatTheRefusalSays::Again(lock));
}
