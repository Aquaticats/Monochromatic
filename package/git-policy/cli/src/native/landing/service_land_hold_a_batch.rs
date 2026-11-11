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
use crate::transaction_git::{GitContext, NO_REFLOG_SETTING};
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
            crate::service_log!("batch {} is held behind a reference lock: {}", refused.seq, refused.words);
            let objects: PathBuf = refused.first.common_dir.join("objects");
            let short: String = short_name(refused.git, refused.repository, objects.as_path(), None, refused.new);
            let held_now: usize = list(refused.first.git_dir.as_path()).0.len();
            let answer: String = answer(&held_batch, short.as_str(), held_now);
            refused.announce.announce(Announce::Holder(waiting(&held_batch, short.as_str())));
            return Ok(vec![Outcome::Held { commit: refused.new.to_string(), words: answer }; refused.requests]);
        }
        // A record that cannot be written is a commit that could not be made durable: the words are the lock's own, and nothing
        //  was changed.
        Err(error) => {
            crate::service_error!(
                "batch {} could not be held behind a reference lock: its record cannot be written ({error}); {}",
                refused.seq,
                refused.words
            );
            refused.announce.announce(Announce::Holder(refused.words));
            drop_the_batch(&DropTheBatch {
                git: refused.git,
                context: refused.context,
                repository: refused.repository,
                seq: refused.seq,
                intent: refused.intent,
                scratch: refused.scratch,
            });
            return Err(FailureCode::ReferenceLocked);
        }
    }
}

/// What:
///  Puts what the landing waits for and how long in the journal.
/// Why:
///  A person reading the journal sees the lock a commit met,
///  and the bound is the margin the lock is judged at,
///  so the line names the seconds a holder has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// warn(`batch ${seq} was refused the write of ${ref} while the branch still names ${old}, so a reference lock stands in the way; the landing waits for it until ${REFERENCE_LOCK_WAIT}s after the first refusal`)
/// ```
pub(crate) fn say_it_waits(seq: u64, reference: &crate::ref_name::RefName, old: &str, within_bound: bool) {
    if !within_bound {
        return;
    }
    crate::service_warn!(
        "batch {seq} was refused the write of {} while the branch still names {old}, so a reference lock stands in the way; the landing waits for it until {} seconds after the first refusal",
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
    /// The branch,
    /// by the bytes Git printed for its name.
    pub reference: &'a crate::ref_name::RefName,
    /// What Git said when it refused the write.
    pub stderr: &'a [u8],
    /// The clients of the batch.
    pub announce: &'a dyn Announcer,
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
    crate::service_error!("batch {} could not move {}: {said}", given.seq, given.reference);
    given.announce.announce(Announce::Holder(format!("The wait for the lock has ended, and Git said: {said}")));
    drop_the_batch(&DropTheBatch { git: given.git, context: given.context, repository: given.repository, seq: given.seq, intent: given.intent, scratch: given.scratch });
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
}

/// What:
///  Gives back everything a batch that will never land took:
///  its intent,
///  what it owed,
///  the witness of the index it saw,
///  and the side reference that held its commit.
/// Why:
///   Nothing of it will land,
///  so no record of it survives
///  and the objects it imported are left to Git's own pruning,
///  as they were before a commit could be held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(intent); forgetWhatABatchOwes(intent); rm(start); updateRef(['-d', scratch])
/// ```
pub(crate) fn drop_the_batch(drop: &DropTheBatch<'_>) {
    forget_the_batch_that_will_not_land(drop);
    let _deleted = crate::service_land_git_steps_and_import_stream::run(drop.git, drop.repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", drop.scratch], None);
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
    drop_the_batch(drop);
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
