//! What:
//!  What follows the branch move of a landing and of a restart: the conclusion state and the index update, and the decision
//!  whether a batch a killed or failed landing left open may be replayed.
//! Why:
//!  A replay runs after the lock the landing held was given up or lost, so another program may have staged into the index since.
//!  A replay is made only over the very index file the landing saw under its lock (`service_land_start.rs`), under the lock the
//!  restart takes, and the head is checked again under that lock; every other case leaves the index as its writers left it,
//!  says so in the service's log, and finishes what does not touch the index. The live landing and the restart share the one
//!  routine that finishes a batch, so an alternate index and the conclusion state are settled the same way by both
//!  (`doc/planning/cli-git-main-commit-within-native-plus-five.md`, "A replay only over the index the landing saw").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await finishAfterMove(git, intent, lock, journal, headCheck) // true: every obligation of the batch is settled
//! ```

use crate::service_head::still_checked_out;
use crate::service_instance::instance_of;
use crate::service_journal::{INDEXED_EXTENSION, Indexed, Intent, OPEN_EXTENSION, Recovery, Started, indexed_state, list_intents, mark_done, mark_indexed, read_start, recovery_for, remove_start};
use crate::service_land_follow::{IndexPaths, build_index, remove_conclusion_state};
use crate::service_land_start::witness_of;
use crate::service_index_lock::{IndexLock, remove_stale, try_acquire};
use crate::autostash_carry::carry_stages;
use crate::service_land::LandContext;
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// The index entries a batch's intent names.
pub(crate) fn intent_paths(intent: &Intent) -> IndexPaths<'_> {
    return if intent.derived { IndexPaths::Derived { old: intent.old.as_str() } } else { IndexPaths::Listed(intent.paths.as_slice()) };
}

/// Removes the conclusion state a landed batch leaves, when it has one. Whether it is gone.
pub(crate) fn cleanup_done(git: &GitContext, repository: &Path, intent: &Intent) -> bool {
    return intent.cleanup.as_ref().is_none_or(|cleanup| { return remove_conclusion_state(git, repository, cleanup); });
}

