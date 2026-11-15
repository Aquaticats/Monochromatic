//! What:
//!  The stop of a running push (ask,
//! wait the grace,
//! kill,
//! collect) and the sweep of the lock files a stopped push and everything it
//!  started leave,
//! all awaited on the runtime.
//! Why:
//!  `bounded_push.rs` holds the run of a push and its limits;
//! the stop and the sweep are the part that outlives the run
//!  (a killed child in uninterruptible I/O ends late and leaves a lock after the push was reported),
//! and this file keeps it apart so
//!  each file stays within the line limit.
//! The names are re-exported from `bounded_push.rs`.
//!
//! What:
//!  Every wait and every blocking call between the moment a stop is asked and the moment `stop_push` answers,
//!  and the bound each one is under.
//! Why:
//!   The bound of a stop is what a person waits, so no wait of a stop may read it afresh
//!  (`process/stop_waits_while_a_member_cannot_be_interrupted.rs`,
//! `STOP_CEILING`),
//!   and no wait outside it may hold the answer.
//!  Every line here names the figure that ends it:
//!  - reading the tree: `descendants_of` and `member_of` read one file per process of `/proc`
//!  (a blocking read, bounded by the size of the table, with no clock of its own);
//!  - the two signals to the tree, and the kill:
//!    `ask_to_stop`,
//!    `terminate_members` and `kill_members`
//!  send and answer, they never wait;
//!  - the polite wait, looked at every `POLL_PAUSE`:
//!  `escalation_deadline`,
//!    which is `STOP_CEILING` less the parts kept back below,
//!  and inside the grace it always waits,
//!    past it only while a member sits in an uninterruptible call;
//!  - the reap after the kill: `REAP_AFTER_THE_KILL`,
//!    which is part of the same bound and not an addition to it;
//!  - the wait for what the push started: `escalation_deadline` again;
//!  - the wait for the rest of the tree in `settle_locks`: what is left of `escalation_deadline`;
//!  - the sweep of the locks, when the tree has gone: what is left of `STOP_CEILING`,
//!    which is at least
//!  `SWEEP_AFTER_THE_STOP`,
//!    and a sweep that does not run or does not answer inside it is kept as owed (below);
//!  - the task `settle_locks` and `sweep_after_exit` may start:
//!    its own waits are `STOP_CEILING` each,
//!  and nobody waits for them,
//!    so they are never the bound a person or the service is held by.
//!  Ownership:
//!   the child of a stop is reaped here or kept in the abandoned list here,
//!   and the ledger is swept here or kept as owed here,
//!  so neither ever lives only inside a future the runtime may refuse to start or cancel before its first poll.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! child.kill('SIGTERM'); await Promise.race([exited, delay(graceMs)]) || child.kill('SIGKILL'); locks.sweep(tree);
//! ```

/// The diagnostic log of the wrapper.
use crate::diagnostic_log::{debug, warn};
/// A blocking job awaited no longer than a ceiling,
/// and what its answer says about the work it took.
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_once_awaited_no_longer_than};
/// A child owned until it is reaped,
/// awaited through its pidfd.
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
/// The process's runtime,
/// for the task that sweeps after the child ends.
use crate::lazy_runtime::spawn_on_runtime;
/// When a stop that asked its tree to end may force the end.
use crate::stop_waits_while_a_member_cannot_be_interrupted::{STOP_CEILING, bound_left, escalation_waits};
/// The processes a push started,
/// and the calls that look at and end them.
use crate::process_tree::{TreeMember, descendants_of, member_of, still_running};
use crate::member_signalled_through_its_pidfd::{kill_members, terminate_members};
/// The lock files a stopped push leaves,
/// and the ledger that finds them.
use crate::stopped_git_locks::LockLedger;
/// `Child` is a push handed to the sweeper.
use std::process::Child;
/// `Arc` and `Mutex` keep what a stop hands over where the stop can reach it until a task has taken it.
use std::sync::{Arc, Mutex};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Pause between two looks at the running push.
pub(crate) const POLL_PAUSE: Duration = Duration::from_millis(10);

