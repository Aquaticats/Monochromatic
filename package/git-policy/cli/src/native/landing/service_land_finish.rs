//! What:
//!  What follows the branch move of a landing and of a restart:
//! the conclusion state and the index update,
//! and the decision
//!  whether a batch a killed or failed landing left open may be replayed.
//! Why:
//!  A replay runs after the lock the landing held was given up or lost,
//! so another program may have staged into the index since.
//!  A replay is made only over the very index file the landing saw under its lock (`service_land_start.rs`),
//! under the lock the
//!  restart takes,
//! and the head is checked again under that lock;
//! every other case leaves the index as its writers left it,
//!  says so in the service's log,
//! and finishes what does not touch the index.
//! The live landing and the restart share the one
//!  routine that finishes a batch,
//! so an alternate index and the conclusion state are settled the same way by both
//!  (`doc/planning/cli-git-main-commit-within-native-plus-five.md`,
//! "A replay only over the index the landing saw").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await finishAfterMove(git, intent, lock, journal, headCheck) // true: every obligation of the batch is settled
//! ```

use crate::service_land_git_steps_and_import_stream::keep_on_a_side_ref;
use crate::service_head::still_checked_out;
use crate::service_instance::instance_of;
use crate::service_journal::{INDEXED_EXTENSION, Indexed, Intent, OPEN_EXTENSION, Recovery, Started, indexed_state, list_intents, list_intents_read_only, mark_done, mark_indexed, read_start, recovery_for, remove_start};
use crate::service_land_follow::{IndexPaths, build_index};
use crate::service_land_start::witness_of;
use crate::service_index_lock::{IndexLock, Record, remove_stale, try_acquire};
use crate::service_scratch_ref_marker_of_a_batch::clear_recorded_scratch_refs;
use crate::service_scratch_ref_of_a_dropped_batch::{RefValue, clear_after_dropped_batch, kept_ref, value_of};
use crate::autostash_carry::carry_stages;
use crate::service_land_context::LandContext;
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use std::path::Path;

/// The index entries a batch's intent names.
pub(crate) fn intent_paths(intent: &Intent) -> IndexPaths<'_> {
    return if intent.derived { IndexPaths::Derived { old: intent.old.as_str() } } else { IndexPaths::Named(intent.paths.as_slice()) };
}

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
    return if finished { Settled::Done } else { Settled::Open };
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
pub fn follow_up(git: &GitContext, repository: &Path, intent: &Intent, lock: IndexLock, journal: &Path) -> bool {
    let cleaned: bool = crate::service_land_follow::cleanup_done(git, repository, intent, journal);
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
    let recorded: RecordIndexed<'_> = RecordIndexed { journal, intent };
    let installed: bool = lock.release_flushed_with_record(built.path(), built.is_flushed(), &recorded).is_ok();
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
pub fn finish_after_move(git: &GitContext, intent: &Intent, lock: IndexLock, journal: &Path, follows_move: Result<bool, FailureCode>) -> bool {
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
            crate::service_error!("batch {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is", intent.seq);
            false
        }
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
    /// Something is left,
    /// and the batch stays open.
    Open,
}

/// Settles one batch whose branch has moved,
/// found open.
pub(crate) fn settle(git: &GitContext, intent: &Intent, journal: &Path) -> Settled {
    let repository: &Path = intent.repository.as_path();
    let own_index: bool = intent.index == intent.git_dir.join("index");
    // A worktree that has since left the batch's branch has a default index that describes another branch: it is not updated.
    //  Skipping the index is safe whatever the head does next, so this look needs no lock. A head that cannot be read stops here.
    if own_index {
        match still_checked_out(git, repository, intent.git_dir.as_path(), intent.common_dir.as_path(), intent.reference.as_str()) {
            Ok(false) => return settled_of(crate::service_land_follow::cleanup_done(git, repository, intent, journal)),
            Ok(true) => {}
            Err(code) => {
                crate::service_error!("batch {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is", intent.seq);
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
            return settled_of(crate::service_land_follow::cleanup_done(git, repository, intent, journal));
        }
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
        match still_checked_out(git, repository, intent.git_dir.as_path(), intent.common_dir.as_path(), intent.reference.as_str()) {
            Ok(true) => {}
            Ok(false) => {
                drop(held);
                return settled_of(crate::service_land_follow::cleanup_done(git, repository, intent, journal));
            }
            Err(code) => {
                drop(held);
                crate::service_error!("batch {} stays open: the head of the worktree cannot be read under the lock ({code:?}), so the index is left as it is", intent.seq);
                return Settled::Open;
            }
        }
    }
    return match gate_of(journal, intent) {
        Gate::Replay => settled_of(follow_up(git, repository, intent, held, journal)),
        Gate::Wait => {
            drop(held);
            crate::service_error!("batch {} stays open: the record of the index file its landing saw cannot be read now", intent.seq);
            Settled::Open
        }
        Gate::Leave(reason) => {
            drop(held);
            crate::service_error!(
                "batch {}: the index entries of its paths were not updated and are left as they are, because {reason}; the record of what it \
owes is kept at {}, and the next commit in the worktree sets those entries to the commit before it makes anything of this index, \
so `git status` shows the commit's paths against the index until then",
                intent.seq,
                crate::owed_index_update::path_of(intent.git_dir.as_path(), intent.seq).display()
            );
            settled_of(crate::service_land_follow::cleanup_done(git, repository, intent, journal))
        }
    };
}

/// What a restart did with a batch it decided on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Handled {
    /// The batch landed and every obligation of it is settled;
    /// it is marked done.
    Finished,
    /// The batch landed and something is left (the index lock is held,
    /// a record cannot be read);
    /// it stays open.
    StillOpen,
    /// The batch never landed;
    /// its commit is written to `refs/cli-git/kept/<number>` when Git writes that ref,
    /// and its intent and records are removed.
    Dropped,
    /// The batch looks as if it never landed,
    /// and its journal is another service's:
    /// the intent and records stay on disk.
    Left,
}

