//! What:
//!  The commit a command makes when the commit service stopped answering it:
//!  the objects,
//!  the batch's record,
//!  and the branch move,
//!  in this process,
//!  through the whole machinery a commit a reference lock held already uses.
//! Why:
//!  A service that stops answering is an ordinary event on this machine
//!  (a process frozen by the kernel under memory pressure,
//!  one stopped by a debugger or a signal,
//!  a disk stall of tens of seconds,
//!  a service being replaced),
//!  and the rule it meets is that a commit never fails where native Git would succeed:
//!  native Git, on the same repository with no service at all, makes the commit.
//!  The commit was prepared here already:
//!  the private index,
//!  the policies,
//!  the message and the identities are the ones this process would have sent,
//!  so what is left is what a landing does,
//!  and it is done with the service's own steps:
//!  the objects (`objects_of_batch`),
//!  the record of the batch (`record_the_batch`),
//!  the record of the hold,
//!  and the settlement a held commit already is (`held_commit_settle`),
//!  which moves the branch by one compare-and-swap,
//!  installs the index under Git's own lock,
//!  removes the conclusion state and holds the pushes.
//!  Two things make that safe beside a service that may wake up and land the same request:
//!  the commit's identity is a function of the request
//!  (the same parents, tree, identities and message through the same import),
//!  so a branch that names it is the answer and not a second commit,
//!  and the branch moves only from the value the batch was prepared on,
//!  so exactly one of the two moves it,
//!  and the other finds it there (`landed_below`).
//!  The connection is closed before any of this, which is the withdrawal the service checks before it acts
//!  (`service_land_requests_whose_client_is_gone.rs`),
//!  so a service that wakes finds no client and makes nothing.
//!  Nothing here waits:
//!  the side reference and Git's index lock are taken without a wait,
//!  a lock that stands leaves the commit held with its record,
//!  which the next command settles.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await landAfterTheServiceStopped(git, request); // { landed } | { held } | { notMade, words }
//! ```

use crate::held_commit_record::{Held, Holding, hold};
use crate::held_commit_settle::{Clearing, Settled, one};
use crate::service_client::failure_for;
use crate::service_journal::{BatchClaim, claim_batch};
use crate::service_land_context::LandContext;
use crate::service_land_objects_of_batch::objects_of_batch;
use crate::service_land_record_the_batch::{TheBatch, of as record_the_batch};
use crate::service_messages::{ClientFailure, render};
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::service_land_git_steps_and_import_stream::checked;
use crate::service_scratch_ref_of_a_dropped_batch::scratch_ref;
use crate::service_timing::LandTiming;
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// What became of the commit a command made for itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WithoutTheService {
    /// The commit is on the branch,
    /// and the branch move happened here.
    Landed {
        /// The commit.
        commit: String,
        /// Whether the index update is still owed,
        /// which the record of the batch settles at the next command.
        the_index_waits: bool,
    },
    /// The commit is written and kept,
    /// and a reference lock keeps its branch where it is:
    /// the record says what it waits for and the next command moves the branch.
    Held {
        /// The commit kept on the batch's side reference.
        commit: String,
        /// What the command says about it.
        words: String,
    },
    /// The commit could not be made here either,
    /// and the person is told what became of it and where it is kept.
    NotMade {
        /// The commit that was written and is kept by the reference,
        /// nothing when nothing was written.
        commit: Option<String>,
        /// What the command says about it.
        words: String,
    },
}

