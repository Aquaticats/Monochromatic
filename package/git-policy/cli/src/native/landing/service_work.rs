//! What:
//!  The service's consumer of after-commit work:
//! pending `push` and `maintenance` work items of one repository are
//!  claimed,
//! coalesced,
//! run,
//! settled,
//! and a push that failed leaves a record that the person's next wrapper command shows.
//! Why:
//!  A commit must not wait for the network or for upkeep,
//! so the wrapper leaves a record and the resident service does
//!  the work on its own thread (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! The service is the
//!  only pusher of the repositories it serves,
//! so no lock is needed:
//! several pending pushes of one branch become one
//!  push,
//! and a service that died mid-push is noticed by the lease of its claims running out,
//! never by a process id:
//! the
//!  holder renews the lease while it works,
//! so a successor that starts beside a holder that is still running leaves its claims
//!  alone,
//! and one that starts after the holder died takes them back as soon as the lease is over.
//! Other kinds of work
//!  (the standards review) are never claimed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const group of coalesce(await claimAll(workDir, ['push', 'maintenance']))) settle(group, await handler(group));
//! ```

pub use crate::service_work_handlers_replanning_and_acceptance::*;
pub(crate) use crate::service_work_branch_is_gone::branch_is_gone_words_awaiting;
pub use crate::service_work_drains_awaiting::{drain_common_records_planned_awaiting, drain_planned_awaiting, drain_planned_with_awaiting};
use crate::auto_push::push_once_awaiting;
use crate::auto_push_single_flight::PushGit;
use crate::child_environment::{environment_with_overlay, service_child_overlay};
use crate::config_snapshot::read_config_snapshot_awaiting;
/// The synchronous configuration read,
/// which only the control that builds the push's Git without the runtime asks.
#[cfg(test)]
use crate::config_snapshot::read_config_snapshot;
use crate::lfs_pre_push::uses_lfs;
use crate::maintenance_run::{AutoMaintenanceOutcome, run_auto_maintenance_with_awaiting};
use crate::auto_push::AutoPushOutcome;
use crate::claim_time::ClaimTime;
use crate::push_retry::Retrying;
use crate::push_retry_failure::{final_failure, plain_words, unrecognised_rejection};
use crate::push_failure_notice::record_push_report;
use crate::auto_push_reconcile_basics_and_limits::RECONCILE_LINE_PREFIX;
use crate::service_claim_renewal_running::Renewal;
use crate::service_review::{options_for, review_item, review_item_awaiting, review_variables};
use crate::service_review_key::ReviewKey;
use crate::standards_review_run::ReviewOptions;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::work_item_holding::{BlockedVerdict, blocked_verdicts};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How long a claim goes without a renewal before another instance runs its item again;
/// the holder renews it three times in this
/// span while it works (`service_claim_renewal.rs`),
/// so a holder that is slow keeps its claim and one that died loses it
/// after this long.
///  Measured by killing a throwaway service during a claim:
/// the service binary in a fixture repository claimed a push held by the
///  remote's hook,
/// was killed with SIGKILL 1.5 s after the claim (with its process group),
/// and a new service was started and woken;
///  the push was made again 119.57,
/// 119.56 and 119.55 s after the kill in three runs (2026-11-03),
/// so the cost of a kill is the lease
///  to within half a second.
/// No kill of a commit service appears in the user journal of this host (19 boots),
/// so the kills a person
///  meets were not counted.
/// The renewal thread is independent of the landing (`service_claim_renewal.rs`),
/// so a landing that stalled for
///  119.8 s (the longest `total_ms` in the service journal) does not stop renewals;
/// a stall of the whole process longer than the
///  80 s a renewal may be late would.
/// Kept at 120 s as the coordinator's pick under the owner's delegation,
/// open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Picked for these follow-ups",
/// batch size and claim lease,
/// option B).
pub const CLAIM_LEASE: Duration = Duration::from_secs(120);

