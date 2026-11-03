//! What:
//!  The service's own retry of a push it stopped or that failed: the pause before each new try, the list of branches waiting
//!  for one, and the test for the one failure no retry can mend.
//! Why:
//!  The owner's rule is that auto-push must land (`SPEC.md`, "Merges and commits never fail"). A push stopped by the stall
//!  window, by the ceiling, by the network or by a reference lock passes by itself, so the service tries again on its own,
//!  with pauses that grow, until the push lands or a conflict needs the person. Nothing waits for the next commit: the work
//!  loop is woken when the pause is over. The list lives in the work thread's memory (a restart of the service forgets it, and
//!  the person's failure record, which is on disk, still names the push for the person).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pause = retries.schedule(item, attempt); /* later */ for (const due of retries.takeDue(gitDir)) await push(due);
//! ```

/// The positive-seconds reading shared by the wait limits.
use super::bounded_push::positive_seconds;
/// The plan on disk.
use super::push_retry_store::{StoredPlan, load, remove, save};
/// What the retry pushes again.
use super::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable that sets, in whole seconds, the pause before the first new try of a stopped push.
pub const RETRY_VARIABLE: &str = "CLI_GIT_PUSH_RETRY_SECONDS";

/// Directory, under the service instance's journal, that holds the plan of the pushes waiting for a new try, so that a restarted
/// service keeps trying them.
pub const PLAN_DIRECTORY: &str = "push-retries";

/// The pause before the first new try when nothing configures it.
pub const FIRST_PAUSE: Duration = Duration::from_secs(10);

/// The longest pause between two tries: a push that keeps failing is tried about this often for as long as it fails.
pub const LONGEST_PAUSE: Duration = Duration::from_secs(10 * 60);

/// The words of the reconcile's report of a conflict, the one failure that retrying cannot mend.
pub const CONFLICT_PHRASE: &str = " conflicts in: ";

/// What:
///  The first pause for a value of the variable.
/// Why:
///   A missing, zero or malformed value falls back to the default, never to a tight loop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// positiveSeconds(value) ?? FIRST_PAUSE_MS
/// ```
pub fn first_pause_from(value: Option<&std::ffi::OsStr>) -> Duration {
    let Some(text) = value else { return FIRST_PAUSE };
    return positive_seconds(RETRY_VARIABLE, text).unwrap_or(FIRST_PAUSE).min(LONGEST_PAUSE);
}

/// What:
///  The pause before try number `attempt` (the first new try is number one): the first pause, doubled each time, never past
///  the longest.
/// Why:
///   Short pauses mend a stall quickly; longer ones keep a host that is offline for an hour from being asked every few seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(first * 2 ** (attempt - 1), LONGEST)
/// ```
pub fn pause_before(attempt: u32, first: Duration) -> Duration {
    let doublings: u32 = attempt.saturating_sub(1).min(30);
    return first.saturating_mul(1_u32 << doublings).min(LONGEST_PAUSE);
}

/// Whether a failed push's output says a conflict stopped it, which only the person can resolve.
pub fn needs_the_person(output: &str) -> bool {
    return output.contains(CONFLICT_PHRASE);
}

/// One branch waiting for a new try.
#[derive(Clone, Debug)]
struct Waiting {
    /// The Git directory of the worktree whose service pushes.
    git_dir: PathBuf,
    /// The newest push item of the branch; its branch is what is pushed again.
    item: WorkItem,
    /// The number of the try that is due.
    attempt: u32,
    /// When it is due.
    due: Instant,
}

/// A push that is due again.
#[derive(Clone, Debug)]
pub struct Due {
    /// The Git directory whose service pushes; the plan is keyed by it.
    pub git_dir: PathBuf,
    /// The item to push again.
    pub item: WorkItem,
    /// The number of this try.
    pub attempt: u32,
}

/// What:
///  What the worker of one directory is handed about retries: the first pause (nothing when its handlers do not retry) and the
///  pushes of that directory that are due again.
/// Why:
///   The plan belongs to the service loop alone. A worker never reads it: it gets its own copy of what is due for its directory,
///  as an owned value inside its request, so no worker shares the plan with another and nothing waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Retrying = { first?: number; due: Due[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct Retrying {
    /// The pause before the first new try; nothing when a retryable failure is to be reported as a failed push.
    pub first: Option<Duration>,
    /// The pushes due again, each naming the directory it belongs to.
    pub due: Vec<Due>,
}

/// Methods over a request.
impl Retrying {
    /// A request for a drain that keeps no plan: a push that could pass is reported as failed, as it is for handlers that do not retry.
    pub fn none() -> Self {
        return Self { first: None, due: Vec::new() };
    }
}

/// What:
///  One change a worker asks of the plan: plan a try, or drop the plan of a branch.
/// Why:
///   A worker cannot change the plan (it does not hold it), so it returns what it did and the service loop applies the changes
///  on its own thread, in the order they were made, which is how the last plan of a branch wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RetryChange = { plan: { gitDir; item; attempt; due } } | { forget: { gitDir; branchRef } };
/// ```
#[derive(Clone, Debug)]
pub enum RetryChange {
    /// Try the branch of `item` again as try number `attempt`, at `due`, replacing any plan of the same branch.
    Plan {
        /// The Git directory whose service pushes.
        git_dir: PathBuf,
        /// The newest push item of the branch.
        item: WorkItem,
        /// The number of the try that is planned.
        attempt: u32,
        /// When it is due.
        due: Instant,
    },
    /// The branch needs no new try: its push landed, or the person must act.
    Forget {
        /// The Git directory whose service pushes.
        git_dir: PathBuf,
        /// The branch reference.
        branch_ref: String,
    },
}

