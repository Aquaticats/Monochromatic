//! What:
//!  What a landing does when a reference lock keeps its branch where it is:
//!  it publishes the record of the hold,
//!  keeps the intent and the side reference that hold the commit,
//!  and answers the client with the words the person reads.
//! Why:
//!   The owner chose "Make it,
//!  hold until lock goes (Recommended)" (2026-11-09),
//!  and the rule it answers is "Commits should never fail."
//!  The commit is written and kept,
//!  the command exits `0`,
//!  and the next command that finds the lock gone moves the branch
//!  (`held_commit_settle.rs`).
//!  This file holds that arm of the landing,
//!  and the two ways a batch ends without landing,
//!  so the landing's body stays under the file limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! return await holdABatch({ ...landing, locks }) // one held outcome per request, or the failure that names the lock
//! ```

use crate::held_commit_record::{Holding, hold, list};
use crate::held_commit_words::{answer, waiting};
use crate::landed_summary::short_name;
use crate::service_index_lock::{Announce, Announcer};
use crate::service_journal::{Intent, OPEN_EXTENSION, batch_path, remove_start};
use crate::service_land_context::LandContext;
use crate::service_land_outcomes::Outcome;
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::service_server_land_group::forget_what_a_batch_owes;
use crate::service_timing::LandTiming;
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What a landing knows when a reference lock kept its branch where it was.
pub struct ARefusedBatch<'a> {
    /// How the landing runs Git.
    pub git: &'a GitContext,
    /// The journal and the Git settings of the landing.
    pub context: &'a LandContext,
    /// The worktree root.
    pub repository: &'a Path,
    /// The first request of the batch.
    pub first: &'a CommitRequest,
    /// The batch number.
    pub seq: u64,
    /// The batch's intent,
    /// written before the branch write.
    pub intent: &'a Intent,
    /// The side reference that holds the batch's commits.
    pub scratch: &'a str,
    /// The commit the batch would put on the branch.
    pub new: &'a str,
    /// The message the branch move writes in the reflog.
    pub reflog: &'a OsString,
    /// The locks that stood when the write was refused.
    pub locks: Vec<PathBuf>,
    /// The words the judgement gives those locks.
    pub words: String,
    /// The clients of the batch.
    pub announce: &'a dyn Announcer,
    /// The landing's clock.
    pub timing: &'a mut LandTiming,
    /// When the landing took the index lock,
    /// which the hold and the wait are counted from.
    pub held_at: Instant,
    /// How many requests the batch carries.
    pub requests: usize,
}

/// What:
///  Holds the batch behind the locks that stand,
///  and answers one held outcome per request.
/// Why:
///   The record is published before the client is answered,
///  so a kill at any instruction leaves either no record and the batch's intent
///  (which a restart settles from the branch,
///  keeping the commit on `refs/cli-git/kept/<batch>`)
///  or a record that names the batch.
///  Nothing is removed here:
///  the lock is left where it is,
///  and the side reference and the intent stay,
///  because the branch is theirs to move yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await hold(recordOf(refused)); forgetThePushes(); return requests.map(() => heldOutcome(held))
/// ```
///
/// # Errors
///
/// `ReferenceLocked` when the record cannot be written,
/// which is the one way such a commit still ends,
/// with the lock named.
pub(crate) fn hold_the_batch(refused: ARefusedBatch<'_>) -> Result<Vec<Outcome>, FailureCode> {
    let holding: Holding<'_> = Holding {
        journal: refused.context.journal.as_path(),
        repository: refused.repository,
        common_dir: refused.first.common_dir.as_path(),
        git_dir: refused.first.git_dir.as_path(),
        index: refused.first.index.as_path(),
        reference: &refused.first.reference,
        seq: refused.seq,
        intent: refused.intent,
        locks: refused.locks,
        reflog: crate::path_bytes::path_bytes_of(Path::new(refused.reflog.as_os_str())),
    };
    match hold(holding) {
        Ok(held_batch) => {
            refused.timing.hold += refused.held_at.elapsed();
            // Nothing landed, so nothing of this batch is owed: no push of it may run, and no index update is owed, since its
            //  branch has not moved. Both records are taken again when the branch actually moves (`held_commit_settle.rs`), and
            //  one left here would say a commit no branch names waits for a push, or that a commit that landed did not finish
            //  the index when no commit landed at all.
            forget_what_a_batch_owes(refused.intent);
            crate::service_log!(
                "batch {} {} is held: the commit {} is kept, and {} has not moved yet because a reference lock stands in the way of its write. \
The branch moves as soon as the lock is gone. {}",
                refused.seq,
                crate::service_timing::in_the_repository_at(refused.repository),
                refused.new,
                refused.first.reference,
                refused.words
            );
            let objects: PathBuf = refused.first.common_dir.join("objects");
            let short: String = short_name(refused.git, refused.repository, objects.as_path(), None, refused.new);
            let held_now: usize = list(refused.first.git_dir.as_path()).0.len();
            let answer: String = answer(&held_batch, short.as_str(), held_now);
            refused.announce.announce(Announce::Holder(waiting(&held_batch, short.as_str(), held_batch.locks.as_slice())));
            return Ok(vec![Outcome::Held { commit: refused.new.to_string(), words: answer }; refused.requests]);
        }
        // A record that cannot be written is a commit that could not be made durable: the words are the lock's own, and nothing
        //  was changed.
        Err(error) => {
            crate::service_error!(
                "batch {} {} could not be held: the record that keeps its commit cannot be written ({error}), so the branch did not move and \
its commit is not kept; {} Run the commit again once the journal at {} can be written",
                refused.seq,
                crate::service_timing::in_the_repository_at(refused.repository),
                refused.words,
                refused.context.journal.display()
            );
            refused.announce.announce(Announce::Holder(refused.words));
            // The commit was built and is on no branch, so it is kept where a commit that did not land is kept: the record
            //  that would have held it could not be written, and the person is told where the commit is
            //  (`end_a_batch_that_will_not_land`).
            end_a_batch_that_will_not_land(&DropTheBatch {
                git: refused.git,
                context: refused.context,
                repository: refused.repository,
                seq: refused.seq,
                intent: refused.intent,
                scratch: refused.scratch,
                new: refused.new,
                announce: refused.announce,
            });
            return Err(FailureCode::ReferenceLocked);
        }
    }
}

