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

use super::auto_push::push_once;
use super::auto_push_single_flight::PushGit;
use super::config_snapshot::read_config_snapshot;
use super::lfs_pre_push::uses_lfs;
use super::maintenance_run::{AutoMaintenanceOutcome, run_auto_maintenance_with};
use super::auto_push::AutoPushOutcome;
use super::claim_time::ClaimTime;
use super::push_failure_notice::{clear_push_failures_before, record_push_failure, record_push_skip};
use super::repo_read_config_text::{last_entry, parse_bool, parse_config_text};
use super::service_claim_renewal::{ClaimRenewal, HeldClaims, RENEWALS_PER_LEASE};
use super::service_review::{BUDGET, options_for, review_item, review_variables};
use super::service_review_key::ReviewKey;
use super::standards_review_run::ReviewOptions;
use super::transaction_git::GitContext;
use super::work_item::{Claimed, DONE, FAILED, WORK_DIRECTORY, WorkItem, WorkKind, claim_next_of, next_claim_expiry, requeue_expired_reporting};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The kinds of work the service runs itself.
pub const SERVICE_KINDS: [WorkKind; 3] = [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review];

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
}

/// What runs each kind of work.
pub trait WorkHandlers {
    /// Push the branch of these items, which share a repository and a branch; the last is the newest.
    fn push(&self, items: &[WorkItem]) -> Verdict;
    /// Run the repository's automatic upkeep.
    fn maintenance(&self, item: &WorkItem) -> Verdict;
    /// Review one landed commit into the repository's shadow log.
    fn review(&self, item: &WorkItem) -> Verdict;
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
fn still_held(held: &HeldClaims, claimed: Vec<Claimed>) -> Vec<Claimed> {
    let (mine, lost): (Vec<Claimed>, Vec<Claimed>) = claimed.into_iter().partition(|item| { return held.is_held(item.id.as_str()); });
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

/// How long a claim goes without a renewal before another instance runs its item again; the holder renews it three times in this
/// span while it works (`service_claim_renewal.rs`), so a holder that is slow keeps its claim and one that died loses it
/// after this long.
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
///  to look again, because nothing else would wake it once the lease is over.
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
    let work_dir: PathBuf = work_directory(git_dir);
    tell_about_given_up(git_dir, requeue_expired_reporting(work_dir.as_path(), now, lease, MAX_TRIES).given_up.as_slice());
    drain_pending(git_dir, handlers, lease);
    return next_claim_expiry(work_dir.as_path(), &ClaimTime::read(), lease);
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
    let common = common_directory_of(git_dir)?;
    if common == git_dir || !is_bare_common_directory(common.as_path()) || !work_directory(common.as_path()).is_dir() {
        return None;
    }
    tell_about_given_up(common.as_path(), requeue_expired_reporting(work_directory(common.as_path()).as_path(), now, lease, MAX_TRIES).given_up.as_slice());
    drain_pending(common.as_path(), handlers, lease);
    return next_claim_expiry(work_directory(common.as_path()).as_path(), &ClaimTime::read(), lease);
}

/// The common directory a linked worktree's Git directory names in its `commondir` file, or nothing for any other directory.
fn common_directory_of(git_dir: &Path) -> Option<PathBuf> {
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
fn drain_pending(git_dir: &Path, handlers: &dyn WorkHandlers, lease: Duration) {
    let work_dir: PathBuf = work_directory(git_dir);
    let held: HeldClaims = HeldClaims::default();
    let _renewal: ClaimRenewal = ClaimRenewal::start(work_dir.clone(), held.clone(), lease / RENEWALS_PER_LEASE);
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
        let (reviews, upkeep): (Vec<_>, Vec<_>) = rest.into_iter().partition(|item| { return item.item.kind == WorkKind::Review; });
        let mut groups: Vec<((String, String), Vec<Claimed>)> = Vec::new();
        for item in pushes {
            let key = (item.item.repository.clone(), item.item.branch_ref.clone());
            match groups.iter_mut().find(|(existing, _)| { return *existing == key; }) {
                Some((_, members)) => members.push(item),
                None => groups.push((key, vec![item])),
            }
        }
        for (_, claimed_members) in groups {
            let members: Vec<Claimed> = still_held(&held, claimed_members);
            if members.is_empty() {
                continue;
            }
            let items: Vec<WorkItem> = members.iter().map(|member| { return member.item.clone(); }).collect();
            // The push carries what existed when it began; a notice written after this moment is about something newer.
            let pushing_since: std::time::SystemTime = std::time::SystemTime::now();
            match handlers.push(items.as_slice()) {
                Verdict::Done => {
                    // A push of the branch went through, so an earlier failure to push it is no longer true. The records go
                    //  before the items settle: a service that dies between the two pushes again, and nothing is lost.
                    if let Some(newest) = members.last() {
                        clear_push_failures_before(git_dir, newest.item.branch_ref.as_str(), pushing_since);
                    }
                    settle_all(work_dir.as_path(), &held, &members, DONE);
                }
                Verdict::Failed(text) => {
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
        for item in still_held(&held, upkeep) {
            if !upkept.contains(&item.item.repository) {
                upkept.push(item.item.repository.clone());
                if let Verdict::Failed(reason) = handlers.maintenance(&item.item) {
                    crate::service_warn!("automatic maintenance did not run: {reason}");
                }
            }
            settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE);
        }
        // Reviews go last: a push is the person's backup, a review is a shadow record, and each is bounded by its budget.
        for item in still_held(&held, reviews) {
            match handlers.review(&item.item) {
                Verdict::Done => settle_all(work_dir.as_path(), &held, std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) => {
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
///  Runs one round of wakes: keeps each wake's review credential in memory, drains each named repository once, and
///  forgets the credentials when the repository is done. Gives, for each repository that has a claim another instance holds,
///  how long until that claim's lease runs out.
/// Why:
///   The credential is held only for the reviews of the round that brought it; a service that restarts before the round
///  runs has none, and its reviews record the plain "no key" outcome. The service looks at such a repository again when the
///  time is over, because the holder may have died and nothing else would wake the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [dir, key] of wakes) { remember(dir, key); await drain(dir); forget(dir) }
/// ```
pub fn run_round(wakes: Vec<Wake>, handlers: &GitHandlers, now: &ClaimTime) -> Vec<(PathBuf, Duration)> {
    let mut directories: Vec<PathBuf> = Vec::new();
    for (directory, review_key) in wakes {
        if let Some(key) = review_key {
            handlers.remember_review_key(directory.as_path(), key);
        }
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    let mut retries: Vec<(PathBuf, Duration)> = Vec::new();
    for directory in directories {
        let waits: [Option<Duration>; 2] = [drain(directory.as_path(), handlers, now), drain_common_records(directory.as_path(), handlers, now)];
        if let Some(wait) = waits.into_iter().flatten().min() {
            retries.push((directory.clone(), wait));
        }
        handlers.forget_review_key(directory.as_path());
    }
    return retries;
}

/// The handlers that run real Git.
pub struct GitHandlers {
    /// The context every Git child of the service runs in.
    pub context: GitContext,
    /// The review credential of each repository whose wake message carried one. Held in memory only, removed when the
    /// round that used it ends.
    review_keys: std::sync::Mutex<std::collections::HashMap<PathBuf, ReviewKey>>,
    /// Where tests point the review's gateway call, so no test reaches the real endpoint.
    #[cfg(test)]
    review_endpoint: Option<String>,
}

/// Methods over the real handlers.
impl GitHandlers {
    /// Handlers that run real Git in `context`, with no review credential yet.
    pub fn new(context: GitContext) -> GitHandlers {
        return GitHandlers {
            context,
            review_keys: std::sync::Mutex::new(std::collections::HashMap::new()),
            #[cfg(test)]
            review_endpoint: None,
        };
    }

    /// The same handlers with the review's gateway call sent to `endpoint`; tests only.
    #[cfg(test)]
    pub fn with_review_endpoint(mut self, endpoint: &str) -> GitHandlers {
        self.review_endpoint = Some(endpoint.to_string());
        return self;
    }

    /// Keep the credential for the reviews of this repository until `forget_review_key`.
    pub fn remember_review_key(&self, git_dir: &Path, key: ReviewKey) {
        self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(git_dir.to_path_buf(), key);
    }

    /// Drop the credential of this repository.
    pub fn forget_review_key(&self, git_dir: &Path) {
        self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(git_dir);
    }

    fn review_key_of(&self, git_dir: &Path) -> Option<ReviewKey> {
        return self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(git_dir).cloned();
    }

    fn push_git(&self, item: &WorkItem) -> PushGit {
        let cwd: PathBuf = PathBuf::from(item.repository.as_str());
        return PushGit {
            real_git: self.context.real_git.clone(),
            lfs: read_config_snapshot(&self.context, cwd.as_path()).as_ref().is_none_or(uses_lfs),
            cwd,
            environment: std::env::vars_os().collect(),
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
            Ok(result) => Verdict::Failed(result.shown),
            Err(error) => Verdict::Failed(format!("{error}")),
        };
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
        return review_item(&self.context, item, options, BUDGET);
    }
}

/// Work consumer controls stay out of the release executable.
#[cfg(test)]
#[path = "service_work_tests.rs"]
mod tests;