/// What:
///  Runs everything pending for one repository:
/// takes back the claims whose holder stopped renewing them,
/// claims every pending
///  push,
/// maintenance and review item,
/// runs one push per branch and one upkeep per repository,
/// settles the items,
/// and leaves a
///  notice for each failed or skipped push.
/// Gives how long until a claim another instance still holds runs out its lease,
/// or
///  nothing when there is none.
/// Why:
///   Coalescing keeps a burst of commits to one push;
/// a handler that fails leaves the items in `failed`,
/// not in a loop,
///  and one that answers `Retry` leaves its item claimed,
/// so the lease running out queues it once more and no round spins on it.
/// A
///  claim found in this directory may belong to a live instance (a restarted service starts beside the one it replaces,
/// and the
///  linked worktrees of a bare repository share a directory),
/// so only its lease says it was abandoned;
/// the caller is told when
///  to look again,
/// because nothing else would wake it once the lease is over.
/// This keeps no retry plan:
/// a push that could pass
///  is reported as failed,
/// as for handlers that do not retry.
/// The service's rounds use `drain_planned` instead.
/// It claims every kind the service runs (`SERVICE_KINDS`),
/// because no lane runs beside it as it does beside a round
///  (`landing/service_work_slow_lane.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drain(gitDir, handlers, readClaimTime())
/// ```
pub fn drain(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime) -> Option<Duration> {
    return drain_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain` with the lease given,
/// so a test can use a short one.
pub fn drain_with_lease(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    return drain_with_renewal(git_dir, handlers, now, lease, Renewal::Timed);
}

/// `drain_with_lease` with the way its claims are kept fresh given,
/// so a control can drive the renewal itself.
pub fn drain_with_renewal(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, renewal: Renewal<'_>) -> Option<Duration> {
    return drain_planned_with(git_dir, handlers, now, lease, renewal, &Retrying::none()).wait;
}

/// `drain` for a service round:
/// the pushes due again come with the request,
/// and the changes the drain asks of the retry plan are
/// given back with the wait,
/// for the service loop to apply.
pub fn drain_planned(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, retrying: &Retrying) -> Drained {
    return drain_planned_with(git_dir, handlers, now, CLAIM_LEASE, Renewal::Timed, retrying);
}

/// What:
///  `drain_planned` with the lease and the renewal given,
/// its waits made by entering the runtime once.
/// Why:
///   The body is the awaited one (`service_work_drains_awaiting.rs`),
/// so the two faces cannot drain differently;
/// a caller inside the
///  runtime cannot enter it and is told so,
/// which leaves every record it holds to its lease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(drainPlannedWithAwaiting(gitDir, handlers, now, lease, renewal, retrying))
/// ```
pub fn drain_planned_with(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, renewal: Renewal<'_>, retrying: &Retrying) -> Drained {
    return match crate::lazy_runtime::run_on_runtime(drain_planned_with_awaiting(git_dir, handlers, now, lease, renewal, retrying, &SERVICE_KINDS)) {
        Ok(drained) => drained,
        Err(refusal) => {
            crate::diagnostic_log::debug("drainPlanned", format!("the drain of {} was not run: {refusal}", git_dir.display()).as_str());
            Drained::default()
        }
    };
}

/// What:
///  Runs the records the releases before one service per worktree left in the common directory of a bare repository,
/// from the
///  instance of a linked worktree:
/// nothing for a repository with a main worktree,
/// for a main worktree and for a directory with
///  no `commondir`.
/// Why:
///   Those releases kept every worktree's records in the common directory.
/// A repository with a main worktree has an instance
///  whose own directory that is,
/// and it runs them (the main worktree's current records live there too),
/// so a linked instance
///  must not.
/// A repository whose worktrees are all linked has no such instance,
/// and its pushes would sit there with no notice.
///   Several linked instances may drain the directory at once,
/// so a claim is taken back only after its lease,
/// which the holder
///  renews while it works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drainCommonRecords(linkedGitDir, handlers, now)
/// ```
pub fn drain_common_records(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime) -> Option<Duration> {
    return drain_common_records_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain_common_records` with the lease given,
/// so a test can use a short one.
pub fn drain_common_records_with_lease(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    return drain_common_records_planned(git_dir, handlers, now, lease, &Retrying::none()).wait;
}

/// `drain_common_records` for a service round:
/// `first` is the pause before the first new try of a push that could pass (nothing
/// keeps no plan),
/// and the plans the drain asks for are keyed by the common directory,
/// which the service loop wakes by itself
/// when one is due;
/// that wake drains the common directory as a directory of its own,
/// which has no `commondir`.
/// What:
///  The same drain,
/// its waits made by entering the runtime once.
/// Why:
///   The body is the awaited one (`service_work_drains_awaiting.rs`);
/// a caller inside the runtime cannot enter it and is told so,
/// which leaves
///  the records where they are until their lease ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runOnRuntime(drainCommonRecordsPlannedAwaiting(gitDir, handlers, now, lease, retrying))
/// ```
pub fn drain_common_records_planned(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, retrying: &Retrying) -> Drained {
    return match crate::lazy_runtime::run_on_runtime(drain_common_records_planned_awaiting(git_dir, handlers, now, lease, retrying, &SERVICE_KINDS)) {
        Ok(drained) => drained,
        Err(refusal) => {
            crate::diagnostic_log::debug("drainCommonRecords", format!("the drain of the common records of {} was not run: {refusal}", git_dir.display()).as_str());
            Drained::default()
        }
    };
}

/// The handlers that run real Git.
pub struct GitHandlers {
    /// The context every Git child of the service runs in.
    pub context: GitContext,
    /// The review credential of each repository whose wake message carried one,
    /// for the round these handlers were made for.
    /// Held
    /// in memory only and never changed after the handlers are made:
    /// a round makes its own copy (`for_round`) and drops it when
    /// the round ends,
    /// so the workers of a round only read it and nothing waits for it.
    review_keys: std::collections::HashMap<PathBuf, ReviewKey>,
    /// Where tests point the review's gateway call,
    /// so no test reaches the real endpoint.
    #[cfg(test)]
    review_endpoint: Option<String>,
    /// Variables a test adds to the environment of the push,
    /// such as a short stall window.
    #[cfg(test)]
    push_environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// Methods over the real handlers.
impl GitHandlers {
    /// Handlers that run real Git in `context`,
    /// with no review credential yet.
    pub fn new(context: GitContext) -> GitHandlers {
        return GitHandlers {
            context,
            review_keys: std::collections::HashMap::new(),
            #[cfg(test)]
            review_endpoint: None,
            #[cfg(test)]
            push_environment: Vec::new(),
        };
    }

    /// The same handlers with variables added to the environment of every push;
    /// tests only.
    #[cfg(test)]
    pub fn with_push_environment(mut self, variables: &[(&str, &str)]) -> GitHandlers {
        self.push_environment = variables.iter().map(|(name, value)| { return (std::ffi::OsString::from(name), std::ffi::OsString::from(value)); }).collect();
        return self;
    }

    /// The same handlers with the review's gateway call sent to `endpoint`;
    /// tests only.
    #[cfg(test)]
    pub fn with_review_endpoint(mut self, endpoint: &str) -> GitHandlers {
        self.review_endpoint = Some(endpoint.to_string());
        return self;
    }

    /// A copy of these handlers that holds `keys` as the review credentials of one round;
    /// the copy is dropped with the round.
    /// The test-only settings (the review endpoint,
    /// the push environment) are carried into the copy,
    /// so a round runs the way the
    /// handlers it was made from are set to run.
    pub fn for_round(&self, keys: std::collections::HashMap<PathBuf, ReviewKey>) -> GitHandlers {
        return GitHandlers {
            context: self.context.clone(),
            review_keys: keys,
            #[cfg(test)]
            review_endpoint: self.review_endpoint.clone(),
            #[cfg(test)]
            push_environment: self.push_environment.clone(),
        };
    }

    /// The credential held for this repository,
    /// if the round brought one.
    fn review_key_of(&self, git_dir: &Path) -> Option<ReviewKey> {
        return self.review_keys.get(git_dir).cloned();
    }

    /// The push's Git,
    /// asked without the runtime;
    /// a control builds it this way.
    #[cfg(test)]
    fn push_git(&self, item: &WorkItem) -> PushGit {
        let cwd: PathBuf = item.repository.clone();
        let (inherited, overlay): (Vec<(std::ffi::OsString, std::ffi::OsString)>, Vec<(std::ffi::OsString, std::ffi::OsString)>) =
            self.push_environment_of();
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot(&self.context, cwd.as_path()).as_ref().is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), overlay.as_slice()),
        };
    }

    /// The same Git,
    /// awaited by a caller that is itself a task of the runtime.
    /// Why:
    ///   The configuration read starts Git,
    /// and the synchronous read enters the runtime and is refused from there;
    /// the environment and the directory are the synchronous face's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// async pushGitOnTheRuntime(item): Promise<PushGit>
    /// ```
    async fn push_git_awaiting(&self, item: &WorkItem) -> PushGit {
        let cwd: PathBuf = item.repository.clone();
        let (inherited, overlay): (Vec<(std::ffi::OsString, std::ffi::OsString)>, Vec<(std::ffi::OsString, std::ffi::OsString)>) =
            self.push_environment_of();
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot_awaiting(&self.context, cwd.as_path()).await.as_ref().is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), overlay.as_slice()),
        };
    }

    /// What every Git child of a push receives,
    /// and the overlay the service adds to it.
    /// Why:
    ///   Both faces of the push Git build their environment here,
    /// so the two cannot hand Git different variables.
    ///  Git writes a `~pid.lock` file beside every lock it takes when `core.lockfilePid` is on,
    /// which names the owner of a lock
    ///  that a stopped push leaves behind;
    /// the service's own environment does not turn it on.
    /// The overlay also says never to ask for
    ///  a login (`never_prompt_overlay`):
    /// the host's askpass program is in this environment,
    /// and a dialog nobody answers is a push that waits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const [inherited, overlay] = pushEnvironmentOf();
    /// ```
    fn push_environment_of(&self) -> (Vec<(std::ffi::OsString, std::ffi::OsString)>, Vec<(std::ffi::OsString, std::ffi::OsString)>) {
        #[cfg_attr(not(test), expect(unused_mut, reason = "only a test adds variables"))]
        let mut inherited: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
        #[cfg(test)]
        inherited.extend(self.push_environment.iter().cloned());
        let overlay: Vec<(std::ffi::OsString, std::ffi::OsString)> = service_child_overlay(inherited.as_slice());
        return (inherited, overlay);
    }
}

