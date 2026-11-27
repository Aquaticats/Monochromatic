//! What:
//!  What the workers of a round asked of the plan,
//!  applied to the list the service loop reads:
//!  a try planned,
//!  a branch forgotten,
//!  and the tries a round takes as due.
//! Why:
//!   The workers told the journal what each change is when they made it,
//!  so this applies the same changes to the list and touches no file:
//!  a change replayed here late cannot take away a plan a later round wrote.
//!  A forget that rests on a reading of the filesystem is applied only while that reading holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! retries.apply(changes); const due = retries.takeDue(gitDir, now); retries.nextWait(gitDir, now);
//! ```

/// Import the list this file's own `impl` block extends.
use super::push_retry::PushRetries;
/// Import the seam a control kills a service through,
/// which only a build with the controls has.
#[cfg(test)]
use super::push_retry_waiting_branches::stopped_after;
/// Import the shapes the changes and the tries are written in,
/// the plan the list holds,
/// and the seam a control kills a service through.
use super::push_retry_waiting_branches::{
    Due,
    RetryChange,
    Waiting,
    without_branch,
};
/// Import the push item a planned try pushes again.
use crate::work_item::WorkItem;
/// Import the borrowed and owned path types a plan names its Git directory with.
use std::path::{
    Path,
    PathBuf,
};
/// Import the pause a due plan is waited by,
/// and the clock reading a plan is due at.
use std::time::{
    Duration,
    Instant,
};

/// Methods over the list.
impl PushRetries {
    /// What:
    ///  Plan try number `attempt` of the branch of `item` for `due`,
    ///  replacing any plan for the same branch,
    ///  and give up the values of the record a replaced plan carried,
    ///  which `item` does not name.
    /// Why:
    ///   A plan is the one thing that runs a retryable push's record again
    ///  (`service_push_programs_held_in_memory`),
    ///  so a plan this one replaces leaves a record that can never run,
    ///  and a value that can hold a password may not outlive it here.
    /// A plan replaced by one of the same record keeps the values:
    ///  that is the next try of the same push,
    ///  and the plan a stale proof of a gone repository puts back.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const replaced = waiting.filter((entry) => entry.item.branchRef === item.branchRef && entry.item.id !== item.id());
    /// const kept = waiting.filter((entry) => entry.item.branchRef !== item.branchRef); kept.push({ item, attempt, due });
    /// replaced.forEach((entry) => forgetPrograms(entry.item.gitDir, entry.item.id()));
    /// ```
    pub(super) fn plan(
        &mut self,
        git_dir: &Path,
        item: &WorkItem,
        attempt: u32,
        due: Instant,
    ) {
        let id: String = item.id();
        let replaced: Vec<String> = self
            .waiting
            .iter()
            .filter(|entry| {
                return entry.git_dir == git_dir
                    && entry.item.branch_ref == item.branch_ref
                    && entry.item.id() != id;
            })
            .map(|entry| {
                return entry.item.id();
            })
            .collect();
        self.waiting = without_branch(
            std::mem::take(&mut self.waiting),
            git_dir,
            item.branch_ref.as_str(),
        );
        self.waiting.push(Waiting {
            git_dir: git_dir.to_path_buf(),
            item: item.clone(),
            attempt,
            due,
        });
        for gone in replaced {
            crate::service_push_programs_held_in_memory::forget(git_dir, gone.as_str());
        }
    }

    /// What:
    ///  Drop the plan of a branch:
    ///  its push landed,
    ///  or the person must act,
    ///  and give up the values of every record that plan carried.
    /// Why:
    ///   A plan is the one thing that keeps a retryable push's values (`service_push_programs_held_in_memory`),
    ///  and a later push of the branch that lands ends such a plan just as a try that fails for good does;
    ///  the record a dropped plan carried can never run again,
    ///  so a value that can hold a password may not outlive it in this service's memory.
    /// Writes nothing:
    /// the worker that decided it removes the file (`Replanning::forget`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// for (const entry of waiting) { if (sameBranch(entry)) forgetPrograms(entry.item.gitDir, entry.item.id()); }
    /// ```
    pub fn forget(
        &mut self,
        git_dir: &Path,
        branch_ref: &str,
    ) {
        let ended: Vec<String> = self
            .waiting
            .iter()
            .filter(|entry| {
                return entry.git_dir == git_dir && entry.item.branch_ref == branch_ref;
            })
            .map(|entry| {
                return entry.item.id();
            })
            .collect();
        self.waiting = without_branch(std::mem::take(&mut self.waiting), git_dir, branch_ref);
        for item in ended {
            crate::service_push_programs_held_in_memory::forget(git_dir, item.as_str());
        }
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
    pub fn apply(
        &mut self,
        changes: Vec<RetryChange>,
    ) {
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
                RetryChange::Plan {
                    git_dir,
                    item,
                    attempt,
                    due,
                } => self.plan(git_dir.as_path(), &item, attempt, due),
                RetryChange::Forget {
                    git_dir,
                    branch_ref,
                    only_while_unreachable,
                    taken,
                } => {
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
    pub fn take_due(
        &mut self,
        git_dir: &Path,
        now: Instant,
    ) -> Vec<Due> {
        let mut kept: Vec<Waiting> = Vec::new();
        let mut due: Vec<Due> = Vec::new();
        for entry in std::mem::take(&mut self.waiting) {
            if entry.git_dir == git_dir && entry.due <= now {
                due.push(Due {
                    git_dir: entry.git_dir,
                    item: entry.item,
                    attempt: entry.attempt,
                });
            } else {
                kept.push(entry);
            }
        }
        self.waiting = kept;
        return due;
    }

    /// How long until the first plan of this Git directory is due (zero for one already due),
    /// or nothing when none waits.
    pub fn next_wait(
        &self,
        git_dir: &Path,
        now: Instant,
    ) -> Option<Duration> {
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