/// What:
///  Puts what the landing waits for and how long in the journal.
/// Why:
///  A person reading the journal sees the lock a commit met,
///  the repository it belongs to,
///  and the bound is the margin the lock is judged at,
///  so the line names the seconds a holder has.
///  The commit is kept and nothing failed,
///  so the line never says the write was refused:
///  the line a lock raises says what waits,
/// what happens next,
/// and what,
/// if anything,
/// the person may do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// warn(`batch ${seq} in the repository at ${repo}: the write of ${ref} has not happened yet ...`)
/// ```
pub(crate) fn say_it_waits(seq: u64, repository: &Path, reference: &crate::ref_name::RefName, old: &str, within_bound: bool) {
    if !within_bound {
        return;
    }
    crate::service_warn!(
        "batch {seq} {}: the write of {} has not happened yet, because the branch still names {old}, so a reference lock \
stands in the way. The commit is kept, and this landing keeps trying for {} seconds from its first try: if the lock goes before that, \
this command moves the branch before it answers, and otherwise the next command in that worktree moves it. Nothing is needed from you \
unless no Git process is using that repository, and then removing the lock lets the commit land.",
        crate::service_timing::in_the_repository_at(repository),
        reference.spoken(),
        crate::wait_ceilings::REFERENCE_LOCK_WAIT.as_secs()
    );
}

/// What a landing knows when the wait for the locks ended and nothing stands:
/// the refusal this batch keeps meeting is Git's own.
pub struct GivenUp<'a> {
    /// How the landing runs Git.
    pub git: &'a GitContext,
    /// The journal and the Git settings of the landing.
    pub context: &'a LandContext,
    /// The worktree root.
    pub repository: &'a Path,
    /// The batch number.
    pub seq: u64,
    /// The batch's intent.
    pub intent: &'a Intent,
    /// The side reference that held the batch's commits.
    pub scratch: &'a str,
    /// The commit the batch built.
    pub new: &'a str,
    /// The branch,
    /// by the bytes Git printed for its name.
    pub reference: &'a crate::ref_name::RefName,
    /// What Git said when it refused the write.
    pub stderr: &'a [u8],
    /// The clients of the batch.
    pub announce: &'a dyn Announcer,
}

/// What:
///  Gives a batch that will never land back,
///  and keeps the commit it built where a commit that did not land is kept,
///  so a commit the person is told was not made is still findable.
/// Why:
///   A commit no branch names and no reference names is one `git gc --prune=now` takes,
///  and the person is told the commit was not made and where it is kept
///  (the owner's rule "never discard user data").
///  The reference the batch held it on goes only once the kept reference names the commit,
///  and stays when that keep cannot be written
///  (`keep_an_unlanded_commit`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = keepAnUnlandedCommit(...); forgetTheBatch(); if (kept.named) updateRef(['-d', scratch, new])
/// ```
pub(crate) fn end_a_batch_that_will_not_land(drop: &DropTheBatch<'_>) {
    let kept: crate::service_land_side_reference_write::AKeptCommit =
        crate::service_land_side_reference_write::keep_an_unlanded_commit(drop.git, drop.repository, drop.seq, drop.scratch, drop.new, drop.announce);
    forget_the_batch_that_will_not_land(drop);
    if kept.named {
        let _let_go = crate::service_land_side_reference_write::let_go_of_a_reference_only_while_it_names(drop.git, drop.repository, drop.scratch, drop.new);
    }
}