/// What:
///  The same question,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The question starts Git,
/// and the synchronous query enters the runtime and is refused from there.
/// The arguments and the reading of the exit status are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await branchExistsOnTheRuntime(git, item.branch_ref);
/// ```
async fn branch_exists_awaiting(git: &PushGit, branch_ref: &str) -> Option<bool> {
    let asked = crate::auto_push_single_flight::run_query_awaiting(git, &crate::auto_push_reconcile_basics_and_limits::argv(&["rev-parse", "--verify", "--quiet", branch_ref])).await.ok()?;
    return match asked.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    };
}

/// What:
///  The commit a branch reference names,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The question starts Git,
/// and the synchronous query enters the runtime and is refused from there.
/// A reference that names no commit is the answer "nothing",
/// which is what a branch that was deleted reads as.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tip = await branchTipOnTheRuntime(git, item.branchRef);
/// ```
async fn branch_tip_awaiting(git: &PushGit, branch_ref: &str) -> Option<String> {
    let asked: String = format!("{branch_ref}^{{commit}}");
    let answer = crate::auto_push_single_flight::run_query_awaiting(git, &crate::auto_push_reconcile_basics_and_limits::argv(&["rev-parse", "--verify", "--quiet", asked.as_str()])).await.ok()?;
    if answer.code != Some(0) {
        return None;
    }
    let named: String = String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_owned();
    return (!named.is_empty()).then_some(named);
}

