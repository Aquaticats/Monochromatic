//! What:
//!  What one attempt of a landing does once its branch moved:
//!  the head read again,
//!  the index installed from what the batch built,
//!  the done mark of a settled batch,
//!  and the answer every landed place gets.
//! Why:
//!   Git refuses a `symref-verify HEAD` beside an update of the branch `HEAD` names,
//!  so a switch after the last check cannot be caught inside the transaction and is read here instead;
//!  the commits exist whatever happens to the index,
//!  which recovery finishes.
//!  This is the one step of a landing that ends every place's answer,
//!  so it stands after the branch write and after a refusal was judged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return finishTheAttempt(the, built, held);
//! ```

use crate::service_fault::reach;
use crate::service_head::follows_move;
use crate::service_land_attempt_batch_built::WhatTheBatchBuilt;
use crate::service_land_attempt_index_lock::Held;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_finish::finish_after_move;
use crate::service_land_outcomes::Outcome;
use crate::service_protocol::FailureCode;
use std::time::Instant;

/// What:
///  Reads the head again,
///  installs the index,
///  marks the batch done when it settled,
///  and answers every place with the commit that landed.
/// Why:
///  A worktree that left the branch keeps its own index,
///  so the head is read before the index is written,
///  and a batch whose index install did not settle is left for a restart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishTheAttempt(the: TheAttempt, built: WhatTheBatchBuilt, held: Held): Outcome[]
/// ```
pub(crate) fn finish_the_attempt(
    the: &mut TheAttempt<'_>,
    built: WhatTheBatchBuilt,
    held: Held,
) -> Vec<Outcome> {
    reach("after-ref");
    // Git refuses a `symref-verify HEAD` beside an update of the branch `HEAD` names, so a switch after the last check cannot
    //  be caught inside the transaction. The head is read again with the lock held: a worktree that left the branch keeps
    //  its own index, and the commits stay on the branch they were made for. A switch after this look is the state native
    //  Git leaves too (`git commit` then `git symbolic-ref HEAD <other>`): the index describes the commit made.
    let follow_began: Instant = Instant::now();
    let followed: Result<bool, FailureCode> = follows_move(
        &the.git,
        the.repository,
        the.first.git_dir.as_path(),
        the.first.common_dir.as_path(),
        &the.first.reference,
        built.new.as_str(),
    );
    the.timing.reads_split.follow += follow_began.elapsed();
    // The branch has moved: the commits exist whatever happens to the index, which recovery finishes.
    let index_update_began: Instant = Instant::now();
    let settled: bool = finish_after_move(
        &the.git,
        &built.intent,
        held.lock,
        the.context.journal.as_path(),
        followed,
    );
    the.timing.index_update += index_update_began.elapsed();
    the.timing.hold += held.since.elapsed();
    if settled {
        reach("after-index");
        let _marked = crate::service_journal::mark_done(the.context.journal.as_path(), the.seq);
    }
    for (slot, (index, _request)) in the.landing.iter().enumerate() {
        the.outcomes[*index] = Outcome::Landed {
            commit: built.commits[slot].clone(),
        };
    }
    return std::mem::take(&mut the.outcomes);
}
