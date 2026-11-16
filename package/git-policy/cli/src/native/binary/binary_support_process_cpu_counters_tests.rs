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
use std::time::{Duration, Instant};

/// What a worker thread of a control fills, so a reading of it is far above the reading's own noise.
///  One pass over the megabyte array touches enough pages that the kernel counts ticks for it,
/// and a tick is a hundredth of a second on every machine this package runs on.
const WORKER_BYTES: usize = 64 * 1024 * 1024;

/// How many passes a worker makes over its array.
const WORKER_PASSES: u8 = 8;

/// How long a control waits for a killed child to become a zombie.
const ZOMBIE_PATIENCE: Duration = Duration::from_secs(2);

/// What:
///  A worker thread that fills its array `WORKER_PASSES` times and answers when it has,
///  then waits for its release so the control can read the process while the thread is still there.
/// Why:
///   The reading this control judges has to come from work that a thread spends ticks on,
///  and the thread has to end while the process stays alive,
///  which is the one state a sum over the threads cannot carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worker = spawnWorker(); await worker.ready; const before = cpuTicks(pid); worker.release(); await worker.join();
/// ```
fn a_worker_that_spends_ticks_and_then_ends() -> (std::sync::mpsc::Receiver<()>, std::sync::mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel::<()>();
    let (release_sender, release_receiver) = std::sync::mpsc::channel::<()>();
    let worker: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let mut bytes: Vec<u8> = vec![0; WORKER_BYTES];
        for value in 1..=WORKER_PASSES {
            bytes.fill(value);
        }
        std::hint::black_box(&bytes);
        ready_sender.send(()).expect("the control receives the worker's readiness");
        let _released = release_receiver.recv();
    });
    return (ready_receiver, release_sender, worker);
}

/// What:
///  A process's processor time never goes down while the process lives,
///  so a difference of two readings is a duration and never an overflow.
/// Why:
///   A sum over the threads of a listing loses every thread that ended,
///  and a service ends threads while it serves:
/// with such a sum the reading after a thread ended is smaller than the one before,
///  and the idle-service control's `after - before` panics in the test binary
///  (8 ticks before and 5 after in the review's reproduction).
/// The reading that cannot go down is the process's own file,
/// which keeps the ticks of the threads that ended
///  (`/proc/<pid>/stat` reports the whole thread group: measured 4 ticks before a worker's work, 123 while it worked,
///  and 123 again after the worker ended, while the sum over the threads fell back to 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = cpuTicks(pid); worker.join(); const after = cpuTicks(pid); expect(after).toBeGreaterThanOrEqual(before);
/// ```
#[test]
fn a_process_that_ends_a_thread_keeps_its_processor_time() {
    let pid: u32 = std::process::id();
    let (ready, release, worker) = a_worker_that_spends_ticks_and_then_ends();
    ready.recv().expect("the worker is ready");
    let before: u64 = cpu_ticks(pid).expect("this process lives");
    let _released = release.send(());
    worker.join().expect("the worker ends");
    let after: u64 = cpu_ticks(pid).expect("this process lives after a thread of it ended");
    assert!(
        after >= before,
        "the processor time of a live process never goes down, and a thread of it ended between the two readings: {before} then {after}"
    );
    // The subtraction the idle-service control makes, which a reading that can go down turns into a panic.
    let used: u64 = after - before;
    assert!(used < u64::MAX, "the difference of two readings is a duration: {used} ticks");
}

/// What:
///  The ticks a worker thread spends are the process's own,
///  which is why the first round of this branch needed no sum over the threads to see a service's look at work.
/// Why:
///   The first round summed one reading per thread,
/// on the comment that `/proc/<pid>/stat` is the main thread's alone and that the look runs on a worker of the service's
/// runtime, so a reading of the main thread alone showed no processor time.
///  Measured, the process's own file reports the whole thread group,
/// so the work a look does on a worker is visible in the process's total and the sum was never what made it visible.
/// A control that showed only `after >= before` would pass with a reading that saw no thread's work at all,
///  so this one takes work that a worker has certainly done and asks for a rise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = cpuTicks(pid); await worker.ready; expect(cpuTicks(pid)).toBeGreaterThan(before);
/// ```
#[test]
fn work_on_a_worker_thread_is_in_the_process_total() {
    let pid: u32 = std::process::id();
    let before: u64 = cpu_ticks(pid).expect("this process lives");
    let (ready, release, worker) = a_worker_that_spends_ticks_and_then_ends();
    ready.recv().expect("the worker is ready, which is after its work");
    let after: u64 = cpu_ticks(pid).expect("this process lives");
    let _released = release.send(());
    worker.join().expect("the worker ends");
    assert!(
        after > before,
        "the ticks a worker thread spent are in the process's own total, which is what the look's work is read from: {before} then {after}"
    );
}

/// What:
///  A killed child that nothing has reaped yet is not a live process,
///  so the reader answers nothing for it.
/// Why:
///   `/proc/<pid>/stat` and its thread files stay readable while a zombie stands in the table,
///  so a reader that asked only whether the file could be read answered `Some` for a process that had ended
///  (the review's reproduction: `cpu_ticks` returned `Some(0)` for a zombie),
///  and the liveness assertion of the idle-service control accepted it.
/// The state in the process's own file is what says whether the process lives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// child.kill(); await untilZombie(child.pid); expect(cpuTicks(child.pid)).toBeUndefined();
/// ```
#[test]
fn a_zombie_is_not_a_live_process() {
    let mut child: std::process::Child = std::process::Command::new("/usr/bin/sleep").arg("60").spawn().expect("a child starts");
    child.kill().expect("the child this control started is killed");
    let pid: u32 = child.id();
    let began: Instant = Instant::now();
    loop {
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
            panic!("the killed child's own file is readable while nothing has reaped it")
        };
        let after_name: &str = stat.rsplit_once(')').expect("a process name delimiter").1;
        if after_name.split_whitespace().next() == Some("Z") {
            break;
        }
        assert!(began.elapsed() < ZOMBIE_PATIENCE, "the killed child becomes a zombie within {ZOMBIE_PATIENCE:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
    let counters: Option<u64> = cpu_ticks(pid);
    assert!(counters.is_none(), "a process that ended has no processor time to read, and a zombie is one that ended: {counters:?}");
    let _reaped: std::process::ExitStatus = child.wait().expect("the child this control started is reaped");
}

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
