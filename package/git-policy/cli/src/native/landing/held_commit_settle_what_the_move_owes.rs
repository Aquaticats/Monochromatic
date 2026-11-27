//! What:
//!  What a settlement owes after the branch of a held commit moved:
//!  the index lock the batch's entries are installed under,
//!  the index file the settlement itself sees,
//!  the install,
//!  and the conclusion state,
//!  the held push and the owed record that follow it.
//! Why:
//!   A held commit's landing gave Git's index lock up before it waited for the reference lock,
//!  and the person may have run commands since,
//!  so the witness it left describes an older file:
//!  taking the lock again and recording the index the settlement sees restores the landing's own guarantee
//!  instead of leaving the entries to the owed record
//!  (`owed_index_update.rs`),
//!  which a commit after a pathless index would otherwise be refused over.
//!  A lock another program holds is not waited for:
//!  the entries are left to that record and settle at the next command,
//!  so a kill between the move and the index update leaves nothing unsettled for good.
//!  It sits in a file of its own because the settlement was over the limit the repository linter enforces
//!  (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lock = indexLockOf(held, intent); const waits = finish(git, held, intent, lock);
//! ```

use crate::held_commit_record::{
    Held,
    forget,
};
use crate::held_commit_settle_outcomes::Settled;
use crate::service_journal::Intent;
use crate::transaction_git::GitContext;

/// What:
///  Takes the index lock the batch's entries are installed under,
///  and records the index file the settlement itself sees.
/// Why:
///   A landing holds Git's index lock from the witness of the index file to the installation of the entries
///  (`service_land_batch_body.rs`),
///  so nothing can stage in that window and a replay discards nothing.
///  A held commit's landing gave that lock up before it waited for the reference lock,
///  and the person may have run commands since,
///  so the witness it left describes an older file:
///  taking the lock again and recording the index the settlement sees restores the landing's own guarantee
///  instead of leaving the entries to the owed record
///  (`owed_index_update.rs`),
///  which a commit after a pathless index would otherwise be refused over.
///  A lock another program holds is not waited for:
///  the entries are left to that record and settle at the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = tryAcquire(index, seq); if (lock) await writeStart(witnessOf(index)); return lock
/// ```
pub(crate) fn index_lock_of(
    held: &Held,
    intent: &Intent,
) -> Option<crate::service_index_lock::IndexLock> {
    let lock: crate::service_index_lock::IndexLock =
        crate::service_index_lock::try_acquire(intent.index.as_path(), intent.seq)?;
    match crate::service_land_start::witness_of(intent.index.as_path()) {
        Ok(witness) => {
            if let Err(error) =
                crate::service_journal::write_start(held.journal.as_path(), intent.seq, &witness)
            {
                crate::service_warn!(
                    "batch {} {}: the index file the settlement saw could not be recorded ({error}); the entries are left to the owed record",
                    intent.seq,
                    crate::service_timing::in_the_repository_at(intent.repository.as_path())
                );
            }
        }
        Err(error) => crate::service_warn!(
            "batch {} {}: the index file cannot be looked at ({error}); the entries are left to the owed record",
            intent.seq,
            crate::service_timing::in_the_repository_at(intent.repository.as_path())
        ),
    }
    return Some(lock);
}

/// What:
///  Installs the batch's index entries under a lock the settlement holds,
///  and answers whether they still wait.
/// Why:
///  This is the landing's own step after the branch moved,
///  with the one difference a held commit forces:
///  the lock,
/// and the witness written under it,
/// are the settlement's,
///  so the entries are installed whatever another command did to the index while the lock stood.
///  Nothing is installed for a worktree that left the branch,
///  whose index describes another branch,
///  and the conclusion state is removed in every case
///  (`service_land_finish.rs`,
/// `finish_after_move`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (stillCheckedOut) await followUp(intent, lock) else { drop(lock); await cleanupDone(intent) }
/// ```
fn install_entries(
    git: &GitContext,
    held: &Held,
    intent: &Intent,
    lock: crate::service_index_lock::IndexLock,
) -> bool {
    let own_index: bool = intent.index == intent.git_dir.join("index");
    let ours: Result<bool, crate::service_protocol::FailureCode> =
        crate::service_head::still_checked_out(
            git,
            intent.repository.as_path(),
            intent.git_dir.as_path(),
            intent.common_dir.as_path(),
            &intent.reference,
        );
    let installed: bool = match ours {
        Ok(false) if own_index => {
            drop(lock);
            crate::service_land_follow::cleanup_done(
                git,
                intent.repository.as_path(),
                intent,
                held.journal.as_path(),
            )
        }
        Err(code) => {
            drop(lock);
            crate::service_error!(
                "batch {} {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is",
                intent.seq,
                crate::service_timing::in_the_repository_at(intent.repository.as_path())
            );
            false
        }
        Ok(_) => crate::service_land_finish_after_the_move::follow_up(
            git,
            intent.repository.as_path(),
            intent,
            lock,
            held.journal.as_path(),
        ),
    };
    if installed {
        crate::owed_index_update::settle(intent.git_dir.as_path(), intent.seq);
        crate::service_journal::remove_start(held.journal.as_path(), intent.seq);
    }
    return !installed;
}

/// What:
///  Finishes what the batch owes after its branch moved,
///  and answers whether its index entries still wait.
/// Why:
///   The index update,
/// the conclusion state and the held push are the landing's own steps after a move
///  (`service_land_finish.rs`),
///  and they are settled here the way a restart settles them when the settlement could not take the index lock,
///  so a kill between the move and the index update leaves nothing unsettled for good.
///  Whether the entries wait is the owed record's own answer:
///  it goes when they are installed and stays when they are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await holdThePushes(intent); if (lock) return installEntries(intent, lock); if (await settle(intent) === 'done') await markDone(seq)
/// ```
pub(crate) fn finish(
    git: &GitContext,
    held: &Held,
    intent: &Intent,
    lock: Option<crate::service_index_lock::IndexLock>,
) -> bool {
    crate::service_server_land_group::hold_the_pushes_of(intent);
    crate::service_index_lock::remove_stale(intent.index.as_path(), intent.seq);
    let waits: bool = match lock {
        Some(held_lock) => install_entries(git, held, intent, held_lock),
        None => {
            if crate::service_land_finish_replay_gate::settle(git, intent, held.journal.as_path())
                == crate::service_land_finish_replay_gate::Settled::Done
            {
                let _marked = crate::service_journal::mark_done(held.journal.as_path(), intent.seq);
            }
            crate::owed_index_update::path_of(held.git_dir.as_path(), held.seq).exists()
        }
    };
    if !waits {
        let _marked = crate::service_journal::mark_done(held.journal.as_path(), intent.seq);
    }
    return waits;
}

/// What:
///  Takes a batch whose branch the settlement found already moved,
///  and removes its record.
/// Why:
///   A kill between the branch move and the removal of the record leaves exactly this,
///  and the index update is what is still owed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finish(held, { ...intent, new: current }, lock); rm(record)
/// ```
pub(crate) fn finish_a_moved_branch(
    git: &GitContext,
    held: &Held,
    intent: &Intent,
    current: &str,
    lock: Option<crate::service_index_lock::IndexLock>,
) -> Settled {
    let mut moved: Intent = intent.clone();
    moved.new = current.to_string();
    let waits: bool = finish(git, held, &moved, lock);
    forget(held.git_dir.as_path(), held.seq);
    return if waits {
        Settled::LandedTheIndexWaits {
            commit: current.to_string(),
        }
    } else {
        Settled::Landed {
            commit: current.to_string(),
        }
    };
}
