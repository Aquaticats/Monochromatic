//! What:
//!  The loop of a child watch's task and the writing of its notices:
//! wait for the next mark or the next look at the child's work,
//! print or end the child,
//! repeat.
//! Why:
//!  `child_watch.rs` holds the watch the waiting thread uses and the running of watched steps;
//! the task's loop uses none of that,
//! so it sits in a file of its own and `child_watch.rs` starts it on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const mark of marksUntilTimeout()) { if (stoppable && clock.stalled()) await stop(pid); else if (ticker.due(elapsed)) await notice(); }
//! ```

use crate::bounded_push::StopCause;
use crate::child_watch_clock_marks_and_stopping::{Mark, Marks, SHORTEST_SLEEP, WatchFlags, WatchTerms, stop_process};
use crate::push_progress::{Clock, SAMPLE_INTERVAL};
use crate::stop_of_the_service_noticed_by_its_children::StopArm;
use crate::stopped_git_locks::LockLedger;
use crate::wait_progress::{ChildWaitFacts, Ticker, child_wait_line};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;

/// What:
///  The watcher task's loop:
/// wait until the next notice or the next look at the child's work is due,
/// print or end,
/// repeat.
/// Why:
///   Between waits the task awaits the next mark without a timeout,
/// so an idle watch costs nothing,
/// and a dropped
///  watch ends it because its sender is gone.
/// During a wait it wakes at the notice cadence and at the sampling interval of
///  the push meter,
/// whichever is sooner,
/// and at the window itself when that is shorter.
/// The notice is written on the blocking
///  pool,
/// because a terminal that does not take a line must not hold a worker;
/// where no thread can start the notices stop and the
///  judgment goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const mark of marksUntilTimeout()) { if (since === undefined) continue; if (stoppable && clock.stalled()) await stop(pid); else if (ticker.due(elapsed)) await notice(); }
/// ```
pub(crate) async fn watch_loop(
    mut incoming: UnboundedReceiver<Mark>,
    flags: WatchFlags,
    waiting_for: String,
    pid: u32,
    terms: WatchTerms,
    locks: LockLedger,
    mut stop: Option<StopArm>,
    root_pin: crate::child_watch_clock_marks_and_stopping::RootPin,
) {
    let expired: &AtomicBool = flags.expired.as_ref();
    let stoppable: &AtomicBool = flags.stoppable.as_ref();
    let longest: Arc<AtomicU64> = Arc::clone(&flags.longest);
    let (limit, cadence): (Duration, crate::wait_progress::Cadence) = (terms.limit, terms.cadence);
    let mut notices: Option<Box<dyn Write + Send>> = Some(terms.notices);
    let mut marks: Marks = Marks { since: None, ticker: Ticker::new(cadence), printed: 0, clock: None, pid, window: limit, longest };
    loop {
        let Some(since) = marks.since else {
            match incoming.recv().await {
                Some(mark) => marks.apply(mark, cadence),
                None => return,
            }
            continue;
        };
        let waited: Duration = since.elapsed();
        // A notice that is due goes out before the stall is judged, so a child ended for being quiet was announced.
        if marks.ticker.due(waited) {
            let line: String = child_wait_line(&ChildWaitFacts {
                waiting_for: waiting_for.as_str(),
                pid,
                waited,
                limit,
                limit_note: " (CLI_GIT_CHILD_WAIT_SECONDS changes this)",
                repeat: marks.printed > 0,
                ends_when_quiet: stoppable.load(Ordering::Acquire),
                left_running_after: left_running_after(flags.left_running_after.load(Ordering::Acquire)),
            });
            marks.printed += 1;
            if let Some(sink) = notices.take() {
                match or_the_stop(&mut stop, write_notice(sink, line)).await {
                    Some(kept) => notices = kept,
                    None => {
                        stop = None;
                        end_for_the_stop(&mut marks, flags.stopped.as_ref(), pid, &locks, &mut incoming, &root_pin).await;
                        continue;
                    }
                }
            }
            // The wait may have ended, or another begun, while the line was written.
            while let Ok(mark) = incoming.try_recv() {
                marks.apply(mark, cadence);
            }
            if marks.since != Some(since) {
                continue;
            }
        }
        // The clock is read at every look, a step that may not be run twice included, so its quiet gap is measured even though
        // the step is never ended. Printing can never keep a child that shows no sign of work past its window, because the
        // judgment follows the notice however long the notice took.
        // The ceiling ends the child whatever it shows and whether or not a quiet child may be ended.
        let ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire));
        if waited >= ceiling {
            // The status a wait already collected decides: a root the pin reports gone has ended, and reporting it as ended at
            //  its ceiling would turn a finished step into a failed one (the fourth review's finding 2).
            if !crate::child_watch_clock_marks_and_stopping::the_root_is_still_there(&root_pin) {
                // The wait is over, so it ends here as its own thread's end mark would: the loop then waits for the next mark.
                // Turning round instead took this branch again, because the wait stays begun and the ceiling stays passed, so the
                //  watcher held a worker and never read the end mark queued behind it (the fifth review's finding 3).
                marks.end();
                continue;
            }
            flags.ceiling_hit.store(true, Ordering::Release);
            expired.store(true, Ordering::Release);
            marks.end();
            stop_process(pid, &locks, &mut incoming, &root_pin).await;
            continue;
        }
        let verdict: Option<StopCause> = marks.clock.as_mut().and_then(Clock::check);
        let quiet: bool = match verdict {
            Some(cause) => matches!(cause, StopCause::Stalled(_) | StopCause::Deadline),
            None => false,
        };
        if quiet && stoppable.load(Ordering::Acquire) {
            // The same answer decides here: a step that ended before its quiet window has a status, and a stop does not replace it.
            if !crate::child_watch_clock_marks_and_stopping::the_root_is_still_there(&root_pin) {
                // As above: a wait that is over ends, so the loop waits for a mark rather than judging a child that has gone again.
                marks.end();
                continue;
            }
            // Raised before the child is signalled, so the waiting thread that its end releases reads it as true.
            expired.store(true, Ordering::Release);
            marks.end();
            stop_process(pid, &locks, &mut incoming, &root_pin).await;
            continue;
        }
        let until_notice: Duration = marks.ticker.next_in(since.elapsed());
        let until_ceiling: Duration = Duration::from_micros(flags.ceiling.load(Ordering::Acquire)).saturating_sub(since.elapsed());
        let sleep: Duration = until_notice.min(SAMPLE_INTERVAL).min(limit).min(until_ceiling).max(SHORTEST_SLEEP);
        match or_the_stop(&mut stop, tokio::time::timeout(sleep, incoming.recv())).await {
            Some(Ok(Some(mark))) => marks.apply(mark, cadence),
            Some(Ok(None)) => return,
            Some(Err(_slept)) => {}
            None => {
                stop = None;
                end_for_the_stop(&mut marks, flags.stopped.as_ref(), pid, &locks, &mut incoming, &root_pin).await;
            }
        }
    }
}