/// What:
///  How long the reap after the kill gets,
/// out of the stop's own bound.
/// Why:
///   A process sent `SIGKILL` ends as soon as the kernel runs it once more,
/// so a stop that has spent its whole bound waiting for one still collects it
///  in a moment;
/// a process the kernel holds in an uninterruptible call is not collected by waiting at all
///  (`doc/troubleshooting/sigkill-does-not-end-a-process-in-uninterruptible-sleep.md`)
/// and is kept for a later round instead.
/// Citation:
/// partial.
/// Measured on 2026-11-16: every stop of a fast run reaped its killed push in 1 ms
/// (`stop after the kill and its wait` of the
///  instrumented service),
/// and a killed process whose tracer was stopped was not collected in a second of waiting
///  (`traced-holder-experiment`),
/// so this covers a scheduler round trip on a loaded host many times over.
/// An open question for the owner.
pub(crate) const REAP_AFTER_THE_KILL: Duration = Duration::from_millis(50);

/// What:
///  How long the sweep of a stopped push's locks is guaranteed out of the stop's own bound.
/// Why:
///   The sweep reads the reference directories of one repository and,
/// for a lock old enough to be judged,
///  the descriptors of every process;
/// the bound a stop has is what a person waits,
/// so the sweep must fit inside it and not be
///  the wait that passes it.
/// Citation:
/// partial.
/// `SPEC.md` ("Reference locks left by a Git that was killed",
/// "Where it applies") measured that judgement at a median 53 ms (47 to 63 ms)
///  with 1,182 processes on this host,
/// and this run measured 0 ms in every stop of a fast run and 0 ms in the gate's container
///  (`settle after the awaited sweep` of the instrumented service).
/// Three times the longest of those is the owner's margin
///  for a bound taken from a measurement (`foundation/worst_delay_seen_on_this_host.rs`,
/// `OWNER_MARGIN`),
/// rounded up.
/// An open question for the owner.
pub(crate) const SWEEP_AFTER_THE_STOP: Duration = Duration::from_millis(200);

/// What:
///  The ledgers of stopped pushes whose locks were not judged inside the bound of their stop.
/// Why:
///   A sweep that could not run,
/// or did not answer inside the bound,
/// removed nothing,
/// and the judgement before every
///  fetch and push of the wrapper's own (`push_lock_clearing.rs`) is what clears what a lock left that way holds;
/// the
///  ledger is kept here so that the line `keep_the_sweep_owed` writes is not the only place it can be read,
/// and so a
///  control can say how many sweeps a run owed.
static OWED_SWEEPS: Mutex<Vec<LockLedger>> = Mutex::new(Vec::new());

/// How many sweeps of stopped pushes this process owes a judgement.
pub fn sweeps_owed() -> usize {
    return OWED_SWEEPS.lock().map(|owed| { return owed.len(); }).unwrap_or(0);
}

/// Ask a running push to end,
///  so Git can stop the helper processes it started.
/// A child that was already reaped has no number (`OwnedChild::id` answers nothing for it),
/// and a signal sent to
///  number zero would reach every process in this command's group,
/// so no signal is sent then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (id > 0) child.kill('SIGTERM');
/// ```
#[cfg(unix)]
fn ask_to_stop(id: u32) {
    let Ok(pid) = libc::pid_t::try_from(id) else {
        return;
    };
    if pid <= 0 {
        return;
    }
    // SAFETY: `kill` sends one signal to the PID of a child this module started and has not reaped;
    // a failure is ignored because the child may have ended.
    unsafe { libc::kill(pid, libc::SIGTERM) };
}

/// Platforms without a polite stop signal go straight to the kill.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn ask_to_stop(_id: u32) {}

/// Whether part of the grace is still left after the stop request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asked < grace
/// ```
pub(crate) fn grace_remains(asked: Duration, grace: Duration) -> bool {
    return asked < grace;
}

