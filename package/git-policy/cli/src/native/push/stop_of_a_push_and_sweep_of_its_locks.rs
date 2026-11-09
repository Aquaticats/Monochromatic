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
//! In TS you'd write (pseudocode):
//! ```ts
//! child.kill('SIGTERM'); await Promise.race([exited, delay(graceMs)]) || child.kill('SIGKILL'); locks.sweep(tree);
//! ```

/// The diagnostic log of the wrapper.
use crate::diagnostic_log::debug;
/// The blocking pool,
/// for the sweep that reads directories.
use crate::blocking_job::run_blocking_job;
/// A child owned until it is reaped,
/// awaited through its pidfd.
use crate::child_awaited_through_a_pidfd::{OwnedChild, adopt};
/// The process's runtime,
/// for the task that sweeps after the child ends.
use crate::lazy_runtime::spawn_on_runtime;
/// When a stop that asked its tree to end may force the end.
use crate::stop_waits_while_a_member_cannot_be_interrupted::{STOP_CEILING, escalation_waits};
/// The processes a push started,
/// and the calls that look at and end them.
use crate::process_tree::{TreeMember, descendants_of, member_of, still_running};
use crate::member_signalled_through_its_pidfd::{kill_members, terminate_members};
/// The lock files a stopped push leaves,
/// and the ledger that finds them.
use crate::stopped_git_locks::LockLedger;
/// `Child` is a push handed to the sweeper.
use std::process::Child;
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// Pause between two looks at the running push.
pub(crate) const POLL_PAUSE: Duration = Duration::from_millis(10);

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
///  The child is handed over as the standard library's value and adopted inside the task,
/// so a task that never runs
///  leaves it as it was and never waits for it.
/// The wait for the child has the ceiling every stop has:
/// past it the child is left to the
///  kernel and said here,
/// never waited for,
/// because a task nobody waits for must not be held by a process the
///  kernel may never release.
/// Returns nothing where the runtime has no worker to run the task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// void Promise.race([child.exited, delay(ceiling)]).then(() => locks.sweep(tree));
/// ```
pub fn sweep_after_exit(child: Child, tree: Vec<TreeMember>, locks: LockLedger) -> Option<tokio::task::JoinHandle<()>> {
    return sweep_task(Some(child), tree, locks);
}

/// `sweep_after_exit` for a push that may already have been collected:
/// with no child there is only the tree to wait for.
fn sweep_task(child: Option<Child>, tree: Vec<TreeMember>, locks: LockLedger) -> Option<tokio::task::JoinHandle<()>> {
    let started = spawn_on_runtime(async move {
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
        // The sweep reads directories, so it runs on the blocking pool and not on the worker that polls tasks.
        if let Err(error) = run_blocking_job(move || { locks.sweep(); }).await {
            debug(TAG, &format!("the sweep of a stopped push's locks did not run: {error}"));
        }
    });
    if started.is_none() {
        debug(TAG, "the runtime has no worker to sweep a stopped push's locks");
    }
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
/// Why:
///   Git's own signal handling ends the helpers it started and removes its lock files,
///  and a push that ignores the request cannot outlive the kill.
///  A killed push leaves its lock files,
/// which fail every later Git command on the same refs,
/// so once the push and
///  what it started are gone the locks they hold are swept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill('SIGTERM'); await Promise.race([exited, delay(graceMs)]) || child.kill('SIGKILL'); locks.sweep();
/// ```
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
    let mut ended: bool = false;
    // Past the grace the wait goes on while a member of the tree is in uninterruptible sleep, up to the stop's ceiling, because a kill
    //  then only cuts the cleanup that follows the call (Git removes a lock's pid file, then the lock).
    //  The exit is awaited on the pidfd; the pause is the look at the tree's state, made at the interval the loop always used.
    while !ended && escalation_waits(asked.elapsed(), grace, tree.as_slice()) {
        ended = tokio::time::timeout(POLL_PAUSE, child.wait()).await.is_ok();
    }
    if !ended {
        // The child is being abandoned; its exit is not reported, only the cause.
        let _killed = child.kill();
        // A killed process that sits in uninterruptible I/O (a stalled disk or network file system) cannot be
        //  collected until the kernel releases it, so the wait is bounded by the grace again and the child is then
        //  left to be collected by a task of its own, instead of holding the service's push for as long as the stall.
        if grace_remains(Duration::ZERO, grace) {
            // Whether it ended in time shows below: a collected child has nothing left to hand over.
            let _ended_in_time = tokio::time::timeout(grace, child.wait()).await;
        }
    }
    // What the push started outlives it: its transport helper holds the connection and its hook may never exit
    // (measured: 60 seconds for a helper, for good for a `git-lfs` that never answers). They were asked to end with
    // the push, so they get what is left of the grace and are then killed.
    while escalation_waits(asked.elapsed(), grace, below.as_slice()) && below.iter().any(still_running) {
        tokio::time::sleep(POLL_PAUSE).await;
    }
    kill_members(below.as_slice());
    // A child that was collected is gone; one that was not goes on to the task that waits for it.
    let remaining: Option<Child> = child.into_std_when_unreaped();
    let _sweeper = settle_locks(remaining, tree, claimed_ledger, grace).await;
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
/// `child` is the stopped push when it was not collected,
/// and nothing when it was.
/// Returns the
///  task,
/// if one was started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (child === undefined && allEnded(tree)) { locks.sweep(tree); return undefined; } return sweepAfterExit(child, tree, locks);
/// ```
pub async fn settle_locks(child: Option<Child>, tree: Vec<TreeMember>, locks: LockLedger, grace: Duration) -> Option<tokio::task::JoinHandle<()>> {
    let Some(uncollected) = child else {
        wait_for_tree(tree.as_slice(), grace).await;
        if !tree.iter().any(still_running) {
            // The sweep reads directories, so it runs on the blocking pool and not on the worker that polls tasks.
            if let Err(error) = run_blocking_job(move || { locks.sweep(); }).await {
                debug(TAG, &format!("the sweep of a stopped push's locks did not run: {error}"));
            }
            return None;
        }
        // The push was collected and the tree is still running: the task waits for the tree alone.
        return sweep_task(None, tree, locks);
    };
    return sweep_task(Some(uncollected), tree, locks);
}

/// The sweep's wait and its bound are controlled in the test module.
#[cfg(test)]
#[path = "stop_of_a_push_and_sweep_of_its_locks_tests.rs"]
mod tests;
