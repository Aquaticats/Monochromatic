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
/// What the retry pushes again.
use super::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable that sets, in whole seconds, the pause before the first new try of a stopped push.
pub const RETRY_VARIABLE: &str = "CLI_GIT_PUSH_RETRY_SECONDS";

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
    /// The item to push again.
    pub item: WorkItem,
    /// The number of this try.
    pub attempt: u32,
}

/// What:
///  The branches whose push is waiting for a new try, with the first pause.
/// Why:
///   The work thread alone reads and writes it, so it is a plain cell and nothing waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushRetries { schedule(); forget(); takeDue(); nextWait(); }
/// ```
#[derive(Debug)]
pub struct PushRetries {
    /// The pause before the first new try.
    first: Duration,
    /// The waiting pushes, one per branch of a Git directory.
    waiting: std::cell::RefCell<Vec<Waiting>>,
}

/// Methods over the list.
impl PushRetries {
    /// A list whose first pause is the variable's value or the default.
    pub fn from_environment() -> Self {
        return Self::with_first_pause(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()));
    }

    /// A list with this first pause.
    pub fn with_first_pause(first: Duration) -> Self {
        return Self { first, waiting: std::cell::RefCell::new(Vec::new()) };
    }

    /// Plan try number `attempt` of the branch of `item`, replacing any plan for the same branch; gives the pause.
    pub fn schedule(&self, git_dir: &Path, item: &WorkItem, attempt: u32, now: Instant) -> Duration {
        let pause: Duration = pause_before(attempt, self.first);
        self.forget(git_dir, item.branch_ref.as_str());
        self.waiting.borrow_mut().push(Waiting { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due: now + pause });
        return pause;
    }

    /// Drop the plan of a branch: its push landed, or the person must act.
    pub fn forget(&self, git_dir: &Path, branch_ref: &str) {
        self.waiting.borrow_mut().retain(|entry| { return !(entry.git_dir == git_dir && entry.item.branch_ref == branch_ref); });
    }

    /// Remove and return the plans of this Git directory that are due at `now`.
    pub fn take_due(&self, git_dir: &Path, now: Instant) -> Vec<Due> {
        let mut list = self.waiting.borrow_mut();
        let (due, kept): (Vec<Waiting>, Vec<Waiting>) = std::mem::take(&mut *list).into_iter().partition(|entry| { return entry.git_dir == git_dir && entry.due <= now; });
        *list = kept;
        return due.into_iter().map(|entry| { return Due { item: entry.item, attempt: entry.attempt }; }).collect();
    }

    /// How long until the first plan of this Git directory is due (zero for one already due), or nothing when none waits.
    pub fn next_wait(&self, git_dir: &Path, now: Instant) -> Option<Duration> {
        return self.waiting.borrow().iter().filter(|entry| { return entry.git_dir == git_dir; }).map(|entry| { return entry.due.saturating_duration_since(now); }).min();
    }
}

/// The pauses and the list are controlled in the test module.
#[cfg(test)]
#[path = "push_retry_tests.rs"]
mod tests;
