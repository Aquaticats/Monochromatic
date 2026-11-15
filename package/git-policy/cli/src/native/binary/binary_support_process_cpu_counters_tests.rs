//! What:
//!  Controls of the reader that sums one process's processor time over its threads.
//! Why:
//!   A process starts and ends threads while it serves,
//! so the reader meets listings one of whose threads is gone by the time its own file is read.
//! A read that failed over such a thread would name a process that is still there as one that ended,
//! which is what the gate of 2026-11-15T06:00-05:00 made of a service that was alive throughout
//! (`a_record_nothing_announced_is_found::an_idle_service_runs_no_work`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const listing = threadNumbers(pid); await endOneOfThem(); expect(ticksOfThreads(pid, listing)).toBe(ticksOfThreads(pid, withoutIt));
//! ```

use super::support::{cpu_ticks, thread_numbers, thread_ticks, ticks_of_threads};

/// What:
///  A thread that a listing names and that ends before its own file is read is a thread that spent no more,
///  never a process that ended.
/// Why:
///   The listing and the reading of one file per thread are two moments,
/// and this control makes the difference between them certain rather than likely:
/// a thread of this process waits, the listing is taken while it waits,
/// the thread ends and is joined,
/// and the read is given that same listing.
/// The thread's own file is gone,
/// which is the state the reader meets,
/// and the sum is the one the listing without it gives,
///  while the process itself still answers with counters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => readFile(`/proc/${pid}/task/${ended}/stat`)).toThrow(); expect(cpu_ticks(pid)).toBeDefined();
/// ```
#[test]
fn a_thread_that_ended_between_the_listing_and_the_read_is_not_a_process_that_ended() {
    let pid: u32 = std::process::id();
    let before: Vec<u32> = thread_numbers(pid).expect("the threads of this process");
    let (release, held) = std::sync::mpsc::channel::<()>();
    let waiting: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let _released = held.recv();
    });
    let listing: Vec<u32> = thread_numbers(pid).expect("the threads of this process");
    // Any thread this listing has and the earlier one had not is the one that waited, and one of them is proof enough: which one
    //  it is does not matter, and another short-lived thread beside it changes nothing the control claims.
    let ended: u32 = *listing
        .iter()
        .find(|number| { return !before.contains(number); })
        .expect("the thread that waits is in the listing taken while it waits");
    let _released = release.send(());
    waiting.join().expect("the thread that waited ends");
    assert!(
        thread_ticks(pid, ended).is_none(),
        "a thread's own file is gone once the thread ended, which is the reading a listing meets"
    );
    let without: Vec<u32> = listing.iter().copied().filter(|number| { return *number != ended; }).collect();
    // The threads that are there go on working between the two readings, so the sum over the listing names no more than the sum
    //  over the same threads a moment later; a thread that ended adds nothing to it.
    let over_the_listing: u64 = ticks_of_threads(pid, listing.as_slice());
    let over_the_live_threads: u64 = ticks_of_threads(pid, without.as_slice());
    assert!(
        over_the_listing <= over_the_live_threads,
        "a thread that ended spent nothing more, so the listing sums to no more than the threads that are there sum to a moment later: {over_the_listing} against {over_the_live_threads}"
    );
    assert!(cpu_ticks(pid).is_some(), "the process is still here, so it has counters");
}
