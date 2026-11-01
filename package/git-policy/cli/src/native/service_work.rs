//! What:
//!  The service's consumer of after-commit work: pending `push` and `maintenance` work items of one repository are
//!  claimed, coalesced, run, settled, and a push that failed leaves a plain notice for the person's next commit.
//! Why:
//!  A commit must not wait for the network or for upkeep, so the wrapper leaves a record and the resident service does
//!  the work on its own thread (`doc/planning/cli-git-resident-service.md`, "After-commit work"). The service is the
//!  only pusher of the repositories it serves, so no lock is needed: several pending pushes of one branch become one
//!  push, and a service that died mid-push is noticed by the claim's lease, never by a process id. Other kinds of work
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
use super::service_messages::push_failed_notice;
use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use super::service_review::{BUDGET, options_for, review_item, review_variables};
use super::service_review_key::ReviewKey;
use super::standards_review_run::ReviewOptions;
use super::transaction_git::GitContext;
use super::work_item::{DONE, FAILED, WORK_DIRECTORY, WorkItem, WorkKind, claim_next_of, requeue_expired, settle};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The kinds of work the service runs itself.
pub const SERVICE_KINDS: [WorkKind; 3] = [WorkKind::Push, WorkKind::Maintenance, WorkKind::Review];

/// How long a claim stands before it is taken back: longer than a push's whole budget, so a push that is merely slow is
///  never run twice at once.
pub const LEASE: Duration = Duration::from_secs(30 * 60);

/// How many times an item is tried before it is given up on.
pub const MAX_TRIES: u32 = 3;

/// Directory under the common Git directory that holds notices for the person.
pub const NOTICE_DIRECTORY: &str = "cli-git/notices";

/// What a wake message brings: the repository to work on and the review credential that came with it, if any.
pub type Wake = (PathBuf, Option<ReviewKey>);

/// What a handler came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The work was done, or there was nothing to do.
    Done,
    /// The work failed; the text is what the person should be told.
    Failed(String),
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
pub fn work_directory(common_dir: &Path) -> PathBuf {
    return common_dir.join(WORK_DIRECTORY);
}

/// The notice directory of a repository.
pub fn notice_directory(common_dir: &Path) -> PathBuf {
    return common_dir.join(NOTICE_DIRECTORY);
}

/// What:
///  Whether an item is one this service should act on for this repository.
/// Why:
///   Anyone who can write the work directory can write an item, so the fields that become a directory, a reference or a
///  program argument are checked before they are used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAcceptable(item, commonDir)
/// ```
pub fn is_acceptable(item: &WorkItem, common_dir: &Path) -> bool {
    return Path::new(item.repository.as_str()).is_absolute()
        && Path::new(item.repository.as_str()).is_dir()
        && Path::new(item.git_dir.as_str()) == common_dir
        && item.branch_ref.starts_with("refs/")
        && !item.branch_ref.contains(['\0', '\n', ' '])
        && item.oid.len() >= 40
        && item.oid.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Leaves a notice for the person, named after the work item so a repeat replaces it.
pub fn write_notice(common_dir: &Path, id: &str, text: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(common_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| { return format!("creating the notice directory failed: {error}"); })?;
    let path: PathBuf = directory.join(format!("{id}.txt"));
    // A repeat of the same item replaces its notice.
    let _removed = std::fs::remove_file(path.as_path());
    return write_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| { return format!("writing the notice failed: {error}"); });
}

/// What:
///  Reads and removes the repository's notices, oldest name first.
/// Why:
///   A notice is shown once, by the next commit that goes through the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await takeNotices(commonDir)
/// ```
pub fn take_notices(common_dir: &Path) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(notice_directory(common_dir)) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).collect();
    paths.sort();
    let mut texts: Vec<String> = Vec::new();
    for path in paths {
        if let Ok(text) = std::fs::read_to_string(path.as_path()) {
            texts.push(text);
        }
        let _removed = std::fs::remove_file(path.as_path());
    }
    return texts;
}

fn settle_all(work_dir: &Path, claimed: &[super::work_item::Claimed], state: &str) {
    for item in claimed {
        if let Err(reason) = settle(work_dir, item, state) {
            eprintln!("cli-git-service: {reason}");
        }
    }
}

