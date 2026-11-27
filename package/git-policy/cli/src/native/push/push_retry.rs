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
//! and reads the journal again after a pause that doubles with each read whose job outlasted its ceiling,
//! because that read is still holding a place of the blocking bound.
//! What a failed push's own words mean (whether it is final,
//! whether it is counted)
//! and the words the person reads are in `push_retry_failure.rs`;
//! the first of Git's own lines that gives a reason for a failure is in `push_retry_cause_line.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pause = retries.schedule(item, attempt); /* later */ for (const due of retries.takeDue(gitDir)) await push(due);
//! ```

/// Publish the pause before each new try,
/// the two bounds it is counted between,
/// and the directory the plan is kept in.
pub use super::push_retry_pauses::{
    FIRST_PAUSE,
    LONGEST_PAUSE,
    PLAN_DIRECTORY,
    RETRY_VARIABLE,
    first_pause_from,
    pause_before,
};
/// Import the branch waiting for a new try,
/// which the list keeps,
/// and the seam a control kills a service through.
use super::push_retry_waiting_branches::Waiting;
#[cfg(test)]
pub(crate) use super::push_retry_waiting_branches::stop_after_in_a_control;
/// Publish the branch waiting for a new try,
/// the tries due again,
/// the request a worker is given,
/// and the change it asks of the plan.
pub use super::push_retry_waiting_branches::{
    Due,
    RetryChange,
    Retrying,
};
/// Import the plan on disk:
/// what a start reads,
/// and what the workers write.
use crate::push_retry_store::LoadedPlans;
/// Import the push item a plan pushes again.
use crate::work_item::WorkItem;
/// Import the borrowed and owned path types,
/// and the two clock readings the pauses are counted between.
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    Instant,
};

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
    /// one per branch of a Git directory,
    /// which `push_retry_changes_applied.rs` changes.
    pub(super) waiting: Vec<Waiting>,
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
        return Self::with_first_pause(first_pause_from(
            std::env::var_os(RETRY_VARIABLE).as_deref(),
        ));
    }

    /// A list with this first pause.
    pub fn with_first_pause(first: Duration) -> Self {
        return Self {
            first,
            waiting: Vec::new(),
            store: None,
            unread: false,
            read_failures: 0,
        };
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
    pub fn restored(
        first: Duration,
        directory: &Path,
    ) -> Self {
        let mut list: Self = Self {
            first,
            waiting: Vec::new(),
            store: Some(directory.to_path_buf()),
            unread: false,
            read_failures: 0,
        };
        let read: LoadedPlans = crate::push_retry_store::load_reporting(directory);
        let now: Instant = Instant::now();
        for saved in read.plans {
            list.waiting.push(Waiting {
                git_dir: saved.git_dir,
                item: saved.item,
                attempt: saved.attempt,
                due: now + pause_before(saved.attempt, first),
            });
        }
        list.unread = read.unread;
        return list;
    }

    /// A list for a service that starts now,
    /// with the first pause the variable gives or the default.
    pub fn restored_from_environment(directory: &Path) -> Self {
        return Self::restored(
            first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()),
            directory,
        );
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
    pub fn unread(
        first: Duration,
        directory: &Path,
    ) -> Self {
        return Self {
            first,
            waiting: Vec::new(),
            store: Some(directory.to_path_buf()),
            unread: true,
            read_failures: 1,
        };
    }

    /// `unread` with the first pause the variable gives or the default.
    pub fn unread_from_environment(directory: &Path) -> Self {
        return Self::unread(
            first_pause_from(std::env::var_os(RETRY_VARIABLE).as_deref()),
            directory,
        );
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
    ///  Note that a read of the journal outlasted its ceiling and gave no answer,
    /// so it is owed again at the next pause.
    /// Why:
    ///   The read is tried again rather than given up on,
    /// and that job is still holding a place of the blocking bound until the journal answers,
    /// so a journal that stopped answering may not be read at the shortest pause for ever:
    /// the pause grows as it does for a plan and the attempts thin out.
    /// A job the system gave no thread for promises that nothing ran and holds no place,
    /// so that answer does not come here.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// retries.restoreCameToNothing(); // the next read is owed at pauseBefore(++readFailures, first)
    /// ```
    pub fn restore_came_to_nothing(&mut self) {
        self.unread = true;
        self.read_failures = self.read_failures.saturating_add(1);
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
            self.waiting.push(Waiting {
                git_dir: saved.git_dir,
                item: saved.item,
                attempt: saved.attempt,
                due: now + pause_before(saved.attempt, self.first),
            });
        }
        self.unread = read.unread;
        self.read_failures = if self.unread {
            self.read_failures.saturating_add(1)
        } else {
            0
        };
        return found;
    }

    /// Whether the list already plans a try of this branch of this Git directory.
    fn holds(
        &self,
        git_dir: &Path,
        branch_ref: &str,
    ) -> bool {
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
    pub fn schedule(
        &mut self,
        git_dir: &Path,
        item: &WorkItem,
        attempt: u32,
        now: Instant,
    ) -> Duration {
        let pause: Duration = pause_before(attempt, self.first);
        self.plan(git_dir, item, attempt, now + pause);
        return pause;
    }
}

/// The pauses and the list are controlled in the test module.
#[cfg(test)]
#[path = "push_retry_tests.rs"]
mod tests;