/// Wait,
/// bounded by `grace`,
/// until every member of a stopped tree has ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.race([allEnded(tree), delay(graceMs)]);
/// ```
async fn wait_for_tree(tree: &[TreeMember], grace: Duration) {
    let waited: Instant = Instant::now();
    while grace_remains(waited.elapsed(), grace) && tree.iter().any(still_running) {
        tokio::time::sleep(POLL_PAUSE).await;
    }
    return;
}

/// What:
///  What a stop hands a task when its tree has not ended:
/// the child it could not collect,
/// the tree,
///  and the ledger of the locks it left.
/// Why:
///   The child and the ledger outlive the stop,
/// and both must stay reachable by the stop until a task has truly taken them
///  (`HandedOver`):
/// a child dropped here is a zombie of this process for its life,
/// and a ledger dropped here is a
///  sweep nothing owes and nothing judges.
struct AStopHandedToATask {
    /// The child the stop could not collect.
    child: Option<Child>,
    /// What the push started and has not ended.
    tree: Vec<TreeMember>,
    /// The locks the stopped push may have left.
    locks: LockLedger,
}

/// What:
///  A stopped push's child and ledger,
/// kept where the stop can still reach them until a task has taken them.
/// Why:
///   `spawn_on_runtime` drops the future it is given where the runtime has no worker
/// (`foundation/lazy_runtime.rs`),
/// and a task cancelled before
///  its first poll drops it too,
/// so whatever lives only inside that future is lost with nothing done about it:
/// the child is never
///  reaped and the locks are never judged.
/// The task takes what it needs at its first poll,
/// and what is still here when the last holder
///  goes is settled here instead,
/// the child into the abandoned list and the ledger into the sweeps owed.
struct HandedOver {
    /// What has not been taken yet.
    kept: Mutex<Option<AStopHandedToATask>>,
}

impl HandedOver {
    /// The handover of `sweep`,
    /// one reference for the task and one for the stop that hands it over.
    fn of(sweep: AStopHandedToATask) -> Arc<HandedOver> {
        return Arc::new(HandedOver { kept: Mutex::new(Some(sweep)) });
    }

    /// What the task takes at its first poll,
    /// and nothing when the handover was settled here already.
    fn take(&self) -> Option<AStopHandedToATask> {
        return self.kept.lock().ok()?.take();
    }
}

impl Drop for HandedOver {
    /// Settles what no task took while it is still reachable:
    /// the child for the round that reaps abandoned children,
    /// and the
    ///  ledger as a sweep this process owes.
    fn drop(&mut self) {
        let Ok(mut slot) = self.kept.lock() else {
            return;
        };
        let Some(sweep) = slot.take() else {
            return;
        };
        if let Some(unreleased) = sweep.child {
            keep_abandoned(unreleased);
        }
        keep_the_sweep_owed(sweep.locks, NO_TASK_TOOK_THE_SWEEP);
    }
}

/// What a sweep owed says when no task took it,
/// so nothing was even started.
const NO_TASK_TOOK_THE_SWEEP: &str = "the runtime gave the task no worker";

/// What a sweep owed says when its job was given no thread and will not run.
const THE_SWEEP_WAS_GIVEN_NO_THREAD: &str = "the blocking pool gave the job no thread";

/// What a sweep owed says when its job started,
/// or was queued,
/// and did not answer inside the bound.
const THE_SWEEP_DID_NOT_ANSWER: &str = "the job did not answer inside the bound";

/// What:
///  Keep `locks` as a sweep this process owes,
/// and say so where the service's journal and a person read it.
/// Why:
///   The reference locks of a repository are judged before every fetch and push of the wrapper's own
/// (`lock/push_lock_clearing.rs`),
/// so a lock this sweep
///  would have removed is cleared by the next command that needs that repository;
/// nothing says so unless this line
///  is written,
/// and a person who reads a stop and then silence cannot tell an empty sweep from a missing one.
fn keep_the_sweep_owed(locks: LockLedger, why: &str) {
    let roots: Vec<String> = locks.roots().iter().map(|root| { return root.display().to_string(); }).collect();
    let of: String = if roots.is_empty() { String::from("that repository") } else { roots.join(", ") };
    warn(
        TAG,
        &format!("the locks of a stopped push were not judged inside the bound of its stop ({why}); the next fetch or push of {of} judges them"),
    );
    if let Ok(mut owed) = OWED_SWEEPS.lock() {
        owed.push(locks);
    }
}