/// What:
///  Makes the commit of `request` in this process,
///  and answers what became of it.
///  `journal` is the folder of the worktree's service journal,
///  which the batch is recorded in,
///  and nothing when the environment names no state folder or the Git directory no instance.
/// Why:
///   The caller sends this only when the service gave no answer,
///  so the branch is moved here or the commit is kept with its record;
///  nothing of the person's index or worktree is touched before the branch moves,
///  and the steps are the landing's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await landAfterTheServiceStopped(git, request)
/// ```
pub(crate) fn land_after_the_service_stopped(git: &GitContext, request: &CommitRequest, journal: Option<&Path>) -> WithoutTheService {
    let Some(folder) = journal.map(Path::to_path_buf) else {
        return WithoutTheService::NotMade {
            commit: None,
            words: String::from("this commit was not made: the commit service gave no answer, and the batch this commit would be recorded in has no folder to be recorded in, because the environment names no state folder or the Git directory no service instance. The index and the worktree are as they were."),
        };
    };
    if let Err(error) = std::fs::create_dir_all(folder.as_path()) {
        return without_a_record(error.to_string().as_str());
    }
    let claim: BatchClaim = match claim_batch(folder.as_path(), a_first_batch_number()) {
        Ok(taken) => taken,
        Err(error) => return without_a_record(error.to_string().as_str()),
    };
    let context: LandContext = LandContext {
        git: git.clone(),
        journal: folder.clone(),
        on_objects_written: None,
        lock_ceiling: crate::wait_ceilings::LANDING_LOCK_CEILING,
    };
    let seq: u64 = claim.seq();
    // The batch is the one request, prepared against the branch as it was when this command read it.
    let old: &str = request.base.as_str();
    let unborn: bool = is_zero_object(old);
    let amended_parents: Vec<String> = match replaced_parents(git, request) {
        Ok(parents) => parents,
        Err(code) => return not_made_by_the_repository(code, request, None),
    };
    let mut timing: LandTiming = LandTiming::default();
    let (commits, new): (Vec<String>, String) = match objects_of_batch(
        &context.git,
        folder.as_path(),
        seq,
        request,
        scratch_ref(seq).as_str(),
        old,
        &[request],
        amended_parents.as_slice(),
        // A commit Git built is checked against Git's own listing here: the settings a service read once before its first
        //  landing are not this process's, and the safe answer costs one `rev-list`.
        false,
        None,
    ) {
        Ok(made) => made,
        Err(code) => return not_made_by_the_repository(code, request, None),
    };
    let (intent, reflog) = match record_the_batch(TheBatch {
        context: &context,
        timing: &mut timing,
        seq,
        first: request,
        batch: &[request],
        commits: commits.as_slice(),
        old,
        new: new.as_str(),
        unborn,
    }) {
        Ok(recorded) => recorded,
        Err(code) => return not_made_by_the_repository(code, request, Some(new.as_str())).with_commit(commits.last().cloned()),
    };
    // The record of the hold is published before the branch moves, so a kill at any instruction leaves a commit the next
    //  command settles instead of an object nothing names.
    let held: Held = match hold(Holding {
        journal: folder.as_path(),
        repository: request.repository.as_path(),
        common_dir: request.common_dir.as_path(),
        git_dir: request.git_dir.as_path(),
        index: request.index.as_path(),
        reference: &request.reference,
        seq,
        intent: &intent,
        locks: Vec::new(),
        reflog: crate::path_bytes::path_bytes_of(Path::new(reflog.as_os_str())),
    }) {
        Ok(record) => record,
        Err(error) => return not_made_by_the_repository(FailureCode::Unavailable, request, Some(new.as_str())).with_words(error.to_string().as_str()).with_commit(commits.last().cloned()),
    };
    return match one(&context.git, &held, Clearing::Proved) {
        Settled::Landed { commit } | Settled::AlreadyThere { commit } => WithoutTheService::Landed { commit, the_index_waits: false },
        Settled::LandedTheIndexWaits { commit } => WithoutTheService::Landed { commit, the_index_waits: true },
        Settled::WaitsForLocks(standing) => {
            // Nothing is removed and nothing is forced: the record stays, and the next command that finds the lock gone moves
            //  the branch (`held_commit_settle.rs`).
            // The locks the settlement found are recorded beside the commit, so the words here and every notice of a later
            //  command name them and the command that clears each.
            let mut named: Held = held.clone();
            named.locks = standing;
            let _recorded = crate::held_commit_record::write(request.git_dir.as_path(), &named);
            let short: String = crate::landed_summary::short_name(&context.git, request.repository.as_path(), request.common_dir.join("objects").as_path(), None, held.new.as_str());
            let waiting: usize = crate::held_commit_record::list(request.git_dir.as_path()).0.len();
            WithoutTheService::Held { commit: held.new.clone(), words: crate::held_commit_words::answer(&named, short.as_str(), waiting) }
        }
        Settled::Conflicts { commit, paths } => WithoutTheService::NotMade {
            commit: Some(commit),
            words: kept_words(crate::held_commit_words::conflicts(&held, held.new.as_str(), paths.as_slice()).as_str(), seq, request.git_dir.as_path()),
        },
        Settled::Cannot { commit, because } => WithoutTheService::NotMade {
            commit: Some(commit),
            words: kept_words(crate::held_commit_words::cannot(&held, held.new.as_str(), because.as_str()).as_str(), seq, request.git_dir.as_path()),
        },
    };
}

