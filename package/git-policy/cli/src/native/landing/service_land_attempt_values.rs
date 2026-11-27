//! What:
//!  The values one attempt of a landing carries from step to step,
//!  and the checks that a batch may land at all.
//! Why:
//!  The steps of a landing stand in files of their own
//!  (`service_land_attempt_reads.rs` and the six beside it),
//!  so the values they hand on between them stand in one record here
//!  rather than in eight loose bindings of the body,
//!  and the body holds only the order the steps run in.
//!  A batch that names two worktrees or two branches,
//!  or that holds a request which must land alone beside another,
//!  is refused here,
//! before any step reads or writes anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const the = TheAttempt.of({ context, seq, requests, announce, timing, settings });
//! ```

use crate::service_index_lock::Announcer;
use crate::service_land_context::LandContext;
use crate::service_land_outcomes::Outcome;
use crate::service_land_settings::{
    LANDING_ATTEMPTS,
    Settings,
};
use crate::service_protocol::{
    CommitRequest,
    FailureCode,
    is_zero_object,
};
use crate::service_scratch_ref_of_a_dropped_batch::scratch_ref;
use crate::service_timing::LandTiming;
use crate::transaction_git::GitContext;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

/// What one attempt of a landing works on,
/// and the values its steps hand on to each other.
pub(crate) struct TheAttempt<'a> {
    /// The journal,
    /// the Git and the lock ceiling of the landing.
    pub(crate) context: &'a LandContext,
    /// The landing's clock,
    /// which states what each stretch of the work cost.
    pub(crate) timing: &'a mut LandTiming,
    /// The batch number.
    pub(crate) seq: u64,
    /// Every request the batch was handed,
    /// in the order the client sent them.
    pub(crate) requests: &'a [CommitRequest],
    /// What the landing tells the client that waits,
    /// and how it asks one whether its client is gone.
    pub(crate) announce: &'a dyn Announcer,
    /// What the landing read once before its first attempt.
    pub(crate) settings: &'a Settings<'a>,
    /// The first request,
    /// which names the worktree,
    /// the Git directory,
    /// the branch and the index the batch is made for.
    pub(crate) first: &'a CommitRequest,
    /// The work tree the batch's Git reads.
    pub(crate) repository: &'a Path,
    /// How to run Git for this batch:
    /// the context's own,
    /// told the common directory this batch writes its references in.
    pub(crate) git: GitContext,
    /// The Git directories whose reference locks may refuse this batch's branch write:
    /// the common one,
    /// and a linked worktree's own.
    pub(crate) lock_roots: Vec<PathBuf>,
    /// The scratch reference the batch's commits are held on until the branch moves.
    pub(crate) scratch: String,
    /// Whether the branch does not exist yet,
    /// which is prepared against the all-zero name.
    pub(crate) unborn: bool,
    /// Whether the batch holds a request that lands alone,
    /// which is only valid while the branch is still at its base.
    pub(crate) alone: bool,
    /// The commit the branch named when this attempt read it.
    pub(crate) old: String,
    /// The parents of the commit an amend replaces,
    /// which no other batch has.
    pub(crate) amended_parents: Vec<String>,
    /// The places that still want their commit,
    /// each with its place in `requests`.
    pub(crate) landing: Vec<(usize, &'a CommitRequest)>,
    /// The requests of those places,
    /// in the order their commits are made.
    pub(crate) batch: Vec<&'a CommitRequest>,
    /// The answer the landing is building,
    /// one entry per request.
    pub(crate) outcomes: Vec<Outcome>,
    /// The places whose client went away before its commit landed,
    /// which nothing reads an answer for and which no further attempt builds.
    pub(crate) gone: Vec<usize>,
    /// How many times the batch may still be built again.
    pub(crate) attempts_left: usize,
    /// When the branch write was first refused while the branch stood still,
    /// which every wait for a reference lock shares.
    pub(crate) lock_deadline: Option<Instant>,
}

impl<'a> TheAttempt<'a> {
    /// What:
    ///  The values one attempt starts with,
    ///  once the batch is proven to be one worktree's one branch.
    /// Why:
    ///  Every step below reads these,
    ///  and a batch that names two worktrees would write in both.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function of({ context, seq, requests, announce, timing, settings }): TheAttempt
    /// ```
    ///
    /// # Errors
    ///
    /// `Rejected` when the batch is empty,
    /// when its requests name more than one worktree,
    /// Git directory or branch,
    /// or when a request that lands alone comes beside another.
    pub(crate) fn of(
        context: &'a LandContext,
        seq: u64,
        requests: &'a [CommitRequest],
        announce: &'a dyn Announcer,
        timing: &'a mut LandTiming,
        settings: &'a Settings<'a>,
    ) -> Result<TheAttempt<'a>, FailureCode> {
        let first: &CommitRequest = requests.first().ok_or(FailureCode::Rejected)?;
        for request in requests {
            if request.repository != first.repository
                || request.common_dir != first.common_dir
                || request.reference != first.reference
            {
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
        let git: GitContext = GitContext {
            common_dir: Some(first.common_dir.clone()),
            ..context.git.clone()
        };
        // The Git directories whose reference locks may refuse this batch's branch write: the common one holds the references
        //  and the packed references, and a linked worktree keeps its own `HEAD.lock` in its own Git directory.
        let mut lock_roots: Vec<PathBuf> = vec![first.common_dir.clone()];
        if first.git_dir != first.common_dir {
            lock_roots.push(first.git_dir.clone());
        }
        let scratch: String = scratch_ref(seq);
        // The places of the batch whose client went away before its commit landed: nothing reads an answer written for them, so
        //  their commits are not made and the rest of the batch is built again without them.
        let gone: Vec<usize> = Vec::new();
        // When the branch write was first refused while the branch stood still: every wait for a reference lock that refused
        //  it shares this one bound, so a lock that keeps being taken again cannot hold the landing past it.
        let lock_deadline: Option<Instant> = None;
        // A branch that does not exist yet is prepared against the all-zero name and lands only while it still does not.
        let unborn: bool = is_zero_object(first.base.as_str());
        return Ok(TheAttempt {
            context,
            timing,
            seq,
            requests,
            announce,
            settings,
            first,
            repository,
            git,
            lock_roots,
            scratch,
            unborn,
            alone,
            old: String::new(),
            amended_parents: Vec::new(),
            landing: Vec::new(),
            batch: Vec::new(),
            outcomes: Vec::new(),
            gone,
            attempts_left: LANDING_ATTEMPTS,
            lock_deadline,
        });
    }
}