/// What:
///  Collect a killed child that the kernel has not released,
/// and sweep the locks it leaves,
/// as a task of the runtime.
/// Why:
///   A killed process in an uninterruptible call (a stalled disk or file system) ends only when the kernel lets it go,
///  and on its way out it can create a lock file that nothing will remove (measured on 2026-11-03:
/// empty tracking-ref
///  locks born about 37 seconds after the stop).
/// The push is reported at its deadline and does not wait for that,
/// so
///  the child is collected here,
/// and its locks are swept when it is gone.
/// The task holds no lock and ends with the child.
///  The child and the ledger stay reachable by the caller until the task has taken them (`HandedOver`),
/// so a runtime that refuses the task
///  leaves neither lost:
/// the child goes to the round that reaps abandoned children and the sweep is said to be owed.
/// The wait for the child has the ceiling every stop has:
/// past it the child is left to the
///  kernel and said here,
/// never waited for,
/// because a task nobody waits for must not be held by a process the
///  kernel may never release.
/// Returns nothing where the runtime has no worker to run the task,
/// having kept what the task was for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// void Promise.race([child.exited, delay(ceiling)]).then(() => locks.sweep(tree));
/// ```
pub fn sweep_after_exit(child: Child, tree: Vec<TreeMember>, locks: LockLedger) -> Option<tokio::task::JoinHandle<()>> {
    return start_the_sweep_task(AStopHandedToATask { child: Some(child), tree, locks });
}

/// What:
///  Start the task that waits for the rest of a stopped push's tree and judges the locks it left,
///  keeping what it is given where the stop can reach it until the task has taken it.
/// Why:
///   The task is the only caller of the sweep when the tree has not ended,
/// and the runtime may refuse to start it,
///  so
/// what it works on cannot travel only inside its future (`HandedOver`).
fn start_the_sweep_task(sweep: AStopHandedToATask) -> Option<tokio::task::JoinHandle<()>> {
    let handed: Arc<HandedOver> = HandedOver::of(sweep);
    let for_the_task: Arc<HandedOver> = Arc::clone(&handed);
    let started: Option<tokio::task::JoinHandle<()>> = spawn_on_runtime(async move {
        let Some(AStopHandedToATask { child, tree, locks }) = for_the_task.take() else {
            return;
        };
        // Whatever an earlier stop left behind and the kernel has released is collected first, so no sweep waits on that list.
        let _earlier: usize = reap_abandoned_children();
        if let Some(uncollected) = child {
            let mut owned: OwnedChild = adopt(uncollected);
            // The exit status is not reported: the push was reported at its deadline.
            let ceiling: Duration = ceiling_for_a_child(owned.id());
            match tokio::time::timeout(ceiling, owned.wait()).await {
                Ok(_collected) => {}
                Err(_late) => {
                    // The child is left to the kernel, as one it has not released yet must be, and its handle is kept: a handle
                    //  dropped here would leave the process a zombie for the life of the service once the kernel does let it go.
                    if let Some(left) = owned.into_std_when_unreaped() {
                        keep_abandoned(left);
                    }
                    debug(TAG, "the child of the stopped push was not collected within the ceiling of a stop; it is left to the kernel for the next round to collect, and its locks to the stale-lock judgement");
                }
            }
        }
        // Nothing waits for this task, so it waits for the rest of the tree no longer than every other wait of a stop:
        //  a descendant wedged in a dead file system, or one that left the process group the signal reached, is left to the
        //  stale-lock judgement every later push and fetch makes.
        let gone: bool = wait_until_gone(tree.as_slice(), STOP_CEILING).await;
        if !gone {
            debug(TAG, "a process of the stopped push did not end within the ceiling of a stop; its locks are left to the stale-lock judgement");
        }
        // The sweep reads directories, so it runs on the blocking pool and not on the worker that polls tasks, and even here
        //  where nobody waits for it its job is awaited no longer than a stop's ceiling: a job that does not answer leaves the
        //  ledger owed rather than a task waiting for good.
        let owed_if_no_answer: LockLedger = locks.clone();
        match run_a_job_once_awaited_no_longer_than(STOP_CEILING, move || { locks.sweep(); }).await {
            TheAnswerOfAJobAwaited::Ran(_removed) => {}
            TheAnswerOfAJobAwaited::NotRun => keep_the_sweep_owed(owed_if_no_answer, THE_SWEEP_WAS_GIVEN_NO_THREAD),
            TheAnswerOfAJobAwaited::NoAnswer => keep_the_sweep_owed(owed_if_no_answer, THE_SWEEP_DID_NOT_ANSWER),
        }
    });
    if started.is_none() {
        debug(TAG, "the runtime has no worker to sweep a stopped push's locks");
    }
    // The last holder that goes settles what no task took, so the answer is the same where the runtime refused the task:
    //  nothing of the stop is left inside a future nobody polls.
    drop(handed);
    return started;
}

