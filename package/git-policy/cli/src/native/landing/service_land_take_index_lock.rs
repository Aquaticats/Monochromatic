//! What:
//!  Takes the index lock for a batch that is ready to land,
//! notes the wait in the timing and the log,
//! starts making the markers of the two batches after it,
//! and cleans up when the lock is refused.
//! Why:
//!  It is a self-contained step of a landing,
//! and a file of its own keeps `land_batch_body` short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = await takeIndexLock(...)
//! ```

use std::time::Duration;
use crate::service_land_git_steps_and_import_stream::run;
use crate::service_land_side_reference_write::keep_unlanded;
use crate::journald_native::Priority;
use crate::service_journal::OPEN_EXTENSION;
use crate::service_index_lock::{Announce, Announcer, IndexLock, WaitNotes, acquire_noting_wait_within};
use crate::service_timing::{LandTiming, wait_line};
use crate::service_protocol::{CommitRequest, FailureCode};
use crate::transaction_git::{GitContext, NO_REFLOG_SETTING};
use std::path::Path;
use std::time::Instant;

/// What:
///  Takes the index lock for a batch,
/// noting how long it waited and who held it;
/// when the lock is refused the intent is forgotten,
///  the built commit is kept on a side ref and the scratch branch is deleted.
/// Why:
///  The lock is asked for after the objects and the intent are made,
/// so it is held for the branch move and the index build only;
///  a refusal leaves nothing that a restart of the service could land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await takeIndexLock(git, journal, lockCeiling, first, seq, announce, timing, newTip, scratch) // throws a failure code
/// ```
pub(crate) fn take_index_lock(
    git: &GitContext,
    journal: &Path,
    lock_ceiling: Duration,
    first: &CommitRequest,
    seq: u64,
    announce: &dyn Announcer,
    timing: &mut LandTiming,
    new: &str,
    scratch: &str,
) -> Result<IndexLock, FailureCode> {
    let repository: &Path = first.repository.as_path();
    let waiting_since: Instant = Instant::now();
    let mut notes: WaitNotes = WaitNotes::default();
    let taken = acquire_noting_wait_within(first.index.as_path(), seq, lock_ceiling, announce, &mut notes);
    let waited: std::time::Duration = waiting_since.elapsed();
    timing.lock_wait += waited;
    // A pass over the batch that found the lock free has nothing to add to what an earlier pass found held.
    if notes.looks > 0 || timing.attempts == 1 {
        timing.lock_holder.clone_from(&notes.holder);
        timing.lock_notes = notes.clone();
    }
    // The wait is written down when it ends, not with the answer, so a landing that fails later still leaves it.
    if let Some(line) = wait_line(seq, repository, waited, &notes, taken.is_ok()) {
        // A lock that was refused fails the commit, so the line stands out from a wait that ended in the lock.
        let priority: Priority = if taken.is_ok() { Priority::Info } else { Priority::Error };
        crate::service_timing::log_entry(priority, line.as_str(), &[("CLI_GIT_BATCH", seq.to_string())]);
    }
    match taken {
        Ok(lock) => {
            // What:
            //  The markers of the two batches after this one are started now,
            // while this landing holds the lock and moves the branch.
            // Why:
            //  The making is seconds long on a loaded disk when its flush is the caller's,
            // and it used to start after the landing's answer,
            // so the next commit
            //  met it still running (`lock_stage_kind=pre`,
            // 9 to 13 s measured 2026-11-10).
            //  Starting them here spends the landing's own seconds on them and leaves the next commit a finished marker usually,
            //  and the one after it the spare.
            //  The batches after this one land on this index and are numbered one and two more, which is known now.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const held = await takeIndexLock(...); prestage.start(first.index, seq + 1); prestage.start(first.index, seq + 2);
            // ```
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(1));
            crate::service_index_prestage::start(first.index.as_path(), seq.saturating_add(2));
            return Ok(lock);
        }
        Err(failure) => {
            // Nothing moved: forget the intent, so a restart of the service cannot land the batch later (owner, 2026-11-03: "No,
            //  keep it for a retry"), keep the built commit on a side ref the message names, and forget the scratch branch.
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(journal, seq, OPEN_EXTENSION));
            if failure.code == FailureCode::IndexInUse {
                announce.announce(Announce::Waiting(keep_unlanded(git, repository, seq, new)));
            }
            let _deleted = run(git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch], None);
            return Err(failure.code);
        }
    }
}

/// Controls for taking the index lock stay out of the release executable.
#[cfg(test)]
#[path = "service_land_take_index_lock_tests.rs"]
mod tests;
