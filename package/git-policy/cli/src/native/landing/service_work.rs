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
pub(crate) use crate::service_work_drain_pending_claims::drain_pending;
use crate::auto_push::push_once;
use crate::auto_push_single_flight::PushGit;
use crate::child_environment::{environment_with_overlay, service_child_overlay};
use crate::config_snapshot::read_config_snapshot;
use crate::lfs_pre_push::uses_lfs;
use crate::maintenance_run::{AutoMaintenanceOutcome, run_auto_maintenance_with};
use crate::auto_push::AutoPushOutcome;
use crate::claim_time::ClaimTime;
use crate::push_retry::{Retrying, final_failure, plain_words, unrecognised_rejection};
use crate::push_failure_notice::record_push_report;
use crate::auto_push_reconcile_basics_and_limits::RECONCILE_LINE_PREFIX;
use crate::service_claim_renewal_running::Renewal;
use crate::service_review::{options_for, review_item, review_variables};
use crate::service_review_key::ReviewKey;
use crate::standards_review_run::ReviewOptions;
use crate::transaction_git::GitContext;
use crate::work_item::WorkItem;
use crate::work_item_recovery::{Recovered, next_claim_expiry, requeue_expired_reporting_telling};
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
/// not in a loop.
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

/// `drain_planned` with the lease and the renewal given.
pub fn drain_planned_with(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, renewal: Renewal<'_>, retrying: &Retrying) -> Drained {
    // A directory that cannot be reached runs nothing and keeps its push plans (plan item W3).
    if let Err(unreachable) = crate::service_instance::try_instance_of(git_dir) {
        return keep_plans_while_unreachable(git_dir, retrying, &unreachable);
    }
    let work_dir: PathBuf = work_directory(git_dir);
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    // The notice is written before the item moves to `failed`: nothing scans `failed`, so the order is the only thing that
    //  keeps a kill in between from abandoning the branch in silence.
    let _recovered: Recovered = requeue_expired_reporting_telling(work_dir.as_path(), now, lease, MAX_TRIES, &SkipNotice(git_dir));
    run_due_push_retries(git_dir, handlers, retrying.due.as_slice(), &mut replanning);
    drain_pending(git_dir, handlers, lease, renewal, &mut replanning);
    let wait: Option<Duration> = next_claim_expiry(work_dir.as_path(), &ClaimTime::read(), lease);
    return Drained { wait, changes: replanning.changes };
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
pub fn drain_common_records_planned(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, retrying: &Retrying) -> Drained {
    let Some(common) = common_directory_of(git_dir) else {
        return Drained::default();
    };
    if common == git_dir || !is_bare_common_directory(common.as_path()) || !work_directory(common.as_path()).is_dir() {
        return Drained::default();
    }
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new(), plans: retrying.plans.clone() };
    let _recovered: Recovered = requeue_expired_reporting_telling(work_directory(common.as_path()).as_path(), now, lease, MAX_TRIES, &SkipNotice(common.as_path()));
    drain_pending(common.as_path(), handlers, lease, Renewal::Timed, &mut replanning);
    let wait: Option<Duration> = next_claim_expiry(work_directory(common.as_path()).as_path(), &ClaimTime::read(), lease);
    return Drained { wait, changes: replanning.changes };
}

/// The common directory a linked worktree's Git directory names in its `commondir` file,
/// or nothing for any other directory.
pub(crate) fn common_directory_of(git_dir: &Path) -> Option<PathBuf> {
    let text: String = std::fs::read_to_string(git_dir.join("commondir")).ok()?;
    let named: &Path = Path::new(text.trim_end_matches(['\n', '\r']));
    if named.as_os_str().is_empty() {
        return None;
    }
    // Resolved by the text alone, so the result spells the path the way the worktree's own Git directory does, which is how the
    //  records name their directory.
    let mut resolved: PathBuf = PathBuf::new();
    for part in git_dir.join(named).components() {
        match part {
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            other => resolved.push(other.as_os_str()),
        }
    }
    return Some(resolved);
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

    fn push_git(&self, item: &WorkItem) -> PushGit {
        let cwd: PathBuf = item.repository.clone();
        // Git writes a `~pid.lock` file beside every lock it takes when `core.lockfilePid` is on, which names the owner of a lock
        //  that a stopped push leaves behind; the service's own environment does not turn it on. The overlay also says never to ask for
        //  a login (`never_prompt_overlay`): the host's askpass program is in this environment, and a dialog nobody answers is a push that waits.
        #[cfg_attr(not(test), expect(unused_mut, reason = "only a test adds variables"))]
        let mut inherited: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
        #[cfg(test)]
        inherited.extend(self.push_environment.iter().cloned());
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot(&self.context, cwd.as_path()).as_ref().is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), service_child_overlay(inherited.as_slice()).as_slice()),
        };
    }
}