/// One thing a restart did,
/// for the service's log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// The batch.
    pub seq: u64,
    /// What was decided.
    pub decision: Recovery,
    /// What was done about it.
    pub handled: Handled,
}

/// Whose journal a recovery works on,
/// which decides whether it may remove the intent of a batch that looks unlanded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JournalOwner {
    /// This service's own:
    /// no landing of another service is in progress in it,
    /// so an unlanded intent is dropped.
    ThisService,
    /// An earlier helper's,
    /// which may be alive:
    /// its landing writes the intent before it moves the branch,
    /// so "unlanded" may be
    ///  "about to land",
    /// and the intent stays for that helper's own start-up recovery.
    EarlierHelper,
}

/// What:
///  Keeps the commit of a batch that recovery drops on `refs/cli-git/kept/<number>`,
/// the ref the wrapper names for a commit it did not land,
/// before the batch's intent goes,
/// and never replaces a kept ref that already names another commit.
/// Why:
///   A service stopped inside the branch write never cuts that write:
/// it ends at its bound with the write still running,
/// so the landing cannot unwind and keep its commit,
/// and a service killed between its intent and its branch move leaves the same shape.
///   A commit only Git can build makes no scratch branch for that write to leave behind,
/// so this is the only place its commit reaches a ref;
/// where the batch did make one,
/// the drop keeps the commit here and then removes the scratch branch
/// (`service_scratch_ref_of_a_dropped_batch.rs`).
///   A kept ref that already names another commit is work an earlier batch was kept under,
/// so it is left alone and said;
/// only a number the clock and the journal deal twice can bring that about.
/// A write that cannot be made keeps nothing,
/// and it does not hold up the drop either:
/// the drop is what releases the worktree's index lock and forgets the batch,
/// so a commit whose ref cannot be written is still the commit the person runs again.
fn keep_what_the_branch_does_not_show(git: &GitContext, intent: &Intent) {
    let kept: String = kept_ref(intent.seq);
    match value_of(git, intent.repository.as_path(), kept.as_str()) {
        RefValue::Names(named) if named == intent.new => {
            crate::service_log!("batch {} is not on its branch, and its commit {} is already kept at {kept}", intent.seq, intent.new);
        }
        RefValue::Names(named) => crate::service_error!(
            "batch {} is not on its branch, and {kept} names {named}, so its commit {} is not kept there and is left where the landing put it",
            intent.seq,
            intent.new
        ),
        RefValue::Unasked => crate::service_error!(
            "batch {} is not on its branch, and {kept} cannot be read, so its commit {} is left where the landing put it",
            intent.seq,
            intent.new
        ),
        RefValue::Absent => match keep_on_a_side_ref(git, intent.repository.as_path(), intent.seq, intent.new.as_str()) {
            None => crate::service_log!("batch {} is not on its branch, so its commit {} is kept at {kept}", intent.seq, intent.new),
            Some(words) => crate::service_error!(
                "batch {} is not on its branch, and its commit {} could not be kept at {kept} ({words}), so the commit is left where the landing put it",
                intent.seq,
                intent.new
            ),
        },
    }
}

/// What:
///  Finishes or drops every batch the journal holds open.
/// Why:
///   Runs before each batch of a service that serves one worktree (`settle_open_batches`),
/// and before each batch of a development run that serves none,
/// so an index update
///  that gave up while another program held Git's index lock is finished by the next commit and not only by a restart.
/// An earlier release's shared journal is settled at a service's start instead
/// (`service_main.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recover(context)
/// ```
pub fn recover(context: &LandContext) -> Vec<RecoveryReport> {
    return recover_selected(context, None, JournalOwner::ThisService);
}