/// What:
///  The lines of a push's own output that only a reconcile writes,
/// joined as they were printed.
/// Why:
///   A successful push prints its `remote:` lines and,
/// when the remote had moved,
/// the reconcile's account of the merge and of what became of the local
///  branch;
/// only the reconcile's lines are the person's explanation,
/// and they already name the tool,
/// the branch and the
///  command that finishes the move.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reconcileLines = (shown) => shown.split('\n').filter((line) => line.startsWith(RECONCILE_LINE_PREFIX)).join('\n');
/// ```
fn reconcile_lines(shown: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in shown.split('\n') {
        if line.starts_with(RECONCILE_LINE_PREFIX) {
            kept.push(line);
        }
    }
    return kept.join("\n");
}

/// Trait implementation for the real Git.
impl WorkHandlers for GitHandlers {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        return match crate::lazy_runtime::run_on_runtime(self.push_awaiting(items)) {
            Ok(verdict) => verdict,
            // The work did not happen, so the item is tried again rather than reported as a push that failed.
            Err(refusal) => Verdict::Retry(format!("the push could not run: {refusal}")),
        };
    }

    fn push_awaiting<'a>(&'a self, items: &'a [WorkItem]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
        let Some(newest) = items.last() else { return Verdict::Done };
        let git: PushGit = self.push_git_awaiting(newest).await;
        // The commit this push may send is fixed before the worktree's records are read. Every commit's record is written before its
        //  branch moves, so every commit of the branch at this reading has a record in the reading, while a commit that lands after
        //  it has none; sending the branch's reference instead would publish that later commit before its own gate answered, and that
        //  gate may keep it local.
        let Some(sent_up_to) = branch_tip_awaiting(&git, newest.branch_ref.as_str()).await else {
            // The branch named no commit at that reading, so nothing of it can be sent: it is either gone, which is said with wherever
            //  the commit still lives, or Git could not be asked, and the push is tried again. Never is the branch's reference sent
            //  instead, because it may name a commit by now.
            return match branch_exists_awaiting(&git, newest.branch_ref.as_str()).await {
                Some(false) => Verdict::Skipped(branch_is_gone_words_awaiting(&git, newest).await),
                _ => Verdict::Retry(format!(
                    "the commit {} was not pushed: {} named no commit in this repository when the push read it, and the push is tried again",
                    newest.oid, newest.branch_ref
                )),
            };
        };
        // The verdicts a post-commit gate kept are the worktree's own records: they are what makes this push stop before a commit the
        //  gate refused, however far the branch has moved since.
        let work: PathBuf = work_directory(newest.git_dir.as_path());
        let blocked: Vec<BlockedVerdict> = blocked_verdicts(work.as_path());
        // A commit whose own gate still runs is not decided either, and a push of the branch sends every commit up to the tip: the
        //  record is what keeps the tip where the gate has not answered yet (`work_item_holding.rs`).
        let undecided: Vec<String> = crate::work_item_holding::undecided_commits(work.as_path(), newest.branch_ref.as_str());
        let mut output: Vec<u8> = Vec::new();
        return match push_once_awaiting(&git, newest.branch_ref.as_str(), sent_up_to.as_str(), blocked.as_slice(), undecided.as_slice(), &mut output).await {
            // A skip sent nothing: with words they are told; with none (no upstream and no origin) the repository has nowhere
            //  to back up to and there is nothing to say, but an earlier failure of the branch is still true.
            Ok(result) if result.outcome == AutoPushOutcome::Skipped => Verdict::Skipped(result.shown),
            // A verdict held the push back: nothing was sent and nothing failed, so the branch is tried again by the retry plan the
            //  service keeps, and the words go to the person's record with the promise that plan keeps.
            Ok(result) if result.outcome == AutoPushOutcome::HeldBack => Verdict::Retry(result.shown),
            Ok(result) if result.exit_code == 0 => {
                // A matching push can carry a sibling the remote refused while this branch arrived: the branch is done, and the
                //  refused sibling is published nowhere, so the journal is the only place a person can learn it.
                for line in crate::matching_push_delivered_the_recorded_branch::refused_lines(result.shown.as_str()) {
                    crate::service_warn!("a branch of this push was refused and was not published: {line}");
                }
                // A push that reconciled a moved remote published a merge and said what became of the local branch in its own
                //  output; the command that ran the push returned long before it ended, so those words go to a record the next
                //  command shows once. A push that simply landed says nothing and leaves no record.
                let report: String = reconcile_lines(result.shown.as_str());
                if !report.is_empty()
                    && let Err(reason) = record_push_report(newest.git_dir.as_path(), newest.branch_ref.as_str(), newest.oid.as_str(), report.as_str())
                {
                    crate::service_error!("what the push of {} did could not be recorded for the person: {reason}", newest.oid);
                }
                Verdict::Done
            }
            // A branch the person deleted is not there to push, now or ever: nothing was sent, so nothing can pass by waiting, and
            //  the commit is named for the person with wherever it still lives. The repository is asked, never the words of the
            //  output: any program a push runs can print a line about a branch of its own.
            Ok(_result) if branch_exists_awaiting(&git, newest.branch_ref.as_str()).await == Some(false) => {
                Verdict::Skipped(branch_is_gone_words_awaiting(&git, newest).await)
            }
            // A conflict, a login the service does not have and a server hook's decline need the person; every other failure can pass
            //  (a stall, the network, an LFS server error, a held lock).
            Ok(result) => match final_failure(result.shown.as_str()) {
                Some(cause) => Verdict::Failed(match plain_words(cause) {
                    Some(words) => format!("{words}\n{}", result.shown),
                    None => result.shown,
                }),
                None => {
                    // A rejection by the server whose reason is not known is tried again, and named so that it can be learned.
                    if let Some(line) = unrecognised_rejection(result.shown.as_str()) {
                        crate::service_warn!("the server rejected the push for a reason this release does not know, so it is tried again: {line}");
                    }
                    Verdict::Retry(result.shown)
                }
            },
            Err(error) => Verdict::Retry(format!("{error}")),
        };
        });
    }

    fn trash(&self, item: &WorkItem) -> Verdict {
        return crate::worktree_trash::empty_trash(item);
    }

    /// What:
    ///  The same emptying,
    /// its entry removals awaited on the runtime.
    /// Why:
    ///   The drain of a round is a task of the runtime,
    /// and the synchronous face enters the runtime for
    /// `remove_entries`,
    /// which is refused there:
    /// the trash would stay and the item would settle as failed.
    /// The record and the reasons are the synchronous face's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// trashAwaiting(item) { return emptyTrashAwaiting(item); }
    /// ```
    fn trash_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move { return crate::worktree_trash::empty_trash_awaiting(item).await; });
    }

    /// What:
    ///  Fill the Cargo build directories a new worktree's record names,
    /// its clone and its clean awaited on the runtime.
    /// Why:
    ///   The add hands this work over instead of waiting for it
    /// (`package_repair_prewarm.rs`),
    /// and the round is a task of the runtime,
    /// so the tree clone and the Cargo child are awaited here rather than entered from a
    ///  synchronous face.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// prewarmAwaiting(item) { return prewarmAwaiting(item); }
    /// ```
    #[cfg(target_os = "linux")]
    fn prewarm_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move { return crate::package_repair_prewarm_job::prewarm_awaiting(item).await; });
    }

    /// What:
    ///  Relink the copied package state a new worktree's record names,
    /// each tool's own install awaited on the runtime.
    /// Why:
    ///   The add hands this work over instead of waiting for it
    /// (`package_repair_relink.rs`),
    /// and the round is a task of the runtime,
    /// so each tool is awaited here rather than started from a synchronous face.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// relinkAwaiting(item) { return relinkAwaiting(item); }
    /// ```
    #[cfg(target_os = "linux")]
    fn relink_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move { return crate::package_repair_relink_job::relink_awaiting(item).await; });
    }

    /// What:
    ///  Delete the subvolume a removal renamed out of the person's way.
    /// Why:
    ///  The removal records the path and returns,
    /// so this is where the one kernel call happens,
    /// with no person waiting for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// subvolumeDelete(item) { return deleteSubvolumeOf(item); }
    /// ```
    fn subvolume_delete(&self, item: &WorkItem) -> Verdict {
        return crate::worktree_remove_hand_over::delete_subvolume_of(item);
    }

    /// What:
    ///  Make one snapshot of a worktree ahead of the next add,
    /// where the caller has no runtime to hand a job to.
    /// Why:
    ///   The call is one btrfs ioctl and starts no child,
    /// so a synchronous caller blocks here itself;
    /// the round
    ///  awaits it on the blocking pool through `snapshot_ahead_awaiting`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshotAhead(item) { return makeOneAhead(item); }
    /// ```
    #[cfg(target_os = "linux")]
    fn snapshot_ahead(&self, item: &WorkItem) -> Verdict {
        return crate::worktree_snapshot_ahead_job::snapshot_ahead(item);
    }

    /// What:
    ///  Make a snapshot of a worktree ahead of the next add,
    /// on a thread of the blocking pool.
    /// Why:
    ///  The ioctl waits for a btrfs transaction commit of the whole filesystem,
    /// so a worker of the runtime must not hold it:
    /// the
    ///  commits queued behind this one would wait on it too.
    /// The service's own thread is what takes the wait,
    /// and the
    ///  add that asked for the snapshot waits for the answer instead of for the kernel
    /// (`worktree_snapshot_ahead.rs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// snapshotAheadAwaiting(item) { return makeOneAhead(item); }
    /// ```
    #[cfg(target_os = "linux")]
    fn snapshot_ahead_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            let held: WorkItem = item.clone();
            return match crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget(move || { return crate::worktree_snapshot_ahead_job::snapshot_ahead(&held); }).await {
                Ok(verdict) => verdict,
                // Nothing was made and nothing is lost: the record keeps its lease and the next round finds it.
                Err(reason) => Verdict::Retry(format!("the snapshot ahead of {} did not run ({reason})", item.repository.display())),
            };
        });
    }

    fn maintenance(&self, item: &WorkItem) -> Verdict {
        return match crate::lazy_runtime::run_on_runtime(self.maintenance_awaiting(item)) {
            Ok(verdict) => verdict,
            Err(refusal) => Verdict::Failed(format!("the automatic upkeep could not run: {refusal}")),
        };
    }

    fn maintenance_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            return match run_auto_maintenance_with_awaiting(&self.context, item.repository.as_path(), None).await {
                Ok(AutoMaintenanceOutcome::Exited(code)) if code != 0 => Verdict::Failed(format!("it exited with status {code}")),
                Ok(_) => Verdict::Done,
                Err(error) => Verdict::Failed(format!("{error:?}")),
            };
        });
    }

    fn review(&self, item: &WorkItem) -> Verdict {
        let key: Option<ReviewKey> = self.review_key_of(item.git_dir.as_path());
        #[cfg_attr(not(test), expect(unused_mut, reason = "only a test replaces the endpoint"))]
        let mut options: ReviewOptions = options_for(item, &review_variables(key.as_ref()));
        #[cfg(test)]
        if let Some(endpoint) = self.review_endpoint.as_ref() {
            options.endpoint = endpoint.clone();
        }
        return review_item(&self.context, item, options);
    }

    /// What:
    ///  The same review,
    /// its wait for the contained job made on the runtime.
    /// Why:
    ///   The round of the service's work is tasks of the runtime
    /// (`service_round.rs`),
    /// and the synchronous wait for a review's job blocks the thread that asks,
    ///  which a worker may not do:
    /// the job starts Git.
    /// The key,
    /// the options and the endpoint are the synchronous face's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reviewAwaiting(item) { return reviewItemAwaiting(context, item, optionsOf(item)); }
    /// ```
    fn review_awaiting<'a>(&'a self, item: &'a WorkItem) -> std::pin::Pin<Box<dyn std::future::Future<Output = Verdict> + Send + 'a>> {
        return Box::pin(async move {
            let key: Option<ReviewKey> = self.review_key_of(item.git_dir.as_path());
            #[cfg_attr(not(test), expect(unused_mut, reason = "only a test replaces the endpoint"))]
            let mut options: ReviewOptions = options_for(item, &review_variables(key.as_ref()));
            #[cfg(test)]
            if let Some(endpoint) = self.review_endpoint.as_ref() {
                options.endpoint = endpoint.clone();
            }
            return review_item_awaiting(&self.context, item, options).await;
        });
    }
}

/// Work consumer controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_tests.rs"]
mod tests;

/// A push the service stopped is tried again on its own:
/// the controls with a remote that stalls once.
#[cfg(test)]
#[path = "service_push_retry_tests.rs"]
mod push_retry_tests;
