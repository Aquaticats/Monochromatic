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
//!  written before the branch write that a lock held off,
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
//!
//! The files beside this one hold what it reads and writes,
//! in the order it uses them:
//! `held_commit_settle_the_intent_of_a_batch.rs`,
//! `held_commit_settle_what_stands_in_the_way.rs`,
//! `held_commit_settle_carrying_the_change.rs`
//! and `held_commit_settle_what_the_move_owes.rs`;
//! this file holds the settlement that runs them in order.

use crate::held_commit_record::{
    Held,
    forget,
    list,
};
use crate::held_commit_replay::value_now;
use crate::held_commit_settle_carrying_the_change::carry;
use crate::held_commit_settle_the_intent_of_a_batch::{
    finished_of,
    intent_of,
    landed_below,
};
use crate::held_commit_settle_what_stands_in_the_way::readiness;
use crate::held_commit_settle_what_the_move_owes::{
    finish,
    finish_a_moved_branch,
    index_lock_of,
};
use crate::service_journal::Intent;
use crate::transaction_git::GitContext;
use std::path::{
    Path,
    PathBuf,
};

/// What became of one held commit,
/// and the words the service writes about it:
/// both live beside the settlement (`held_commit_settle_outcomes.rs`).
pub use crate::held_commit_settle_outcomes::Settled;

/// Whether a settlement may remove a reference lock that the judgement proves stale,
/// which every caller names through this file.
pub use crate::held_commit_settle_what_stands_in_the_way::Clearing;

/// How many times the settlement reads the branch again when another writer moved it.
const ATTEMPTS: usize = 3;

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
fn after_a_settled_batch(
    git: &GitContext,
    held: &Held,
) -> Settled {
    let Some(current) = value_now(git, held) else {
        return Settled::Cannot {
            commit: held.new.clone(),
            because: String::from("Git gave no answer about the branch"),
        };
    };
    if current == held.new {
        forget(held.git_dir.as_path(), held.seq);
        return Settled::AlreadyThere {
            commit: held.new.clone(),
        };
    }
    if let Some(done) = finished_of(held)
        && landed_below(git, held, &done, current.as_str())
    {
        forget(held.git_dir.as_path(), held.seq);
        return Settled::AlreadyThere {
            commit: done.new,
        };
    }
    return Settled::Cannot {
        commit: held.new.clone(),
        because: String::from("the batch of this record ended without landing its commit"),
    };
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
pub fn one(
    git: &GitContext,
    held: &Held,
    clearing: Clearing,
) -> Settled {
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
fn context_of(
    git: &GitContext,
    held: &Held,
) -> GitContext {
    let mut prefix: Vec<std::ffi::OsString> = git.global_prefix.clone();
    prefix.extend(crate::repository_selection::of_paths(
        held.git_dir.as_path(),
        held.repository.as_path(),
    ));
    return GitContext {
        global_prefix: prefix,
        ..git.clone()
    };
}

/// What:
///  `one` over a context that already names the record's repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// oneFor(acting, held, clearing)
/// ```
fn one_for(
    git: &GitContext,
    held: &Held,
    clearing: Clearing,
) -> Settled {
    if let Err(locks) = readiness(held, clearing) {
        return Settled::WaitsForLocks(locks);
    }
    let mut intent: Intent = match intent_of(held) {
        Ok(Some(found)) => found,
        Ok(None) => return after_a_settled_batch(git, held),
        Err(because) => {
            return Settled::Cannot {
                commit: held.new.clone(),
                because,
            };
        }
    };
    // The entries are installed under Git's index lock, taken before the branch moves and before the index file is recorded, so
    //  nothing can stage between that look and the installation (`index_lock_of`).
    let mut lock: Option<crate::service_index_lock::IndexLock> = index_lock_of(held, &intent);
    // A control settles the same record from inside this instant, which is where two commands that read the record and
    //  decide at once are (`held_commit_settle_tests.rs`).
    crate::service_fault::reach("held-before-a-settlement-decides");
    for _attempt in 0..ATTEMPTS {
        let Some(current) = value_now(git, held) else {
            return Settled::Cannot {
                commit: held.new.clone(),
                because: String::from("Git gave no answer about the branch"),
            };
        };
        // Another settlement of this same record can carry the change onto the branch and rewrite the batch's record while
        //  this one reads the branch, so the record on disk is the newer one: deciding from the value read before it would
        //  put the change on the branch a second time (`carry`).
        intent = match intent_of(held) {
            Ok(Some(found)) => found,
            Ok(None) => return after_a_settled_batch(git, held),
            Err(because) => {
                return Settled::Cannot {
                    commit: held.new.clone(),
                    because,
                };
            }
        };
        if landed_below(git, held, &intent, current.as_str()) {
            return finish_a_moved_branch(git, held, &intent, current.as_str(), lock.take());
        }
        let unborn: bool =
            current.is_empty() && crate::service_protocol::is_zero_object(intent.old.as_str());
        let (written, commit): (Intent, String) = if current == intent.old || unborn {
            (intent.clone(), intent.new.clone())
        } else {
            match carry(git, held, &intent, current.as_str()) {
                Ok(found) => found,
                Err(settled) => return settled,
            }
        };
        match crate::held_commit_settle_move::move_the_branch(
            git,
            held,
            &written,
            commit.as_str(),
            current.as_str(),
        ) {
            Ok(true) => {
                let waits: bool = finish(git, held, &written, lock.take());
                forget(held.git_dir.as_path(), held.seq);
                return if waits {
                    Settled::LandedTheIndexWaits {
                        commit,
                    }
                } else {
                    Settled::Landed {
                        commit,
                    }
                };
            }
            // The branch moved between the read and the write: read it and decide again.
            //  The record this round wrote is read back by the next one, so the decision is made from the branch and the record
            //  as they are then, and the witness recorded under the lock describes the index that is about to be read again;
            //  nothing was staged meanwhile because the lock is still held.
            Ok(false) => {}
            Err(said) => {
                return Settled::Cannot {
                    commit,
                    because: format!(
                        "Git refused the branch write ({said}), and the settlement waits for the next command"
                    ),
                };
            }
        }
    }
    return Settled::Cannot {
        commit: intent.new.clone(),
        because: String::from("the branch kept moving while its write was being made"),
    };
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
pub fn all(
    git: &GitContext,
    git_dir: &Path,
    clearing: Clearing,
) -> (Vec<(Held, Settled)>, Vec<PathBuf>) {
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
