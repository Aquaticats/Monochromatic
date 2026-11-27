//! What:
//!  What one attempt of a landing reads before it writes anything:
//!  the commit the branch names,
//!  whether the worktree still names that branch,
//!  the parents an amend replaces,
//!  the places that still want their commit,
//!  and whether the content of their batch is still in the object store.
//! Why:
//!  Every answer here decides whether the attempt writes at all,
//!  so the reads stand together before the first write,
//!  and the body reads only the one word they come to
//!  (`service_land_attempt_values.rs` holds the values they read and write).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! switch (readWhatTheBranchHolds(the)) {
//!   case 'land': break;
//!   case 'again': continue;
//!   case 'finish': return answer;
//! }
//! ```

use crate::service_built_checks::{
    BuiltCommit,
    read_commit,
};
use crate::service_head::still_checked_out;
use crate::service_land_attempt_values::TheAttempt;
use crate::service_land_git_steps_and_import_stream::{
    checked,
    requests_that_land,
};
use crate::service_land_outcomes::Outcome;
use crate::service_land_requests_whose_client_is_gone::{
    Wanted,
    places_still_wanted,
    take,
};
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
};
use crate::service_pruned::batch_content_missing;
use std::time::Instant;

/// What the reads of one attempt came to.
pub(crate) enum WhatTheReadsSay {
    /// Every place still wants its commit,
    /// and nothing this attempt needs is gone.
    Land,
    /// A place's client went away,
    /// so the batch is built again from the rest.
    Again,
    /// The batch lands nothing,
    /// and this is the answer the landing gives.
    Finish(Vec<Outcome>),
}

/// What:
///  The commit the branch names,
///  the parents an amend replaces,
///  the places that still want their commit with their requests,
///  and whether their content is still there.
/// Why:
///  Nothing is written before these answers are known:
///  a batch that lands nothing must not leave a record,
///  and a batch whose content a prune removed is refused before it is imported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readWhatTheBranchHolds(the: TheAttempt): 'land' | 'again' | { answer: Outcome[] }
/// ```
///
/// # Errors
///
/// The failure of asking Git for the branch's commit,
/// for the parents of the commit an amend replaces,
/// or for the paths that changed since each base;
/// and `Unavailable` when the content a prune removed is gone.
pub(crate) fn read_what_the_branch_holds(
    the: &mut TheAttempt<'_>
) -> Result<WhatTheReadsSay, FailureCode> {
    // The commit the branch names, asked in the one spelling the in-process readers answer (`<ref>^{commit}`), so the question
    //  starts no Git; for a branch, which only ever names a commit, the answer is the same as for the bare name.
    let branch_began: Instant = Instant::now();
    let branch_value: [std::ffi::OsString; 4] =
        crate::service_land_branch_move_transaction::branch_commit_question(&the.first.reference);
    let old: String = match checked(&the.git, the.repository, &branch_value, None) {
        Ok(found) => found,
        Err(_) if the.unborn => the.first.base.clone(),
        Err(code) => return Err(code),
    };
    the.old = old;
    if the.alone && the.first.base != the.old {
        return Ok(WhatTheReadsSay::Finish(vec![Outcome::BranchMoved]));
    }
    // Native Git updates `HEAD`, so a worktree that switched branches while the commit waited fails the commit with nothing
    //  changed. The named branch would move all the same, and the index built for it would replace the other branch's. A
    //  symbolic `HEAD` is not refused by Git's own compare when it is told not to follow it (measured: it replaces the
    //  symbolic reference), so a head that became a branch is found here, before the move. The same question is asked
    //  again once the index lock is held, because the worktree can switch while the lock is awaited.
    let still_ours: bool = still_checked_out(
        &the.git,
        the.repository,
        the.first.git_dir.as_path(),
        the.first.common_dir.as_path(),
        &the.first.reference,
    )?;
    the.timing.reads_split.branch += branch_began.elapsed();
    if !still_ours {
        return Ok(WhatTheReadsSay::Finish(vec![
            Outcome::BranchMoved;
            the.requests.len()
        ]));
    }
    // Only an amend uses the parents of the commit it replaces; they are read from the object store, and listed by Git when the
    //  store cannot say (the same rule as for the commit Git built).
    let parents_began: Instant = Instant::now();
    let amended_parents: Vec<String> = if the.alone && !the.unborn && the.first.amend {
        let replaced_commit: Option<BuiltCommit> = if the.settings.plain_history {
            read_commit(
                &the.git,
                the.repository,
                the.first.common_dir.as_path(),
                the.first.base.as_str(),
            )
        } else {
            None
        };
        match replaced_commit {
            Some(replaced) => replaced.parents,
            None => {
                let listing: String = checked(
                    &the.git,
                    the.repository,
                    &[
                        "rev-list",
                        "--parents",
                        "--max-count=1",
                        the.first.base.as_str(),
                    ],
                    None,
                )?;
                listing
                    .split_whitespace()
                    .skip(1)
                    .map(str::to_string)
                    .collect()
            }
        }
    } else {
        Vec::new()
    };
    the.amended_parents = amended_parents;
    the.timing.reads_split.parents += parents_began.elapsed();
    // One tree comparison per distinct base: the paths that changed on the branch since then.
    the.outcomes = vec![
        Outcome::RetryPrepare {
            base: the.old.clone()
        };
        the.requests.len()
    ];
    let changed_began: Instant = Instant::now();
    let landing: Vec<(usize, &CommitRequest)> = places_still_wanted(
        requests_that_land(&the.git, the.repository, the.requests, the.old.as_str())?,
        the.gone.as_slice(),
    );
    the.landing = landing;
    the.timing.reads_split.changed += changed_began.elapsed();
    if the.landing.is_empty() {
        return Ok(WhatTheReadsSay::Finish(std::mem::take(&mut the.outcomes)));
    }
    // What:
    //  A request whose client went away before its commit landed is not made,
    //  and the rest of the batch is built again without it.
    // Why:
    //  A commit exists for the person who asked for it,
    //  and nothing reads the answer of a client that is gone.
    // Nothing of this attempt is written yet,
    //  so the batch is built again from the rest.
    match take(
        &mut the.outcomes,
        &mut the.gone,
        &the.landing,
        the.announce,
        the.requests.len(),
    ) {
        Wanted::All => {}
        Wanted::Some => return Ok(WhatTheReadsSay::Again),
        Wanted::None => {
            return Ok(WhatTheReadsSay::Finish(std::mem::take(&mut the.outcomes)));
        }
    }
    the.batch.clear();
    for (_index, request) in &the.landing {
        the.batch.push(*request);
    }
    // Content that `git gc --prune=now` removed after the preparation is found before anything is imported or moved: the
    //  client prepares the commit again, and says so plainly when that is not enough.
    let content_began: Instant = Instant::now();
    let missing: bool = batch_content_missing(
        &the.git,
        the.repository,
        the.first.common_dir.as_path(),
        the.batch.as_slice(),
    );
    the.timing.reads_split.content += content_began.elapsed();
    if missing {
        return Err(FailureCode::Unavailable);
    }
    return Ok(WhatTheReadsSay::Land);
}
