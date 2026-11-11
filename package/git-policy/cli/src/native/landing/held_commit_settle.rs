//! What:
//!  Moving the branch of a held commit once nothing stands in the way,
//!  and finishing what a landing owes after the move.
//! Why:
//!  The owner chose "Make it,
//! hold until lock goes" (2026-11-09):
//!  the commit is written and kept,
//!  the command exits `0`,
//!  and the next command that finds the lock gone moves the branch.
//!  What it finds is the batch's intent,
//!  written before the refused branch write,
//!  and the record of the hold beside it (`held_commit_record.rs`).
//!  The branch may have moved while the lock stood,
//!  and no client is left to prepare the commit again,
//!  so the change of the batch is carried onto the branch as it is
//!  (`held_commit_replay.rs`),
//!  which is the rule a commit that lands on a branch another session moved follows
//!  (`doc/decision/cli-git-concurrent-commits.md`).
//!  Every step is one compare-and-swap or the finish the recovery already makes,
//!  so a kill at any instruction leaves a state the next command settles again:
//!  the branch moves only from the value it was read at,
//!  and the index update is recorded before it is installed.
//!  Nothing here removes a lock:
//!  the proof that would allow it is the landing's own
//!  (`stale_ref_locks.rs`),
//!  and a command that merely resumes work never removes one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const [held, settled] of await settleHeldCommits(gitDir, clearing)) console.error(lineOf(held, settled));
//! ```

use crate::held_commit_record::{Held, forget, list};
use crate::held_commit_replay::{Replay, onto, value_now};
use crate::service_journal::{DONE_EXTENSION, Intent, OPEN_EXTENSION, batch_path, parse_intent, write_intent};
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// What a Git that could not be started is said to be,
/// in one sentence the callers share.
fn git_did_not_start(error: std::io::Error) -> String {
    return format!("Git could not be started ({error})");
}

/// The exit status `git merge-base --is-ancestor` gives when the first commit is not below the second.
const NOT_AN_ANCESTOR: i32 = 1;

/// How many times the settlement reads the branch again when another writer moved it.
const ATTEMPTS: usize = 3;

/// What became of one held commit.
#[derive(Debug, Eq, PartialEq)]
pub enum Settled {
    /// The branch moved and the index entries of the batch are installed.
    Landed {
        /// The commit the branch now names.
        commit: String,
    },
    /// The branch moved,
    /// and the index entries wait for the index lock or for a record this build cannot read.
    /// The record of the owed update finishes them at the next command (`owed_index_update.rs`).
    LandedTheIndexWaits {
        /// The commit the branch now names.
        commit: String,
    },
    /// A lock stands in the way and is named.
    WaitsForLocks(Vec<PathBuf>),
    /// The change does not combine with the branch as it is now.
    Conflicts {
        /// The commit kept on its side reference.
        commit: String,
        /// The paths that conflict.
        paths: Vec<Vec<u8>>,
    },
    /// Nothing can be done about it from here,
    /// for the reason given.
    Cannot {
        /// The commit kept on its side reference.
        commit: String,
        /// Why nothing was moved.
        because: String,
    },
    /// The branch already shows the commit of the record.
    AlreadyThere {
        /// The commit the branch names.
        commit: String,
    },
}

/// Whether a settlement may remove a reference lock that the judgement proves stale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Clearing {
    /// Never remove one:
    /// the caller is a command resuming work,
    /// not a landing.
    Never,
    /// Clear the ones a proof reaches,
    /// as the landing that waits for them does.
    Proved,
}

/// The reference a batch's commit is kept on while its branch has not moved.
pub fn kept_reference(sequence: u64) -> String {
    return crate::service_scratch_ref_of_a_dropped_batch::scratch_ref(sequence);
}

/// The Git directories whose reference locks can refuse a batch's branch write:
/// the common one holds the references,
/// and a linked worktree keeps its own `HEAD.lock` in its own Git directory.
fn lock_roots(held: &Held) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = vec![held.common_dir.clone()];
    if held.git_dir != held.common_dir {
        roots.push(held.git_dir.clone());
    }
    return roots;
}

