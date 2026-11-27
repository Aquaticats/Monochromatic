//! What:
//!  The landing itself:
//! one batch of prepared commits written,
//! journaled and moved onto a branch.
//! Why:
//!  The entry points decide what is told to the clients and what a step ended at its ceiling records;
//!  the order of the durable writes and the branch move is the body's own,
//!  so it stands in a file that the entry points call and nothing else does.
//!  Each step of one attempt is a file of its own
//!  (`service_land_attempt_reads.rs` and the six beside it),
//!  so this file holds the order the steps run in and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await landBatchBody(context, seq, requests, announce, timing, settings);
//! ```

use crate::service_index_lock::Announcer;
use crate::service_land_attempt_after_the_move::finish_the_attempt;
use crate::service_land_attempt_batch_built::{
    WhatTheBatchBuilt,
    build_the_batch,
};
use crate::service_land_attempt_branch_move::move_the_branch;
use crate::service_land_attempt_index_lock::{
    Held,
    TheIndexLock,
    hold_the_index_lock,
};
use crate::service_land_attempt_owed_and_witness::take_what_the_batch_owes;
use crate::service_land_attempt_reads::{
    WhatTheReadsSay,
    read_what_the_branch_holds,
};
use crate::service_land_attempt_refused_move::{
    WhatTheRefusalSays,
    judge_the_refused_move,
};
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_context::LandContext;
use crate::service_land_outcomes::Outcome;
use crate::service_land_settings::Settings;
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
};
use crate::service_timing::LandTiming;
use crate::transaction_git::GitOutput;

/// The landing itself;
/// `land_batch_timed` adds what is told when a step is ended at its ceiling.
///
/// # Errors
///
/// As `land_batch_timed`.
pub(crate) fn land_batch_body(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Announcer,
    timing: &mut LandTiming,
    settings: &Settings<'_>,
) -> Result<Vec<Outcome>, FailureCode> {
    let mut the: TheAttempt<'_> =
        TheAttempt::of(context, seq, requests, announce, timing, settings)?;
    loop {
        the.timing.attempts += 1;
        // What an earlier attempt of this batch could not prove about its own reference says nothing about this one: a batch a
        //  moved branch sends back is built again, and this attempt writes the reference for itself.
        crate::service_land_without_its_scratch_reference::forget_why_it_had_none();
        match read_what_the_branch_holds(&mut the)? {
            WhatTheReadsSay::Land => {}
            WhatTheReadsSay::Again => continue,
            WhatTheReadsSay::Finish(answer) => return Ok(answer),
        }
        let built: WhatTheBatchBuilt = build_the_batch(&mut the)?;
        let held: Held = match hold_the_index_lock(&mut the, &built)? {
            TheIndexLock::Held(held) => held,
            TheIndexLock::Again => continue,
            TheIndexLock::Finish(answer) => return answer,
        };
        take_what_the_batch_owes(&mut the, &built);
        let moved: GitOutput = move_the_branch(&mut the, &built)?;
        if !moved.succeeded() {
            match judge_the_refused_move(&mut the, &built, &moved, held)? {
                WhatTheRefusalSays::WriteAgain => continue,
                WhatTheRefusalSays::Again(lock) => {
                    // The lock this attempt still held is given up where the attempt ended before.
                    drop(lock);
                    continue;
                }
                WhatTheRefusalSays::Finish(answer) => return answer,
            }
        }
        return Ok(finish_the_attempt(&mut the, built, held));
    }
}
