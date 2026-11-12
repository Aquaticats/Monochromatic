//! What:
//!  What the work of the commit service is made of:
//! the kinds it runs,
//! the handlers that run each kind,
//! which records are
//!  acceptable work,
//! how claims are settled,
//! what a drain returned and how a push that failed is planned again.
//! Why:
//!  The drain of pending claims (`service_work_drain_pending_claims.rs`),
//! the drains of directories (`service_work.rs`) and the
//!  rounds (`service_work_run_round.rs`) all speak in these;
//! they sit in a file that uses none of those.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface WorkHandlers { review(...); push(...); maintenance(...); trash(...) }; function isAcceptable(item, gitDir): boolean
//! ```

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::journald_native::Priority;
use crate::push_failure_notice::record_push_skip;
use crate::push_retry::{Due, RetryChange, Retrying, pause_before};
use crate::push_retry_cause_line::cause_line;
use crate::push_retry_failure::{PROTECTED_BRANCH_TRIES, protected_branch_refusal};
use crate::service_instance::GitDirUnreachable;
use crate::unreachable_git_dir_proven_gone::Gone;
use crate::service_claim_renewal::HeldClaims;
use crate::service_review_key::ReviewKey;
use crate::service_timing::log_entry;
use crate::work_item::{Claimed, WORK_DIRECTORY, WorkItem, WorkKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
/// The kinds of work the service runs itself:
/// the push and every slow kind.
pub const SERVICE_KINDS: [WorkKind; 8] = [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review, WorkKind::Trash, WorkKind::Prewarm, WorkKind::Relink, WorkKind::SubvolumeDelete, WorkKind::SnapshotAhead];

/// The kinds the round that answers a wake claims:
/// the push,
/// which is the person's backup.
/// Why:
///   A round holds the service until it ends,
/// so nothing else may be claimed beside a push:
/// on `2026-11-10` six landings of `refs/heads/main` waited 18 minutes for one round that was
/// emptying trashes of removed worktrees,
/// and every round that held a push that day was measured the same way
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// The other kinds are the slow lane's (`landing/service_work_slow_lane.rs`).
pub const PUSH_KINDS: [WorkKind; 1] = [WorkKind::Push];

/// The kinds whose steps can take minutes:
/// the upkeep,
/// the prewarm of a new worktree's build directories,
/// the relink of a new worktree's copied package state,
/// the delete of a
/// removed worktree's subvolume,
/// the emptying of a trash,
/// the snapshot made ahead of an add,
/// and the shadow review.
/// Why:
///   No command waits for these steps to end:
/// a person's add that wrote a snapshot ahead looks for one and makes its own when none is ready in time
/// (`worktree/worktree_snapshot_ahead.rs`),
/// and the whole of `2026-11-10` shows that a step of these kinds that ran for minutes held every push of
/// `refs/heads/main` of the round it ran in
/// (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
/// They are claimed by the slow lane (`landing/service_work_slow_lane.rs`),
/// one at a time,
/// and never by the round.
pub const SLOW_KINDS: [WorkKind; 7] = [
    WorkKind::Maintenance,
    WorkKind::Prewarm,
    WorkKind::Relink,
    WorkKind::SubvolumeDelete,
    WorkKind::Trash,
    WorkKind::SnapshotAhead,
    WorkKind::Review,
];

/// How many times an item is tried before it is given up on.
pub const MAX_TRIES: u32 = 3;

/// What a wake message brings:
/// the repository to work on and the review credential that came with it,
/// if any.
pub type Wake = (PathBuf, Option<ReviewKey>);

/// What a handler came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The work was done,
    /// or there was nothing to do.
    Done,
    /// The work failed;
    /// the text is what the person should be told.
    Failed(String),
    /// Nothing was done and nothing is left to retry,
    /// but the person should be told why;
    /// the text is what they are told.
    Skipped(String),
    /// The push failed for a reason that can pass (a stall,
    /// the network,
    /// an LFS server error,
    /// a held lock,
    /// a remote that advanced again,
    /// the server's protected-branch refusal);
    /// the text is what the person should be told,
    /// and the service tries the push again on its own,
    /// with growing pauses,
    /// for a fixed number of tries when the cause is the
    ///  server's protected-branch refusal.
    Retry(String),
}

