//! What:
//!  The task a watch runs and the bound its end is given,
//!  with the refusal a control arms to stand in for a system that cannot make another thread.
//! Why:
//!  Dropping a watch must end its task within a stated bound,
//!  and the only way to control that end is to refuse the thread the task needs,
//!  so both live together and away from the watch that uses them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const running = { handle, finished }; const endBound = stopCeiling + stopTailAllowance;
//! ```

use std::time::Duration;
use tokio::task::JoinHandle;
/// The running watcher for a task that started,
/// or nothing when no thread could run a task.
pub(crate) fn running_watcher_of(spawned: Option<JoinHandle<()>>, finished: std::sync::mpsc::Receiver<()>) -> Option<RunningWatcher> {
    let handle: JoinHandle<()> = spawned?;
    return Some(RunningWatcher { handle, finished });
}

#[cfg(test)]
thread_local! {
    /// How many waiter threads the next starts are refused.
    static WAITER_THREAD_REFUSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Refuse the next `count` threads that would wait for a step left running.
#[cfg(test)]
pub(crate) fn refuse_the_next_left_running_waiter(count: usize) {
    WAITER_THREAD_REFUSALS.with(|left| { left.set(count); });
}

/// What:
///  The refusal a control armed,
///  standing in for a system that cannot make another thread.
/// Why:
///  The start of the thread that waits for a step left running answers a refused thread the way it answers any other,
/// and only a control tells the two apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusedNow = () => armedRefusal-- > 0 ? new Error('refused') : undefined;
/// ```
pub(crate) fn a_waiter_thread_is_refused_now() -> Option<std::io::Error> {
    #[cfg(test)]
    if WAITER_THREAD_REFUSALS.with(|left| {
        let armed: usize = left.get();
        left.set(armed.saturating_sub(1));
        return armed > 0;
    }) {
        return Some(std::io::Error::other("a control refused the thread that waits for a step left running"));
    }
    return None;
}

/// How long dropping a watch waits for its task to end:
/// the bound of "nothing should ever wait forever" on a wait that is over in
///  a few hundred microseconds.
/// Measured on 2026-11-07 (`push/child_watch_drop_time_measurement_tests.rs`,
/// three runs by hand under
///  a load average of 15 to 40):
/// dropping a watch between waits took at most 505 us and in the middle of a wait at most 389 us,
/// the
///  lock sweep alone at most 569 us,
/// and a whole stop of a child that answers the request at most 41680 us past the window.
/// A task
///  can legitimately outlast those only inside a stop that waits for a member in uninterruptible sleep,
/// which is at most
///  `STOP_CEILING` after the request (the owner's "3x worst case" of the longest delay measured;
/// a descendant that ignores the request
///  is killed when that wait ends),
/// so the bound is that ceiling plus `STOP_TAIL_ALLOWANCE`:
/// aborting earlier would cancel the kill of a
///  stubborn descendant that holds no pipe.
pub(crate) const WATCHER_END_BOUND: Duration = crate::stop_waits_while_a_member_cannot_be_interrupted::STOP_CEILING.saturating_add(STOP_TAIL_ALLOWANCE);

/// What a stop needs beyond its grace:
/// the kill,
/// the look at the processes,
/// the lock sweep and the end of the task.
/// One second is
///  twenty-four times the longest tail measured (41680 us),
/// see `WATCHER_END_BOUND`.
const STOP_TAIL_ALLOWANCE: Duration = Duration::from_secs(1);

/// A watcher task and the signal that it has ended.
pub(crate) struct RunningWatcher {
    /// The task,
    /// aborted when it does not end by `WATCHER_END_BOUND`.
    pub(crate) handle: JoinHandle<()>,
    /// Disconnects when the task ends.
    pub(crate) finished: std::sync::mpsc::Receiver<()>,
}
