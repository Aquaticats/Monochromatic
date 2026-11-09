//! What:
//!  The service's serialized step:
//! land a batch of prepared commits on a branch,
//! and finish the batches a killed
//!  service left open.
//! Why:
//!  Everything that must happen one commit at a time happens here,
//! in a fixed order,
//! with one durable write:
//! the objects
//!  of the whole batch are written by one `fast-import`,
//! the intent is journaled,
//! the branch moves by one compare-and-swap,
//!  the index entries of the named paths are updated,
//! and the intent is marked done
//!  (`doc/planning/cli-git-resident-service.md`,
//! "The serialized step,
//! group commit,
//! FIFO and backpressure").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await landBatch(context, seq, requests);
//! ```

#[cfg(test)]
use std::path::PathBuf;
pub use crate::service_land_context::LandContext;
use crate::service_land_take_index_lock::take_index_lock;
use crate::service_land_git_steps_and_import_stream::{checked, ids_of, requests_that_land, run};
use crate::service_land_objects_of_batch::objects_of_batch;
use crate::service_fault::reach;
use crate::service_built_checks::{BuiltCommit, history_is_plain, read_commit};
use crate::service_head::{follows_move, still_checked_out};
use crate::service_pruned::batch_content_missing;
use crate::service_journal::{Intent, OPEN_EXTENSION, remove_start, write_intent, write_start};
pub use crate::service_land_finish::{Handled, RecoveryReport, recover, recover_instance, recover_instance_without_drops};
#[cfg(test)]
use crate::service_land_finish::follow_up;
#[cfg(test)]
use crate::service_journal::{Recovery, list_intents};
#[cfg(test)]
use crate::service_index_lock::try_acquire;
use crate::service_land_finish::finish_after_move;
use crate::service_land_start::witness_of;
use crate::service_index_lock::{Announce, Announcer, IndexLock};
use crate::service_timing::LandTiming;
use crate::config_snapshot::{ConfigSnapshot, read_config_snapshot};
use crate::service_scratch_ref_marker_of_a_batch::end_scratch_ref_record;
use crate::service_scratch_ref_of_a_dropped_batch::scratch_ref;
use crate::service_land_follow::storage_failure;
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, NO_REFLOG_SETTING};
use std::path::Path;
use std::time::Instant;

/// How many times a batch is built again when the branch moved under it before the failure is reported.
pub const LANDING_ATTEMPTS: usize = 3;

/// What the landing reads once before it starts and hands to its body.
struct Settings<'a> {
    /// The `core.fsync` value for the one import,
    /// when the repository's own would flush a pack that is thrown away or leave a kept one unflushed.
    import_fsync: Option<&'a str>,
    /// Whether Git shows the history the commit objects say,
    /// so the object store may answer the checks of a commit Git built.
    plain_history: bool,
}

/// What became of one request of a batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The commit is on the branch.
    Landed { commit: String },
    /// A named path changed on the branch since the request's base.
    RetryPrepare { base: String },
    /// An amend,
    /// whose parents are the base's parents,
    /// found the branch somewhere else than at its base.
    BranchMoved,
}

/// What:
///  Lands a batch of requests that share a repository,
/// a common git directory and a branch.
/// Why:
///   Requests whose paths changed on the branch since their base are sent back to be prepared again;
/// the others become
///  one commit each,
/// in order,
/// on the branch,
/// with one journal write and one branch update for all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatch(context, seq, requests) // one outcome per request, in order
/// ```
pub fn land_batch(context: &LandContext, seq: u64, requests: &[CommitRequest]) -> Result<Vec<Outcome>, FailureCode> {
    return land_batch_announcing(context, seq, requests, &IgnoreAnnouncements);
}