/// Finishes what follows the branch move: the conclusion state is removed, then the index entries are updated. Both are
///  idempotent, and they are recorded apart: the installed index is recorded in the journal while the lock is still held
///  (`<batch>.indexed`), so a restart never replays the index update over what another program staged after the lock was given
///  up, whether or not the conclusion state could be removed. Whether both are done.
pub fn follow_up(git: &GitContext, repository: &Path, intent: &Intent, lock: IndexLock, journal: &Path) -> bool {
    let cleaned: bool = cleanup_done(git, repository, intent);
    if crate::service_fault::fails("build-index") {
        return false;
    }
    // The new index is built beside the real one and installed by the release of the lock the landing holds.
    let Ok(built) = build_index(git, repository, intent.new.as_str(), intent_paths(intent), &lock, intent.seq) else {
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
    let recorded = || -> std::io::Result<()> { return mark_indexed(journal, intent.seq, intent.new.as_str()); };
    return lock.release_flushed_with_record(built.path(), built.is_flushed(), &recorded).is_ok() && cleaned;
}

/// What:
///  Settles a batch whose branch has moved, while its caller holds the index lock: whether the worktree's head still follows the
///  move decides if the index is updated, and the conclusion state is removed in every case. Whether the batch is done.
/// Why:
///   A worktree that left the batch's branch has a default index that describes another branch, so that index is not updated;
///  an index kept in another file (`GIT_INDEX_FILE`) belongs to no branch and is still finished. A head that cannot be read
///  decides nothing: the lock is given up, the index is not touched, and the batch stays open for a restart to settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await finishAfterMove(git, intent, lock, journal, followsMove)
/// ```
pub fn finish_after_move(git: &GitContext, intent: &Intent, lock: IndexLock, journal: &Path, follows_move: Result<bool, FailureCode>) -> bool {
    let repository: &Path = intent.repository.as_path();
    let own_index: bool = intent.index == intent.git_dir.join("index");
    return match follows_move {
        Ok(false) if own_index => {
            drop(lock);
            cleanup_done(git, repository, intent)
        }
        Ok(_) => follow_up(git, repository, intent, lock, journal),
        Err(code) => {
            drop(lock);
            crate::service_error!("batch {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is", intent.seq);
            false
        }
    };
}

/// What a restart makes of the evidence for replaying an open batch's index update.
#[derive(Debug, Eq, PartialEq)]
enum Gate {
    /// The index file is the one the landing saw: nothing was staged since.
    Replay,
    /// The evidence cannot be read now: the batch stays open.
    Wait,
    /// The index may hold staging: it is left as it is, for this reason.
    Leave(&'static str),
}

/// What:
///  Whether the index update of the open batch may be replayed over the index as it is now, judged under the lock the caller
///  holds.
/// Why:
///   The landing recorded the witness of the index file it saw under its lock. A file that is still that file has been written
///  by nobody since, and a replay over it discards nothing. Any other index, and a batch with no record, may hold what another
///  session staged, so no path of it is reset: whether a path holds the entry the landing started from does not say whether it
///  was staged back to that entry, and a reset also rewrites the entries under and above its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gate = gateOf(journal, intent) // 'replay' | 'wait' | { leave: reason }
/// ```
fn gate_of(journal: &Path, intent: &Intent) -> Gate {
    let recorded = match read_start(journal, intent.seq) {
        Started::Absent => return Gate::Leave("no record says which index file the landing saw"),
        Started::Unreadable => return Gate::Wait,
        Started::Recorded(witness) => witness,
    };
    return match witness_of(intent.index.as_path()) {
        Ok(now) if now == recorded => Gate::Replay,
        Ok(_) => Gate::Leave("the index file is not the one the landing saw, so another program may have staged into it"),
        Err(_) => Gate::Wait,
    };
}

/// What a restart made of one open batch.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Settled {
    /// Every obligation of the batch is settled.
    Done,
    /// Something is left, and the batch stays open.
    Open,
}

/// Settles one batch whose branch has moved, found open.
pub(crate) fn settle(git: &GitContext, intent: &Intent, journal: &Path) -> Settled {
    let repository: &Path = intent.repository.as_path();
    let own_index: bool = intent.index == intent.git_dir.join("index");
    let head = || { return still_checked_out(git, repository, intent.git_dir.as_path(), intent.common_dir.as_path(), intent.reference.as_str()); };
    let done = |finished: bool| -> Settled { return if finished { Settled::Done } else { Settled::Open }; };
    // A worktree that has since left the batch's branch has a default index that describes another branch: it is not updated.
    //  Skipping the index is safe whatever the head does next, so this look needs no lock. A head that cannot be read stops here.
    if own_index {
        match head() {
            Ok(false) => return done(cleanup_done(git, repository, intent)),
            Ok(true) => {}
            Err(code) => {
                crate::service_error!("batch {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is", intent.seq);
                return Settled::Open;
            }
        }
    }
    match indexed_state(journal, intent.seq, intent.new.as_str()) {
        // The installed index is recorded: it is not replayed, and only the conclusion state may be left.
        Indexed::Installed => return done(cleanup_done(git, repository, intent)),
        // A record that cannot be read may say the index was installed: the batch stays open and the index alone.
        Indexed::Unreadable => {
            crate::service_error!("batch {} stays open: its completion record cannot be read, so the index is left as it is", intent.seq);
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
        match head() {
            Ok(true) => {}
            Ok(false) => {
                drop(held);
                return done(cleanup_done(git, repository, intent));
            }
            Err(code) => {
                drop(held);
                crate::service_error!("batch {} stays open: the head of the worktree cannot be read under the lock ({code:?}), so the index is left as it is", intent.seq);
                return Settled::Open;
            }
        }
    }
    return match gate_of(journal, intent) {
        Gate::Replay => done(follow_up(git, repository, intent, held, journal)),
        Gate::Wait => {
            drop(held);
            crate::service_error!("batch {} stays open: the record of the index file its landing saw cannot be read now", intent.seq);
            Settled::Open
        }
        Gate::Leave(reason) => {
            drop(held);
            crate::service_error!(
                "batch {}: the index entries of its paths were not updated and are left as they are, because {reason}; `git status` shows the commit's paths against the index until they are staged again",
                intent.seq
            );
            done(cleanup_done(git, repository, intent))
        }
    };
}

/// One thing a restart did, for the service's log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// The batch.
    pub seq: u64,
    /// What was decided.
    pub decision: Recovery,
}

/// What:
///  Finishes or drops every batch the journal holds open.
/// Why:
///   Runs before the service accepts a connection, and again before each batch (`service_server.rs`), so an index update
///  that gave up while another program held Git's index lock is finished by the next commit and not only by a restart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recover(context)
/// ```
pub fn recover(context: &LandContext) -> Vec<RecoveryReport> {
    return recover_selected(context, &|_| { return true; }, true);
}

/// What:
///  Finishes or drops the open batches of the worktree one instance serves, found in a journal that other worktrees' batches
///  may share.
/// Why:
///   The single service of the releases before one service per worktree kept one journal for every worktree, in the directory
///  above each instance's own, and an instance takes over the batches of its worktree there (found by the Git directory the
///  intent records, which an index outside it, as `GIT_INDEX_FILE` names one, cannot change; an intent that records none is
///  resolved from its repository). A development run may also give several instances one journal. Whatever belongs to
///  another worktree is left to its instance, and a record that cannot be read is not any instance's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstance(context, instance)
/// ```
pub fn recover_instance(context: &LandContext, instance: &str) -> Vec<RecoveryReport> {
    let ours = |intent: &Intent| -> bool { return instance_of(intent.git_dir.as_path()) == instance; };
    return recover_selected(context, &ours, false);
}

/// The value of `reference` in the repository, empty when it cannot be read.
fn current_value(git: &GitContext, intent: &Intent) -> String {
    let request: GitRequest = GitRequest::new(intent.repository.as_path(), &["rev-parse", "--verify", intent.reference.as_str()]);
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string(),
        _ => String::new(),
    };
}

