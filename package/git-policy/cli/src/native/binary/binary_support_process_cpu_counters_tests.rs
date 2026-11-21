//! What:
//!  Controls of the reader that sums one process's processor time over its threads.
//! Why:
//!   A process starts and ends threads while it serves,
//! so the reader meets listings one of whose threads is gone by the time its own file is read.
//! A read that failed over such a thread would name a process that is still there as one that ended,
//! which is what the gate of 2026-11-15T06:00-05:00 made of a service that was alive throughout
//! (`a_record_nothing_announced_is_found::an_idle_service_runs_no_work`).
//!
//! The kernel wakes a join inside the thread's own exit and removes its `/proc` entry later:
//! `mm_release` clears the thread id the join waits on and wakes that futex
//! (`kernel/fork.c:1617-1627`,
//! reached from `exit_mm` at `kernel/exit.c:926`),
//! and the task's `/proc` entry goes only in `release_task` (`proc_flush_pid`,
//! `kernel/exit.c:280`),
//! reached from `exit_notify` at `kernel/exit.c:959`,
//! later in the same `do_exit`.
//! So a thread's own file may still be readable when `join` returns,
//! which one loaded gate did meet (issue 6330):
//! this control waits for that file to go instead of assuming it has gone,
//! so it reaches its named state under either order,
//! and never depends on one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = threadNumbers(pid); await endAndJoin(one); await untilUnreadable(pid, one); expect(ticksOfThreads(pid, listing)).toBe(ticksOfThreads(pid, withoutIt));
//! ```

use super::support::{cpu_ticks, thread_numbers, thread_ticks, ticks_of_threads};
use git_policy_cli::thread_quiet::current_thread_id;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

/// How long the control waits for the kernel to remove a joined thread's own file.
/// That removal is bookkeeping on the way out of the very exit that woke the join,
/// and the whole arm of the reproduction that forces the other order (issue 6330) finishes in about a millisecond,
/// so this patience is far past any release a loaded host delays.
const RELEASE_PATIENCE: Duration = Duration::from_secs(30);

/// How often the wait looks for the file.
/// A gap of this size keeps the wait's own reads negligible beside {@link RELEASE_PATIENCE},
/// and it sleeps between looks,
/// so the wait adds no load to the host it runs on.
const RELEASE_POLL: Duration = Duration::from_millis(1);

/// What:
///  The numbers of the threads that are there at every moment of this control.
/// Why:
///   A sum over threads that may come and go cannot be compared with a sum taken a moment later,
/// and two threads of this process outlive the control:
/// the thread-group leader,
/// whose number is the process's own,
/// and the thread the control itself runs on.
/// Their clock ticks only grow,
/// so a sum over them taken earlier bounds one taken later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stayed = [...new Set([pid, currentThreadId()])];
/// ```
fn threads_that_stay(pid: u32) -> Vec<u32> {
    let ours: u32 = u32::try_from(current_thread_id().expect("the kernel's number for this thread")).expect("a thread number");
    let mut stayed: Vec<u32> = vec![pid];
    if ours != pid {
        stayed.push(ours);
    }
    return stayed;
}

/// What:
///  Waits,
/// bounded,
/// until `thread`'s own file under `pid` can no longer be read.
/// Why:
///   A joined thread's `/proc` entry is removed by `release_task`,
/// after the futex the join waits on has woken,
/// so its absence is a state to reach,
/// never a moment to assume at the join.
///  The wait sleeps between looks,
/// so it adds nothing to the host's load while it waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (readable(pid, thread) && began.elapsed() < PATIENCE) await sleep(POLL);
/// ```
///
/// # Panics
/// When the kernel still lists the thread after {@link RELEASE_PATIENCE},
/// which is a thread this control could not observe in the state it names.
fn wait_until_the_thread_file_is_gone(pid: u32, thread: u32) {
    let began: Instant = Instant::now();
    while thread_ticks(pid, thread).is_some() {
        assert!(
            began.elapsed() < RELEASE_PATIENCE,
            "thread {thread} of process {pid} ended and was joined, and the kernel still answers its own file after {:?}, so this control did not reach the released thread it names",
            RELEASE_PATIENCE
        );
        std::thread::sleep(RELEASE_POLL);
    }
}

/// What:
///  A thread that a listing names and that ends before its own file is read is a thread that spent no more,
///  never a process that ended.
/// Why:
///   The listing and the reading of one file per thread are two moments,
/// and this control makes the difference between them certain rather than likely:
/// a thread of this process waits,
/// the listing is taken while it waits,
/// the thread ends and is joined,
/// and the read is given that same listing.
/// The thread's own file goes when the kernel releases the task,
/// which this control waits for,
/// bounded,
/// so it reaches that state under any order the kernel takes between the join and the release.
/// The sum then counts the threads that are there,
/// while the process itself still answers with counters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await untilUnreadable(pid, ended); expect(cpu_ticks(pid)).toBeDefined();
/// ```
#[test]
fn a_thread_that_ended_between_the_listing_and_the_read_is_not_a_process_that_ended() {
    let pid: u32 = std::process::id();
    let (report, reported): (Sender<u32>, Receiver<u32>) = channel();
    let (release, held): (Sender<()>, Receiver<()>) = channel();
    // The thread names itself, so the tid this control ends is the thread that waited and never another one that a
    // runtime starts or ends beside it.
    let waiting: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let number: i32 = current_thread_id().expect("the kernel's number for this thread");
        report.send(u32::try_from(number).expect("a thread number")).expect("the control knows which thread waits");
        let _released = held.recv();
    });
    let ended: u32 = reported.recv().expect("the thread that waits names itself");
    let listing: Vec<u32> = thread_numbers(pid).expect("the threads of this process");
    assert!(
        listing.contains(&ended),
        "the listing is taken while that thread waits, so it names it: {listing:?} has no {ended}"
    );
    let without: Vec<u32> = listing.iter().copied().filter(|number| { return *number != ended; }).collect();
    // The threads that stay go on working, so the ticks they had while it waited are the least the listing must reach.
    let while_it_waited: u64 = ticks_of_threads(pid, threads_that_stay(pid).as_slice());
    release.send(()).expect("the thread that waits ends");
    waiting.join().expect("the thread that waited ends");
    // The kernel wakes the join inside the thread's own exit and removes its own file afterwards, in `release_task`,
    // so the file can still be readable here; the wait reaches the state this control is named for under either order.
    wait_until_the_thread_file_is_gone(pid, ended);
    assert!(
        thread_ticks(pid, ended).is_none(),
        "a thread's own file is gone once the kernel released the thread, which is the reading a listing meets"
    );
    // The threads that are there go on working between the two readings, so the sum over the listing names no less
    //  than they did while it waited and no more than the same threads sum to a moment later; a thread that ended
    //  adds nothing to it.
    let over_the_listing: u64 = ticks_of_threads(pid, listing.as_slice());
    let over_the_live_threads: u64 = ticks_of_threads(pid, without.as_slice());
    assert!(
        over_the_listing >= while_it_waited,
        "the threads that stay still count, so the listing sums to no less than they did while it waited: {over_the_listing} against {while_it_waited}"
    );
    assert!(
        over_the_listing <= over_the_live_threads,
        "a thread that ended spent nothing more, so the listing sums to no more than the threads that are there sum to a moment later: {over_the_listing} against {over_the_live_threads}"
    );
    assert!(cpu_ticks(pid).is_some(), "the process is still here, so it has counters");
}
