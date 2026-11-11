//! What:
//!  The service's own retry of a push it stopped or that failed:
//! the pause before each new try,
//! and the list of branches waiting
//!  for one.
//! Why:
//!  The owner's rule is that auto-push must land (`SPEC.md`,
//! "Merges and commits never fail").
//! A push stopped by the stall
//!  window,
//! by the network,
//! by an LFS server error or by a reference lock passes by itself,
//! so the service tries again on its own,
//!  with pauses that grow,
//! until the push lands or a conflict needs the person.
//! Nothing waits for the next commit:
//! the work
//!  loop is woken when the pause is over.
//! The list is owned by the service's work loop and kept on disk beside the instance's
//!  journal (`push_retry_store.rs`),
//! so a restarted service rebuilds it;
//! the person's failure record is not the plan,
//! since the
//!  person's next command shows it and removes it.
//! The workers of a round write and remove the plan's own files,
//! each on the thread that decided its branch's push needs a new try or none,
//! and the loop only changes the list it reads,
//! so a round that reaches storage late cannot undo a later round's statement.
//! A start whose read of the journal gave no answer keeps the list's directory,
//! writes its own tries there,
//! and reads the journal again after a pause.
//! What a failed push's own words mean (whether it is final,
//! whether it is counted)
//! and the words the person reads are in `push_retry_failure.rs`;
//! the first of Git's own lines that gives a reason for a failure is in `push_retry_cause_line.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pause = retries.schedule(item, attempt); /* later */ for (const due of retries.takeDue(gitDir)) await push(due);
//! ```

/// The positive-seconds reading shared by the wait limits.
use crate::push_deadline_and_patience_settings::positive_seconds;
/// The plan on disk:
/// what a start reads,
/// and what the workers write.
use crate::push_retry_store::LoadedPlans;
/// What the retry pushes again.
use crate::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable that sets,
/// in whole seconds,
/// the pause before the first new try of a stopped push.
pub const RETRY_VARIABLE: &str = "CLI_GIT_PUSH_RETRY_SECONDS";

/// Directory,
/// under the service instance's journal,
/// that holds the plan of the pushes waiting for a new try,
/// so that a restarted
/// service keeps trying them.
pub const PLAN_DIRECTORY: &str = "push-retries";

/// The pause before the first new try when nothing configures it.
pub const FIRST_PAUSE: Duration = Duration::from_secs(10);

/// The longest pause between two tries:
/// a push that keeps failing is tried about this often for as long as it fails.
pub const LONGEST_PAUSE: Duration = Duration::from_secs(10 * 60);

/// What:
///  The first pause for a value of the variable.
/// Why:
///   A missing,
/// zero or malformed value falls back to the default,
/// never to a tight loop.
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
///  The pause before try number `attempt` (the first new try is number one):
/// the first pause,
/// doubled each time,
/// never past
///  the longest.
/// Why:
///   Short pauses mend a stall quickly;
/// longer ones keep a host that is offline for an hour from being asked every few seconds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.min(first * 2 ** (attempt - 1), LONGEST)
/// ```
pub fn pause_before(attempt: u32, first: Duration) -> Duration {
    let doublings: u32 = attempt.saturating_sub(1).min(30);
    return first.saturating_mul(1_u32 << doublings).min(LONGEST_PAUSE);
}

/// One branch waiting for a new try.
#[derive(Clone, Debug)]
struct Waiting {
    /// The Git directory of the worktree whose service pushes.
    git_dir: PathBuf,
    /// The newest push item of the branch;
    /// its branch is what is pushed again.
    item: WorkItem,
    /// The number of the try that is due.
    attempt: u32,
    /// When it is due.
    due: Instant,
}

/// The waiting plans without the one for `branch_ref` of `git_dir`,
/// in their order.
fn without_branch(waiting: Vec<Waiting>, git_dir: &Path, branch_ref: &str) -> Vec<Waiting> {
    let mut kept: Vec<Waiting> = Vec::new();
    for entry in waiting {
        if !(entry.git_dir == git_dir && entry.item.branch_ref == branch_ref) {
            kept.push(entry);
        }
    }
    return kept;
}