/// What:
///  Finishes or drops the open batches of the worktree one instance serves,
/// found in a journal that other worktrees' batches
///  may share.
/// Why:
///   The single service of the releases before one service per worktree kept one journal for every worktree,
/// in the directory
///  above each instance's own,
/// and an instance takes over the batches of its worktree there (found by the Git directory the
///  intent records,
/// which an index outside it,
/// as `GIT_INDEX_FILE` names one,
/// cannot change;
/// an intent that records none is
///  resolved from its repository).
/// A development run may also give several instances one journal.
/// Whatever belongs to
///  another worktree is left to its instance,
/// and a record that cannot be read is not any instance's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstance(context, instance)
/// ```
pub fn recover_instance(context: &LandContext, instance: &str) -> Vec<RecoveryReport> {
    return recover_selected(context, Some(instance), JournalOwner::ThisService);
}

/// What:
///  `recover_instance` over a journal an earlier helper may still be writing:
/// a batch that landed is finished as before,
/// and a
///  batch that looks as if it never landed is left on disk,
/// never dropped.
/// Why:
///   A live landing writes its intent,
/// then waits for the index lock,
/// then moves the branch.
/// A drop in that window removes the
///  record of a commit that lands a moment later,
/// and the person's index shows it reverted if the helper dies before it
///  installs the index.
/// A finish is safe against a live landing:
/// it never moves the branch,
/// its index replay needs the lock and
///  the records the landing writes,
/// and two finishes end in one `.done`.
/// The earlier helper drops what is still unlanded when it
///  starts again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInstanceWithoutDrops(context, instance)
/// ```
pub fn recover_instance_without_drops(context: &LandContext, instance: &str) -> Vec<RecoveryReport> {
    return recover_selected(context, Some(instance), JournalOwner::EarlierHelper);
}

/// The exit status `git rev-parse --verify --quiet` gives for a name that is no object:
/// the one answer that means "not there".
const NAME_IS_NO_OBJECT: i32 = 1;

/// The exit status `git merge-base --is-ancestor` gives when the first commit is not below the second.
const NOT_AN_ANCESTOR: i32 = 1;

/// What:
///  The value of `reference` in the repository:
/// its commit name,
/// an empty name when Git says for certain that the reference is
///  not there,
/// and nothing when Git gave no answer.
///  `Option<String>` is "an answer or none".
/// Why:
///   A restart drops the intent of a batch whose branch never moved.
/// A Git that could not be started,
/// or that failed in any way
///  other than "no such reference" (a lock,
/// a stuck mount,
/// a repository that cannot be read),
/// has not said the branch is
///  unmoved,
/// so the caller keeps the batch open.
/// `--quiet` makes a missing reference exit 1 with nothing on the standard error;
///  every other failure exits 128,
/// and a reference file Git cannot read exits 1 with a warning,
/// which is not "missing" either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function currentValue(git, intent): Promise<string | undefined> // '' for a missing reference
/// ```
fn current_value(git: &GitContext, intent: &Intent) -> Option<String> {
    let request: GitRequest = GitRequest::new(intent.repository.as_path(), &["rev-parse", "--verify", "--quiet", intent.reference.as_str()]);
    let Ok(output) = run_git(git, &request) else {
        return None;
    };
    if output.succeeded() {
        return Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string());
    }
    if output.code == Some(NAME_IS_NO_OBJECT) && output.stderr.is_empty() {
        return Some(String::new());
    }
    return None;
}

/// What:
///  Whether the commit the batch landed is `current` or below it;
/// nothing when Git gave no answer.
/// Why:
///   Exit status 1 is Git's "not below";
/// any other failure (an object Git cannot find,
/// a lock,
/// a stuck mount) is no answer,
/// and
///  a batch is not dropped on no answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function landedBelow(git, intent, current): Promise<boolean | undefined>
/// ```
fn landed_below(git: &GitContext, intent: &Intent, current: &str) -> Option<bool> {
    let request: GitRequest = GitRequest::new(intent.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current]);
    let Ok(output) = run_git(git, &request) else {
        return None;
    };
    if output.succeeded() {
        return Some(true);
    }
    return if output.code == Some(NOT_AN_ANCESTOR) { Some(false) } else { None };
}

