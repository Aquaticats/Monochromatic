//! What:
//!  Looks for the commit a request that was already sent to the commit service made,
//! for a bounded time,
//! in the service's own journal and on the branch.
//! Why:
//!  An answer that was lost is not proof that the service never took the request.
//! The audit of service restarts and version skew (2026-11-09) saw the client end with
//!  "this commit was not made ...
//! Nothing in the repository changed" while the commit was on the branch,
//! and with the honest
//!  "not known whether the commit was made" while the service landed the commit seconds later.
//! The journal names the commit a finished batch made (`<seq>.done`) and the commit an open batch is
//!  making (`<seq>.intent`),
//! and the branch says whether that commit is made,
//! so one bounded look turns "not known" into the true answer instead of a re-run that the person has to make.
//!  The look never says a commit was not made:
//! when it finds nothing it leaves the words the failure already had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const commit = await lookingForWhatLanded({ context, cwd, journal, id, wait });
//! ```

use crate::held_commit_record::Held;
use crate::service_branch_shows::what_the_branch_shows;
use crate::service_messages::WhatTheBranchShows;
use crate::service_journal::{Intent, OPEN_EXTENSION, list_intents_read_only, newest_finished_batches_read_only};
use crate::service_protocol::CommitRequest;
use crate::service_units_of_removed_worktrees::journal_of;
use crate::scratch_location::state_directory;
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How often the look reads the journal and the branch again.
/// Why:
///   The look waits for a landing that is running,
/// which takes seconds,
/// so a quarter of a second is finer than the
///  thing waited for and costs one directory listing and,
/// while a batch is open,
/// one `git merge-base` each time.
pub const LOOK_EVERY: Duration = Duration::from_millis(250);

/// How many of the newest finished batches the look reads.
/// Why:
///   The request was sent seconds ago,
/// so the batch that made it is one of the newest.
/// Measured:
/// the busiest
///  unit in the service journal landed four batches in its busiest ten seconds
/// (cited with `DONE_RETAINED`,
///  2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z),
/// so this many reach back over forty seconds of that traffic,
///  four times the longest look.
pub const NEWEST_FINISHED_BATCHES: usize = 16;

/// What the look is given.
pub(crate) struct LookForWhatLanded<'look> {
    /// The context a Git read runs in.
    pub(crate) context: &'look GitContext,
    /// The directory the Git read runs in.
    pub(crate) cwd: &'look Path,
    /// The journal folder of the worktree's service,
    /// nothing when the environment names no state folder or the Git directory no instance.
    pub(crate) journal: Option<&'look Path>,
    /// The identity the request was sent with.
    pub(crate) id: &'look str,
    /// The longest the look lasts.
    pub(crate) wait: Duration,
}

/// What:
///  The commit this request made,
/// found in the journal and on the branch within `wait`.
/// Why:
///   A commit that is on the branch is a landed commit,
/// whatever the answer said,
/// and the owner's rule is that commits never fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lookingForWhatLanded(look) // undefined: nothing says the commit was made
/// ```
pub(crate) fn looking_for_what_landed(look: LookForWhatLanded<'_>) -> Option<String> {
    let journal: &Path = look.journal?;
    let began: Instant = Instant::now();
    loop {
        // A finished batch names the commit each of its requests became, and it is written only once the branch moved.
        if let Some(commit) = a_finished_batch_made(journal, look.id) {
            return Some(commit);
        }
        // An open batch names the commit it is making; the branch says whether it is made.
        match an_open_batch_making(journal, look.id) {
            Some((intent, commit)) => {
                if what_the_branch_shows(look.context, look.cwd, intent.reference.as_str(), commit.as_str()) == WhatTheBranchShows::Made {
                    return Some(commit);
                }
            }
            // No batch is landing this request, so nothing is about to move the branch for it: waiting cannot help.
            None => return None,
        }
        let waited: Duration = began.elapsed();
        if waited >= look.wait {
            return None;
        }
        std::thread::sleep(LOOK_EVERY.min(look.wait - waited));
    }
}

