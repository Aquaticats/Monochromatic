//! What:
//!  The service's consumer of after-commit work: pending `push` and `maintenance` work items of one repository are
//!  claimed, coalesced, run, settled, and a push that failed leaves a record that the person's next wrapper command shows.
//! Why:
//!  A commit must not wait for the network or for upkeep, so the wrapper leaves a record and the resident service does
//!  the work on its own thread (`doc/planning/cli-git-resident-service.md`, "After-commit work"). The service is the
//!  only pusher of the repositories it serves, so no lock is needed: several pending pushes of one branch become one
//!  push, and a service that died mid-push is noticed by the lease of its claims running out, never by a process id: the
//!  holder renews the lease while it works, so a successor that starts beside a holder that is still running leaves its claims
//!  alone, and one that starts after the holder died takes them back as soon as the lease is over. Other kinds of work
//!  (the standards review) are never claimed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const group of coalesce(await claimAll(workDir, ['push', 'maintenance']))) settle(group, await handler(group));
//! ```

use crate::auto_push::push_once;
use crate::journald_native::Priority;
use crate::auto_push_single_flight::PushGit;
use crate::child_environment::{environment_with_overlay, lockfile_pid_overlay};
use crate::config_snapshot::read_config_snapshot;
use crate::lfs_pre_push::uses_lfs;
use crate::maintenance_run::{AutoMaintenanceOutcome, run_auto_maintenance_with};
use crate::auto_push::AutoPushOutcome;
use crate::claim_time::ClaimTime;
use crate::push_failure_notice::{clear_push_failures_before, record_push_failure, record_push_skip};
use crate::push_retry::{Due, PushRetries, RetryChange, Retrying, final_failure, pause_before, plain_words, unrecognised_rejection};
use crate::repo_read_config_text::{last_entry, parse_bool, parse_config_text};
use crate::service_claim_renewal::{ClaimRenewal, HeldClaims, Renewal, held_claims, renewal_interval};
use crate::service_review::{options_for, review_item, review_variables};
use crate::parallel_budget::HostResources;
use crate::service_review_key::ReviewKey;
use crate::service_round::{Assignment, RoundAnswer, RoundDrain, drain_directories};
use crate::service_timing::log_entry;
use crate::standards_review_run::ReviewOptions;
use crate::transaction_git::GitContext;
use crate::work_item::{Claimed, DONE, FAILED, WORK_DIRECTORY, WorkItem, WorkKind, claim_next_of, next_claim_expiry, requeue_expired_reporting};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The kinds of work the service runs itself.
pub const SERVICE_KINDS: [WorkKind; 4] = [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review, WorkKind::Trash];

/// How many times an item is tried before it is given up on.
pub const MAX_TRIES: u32 = 3;

/// What a wake message brings: the repository to work on and the review credential that came with it, if any.
pub type Wake = (PathBuf, Option<ReviewKey>);

/// What a handler came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The work was done, or there was nothing to do.
    Done,
    /// The work failed; the text is what the person should be told.
    Failed(String),
    /// Nothing was done and nothing is left to retry, but the person should be told why; the text is what they are told.
    Skipped(String),
    /// The push failed for a reason that can pass (a stall, the network, an LFS server error, a held lock, a remote that advanced again);
    /// the text is what the person should be told, and the service tries the push again on its own, with growing pauses.
    Retry(String),
}

/// What runs each kind of work.
pub trait WorkHandlers {
    /// Push the branch of these items, which share a repository and a branch; the last is the newest.
    fn push(&self, items: &[WorkItem]) -> Verdict;
    /// Run the repository's automatic upkeep.
    fn maintenance(&self, item: &WorkItem) -> Verdict;
    /// Review one landed commit into the repository's shadow log.
    fn review(&self, item: &WorkItem) -> Verdict;
    /// Empty the directory of left-out files a snapshot-made worktree set aside; a handler that does not know it skips.
    fn trash(&self, _item: &WorkItem) -> Verdict {
        return Verdict::Skipped(String::from("this handler does not empty trash directories"));
    }
}

/// The work directory of a repository.
pub fn work_directory(git_dir: &Path) -> PathBuf {
    return git_dir.join(WORK_DIRECTORY);
}