/// `land_batch`,
/// telling the clients of the batch what it waits for when another program holds the index.
pub fn land_batch_announcing(context: &LandContext, seq: u64, requests: &[CommitRequest], announce: &dyn Announcer) -> Result<Vec<Outcome>, FailureCode> {
    return land_batch_timed(context, seq, requests, announce, &mut LandTiming::default());
}

/// What:
///  `land_batch_announcing`,
/// leaving in `timing` the time each phase of the landing took.
/// Why:
///   The service's log line for each request states the wait for another program's hold on the index,
/// the objects,
/// the journal
///  write,
/// the branch move and the index update;
/// `timing` is filled as the phases end,
/// so a landing that fails part way still
///  reports the phases it reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await landBatchTimed(context, seq, requests, announce, timing)
/// ```
///
/// # Errors
///
/// The failure code of the landing,
/// as `land_batch_announcing`.
pub fn land_batch_timed(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Announcer,
    timing: &mut LandTiming,
) -> Result<Vec<Outcome>, FailureCode> {
    // The landing queues its own syncs and settles them when it ends: nothing is flushed while the person waits for a commit that
    //  is published, and what a landing that publishes nothing wrote is flushed before it returns (`service_durable.rs`). The
    //  index file follows `core.fsync` as native Git's index write does; a configuration that cannot be read is taken to name it.
    let config_began: Instant = Instant::now();
    let snapshot: Option<ConfigSnapshot> = match requests.first() {
        Some(first) => read_config_snapshot(&context.git, first.repository.as_path()),
        None => None,
    };
    let hardens_index: bool = snapshot.as_ref().and_then(ConfigSnapshot::hardens_index).unwrap_or(true);
    // Native Git writes a commit's objects as loose objects and flushes none by default; the import of a small batch writes a
    //  temporary pack, flushes it and deletes it, so the flush is dropped unless the repository asks for flushed objects.
    let import_fsync: Option<String> = snapshot.as_ref().and_then(ConfigSnapshot::import_fsync_value);
    let deferral = crate::service_durable::begin(hardens_index);
    // The object store may answer the checks of a commit Git built only where Git shows the history the files say: the settings that
    //  change it (a shallow or graft file elsewhere, another replacement namespace, replacements switched off) are read here from
    //  the environment and configuration the landing's Git children run under, and a configuration that cannot be read counts as one.
    let plain_history: bool = match requests.first() {
        Some(first) => history_is_plain(first.common_dir.as_path(), &context.git, snapshot.as_ref()),
        None => false,
    };
    timing.reads_split.config = config_began.elapsed();
    let settings: Settings<'_> = Settings { import_fsync: import_fsync.as_deref(), plain_history };
    let result: Result<Vec<Outcome>, FailureCode> = land_batch_body(context, seq, requests, announce, timing, &settings);
    // A step ended at its ceiling anywhere else in the landing: the clients are told which, and the commit the batch built, when it
    //  built one, is kept on a side ref. No open intent finishes a batch whose branch has not moved, so nothing lands later.
    if result == Err(FailureCode::StepTooSlow) {
        crate::service_land_git_steps_and_import_stream::keep_what_an_ended_step_built(context, announce, requests, seq);
    }
    // A landing that ends before its intent is in the journal leaves its batch's objects on the scratch branch, and no restart
    //  can land them: only an intent lets a restart decide the batch from the branch, so a failed landing keeps the commit and
    //  removes the branch itself (`service_scratch_ref_of_a_dropped_batch.rs`). One that wrote its intent leaves it to the restart.
    // The record that names this batch's worktree (`service_scratch_ref_marker_of_a_batch.rs`) goes when the reference it names
    //  does, and stays beside a reference a failed landing had to leave where it is.
    end_scratch_ref_record(context, requests, seq, result.as_ref().err());
    let published: bool = match result.as_ref() {
        Ok(outcomes) => any_landed(outcomes.as_slice()),
        Err(_failure) => false,
    };
    deferral.settle(published);
    return result;
}

