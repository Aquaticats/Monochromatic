//! What:
//!  The index lock of one attempt of a landing,
//!  the checked-out branch read again under it,
//!  and the places whose client went away while the lock was awaited.
//! Why:
//!  Git's own rule is that the index lock is held before the branch moves
//!  and given up when the new index is in place,
//!  so the lock is asked for only once everything that needs no index is done
//!  (`service_land_attempt_batch_built.rs`),
//!  and this file is the one step that decides whether the attempt goes on holding it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! switch (holdTheIndexLock(the, built)) {
//!   case 'held': break;
//!   case 'again': continue;
//!   case 'finish': return answer;
//! }
//! ```

use crate::service_head::still_checked_out;
use crate::service_index_lock::IndexLock;
use crate::service_land_attempt_batch_built::WhatTheBatchBuilt;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_hold_a_batch::{
    DropTheBatch,
    drop_the_batch,
};
use crate::service_land_outcomes::Outcome;
use crate::service_land_requests_whose_client_is_gone::{
    Wanted,
    take,
};
use crate::service_land_take_index_lock::take_index_lock;
use crate::service_protocol::FailureCode;
use std::time::Instant;

/// The index lock of one attempt,
/// and the instant the attempt began holding it.
pub(crate) struct Held {
    /// The lock itself,
    /// which the index build gives up.
    pub(crate) lock: IndexLock,
    /// When the lock was held,
    /// which the stretches from here to the move are measured from.
    pub(crate) since: Instant,
}

/// What the index lock of one attempt came to.
pub(crate) enum TheIndexLock {
    /// The lock is held,
    /// and every place still wants its commit.
    Held(Held),
    /// A place's client went away while the lock was awaited,
    /// so the batch is built again from the rest.
    Again,
    /// The attempt is over,
    /// and this is the answer the landing gives.
    Finish(Result<Vec<Outcome>, FailureCode>),
}

/// What:
///  Takes the index lock,
///  reads the checked-out branch again under it,
///  and takes out the places whose client went away in the wait.
/// Why:
///  The wait for the lock is where a person's interrupt lands,
///  so a commit whose client is gone by then is not made either,
///  and a worktree that switched branches meanwhile keeps its own index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdTheIndexLock(the: TheAttempt, built: WhatTheBatchBuilt): TheIndexLock
/// ```
///
/// # Errors
///
/// As `take_index_lock`.
pub(crate) fn hold_the_index_lock(
    the: &mut TheAttempt<'_>,
    built: &WhatTheBatchBuilt,
) -> Result<TheIndexLock, FailureCode> {
    // Git's own rule: the index lock is held before the branch is moved and given up when the new index is in place, so
    //  a foreign `git` that holds it finishes first and this landing builds on what it left. Everything that can be done
    //  without the index is done before the lock is asked for (the reads, the objects, the intent), so the lock is held
    //  for the branch move and the index build only, and a foreign holder is waited for while the work above is done,
    //  not before it. A lock that is not given up here (a refusal, a failure) is removed when it goes out of scope, and
    //  the index stays as it was.
    let held: IndexLock = take_index_lock(
        &the.git,
        the.context.journal.as_path(),
        the.context.lock_ceiling,
        the.first,
        the.seq,
        the.announce,
        &mut *the.timing,
        built.new.as_str(),
        the.scratch.as_str(),
    )?;
    let hold_began: Instant = Instant::now();
    // The worktree may have switched while the lock was awaited: the checked-out branch is read again with the lock held and
    //  before the ref moves, and a landing that no longer belongs to this worktree's head stops with nothing changed.
    let still: Result<bool, FailureCode> = still_checked_out(
        &the.git,
        the.repository,
        the.first.git_dir.as_path(),
        the.first.common_dir.as_path(),
        &the.first.reference,
    );
    if !matches!(still, Ok(true)) {
        let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(
            the.context.journal.as_path(),
            the.seq,
            crate::service_journal::OPEN_EXTENSION,
        ));
        crate::service_server_land_group::forget_what_a_batch_owes(&built.intent);
        let _let_go =
            crate::service_land_side_reference_write::let_go_of_a_reference_only_while_it_names(
                &the.git,
                the.repository,
                the.scratch.as_str(),
                built.new.as_str(),
            );
        return Ok(TheIndexLock::Finish(match still {
            Ok(_) => Ok(vec![Outcome::BranchMoved; the.requests.len()]),
            Err(code) => Err(code),
        }));
    }
    // What:
    //  The same question is asked with the index lock held,
    //  because the wait for another program's lock is where a person's interrupt lands.
    // Why:
    //  That wait can last a minute,
    //  and a commit whose client went away in it is not made either.
    //  What this attempt took is given back and the batch is built again from the rest.
    let left: Wanted = take(
        &mut the.outcomes,
        &mut the.gone,
        &the.landing,
        the.announce,
        the.requests.len(),
    );
    if left != Wanted::All {
        drop(held);
        drop_the_batch(&DropTheBatch {
            git: &the.git,
            context: the.context,
            repository: the.repository,
            seq: the.seq,
            intent: &built.intent,
            scratch: the.scratch.as_str(),
        });
        if left == Wanted::None {
            return Ok(TheIndexLock::Finish(Ok(std::mem::take(&mut the.outcomes))));
        }
        return Ok(TheIndexLock::Again);
    }
    return Ok(TheIndexLock::Held(Held {
        lock: held,
        since: hold_began,
    }));
}