/// What:
///  The ceiling the sweep gives the child it was handed:
/// the stop's own,
/// or a control's for one child.
/// Why:
///   A control must see a child the kernel does not release without waiting the stop's whole ceiling,
///  which is three
///  times the longest delay measured on this host;
/// and one child's ceiling belongs to that child alone,
/// so no other
///  sweep of the run is shortened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// controlCeilingFor(pid) ?? STOP_CEILING
/// ```
fn ceiling_for_a_child(id: u32) -> Duration {
    #[cfg(test)]
    if let Some(ceiling) = control_ceiling_for(id) {
        return ceiling;
    }
    let _ = id;
    return STOP_CEILING;
}

/// The ceiling a control gives one child,
/// and the child's number;
/// nothing outside a control.
#[cfg(test)]
static CONTROL_CEILING: std::sync::Mutex<Option<(u32, Duration)>> = std::sync::Mutex::new(None);

/// Gives the child `id` a ceiling of its own for a control.
#[cfg(test)]
pub(crate) fn ceiling_for_a_child_in_a_control(id: u32, ceiling: Duration) {
    if let Ok(mut slot) = CONTROL_CEILING.lock() {
        *slot = Some((id, ceiling));
    }
}

/// The ceiling a control gave the child `id`,
/// or nothing.
#[cfg(test)]
fn control_ceiling_for(id: u32) -> Option<Duration> {
    let slot = CONTROL_CEILING.lock().ok()?;
    return match *slot {
        Some((control_id, ceiling)) if control_id == id => Some(ceiling),
        _ => None,
    };
}

/// The children a stop left behind whose exit nothing else collects,
/// kept so that a later round reaps them.
static ABANDONED: std::sync::Mutex<Vec<Child>> = std::sync::Mutex::new(Vec::new());

/// Keeps `child` for a later round to collect.
fn keep_abandoned(child: Child) {
    if let Ok(mut list) = ABANDONED.lock() {
        list.push(child);
    }
}

/// Whether `child` has not ended yet,
/// collecting its exit when it has.
/// A handle that cannot be judged is given up,
/// since keeping it would hold a place for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (child) => { const status = child.pollExit(); return status === undefined }
/// ```
fn has_not_ended(child: &mut Child) -> bool {
    return match child.try_wait() {
        Ok(Some(_status)) => false,
        Ok(None) => true,
        Err(_) => false,
    };
}

/// What:
///  Collects every kept child the kernel has released,
/// without waiting for one it has not,
/// and answers how many were collected.
/// Why:
///   Dropping a `Child` reaps nothing:
/// a process that exits after its stop was reported stays a zombie for the life of
///  the service,
/// and one entry of the process table per such stop is a leak the service cannot afford.
/// `try_wait` is
///  the non-blocking collection:
/// it answers the exit of a child that ended and nothing of one that has not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abandoned = abandoned.filter((child) => child.exitCode === null && child.pidAlive())
/// ```
pub fn reap_abandoned_children() -> usize {
    let Ok(mut list) = ABANDONED.lock() else {
        return 0;
    };
    let before: usize = list.len();
    list.retain_mut(has_not_ended);
    return before.saturating_sub(list.len());
}

