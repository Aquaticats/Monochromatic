//! What:
//!  What became of one held commit when a settlement looked at it,
//!  and the line the service writes about it:
//!  still held,
//!  landed,
//!  landed with the index still owed,
//!  a conflict,
//!  a state this command cannot move,
//!  and a record whose commit the branch already shows.
//! Why:
//!  The settlement decides between these six,
//!  and the journal is the record a person reads across a whole machine,
//!  so every line names the repository its batch belongs to,
//!  says the commit is kept while it is held,
//!  and never words a held commit as one a lock refused:
//!  it is kept,
//!  waits for the lock and lands when the lock is gone
//!  (`SPEC.md`,
//!  "A commit held behind a reference lock").
//!  The states live here rather than in the settlement
//!  because the words need every one of them and the settlement needs nothing of the words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = settledWords(seq, held, settled); // one sentence per state, each naming the repository
//! ```

use crate::held_commit_record::Held;
use std::path::PathBuf;

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

/// What:
///  The one line the service writes about a recorded hold it has just settled,
///  worded by what became of the commit.
/// Why:
///   A held commit is kept and waits:
///  it lands when the lock is gone and never ends because the lock was there,
///  so the line says which repository the batch belongs to and what became of the branch,
///  and never a state a reader can take for a failed commit.
///  The repository comes first,
///  because a fixture's service writes into the same journal a person reads across a whole machine
///  and a line that says only `batch 41` reads as one about the repository its reader is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settledWords(7, held, settled) // 'batch 7 in the repository at /repo is still held: ...'
/// ```
pub fn settled_words(seq: u64, held: &Held, settled: &Settled) -> String {
    let repository: String = crate::service_timing::in_the_repository_at(held.repository.as_path());
    let branch: String = held.reference.clone();
    return match settled {
        Settled::WaitsForLocks(locks) => format!("batch {seq} {repository} is still held: the commit {} is kept, {branch} has not moved yet, and it waits for {}. The branch moves as soon as the lock is gone, on this command or the next one in that worktree; nothing is needed from you unless no Git process is using that repository, and then removing the lock with `{}` lets the commit land.", held.new, crate::held_commit_words::locks_named(locks.as_slice()), crate::held_commit_words::removal(locks.as_slice())),
        Settled::Landed { commit } => format!("batch {seq} {repository} was held: the lock is gone, and the commit {commit} is on {branch} now"),
        Settled::LandedTheIndexWaits { commit } => format!("batch {seq} {repository} was held: the lock is gone, and the commit {commit} is on {branch} now; the index catches up on the next command in that worktree"),
        Settled::Conflicts { commit, paths } => format!("batch {seq} {repository} is still held: the commit {commit} is kept, and it does not combine with {branch} as it is now on {}", paths_named(paths.as_slice())),
        Settled::Cannot { commit, because } => format!("batch {seq} {repository} is still held: the commit {commit} is kept, and this command cannot move {branch}, because {because}"),
        Settled::AlreadyThere { commit } => format!("batch {seq} {repository} is not held any more: {branch} already names {commit}"),
    };
}

/// The paths a conflict names,
/// each as the bytes the branch holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathsNamed(paths) // 'a.txt, b.txt'
/// ```
fn paths_named(paths: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::new();
    for path in paths {
        named.push(String::from_utf8_lossy(path).into_owned());
    }
    return named.join(", ");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "held_commit_settle_outcomes_tests.rs"]
mod tests;
