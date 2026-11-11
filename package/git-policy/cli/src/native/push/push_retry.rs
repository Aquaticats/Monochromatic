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
//! A removal the round that asked for it could not write is owed to a later round,
//! and the branch is named until that write happens,
//! so the plan of a push that ended never outlives the round that forgot it
//!  (`UnwrittenForget`).
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
/// The plan on disk.
use crate::push_retry_store::{StoredPlan, load, micros_now, remove_except, save};
/// What the retry pushes again.
use crate::work_item::WorkItem;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Environment variable that sets,
/// in whole seconds,
/// the pause before the first new try of a stopped push.
pub const RETRY_VARIABLE: &str = "CLI_GIT_PUSH_RETRY_SECONDS";

/// What a worker is handed about a branch's new try and what it hands back,
/// in `push_retry_worker_values.rs`.
pub use crate::push_retry_worker_values::{Due, RetryChange, Retrying};
/// The changes one round asked of the plan,
/// read by the loop that applies them.
use crate::push_retry_worker_values::{last_plan_after, planned_again_after};
/// The bound a control puts on `apply`,
/// which a killed service leaves half applied.
#[cfg(test)]
pub(crate) use crate::push_retry_worker_values::stop_after_in_a_control;
#[cfg(test)]
use crate::push_retry_worker_values::stopped_after;

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

/// What:
///  The branches whose push is waiting for a new try,
/// with the first pause.
/// Why:
///   The service loop alone owns it and changes it through `&mut self`,
/// so it needs no cell and no lock;
/// workers get what is due
///  as owned values (`Retrying`) and hand their changes back (`RetryChange`).
/// The list is cloned into a job of the blocking pool when the loop that owns it is a task,
///  because a job may not borrow it;
/// the job hands the changed list back and the clone is dropped,
/// so only one list is ever the
///  owner's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PushRetries { schedule(); forget(); takeDue(); nextWait(); apply(changes); }
/// ```
#[derive(Clone, Debug)]
pub struct PushRetries {
    /// The pause before the first new try.
    first: Duration,
    /// The waiting pushes,
    /// one per branch of a Git directory.
    waiting: Vec<Waiting>,
    /// The directory the plan is kept in,
    /// so that a restarted service rebuilds it;
    /// nothing for a list kept in memory only.
    store: Option<PathBuf>,
    /// Whether a plan an earlier service left has not been read yet.
    /// Why:
    ///   The read of the journal is bounded,
    /// and a filesystem that stopped answering can outlast the ceiling,
    /// so the service starts before its plan is known.
    /// The list keeps the journal's own directory either way,
    /// so a retry planned now is kept for a restart,
    /// and the read is made again (`restore_now`) rather than given up on:
    /// a plan nobody read is a push nobody tries.
    restore_pending: bool,
    /// How many reads of the journal have come to nothing so far,
    /// which is the pause before the next one.
    restore_failures: u32,
    /// The forgets this list applied without writing them,
    /// each with the instant it was asked and the number of writes that came to nothing.
    /// Why:
    ///   A plan job that gives no answer changed the list and not the journal,
    /// and a forget asks for a file to go:
    /// the file the worker wrote is still there,
    /// so a service started after a kill would restore the plan of a push that had landed.
    /// The branch is named here until a round's write removes it,
    /// and the list keeps a wake for it meanwhile (`next_wait`).
    unwritten: Vec<UnwrittenForget>,
}

/// A forget the list applied without writing it.
#[derive(Clone, Debug)]
struct UnwrittenForget {
    /// The Git directory whose service pushes.
    git_dir: PathBuf,
    /// The branch reference its plan ends.
    branch_ref: String,
    /// When the worker asked for the branch's plans to end,
    /// in microseconds since the epoch.
    asked: u64,
    /// How many writes of it have come to nothing,
    /// which is the pause before the next one.
    tries: u32,
}