/// What:
///  Runs everything pending for one repository: takes back expired claims, claims every pending push and maintenance
///  item, runs one push per branch and one upkeep per repository, settles the items, and leaves a notice for each
///  failed push.
/// Why:
///   Coalescing keeps a burst of commits to one push; a handler that fails leaves the items in `failed`, not in a loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await drain(commonDir, handlers, new Date())
/// ```
pub fn drain(common_dir: &Path, handlers: &dyn WorkHandlers, now: SystemTime) {
    let work_dir: PathBuf = work_directory(common_dir);
    let _requeued = requeue_expired(work_dir.as_path(), now, LEASE, MAX_TRIES);
    loop {
        let mut claimed: Vec<super::work_item::Claimed> = Vec::new();
        loop {
            match claim_next_of(work_dir.as_path(), now, &SERVICE_KINDS) {
                Ok(Some(item)) => claimed.push(item),
                Ok(None) => break,
                Err(reason) => {
                    eprintln!("cli-git-service: {reason}");
                    break;
                }
            }
        }
        if claimed.is_empty() {
            return;
        }
        let (acceptable, refused): (Vec<_>, Vec<_>) = claimed.into_iter().partition(|item| { return is_acceptable(&item.item, common_dir); });
        settle_all(work_dir.as_path(), &refused, FAILED);
        let (pushes, rest): (Vec<_>, Vec<_>) = acceptable.into_iter().partition(|item| { return item.item.kind == WorkKind::Push; });
        let (reviews, upkeep): (Vec<_>, Vec<_>) = rest.into_iter().partition(|item| { return item.item.kind == WorkKind::Review; });
        let mut groups: Vec<((String, String), Vec<super::work_item::Claimed>)> = Vec::new();
        for item in pushes {
            let key = (item.item.repository.clone(), item.item.branch_ref.clone());
            match groups.iter_mut().find(|(existing, _)| { return *existing == key; }) {
                Some((_, members)) => members.push(item),
                None => groups.push((key, vec![item])),
            }
        }
        for (_, members) in groups {
            let items: Vec<WorkItem> = members.iter().map(|member| { return member.item.clone(); }).collect();
            match handlers.push(items.as_slice()) {
                Verdict::Done => settle_all(work_dir.as_path(), &members, DONE),
                Verdict::Failed(text) => {
                    settle_all(work_dir.as_path(), &members, FAILED);
                    if let Some(newest) = members.last() {
                        let notice: String = push_failed_notice(newest.item.oid.as_str(), text.as_str());
                        let _noticed = write_notice(common_dir, newest.id.as_str(), notice.as_str());
                    }
                }
            }
        }
        let mut upkept: Vec<String> = Vec::new();
        for item in upkeep {
            if !upkept.contains(&item.item.repository) {
                upkept.push(item.item.repository.clone());
                if let Verdict::Failed(reason) = handlers.maintenance(&item.item) {
                    eprintln!("cli-git-service: automatic maintenance did not run: {reason}");
                }
            }
            settle_all(work_dir.as_path(), std::slice::from_ref(&item), DONE);
        }
        // Reviews go last: a push is the person's backup, a review is a shadow record, and each is bounded by its budget.
        for item in reviews {
            match handlers.review(&item.item) {
                Verdict::Done => settle_all(work_dir.as_path(), std::slice::from_ref(&item), DONE),
                Verdict::Failed(reason) => {
                    eprintln!("cli-git-service: the review of {} failed: {reason}", item.item.oid);
                    settle_all(work_dir.as_path(), std::slice::from_ref(&item), FAILED);
                }
            }
        }
    }
}

/// What:
///  Runs one round of wakes: keeps each wake's review credential in memory, drains each named repository once, and
///  forgets the credentials when the repository is done.
/// Why:
///   The credential is held only for the reviews of the round that brought it; a service that restarts before the round
///  runs has none, and its reviews record the plain "no key" outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [dir, key] of wakes) { remember(dir, key); await drain(dir); forget(dir) }
/// ```
pub fn run_round(wakes: Vec<Wake>, handlers: &GitHandlers, now: SystemTime) {
    let mut directories: Vec<PathBuf> = Vec::new();
    for (directory, review_key) in wakes {
        if let Some(key) = review_key {
            handlers.remember_review_key(directory.as_path(), key);
        }
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }
    for directory in directories {
        drain(directory.as_path(), handlers, now);
        handlers.forget_review_key(directory.as_path());
    }
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
    pub fn remember_review_key(&self, common_dir: &Path, key: ReviewKey) {
        self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(common_dir.to_path_buf(), key);
    }

    /// Drop the credential of this repository.
    pub fn forget_review_key(&self, common_dir: &Path) {
        self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(common_dir);
    }

    fn review_key_of(&self, common_dir: &Path) -> Option<ReviewKey> {
        return self.review_keys.lock().unwrap_or_else(std::sync::PoisonError::into_inner).get(common_dir).cloned();
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
        return match push_once(&git, newest.oid.as_str(), &mut output) {
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
