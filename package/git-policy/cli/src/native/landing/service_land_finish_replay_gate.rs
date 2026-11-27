//! What:
//!  Whether a restart may replay an open batch's index update,
//! and what it settles when it may:
//! the evidence the landing recorded about the index file it saw under its lock,
//! the head checked again under the lock the restart takes,
//! and the two words a settlement comes to.
//! Why:
//!  A replay runs after the lock the landing held was given up or lost,
//! so another program may have staged into the index since;
//! a replay is made only over the very index file the landing saw,
//! every other case leaves the index as its writers left it and says so,
//! and only the file that finishes a batch (`service_land_finish_after_the_move.rs`) touches what it owes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settled = settle(git, intent, journal); // 'done' | 'open'
//! ```

use crate::service_head::still_checked_out;
use crate::service_index_lock::try_acquire;
use crate::service_journal::{
    Indexed,
    Intent,
    Started,
    indexed_state,
    read_start,
};
use crate::service_land_finish_after_the_move::follow_up;
use crate::service_land_start::witness_of;
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  The settlement a finished step stands for:
/// done when it finished,
/// open when it did not.
/// Why:
///  `settle` turns the result of each step into the word the caller records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settledOf = (finished: boolean) => finished ? 'done' : 'open';
/// ```
fn settled_of(finished: bool) -> Settled {
    return if finished {
        Settled::Done
    } else {
        Settled::Open
    };
}

/// What a restart makes of the evidence for replaying an open batch's index update.
#[derive(Debug, Eq, PartialEq)]
enum Gate {
    /// The index file is the one the landing saw:
    /// nothing was staged since.
    Replay,
    /// The evidence cannot be read now:
    /// the batch stays open.
    Wait,
    /// The index may hold staging:
    /// it is left as it is,
    /// for this reason.
    Leave(&'static str),
}

/// What:
///  Whether the index update of the open batch may be replayed over the index as it is now,
/// judged under the lock the caller
///  holds.
/// Why:
///   The landing recorded the witness of the index file it saw under its lock.
/// A file that is still that file has been written
///  by nobody since,
/// and a replay over it discards nothing.
/// Any other index,
/// and a batch with no record,
/// may hold what another
///  session staged,
/// so no path of it is reset:
/// whether a path holds the entry the landing started from does not say whether it
///  was staged back to that entry,
/// and a reset also rewrites the entries under and above its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = gateOf(journal, intent) // 'replay' | 'wait' | { leave: reason }
/// ```
fn gate_of(
    journal: &Path,
    intent: &Intent,
) -> Gate {
    let recorded = match read_start(journal, intent.seq) {
        Started::Absent => return Gate::Leave("no record says which index file the landing saw"),
        Started::Unreadable => return Gate::Wait,
        Started::Recorded(witness) => witness,
    };
    return match witness_of(intent.index.as_path()) {
        Ok(now) if now == recorded => Gate::Replay,
        Ok(_) => Gate::Leave(
            "the index file is not the one the landing saw, so another program may have staged into it",
        ),
        Err(_) => Gate::Wait,
    };
}

/// What a restart made of one open batch.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Settled {
    /// Every obligation of the batch is settled.
    Done,
    /// Something is left,
    /// and the batch stays open.
    Open,
}

/// Settles one batch whose branch has moved,
/// found open.
pub(crate) fn settle(
    git: &GitContext,
    intent: &Intent,
    journal: &Path,
) -> Settled {
    let repository: &Path = intent.repository.as_path();
    let own_index: bool = intent.index == intent.git_dir.join("index");
    // A worktree that has since left the batch's branch has a default index that describes another branch: it is not updated.
    //  Skipping the index is safe whatever the head does next, so this look needs no lock. A head that cannot be read stops here.
    if own_index {
        match still_checked_out(
            git,
            repository,
            intent.git_dir.as_path(),
            intent.common_dir.as_path(),
            &intent.reference,
        ) {
            Ok(false) => {
                return settled_of(crate::service_land_follow::cleanup_done(
                    git, repository, intent, journal,
                ));
            }
            Ok(true) => {}
            Err(code) => {
                crate::service_error!(
                    "batch {} {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is",
                    intent.seq,
                    crate::service_timing::in_the_repository_at(repository)
                );
                return Settled::Open;
            }
        }
    }
    match indexed_state(journal, intent.seq, intent.new.as_str()) {
        // The installed index is recorded: it is not replayed, and only the conclusion state may be left.
        Indexed::Installed => {
            // The index was installed and recorded, so nothing is owed: a kill between that installation and the landing's own
            //  settling leaves the record behind, and this is where it goes.
            crate::owed_index_update::settle(intent.git_dir.as_path(), intent.seq);
            return settled_of(crate::service_land_follow::cleanup_done(
                git, repository, intent, journal,
            ));
        }
        // A record that cannot be read may say the index was installed: the batch stays open and the index alone.
        Indexed::Unreadable => {
            crate::service_error!(
                "batch {} {} stays open: its completion record cannot be read, so the index is left as it is",
                intent.seq,
                crate::service_timing::in_the_repository_at(repository)
            );
            return Settled::Open;
        }
        Indexed::Absent => {}
    }
    crate::service_fault::reach("restart-unlocked");
    let Some(held) = try_acquire(intent.index.as_path(), intent.seq) else {
        return Settled::Open;
    };
    crate::service_fault::reach("restart-locked");
    // Everything the replay depends on is judged again under the lock this restart holds: the head, then the index file.
    if own_index {
        match still_checked_out(
            git,
            repository,
            intent.git_dir.as_path(),
            intent.common_dir.as_path(),
            &intent.reference,
        ) {
            Ok(true) => {}
            Ok(false) => {
                drop(held);
                return settled_of(crate::service_land_follow::cleanup_done(
                    git, repository, intent, journal,
                ));
            }
            Err(code) => {
                drop(held);
                crate::service_error!(
                    "batch {} {} stays open: the head of the worktree cannot be read under the lock ({code:?}), so the index is left as it is",
                    intent.seq,
                    crate::service_timing::in_the_repository_at(repository)
                );
                return Settled::Open;
            }
        }
    }
    return match gate_of(journal, intent) {
        Gate::Replay => settled_of(follow_up(git, repository, intent, held, journal)),
        Gate::Wait => {
            drop(held);
            crate::service_error!(
                "batch {} {} stays open: the record of the index file its landing saw cannot be read now",
                intent.seq,
                crate::service_timing::in_the_repository_at(repository)
            );
            Settled::Open
        }
        Gate::Leave(reason) => {
            drop(held);
            crate::service_error!(
                "batch {} {}: the index entries of its paths were not updated and are left as they are, because {reason}; the record of what \
it owes is kept at {}, and the next commit in the worktree sets those entries to the commit before it makes anything of this index, \
so `git status` shows the commit's paths against the index until then",
                intent.seq,
                crate::service_timing::in_the_repository_at(repository),
                crate::owed_index_update::path_of(intent.git_dir.as_path(), intent.seq).display()
            );
            settled_of(crate::service_land_follow::cleanup_done(
                git, repository, intent, journal,
            ))
        }
    };
}