/// Methods over the list.
impl PushRetries {
    /// A list whose first pause is the variable's value or the default.
    pub fn from_environment() -> Self {
        return Self::with_first_pause(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()));
    }

    /// A list with this first pause.
    pub fn with_first_pause(first: Duration) -> Self {
        return Self { first, waiting: Vec::new(), store: None, restore_pending: false, restore_failures: 0, unwritten: Vec::new() };
    }

    /// What:
    ///  A list for a service that starts now,
    /// whose plan is kept in `directory`:
    /// the plan the service that stopped before this one
    ///  left there is rebuilt,
    /// each branch due after its pause counted from now.
    /// Why:
    ///   A pause that was running when the service stopped starts again,
    /// so a service restarted in a loop does not push every few
    ///  seconds;
    /// the try number is kept,
    /// so the pauses go on growing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const retries = new PushRetries(first, dir); for (const saved of await load(dir)) retries.plan(saved, now + pauseBefore(saved.attempt, first));
    /// ```
    pub fn restored(first: Duration, directory: &Path) -> Self {
        let mut list: Self = Self::unread(first, directory);
        let _found: Vec<PathBuf> = list.restore_now(directory);
        return list;
    }

    /// What:
    ///  A list for a service that starts now,
    /// whose journal is in `directory` and has not been read yet.
    /// Why:
    ///   The read of the journal is bounded,
    /// and a plan or a note that outlasts its ceiling leaves the service starting first;
    /// the journal's own directory is kept,
    /// so a retry planned after that is kept for a restart,
    /// and `next_wait` names the journal for a read that is tried again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const retries = new PushRetries(first, dir, { unread: true })
    /// ```
    pub fn unread(first: Duration, directory: &Path) -> Self {
        return Self {
            first,
            waiting: Vec::new(),
            store: Some(directory.to_path_buf()),
            restore_pending: true,
            restore_failures: 1,
            unwritten: Vec::new(),
        };
    }

    /// A list whose journal has not been read yet,
    /// with the first pause the variable gives or the default.
    pub fn unread_from_environment(directory: &Path) -> Self {
        return Self::unread(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()), directory);
    }

    /// Whether a plan an earlier service left has not been read yet.
    pub fn needs_restore(&self) -> bool {
        return self.restore_pending;
    }

    /// How long until the read of the journal is tried again:
    /// the first pause,
    /// doubled for each read that came to nothing.
    pub fn restore_pause(&self) -> Duration {
        return pause_before(self.restore_failures.max(1), self.first);
    }

    /// What:
    ///  Note that a read of the journal made to learn it gave no answer,
    /// so it is owed again at the next pause.
    /// Why:
    ///   The read is tried again rather than given up on,
    /// and a journal that stopped answering may not be read at the shortest pause for ever:
    /// each attempt leaves its read behind in a job that holds a place of the blocking bound until the journal answers,
    /// so the pause grows as it does for a plan and the attempts thin out.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// retries.restoreCameToNothing(); // the next read is owed at pauseBefore(++failures, first)
    /// ```
    pub fn restore_came_to_nothing(&mut self) {
        self.restore_pending = true;
        self.restore_failures = self.restore_failures.saturating_add(1);
    }

    /// What:
    ///  Read the journal in `directory` into this list
    /// and answer the Git directories it made the list hold.
    /// Why:
    ///   A service that started before its plan could be read makes the read again,
    /// and every plan the files hold is then due after its own pause counted from now.
    /// A branch the list already holds keeps what it has,
    /// which is newer than the file the worker wrote before this read;
    /// a branch whose forget this round owes is left out,
    /// because that removal is the newer statement about it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const found = retries.restoreNow(dir)
    /// ```
    pub fn restore_now(&mut self, directory: &Path) -> Vec<PathBuf> {
        let now: Instant = Instant::now();
        let mut found: Vec<PathBuf> = Vec::new();
        for saved in load(directory) {
            let branch: &str = saved.item.branch_ref.as_str();
            if self.holds_or_owes(saved.git_dir.as_path(), branch) {
                continue;
            }
            let due: Instant = now + pause_before(saved.attempt, self.first);
            if !found.contains(&saved.git_dir) {
                found.push(saved.git_dir.clone());
            }
            self.waiting.push(Waiting { git_dir: saved.git_dir, item: saved.item, attempt: saved.attempt, due });
        }
        self.restore_pending = false;
        self.restore_failures = 0;
        return found;
    }

    /// What:
    ///  Whether this list already holds `branch` of `git_dir`,
    ///  or owes the removal of its plan.
    /// Why:
    ///   A read of the journal must not replace a plan the list already has,
    /// which is newer than the file,
    /// nor one whose removal this round owes,
    /// which is the newer statement about that branch.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// retries.holdsOrOwes(gitDir, branch)
    /// ```
    fn holds_or_owes(&self, git_dir: &Path, branch: &str) -> bool {
        for held in &self.waiting {
            if held.git_dir == git_dir && held.item.branch_ref == branch {
                return true;
            }
        }
        for owed in &self.unwritten {
            if owed.git_dir == git_dir && owed.branch_ref == branch {
                return true;
            }
        }
        return false;
    }

    /// A list for a service that starts now,
    /// with the first pause the variable gives or the default.
    pub fn restored_from_environment(directory: &Path) -> Self {
        return Self::restored(first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()), directory);
    }

    /// The pause before the first new try.
    pub fn first(&self) -> Duration {
        return self.first;
    }

    /// The directory the plan is kept in,
    /// or nothing for a list kept in memory only.
    pub fn plans(&self) -> Option<&Path> {
        return self.store.as_deref();
    }

    /// Plan try number `attempt` of the branch of `item`,
    /// replacing any plan for the same branch;
    /// gives the pause.
    pub fn schedule(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, now: Instant) -> Duration {
        let pause: Duration = pause_before(attempt, self.first);
        self.plan(git_dir, item, attempt, now + pause);
        return pause;
    }

    /// Plan try number `attempt` of the branch of `item` for `due`,
    /// replacing any plan for the same branch.
    fn plan(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, due: Instant) {
        self.plan_writing(git_dir, item, attempt, due, true);
    }

    /// `plan` with the write of the plan's own file given,
    /// so that a round whose job gave no answer changes the list alone.
    fn plan_writing(&mut self, git_dir: &Path, item: &WorkItem, attempt: u32, due: Instant, write: bool) {
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, item.branch_ref.as_str());
        self.waiting.push(Waiting { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt, due });
        if write
            && let Some(directory) = self.store.as_deref()
            && let Err(reason) = save(directory, &StoredPlan { git_dir: git_dir.to_path_buf(), item: item.clone(), attempt })
        {
            crate::service_warn!("the plan to push {:?} again could not be kept for a restart: {reason}", item.branch_ref);
        }
    }

    /// Drop the plan of a branch:
    /// its push landed,
    /// or the person must act.
    pub fn forget(&mut self, git_dir: &Path, branch_ref: &str) {
        self.forget_keeping(git_dir, branch_ref, &[], micros_now());
    }

    /// `forget` for a branch whose plan the rest of this round keeps,
    /// under the file names in `keep`:
    /// those are newer statements than the forget and stay on disk,
    /// as does every plan written at or since `asked`.
    pub fn forget_keeping(&mut self, git_dir: &Path, branch_ref: &str, keep: &[String], asked: u64) {
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, branch_ref);
        if let Some(directory) = self.store.as_deref() {
            remove_except(directory, git_dir, branch_ref, keep, asked);
        }
    }

    /// What:
    ///  Apply what the workers of a round asked,
    /// in the order they asked it.
    /// Why:
    ///   A forget is applied after the round,
    /// so a plan another worker of the same round kept on disk is already there;
    /// its
    ///  file name is known,
    /// so the forget keeps it and the round's own plan stays.
    ///  A plan the same round plans again later is not published at all:
    /// the last plan of a branch is the round's statement,
    ///  and publishing an earlier one would remove the file the last one's worker already wrote,
    /// leaving a forget
    ///  replayed between them nothing to keep
    /// and a restarted service no plan where a notice promised a retry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const [at, change] of changes.entries()) {
    ///   if (change.forget) forgetKeeping(branch, lastPlanName(changes.slice(at + 1)))
    ///   else if (!plannedAgainAfter(changes, at)) plan(change)
    /// }
    /// ```
    pub fn apply(&mut self, changes: Vec<RetryChange>) {
        // A forget an earlier round could not write goes first: it is the older statement about its branch.
        self.write_the_forgets_that_were_left();
        self.apply_changes_writing(changes, true);
    }

    /// What:
    ///  `apply` with the writes of the plan's own files given:
    /// the list changes and the journal is left as it is.
    /// Why:
    ///   A task of the runtime may not write storage itself,
    /// and a job that gave no answer may have written or not,
    /// so the list keeps what the workers asked.
    /// A plan a worker asked for is already on disk,
    /// because the worker writes it before the person is told a retry comes;
    ///  a forget is not,
    /// so the branch is named for a later round's write,
    /// which is what keeps a killed service from restoring the plan of a
    ///  push that had landed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// applyWithoutWriting(changes) // the list changes; every forget is owed to a later round
    /// ```
    pub fn apply_without_writing(&mut self, changes: Vec<RetryChange>) {
        self.apply_changes_writing(changes, false);
    }

    /// What:
    ///  Apply what the workers asked,
    /// in the order they asked it,
    /// writing the plan's own files or not.
    /// Why:
    ///   See `apply` and `apply_without_writing`.
    fn apply_changes_writing(&mut self, changes: Vec<RetryChange>, write: bool) {
        for index in 0..changes.len() {
            // A service killed here has applied the changes before this one and nothing after it.
            #[cfg(test)]
            if stopped_after(index) {
                return;
            }
            match &changes[index] {
                RetryChange::Plan { git_dir, item, attempt, due } => {
                    if planned_again_after(changes.as_slice(), index, git_dir, item.branch_ref.as_str()) {
                        continue;
                    }
                    self.plan_writing(git_dir.as_path(), item, *attempt, *due, write)
                }
                RetryChange::Forget { git_dir, branch_ref, only_while_unreachable, asked } => {
                    // The plan a proof of a missing repository ends was read before this change was applied, and a repository
                    //  renamed back is there again: a plan that may still run is not dropped on a stale reading.
                    if *only_while_unreachable && git_dir.is_dir() {
                        continue;
                    }
                    let keep: Vec<String> = last_plan_after(changes.as_slice(), index, git_dir, branch_ref).into_iter().collect();
                    if write {
                        self.forget_keeping(git_dir.as_path(), branch_ref.as_str(), keep.as_slice(), *asked);
                    } else {
                        self.forget_owed_to_a_later_round(git_dir.as_path(), branch_ref.as_str(), *asked);
                    }
                }
            }
        }
    }

    /// What:
    ///  Drop the branch's plans from the list and name the branch for a later round's write.
    /// Why:
    ///  A round whose job gave no answer may have written the forget or not,
    /// and the list cannot read which;
    /// naming the branch writes the same removal twice at worst,
    /// and leaving it unnamed would keep the plan of a push that ended on disk for a service started after a kill.
    fn forget_owed_to_a_later_round(&mut self, git_dir: &Path, branch_ref: &str, asked: u64) {
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, branch_ref);
        if self.store.is_none() {
            return;
        }
        for owed in &mut self.unwritten {
            if owed.git_dir == git_dir && owed.branch_ref == branch_ref {
                owed.asked = owed.asked.max(asked);
                return;
            }
        }
        self.unwritten.push(UnwrittenForget {
            git_dir: git_dir.to_path_buf(),
            branch_ref: branch_ref.to_owned(),
            asked,
            // The first write of it is due one first pause from now, which is what the loop's wake reads.
            tries: 1,
        });
    }

    /// What:
    ///  Write the removals an earlier round could not write,
    /// and keep the ones whose write came to nothing.
    /// Why:
    ///   A plan file that names its branch and is still there is the one a restarted service would run;
    /// the write of its removal is tried again after the pause `next_wait` names,
    /// so the branch is named until its plan is really gone.
    fn write_the_forgets_that_were_left(&mut self) {
        let Some(directory) = self.store.clone() else {
            self.unwritten.clear();
            return;
        };
        let mut left: Vec<UnwrittenForget> = Vec::new();
        for mut owed in std::mem::take(&mut self.unwritten) {
            if remove_except(directory.as_path(), owed.git_dir.as_path(), owed.branch_ref.as_str(), &[], owed.asked) {
                owed.tries = owed.tries.saturating_add(1);
                left.push(owed);
            }
        }
        self.unwritten = left;
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

    /// How long until the first plan of this Git directory is due (zero for one already due)
    /// or the first removal a round could not write is tried again,
    /// or nothing when neither waits.
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
        // A removal a round could not write is a statement on disk that its own list contradicts, so it is due too, after a
        //  pause that grows with each write that came to nothing.
        for owed in &self.unwritten {
            if owed.git_dir == git_dir {
                let wait: Duration = pause_before(owed.tries.max(1), self.first);
                soonest = Some(match soonest {
                    Some(earlier) => earlier.min(wait),
                    None => wait,
                });
            }
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