/// What:
///  Wait until `running` answers nothing,
/// or until `ceiling` has passed,
/// answering whether the tree was gone.
/// Why:
///   A wait nobody waits for still needs a bound:
/// a task that wakes every `POLL_PAUSE` for the life of the service does nothing but hold the tree in memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const began = Date.now(); while (running() && Date.now() - began < ceilingMs) await sleep(POLL_PAUSE);
/// ```
async fn wait_until_gone(tree: &[TreeMember], ceiling: Duration) -> bool {
    let began: Instant = Instant::now();
    while tree.iter().any(still_running) {
        if began.elapsed() >= ceiling {
            return false;
        }
        tokio::time::sleep(POLL_PAUSE).await;
    }
    return true;
}

/// What:
///  Stop a running push:
/// ask politely,
///  wait the grace,
///  then kill and collect it,
///  and judge the reference locks it may have left.
/// The whole stop is bounded by the ceiling of a stop,
/// measured from the moment it asks:
///  the waiting for a member that cannot be interrupted,
/// the reap after the kill and the sweep of the locks are all taken out
///  of that one bound,
/// so a wait that would start past it does not start and a wait that does not answer inside it is
///  kept as owed.
/// A child the kernel has not released by then is kept in the abandoned list here,
/// and a sweep that did not run is
///  recorded here,
/// so neither travels only inside a task the runtime may refuse to start.
/// Why:
///   Git's own signal handling ends the helpers it started and removes its lock files,
///  and a push that ignores the request cannot outlive the kill.
///  A killed push leaves its lock files,
/// which fail every later Git command on the same refs,
/// so once the push and
///  what it started are gone the locks they hold are swept.
pub async fn stop_push(mut child: OwnedChild, grace: Duration, locks: LockLedger) {
    // Read before the push is signalled: once it has exited, its helpers and hooks no longer point to it.
    let below: Vec<TreeMember> = i32::try_from(child.id()).map(descendants_of).unwrap_or_default();
    let mut tree: Vec<TreeMember> = i32::try_from(child.id()).ok().and_then(member_of).into_iter().collect();
    tree.extend(below.iter().cloned());
    // Nothing is claimed before the stop: a number in a pid file does not say which PID namespace wrote it, and a lock a member
    //  of the tree was seen to hold is a lock another Git may hold at the same path (cross-vendor review of 2026-11-09,
    //  finding 1). The sweep therefore removes a lock only on the judgement's own proof, and names the rest.
    let claimed_ledger: LockLedger = locks.clone();
    ask_to_stop(child.id());
    terminate_members(below.as_slice());
    let asked: Instant = Instant::now();
    // The reap after the kill and the sweep are part of the stop's own work, and their parts of the bound are kept back here so
    //  that the waiting for a member that cannot be interrupted cannot spend them: a stop that gave every one of them to that
    //  wait answered past its bound with a child it never reaped and a sweep it never ran.
    let bound: Duration = bound_left(Duration::ZERO);
    let reap_part: Duration = REAP_AFTER_THE_KILL.min(bound);
    let sweep_part: Duration = SWEEP_AFTER_THE_STOP.min(bound.saturating_sub(reap_part));
    let escalation_deadline: Duration = bound.saturating_sub(reap_part.saturating_add(sweep_part));
    let mut ended: bool = false;
    // Past the grace the wait goes on while a member of the tree is in uninterruptible sleep, up to the stop's ceiling, because a kill
    //  then only cuts the cleanup that follows the call (Git removes a lock's pid file, then the lock).
    //  The exit is awaited on the pidfd; the pause is the look at the tree's state, made at the interval the loop always used.
    while !ended && asked.elapsed() < escalation_deadline && escalation_waits(asked.elapsed(), grace, tree.as_slice()) {
        ended = tokio::time::timeout(POLL_PAUSE, child.wait()).await.is_ok();
    }
    if !ended {
        // The child is being abandoned; its exit is not reported, only the cause.
        let _killed = child.kill();
        // A killed process ends as soon as the kernel runs it once more unless it sits in a call that does not end, so this reap
        //  is always tried, out of the part of the bound kept for it. What it does not collect is kept below, here, and never
        //  inside a task the runtime may refuse to start.
        let _reaped: bool = tokio::time::timeout(reap_part, child.wait()).await.is_ok();
    }
    // What the push started outlives it: its transport helper holds the connection and its hook may never exit
    // (measured: 60 seconds for a helper, for good for a `git-lfs` that never answers). They were asked to end with
    // the push, so they get what is left of the grace and are then killed.
    while asked.elapsed() < escalation_deadline && escalation_waits(asked.elapsed(), grace, below.as_slice()) && below.iter().any(still_running) {
        tokio::time::sleep(POLL_PAUSE).await;
    }
    kill_members(below.as_slice());
    // A child that was collected is gone; one the kernel has not released is kept here, where the round that reaps abandoned
    //  children collects it, whatever the runtime does afterwards.
    let remaining: Option<Child> = child.into_std_when_unreaped();
    if let Some(unreleased) = remaining {
        keep_abandoned(unreleased);
    }
    let tree_left: Duration = escalation_deadline.saturating_sub(asked.elapsed());
    let sweep_left: Duration = bound_left(asked.elapsed());
    let _sweeper = settle_locks(tree, claimed_ledger, tree_left, sweep_left).await;
}

