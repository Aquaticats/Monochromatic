//! What:
//!  What follows the branch move of a landing:
//! the conclusion state is removed,
//! then the index entries are updated,
//! the installed index is recorded in the journal while the lock is still held,
//! and a worktree that left the batch's branch leaves that index alone.
//! Why:
//!  Both steps are idempotent and are recorded apart,
//! so a restart never replays the index update over what another program staged after the lock was given up,
//! and the batch's own entry points (`service_land_finish.rs`)
//! and the restart that settles a batch (`service_land_finish_replay_gate.rs`)
//! ask this one routine to finish it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = finishAfterMove(git, intent, lock, journal, followsMove) // true: every obligation of the batch is settled
//! ```

use crate::autostash_carry::carry_stages;
use crate::service_index_lock::{
    IndexLock,
    Record,
};
use crate::service_journal::{
    Intent,
    mark_indexed,
};
use crate::service_land_follow::{
    IndexPaths,
    build_index,
};
use crate::service_protocol::FailureCode;
use crate::transaction_git::GitContext;
use std::path::Path;

/// The index entries a batch's intent names.
pub(crate) fn intent_paths(intent: &Intent) -> IndexPaths<'_> {
    return if intent.derived {
        IndexPaths::Derived {
            old: intent.old.as_str(),
        }
    } else {
        IndexPaths::Named(intent.paths.as_slice())
    };
}

/// What:
///  The record that the installed index is complete,
/// written to the batch's journal while the lock is still held.
/// Why:
///   A restart that finds this record never replays the index update over what another program staged after the lock was given up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RecordIndexed implements Record { record() { markIndexed(journal, intent.seq, intent.new); } }
/// ```
struct RecordIndexed<'a> {
    /// The directory of the batch's journal.
    journal: &'a Path,
    /// The batch whose index was installed.
    intent: &'a Intent,
}

impl Record for RecordIndexed<'_> {
    fn record(&self) -> std::io::Result<()> {
        return mark_indexed(self.journal, self.intent.seq, self.intent.new.as_str());
    }
}

/// Finishes what follows the branch move:
/// the conclusion state is removed,
/// then the index entries are updated.
/// Both are
///  idempotent,
/// and they are recorded apart:
/// the installed index is recorded in the journal while the lock is still held
///  (`<batch>.indexed`),
/// so a restart never replays the index update over what another program staged after the lock was given
///  up,
/// whether or not the conclusion state could be removed.
/// Whether both are done.
pub fn follow_up(
    git: &GitContext,
    repository: &Path,
    intent: &Intent,
    lock: IndexLock,
    journal: &Path,
) -> bool {
    let cleaned: bool = crate::service_land_follow::cleanup_done(git, repository, intent, journal);
    if crate::service_fault::fails("build-index") {
        return false;
    }
    // The new index is built beside the real one and installed by the release of the lock the landing holds.
    let Ok(built) = build_index(
        git,
        repository,
        intent.new.as_str(),
        intent_paths(intent),
        &lock,
        intent.seq,
    ) else {
        return false;
    };
    // The files a conflicted autostash apply left unmerged are unmerged in the index the landing installs.
    if let Some(cleanup) = intent.cleanup.as_ref() {
        // Git's `update-index` may write the copy, and does not flush it.
        built.set_flushed(false);
        if !carry_stages(git, repository, cleanup, built.path()) {
            // The private copy goes with `built`, which is dropped here.
            return false;
        }
    }
    // Every writer of the copy is done, so it takes the live index's times for the last time before it is installed:
    // an index newer than the stat data it holds hides a same-second edit from Git (`PrivateCopy::keep_the_live_times`).
    crate::service_land_follow::put_the_live_times_back(&built);
    let recorded: RecordIndexed<'_> = RecordIndexed {
        journal,
        intent,
    };
    let installed: bool = lock
        .release_flushed_with_record(built.path(), built.is_flushed(), &recorded)
        .is_ok();
    if installed {
        // The index holds the commit now, so nothing is owed any more; the record goes by a rename, so a kill here leaves either
        //  an owed record or none, never a settled obligation that still looks owed.
        crate::owed_index_update::settle(intent.git_dir.as_path(), intent.seq);
    }
    return installed && cleaned;
}

/// What:
///  Settles a batch whose branch has moved,
/// while its caller holds the index lock:
/// whether the worktree's head still follows the
///  move decides if the index is updated,
/// and the conclusion state is removed in every case.
/// Whether the batch is done.
/// Why:
///   A worktree that left the batch's branch has a default index that describes another branch,
/// so that index is not updated;
///  an index kept in another file (`GIT_INDEX_FILE`) belongs to no branch and is still finished.
/// A head that cannot be read
///  decides nothing:
/// the lock is given up,
/// the index is not touched,
/// and the batch stays open for a restart to settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await finishAfterMove(git, intent, lock, journal, followsMove)
/// ```
pub fn finish_after_move(
    git: &GitContext,
    intent: &Intent,
    lock: IndexLock,
    journal: &Path,
    follows_move: Result<bool, FailureCode>,
) -> bool {
    let repository: &Path = intent.repository.as_path();
    let own_index: bool = intent.index == intent.git_dir.join("index");
    return match follows_move {
        Ok(false) if own_index => {
            drop(lock);
            crate::service_land_follow::cleanup_done(git, repository, intent, journal)
        }
        Ok(_) => follow_up(git, repository, intent, lock, journal),
        Err(code) => {
            drop(lock);
            crate::service_error!(
                "batch {} {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is",
                intent.seq,
                crate::service_timing::in_the_repository_at(repository)
            );
            false
        }
    };
}