/// What:
///  Whether an item is one this service should act on for this repository.
/// Why:
///   Anyone who can write the work directory can write an item, so the fields that become a directory, a reference or a
///  program argument are checked before they are used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAcceptable(item, gitDir)
/// ```
pub fn is_acceptable(item: &WorkItem, git_dir: &Path) -> bool {
    return Path::new(item.repository.as_str()).is_absolute()
        && Path::new(item.repository.as_str()).is_dir()
        && Path::new(item.git_dir.as_str()) == git_dir
        && item.branch_ref.starts_with("refs/")
        && !item.branch_ref.contains(['\0', '\n', ' '])
        && item.oid.len() >= 40
        && item.oid.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Moves the claims to `state` and stops renewing them; a claim that another instance took over is said, and left to it.
fn settle_all(work_dir: &Path, held: &HeldClaims, claimed: &[Claimed], state: &str) {
    for item in claimed {
        if let Err(reason) = held.settle(work_dir, item.id.as_str(), state) {
            crate::service_warn!("{reason}");
        }
    }
}

/// What:
///  The claims of `claimed` that this instance still holds, before their work starts.
/// Why:
///   A claim whose renewal found it gone was taken over: another instance runs that item now, so running it here as well would
///  do it twice. The claim is forgotten and the loss said, and nothing else is done for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimed.filter((c) => held.isHeld(c.id))
/// ```
fn still_held(held: &HeldClaims, work_dir: &Path, claimed: Vec<Claimed>) -> Vec<Claimed> {
    let (mine, lost): (Vec<Claimed>, Vec<Claimed>) = claimed.into_iter().partition(|item| { return held.is_held_on_disk(work_dir, item.id.as_str()); });
    for item in lost {
        crate::service_warn!("the claim on {} ran out before this instance could run it, so another instance took the work over and it is not run here",
            item.id
        );
        held.release(item.id.as_str());
    }
    return mine;
}

/// What:
///  Whether a common Git directory belongs to a repository with no main worktree, which `core.bare` in its own `config`, or in its
///  `config.worktree` when `extensions.worktreeConfig` turns that file on, says.
/// Why:
///   A bare repository has no instance whose Git directory is the common one, so the instances of its linked worktrees are the
///  only ones that can run the records left there; a repository with a main worktree has that instance, which owns them.
///   Git moves `core.bare` into `config.worktree` when the extension is enabled (`git help config`, "extensions.worktreeConfig"),
///  and reads that file after `config` only while the extension is on and the repository has a format version
///  (`repo_read_layout.rs`), so the last `core.bare` of the two decides. An unreadable or unparsable file, or a spelling this
///  reader leaves to Git, reads as not bare, and the records then stay for the main worktree's instance, which is the safe side
///  of a wrong answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isBareCommonDirectory(commonDir)
/// ```
fn is_bare_common_directory(common: &Path) -> bool {
    let Ok(text) = std::fs::read(common.join("config")) else {
        return false;
    };
    let Some(mut entries) = parse_config_text(text.as_slice()) else {
        return false;
    };
    let worktree_config: bool = last_entry(entries.as_slice(), "extensions.worktreeconfig").is_some_and(|entry| { return parse_bool(entry.value.as_deref()) == Some(true); })
        && last_entry(entries.as_slice(), "core.repositoryformatversion").is_some();
    if worktree_config {
        // A missing file is an empty file; one that is there and cannot be read or parsed leaves the answer to Git.
        match std::fs::read(common.join("config.worktree")) {
            Ok(bytes) => match parse_config_text(bytes.as_slice()) {
                Some(more) => entries.extend(more),
                None => return false,
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
    }
    return last_entry(entries.as_slice(), "core.bare").is_some_and(|entry| { return parse_bool(entry.value.as_deref()) == Some(true); });
}

/// What:
///  Leaves a notice for each push the service gave up on without ever finishing it: its holder was stopped, or lost, on every
///  try.
/// Why:
///   Such an item moves to `failed` by recovery, not by a handler, so nothing else records why the commit was never backed up
///  and the person would not be told. The notice is the one a skipped push leaves, because nothing failed and nothing was sent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const item of givenUp.filter(isPush)) await recordPushSkip(gitDir, item, reason)
/// ```
fn tell_about_given_up(git_dir: &Path, given_up: &[WorkItem]) {
    for item in given_up.iter().filter(|item| { return item.kind == WorkKind::Push; }) {
        let reason: String = format!(
            "cli-git: the commit service stopped {MAX_TRIES} times before it finished pushing {} (a restart, a kill or a crash each time), so it gave up on that push.",
            item.branch_ref
        );
        if let Err(problem) = record_push_skip(git_dir, item.branch_ref.as_str(), item.oid.as_str(), reason.as_str()) {
            crate::service_error!("the abandoned push of {} could not be recorded for the person: {problem}", item.oid);
        }
    }
}

/// What:
///  What one drain of a directory hands back: how long until it must be looked at again for a claim another instance holds, and
///  the changes it asks of the retry plan.
/// Why:
///   A drain runs on a worker and cannot change the plan, which the service loop owns; it returns what it did and the loop
///  applies it (`PushRetries::apply`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Drained = { wait?: number; changes: RetryChange[] };
/// ```
#[derive(Debug, Default)]
pub struct Drained {
    /// How long until a claim another instance holds runs out its lease, or nothing when there is none.
    pub wait: Option<Duration>,
    /// The plans and the forgets the drain asks of the retry plan, in the order they were made.
    pub changes: Vec<RetryChange>,
}

/// What:
///  The retry state one drain works with: the first pause (nothing when a retryable push is to be reported as failed) and the
///  changes it has asked of the plan so far.
/// Why:
///   One value is passed down to the places that plan or forget a retry, so none of them reads the plan or shares it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Replanning = { first?: number; changes: RetryChange[] };
/// ```
struct Replanning {
    /// The pause before the first new try, or nothing for a drain that keeps no plan.
    first: Option<Duration>,
    /// What was asked of the plan.
    changes: Vec<RetryChange>,
}

/// Methods over the state.
impl Replanning {
    /// Ask the plan to drop the branch: its push landed, or the person must act.
    fn forget(&mut self, git_dir: &Path, branch_ref: &str) {
        self.changes.push(RetryChange::Forget { git_dir: git_dir.to_path_buf(), branch_ref: branch_ref.to_string() });
    }
}

/// What:
///  Plan the next try of a push that failed for a reason that can pass, say so in the journal, and give the words the person's
///  failure record carries.
/// Why:
///   The journal line names the cause, the number of the try that is planned and the pause before it, so the service's quiet
///  between tries is explained; the person's record says the service keeps trying, so they do not push by hand for nothing.
///  The plan is asked of the service loop (`Replanning`), which owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = plannedRetry(gitDir, replanning, first, item, attempt, text);
/// ```
fn planned_retry(git_dir: &Path, replanning: &mut Replanning, first: Duration, item: &WorkItem, attempt: u32, text: &str) -> String {
    let pause: Duration = pause_before(attempt, first);
    replanning.changes.push(RetryChange::Plan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due: Instant::now() + pause });
    let plan: String = format!(
        "cli-git: the commit service tries this push again by itself in {} seconds (try {}), and keeps trying with longer pauses until the push lands or a conflict needs you.",
        pause.as_secs(),
        attempt + 1
    );
    let fields: [(&str, String); 3] = [
        ("CLI_GIT_BRANCH", item.branch_ref.clone()),
        ("CLI_GIT_OID", item.oid.clone()),
        ("CLI_GIT_OUTCOME", String::from("retrying")),
    ];
    log_entry(
        Priority::Warning,
        format!("push will be tried again branch={:?} oid={} next_try={} in_seconds={} cause: {}", item.branch_ref, item.oid, attempt + 1, pause.as_secs(), text.lines().next().unwrap_or_default()).as_str(),
        &fields,
    );
    return format!("{plan}\n{text}");
}

/// What:
///  Push again every branch of this Git directory whose pause is over, with no new commit needed.
/// Why:
///   The work loop looks at the repository when `drain` tells it to (the wait it returns), and this is what the look does for
///  a push the service stopped: it lands and the person's failure record goes, it is planned again with a longer pause, or it
///  is given up on because a conflict needs the person. The due pushes come with the directory's request, taken from the plan
///  by the service loop before the round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const due of request.due.filter((d) => d.gitDir === gitDir)) await settleRetry(await handlers.push([due.item]));
/// ```
fn run_due_push_retries(git_dir: &Path, handlers: &dyn WorkHandlers, candidates: &[Due], replanning: &mut Replanning) {
    for due in candidates.iter().filter(|candidate| { return candidate.git_dir == git_dir; }) {
        let branch: &str = due.item.branch_ref.as_str();
        let commit: &str = due.item.oid.as_str();
        if !is_acceptable(&due.item, git_dir) {
            crate::service_warn!("the push of {branch} is not tried again: its repository or branch record is no longer acceptable");
            replanning.forget(git_dir, branch);
            continue;
        }
        let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
        let began: Instant = Instant::now();
        let verdict: Verdict = handlers.push(std::slice::from_ref(&due.item));
        match verdict {
            Verdict::Done => {
                clear_push_failures_before(git_dir, branch, pushing_since);
                replanning.forget(git_dir, branch);
                crate::service_log!("the push of {branch:?} landed on try {} push_ms={}", due.attempt + 1, began.elapsed().as_millis());
            }
            Verdict::Retry(text) => {
                let said: String = match replanning.first {
                    Some(first) => planned_retry(git_dir, replanning, first, &due.item, due.attempt + 1, text.as_str()),
                    None => {
                        replanning.forget(git_dir, branch);
                        text
                    }
                };
                if let Err(reason) = record_push_failure(git_dir, branch, commit, said.as_str()) {
                    crate::service_error!("the failed push of {commit} could not be recorded for the person: {reason}");
                }
            }
            Verdict::Failed(text) => {
                crate::service_warn!("the push of {branch:?} failed on try {} and needs you: {}", due.attempt + 1, text.lines().next().unwrap_or_default());
                replanning.forget(git_dir, branch);
                if let Err(reason) = record_push_failure(git_dir, branch, commit, text.as_str()) {
                    crate::service_error!("the failed push of {commit} could not be recorded for the person: {reason}");
                }
            }
            Verdict::Skipped(text) => {
                replanning.forget(git_dir, branch);
                if !text.is_empty() && let Err(reason) = record_push_skip(git_dir, branch, commit, text.as_str()) {
                    crate::service_error!("the skipped push of {commit} could not be recorded for the person: {reason}");
                }
            }
        }
    }
}

/// How long a claim goes without a renewal before another instance runs its item again; the holder renews it three times in this
/// span while it works (`service_claim_renewal.rs`), so a holder that is slow keeps its claim and one that died loses it
/// after this long.
///  Measured by killing a throwaway service during a claim: the service binary in a fixture repository claimed a push held by the
///  remote's hook, was killed with SIGKILL 1.5 s after the claim (with its process group), and a new service was started and woken;
///  the push was made again 119.57, 119.56 and 119.55 s after the kill in three runs (2026-11-03), so the cost of a kill is the lease
///  to within half a second. No kill of a commit service appears in the user journal of this host (19 boots), so the kills a person
///  meets were not counted. The renewal thread is independent of the landing (`service_claim_renewal.rs`), so a landing that stalled for
///  119.8 s (the longest `total_ms` in the service journal) does not stop renewals; a stall of the whole process longer than the
///  80 s a renewal may be late would. Kept at 120 s as the coordinator's pick under the owner's delegation, open to the owner's veto
///  (`doc/planning/cli-git-shipped-limits-audit.md`, "Picked for these follow-ups", batch size and claim lease, option B).
pub const CLAIM_LEASE: Duration = Duration::from_secs(120);

/// What:
///  Runs everything pending for one repository: takes back the claims whose holder stopped renewing them, claims every pending
///  push, maintenance and review item, runs one push per branch and one upkeep per repository, settles the items, and leaves a
///  notice for each failed or skipped push. Gives how long until a claim another instance still holds runs out its lease, or
///  nothing when there is none.
/// Why:
///   Coalescing keeps a burst of commits to one push; a handler that fails leaves the items in `failed`, not in a loop. A
///  claim found in this directory may belong to a live instance (a restarted service starts beside the one it replaces, and the
///  linked worktrees of a bare repository share a directory), so only its lease says it was abandoned; the caller is told when
///  to look again, because nothing else would wake it once the lease is over. This keeps no retry plan: a push that could pass
///  is reported as failed, as for handlers that do not retry. The service's rounds use `drain_planned` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drain(gitDir, handlers, readClaimTime())
/// ```
pub fn drain(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime) -> Option<Duration> {
    return drain_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain` with the lease given, so a test can use a short one.
pub fn drain_with_lease(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    return drain_with_renewal(git_dir, handlers, now, lease, Renewal::Timed);
}

/// `drain_with_lease` with the way its claims are kept fresh given,
/// so a control can drive the renewal itself.
pub fn drain_with_renewal(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, renewal: Renewal<'_>) -> Option<Duration> {
    return drain_planned_with(git_dir, handlers, now, lease, renewal, &Retrying::none()).wait;
}

/// `drain` for a service round: the pushes due again come with the request, and the changes the drain asks of the retry plan are
/// given back with the wait, for the service loop to apply.
pub fn drain_planned(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, retrying: &Retrying) -> Drained {
    return drain_planned_with(git_dir, handlers, now, CLAIM_LEASE, Renewal::Timed, retrying);
}

/// `drain_planned` with the lease and the renewal given.
pub fn drain_planned_with(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, renewal: Renewal<'_>, retrying: &Retrying) -> Drained {
    let work_dir: PathBuf = work_directory(git_dir);
    let mut replanning: Replanning = Replanning { first: retrying.first, changes: Vec::new() };
    tell_about_given_up(git_dir, requeue_expired_reporting(work_dir.as_path(), now, lease, MAX_TRIES).given_up.as_slice());
    run_due_push_retries(git_dir, handlers, retrying.due.as_slice(), &mut replanning);
    drain_pending(git_dir, handlers, lease, renewal, &mut replanning);
    let wait: Option<Duration> = next_claim_expiry(work_dir.as_path(), &ClaimTime::read(), lease);
    return Drained { wait, changes: replanning.changes };
}

/// What:
///  Runs the records the releases before one service per worktree left in the common directory of a bare repository, from the
///  instance of a linked worktree: nothing for a repository with a main worktree, for a main worktree and for a directory with
///  no `commondir`.
/// Why:
///   Those releases kept every worktree's records in the common directory. A repository with a main worktree has an instance
///  whose own directory that is, and it runs them (the main worktree's current records live there too), so a linked instance
///  must not. A repository whose worktrees are all linked has no such instance, and its pushes would sit there with no notice.
///   Several linked instances may drain the directory at once, so a claim is taken back only after its lease, which the holder
///  renews while it works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wait = await drainCommonRecords(linkedGitDir, handlers, now)
/// ```
pub fn drain_common_records(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime) -> Option<Duration> {
    return drain_common_records_with_lease(git_dir, handlers, now, CLAIM_LEASE);
}

/// `drain_common_records` with the lease given, so a test can use a short one.
pub fn drain_common_records_with_lease(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration) -> Option<Duration> {
    return drain_common_records_planned(git_dir, handlers, now, lease, None).wait;
}

/// `drain_common_records` for a service round: `first` is the pause before the first new try of a push that could pass (nothing
/// keeps no plan), and the plans the drain asks for are keyed by the common directory, which the service loop wakes by itself
/// when one is due; that wake drains the common directory as a directory of its own, which has no `commondir`.
pub fn drain_common_records_planned(git_dir: &Path, handlers: &dyn WorkHandlers, now: &ClaimTime, lease: Duration, first: Option<Duration>) -> Drained {
    let Some(common) = common_directory_of(git_dir) else {
        return Drained::default();
    };
    if common == git_dir || !is_bare_common_directory(common.as_path()) || !work_directory(common.as_path()).is_dir() {
        return Drained::default();
    }
    let mut replanning: Replanning = Replanning { first, changes: Vec::new() };
    tell_about_given_up(common.as_path(), requeue_expired_reporting(work_directory(common.as_path()).as_path(), now, lease, MAX_TRIES).given_up.as_slice());
    drain_pending(common.as_path(), handlers, lease, Renewal::Timed, &mut replanning);
    let wait: Option<Duration> = next_claim_expiry(work_directory(common.as_path()).as_path(), &ClaimTime::read(), lease);
    return Drained { wait, changes: replanning.changes };
}

/// The common directory a linked worktree's Git directory names in its `commondir` file, or nothing for any other directory.
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

/// Claims and runs every pending item of one work directory until none is left; the claims held are renewed every third of
/// `lease`, so that none of them looks abandoned to another instance while it waits or runs.
fn drain_pending(git_dir: &Path, handlers: &dyn WorkHandlers, lease: Duration, renewal: Renewal<'_>, replanning: &mut Replanning) {
    let work_dir: PathBuf = work_directory(git_dir);
    let (held, renewer) = held_claims(work_dir.as_path());
    let _renewing: ClaimRenewal = match renewal {
        Renewal::Timed => ClaimRenewal::start(renewer, renewal_interval(lease)),
        Renewal::Driven(driver) => ClaimRenewal::driven(renewer, driver),
    };
    loop {
        let mut claimed: Vec<Claimed> = Vec::new();
        loop {
            // Each claim carries the reading of the moment it is taken, not of the start of the round, which a long round makes
            //  most of a lease old.
            match claim_next_of(work_dir.as_path(), &ClaimTime::read(), &SERVICE_KINDS) {
                Ok(Some(item)) => {
                    held.hold(&item);
                    claimed.push(item);
                }
                Ok(None) => break,
                Err(reason) => {
                    crate::service_error!("{reason}");
                    break;
                }
            }
        }
        if claimed.is_empty() {
            return;
        }
        let (acceptable, refused): (Vec<_>, Vec<_>) = claimed.into_iter().partition(|item| { return is_acceptable(&item.item, git_dir); });
        settle_all(work_dir.as_path(), &held, &refused, FAILED);
        let (pushes, rest): (Vec<_>, Vec<_>) = acceptable.into_iter().partition(|item| { return item.item.kind == WorkKind::Push; });
        let (trashes, after_trash): (Vec<_>, Vec<_>) = rest.into_iter().partition(|item| { return item.item.kind == WorkKind::Trash; });
        let (reviews, upkeep): (Vec<_>, Vec<_>) = after_trash.into_iter().partition(|item| { return item.item.kind == WorkKind::Review; });
        let mut groups: Vec<((String, String), Vec<Claimed>)> = Vec::new();
        for item in pushes {
            let key = (item.item.repository.clone(), item.item.branch_ref.clone());
            match groups.iter_mut().find(|(existing, _)| { return *existing == key; }) {
                Some((_, members)) => members.push(item),
                None => groups.push((key, vec![item])),
            }
        }
        for (_, claimed_members) in groups {
            let members: Vec<Claimed> = still_held(&held, work_dir.as_path(), claimed_members);
            if members.is_empty() {
                continue;
            }
            let items: Vec<WorkItem> = members.iter().map(|member| { return member.item.clone(); }).collect();
            // The push carries what existed when it began; a notice written after this moment is about something newer.
            let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
            let pushing: Instant = Instant::now();
            let verdict: Verdict = handlers.push(items.as_slice());
            if let Some(newest) = members.last() {
                // One line for every push the service ran, so a branch that was not backed up (or was) is found by its name.
                let (priority, outcome, said): (Priority, &str, &str) = match &verdict {
                    Verdict::Done => (Priority::Info, "done", ""),
                    Verdict::Failed(text) => (Priority::Warning, "failed", text.as_str()),
                    Verdict::Retry(text) => (Priority::Warning, "retrying", text.as_str()),
                    Verdict::Skipped(text) => (Priority::Info, "skipped", text.as_str()),
                };
                let fields: [(&str, String); 3] = [
                    ("CLI_GIT_BRANCH", newest.item.branch_ref.clone()),
                    ("CLI_GIT_OID", newest.item.oid.clone()),
                    ("CLI_GIT_OUTCOME", outcome.to_string()),
                ];
                log_entry(
                    priority,
                    format!("pushed branch={:?} oid={} items={} outcome={outcome} push_ms={}{}{said}", newest.item.branch_ref, newest.item.oid, members.len(), pushing.elapsed().as_millis(), if said.is_empty() { "" } else { ": " }).as_str(),
                    &fields,
                );
            }
            match verdict {
                Verdict::Done => {
                    // A push of the branch went through, so an earlier failure to push it is no longer true. The records go
                    //  before the items settle: a service that dies between the two pushes again, and nothing is lost.
                    if let Some(newest) = members.last() {
                        clear_push_failures_before(git_dir, newest.item.branch_ref.as_str(), pushing_since);
                        replanning.forget(git_dir, newest.item.branch_ref.as_str());
                    }
                    settle_all(work_dir.as_path(), &held, &members, DONE);
                }
                Verdict::Retry(text) => {
                    // Without handlers that retry, a retryable push is a failed one.
                    let said: String = match (members.last(), replanning.first) {
                        (Some(newest), Some(first)) => planned_retry(git_dir, replanning, first, &newest.item, 1, text.as_str()),
                        _ => text,
                    };
                    if let Some(newest) = members.last()
                        && let Err(reason) = record_push_failure(git_dir, newest.item.branch_ref.as_str(), newest.item.oid.as_str(), said.as_str())
                    {
                        crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", newest.item.oid);
                    }
                    settle_all(work_dir.as_path(), &held, &members, FAILED);
                }
                Verdict::Failed(text) => {
                    // A failure the service gives up on is no longer waiting for a new try.
                    if let Some(newest) = members.last() {
                        replanning.forget(git_dir, newest.item.branch_ref.as_str());
                    }
                    // The record goes before the items settle, so an item in `failed` always has its record.
                    if let Some(newest) = members.last()
                        && let Err(reason) = record_push_failure(git_dir, newest.item.branch_ref.as_str(), newest.item.oid.as_str(), text.as_str())
                    {
                        crate::service_error!("the failed push of {} could not be recorded for the person: {reason}", newest.item.oid);
                    }
                    settle_all(work_dir.as_path(), &held, &members, FAILED);
                }
                Verdict::Skipped(text) => {
                    // Nothing was pushed, so an earlier failure of the branch is still true and stays. A skip with words leaves
                    //  its own record before the items settle; one that could not be recorded settles as failed, so no item
                    //  is done without the person having been told.
                    let recorded: bool = match members.last() {
                        Some(newest) if !text.is_empty() => match record_push_skip(git_dir, newest.item.branch_ref.as_str(), newest.item.oid.as_str(), text.as_str()) {
                            Ok(()) => true,
                            Err(reason) => {
                                crate::service_error!("the skipped push of {} could not be recorded for the person: {reason}", newest.item.oid);
                                false
                            }
                        },
                        _ => true,
                    };
                    settle_all(work_dir.as_path(), &held, &members, if recorded { DONE } else { FAILED });
                }
            }
        }
        let mut upkept: Vec<String> = Vec::new();
        for item in still_held(&held, work_dir.as_path(), upkeep) {
            if !upkept.contains(&item.item.repository) {
                upkept.push(item.item.repository.clone());
                let upkeep_began: Instant = Instant::now();
                match handlers.maintenance(&item.item) {
                    Verdict::Failed(reason) | Verdict::Retry(reason) => crate::service_warn!("automatic maintenance did not run: {reason}"),
                    Verdict::Done | Verdict::Skipped(_) => crate::service_log!("automatic maintenance ran maintenance_ms={}", upkeep_began.elapsed().as_millis()),
                };
            }
            settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
        }
        // The left-out files of a snapshot-made worktree are emptied after the person's backup and the upkeep, before the shadow
        //  reviews: freeing them is a bounded, local job.
        for item in still_held(&held, work_dir.as_path(), trashes) {
            match handlers.trash(&item.item) {
                Verdict::Done => settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) | Verdict::Retry(reason) => {
                    eprintln!("cli-git-service: the trash {} could not be emptied: {reason}", item.item.parent);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), FAILED);
                }
                Verdict::Skipped(reason) => {
                    eprintln!("cli-git-service: the trash {} was left as it is: {reason}", item.item.parent);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
                }
            }
        }
        // Reviews go last: a push is the person's backup, a review is a shadow record, and each is bounded by its budget.
        for item in still_held(&held, work_dir.as_path(), reviews) {
            let review_began: Instant = Instant::now();
            let verdict: Verdict = handlers.review(&item.item);
            if matches!(verdict, Verdict::Done) {
                crate::service_log!("the review of {} is done review_ms={}", item.item.oid, review_began.elapsed().as_millis());
            }
            match verdict {
                Verdict::Done => settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) | Verdict::Retry(reason) => {
                    crate::service_warn!("the review of {} failed: {reason}", item.item.oid);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), FAILED);
                }
                Verdict::Skipped(reason) => {
                    crate::service_log!("the review of {} was skipped: {reason}", item.item.oid);
                    settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
                }
            }
        }
    }
}

