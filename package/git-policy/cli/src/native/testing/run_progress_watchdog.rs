//! What:
//!  The watchdog itself:
//! the thread that looks at the run at each interval and ends the process when the run has stopped,
//! the look a control takes now,
//! and how many looks a thread has taken.
//! Why:
//!   A stuck test has to end the run itself rather than wait for an outside timeout,
//! which the mutation tool records as a timeout instead of a failed test,
//! and a control that ends its own wait judges by the same evidence the watchdog does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // watch(windowMs): handle | undefined; lookNow(): Look | undefined; looksTakenByThisThread(): number
//! ```

/// The reason the run ends.
use super::ending::Ending;
/// The clock that judges each look.
use super::judge::Judge;
/// The look one look found,
/// and the one look itself.
use super::look_figures::Look;
use super::one_look::look;
/// A thread registered as a reader while it looks.
use super::readers::Reading;
/// How often a run is looked at.
use crate::push_progress::SAMPLE_INTERVAL;
/// The kernel's number for the calling thread,
/// which a look leaves out so its own reading never counts as the run's work.
use crate::thread_quiet::current_thread_id;
/// Writing the report on standard error,
/// whose failure is ignored so the ending always runs.
use std::io::Write as _;
/// Time spans and the monotonic clock.
use std::time::{
    Duration,
    Instant,
};

/// What:
///  Start the one thread that ends this process when the run has shown no sign of work for `window`.
/// Why:
///   A stuck test has to end the run itself rather than wait for an outside timeout,
/// which the mutation tool
///  records as a timeout instead of a failed test;
/// the window is what a stuck run costs,
/// where a wall clock would end
///  every run that is merely slow.
/// The thread is started where a process is allowed one,
/// and a process that is given none runs
///  without it (a control of a site that starts threads).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watch(windowMs) { try { spawnThread(() => run(windowMs)); } catch {} }
/// ```
pub(crate) fn watch(window: Duration) -> Option<std::thread::JoinHandle<()>> {
    return std::thread::Builder::new()
        .name(String::from("unit-test-watchdog"))
        .spawn(move || run(window))
        .ok();
}

/// What:
///  One look at this run now,
/// taken by the thread that calls it,
/// for a control that ends a wait when the run stops working.
/// Why:
///   The controls judge a wait by the same evidence the watchdog does:
/// a run that keeps moving is left alone,
/// however long its event takes,
/// and only a run that shows nothing is over.
/// The calling thread's own readings are left out,
/// as every other reader's are,
/// so a control that looks for its event is not itself the work it waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookNow(): Look | undefined { looksTaken.set(looksTaken.get() + 1); return look(currentThreadId()) }
/// ```
pub(crate) fn look_now() -> Option<Look> {
    LOOKS_TAKEN.with(|taken: &std::cell::Cell<u64>| {
        taken.set(taken.get().saturating_add(1));
    });
    // This thread's own readings of the run are not the run's work (`OWN_READINGS`), so a look it takes with no
    //  registration of its own is taken off its next look, and a run that only reads it is still judged quiet.
    // The registration leaves this thread out of the run's own tasks while its look is taken, as it does for a wait.
    let _reading: Option<Reading> = Reading::begin();
    return current_thread_id().and_then(look);
}

std::thread_local! {
    /// The looks `look_now` has taken on this thread.
    static LOOKS_TAKEN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What:
///  How many looks the calling thread has taken at this run.
/// Why:
///   A control that says a wait with nothing to wait for does not look at the run cannot measure how long the call took:
/// a thread
///  the scheduler holds just before the call makes a wait that returned at once look slow,
/// and a thread it holds just
///  after a sleep makes one that slept look quick.
/// The count is the event itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looksTakenByThisThread(): number;
/// ```
pub(crate) fn looks_taken_by_this_thread() -> u64 {
    return LOOKS_TAKEN.with(std::cell::Cell::get);
}

/// What:
///  Look at the run until it ends the process.
/// Why:
///   The loop is the whole of the watchdog:
/// one look at each interval,
/// one judgment,
/// and the process ends when
///  the judgment says the run stopped moving.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(windowMs) { let started = Date.now(); let judge = new Judge(windowMs); for (;;) { sleep(500); if (judge.observe(Date.now() - started, look(reader))) { abort() } } }
/// ```
fn run(window: Duration) {
    let started: Instant = Instant::now();
    let Some(reader): Option<i32> = current_thread_id() else {
        // Where the kernel numbers no thread, nothing can be read, and the window is a fixed deadline as it was before.
        std::thread::sleep(window);
        end(Ending::Unmeasurable(window));
    };
    let mut judge: Judge = Judge::new(window);
    // The reader's own readings are not the run's work, and neither is another reader's (`look`).
    let _reading: Option<Reading> = Reading::begin();
    loop {
        std::thread::sleep(SAMPLE_INTERVAL);
        if let Some(ending) = judge.observe(started.elapsed(), look(reader)) {
            end(ending);
        }
    }
}

/// What:
///  Say why the run ended and end the process,
/// whatever became of the report.
/// Why:
///   The line is what the person sees in the test log,
/// so it names the window that passed and the signs that were
///  read,
/// and there is no other way to end a stuck run from its own thread.
/// A report that cannot be written (a full disk,
/// a closed pipe) must not be a precondition for ending:
/// a
///  panicking macro would kill only this thread,
/// and the test it watches would stay blocked without a watcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function end(ending) { try { process.stderr.write(`the unit-test run ${ending}: a test is stuck\n`) } catch {} process.abort(); }
/// ```
fn end(ending: Ending) -> ! {
    // The error is ignored on purpose: `eprintln!` panics when the write fails, and only this thread would die.
    let mut stderr = std::io::stderr().lock();
    let _written = writeln!(stderr, "the unit-test run {ending}: a test is stuck");
    std::process::abort();
}
