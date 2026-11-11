//! What:
//!  The service's own retry of a push it stopped or that failed:
//! the pause before each new try,
//! the list of branches waiting
//!  for one,
//! and the test for the one failure no retry can mend.
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
//! A conflict,
//! a login the service does not have and a hook on the server that
//!  declined the push stop the retry until the next commit (owner decision,
//! 2026-11-03).
//! A removal the round that asked for it could not write is owed to a later round,
//! and the branch is named until that write happens,
//! so the plan of a push that ended never outlives the round that forgot it
//!  (`UnwrittenForget`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pause = retries.schedule(item, attempt); /* later */ for (const due of retries.takeDue(gitDir)) await push(due);
//! ```

/// The positive-seconds reading shared by the wait limits.
use crate::push_deadline_and_patience_settings::positive_seconds;
/// The plan on disk.
use crate::push_retry_store::{StoredPlan, load, micros_now, name_of, remove_except, save};
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

/// The words of the reconcile's report of a conflict,
/// the one failure that retrying cannot mend.
pub const CONFLICT_PHRASE: &str = " conflicts in: ";

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

/// Why a failed push cannot pass by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalFailure {
    /// The reconcile reported a conflict.
    Conflict,
    /// The remote asked for a login the service does not have.
    Login,
    /// A hook on the server declined the push.
    Declined,
}

/// The mark of a line in which the server rejected one reference;
/// Git prints it for every refused reference
/// (`transport.c`,
/// `print_ref_status('!', "[remote rejected]", ...)`,
/// Git 2.56.0).
pub const REMOTE_REJECTED: &str = "[remote rejected]";

/// The words every hook decline of Git's own server side carries in the rejection's reason:
/// `"pre-receive hook declined"` and
/// `"push-to-checkout hook declined"` and `"hook declined"` (the update hook) in `builtin/receive-pack.c`,
/// Git 2.56.0;
/// a hosting
/// service's own reason that names a hook,
/// such as `protected branch hook declined`,
/// ends the same way.
pub const HOOK_DECLINED: &str = "hook declined";

/// The words with which Git and the programs it runs say that the remote wanted a login the push did not have,
/// each from a
/// fixture run of `git push` from Git 2.56.0 on 2026-11-03:
/// - `remote-curl.c` `die(_("Authentication failed for '%s'"))`:
///   a 401 and no usable password;
/// - `prompt.c` `die("could not read %s%s", prompt, "terminal prompts disabled")`:
///   a 401 with no user name and no prompt allowed;
/// - `http.c` `"The requested URL returned error: %ld"` with 401 or 403:
///   the server refused the login or the permission;
/// - OpenSSH's `Permission denied (publickey).`,
///   which Git relays when the key is refused.
pub const LOGIN_PHRASES: [&str; 5] = ["Authentication failed for", "terminal prompts disabled", "returned error: 401", "returned error: 403", "Permission denied ("];

/// The lines of an output in which the server rejected a reference.
fn rejection_lines(output: &str) -> Vec<&str> {
    let mut rejected: Vec<&str> = Vec::new();
    for line in output.lines() {
        let trimmed: &str = line.trim();
        if trimmed.contains(REMOTE_REJECTED) {
            rejected.push(trimmed);
        }
    }
    return rejected;
}

/// The failure of a push that no retry can mend,
/// from Git's own words;
/// nothing for a failure that can pass (a stall,
/// the
/// network,
/// a held lock) and for a rejection whose reason this release does not know.
pub fn final_failure(output: &str) -> Option<FinalFailure> {
    if output.contains(CONFLICT_PHRASE) {
        return Some(FinalFailure::Conflict);
    }
    for line in rejection_lines(output) {
        if line.contains(HOOK_DECLINED) {
            return Some(FinalFailure::Declined);
        }
    }
    for phrase in LOGIN_PHRASES {
        if output.contains(phrase) {
            return Some(FinalFailure::Login);
        }
    }
    return None;
}

/// The first line in which the server rejected the push for a reason this release does not know,
/// or nothing:
/// a push with such a
/// line stays retried,
/// and the journal names the line so that the reason can be learned.
pub fn unrecognised_rejection(output: &str) -> Option<String> {
    if final_failure(output).is_some() {
        return None;
    }
    let rejected: Vec<&str> = rejection_lines(output);
    if let Some(line) = rejected.first() {
        return Some((*line).to_string());
    }
    return None;
}