/// What:
///  The branches whose push is waiting for a new try, with the first pause.
/// Why:
///   The service loop alone owns it and changes it through `&mut self`, so it needs no cell and no lock; workers get what is due
///  as owned values (`Retrying`) and hand their changes back (`RetryChange`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushRetries { schedule(); forget(); takeDue(); nextWait(); apply(changes); }
/// ```
#[derive(Debug)]
pub struct PushRetries {
    /// The pause before the first new try.
    first: Duration,
    /// The waiting pushes, one per branch of a Git directory.
    waiting: Vec<Waiting>,
    /// The directory the plan is kept in, so that a restarted service rebuilds it; nothing for a list kept in memory only.
    store: Option<PathBuf>,
}

/// Methods over the list.
impl PushRetries {
    /// A list whose first pause is the variable's value or the default.
    pub fn from_environment() -> Self {
        return Self::with_first_pause(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()));
    }

    /// A list with this first pause.
    pub fn with_first_pause(first: Duration) -> Self {
        return Self { first, waiting: Vec::new(), store: None };
    }

    /// What:
    ///  A list for a service that starts now, whose plan is kept in `directory`: the plan the service that stopped before this one
    ///  left there is rebuilt, each branch due after its pause counted from now.
    /// Why:
    ///   A pause that was running when the service stopped starts again, so a service restarted in a loop does not push every few
    ///  seconds; the try number is kept, so the pauses go on growing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const retries = new PushRetries(first, dir); for (const saved of await load(dir)) retries.plan(saved, now + pauseBefore(saved.attempt, first));
    /// ```
    pub fn restored(first: Duration, directory: &Path) -> Self {
        let mut list: Self = Self { first, waiting: Vec::new(), store: Some(directory.to_path_buf()) };
        let now: Instant = Instant::now();
        for saved in load(directory) {
            list.waiting.push(Waiting { git_dir: saved.git_dir, item: saved.item, attempt: saved.attempt, due: now + pause_before(saved.attempt, first) });
        }
        return list;
    }

    /// A list for a service that starts now, with the first pause the variable gives or the default.
    pub fn restored_from_environment(directory: &Path) -> Self {
        return Self::restored(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()), directory);
    }

    /// The pause before the first new try.
    pub fn first(&self) -> Duration {
        return self.first;
    }

    /// Plan try number `attempt` of the branch of `item`, replacing any plan for the same branch; gives the pause.
    pub fn schedule(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, now: Instant) -> Duration {
        let pause: Duration = pause_before(attempt, self.first);
        self.plan(git_dir, item, attempt, now + pause);
        return pause;
    }

    /// Plan try number `attempt` of the branch of `item` for `due`, replacing any plan for the same branch.
    fn plan(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, due: Instant) {
        self.waiting.retain(|entry| { return !(entry.git_dir == git_dir && entry.item.branch_ref == item.branch_ref); });
        self.waiting.push(Waiting { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due });
        if let Some(directory) = self.store.as_deref()
            && let Err(reason) = save(directory, &StoredPlan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt })
        {
            crate::service_warn!("the plan to push {:?} again could not be kept for a restart: {reason}", item.branch_ref);
        }
    }

    /// Drop the plan of a branch: its push landed, or the person must act.
    pub fn forget(&mut self, git_dir: &Path, branch_ref: &str) {
        self.waiting.retain(|entry| { return !(entry.git_dir == git_dir && entry.item.branch_ref == branch_ref); });
        if let Some(directory) = self.store.as_deref() {
            remove(directory, git_dir, branch_ref);
        }
    }

    /// Apply what the workers of a round asked, in the order they asked it.
    pub fn apply(&mut self, changes: Vec<RetryChange>) {
        for change in changes {
            match change {
                RetryChange::Plan { git_dir, item, attempt, due } => self.plan(git_dir.as_path(), &item, attempt, due),
                RetryChange::Forget { git_dir, branch_ref } => self.forget(git_dir.as_path(), branch_ref.as_str()),
            }
        }
    }

    /// Remove and return the plans of this Git directory that are due at `now`.
    pub fn take_due(&mut self, git_dir: &Path, now: Instant) -> Vec<Due> {
        let (due, kept): (Vec<Waiting>, Vec<Waiting>) = std::mem::take(&mut self.waiting).into_iter().partition(|entry| { return entry.git_dir == git_dir && entry.due <= now; });
        self.waiting = kept;
        return due.into_iter().map(|entry| { return Due { git_dir: entry.git_dir, item: entry.item, attempt: entry.attempt }; }).collect();
    }

    /// How long until the first plan of this Git directory is due (zero for one already due), or nothing when none waits.
    pub fn next_wait(&self, git_dir: &Path, now: Instant) -> Option<Duration> {
        return self.waiting.iter().filter(|entry| { return entry.git_dir == git_dir; }).map(|entry| { return entry.due.saturating_duration_since(now); }).min();
    }

    /// The Git directories that have a plan, each once.
    pub fn directories(&self) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = Vec::new();
        for entry in &self.waiting {
            if !found.contains(&entry.git_dir) {
                found.push(entry.git_dir.clone());
            }
        }
        return found;
    }
}

/// The pauses and the list are controlled in the test module.
#[cfg(test)]
#[path = "push_retry_tests.rs"]
mod tests;