/// What:
///  Ends a batch whose refused write is Git's own,
/// not a lock's.
/// Why:
///  The wait ended and nothing stands,
///  so the words Git gave are what the person is told,
///  and the batch is given back with them.
///  This is the one way a commit whose branch write met a lock still ends with a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tellTheClients(gitSaid); dropTheBatch(); return ReferenceLocked
/// ```
pub(crate) fn give_up(given: &GivenUp<'_>) -> FailureCode {
    let said: String = String::from_utf8_lossy(given.stderr).trim().to_string();
    crate::service_error!(
        "batch {} {} could not move {}: {said}",
        given.seq,
        crate::service_timing::in_the_repository_at(given.repository),
        given.reference
    );
    given.announce.announce(Announce::Holder(format!("The wait for the lock has ended, and Git said: {said}")));
    end_a_batch_that_will_not_land(&DropTheBatch {
        git: given.git,
        context: given.context,
        repository: given.repository,
        seq: given.seq,
        intent: given.intent,
        scratch: given.scratch,
        new: given.new,
        announce: given.announce,
    });
    return FailureCode::ReferenceLocked;
}

/// What a landing gives back when its batch will never land.
pub struct DropTheBatch<'a> {
    /// How the landing runs Git.
    pub git: &'a GitContext,
    /// The journal and the Git settings of the landing.
    pub context: &'a LandContext,
    /// The worktree root.
    pub repository: &'a Path,
    /// The batch number.
    pub seq: u64,
    /// The batch's intent.
    pub intent: &'a Intent,
    /// The side reference that held the batch's commits.
    pub scratch: &'a str,
    /// The commit the batch built,
    /// the one value that reference is expected to name.
    pub new: &'a str,
    /// The clients of the batch.
    pub announce: &'a dyn Announcer,
}

/// What:
///  Gives back everything a batch that will never land took:
///  its intent,
///  what it owed,
///  the witness of the index it saw,
///  and the side reference that held its commit,
///  and only while that reference still names the commit this batch built.
/// Why:
///   Nothing of it will land,
///  so no record of it survives
///  and the objects it imported are left to Git's own pruning,
///  as they were before a commit could be held.
///  A command that reads a reference and then removes it by name alone removes whatever another command wrote in between,
///  and that may be the only name of another commit,
///  so the removal is Git's own compare-and-delete with the value that was read
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(intent); forgetWhatABatchOwes(intent); rm(start); updateRef(['-d', scratch, new])
/// ```
pub(crate) fn drop_the_batch(drop: &DropTheBatch<'_>) {
    forget_the_batch_that_will_not_land(drop);
    let _let_go = crate::service_land_side_reference_write::let_go_of_a_reference_only_while_it_names(drop.git, drop.repository, drop.scratch, drop.new);
}

/// What:
///  The ending of a batch whose branch write Git never started:
///  the branch did not move,
///  so nothing of the batch lands and everything it took is given back.
/// Why:
///  A step that ended at its ceiling keeps the commit the side reference holds before this
///  (`keep_what_an_ended_step_built`),
///  so that reference is left where the keeping reads it and only the records go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endABatchTheWriteNeverStarted(drop, code) // the records go; a ceiling step keeps the commit first
/// ```
pub(crate) fn end_a_batch_the_write_never_started(drop: &DropTheBatch<'_>, code: FailureCode) {
    if code == FailureCode::StepTooSlow {
        forget_the_batch_that_will_not_land(drop);
        return;
    }
    end_a_batch_that_will_not_land(drop);
}

/// What:
///  The intent,
/// what the batch owes and the witness it saw:
/// everything a batch that will never land gives back,
///  except the side reference that holds what it built.
/// Why:
///   A landing that gives up before its branch move removes those three as its last step;
///  a step ended at its ceiling keeps the commit the reference holds first (`keep_what_an_ended_step_built`),
///  so that caller takes this and leaves the reference where the keeping reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(intent); forgetWhatABatchOwes(intent); rm(start)
/// ```
pub(crate) fn forget_the_batch_that_will_not_land(drop: &DropTheBatch<'_>) {
    let _intent_removed = std::fs::remove_file(batch_path(drop.context.journal.as_path(), drop.seq, OPEN_EXTENSION));
    forget_what_a_batch_owes(drop.intent);
    remove_start(drop.context.journal.as_path(), drop.seq);
}

/// Controls for giving a batch back, and for keeping the commit it built as it is given back.
#[cfg(test)]
#[path = "service_land_hold_a_batch_tests.rs"]
mod tests;