/// What:
///  The plain words the person's failure record starts with for a failure that stops the retry,
/// or nothing for a conflict,
/// whose
///  report already says what to do.
/// Why:
///   The owner's decision is that a login failure and a server hook's decline stop until the next commit
///  ("Offline is retried,
/// but auth and server hook declines stop until your next commit.");
/// the person is told which one it was,
///  in words,
/// above Git's own.
///  A decline is the server's answer at that moment and nothing about a later try,
/// so its words promise no outcome and say that a push by hand may succeed
///  (GitHub refused a push of `main` with `GH006` at `2026-11-09T23:30:11-05:00` and a plain `git push` of the same commit then succeeded:
/// `doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plainWords(cause) // 'monochromatic-git-policy-cli: the remote wants a login ...'
/// ```
pub fn plain_words(cause: FinalFailure) -> Option<&'static str> {
    return match cause {
        FinalFailure::Conflict => None,
        FinalFailure::Login => Some(
            "monochromatic-git-policy-cli: the remote refused this push because it wants a login that the commit service does not have, or because the login it has lacks \
             permission, so the commit service stops pushing this branch until your next commit. Log in in a way the service can use (a credential \
             helper, or an SSH key without a passphrase that its environment reaches) and make sure that login may push, then push by hand or commit again.",
        ),
        FinalFailure::Declined => Some(
            "monochromatic-git-policy-cli: the server refused this push: a hook on the server (a check the server runs before it accepts a push) declined it, with the message shown below. The commit service stops pushing this branch \
             until your next commit. A push by hand may succeed if the cause has cleared: fix what the message names, or wait, then push by hand or commit again.",
        ),
    };
}

/// Whether a failed push's output says something stopped it that only the person can resolve.
pub fn needs_the_person(output: &str) -> bool {
    return final_failure(output).is_some();
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
/// type RetryChange = { plan: { gitDir; item; attempt; due } } | { forget: { gitDir; branchRef; asked } };
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
        /// When the worker asked for the branch's plans to end,
        /// in microseconds since the epoch.
        /// Why:
        ///   A forget is applied by a job that is never cancelled,
        /// so it may be written long after the round that asked for it ended;
        /// this instant is what tells the plans it read from the ones a later round wrote,
        /// which stay.
        asked: u64,
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
///  Whether the round plans the same branch of the same Git directory again after `at`.
/// Why:
///   The last plan of a branch is the round's statement about it;
/// publishing an earlier one would remove the
///  file the worker of the last one already wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.slice(at + 1).some((c) => c.plan && c.plan.gitDir === gitDir && c.plan.item.branchRef === branchRef)
/// ```
fn planned_again_after(changes: &[RetryChange], at: usize, git_dir: &Path, branch_ref: &str) -> bool {
    for change in &changes[at + 1..] {
        if let RetryChange::Plan { git_dir: planned_dir, item, .. } = change
            && planned_dir == git_dir
            && item.branch_ref == branch_ref
        {
            return true;
        }
    }
    return false;
}

/// What:
///  The file name of the last plan the changes after `at` write for one branch of one Git directory,
/// or nothing.
/// Why:
///   A forget replayed before that plan must not remove it,
/// and the name is known before the plan is written;
///  only the last plan of a branch is ever published,
/// so its name is the one to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// changes.slice(at + 1).filter((c) => c.plan && sameBranch(c.plan)).map(planName).at(-1)
/// ```
fn last_plan_after(changes: &[RetryChange], at: usize, git_dir: &Path, branch_ref: &str) -> Option<String> {
    let mut name: Option<String> = None;
    for change in &changes[at + 1..] {
        if let RetryChange::Plan { git_dir: planned_dir, item, attempt, .. } = change
            && planned_dir == git_dir
            && item.branch_ref == branch_ref
        {
            name = Some(name_of(&StoredPlan { git_dir: planned_dir.clone(), item: item.clone(), attempt: *attempt }));
        }
    }
    return name;
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
        return Self { first, waiting: Vec::new(), store: None, unwritten: Vec::new() };
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
        let mut list: Self = Self { first, waiting: Vec::new(), store: Some(directory.to_path_buf()), unwritten: Vec::new() };
        let now: Instant = Instant::now();
        for saved in load(directory) {
            list.waiting.push(Waiting { git_dir: saved.git_dir, item: saved.item, attempt: saved.attempt, due: now + pause_before(saved.attempt, first) });
        }
        return list;
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
    /// A plan a worker asked for is already on disk, because the worker writes it before the person is told a retry comes;
    ///  a forget is not,
    /// so the branch is named for a later round's write, which is what keeps a killed service from restoring the plan of a
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
                RetryChange::Forget { git_dir, branch_ref, asked } => {
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