/// What:
///  An announcement that nobody is told.
/// Why:
///  `land_batch` is `land_batch_announcing` for a caller that has no clients to tell what the landing waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreAnnouncements = { announce: (_note: Announce) => {} };
/// ```
struct IgnoreAnnouncements;

impl Announcer for IgnoreAnnouncements {
    fn announce(&self, _note: Announce) {}
}

/// What:
///  Whether any outcome of a batch is a landed commit.
/// Why:
///  A batch that landed something hands its syncs to the background;
/// one that landed nothing syncs them before it returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcomes.some((outcome) => outcome.kind === 'landed')
/// ```
fn any_landed(outcomes: &[Outcome]) -> bool {
    for outcome in outcomes {
        if matches!(outcome, Outcome::Landed { .. }) {
            return true;
        }
    }
    return false;
}

/// The landing itself;
/// `land_batch_timed` adds what is told when a step is ended at its ceiling.
///
/// # Errors
///
/// As `land_batch_timed`.
fn land_batch_body(
    context: &LandContext,
    seq: u64,
    requests: &[CommitRequest],
    announce: &dyn Announcer,
    timing: &mut LandTiming,
    settings: &Settings<'_>,
) -> Result<Vec<Outcome>, FailureCode> {
    let first: &CommitRequest = requests.first().ok_or(FailureCode::Rejected)?;
    for request in requests {
        if request.repository != first.repository || request.common_dir != first.common_dir || request.reference != first.reference {
            return Err(FailureCode::Rejected);
        }
    }
    // An amend, and a commit built elsewhere, are landed alone: each is only valid while the branch is still at its base,
    //  which a neighbour in the same batch would change.
    let alone: bool = requests.iter().any(CommitRequest::lands_alone);
    if alone && requests.len() != 1 {
        return Err(FailureCode::Rejected);
    }
    let repository: &Path = first.repository.as_path();
    let git: GitContext = GitContext { common_dir: Some(first.common_dir.clone()), ..context.git.clone() };
    let scratch: String = scratch_ref(seq);
    let mut outcomes: Vec<Outcome>;
    let mut attempts_left: usize = LANDING_ATTEMPTS;
    // A branch that does not exist yet is prepared against the all-zero name and lands only while it still does not.
    let unborn: bool = is_zero_object(first.base.as_str());
    loop {
        timing.attempts += 1;
        // The commit the branch names, asked in the one spelling the in-process readers answer (`<ref>^{commit}`), so the question
        //  starts no Git; for a branch, which only ever names a commit, the answer is the same as for the bare name.
        let branch_began: Instant = Instant::now();
        let branch_commit: String = format!("{}^{{commit}}", first.reference);
        let old: String = match checked(&git, repository, &["rev-parse", "--verify", "--quiet", branch_commit.as_str()], None) {
            Ok(found) => found,
            Err(_) if unborn => first.base.clone(),
            Err(code) => return Err(code),
        };
        if alone && first.base != old {
            return Ok(vec![Outcome::BranchMoved]);
        }
        // Native Git updates `HEAD`, so a worktree that switched branches while the commit waited fails the commit with nothing
        //  changed. The named branch would move all the same, and the index built for it would replace the other branch's. A
        //  symbolic `HEAD` is not refused by Git's own compare when it is told not to follow it (measured: it replaces the
        //  symbolic reference), so a head that became a branch is found here, before the move. The same question is asked
        //  again once the index lock is held, because the worktree can switch while the lock is awaited.
        let still_ours: bool = still_checked_out(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str())?;
        timing.reads_split.branch += branch_began.elapsed();
        if !still_ours {
            return Ok(vec![Outcome::BranchMoved; requests.len()]);
        }
        // Only an amend uses the parents of the commit it replaces; they are read from the object store, and listed by Git when the
        //  store cannot say (the same rule as for the commit Git built).
        let parents_began: Instant = Instant::now();
        let amended_parents: Vec<String> = if alone && !unborn && first.amend {
            let replaced_commit: Option<BuiltCommit> = if settings.plain_history { read_commit(first.common_dir.as_path(), first.base.as_str()) } else { None };
            match replaced_commit {
                Some(replaced) => replaced.parents,
                None => {
                    let listing: String = checked(&git, repository, &["rev-list", "--parents", "--max-count=1", first.base.as_str()], None)?;
                    listing.split_whitespace().skip(1).map(str::to_string).collect()
                }
            }
        } else {
            Vec::new()
        };
        timing.reads_split.parents += parents_began.elapsed();
        // One tree comparison per distinct base: the paths that changed on the branch since then.
        outcomes = vec![Outcome::RetryPrepare { base: old.clone() }; requests.len()];
        let changed_began: Instant = Instant::now();
        let landing: Vec<(usize, &CommitRequest)> = requests_that_land(&git, repository, requests, old.as_str())?;
        timing.reads_split.changed += changed_began.elapsed();
        if landing.is_empty() {
            return Ok(outcomes);
        }
        let mut batch: Vec<&CommitRequest> = Vec::new();
        for (_index, request) in &landing {
            batch.push(*request);
        }
        // Content that `git gc --prune=now` removed after the preparation is found before anything is imported or moved: the
        //  client prepares the commit again, and says so plainly when that is not enough.
        let content_began: Instant = Instant::now();
        let missing: bool = batch_content_missing(first.common_dir.as_path(), batch.as_slice());
        timing.reads_split.content += content_began.elapsed();
        if missing {
            return Err(FailureCode::Unavailable);
        }
        let objects_began: Instant = Instant::now();
        let (commits, new): (Vec<String>, String) = objects_of_batch(
            &git,
            context.journal.as_path(),
            seq,
            first,
            scratch.as_str(),
            old.as_str(),
            &batch,
            amended_parents.as_slice(),
            settings.plain_history,
            settings.import_fsync,
        )?;
        timing.objects += objects_began.elapsed();
        reach("after-objects");
        if let Some(seam) = &context.on_objects_written {
            seam();
        }
        // A commit Git built carries no list of changes: its index entries are the paths that differ from the old branch.
        let derived: bool = first.commit.is_some();
        let paths: Vec<Vec<u8>> = if derived {
            Vec::new()
        } else {
            let mut named: Vec<Vec<u8>> = Vec::new();
            for request in &batch {
                for change in &request.changes {
                    named.push(crate::path_bytes::path_bytes_of(change.path.as_path()).to_vec());
                }
            }
            named
        };
        let intent: Intent = Intent {
            seq,
            repository: first.repository.clone(),
            common_dir: first.common_dir.clone(),
            reference: first.reference.clone(),
            old: old.clone(),
            new: new.clone(),
            ids: ids_of(&batch),
            commits: commits.clone(),
            paths,
            derived,
            cleanup: first.cleanup.clone(),
            index: first.index.clone(),
            git_dir: first.git_dir.clone(),
        };
        // The index update is owed from this instant, and its own record is written before the branch moves: a kill between the
        //  move and the index install leaves it where the person's next command reads it (`owed_index_update.rs`).
        // Everything the batch owes is taken here, before the branch moves, so a kill after the move leaves the record of the index
        //  update and the held push of every commit it would land (`work_item_holding.rs`).
        let owed_began: Instant = Instant::now();
        crate::service_server_land_group::what_a_batch_owes(&intent);
        timing.reads_split.owed += owed_began.elapsed();
        // What the conclusion's cleanup compares is the client's to lose: a copy is kept in the journal before the branch moves, so
        //  a cleanup that fails once and is finished by a restart still removes the state (`conclusion_state_copy.rs`).
        let conclusion_began: Instant = Instant::now();
        crate::conclusion_state_copy::keep_of_intent(intent.cleanup.as_ref(), context.journal.as_path(), seq);
        timing.reads_split.conclusion += conclusion_began.elapsed();
        let journal_began: Instant = Instant::now();
        let written = write_intent(context.journal.as_path(), &intent);
        timing.journal += journal_began.elapsed();
        if let Err(error) = written {
            return Err(storage_failure(&error));
        }
        reach("after-intent");
        // Git's own rule: the index lock is held before the branch is moved and given up when the new index is in place, so
        //  a foreign `git` that holds it finishes first and this landing builds on what it left. Everything that can be done
        //  without the index is done before the lock is asked for (the reads, the objects, the intent), so the lock is held
        //  for the branch move and the index build only, and a foreign holder is waited for while the work above is done,
        //  not before it. A lock that is not given up here (a refusal, a failure) is removed when it goes out of scope, and
        //  the index stays as it was.
        let held: IndexLock = take_index_lock(&git, context.journal.as_path(), context.lock_ceiling, first, seq, announce, timing, new.as_str(), scratch.as_str())?;
        let hold_began: Instant = Instant::now();
        // The worktree may have switched while the lock was awaited: the checked-out branch is read again with the lock held and
        //  before the ref moves, and a landing that no longer belongs to this worktree's head stops with nothing changed.
        let still: Result<bool, FailureCode> = still_checked_out(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str());
        if !matches!(still, Ok(true)) {
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            crate::service_server_land_group::forget_what_a_batch_owes(&intent);
            let _deleted = run(&git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()], None);
            return match still {
                Ok(_) => Ok(vec![Outcome::BranchMoved; requests.len()]),
                Err(code) => Err(code),
            };
        }
        // The index file now, under the lock, is what a replay of this batch must still find before it resets a path: a replay is
        //  made only over the file the landing saw. It is recorded for a commit Git built as for any other, since the witness
        //  needs no list of paths. A record that cannot be written leaves a batch no replay is made for, and this landing
        //  still updates the index itself while it holds the lock.
        let witness_began: Instant = Instant::now();
        match witness_of(first.index.as_path()) {
            Ok(witness) => {
                if let Err(error) = write_start(context.journal.as_path(), seq, &witness) {
                    crate::service_error!("batch {seq}: the index file the landing saw could not be recorded ({error}); a restart will not replay the index update");
                }
            }
            Err(error) => crate::service_error!("batch {seq}: the index file cannot be looked at ({error}); a restart will not replay the index update"),
        }
        timing.reads_split.witness += witness_began.elapsed();
        reach("before-ref");
        // A commit built elsewhere made no scratch branch, so there is none to remove with the move.
        let removal: String = if first.commit.is_some() { String::new() } else { format!("delete {scratch}\n") };
        // A detached head is the `HEAD` file itself and is updated as such. The update follows a symbolic `HEAD` as native Git's
        //  own commit does, because a compare that does not follow it replaces a head attached to a branch since the last look
        //  (measured: `option no-deref` with `update HEAD <new> <old>` replaced `ref: refs/heads/B` at the old name with the
        //  detached new one). A head that became a branch at another value fails the compare; at the same value the branch it
        //  names takes the commit, and the head is read again below.
        let transaction: String = format!("start\nupdate {} {new} {old}\n{removal}prepare\ncommit\n", first.reference);
        let ref_began: Instant = Instant::now();
        // The write is never ended by the wrapper (owner, 2026-11-04: "Never cut a branch write (Recommended)"), so a write that never
        //  returns holds this thread; the clients that queue behind it are told what it is stuck in, and the commit it makes is on
        //  the scratch branch until the update renames the reference (`service_busy.rs`).
        let moved: GitOutput = {
            let _described = crate::service_busy_step_record_on_this_thread::detail(if first.commit.is_some() {
                format!("it moves {} to {new}", first.reference)
            } else {
                format!("it moves {} to {new}; the commit is also at {scratch} until the update ends", first.reference)
            });
            run(&git, repository, &["update-ref", "--stdin"], Some(transaction.into_bytes()))?
        };
        timing.ref_update += ref_began.elapsed();
        if !moved.succeeded() {
            // The branch moved under the batch: forget the intent, what the batch owes and the scratch branch, and build it again.
            let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
            crate::service_server_land_group::forget_what_a_batch_owes(&intent);
            remove_start(context.journal.as_path(), seq);
            let _deleted = run(&git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()], None);
            // For an amend, the next round finds the tip away from its base and answers that the branch moved.
            attempts_left -= 1;
            timing.hold += hold_began.elapsed();
            if attempts_left == 0 {
                return Err(FailureCode::BranchKeepsChanging);
            }
            continue;
        }
        reach("after-ref");
        // Git refuses a `symref-verify HEAD` beside an update of the branch `HEAD` names, so a switch after the last check cannot
        //  be caught inside the transaction. The head is read again with the lock held: a worktree that left the branch keeps
        //  its own index, and the commits stay on the branch they were made for. A switch after this look is the state native
        //  Git leaves too (`git commit` then `git symbolic-ref HEAD <other>`): the index describes the commit made.
        let follow_began: Instant = Instant::now();
        let followed: Result<bool, FailureCode> = follows_move(&git, repository, first.git_dir.as_path(), first.common_dir.as_path(), first.reference.as_str(), new.as_str());
        timing.reads_split.follow += follow_began.elapsed();
        // The branch has moved: the commits exist whatever happens to the index, which recovery finishes.
        let index_update_began: Instant = Instant::now();
        let settled: bool = finish_after_move(&git, &intent, held, context.journal.as_path(), followed);
        timing.index_update += index_update_began.elapsed();
        timing.hold += hold_began.elapsed();
        if settled {
            reach("after-index");
            let _marked = crate::service_journal::mark_done(context.journal.as_path(), seq);
        }
        for (slot, (index, _request)) in landing.iter().enumerate() {
            outcomes[*index] = Outcome::Landed { commit: commits[slot].clone() };
        }
        return Ok(outcomes);
    }
}