/// A push that is due again.
#[derive(Clone, Debug)]
pub struct Due {
    /// The Git directory whose service pushes;
    /// the plan is keyed by it.
    pub git_dir: PathBuf,
    /// The item to push again.
    pub item: WorkItem,
    /// The number of this try.
    pub attempt: u32,
}

/// What:
///  What the worker of one directory is handed about retries:
/// the first pause (nothing when its handlers do not retry) and the
///  pushes of that directory that are due again.
/// Why:
///   The plan belongs to the service loop alone.
/// A worker never reads it:
/// it gets its own copy of what is due for its directory,
///  as an owned value inside its request,
/// so no worker shares the plan with another and nothing waits for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Retrying = { first?: number; due: Due[] };
/// ```
#[derive(Clone, Debug, Default)]
pub struct Retrying {
    /// The pause before the first new try;
    /// nothing when a retryable failure is to be reported as a failed push.
    pub first: Option<Duration>,
    /// The pushes due again,
    /// each naming the directory it belongs to.
    pub due: Vec<Due>,
    /// The directory a plan is kept in,
    /// so that a drain can keep one on disk before it tells the person a retry is coming;
    /// nothing for a plan kept in memory only.
    pub plans: Option<PathBuf>,
}

/// Methods over a request.
impl Retrying {
    /// A request for a drain that keeps no plan:
    /// a push that could pass is reported as failed,
    /// as it is for handlers that do not retry.
    pub fn none() -> Self {
        return Self { first: None, due: Vec::new(), plans: None };
    }
}

/// What:
///  One change a worker asks of the plan:
/// plan a try,
/// or drop the plan of a branch.
/// Why:
///   A worker cannot change the plan (it does not hold it),
/// so it returns what it did and the service loop applies the changes
///  on its own thread,
/// in the order they were made,
/// which is how the last plan of a branch wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RetryChange = { plan: { gitDir; item; attempt; due } } | { forget: { gitDir; branchRef; onlyWhileUnreachable; taken } };
/// ```
#[derive(Clone, Debug)]
pub enum RetryChange {
    /// Try the branch of `item` again as try number `attempt`,
    /// at `due`,
    /// replacing any plan of the same branch.
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
    /// The branch needs no new try:
    /// its push landed,
    /// or the person must act.
    Forget {
        /// The Git directory whose service pushes.
        git_dir: PathBuf,
        /// The branch reference.
        branch_ref: String,
        /// Whether this forget rests on a reading that can go stale:
        /// the proof that the repository is gone,
        /// whose plans are dropped only while it still is.
        only_while_unreachable: bool,
        /// The plan the reading ended,
        /// which the round took out of the list before the reading,
        /// so that a repository that is back when this change lands keeps it.
        /// Nothing for a forget that is the final word.
        taken: Option<Due>,
    },
}