/// The words for a commit that could not be recorded at all,
/// so nothing is kept and the person's own state is what remains.
fn without_a_record(because: &str) -> WithoutTheService {
    return WithoutTheService::NotMade {
        commit: None,
        words: format!("this commit was not made: the commit service gave no answer, and the batch this commit would be recorded in could not be written ({because}). The index and the worktree are as they were, and the commit can be made again by running the same command."),
    };
}

/// What:
///  The parents the commit of an amend must have:
///  the replaced commit's own.
/// Why:
///  An amend replaces the parents of the commit it replaces,
///  and the landing checks that the commit it is given has them,
///  so they are read here from the object store as the landing reads them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parents = request.amend ? revListParents(request.base) : []
/// ```
fn replaced_parents(git: &GitContext, request: &CommitRequest) -> Result<Vec<String>, FailureCode> {
    if !request.amend || is_zero_object(request.base.as_str()) {
        return Ok(Vec::new());
    }
    let listing: String = checked(git, request.repository.as_path(), &["rev-list", "--parents", "--max-count=1", request.base.as_str()], None)?;
    return Ok(listing.split_whitespace().skip(1).map(str::to_string).collect());
}

/// The first number this process asks the journal for,
/// the reading the service's own count starts from:
/// microseconds since the epoch,
/// so a client and a service that count from different instants do not meet,
/// and the number itself is still claimed in the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = Date.now() * 1000
/// ```
fn a_first_batch_number() -> u64 {
    return match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => u64::try_from(elapsed.as_micros()).unwrap_or(1),
        Err(_before_the_epoch) => 1,
    };
}

/// The words for a commit the repository itself refused,
/// with what the request was,
/// so the person reads the same sentence the service's own refusal would have given.
fn not_made_by_the_repository(code: FailureCode, request: &CommitRequest, commit: Option<&str>) -> WithoutTheService {
    let failure: ClientFailure = failure_for(code, request);
    return WithoutTheService::NotMade { commit: commit.map(str::to_string), words: render(&failure) }.with_commit(commit.map(str::to_string));
}

/// What:
///  The words of a commit that is written and kept,
///  with where it is kept,
///  so the person knows nothing they wrote was discarded.
/// Why:
///   The commit exists as an object and the batch's record names it,
///  and the record says what stands in the way,
///  which is the two things a person needs to know at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptWords(said, seq, gitDir)
/// ```
fn kept_words(said: &str, seq: u64, git_dir: &Path) -> String {
    return format!(
        "{said} Nothing you staged or wrote was discarded: the commit is written and kept on {}, and the record of the batch at {} says what it waits for, so the next command of this worktree settles it.",
        kept_on(seq),
        record_path(git_dir, seq).display()
    );
}

impl WithoutTheService {
    /// The same outcome with the commit it is about,
    /// for a report that has to name it.
    fn with_commit(self, commit: Option<String>) -> Self {
        return match self {
            Self::NotMade { words, commit: known } => Self::NotMade { commit: known.or(commit), words },
            other => other,
        };
    }

    /// The same outcome with the reason the record could not be written,
    /// which is said beside the sentence a failure code gives.
    fn with_words(self, because: &str) -> Self {
        return match self {
            Self::NotMade { commit, words } => Self::NotMade { commit, words: format!("{words} ({because})") },
            other => other,
        };
    }
}

/// The reference a batch's commit is kept on,
/// for the sentence that names it.
pub(crate) fn kept_on(seq: u64) -> String {
    return scratch_ref(seq);
}

/// The path of a worktree's held record of a batch,
/// for the sentence that names it.
pub(crate) fn record_path(git_dir: &Path, seq: u64) -> PathBuf {
    return crate::held_commit_record::path_of(git_dir, seq);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "service_land_after_the_service_stopped_tests.rs"]
mod tests;