/// What runs each kind of work.
pub trait WorkHandlers {
    /// Push the branch of these items,
    /// which share a repository and a branch;
    /// the last is the newest.
    fn push(&self, items: &[WorkItem]) -> Verdict;
    /// The same push,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The round's items are tasks,
    /// and a handler that starts Git cannot be called from one;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pushAwaiting(items: WorkItem[]): Promise<Verdict>
    /// ```
    fn push_awaiting<'a>(&'a self, items: &'a [WorkItem]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.push(items)));
    }
    /// Run the repository's automatic upkeep.
    fn maintenance(&self, item: &WorkItem) -> Verdict;
    /// The same upkeep,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The upkeep runs Git,
    /// and a task may not start one from a synchronous face;
    /// a handler that does no upkeep keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// maintenanceAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn maintenance_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.maintenance(item)));
    }
    /// Review one landed commit into the repository's shadow log.
    fn review(&self, item: &WorkItem) -> Verdict;
    /// The same review,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The round's items are tasks,
    /// and a review's wait for its contained job is the runtime's there;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reviewAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn review_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.review(item)));
    }
    /// Empty the directory of left-out files a snapshot-made worktree set aside;
    /// a handler that does not know it skips.
    fn trash(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not empty trash directories"));
    }
    /// The same emptying,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and the synchronous disposal enters the runtime for its entry
    /// removals,
    /// which is refused from inside it;
    /// a handler that knows nothing of the runtime keeps the synchronous one
    /// as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trashAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn trash_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.trash(item)));
    }
    /// Fill a new worktree's Cargo build directory from the build state of the checkout it was copied from;
    /// a handler that does not know it skips.
    fn prewarm(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not fill Cargo build directories"));
    }
    /// The same filling,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and the filling starts Cargo children and copies a tree,
    /// which the synchronous face would refuse from inside the runtime;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prewarmAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn prewarm_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.prewarm(item)));
    }
    /// Relink the copied package state of a new worktree with each tool's own install;
    /// a handler that does not know it skips.
    fn relink(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not relink the package state of a worktree"));
    }
    /// The same relinking,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The drain of a round is a task,
    /// and each tool's install is a child process the synchronous face would refuse to start from inside the runtime;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// relinkAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn relink_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.relink(item)));
    }
    /// Delete the subvolume of a removed worktree that the removal handed over;
    /// a handler that does not know it skips.
    fn subvolume_delete(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not delete subvolumes"));
    }
    /// The same delete,
    /// awaited by a caller that is itself a task of the runtime,
    /// which is how the round asks every kind.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// subvolumeDeleteAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn subvolume_delete_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.subvolume_delete(item)));
    }
    /// Make a snapshot of a worktree ahead of the next add;
    /// a handler that does not know it skips.
    fn snapshot_ahead(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler makes no snapshots ahead"));
    }
    /// The same snapshot,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The ioctl waits for a btrfs transaction commit,
    /// so it belongs on the blocking pool and never on a worker the
    ///  runtime needs for the commits queued behind it;
    /// a handler that knows nothing of the runtime keeps the synchronous one as its
    ///  answer.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshotAheadAwaiting(item: WorkItem): Promise<Verdict>
    /// ```
    fn snapshot_ahead_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(std::future::ready(self.snapshot_ahead(item)));
    }
}

/// The work directory of a repository.
pub fn work_directory(git_dir: &Path) -> PathBuf {
    return path_for_reading(git_dir, WORK_DIRECTORY);
}

/// What:
///  Whether an item is one this service should act on for this repository.
/// Why:
///   Anyone who can write the work directory can write an item,
/// so the fields that become a directory,
/// a reference or a
///  program argument are checked before they are used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAcceptable(item, gitDir)
/// ```
pub fn is_acceptable(item: &WorkItem, git_dir: &Path) -> bool {
    return item.repository.is_absolute()
        && item.repository.is_dir()
        && item.git_dir.as_path() == git_dir
        && item.branch_ref.starts_with("refs/")
        && !item.branch_ref.contains(['\0', '\n', ' '])
        && item.oid.len() >= 40
        && item.oid.as_bytes().iter().all(u8::is_ascii_hexdigit);
}