/// What:
///  Remove the locks of a stopped tree now when it has ended,
/// or as a task of the runtime when it has not.
/// Why:
///   A killed process ends only when the kernel has run it once more,
/// which a loaded or stalled host may not do within the
///  grace;
/// a sweep before that finds the tree running and removes nothing,
/// and nothing would try again.
/// The task waits for
///  the whole tree and then sweeps.
/// `tree_left` is what is left of the part of the stop's bound the escalation may use,
/// and the whole of what this wait
///  for the tree may take;
/// `sweep_left` is what is left of the whole bound,
/// and the inline sweep is awaited no longer than
///  what remains of it after the tree's wait.
/// A sweep that does not run or does not answer inside that removes nothing now and is kept as owed
///  (`keep_the_sweep_owed`),
/// so a stop answers inside its bound whether the blocking pool moves or not.
/// Returns the
///  task,
/// if one was started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitForTree(tree, treeLeft); if (allEnded(tree)) { sweepWithin(sweepLeft); return undefined; } return sweepAfterExit(tree, locks);
/// ```
pub async fn settle_locks(tree: Vec<TreeMember>, locks: LockLedger, tree_left: Duration, sweep_left: Duration) -> Option<tokio::task::JoinHandle<()>> {
    let waited: Instant = Instant::now();
    wait_for_tree(tree.as_slice(), tree_left).await;
    if !tree.iter().any(still_running) {
        // The sweep reads directories and the process table, so it runs on the blocking pool and not on the worker that polls
        //  tasks; what is left of the stop's bound is what its answer is awaited for, and no place of the pool or thread to run
        //  it leaves it owed rather than waited for.
        let inside: Duration = sweep_left.saturating_sub(waited.elapsed());
        let owed_if_no_answer: LockLedger = locks.clone();
        return match run_a_job_once_awaited_no_longer_than(inside, move || { locks.sweep(); }).await {
            TheAnswerOfAJobAwaited::Ran(_removed) => None,
            TheAnswerOfAJobAwaited::NotRun => {
                keep_the_sweep_owed(owed_if_no_answer, THE_SWEEP_WAS_GIVEN_NO_THREAD);
                None
            }
            TheAnswerOfAJobAwaited::NoAnswer => {
                keep_the_sweep_owed(owed_if_no_answer, THE_SWEEP_DID_NOT_ANSWER);
                None
            }
        };
    }
    // The push was collected and the tree is still running: the task waits for the tree alone.
    return start_the_sweep_task(AStopHandedToATask { child: None, tree, locks });
}

/// The sweep's wait and its bound are controlled in the test module.
#[cfg(test)]
#[path = "stop_of_a_push_and_sweep_of_its_locks_tests.rs"]
mod tests;