/// What:
///  The journal folder of the service that serves `git_dir`,
/// or nothing when the environment or the Git directory names none.
/// Why:
///   The unit starts the service with `--journal <state>/monochromatic-git-policy-cli/service/<instance>`,
/// and the client
///  that wrote the unit reads the same folder back to look for its commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// journalOfRequest(request) // undefined: no state folder or no instance
/// ```
pub(crate) fn journal_of_request(request: &CommitRequest) -> Option<PathBuf> {
    let instance: String = crate::service_instance::try_instance_of(request.git_dir.as_path()).ok()?;
    // The read of this process's environment is the read of the invocation's: `main` hands the entry `std::env::vars_os()` as
    //  that environment, and nothing in the crate changes the wrapper's own environment, so a command started with another
    //  `HOME` reads that home's state folder here (`repository_move_freeze.rs` reads the same one on the same terms).
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let wrapper_state: PathBuf = state_directory(environment.as_slice())?;
    return Some(journal_of(wrapper_state.as_path(), instance.as_str()));
}

/// What:
///  The commit a finished batch made for `id`,
/// nothing when no finished batch of the newest ones names it.
/// Why:
///   A finished batch's record is written after the branch moved,
/// so the commit it names is a fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishedBatches.find((record) => record.ids.includes(id))?.commits[at]
/// ```
fn a_finished_batch_made(journal: &Path, id: &str) -> Option<String> {
    for (_path, record) in newest_finished_batches_read_only(journal, NEWEST_FINISHED_BATCHES) {
        let Some(parsed) = record else { continue };
        if let Some(commit) = the_commit_of(&parsed, id) {
            return Some(commit);
        }
    }
    return None;
}

/// What:
///  The open batch that is landing `id`,
/// with the commit it is making,
/// nothing when no open batch names it.
/// Why:
///   The intent is written before the branch moves,
/// so an open batch that names the request is a landing that
///  may finish it,
/// and the commit it names is the one the branch will hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openBatches.find((record) => record.ids.includes(id))
/// ```
fn an_open_batch_making(journal: &Path, id: &str) -> Option<(Intent, String)> {
    for (_path, record) in list_intents_read_only(journal, OPEN_EXTENSION) {
        let Some(parsed) = record else { continue };
        if let Some(commit) = the_commit_of(&parsed, id) {
            return Some((parsed, commit));
        }
    }
    return None;
}

/// What:
///  The commit the open batch of `id` left held behind a reference lock,
///  nothing when no record of the worktree names that batch.
/// Why:
///   The service writes the record of the hold before it answers,
///  so a service that answered `held` into a connection that broke left the commit made and kept,
///  which is the state a received answer exits `0` in.
///  The record is the worktree's own,
///  and the batch it names is the one the journal lists for this request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const intent = openBatchMaking(journal, id); return heldRecords(gitDir).find((held) => held.seq === intent.seq && held.journal === journal)
/// ```
pub(crate) fn a_held_batch_of(journal: &Path, git_dir: &Path, id: &str) -> Option<Held> {
    let (intent, _commit): (Intent, String) = an_open_batch_making(journal, id)?;
    let (records, _damaged) = crate::held_commit_record::list(git_dir);
    for held in records {
        if held.seq == intent.seq && held.journal == journal {
            return Some(held);
        }
    }
    return None;
}

/// What:
///  The commit a record says `id` became,
/// nothing when the record does not name the request or names no commit for it.
/// Why:
///   A record that cannot say which commit a request became says nothing about that request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = record.ids.indexOf(id); return at < 0 ? undefined : record.commits[at]
/// ```
fn the_commit_of(record: &Intent, id: &str) -> Option<String> {
    for (at, named) in record.ids.iter().enumerate() {
        if named == id {
            return record.commits.get(at).cloned();
        }
    }
    return None;
}

/// The look is controlled in the test module.
#[cfg(test)]
#[path = "service_commit_looking_for_what_landed_tests.rs"]
mod tests;