/// Moves the claims to `state` and stops renewing them;
/// a claim that another instance took over is said,
/// and left to it.
pub(crate) fn settle_all(work_dir: &Path, held: &HeldClaims, claimed: &[Claimed], state: &str) {
    for item in claimed {
        if let Err(reason) = held.settle(work_dir, item.id.as_str(), state) {
            crate::service_warn!("{reason}");
        }
    }
}

/// What:
///  The claims of `claimed` that this instance still holds,
/// before their work starts.
/// Why:
///   A claim whose renewal found it gone was taken over:
/// another instance runs that item now,
/// so running it here as well would
///  do it twice.
/// The claim is forgotten and the loss said,
/// and nothing else is done for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimed.filter((c) => held.isHeld(c.id))
/// ```
pub(crate) fn still_held(held: &HeldClaims, work_dir: &Path, claimed: Vec<Claimed>) -> Vec<Claimed> {
    let mut mine: Vec<Claimed> = Vec::new();
    let mut lost: Vec<Claimed> = Vec::new();
    for item in claimed {
        if held.is_held_on_disk(work_dir, item.id.as_str()) {
            mine.push(item);
        } else {
            lost.push(item);
        }
    }
    for item in lost {
        crate::service_warn!("the claim on {} ran out before this instance could run it, so another instance took the work over and it is not run here",
            item.id
        );
        held.release(item.id.as_str());
    }
    return mine;
}

/// What:
///  Leaves a notice for each push the service gave up on without ever finishing it:
/// its holder was stopped,
/// or lost,
/// on every
///  try,
/// and answers whether every notice is where a command will show it.
/// Why:
///   Such an item moves to `failed` by recovery,
/// not by a handler,
/// so nothing else records why the commit was never backed up
///  and the person would not be told.
/// The notice is the one a skipped push leaves,
/// because nothing failed and nothing was sent.
///  A notice that could not be written answers that it is not there,
/// so the push is not settled in silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const item of givenUp.filter(isPush)) if (!(await recordPushSkip(gitDir, item, reason))) return false; return true;
/// ```
pub(crate) fn tell_about_given_up(git_dir: &Path, given_up: &[WorkItem]) -> bool {
    let mut told: bool = true;
    for item in given_up {
        if item.kind != WorkKind::Push {
            continue;
        }
        let reason: String = format!(
            "monochromatic-git-policy-cli: the commit service stopped {MAX_TRIES} times before it finished pushing {} (a restart, a kill or a crash each time), so it gave up on that push.",
            item.branch_ref
        );
        if let Err(problem) = record_push_skip(git_dir, item.branch_ref.as_str(), item.oid.as_str(), reason.as_str()) {
            crate::service_error!(
                "the abandoned push of {} {} could not be recorded for the person: {problem}",
                item.oid,
                crate::service_timing::in_the_repository_at(item.repository.as_path())
            );
            told = false;
        }
    }
    return told;
}

/// What:
///  The notice a recovery leaves for a push whose holder died on every try:
/// the skipped-push record of the branch,
/// written into the directory the recovery works on.
/// Why:
///   The record moves to `failed` right after the notice,
/// and nothing scans `failed`,
/// so the notice is the person's only remaining word about that branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SkipNotice { tell(item) { recordPushSkip(this.gitDir, item) } }
/// ```
pub(crate) struct SkipNotice<'a>(pub(crate) &'a Path);

impl crate::work_item_recovery::GivenUpNotice for SkipNotice<'_> {
    fn tell(&self, given_up: &WorkItem) -> bool {
        return tell_about_given_up(self.0, std::slice::from_ref(given_up));
    }
}

/// What:
///  What one drain of a directory hands back:
/// how long until it must be looked at again for a claim another instance holds,
/// and
///  the changes it asks of the retry plan.
/// Why:
///   A drain runs on a worker and cannot change the plan,
/// which the service loop owns;
/// it returns what it did and the loop
///  applies it (`PushRetries::apply`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drained = { wait?: number; changes: RetryChange[] };
/// ```
#[derive(Debug, Default)]
pub struct Drained {
    /// How long until the caller must look at this directory again:
    /// the pause a drain that left housekeeping for a later look asked for, or the end of the earliest claim it keeps,
    /// or nothing when neither is left.
    pub wait: Option<Duration>,
    /// The plans and the forgets the drain asks of the retry plan,
    /// in the order they were made.
    pub changes: Vec<RetryChange>,
}

