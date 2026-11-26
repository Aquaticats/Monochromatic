//! What:
//!  Controls of the reader that sums one process's processor time over its threads.
//! Why:
//!   A process starts and ends threads while it serves,
//! so the reader meets listings one of whose threads is gone by the time its own file is read.
//! A read that failed over such a thread would name a process that is still there as one that ended,
//! which is what the gate of 2026-11-15T06:00-05:00 made of a service that was alive throughout
//! (`a_record_nothing_announced_is_found::an_idle_service_runs_no_work`).
//!
//! The kernel wakes a join inside the thread's own exit and removes its `/proc` entry later.
//! Line numbers are the v6.12 tag (`adc218676eef`) and v7.3,
//! the series the host that met this runs (7.3.5).
//! `mm_release` clears the thread id the join waits on and wakes that futex
//! (`kernel/fork.c:1617-1627` at v6.12,
//! `:1490-1497` at v7.3),
//! reached from `exit_mm` (`kernel/exit.c:926` at v6.12,
//! `:974` at v7.3).
//! The entry stops answering inside `release_task` (`kernel/exit.c:241` at v6.12,
//! `:248` at v7.3),
//! where `__exit_signal` (`:258` at v6.12,
//! `:272` at v7.3)
//! runs `__unhash_process`,
//! which detaches the pid (`:129` at v6.12,
//! `:140` at v7.3);
//! `proc_flush_pid` (`:280` at v6.12,
//! `:297` at v7.3) flushes cached entries after that,
//! and `exit_notify` (`:959` at v6.12,
//! `:998` at v7.3) is where the same `do_exit` reaches `release_task`.
//! So a thread's own file may still be readable when `join` returns,
//! which one loaded gate did meet (issue 6330):
//! this control waits for that file to go instead of assuming it has gone,
//! so it reaches its named state under either order,
//! and never depends on one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = threadNumbers(pid);
//! await endAndJoin(ended);
//! await untilUnreadable(pid, ended);
//! expect(threadTicks(pid, ended)).toBeUndefined();
//! expect(ticksOfThreads(pid, [ended, still])).toBe(threadTicks(pid, still));
//! expect(ticksOfThreads(pid, listing)).toBeGreaterThanOrEqual(threadTicks(pid, still));
//! expect(cpuTicks(pid)).toBeDefined();
//! ```

use super::support::{
    cpu_ticks,
    thread_numbers,
    thread_ticks,
    ticks_of_threads,
};
use git_policy_cli::thread_quiet::current_thread_id;
use std::sync::mpsc::{
    Receiver,
    Sender,
    channel,
};
use std::time::{
    Duration,
    Instant,
};

/// How long the control waits for the kernel to remove a joined thread's own file.
/// This is a ceiling,
/// not a measured latency:
/// it sits far under the 300 s at which nextest stops a test
/// (`slow-timeout = { period = "60s", terminate-after = 5 }` in
/// `package/git-policy/cli/.config/nextest.toml`).
/// The removal itself is bookkeeping on the way out of the very exit that woke the join.
const RELEASE_PATIENCE: Duration = Duration::from_secs(30);

/// How often the wait looks for the file.
/// A gap of this size keeps the wait's own reads negligible beside [`RELEASE_PATIENCE`],
/// and it sleeps between looks,
/// so the wait adds no load to the host it runs on.
const RELEASE_POLL: Duration = Duration::from_millis(1);

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
/// When the kernel still lists the thread after [`RELEASE_PATIENCE`],
/// which is a thread this control could not observe in the state it names.
fn wait_until_the_thread_file_is_gone(
    pid: u32,
    thread: u32,
) {
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
/// The sum then holds the threads that are there,
/// and beside a thread that only waits,
/// whose clock ticks are improbable to have grown since the listing,
/// a sum that over-counts fails,
/// while a sum that under-counts is not pinned:
/// a thread that only waits has usually spent no tick to expect.
/// The process itself still answers with counters.
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
    let (still_report, still_reported): (Sender<u32>, Receiver<u32>) = channel();
    let (leave, stayed): (Sender<()>, Receiver<()>) = channel();
    // The thread names itself, so the tid this control ends is the thread that waited and never another one that a
    // runtime starts or ends beside it.
    let waiting: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let number: i32 = current_thread_id().expect("the kernel's number for this thread");
        report
            .send(u32::try_from(number).expect("a thread number"))
            .expect("the control knows which thread waits");
        let _released = held.recv();
    });
    // A thread that only waits stands still, so its clock ticks are improbable to grow between the two readings
    // that follow: the kernel promises nothing, and spending a tick on purpose would burn CPU, which is banned.
    let staying: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let number: i32 = current_thread_id().expect("the kernel's number for this thread");
        still_report
            .send(u32::try_from(number).expect("a thread number"))
            .expect("the control knows which thread stays");
        let _left = stayed.recv();
    });
    let ended: u32 = reported.recv().expect("the thread that waits names itself");
    let still: u32 = still_reported
        .recv()
        .expect("the thread that stays names itself");
    let listing: Vec<u32> = thread_numbers(pid).expect("the threads of this process");
    assert!(
        listing.contains(&ended) && listing.contains(&still),
        "the listing is taken while that thread waits, so it names it beside the thread that stays: {listing:?} has no {ended} or no {still}"
    );
    release.send(()).expect("the thread that waits ends");
    waiting.join().expect("the thread that waited ends");
    // The kernel wakes the join inside the thread's own exit and removes its own file afterwards, in `release_task`,
    // so the file can still be readable here; the wait reaches the state this control is named for under either order.
    wait_until_the_thread_file_is_gone(pid, ended);
    assert!(
        thread_ticks(pid, ended).is_none(),
        "a thread's own file is gone once the kernel released the thread, which is the reading a listing meets"
    );
    // The thread that stays has spent no tick since the listing in an ordinary run, so a sum that counts the ended
    // thread again reads higher than it; a sum that answers less is not pinned by this assertion.
    let still_ticks: u64 = thread_ticks(pid, still).expect("the thread that stays is there");
    let beside_it: u64 = ticks_of_threads(pid, [ended, still].as_slice());
    assert_eq!(
        beside_it, still_ticks,
        "the sum counts the thread that is there and gives nothing for the one whose own file is gone: {beside_it} against {still_ticks}"
    );
    let over_the_listing: u64 = ticks_of_threads(pid, listing.as_slice());
    assert!(
        over_the_listing >= still_ticks,
        "every thread of the listing spends at least nothing, so the listing sums to no less than the thread that stays: {over_the_listing} against {still_ticks}"
    );
    assert!(
        cpu_ticks(pid).is_some(),
        "the process is still here, so it has counters"
    );
    leave.send(()).expect("the thread that stays ends");
    staying.join().expect("the thread that stayed ends");
}