/// What:
///  Runs one round of wakes: holds each wake's review credential in memory for the round, drains each named repository once,
///  repositories side by side, and drops the credentials when the round ends. Gives, for each repository that has a claim
///  another instance holds, how long until that claim's lease runs out, in the order the repositories were named.
/// Why:
///   The credential is held only for the reviews of the round that brought it; a service that restarts before the round
///  runs has none, and its reviews record the plain "no key" outcome. The service looks at such a repository again when the
///  time is over, because the holder may have died and nothing else would wake the work. Repositories share nothing, so the
///  round drains them side by side (`service_round.rs`); the credentials are passed to the workers as values in a copy of the
///  handlers made for this round, so nothing is shared mutably and no lock is taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const round = handlers.forRound(keysOf(wakes)); await Promise.all(directories.map((dir) => drain(dir, round)));
/// ```
pub fn run_round(wakes: Vec<Wake>, handlers: &GitHandlers, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    return run_round_on(&HostResources::read(), wakes, handlers, retries, now);
}

/// `run_round` with the host's resources chosen by the caller, so a control states a host of any size.
pub fn run_round_on(host: &HostResources, wakes: Vec<Wake>, handlers: &GitHandlers, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    let mut directories: Vec<PathBuf> = Vec::new();
    let mut keys: std::collections::HashMap<PathBuf, ReviewKey> = std::collections::HashMap::new();
    for (directory, review_key) in wakes {
        if let Some(key) = review_key {
            keys.insert(directory.clone(), key);
        }
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    let round: GitHandlers = handlers.for_round(keys);
    return run_planned_round(host, directories.as_slice(), &round, retries, now);
}

/// What:
///  Runs one round over `directories` with the retry plan: takes what is due for each directory out of the plan, hands it to
///  that directory's worker as an owned value, applies the changes the workers ask once they are done, and gives for each
///  directory when to look at it again, whichever is first of its leases and its plan.
/// Why:
///   The plan is owned by the caller and changed here on the caller's thread only, so the workers share nothing and no lock is
///  taken. A directory whose drain planned a push under another directory (a common directory of a bare repository) is named
///  too, so that its wake is not lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const due = dirs.map((d) => retries.takeDue(d)); const answers = await Promise.all(dirs.map((d, i) => drain(d, due[i]))); retries.apply(answers.flatMap(changes));
/// ```
pub fn run_planned_round<H: WorkHandlers + Sync>(host: &HostResources, directories: &[PathBuf], handlers: &H, retries: &mut PushRetries, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    let taken_at: Instant = Instant::now();
    let first: Duration = retries.first();
    let assignments: Vec<Assignment> = directories
        .iter()
        .map(|directory| { return Assignment { directory: directory.clone(), retrying: Retrying { first: Some(first), due: retries.take_due(directory.as_path(), taken_at) } }; })
        .collect();
    let answered: RoundAnswer = drain_directories(&RoundDrain { directories: assignments.as_slice(), handlers, now, host });
    let mut named: Vec<PathBuf> = directories.to_vec();
    for change in &answered.changes {
        if let RetryChange::Plan { git_dir, .. } = change
            && !named.contains(git_dir)
        {
            named.push(git_dir.clone());
        }
    }
    retries.apply(answered.changes);
    let applied_at: Instant = Instant::now();
    let mut asked: Vec<(PathBuf, Duration)> = Vec::new();
    for directory in named {
        let lease: Option<Duration> = answered.waits.iter().find(|(named_directory, _)| { return *named_directory == directory; }).map(|(_, wait)| { return *wait; });
        let planned: Option<Duration> = retries.next_wait(directory.as_path(), applied_at);
        if let Some(wait) = [lease, planned].into_iter().flatten().min() {
            asked.push((directory, wait));
        }
    }
    return asked;
}

/// The handlers that run real Git.
pub struct GitHandlers {
    /// The context every Git child of the service runs in.
    pub context: GitContext,
    /// The review credential of each repository whose wake message carried one, for the round these handlers were made for. Held
    /// in memory only and never changed after the handlers are made: a round makes its own copy (`for_round`) and drops it when
    /// the round ends, so the workers of a round only read it and nothing waits for it.
    review_keys: std::collections::HashMap<PathBuf, ReviewKey>,
    /// Where tests point the review's gateway call, so no test reaches the real endpoint.
    #[cfg(test)]
    review_endpoint: Option<String>,
    /// Variables a test adds to the environment of the push, such as a short stall window.
    #[cfg(test)]
    push_environment: Vec<(std::ffi::OsString, std::ffi::OsString)>,
}

/// Methods over the real handlers.
impl GitHandlers {
    /// Handlers that run real Git in `context`, with no review credential yet.
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

    /// The same handlers with variables added to the environment of every push; tests only.
    #[cfg(test)]
    pub fn with_push_environment(mut self, variables: &[(&str, &str)]) -> GitHandlers {
        self.push_environment = variables.iter().map(|(name, value)| { return (std::ffi::OsString::from(name), std::ffi::OsString::from(value)); }).collect();
        return self;
    }

    /// The same handlers with the review's gateway call sent to `endpoint`; tests only.
    #[cfg(test)]
    pub fn with_review_endpoint(mut self, endpoint: &str) -> GitHandlers {
        self.review_endpoint = Some(endpoint.to_string());
        return self;
    }

    /// A copy of these handlers that holds `keys` as the review credentials of one round; the copy is dropped with the round.
    /// The test-only settings (the review endpoint, the push environment) are carried into the copy, so a round runs the way the
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

    /// The credential held for this repository, if the round brought one.
    fn review_key_of(&self, git_dir: &Path) -> Option<ReviewKey> {
        return self.review_keys.get(git_dir).cloned();
    }

    fn push_git(&self, item: &WorkItem) -> PushGit {
        let cwd: PathBuf = PathBuf::from(item.repository.as_str());
        // Git writes a `~pid.lock` file beside every lock it takes when `core.lockfilePid` is on, which names the owner of a lock
        //  that a stopped push leaves behind; the service's own environment does not turn it on.
        #[cfg_attr(not(test), expect(unused_mut, reason = "only a test adds variables"))]
        let mut inherited: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
        #[cfg(test)]
        inherited.extend(self.push_environment.iter().cloned());
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot(&self.context, cwd.as_path()).as_ref().is_none_or(uses_lfs),
            cwd,
            environment: environment_with_overlay(inherited.as_slice(), lockfile_pid_overlay(inherited.as_slice()).as_slice()),
        };
    }
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
            Ok(result) if result.exit_code == 0 => Verdict::Done,
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
        return match run_auto_maintenance_with(&self.context, Path::new(item.repository.as_str()), None) {
            Ok(AutoMaintenanceOutcome::Exited(code)) if code != 0 => Verdict::Failed(format!("it exited with status {code}")),
            Ok(_) => Verdict::Done,
            Err(error) => Verdict::Failed(format!("{error:?}")),
        };
    }

    fn review(&self, item: &WorkItem) -> Verdict {
        let key: Option<ReviewKey> = self.review_key_of(Path::new(item.git_dir.as_str()));
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

/// A push the service stopped is tried again on its own: the controls with a remote that stalls once.
#[cfg(test)]
#[path = "service_push_retry_tests.rs"]
mod push_retry_tests;