/// What:
///  The retry state one drain works with:
/// the first pause (nothing when a retryable push is to be reported as failed) and the
///  changes it has asked of the plan so far.
/// Why:
///   One value is passed down to the places that plan or forget a retry,
/// so none of them reads the plan or shares it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Replanning = { first?: number; changes: RetryChange[] };
/// ```
pub(crate) struct Replanning {
    /// The pause before the first new try,
    /// or nothing for a drain that keeps no plan.
    pub(crate) first: Option<Duration>,
    /// What was asked of the plan.
    pub(crate) changes: Vec<RetryChange>,
    /// The directory the plan's files are kept in,
    /// which is where this drain writes the plan it promises and removes the one it forgets,
    /// or nothing for a plan kept in memory only.
    pub(crate) plans: Option<PathBuf>,
}

/// Methods over the state.
impl Replanning {
    /// What:
    ///  Ask the plan to drop the branch,
    /// and take its file out of the journal now.
    /// Why:
    ///   The file is what a restarted service rebuilds the plan from,
    /// so a forget that reached only the list of this process would leave the next service trying a push that has already landed
    ///  (finding 3 of the review of `bc3d54f9cd`).
    /// It goes here,
    /// on the thread that decided the branch needs no new try,
    /// as the plan a worker promises is already written here,
    ///  and not by a round that may reach the journal later,
    /// when a newer plan of the branch may have been written since
    ///  (finding 4 of the same review).
    /// A removal that fails is said,
    /// and the next start reads the file again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forget(gitDir, branch): removePlans(this.plans, gitDir, branch); this.changes.push({ forget: { gitDir, branch } });
    /// ```
    pub(crate) fn forget(&mut self, git_dir: &Path, branch_ref: &str) {
        self.take_the_plan_file(git_dir, branch_ref);
        self.push_forget(git_dir, branch_ref, false, None);
    }

    /// What:
    ///  Ask the plan to drop the branch of a push whose repository a reading of the filesystem proved gone,
    /// and take its file out of the journal unless the repository is back.
    /// Why:
    ///   That reading is a point in time,
    /// and this runs after it:
    /// a repository renamed back by then is there again,
    ///  so its plan must not be dropped on the stale proof.
    /// The file goes where every forget's file goes,
    /// on the thread that decided the branch needs no new try,
    /// so this is the last place the filesystem can be asked before the file is gone;
    /// the applier asks it once more before it drops the branch from the list
    /// (`push_retry.rs`,
    /// `PushRetries::apply`),
    ///  which is what keeps a repository that came back during the round trying,
    /// and the plan this reading ended travels with the ask,
    /// so the repository that is back keeps the try the round took out of the plan
    ///  rather than only its on-disk file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forgetWhenUnreachable(gitDir, due):
    ///   if (!isDirectory(gitDir)) removePlans(this.plans, gitDir, due.item.branchRef);
    ///   this.changes.push({ forget: { gitDir, branchRef: due.item.branchRef, onlyWhileUnreachable: true, taken: due } });
    /// ```
    pub(crate) fn forget_when_unreachable(&mut self, git_dir: &Path, due: &Due) {
        if !git_dir.is_dir() {
            self.take_the_plan_file(git_dir, due.item.branch_ref.as_str());
        }
        self.push_forget(git_dir, due.item.branch_ref.as_str(), true, Some(due.clone()));
    }

    /// Removes the plan's file of a branch from the journal,
    /// where a directory keeps one at all.
    fn take_the_plan_file(&mut self, git_dir: &Path, branch_ref: &str) {
        if let Some(directory) = self.plans.as_deref() {
            crate::push_retry_store::remove(directory, git_dir, branch_ref);
        }
    }

    /// The one place a forget is added,
    /// so both kinds name the directory and the branch the same way.
    fn push_forget(&mut self, git_dir: &Path, branch_ref: &str, only_while_unreachable: bool, taken: Option<Due>) {
        self.changes.push(RetryChange::Forget { git_dir: git_dir.to_path_buf(), branch_ref: branch_ref.to_string(), only_while_unreachable, taken });
    }