/// What:
///  Whether the branch the item names exists in the repository,
/// or nothing when the question cannot be asked.
/// Why:
///   A push of a branch that is not there fails the same way on every try,
/// and nothing about it can pass by waiting;
///  a push that failed for another reason must still be retried,
/// so the answer decides,
/// and never the words of the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await git.revParseVerifyQuiet(item.branch_ref); return answer.code === 0 ? true : answer.code === 1 ? false : undefined
/// ```
fn branch_exists(git: &PushGit, branch_ref: &str) -> Option<bool> {
    let asked = crate::auto_push_single_flight::run_query(git, &crate::auto_push_reconcile_basics_and_limits::argv(&["rev-parse", "--verify", "--quiet", branch_ref])).ok()?;
    return match asked.code {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    };
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

/// What:
///  The words for a push whose branch the person deleted:
/// what happened,
/// which commit it named,
/// and every ref
///  that still holds that commit.
/// Why:
///   Nothing was sent and nothing can pass by waiting,
/// so the person is told once that the branch is gone;
/// the commit
///  is not on that branch any more because the person removed the branch,
/// and when no ref holds it any more they are told
///  that,
/// with the commit named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refs = await git.for_each_ref(['--contains', item.oid, '--format=%(refname)']);
/// `... the branch ${branch} no longer exists ... ${refs.length ? `Commit ${oid} is still on ${refs.join(', ')}.` : `Commit ${oid} is on no ref of this repository any more, ...`}`
/// ```
fn branch_is_gone_words(git: &PushGit, item: &WorkItem) -> String {
    let branch: &str = item.branch_ref.strip_prefix("refs/heads/").unwrap_or(item.branch_ref.as_str());
    // A query that failed is not an answer: nothing is said about where the commit lives when the refs could not be read.
    let holding: Option<Vec<String>> = match crate::auto_push_single_flight::run_query(git, &crate::auto_push_reconcile_basics_and_limits::argv(&["for-each-ref", "--contains", item.oid.as_str(), "--format=%(refname)"])) {
        Ok(answer) if answer.code == Some(0) => {
            let mut named: Vec<String> = Vec::new();
            for line in String::from_utf8_lossy(answer.stdout.as_slice()).lines() {
                let held: String = line.trim().to_owned();
                if !held.is_empty() {
                    named.push(held);
                }
            }
            Some(named)
        }
        Ok(answer) => {
            crate::service_warn!("the refs that hold {} could not be read (git for-each-ref exited {}), so the notice does not say where the commit lives", item.oid, match answer.code { Some(code) => code.to_string(), None => String::from("on a signal") });
            None
        }
        Err(error) => {
            crate::service_warn!("the refs that hold {} could not be read ({error}), so the notice does not say where the commit lives", item.oid);
            None
        }
    };
    const NAMED_AT_MOST: usize = 5;
    let where_it_lives: String = match holding {
        None => format!(
            "Whether any ref of this repository still holds commit {} could not be told, because the refs could not be read.",
            item.oid
        ),
        Some(none) if none.is_empty() => format!(
            "Commit {} is on no ref of this repository any more, so only its object store keeps it and nothing was pushed for it.",
            item.oid
        ),
        Some(named) => {
            let shown: Vec<&str> = named.iter().take(NAMED_AT_MOST).map(String::as_str).collect();
            let more: usize = named.len().saturating_sub(shown.len());
            format!(
                "Commit {} is still on {}{}.",
                item.oid,
                shown.join(", "),
                if more == 0 { String::new() } else { format!(" and {more} more refs") }
            )
        }
    };
    return format!(
        "monochromatic-git-policy-cli: the branch {branch} no longer exists in this repository, so this push was not made: the branch was deleted. {where_it_lives}          Nothing is retried for it; a commit you want on a branch again can be pushed by hand after naming it with `git branch`."
    );
}

/// Trait implementation for the real Git.
impl WorkHandlers for GitHandlers {
    fn push(&self, items: &[WorkItem]) -> Verdict {
        let Some(newest) = items.last() else { return Verdict::Done };
        let git: PushGit = self.push_git(newest);
        let mut output: Vec<u8> = Vec::new();
        return match push_once(&git, newest.branch_ref.as_str(), newest.oid.as_str(), &mut output) {
            // A skip sent nothing: with words they are told; with none (no upstream and no origin) the repository has nowhere
            //  to back up to and there is nothing to say, but an earlier failure of the branch is still true.
            Ok(result) if result.outcome == AutoPushOutcome::Skipped => Verdict::Skipped(result.shown),
            Ok(result) if result.exit_code == 0 => {
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
            Ok(_result) if branch_exists(&git, newest.branch_ref.as_str()) == Some(false) => Verdict::Skipped(branch_is_gone_words(&git, newest)),
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
    }

    fn trash(&self, item: &WorkItem) -> Verdict {
        return crate::worktree_trash::empty_trash(item);
    }

    fn maintenance(&self, item: &WorkItem) -> Verdict {
        return match run_auto_maintenance_with(&self.context, item.repository.as_path(), None) {
            Ok(AutoMaintenanceOutcome::Exited(code)) if code != 0 => Verdict::Failed(format!("it exited with status {code}")),
            Ok(_) => Verdict::Done,
            Err(error) => Verdict::Failed(format!("{error:?}")),
        };
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