/// Whether the commit the batch landed is `current` or below it.
fn landed_below(git: &GitContext, intent: &Intent, current: &str) -> bool {
    let request: GitRequest = GitRequest::new(intent.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current]);
    return run_git(git, &request).is_ok_and(|output| { return output.succeeded(); });
}

fn recover_selected(context: &LandContext, selected: &dyn Fn(&Intent) -> bool, remove_unreadable: bool) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    for (path, parsed) in list_intents(context.journal.as_path(), OPEN_EXTENSION) {
        let Some(intent) = parsed else {
            if remove_unreadable {
                let _removed = std::fs::remove_file(&path);
            }
            continue;
        };
        if !selected(&intent) {
            continue;
        }
        let git: GitContext = GitContext { common_dir: Some(intent.common_dir.clone()), ..context.git.clone() };
        let current: String = current_value(&git, &intent);
        let ancestor: bool = landed_below(&git, &intent, current.as_str());
        let decision: Recovery = recovery_for(&intent, current.as_str(), ancestor);
        match decision {
            Recovery::Drop => {
                remove_stale(intent.index.as_path(), intent.seq);
                let _removed = std::fs::remove_file(&path);
                let _record_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), intent.seq, INDEXED_EXTENSION));
                remove_start(context.journal.as_path(), intent.seq);
            }
            Recovery::Finish => {
                remove_stale(intent.index.as_path(), intent.seq);
                if matches!(settle(&git, &intent, context.journal.as_path()), Settled::Done) {
                    let _marked = mark_done(context.journal.as_path(), intent.seq);
                }
            }
        }
        reports.push(RecoveryReport { seq: intent.seq, decision });
    }
    return reports;
}