    /// What:
    ///  Keep the plan of the try `attempt` of the branch of `item` on disk now,
    /// before any words that promise it reach the person,
    /// and answer whether it
    ///  is kept.
    /// Why:
    ///   The loop applies the changes a worker asks only once every repository of the round is drained,
    /// so a kill before it applies them would take the plan,
    /// and a notice that says the service keeps trying would be the only statement left and it would be false.
    /// A worker writes only its own branch's plan,
    /// whose file name the branch key and the try number make,
    /// and the loop writes no file:
    /// what it applies is the list it reads.
    ///  A plan that cannot be written answers that it is not kept,
    /// so the caller promises nothing the next service cannot keep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return savePlans(this.plans, { gitDir, item, attempt });
    /// ```
    pub(crate) fn keep_the_plan(&self, git_dir: &Path, item: &WorkItem, attempt: u32) -> bool {
        let Some(directory) = self.plans.as_deref() else {
            // A plan kept in memory only is kept as long as the service that keeps it lives.
            return true;
        };
        return match crate::push_retry_store::save(directory, &crate::push_retry_store::StoredPlan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt }) {
            Ok(()) => true,
            Err(reason) => {
                crate::service_warn!(
                    "the plan to push {:?} again {} could not be kept for a restart: {reason}",
                    item.branch_ref,
                    crate::service_timing::in_the_repository_at(item.repository.as_path())
                );
                false
            }
        };
    }
}

/// What:
///  Plan the next try of a push that failed for a reason that can pass,
/// say so in the journal,
/// and give the words the person's
///  failure record carries.
/// Why:
///   The journal line names the cause,
/// which is the first of Git's own lines that gives a reason rather than
///  the remote it pushed to (`cause_line`),
/// the number of the try that is planned and the pause before it,
/// so the service's quiet
///  between tries is explained;
/// the person's record says the service keeps trying,
/// so they do not push by hand for nothing,
/// and for a
///  cause the service gives a fixed number of tries it says how many there are.
///  The plan is asked of the service loop (`Replanning`),
/// which owns it,
/// and it is written before those words exist:
/// the promise a record makes
///  is the plan's,
/// so a plan that cannot be kept leaves the record with Git's own words alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = plannedRetry(gitDir, replanning, first, item, attempt, text);
/// ```
/// What:
///  The words a record of a planned retry carries,
/// and whether the plan they promise is kept.
/// Why:
///   The promise and the plan are one fact:
/// main's words say the service keeps trying only when the plan could be kept,
///  and the notice's framing needs the same answer as a value rather than as the presence of a sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlannedRetry = { words: string; kept: boolean };
/// ```
pub(crate) struct PlannedRetry {
    /// The words for the person's record,
    /// with the promise only when the plan is kept.
    pub(crate) words: String,
    /// Whether the plan the words promise exists.
    pub(crate) kept: bool,
}

pub(crate) fn planned_retry(git_dir: &Path, replanning: &mut Replanning, first: Duration, item: &WorkItem, attempt: u32, text: &str) -> PlannedRetry {
    let pause: Duration = pause_before(attempt, first);
    // The notice below says the service keeps trying, so the plan that keeps it is written first,
    // and its answer says whether the words may promise it.
    let kept: bool = replanning.keep_the_plan(git_dir, item, attempt);
    replanning.changes.push(RetryChange::Plan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due: Instant::now() + pause });
    // A protected-branch refusal is retried a fixed number of times, so a record that promised tries until the push lands would be
    //  untrue for the one cause that stops on its own; its words name the last try instead.
    let plan: String = match protected_branch_refusal(text) {
        Some(_) => format!(
            "monochromatic-git-policy-cli: the commit service tries this push again by itself in {} seconds (try {} of {}). The server's refusal of an update to a protected branch can clear by itself; if the server refuses try {} as well, the service stops trying this branch and tells you what the server said.",
            pause.as_secs(),
            attempt + 1,
            PROTECTED_BRANCH_TRIES,
            PROTECTED_BRANCH_TRIES
        ),
        None => format!(
            "monochromatic-git-policy-cli: the commit service tries this push again by itself in {} seconds (try {}), and keeps trying with longer pauses until the push lands or a conflict needs you.",
            pause.as_secs(),
            attempt + 1
        ),
    };
    let fields: [(&str, String); 4] = [
        ("CLI_GIT_BRANCH", item.branch_ref.clone()),
        ("CLI_GIT_OID", item.oid.clone()),
        ("CLI_GIT_OUTCOME", String::from("retrying")),
        ("CLI_GIT_REPOSITORY", item.repository.to_string_lossy().into_owned()),
    ];
    // The line names the repository: one journal covers every worktree of the machine, and a branch name is the same in all of
    //  them, so a retry line without it cannot say which worktree's push it plans.
    log_entry(
        Priority::Warning,
        format!(
            "push will be tried again branch={:?} oid={} repository={:?} next_try={} in_seconds={} cause: {}",
            item.branch_ref,
            item.oid,
            item.repository,
            attempt + 1,
            pause.as_secs(),
            cause_line(text).unwrap_or_default()
        )
        .as_str(),
        &fields,
    );
    // A plan that could not be kept is not promised: the service of this run still tries the push, and a service started in
    //  its place would not know it, so the person is left with Git's own words and pushing by hand.
    let words: String = if kept { format!("{plan}\n{text}") } else { text.to_string() };
    return PlannedRetry { words, kept };
}