/// What:
///  The decision for an open batch from the branch as Git reports it now,
/// or nothing when Git gave no answer.
/// Why:
///   The question about the ancestry is asked only when it decides:
/// a branch at the old or the new value,
/// or one that is gone,
///  settles the batch without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function decisionFor(git, intent): Promise<Recovery | undefined>
/// ```
fn decision_for(git: &GitContext, intent: &Intent) -> Option<Recovery> {
    let current: String = current_value(git, intent)?;
    let decided_without_ancestry: bool = current.is_empty() || current == intent.old || current == intent.new;
    let ancestor: bool = if decided_without_ancestry { false } else { landed_below(git, intent, current.as_str())? };
    return Some(recovery_for(intent, current.as_str(), ancestor));
}

fn recover_selected(context: &LandContext, instance: Option<&str>, owner: JournalOwner) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let listed: Vec<(std::path::PathBuf, Option<Intent>)> = match owner {
        JournalOwner::ThisService => list_intents(context.journal.as_path(), OPEN_EXTENSION),
        JournalOwner::EarlierHelper => list_intents_read_only(context.journal.as_path(), OPEN_EXTENSION),
    };
    for (path, parsed) in listed {
        let Some(intent) = parsed else {
            // A record this build cannot read names a batch's branch values, paths, index and cleanup, and is the only thing that
            //  does: removing it would forget a landed commit's owed index update and leave the person's next commit recording
            //  the commit's own paths against the index (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
            //  finding 1). It is kept and named instead, and what it owed is settled from its own record in the worktree
            //  (`owed_index_update.rs`) or by the person.
            let named: String = crate::owed_index_update::batch_number_of_path(path.as_path());
            crate::service_error!(
                "batch {named}: its open record {} cannot be read, so it is kept and nothing of it is settled; its owed index update is \
settled from the worktree's own record of it, and the file may be removed once that record is gone",
                path.display()
            );
            continue;
        };
        // With an instance named, only the batches of that instance's worktree are recovered.
        let selected: bool = match instance {
            Some(named) => instance_of(intent.git_dir.as_path()) == named,
            None => true,
        };
        if !selected {
            continue;
        }
        // The selection follows whatever this service already ran its Git with, so an absolute path it names wins over a
        //  relative one an earlier option wrote.
        let mut prefix: Vec<std::ffi::OsString> = context.git.global_prefix.clone();
        prefix.extend(crate::repository_selection::of_paths(intent.git_dir.as_path(), intent.repository.as_path()));
        let git: GitContext = GitContext { common_dir: Some(intent.common_dir.clone()), global_prefix: prefix, ..context.git.clone() };
        let Some(decision) = decision_for(&git, &intent) else {
            crate::service_error!("batch {} stays open: Git gave no answer about {} in {}, so its intent is kept", intent.seq, intent.reference, intent.repository.display());
            continue;
        };
        let handled: Handled = match (decision, owner) {
            (Recovery::Drop, JournalOwner::EarlierHelper) => Handled::Left,
            (Recovery::Drop, JournalOwner::ThisService) => {
                keep_what_the_branch_does_not_show(&git, &intent);
                remove_stale(intent.index.as_path(), intent.seq);
                // The batch will never land, so the scratch reference its killed landing left is removed here; the commit it
                //  holds is kept first, on the batch's side reference (`service_scratch_ref_of_a_dropped_batch.rs`).
                clear_after_dropped_batch(&git, intent.repository.as_path(), intent.seq);
                crate::service_server_land_group::forget_the_pushes_of(&intent);
                // A batch that will never land owes no index update, so any record of one is forgotten with it, and so is the copy
                //  of the conclusion state it would have removed.
                crate::owed_index_update::remove(intent.git_dir.as_path(), intent.seq);
                crate::conclusion_state_copy::remove(context.journal.as_path(), intent.seq);
                let _removed = std::fs::remove_file(&path);
                let _record_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), intent.seq, INDEXED_EXTENSION));
                remove_start(context.journal.as_path(), intent.seq);
                Handled::Dropped
            }
            (Recovery::Finish, _) => {
                // The batch landed, so its pushes are owed: a service killed before it held them leaves recovery to do it.
                crate::service_server_land_group::hold_the_pushes_of(&intent);
                remove_stale(intent.index.as_path(), intent.seq);
                if matches!(settle(&git, &intent, context.journal.as_path()), Settled::Done) {
                    let _marked = mark_done(context.journal.as_path(), intent.seq);
                    Handled::Finished
                } else {
                    Handled::StillOpen
                }
            }
        };
        reports.push(RecoveryReport { seq: intent.seq, decision, handled });
    }
    // A landing killed between its import and its intent leaves its batch's scratch reference with no intent to settle it, and
    //  the record it wrote is the only thing that names that reference's worktree: the sweep keeps the commit it holds and then
    //  removes it (`service_scratch_ref_marker_of_a_batch.rs`). An earlier helper's journal is its own to settle.
    if owner == JournalOwner::ThisService {
        clear_recorded_scratch_refs(context, instance);
    }
    return reports;
}
