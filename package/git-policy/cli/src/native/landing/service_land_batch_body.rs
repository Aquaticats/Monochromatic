//! What:
//!  The landing itself:
//! one batch of prepared commits written,
//! journaled and moved onto a branch.
//! Why:
//!  The entry points decide what is told to the clients and what a step ended at its ceiling records;
//!  the order of the durable writes and the branch move is the body's own,
//!  so it stands in a file that the entry points call and nothing else does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await landBatchBody(context, seq, requests, announce, timing, settings);
//! ```

use crate::service_land_take_index_lock::take_index_lock;
use crate::service_land_git_steps_and_import_stream::{checked, ids_of, requests_that_land, run, run_words};
use crate::service_land_objects_of_batch::objects_of_batch;
use crate::service_fault::reach;
use crate::service_built_checks::{BuiltCommit, read_commit};
use crate::service_head::{follows_move, still_checked_out};
use crate::service_pruned::batch_content_missing;
use crate::service_journal::{Intent, OPEN_EXTENSION, remove_start, write_intent, write_start};
use crate::service_land_finish::finish_after_move;
use crate::service_land_start::witness_of;
use crate::service_index_lock::{Announcer, IndexLock};
use crate::service_timing::LandTiming;
use crate::service_scratch_ref_of_a_dropped_batch::scratch_ref;
use crate::service_land_follow::storage_failure;
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput, NO_REFLOG_SETTING};
use std::path::Path;
use std::time::Instant;