/// The locks that stand in the given directories.
fn standing_locks(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut locks: Vec<PathBuf> = Vec::new();
    for candidate in crate::stale_ref_lock_walk::ref_lock_candidates(roots) {
        locks.push(candidate.lock);
    }
    return locks;
}

/// What:
///  Whether the branch write may be tried now,
///  and the locks that stand when it may not.
/// Why:
///   A live holder lets go within its step,
///  so a landing waits for one and lands;
///  a command that merely resumes a held commit does not wait at all,
///  it looks once and says what stands.
///  Under `Clearing::Proved` this is the landing's own wait with no time left in it,
///  which judges every standing lock once and clears the ones the proof reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const locks = standingLocks(roots); if (locks.length === 0) return { ready: true }; ... { ready: false, locks }
/// ```
fn readiness(roots: &[PathBuf], clearing: Clearing) -> Result<(), Vec<PathBuf>> {
    if matches!(clearing, Clearing::Proved) {
        let waiting = crate::service_land_reference_lock::wait_out_the_reference_locks(roots, &crate::stale_ref_locks::Judge::here(), Instant::now());
        if matches!(waiting, crate::service_land_reference_lock::Waiting::Gone) {
            return Ok(());
        }
    }
    let locks: Vec<PathBuf> = standing_locks(roots);
    return if locks.is_empty() { Ok(()) } else { Err(locks) };
}

/// What:
///  Moves the branch from the value it was read at to `new`,
///  and removes the side reference the commit was kept on.
/// Why:
///  The compare-and-swap is the one step that decides whether this settlement or another one
///  (another command,
///  or a landing) moved the branch,
///  so a commit can never be applied twice.
///  The message is one argument of `-m`,
///  never a line of the transaction,
///  so no byte of it can change the grammar Git reads on standard input
///  (`AGENTS.md` rules `SYB` and `STB`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// updateRef(['-m', reflog, '--stdin'], `start\nupdate ${ref} ${new} ${old}\ndelete ${scratch}\nprepare\ncommit\n`)
/// ```
fn move_branch(git: &GitContext, held: &Held, intent: &Intent, new: &str, expected: &str) -> Result<bool, String> {
    let scratch: String = kept_reference(held.seq);
    let text: String = format!("start\nupdate {} {new} {expected}\ndelete {scratch}\nprepare\ncommit\n", intent.reference);
    let mut request: GitRequest = GitRequest::new(held.repository.as_path(), &["update-ref"]);
    request.arguments.push(OsStr::new("-m").to_os_string());
    request.arguments.push(crate::path_bytes::os_from_path_bytes(held.reflog.as_slice()));
    request.arguments.push(OsStr::new("--stdin").to_os_string());
    request.input = Some(text.into_bytes());
    let output: GitOutput = run_git(git, &request).map_err(git_did_not_start)?;
    if output.succeeded() {
        return Ok(true);
    }
    return Err(String::from_utf8_lossy(output.stderr.as_slice()).trim().to_string());
}

/// What:
///  The intent of a held batch,
///  `Ok(None)` when it is gone,
///  and the reason when it cannot be read.
/// Why:
///   The intent is the batch's own record of where the branch was and where it goes;
///  without it nothing is decided about the commit,
///  and a record that cannot be read is said rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readIntent(journal, seq) // { intent } | { absent: true } | { damaged: true }
/// ```
fn intent_of(held: &Held) -> Result<Option<Intent>, String> {
    let path: PathBuf = batch_path(held.journal.as_path(), held.seq, OPEN_EXTENSION);
    let Ok(text) = std::fs::read_to_string(path.as_path()) else {
        return Ok(None);
    };
    return match parse_intent(text.as_str()) {
        Some(intent) => Ok(Some(intent)),
        None => Err(format!("the record of the batch at `{}` cannot be read", path.display())),
    };
}