// The number of changes a control lets `apply` apply before it stops, as a killed service leaves the round half applied;
// nothing outside a control.
#[cfg(test)]
thread_local! {
    /// The bound of the control running on this thread.
    static STOP_AFTER: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

/// Stops `apply` after `changes` changes on this thread until the guard drops.
#[cfg(test)]
pub(crate) fn stop_after_in_a_control(changes: usize) -> StopAfterGuard {
    STOP_AFTER.with(|slot| { slot.set(Some(changes)); });
    return StopAfterGuard;
}

/// Ends the control's bound when dropped.
#[cfg(test)]
pub(crate) struct StopAfterGuard;

#[cfg(test)]
impl Drop for StopAfterGuard {
    fn drop(&mut self) {
        STOP_AFTER.with(|slot| { slot.set(None); });
    }
}

/// Whether a control's bound is reached after `applied` changes.
#[cfg(test)]
fn stopped_after(applied: usize) -> bool {
    return STOP_AFTER.with(|slot| { return slot.get().is_some_and(|bound| { return applied >= bound; }); });
}

/// What:
///  The branches whose push is waiting for a new try,
/// with the first pause.
/// Why:
///   The service loop alone owns it and changes it through `&mut self`,
/// so it needs no cell and no lock;
/// workers get what is due
///  as owned values (`Retrying`) and hand their changes back (`RetryChange`).
/// Only the owner of the list changes it,
/// so a change needs no lock either;
///  the workers never hold it.
/// The list writes nothing:
/// the file of a plan is written or removed by the worker that decided it,
/// on the thread that decided it
///  (`Replanning::keep_the_plan` and `Replanning::forget`),
/// so no round replays a stale statement onto the journal,
///  while the owner of the list applies the same changes to its memory here.
/// A read that gave no answer is made again after a pause,
///  because the plan's files stand in the journal
/// and nothing else would try the pushes they name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushRetries { schedule(); forget(); takeDue(); nextWait(); apply(changes); restore(); }
/// ```
#[derive(Clone, Debug)]
pub struct PushRetries {
    /// The pause before the first new try.
    first: Duration,
    /// The waiting pushes,
    /// one per branch of a Git directory.
    waiting: Vec<Waiting>,
    /// The directory the plan's files are read from,
    /// so that a restarted service rebuilds the list;
    /// nothing for a list kept in memory only.
    /// The workers are told the same directory in their request,
    /// and they are the ones that write there.
    store: Option<PathBuf>,
    /// Whether the journal holds plans this process has not read,
    /// so that the read is made again after a pause.
    unread: bool,
    /// How many reads in a row have failed,
    /// for the growth of the pause.
    read_failures: u32,
}

/// Methods over the list.
impl PushRetries {
    /// A list whose first pause is the variable's value or the default.
    pub fn from_environment() -> Self {
        return Self::with_first_pause(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()));
    }

    /// A list with this first pause.
    pub fn with_first_pause(first: Duration) -> Self {
        return Self { first, waiting: Vec::new(), store: None, unread: false, read_failures: 0 };
    }

    /// What:
    ///  A list for a service that starts now,
    /// whose plan is kept in `directory`:
    /// the plan the service that stopped before this one
    ///  left there is rebuilt,
    /// each branch due after its pause counted from now,
    /// and a journal that could not be read is read again after a pause.
    /// Why:
    ///   A pause that was running when the service stopped starts again,
    /// so a service restarted in a loop does not push every few
    ///  seconds;
    /// the try number is kept,
    /// so the pauses go on growing.
    ///  A read that met a file it could not read leaves pushes it did not see,
    /// and their plans stand in the journal:
    /// the directory is read again (`restore`),
    /// so the service does not run its whole life without the tries an earlier one promised.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const retries = new PushRetries(first, dir); const read = await loadReporting(dir); for (const saved of read.plans) retries.plan(saved, now + pauseBefore(saved.attempt, first)); retries.unread = read.unread;
    /// ```
    pub fn restored(first: Duration, directory: &Path) -> Self {
        let mut list: Self = Self { first, waiting: Vec::new(), store: Some(directory.to_path_buf()), unread: false, read_failures: 0 };
        let read: LoadedPlans = crate::push_retry_store::load_reporting(directory);
        let now: Instant = Instant::now();
        for saved in read.plans {
            list.waiting.push(Waiting { git_dir: saved.git_dir, item: saved.item, attempt: saved.attempt, due: now + pause_before(saved.attempt, first) });
        }
        list.unread = read.unread;
        return list;
    }

    /// A list for a service that starts now,
    /// with the first pause the variable gives or the default.
    pub fn restored_from_environment(directory: &Path) -> Self {
        return Self::restored(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()), directory);
    }

    /// What:
    ///  A list for a service that starts now without having read its journal:
    /// nothing is known about the pushes an earlier service left,
    /// the list knows where they are kept,
    /// and the read is made again after a pause.
    /// Why:
    ///   The read of a journal that stopped answering is bounded,
    /// so a service starts without it rather than never starting;
    /// the plan's files stand there all the same,
    /// so the service starts with what it can do now
    /// and asks again,
    /// instead of leaving those pushes to a start that nothing requires to arrive.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new PushRetries(first, dir, { unread: true, failures: 1 })
    /// ```
    pub fn unread(first: Duration, directory: &Path) -> Self {
        return Self { first, waiting: Vec::new(), store: Some(directory.to_path_buf()), unread: true, read_failures: 1 };
    }

    /// `unread` with the first pause the variable gives or the default.
    pub fn unread_from_environment(directory: &Path) -> Self {
        return Self::unread(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()), directory);
    }

    /// The pause before the first new try.
    pub fn first(&self) -> Duration {
        return self.first;
    }

    /// The directory the plan's files are kept in,
    /// or nothing for a list kept in memory only.
    pub fn plans(&self) -> Option<&Path> {
        return self.store.as_deref();
    }

    /// Whether the journal holds plans this process has not read,
    /// so that the loop keeps a wake for reading it again.
    pub fn needs_restore(&self) -> bool {
        return self.unread;
    }

    /// How long to wait before the next read:
    /// the first pause,
    /// doubled for each read that failed in a row,
    /// never past the longest pause between two tries.
    pub fn restore_pause(&self) -> Duration {
        return pause_before(self.read_failures.max(1), self.first);
    }

    /// What:
    ///  Reads the journal again and plans what it holds that this process does not,
    /// each branch due after its pause counted from now;
    /// gives the Git directories this made known.
    /// Why:
    ///   The read a service makes as it starts has a ceiling,
    /// and a journal that gave no answer then may answer now;
    /// the pushes it names are the ones an earlier service promised to try,
    /// so nothing else would try them.
    /// A plan this process already holds is newer than any file,
    /// because this process wrote nothing to the journal since that plan,
    ///  so it is never replaced by a read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const found = retries.restore(); for (const dir of found) await drain(dir);
    /// ```
    pub fn restore(&mut self) -> Vec<PathBuf> {
        if !self.unread {
            return Vec::new();
        }
        let Some(directory) = self.store.clone() else {
            self.unread = false;
            return Vec::new();
        };
        let read: LoadedPlans = crate::push_retry_store::load_reporting(directory.as_path());
        let now: Instant = Instant::now();
        let mut found: Vec<PathBuf> = Vec::new();
        for saved in read.plans {
            if self.holds(saved.git_dir.as_path(), saved.item.branch_ref.as_str()) {
                continue;
            }
            if !found.contains(&saved.git_dir) {
                found.push(saved.git_dir.clone());
            }
            self.waiting.push(Waiting { git_dir: saved.git_dir, item: saved.item, attempt: saved.attempt, due: now + pause_before(saved.attempt, self.first) });
        }
        self.unread = read.unread;
        self.read_failures = if self.unread { self.read_failures.saturating_add(1) } else { 0 };
        return found;
    }

    /// Whether the list already plans a try of this branch of this Git directory.
    fn holds(&self, git_dir: &Path, branch_ref: &str) -> bool {
        for entry in &self.waiting {
            if entry.git_dir == git_dir && entry.item.branch_ref == branch_ref {
                return true;
            }
        }
        return false;
    }

    /// Plan try number `attempt` of the branch of `item`,
    /// replacing any plan for the same branch;
    /// gives the pause.
    /// Writes nothing:
    /// the worker that decided the try keeps the file (`Replanning::keep_the_plan`).
    pub fn schedule(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, now: Instant) -> Duration {
        let pause: Duration = pause_before(attempt, self.first);
        self.plan(git_dir, item, attempt, now + pause);
        return pause;
    }

    /// Plan try number `attempt` of the branch of `item` for `due`,
    /// replacing any plan for the same branch.
    fn plan(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, due: Instant) {
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, item.branch_ref.as_str());
        self.waiting.push(Waiting { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due });
    }

    /// Drop the plan of a branch:
    /// its push landed,
    /// or the person must act.
    /// Writes nothing:
    /// the worker that decided it removes the file (`Replanning::forget`).
    pub fn forget(&mut self, git_dir: &Path, branch_ref: &str) {
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, branch_ref);
    }

    /// What:
    ///  Apply what the workers of a round asked to the list this process reads,
    /// in the order they asked it.
    /// Why:
    ///   The workers told the journal what each change is when they made it:
    /// the file of a plan is published before the notice that promises the try,
    /// and the file of a branch that needs no try goes with the record that says so.
    /// So this applies the same changes to the list and touches no file:
    ///  a change replayed here late
    /// cannot take away a plan a later round wrote,
    /// and a change that never reached a thread here
    ///  leaves the journal holding what the worker itself wrote.
    ///  A forget that rests on a reading of the filesystem,
    /// the proof that a repository is gone,
    /// is applied only while that directory is still missing:
    ///  a repository renamed back by now is reachable,
    /// and its plan is the only scheduled backup of its commits.
    /// A forget whose reading went stale puts the plan that reading ended back,
    ///  because the round took it out of this list before it drained the directory:
    /// without that,
    ///  the running service keeps no plan and no wake for a repository that is there again,
    /// and the promised
    ///  push waits for a service restart.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const change of changes) {
    ///   if (change.forget && change.onlyWhileUnreachable && isDirectory(change.gitDir)) {
    ///     plan(change.forget.taken);
    ///     continue;
    ///   }
    ///   change.forget ? forget(change) : plan(change);
    /// }
    /// ```
    pub fn apply(&mut self, changes: Vec<RetryChange>) {
        // How many of the changes this call has applied, for the control that kills a service inside it; nothing outside one.
        #[cfg(test)]
        let mut applied: usize = 0;
        for change in changes {
            // A service killed here has applied the changes before this one and nothing after it.
            #[cfg(test)]
            {
                if stopped_after(applied) {
                    return;
                }
                applied += 1;
            }
            match change {
                RetryChange::Plan { git_dir, item, attempt, due } => self.plan(git_dir.as_path(), &item, attempt, due),
                RetryChange::Forget { git_dir, branch_ref, only_while_unreachable, taken } => {
                    // The plan a proof of a missing repository ends was read before this change was applied, and a repository
                    //  renamed back is there again: a plan that may still run is not dropped on a stale reading.
                    if only_while_unreachable && git_dir.is_dir() {
                        // The plan this reading ended is the one the round took out of the list before its drain
                        //  (`service_work_run_round.rs`, `plan_the_round`), so putting it back is what leaves the running
                        //  service with the push it promised; the try that plan was for never ran, so it is due now.
                        //  Only a drain of an unreachable directory asks for this kind of forget, and such a drain runs no
                        //  push and asks nothing else of that branch, so the plans of a branch here are this one's alone.
                        if let Some(plan) = taken {
                            self.plan(git_dir.as_path(), &plan.item, plan.attempt, Instant::now());
                        }
                        continue;
                    }
                    self.forget(git_dir.as_path(), branch_ref.as_str());
                }
            }
        }
    }

    /// Remove and return the plans of this Git directory that are due at `now`.
    pub fn take_due(&mut self, git_dir: &Path, now: Instant) -> Vec<Due> {
        let mut kept: Vec<Waiting> = Vec::new();
        let mut due: Vec<Due> = Vec::new();
        for entry in std::mem::take(&mut self.waiting) {
            if entry.git_dir == git_dir && entry.due <= now {
                due.push(Due { git_dir: entry.git_dir, item: entry.item, attempt: entry.attempt });
            } else {
                kept.push(entry);
            }
        }
        self.waiting = kept;
        return due;
    }

    /// How long until the first plan of this Git directory is due (zero for one already due),
    /// or nothing when none waits.
    pub fn next_wait(&self, git_dir: &Path, now: Instant) -> Option<Duration> {
        let mut soonest: Option<Duration> = None;
        for entry in &self.waiting {
            if entry.git_dir != git_dir {
                continue;
            }
            let wait: Duration = entry.due.saturating_duration_since(now);
            soonest = Some(match soonest {
                Some(earlier) => earlier.min(wait),
                None => wait,
            });
        }
        return soonest;
    }

    /// The Git directories that have a plan,
    /// each once.
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