/// Landing controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_tests.rs"]
mod tests;

/// Controls for the adoption of the batches an earlier release's service left open,
/// which land real batches with the fixtures above.
#[cfg(test)]
#[path = "earlier_service_journal_adopted_tests.rs"]
mod adoption_tests;

/// Controls for settling the batches an earlier release's service left open when the person asks.
#[cfg(test)]
#[path = "earlier_batches_settled_on_request_tests.rs"]
mod settle_request_tests;

/// Controls for a replay over the index the landing saw stay out of the release executable.
#[cfg(test)]
#[path = "service_land_witness_tests.rs"]
mod witness_tests;

/// Controls for a service whose branch write does not return stay out of the release executable.
#[cfg(test)]
#[path = "service_land_hung_write_tests.rs"]
mod hung_write_tests;

/// Controls for the worktree's head around a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_head_tests.rs"]
mod head_tests;

/// Controls for the flushes of the Git children of a landing stay out of the release executable.
#[cfg(test)]
#[path = "service_land_fsync_tests.rs"]
mod fsync_tests;

/// Controls for the checks of a commit Git built stay out of the release executable.
#[cfg(test)]
#[path = "service_land_built_tests.rs"]
mod built_tests;

/// Controls for the push a landed commit leaves before its client is answered stay out of the release executable.
#[cfg(test)]
#[path = "service_land_owed_push_tests.rs"]
mod owed_push_tests;

/// Controls for the index update a landed batch owes stay out of the release executable.
#[cfg(test)]
#[path = "service_land_owed_index_tests.rs"]
mod owed_index_tests;

/// Controls for the restart decisions when a Git question about the branch fails stay out of the release executable.
#[cfg(test)]
#[path = "service_land_finish_git_failure_tests.rs"]
mod finish_git_failure_tests;