/// Whether the commit the branch names is the batch's own or lies above it,
/// so the batch landed and only its index update is owed.
fn landed_below(git: &GitContext, held: &Held, intent: &Intent, current: &str) -> bool {
    let request: GitRequest = GitRequest::new(held.repository.as_path(), &["merge-base", "--is-ancestor", intent.new.as_str(), current]);
    let Ok(output) = run_git(git, &request) else {
        return false;
    };
    if output.succeeded() {
        return true;
    }
    return output.code != Some(NOT_AN_ANCESTOR) && current == intent.new;
}

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
fn index_lock_of(held: &Held, intent: &Intent) -> Option<crate::service_index_lock::IndexLock> {
    let lock: crate::service_index_lock::IndexLock = crate::service_index_lock::try_acquire(intent.index.as_path(), intent.seq)?;
    match crate::service_land_start::witness_of(intent.index.as_path()) {
        Ok(witness) => {
            if let Err(error) = crate::service_journal::write_start(held.journal.as_path(), intent.seq, &witness) {
                crate::service_warn!("batch {}: the index file the settlement saw could not be recorded ({error}); the entries are left to the owed record", intent.seq);
            }
        }
        Err(error) => crate::service_warn!("batch {}: the index file cannot be looked at ({error}); the entries are left to the owed record", intent.seq),
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
fn install_entries(git: &GitContext, held: &Held, intent: &Intent, lock: crate::service_index_lock::IndexLock) -> bool {
    let own_index: bool = intent.index == intent.git_dir.join("index");
    let ours: Result<bool, crate::service_protocol::FailureCode> =
        crate::service_head::still_checked_out(git, intent.repository.as_path(), intent.git_dir.as_path(), intent.common_dir.as_path(), intent.reference.as_str());
    let installed: bool = match ours {
        Ok(false) if own_index => {
            drop(lock);
            crate::service_land_follow::cleanup_done(git, intent.repository.as_path(), intent, held.journal.as_path())
        }
        Err(code) => {
            drop(lock);
            crate::service_error!("batch {} stays open: the head of the worktree cannot be read ({code:?}), so the index is left as it is", intent.seq);
            false
        }
        Ok(_) => crate::service_land_finish::follow_up(git, intent.repository.as_path(), intent, lock, held.journal.as_path()),
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
fn finish(git: &GitContext, held: &Held, intent: &Intent, lock: Option<crate::service_index_lock::IndexLock>) -> bool {
    crate::service_server_land_group::hold_the_pushes_of(intent);
    crate::service_index_lock::remove_stale(intent.index.as_path(), intent.seq);
    let waits: bool = match lock {
        Some(held_lock) => install_entries(git, held, intent, held_lock),
        None => {
            if crate::service_land_finish::settle(git, intent, held.journal.as_path()) == crate::service_land_finish::Settled::Done {
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
fn finish_a_moved_branch(git: &GitContext, held: &Held, intent: &Intent, current: &str, lock: Option<crate::service_index_lock::IndexLock>) -> Settled {
    let mut moved: Intent = intent.clone();
    moved.new = current.to_string();
    let waits: bool = finish(git, held, &moved, lock);
    forget(held.git_dir.as_path(), held.seq);
    return if waits { Settled::LandedTheIndexWaits { commit: current.to_string() } } else { Settled::Landed { commit: current.to_string() } };
}

/// What:
///  Carries the batch onto the branch as it is now,
///  and answers the intent that names the commit the branch will take with it.
/// Why:
///   The replay writes the commits and nothing else;
///  the branch is moved by the caller,
///  from the value it read,
///  so two settlements cannot both move it.
///  The rewritten intent is on disk before the move,
///  so a kill between the two leaves an intent that names the commit this release puts on the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replayed = await carryOnto(held, current); if (replayed.kind !== 'replayed') return replayed; await writeIntent({ ...intent, new: last, commits })
/// ```
fn carry(git: &GitContext, held: &Held, intent: &Intent, current: &str) -> Result<(Intent, String), Settled> {
    if crate::service_protocol::is_zero_object(intent.old.as_str()) {
        return Err(Settled::Cannot {
            commit: held.new.clone(),
            because: String::from("the branch was created while the lock stood, and the commit was made for a branch that did not exist"),
        });
    }
    return match onto(git, held, current, held.common_dir.as_path()) {
        Replay::Replayed { commits } => {
            let Some(last) = commits.last().cloned() else {
                return Err(Settled::Cannot { commit: held.new.clone(), because: String::from("the batch names no commit") });
            };
            let mut written: Intent = intent.clone();
            written.new = last.clone();
            written.commits = commits;
            match write_intent(held.journal.as_path(), &written) {
                Ok(()) => Ok((written, last)),
                Err(error) => Err(Settled::Cannot { commit: last, because: format!("the record of the batch cannot be written ({error})") }),
            }
        }
        Replay::Conflict { paths } => Err(Settled::Conflicts { commit: held.new.clone(), paths }),
        Replay::Cannot { because } => Err(Settled::Cannot { commit: held.new.clone(), because }),
    };
}

/// What:
///  The record of the batch this held record belongs to once that batch ended,
///  nothing when no finished record is there.
/// Why:
///   A settlement that carries the batch onto a branch that moved writes the commit it made in the batch's own record
///  before it moves the branch (`carry`),
///  and the finished record is what names that commit after the branch moved.
///  A kill between the finished record and the removal of this one leaves this one naming the commit the batch started
///  with,
///  so the finished record is the only place the commit that landed is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = readIntent(batchPath(journal, seq, 'done'))
/// ```
fn finished_of(held: &Held) -> Option<Intent> {
    let path: PathBuf = batch_path(held.journal.as_path(), held.seq, DONE_EXTENSION);
    let text: String = std::fs::read_to_string(path.as_path()).ok()?;
    return parse_intent(text.as_str());
}

/// What:
///  What a record whose batch has no intent left stands for.
/// Why:
///   A kill between the end of a batch and the removal of its record leaves one,
///  and the branch says whether the commit is on it.
///  The branch can hold the commit the batch started with,
///  and it can hold the commit a settlement made from it on a branch that moved:
///  the batch's finished record names that one,
///  and a comparison that read only the first would say for ever that the batch ended without landing
///  (`carry`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// current === held.new ? (rm(record), { alreadyThere: held.new }) : done && isAncestor(done.new, current) ? (rm(record), { alreadyThere: done.new }) : { cannot: 'the batch of this record ended without landing its commit' }
/// ```
fn after_a_settled_batch(git: &GitContext, held: &Held) -> Settled {
    let Some(current) = value_now(git, held) else {
        return Settled::Cannot { commit: held.new.clone(), because: String::from("Git gave no answer about the branch") };
    };
    if current == held.new {
        forget(held.git_dir.as_path(), held.seq);
        return Settled::AlreadyThere { commit: held.new.clone() };
    }
    if let Some(done) = finished_of(held)
        && landed_below(git, held, &done, current.as_str())
    {
        forget(held.git_dir.as_path(), held.seq);
        return Settled::AlreadyThere { commit: done.new };
    }
    return Settled::Cannot { commit: held.new.clone(), because: String::from("the batch of this record ended without landing its commit") };
}

/// What:
///  Settles one held commit:
///  looks at the locks,
///  moves the branch or carries the change onto it,
///  and finishes what the move owes.
/// Why:
///  Each outcome is a sentence the person reads
///  (`held_commit_words.rs`),
///  and every path that cannot finish leaves the record where it is,
///  so the next command tries again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (locksStand()) return { waitsFor: locks }; const current = revParse(); ...
/// ```
pub fn one(git: &GitContext, held: &Held, clearing: Clearing) -> Settled {
    let acting: GitContext = context_of(git, held);
    return one_for(&acting, held, clearing);
}

/// What:
///  The Git context every start of a settlement runs in:
///  the caller's,
///  with the repository the record names after whatever its own options wrote.
/// Why:
///  A Git directory beside the work tree is found only when it is named,
///  and the record's absolute paths must win over an earlier relative option
///  (`repository_selection.rs`,
///  `of_paths`):
///  Git reads the last of two `--git-dir` options
///  (measured on 2026-11-10 with 2.56.0,
///  `git --git-dir=a --git-dir=b rev-parse --absolute-git-dir` answers `b`,
///  and the same line the other way round answers `a`),
///  so the record's selection goes after the caller's own.
///  The caller's own options stay in front,
///  so a plain layout keeps the context it was given and nothing starts Git that would not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const acting = { ...context, globalPrefix: [...context.globalPrefix, ...ofPaths(held.gitDir, held.repository)] };
/// ```
fn context_of(git: &GitContext, held: &Held) -> GitContext {
    let mut prefix: Vec<std::ffi::OsString> = git.global_prefix.clone();
    prefix.extend(crate::repository_selection::of_paths(held.git_dir.as_path(), held.repository.as_path()));
    return GitContext { global_prefix: prefix, ..git.clone() };
}

/// What:
///  `one` over a context that already names the record's repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oneFor(acting, held, clearing)
/// ```
fn one_for(git: &GitContext, held: &Held, clearing: Clearing) -> Settled {
    if let Err(locks) = readiness(lock_roots(held).as_slice(), clearing) {
        return Settled::WaitsForLocks(locks);
    }
    let mut intent: Intent = match intent_of(held) {
        Ok(Some(found)) => found,
        Ok(None) => return after_a_settled_batch(git, held),
        Err(because) => return Settled::Cannot { commit: held.new.clone(), because },
    };
    // The entries are installed under Git's index lock, taken before the branch moves and before the index file is recorded, so
    //  nothing can stage between that look and the installation (`index_lock_of`).
    let mut lock: Option<crate::service_index_lock::IndexLock> = index_lock_of(held, &intent);
    // A control settles the same record from inside this instant, which is where two commands that read the record and
    //  decide at once are (`held_commit_settle_tests.rs`).
    crate::service_fault::reach("held-before-a-settlement-decides");
    for _attempt in 0..ATTEMPTS {
        let Some(current) = value_now(git, held) else {
            return Settled::Cannot { commit: held.new.clone(), because: String::from("Git gave no answer about the branch") };
        };
        // Another settlement of this same record can carry the change onto the branch and rewrite the batch's record while
        //  this one reads the branch, so the record on disk is the newer one: deciding from the value read before it would
        //  put the change on the branch a second time (`carry`).
        intent = match intent_of(held) {
            Ok(Some(found)) => found,
            Ok(None) => return after_a_settled_batch(git, held),
            Err(because) => return Settled::Cannot { commit: held.new.clone(), because },
        };
        if landed_below(git, held, &intent, current.as_str()) {
            return finish_a_moved_branch(git, held, &intent, current.as_str(), lock.take());
        }
        let unborn: bool = current.is_empty() && crate::service_protocol::is_zero_object(intent.old.as_str());
        let (written, commit): (Intent, String) = if current == intent.old || unborn {
            (intent.clone(), intent.new.clone())
        } else {
            match carry(git, held, &intent, current.as_str()) {
                Ok(found) => found,
                Err(settled) => return settled,
            }
        };
        // What: Everything the batch owes is taken here, before the branch moves.
        // Why:  A kill between the move and the index install leaves the record of the index update and the held push of
        //  every commit the move lands, and the push is held before the branch names it (`work_item_holding.rs`).
        //  A settlement that is refused takes them back below, because a branch that did not move owes nothing.
        crate::service_server_land_group::what_a_batch_owes(&written);
        match move_branch(git, held, &written, commit.as_str(), current.as_str()) {
            Ok(true) => {
                let waits: bool = finish(git, held, &written, lock.take());
                forget(held.git_dir.as_path(), held.seq);
                return if waits { Settled::LandedTheIndexWaits { commit } } else { Settled::Landed { commit } };
            }
            // The branch moved between the read and the write: read it and decide again.
            //  The record this round wrote is read back by the next one, so the decision is made from the branch and the record
            //  as they are then, and the witness recorded under the lock describes the index that is about to be read again;
            //  nothing was staged meanwhile because the lock is still held.
            Ok(false) => {
                crate::service_server_land_group::forget_what_a_batch_owes(&written);
            }
            Err(said) => {
                crate::service_server_land_group::forget_what_a_batch_owes(&written);
                return Settled::Cannot { commit, because: format!("Git refused the branch write ({said}), and the settlement waits for the next command") };
            }
        }
    }
    return Settled::Cannot { commit: intent.new.clone(), because: String::from("the branch kept moving while its write was being made") };
}

/// What:
///  Settles the batch a restart found open when a record says a reference lock kept it off its branch,
///  and answers the report of a batch still waiting.
/// Why:
///   A restart decides an open batch from the branch,
///  and while the lock stands the branch still names the commit before it,
///  so that decision would drop a commit the person made and kept.
///  The settlement runs first with the clearing a landing may do,
///  and a record that is still there says the batch waits:
///  nothing is decided about it until a command finds the lock gone.
///  Nothing is answered when the record is gone:
///  the batch is then either finished (nothing is owed and its intent is marked done) or still open,
///  and the caller reads the batch's own record to say which and to report it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = recordOf(intent, journal); if (!held) return undefined; settleOne(held); return recordStillThere ? { handled: 'held' } : undefined
/// ```
pub fn settle_a_recorded_hold(git: &GitContext, journal: &Path, intent: &Intent) -> Option<crate::service_land_finish::RecoveryReport> {
    let held: Held = record_of(intent, journal)?;
    let settled: Settled = one(git, &held, Clearing::Proved);
    let seq: u64 = intent.seq;
    crate::service_log!("batch {seq} is held behind a reference lock: {settled:?}");
    let still_held: bool = crate::held_commit_record::path_of(intent.git_dir.as_path(), seq).exists();
    if !still_held {
        return None;
    }
    return Some(crate::service_land_finish::RecoveryReport {
        seq,
        decision: crate::service_journal::Recovery::Hold,
        handled: crate::service_land_finish::Handled::Held,
    });
}

/// What:
///  Says that the record of a batch a settlement tried cannot be read,
///  so the batch is kept and nothing of it is removed.
/// Why:
///   A settlement rewrites a batch's intent,
///  and the record it leaves is the only thing that names what the batch still owes;
///  one this build cannot read is kept and named rather than dropped,
///  and its owed index update is settled from the worktree's own record of it
///  (`owed_index_update.rs`).
///  The settling module says it,
///  beside the settlement whose outcome it describes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saySettledRecordUnreadable(intent, path) { error(`batch ${intent.seq}: its open record ${path} cannot be read ...`) }
/// ```
pub fn say_settled_batch_record_cannot_be_read(intent: &Intent, path: &Path) {
    crate::service_error!(
        "batch {}: its open record {} cannot be read after the batch was settled, so it is kept and nothing of it is settled; its \
owed index update is settled from the worktree's own record of it",
        intent.seq,
        path.display()
    );
}

/// What:
///  The record of the hold of a batch a restart found open,
///  nothing when the batch was not held.
/// Why:
///  A restart reads the journal and knows the batch,
///  and the record is the only thing that says a lock kept it off its branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = await list(gitDir).find((record) => record.seq === intent.seq && record.journal === journal)
/// ```
pub fn record_of(intent: &Intent, journal: &Path) -> Option<Held> {
    let (records, _damaged) = list(intent.git_dir.as_path());
    for held in records {
        if held.seq == intent.seq && held.journal == journal {
            return Some(held);
        }
    }
    return None;
}

/// What:
///  Every held commit of one worktree,
///  settled in the order their batches were made,
///  with what became of each.
/// Why:
///  Two held commits on one branch are settled in order,
///  so the second one is carried onto the branch the first one moved,
///  which is the order the commits were prepared in.
///  A record that cannot be read is kept and named,
///  never removed:
///  it names a commit only the side reference holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const held of await list(gitDir)) results.push([held, settleOne(held, clearing)]);
/// ```
pub fn all(git: &GitContext, git_dir: &Path, clearing: Clearing) -> (Vec<(Held, Settled)>, Vec<PathBuf>) {
    let (held_records, damaged) = list(git_dir);
    let mut settled: Vec<(Held, Settled)> = Vec::new();
    for held in held_records {
        settled.push((held.clone(), one(git, &held, clearing)));
    }
    return (settled, damaged);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "held_commit_settle_tests.rs"]
mod tests;