use crate::service_land_context::LandContext;
use crate::service_land_outcomes::Outcome;
use crate::service_land_reference_lock::{Waiting, wait_out_the_reference_locks};
use crate::service_land_settings::{LANDING_ATTEMPTS, Settings};
/// The landing itself;
/// `land_batch_timed` adds what is told when a step is ended at its ceiling.
///
/// # Errors
///
/// As `land_batch_timed`.
pub(crate) fn land_batch_body(
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
    // The Git directories whose reference locks may refuse this batch's branch write: the common one holds the references
    //  and the packed references, and a linked worktree keeps its own `HEAD.lock` in its own Git directory.
    let mut lock_roots: Vec<std::path::PathBuf> = vec![first.common_dir.clone()];
    if first.git_dir != first.common_dir {
        lock_roots.push(first.git_dir.clone());
    }
    let scratch: String = scratch_ref(seq);
    let mut outcomes: Vec<Outcome>;
    let mut attempts_left: usize = LANDING_ATTEMPTS;
    // When the branch write was first refused while the branch stood still: every wait for a reference lock that refused
    //  it shares this one bound, so a lock that keeps being taken again cannot hold the landing past it.
    let mut lock_deadline: Option<Instant> = None;
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
        // What the reflog row says: the kind of commit and the first line of its message, as native Git words both
        // (`builtin/commit.c`, `update_head_with_reflog` in `sequencer.c`). A landing answers a batch with one branch move, so
        // the row names the commit that landed last, which is the value the branch now holds.
        let last_request: &CommitRequest = batch.last().copied().unwrap_or(first);
        let content: Option<crate::service_built_checks::CommitContent> = crate::service_built_checks::commit_content(first.common_dir.as_path(), new.as_str());
        let parents: usize = match &content {
            Some(read) => read.parents.len(),
            None => 0,
        };
        let copy: Option<std::path::PathBuf> =
            crate::service_land_reflog::conclusion_copy(context.journal.as_path(), seq, first.cleanup.is_some());
        let initial: bool = unborn || is_zero_object(old.as_str());
        let committed: crate::service_land_reflog::Committed = crate::service_land_reflog::kind(last_request.amend, initial, parents, copy.as_deref());
        let subject: Vec<u8> = match content {
            Some(read) => read.message,
            None => last_request.message.clone().into_bytes(),
        };
        let reflog: std::ffi::OsString = crate::service_land_reflog::message(committed, subject.as_slice());
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
        // The message is one argument of `-m`, never a line of the transaction, so no byte of the commit's message can change
        // the grammar Git reads on standard input (`AGENTS.md` rules `SYB` and `STB`).
        let moving: Vec<std::ffi::OsString> = vec![
            std::ffi::OsString::from("update-ref"),
            std::ffi::OsString::from("-m"),
            reflog.clone(),
            std::ffi::OsString::from("--stdin"),
        ];
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
            run_words(&git, repository, moving.as_slice(), Some(transaction.into_bytes()))?
        };
        timing.ref_update += ref_began.elapsed();
        if !moved.succeeded() {
            // A branch that is no longer where this batch found it is the race the loop is built for. A branch that still names
            //  it did not move, so something else refused the write, and a reference lock is what refuses it when one stands: the
            //  wrapper clears the ones nothing live holds, waits out a live holder within one bound, holds the commit behind the
            //  rest, and names them with the command that clears them. Neither a lock nor a wait for one is a branch that changed.
            let standing: String = checked(&git, repository, &["rev-parse", "--verify", "--quiet", branch_commit.as_str()], None).unwrap_or_default();
            if standing == old {
                // The index lock is given up while the wait runs, so another program's commit is not held behind this one.
                drop(held);
                let deadline: Instant = match lock_deadline {
                    Some(known) => known,
                    None => {
                        let first_refusal: Instant = Instant::now() + crate::wait_ceilings::REFERENCE_LOCK_WAIT;
                        lock_deadline = Some(first_refusal);
                        first_refusal
                    }
                };
                let within_bound: bool = Instant::now() < deadline;
                crate::service_land_hold_a_batch::say_it_waits(seq, first.reference.as_str(), old.as_str(), within_bound);
                match wait_out_the_reference_locks(lock_roots.as_slice(), &crate::stale_ref_locks::Judge::here(), deadline) {
                    // Nothing stands any more and the bound for waiting on one has not passed: the branch is where it was,
                    //  so the write may be tried again.
                    Waiting::Gone if within_bound => {
                        timing.hold += hold_began.elapsed();
                        let _intent_removed = std::fs::remove_file(crate::service_journal::batch_path(context.journal.as_path(), seq, OPEN_EXTENSION));
                        crate::service_server_land_group::forget_what_a_batch_owes(&intent);
                        remove_start(context.journal.as_path(), seq);
                        let _deleted = run(&git, repository, &["-c", NO_REFLOG_SETTING, "update-ref", "-d", "--no-deref", scratch.as_str()], None);
                        continue;
                    }
                    // The bound has passed and nothing stands now: the refusal this batch keeps meeting is Git's own, and
                    //  its words are what the person is told (`service_land_hold_a_batch.rs`).
                    Waiting::Gone => {
                        return Err(crate::service_land_hold_a_batch::give_up(&crate::service_land_hold_a_batch::GivenUp {
                            git: &git,
                            context,
                            repository,
                            seq,
                            intent: &intent,
                            scratch: scratch.as_str(),
                            reference: first.reference.as_str(),
                            stderr: moved.stderr.as_slice(),
                            announce,
                        }));
                    }
                    // The owner chose "Make it, hold until lock goes" (2026-11-09): the commit is written and kept, the command
                    //  exits 0, and the next command that finds the lock gone moves the branch (`service_land_hold_a_batch.rs`).
                    Waiting::Blocked { words, locks } => {
                        let refused: crate::service_land_hold_a_batch::ARefusedBatch<'_> = crate::service_land_hold_a_batch::ARefusedBatch {
                            git: &git,
                            context,
                            repository,
                            first,
                            seq,
                            intent: &intent,
                            scratch: scratch.as_str(),
                            new: new.as_str(),
                            reflog: &reflog,
                            locks,
                            words,
                            announce,
                            timing,
                            held_at: hold_began,
                            requests: requests.len(),
                        };
                        return crate::service_land_hold_a_batch::hold_the_batch(refused);
                    }
                }
            }
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