/// What:
///  What a drain does for a Git directory the filesystem proves is gone:
/// its push plans end,
/// and one journal line says which path
///  lost them and what the proof looked at.
/// Why:
///   A repository that was deleted cannot be pushed again,
/// and its work records went with it,
/// so a plan kept for it
///  is a try that can never run:
/// on 2026-11-10 two deleted throwaway repositories were retried every 600 s for hours,
/// at
///  try 48 and climbing
/// (`doc/planning/cli-git-unreachable-repository-ends-its-plans.md`).
/// The person's own notice lives inside the Git
///  directory,
/// which is gone,
/// so the journal is the only place that can say what became of the backup,
/// and the line names the
///  branch and the commit with the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = prove(gitDir, mounts); if (gone) { log(goneLine(gitDir, gone, branches)); for (const due of dueOf(gitDir)) forget(due); }
/// ```
pub(crate) async fn end_the_plans_of_a_gone_repository_awaiting(git_dir: &Path, retrying: &Retrying, unreachable: &GitDirUnreachable, gone: &Gone) -> Drained {
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    let mut named: Vec<String> = Vec::new();
    for due in &retrying.due {
        if due.git_dir != git_dir {
            continue;
        }
        named.push(format!("{} of {}", due.item.branch_ref, due.item.oid));
        replanning.forget_when_unreachable(git_dir, due);
    }
    // Nothing of this drain ran, so the caller has nothing to wait for; the file of each ended plan goes here while its
    //  repository is still missing, and the change drops the branch from the list the loop reads. A repository that is
    //  back by the time the change lands keeps its file, and the change puts its plan back (`PushRetries::apply`).
    if !named.is_empty() {
        crate::service_warn!("{} ({unreachable})", crate::unreachable_git_dir_proven_gone::gone_line(git_dir, gone, named.as_slice()));
    }
    return Drained { wait: None, changes: replanning.changes };
}

/// What:
///  What a drain does for a Git directory that cannot be reached (a link to a drive that is not mounted):
/// nothing runs,
/// its work stays where it is for the next command,
/// and every push that was due keeps its plan with the next pause,
/// doubled the way a failed try doubles it.
/// Why:
///  The push is not given up on and never runs on another path;
/// the plan is asked again for the pause that follows,
/// so the push is tried when the directory is back.
/// Without a first pause (a drain that keeps no plan) nothing is planned.
/// The person's failure record is inside the directory and is not touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const due of retrying.due) if (due.gitDir === gitDir) plan(gitDir, due.item, due.attempt + 1, pauseBefore(due.attempt + 1))
/// ```
pub(crate) fn keep_plans_while_unreachable(git_dir: &Path, retrying: &Retrying, unreachable: &GitDirUnreachable) -> Drained {
    crate::service_warn!("{unreachable}; its work stays where it is for the next command, nothing runs, and its push plans keep their pauses");
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    let Some(first) = retrying.first else {
        return Drained { wait: None, changes: replanning.changes };
    };
    for due in &retrying.due {
        if due.git_dir == git_dir {
            let cause: String = format!("the repository at {} is not reachable now (is its drive mounted?)", git_dir.display());
            let _said: PlannedRetry = planned_retry(git_dir, &mut replanning, first, &due.item, due.attempt + 1, cause.as_str());
        }
    }
    return Drained { wait: None, changes: replanning.changes };
}