/// What:
///  The bound a notice names,
/// from the microseconds the watch keeps it in.
/// Why:
///  Zero is the watch's "no bound of its own",
/// and a Duration of zero microseconds would say the command stops waiting at
///  once,
/// which is not what a step the landing waits for as long as it lives does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftRunningAfter = (micros: number) => micros === 0 ? undefined : Duration.fromMicros(micros)
/// ```
fn left_running_after(micros: u64) -> Option<Duration> {
    return match micros {
        0 => None,
        kept => Some(Duration::from_micros(kept)),
    };
}

/// What:
///  Write one notice line to `sink` on the blocking pool and hand the sink back;
/// nothing where the write could not run or failed.
/// Why:
///   A notice is advisory,
/// so a failed write is ignored as it always was,
/// and a terminal that never takes the line holds a pool
///  thread,
/// not a worker of the runtime;
/// with no thread to start the notices of this watch stop (the owner,
/// 2026-11-07,
/// "Fail
///  plainly":
/// no inline run of a call that can wait).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sink = await runBlockingJob(() => { sink.write(line + '\n'); return sink; }).catch(() => undefined);
/// ```
async fn write_notice(mut sink: Box<dyn Write + Send>, line: String) -> Option<Box<dyn Write + Send>> {
    return match crate::blocking_job::run_blocking_job(move || {
        // Terminal output error: a notice is advisory; a failed write must not stop the watch.
        let _written = writeln!(sink, "{line}");
        return sink;
    })
    .await
    {
        Ok(kept) => Some(kept),
        Err(error) => {
            crate::diagnostic_log::debug("childWatch", format!("a notice was not written: {error}").as_str());
            None
        }
    };
}

/// What:
///  Awaits `work`,
/// or gives up on it and returns nothing when the service's stop is told first.
/// Why:
///  Every await of the watcher races the stop,
/// so a stop is acted on at the next await whatever the watcher was doing,
/// a terminal that holds a notice included.
/// Where there is no arm (a command of the wrapper,
/// or a step that began after the stop and must finish) the work is awaited as it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return arm ? await Promise.race([work, arm.stopTold().then(() => undefined)]) : await work;
/// ```
async fn or_the_stop<T>(stop: &mut Option<StopArm>, work: impl std::future::Future<Output = T>) -> Option<T> {
    let Some(arm) = stop.as_mut() else {
        return Some(work.await);
    };
    return tokio::select! {
        biased;
        () = arm.stop_told() => None,
        done = work => Some(done),
    };
}

/// What:
///  Ends the child because the service was stopped:
/// raises the flag its waiter reads,
/// stops watching its quiet time,
/// and signals the child and what it started,
/// with the grace and the sweep of its locks that every other stop of a step has.
/// Why:
///  The flag is raised before the signal,
/// so the thread that the child's end releases never reads a stale false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopped = true; since = undefined; await stopProcess(pid, locks, rootPin);
/// ```
async fn end_for_the_stop(
    marks: &mut Marks,
    stopped: &AtomicBool,
    pid: u32,
    locks: &LockLedger,
    incoming: &mut UnboundedReceiver<Mark>,
    root_pin: &crate::child_watch_clock_marks_and_stopping::RootPin,
) {
    stopped.store(true, Ordering::Release);
    marks.end();
    stop_process(pid, locks, incoming, root_pin).await;
}
