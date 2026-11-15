//! What:
//!  The service's log lines on standard error,
//!  written by a thread of their own,
//!  so no caller waits for the stream.
//! Why:
//!  A landing wrote its entries straight to standard error with `write_all`,
//! which waits for the reader
//!  for as long as the reader does not read.
//! With the service's standard error connected to a FIFO that was full,
//! a landing was held 7.35 seconds
//!  longer than real Git took,
//!  until the FIFO was drained
//! (the audit of `0.6.1-alpha.14`,
//! `pi-runs/kimi-audit-alpha-14-commit-path-1`).
//! When the service runs as a unit its standard error is the journal,
//! and the journal's disk was the busy one while the commits of 2026-11-12 took
//!  56 to 199 seconds.
//!  A line nobody can afford to wait for is handed to a queue of bounded length and written on a thread that
//! may wait as long as it likes;
//! an entry that finds no room is dropped and counted,
//! and the next entry that gets through
//!  says how many were dropped before it,
//! so a gap in the log is a gap the log states.
//!  The stream's own file description is shared,
//! so the thread writes to the same descriptor the caller would have written to,
//! and nothing about where the lines go changes.
//!  A process about to end waits,
//! once and bounded,
//! for the lines it handed over,
//! because `std::process::exit` ends the thread where it stands
//! and a refusal's last line is the whole of what a person gets.
//!  A line is the thread's until all of its bytes are in the stream:
//! a write a stream refused leaves the rest of the line here,
//! and nothing else is written until the rest of it is,
//! so the stream holds one line whole or a fragment whose remainder is still to come.
//! A fragment is the one thing a reader can be left with,
//! and only when the process ends before the rest of the line reaches the stream:
//! it has no newline of its own,
//! so the next line written into that stream begins right after it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! hand(`${stamped}\n`); // false: dropped and counted
//! ```

/// Writing one line into a stream,
/// and the error it may give.
use std::io::Write;
/// The queue,
/// the condition the thread waits on,
/// and the one-time start.
use std::sync::{Condvar, Mutex, OnceLock};
/// The queue of lines waiting for the stream.
use std::collections::VecDeque;

/// The most lines that wait for the stream.
///
/// Measured on 2026-11-12 over the service's own journal for the six slowest commits of that day:
/// one landing writes a handful of entries,
/// and the busiest minute of any of them holds fewer than a hundred,
/// so a queue of this length holds everything a landing writes
///  while a stream that never drains never costs the landing a wait.
/// 256 lines of a thousand bytes each are a quarter of a mebibyte,
/// the same kind of bound the timing record keeps.
pub const QUEUED_LINES: usize = 256;

/// What a line that was dropped says when the next one is written.
const DROPPED_NOTICE_HEAD: &str = "monochromatic-git-policy-cli-service: ";

/// What it says after the count.
const DROPPED_NOTICE_TAIL: &str = " log entries were dropped before this one because the log stream had no room\n";

/// How long the end of a process waits for the lines it handed over.
///
/// Measured on 2026-11-14 on this host over sixty rounds of one refusal,
/// the arm with this wait and the arm without it alternating in every round:
/// the whole process took 3.16 ms at the median with the wait and 3.05 ms without it,
/// and the spread of either arm alone (2.8 to 3.7 ms between the tenth and ninetieth percentiles)
/// is larger than that difference,
/// while a process that never handed over a line starts no thread and pays nothing.
/// 250 ms is above any such write by two orders of magnitude,
/// so a reader that is keeping up is always served;
/// it is also the service's own answer deadline (`DRAWN_ANSWER_DEADLINE`,
/// `src/native/service/service_client_shadow_hand_out.rs`),
/// so no script that runs the wrapper notices the end of one,
/// and a stream nobody drains costs this at most once,
/// at the end,
/// never per line.
pub const WAIT_BEFORE_AN_EXIT: std::time::Duration = std::time::Duration::from_millis(250);

/// How often the end of a process asks the thread again for a line a stream refused.
///
/// A wait is what it says:
/// the answer is `true` only when every byte of every line is in the stream,
/// and a stream that refused a line may take it a moment later,
/// so the end asks again rather than answering on the thread's first attempt.
/// The bound it asks within is `WAIT_BEFORE_AN_EXIT`,
/// and this is a fiftieth of it.
/// Measured on 2026-11-15 against this queue:
/// a reader that made room 50 ms into the wait had the line 4.6 ms later,
/// which is one tick,
/// and an end of a process with no room at all waited 250.06 ms of its 250 ms bound.
/// So one bound holds the thread to at most `WAIT_BEFORE_AN_EXIT / THE_END_ASKS_AGAIN_EVERY` attempts,
/// a handful of syscalls at the very end of a process and none during its work.
pub const THE_END_ASKS_AGAIN_EVERY: std::time::Duration = std::time::Duration::from_millis(5);

/// What the queue of one process holds,
/// and where the lines of a control go.
struct Queue {
    /// The lines waiting for the stream,
    /// oldest first.
    lines: VecDeque<Vec<u8>>,
    /// How many lines found no room since the last one that got through.
    dropped: u64,
    /// Whether the stream takes nothing more:
    /// a write that failed for any reason other than a momentary lack of room
    /// leaves a stream that no wait can bring back,
    /// so the end of the process is answered `false` at once rather than at its bound.
    stream_is_gone: bool,
    /// Whether the thread holds a line it has not written yet.
    /// The queue being empty is not enough for a caller that waits for the stream:
    /// the last line is out of the queue and in the stream only once this is false again.
    writing: bool,
    /// Where the control's lines go,
    /// or nothing when they go to this process's standard error.
    /// A control sets it once,
    /// before its first line,
    /// so the thread it starts writes into a stream the control holds.
    writer: Option<Box<dyn Write + Send>>,
}

/// The one queue of this process and the condition its thread waits on.
/// Lock kept:
///  the lines waiting for the service's own log stream,
///  and how many found no room,
///  because a caller hands a line and goes on while one thread takes lines and waits on the stream,
///  so the hand and the take are the two ends of one queue and neither may see the other's half.
///  Under it one line is pushed or popped and the writer is handed out or given back;
///  the write itself,
///  which may wait for the reader as long as the reader takes,
///  runs outside it,
///  and no Git process,
///  no file and no other lock runs under it.
static QUEUE: OnceLock<&'static (Mutex<Queue>, Condvar)> = OnceLock::new();

/// The queue,
/// started on the first line.
fn queue() -> &'static (Mutex<Queue>, Condvar) {
    return QUEUE.get_or_init(start_the_queue);
}

/// What:
///  The queue of this process,
///  made once and given its thread.
/// Why:
///   The thread waits on the same queue for as long as the process runs,
/// so the value has to outlive the call that made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startTheQueue = () => { const queue = { lines: [], dropped: 0, writer: null }; spawn(() => writeEveryLine(queue)); return queue }
/// ```
fn start_the_queue() -> &'static (Mutex<Queue>, Condvar) {
    {
        let started: &'static (Mutex<Queue>, Condvar) = Box::leak(Box::new((
            Mutex::new(Queue { lines: VecDeque::new(), dropped: 0, stream_is_gone: false, writing: false, writer: None }),
            Condvar::new(),
        )));
        // The thread holds the queue for as long as this process runs; a failure to start it leaves the lines uncounted,
        // which is the one state this file cannot report, so the process says it once and goes on writing by hand.
        let spawned = std::thread::Builder::new().name(String::from("service log lines")).spawn(move || {
            write_every_line(started);
        });
        if spawned.is_err() {
            crate::diagnostic_log::warn("serviceLog", "the thread that writes the service's log lines could not be started");
        }
        return started;
    }
}

/// What:
///  Writes the lines of the queue for as long as the process runs,
///  and keeps whatever a stream did not take.
/// Why:
///   This thread may wait for the stream as long as the stream takes;
/// the landing that handed it a line does not.
///  A line leaves this thread only when all of its bytes are in the stream,
/// because the end of a process promises exactly that much and a write can end short:
/// a stream with no room now (`WouldBlock`) takes the rest of the line later,
/// so the rest stays here and this thread is asked again,
/// by the next line or by the end of the process;
/// a partial write takes what it took,
/// and the remainder stays here,
/// so nothing else is written into the middle of one line;
/// a signal that ended the call (`Interrupted`) changed nothing about the stream,
/// so the attempt is made again at once;
/// and a stream that is gone (a closed pipe,
/// a descriptor nothing holds) takes nothing ever again,
/// so the rest of the line is dropped and the whole queue answers `false` from then on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const line = take(); if (!line) return; while (line.hasBytes) { const wrote = tryWrite(line.bytes); if (ended) return; if (!wrote) waitToBeAskedAgain() } }
/// ```
fn write_every_line(started: &'static (Mutex<Queue>, Condvar)) {
    let (queue, ready) = started;
    loop {
        let next: Option<Vec<u8>> = {
            let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                if let Some(line) = held.lines.pop_front() {
                    let dropped: u64 = std::mem::take(&mut held.dropped);
                    // Out of the queue and not yet in the stream: a caller that waits for the stream has to be told the
                    //  difference between the two.
                    held.writing = true;
                    break Some(if dropped == 0 { line } else { [format!("{DROPPED_NOTICE_HEAD}{dropped}{DROPPED_NOTICE_TAIL}").as_bytes(), line.as_slice()].concat() });
                }
                // The queue is empty, and this thread waits for the next line rather than ending: a second line after a pause
                // is the ordinary case for a service that runs for hours.
                held = match ready.wait(held) {
                    Ok(ready_now) => ready_now,
                    Err(poisoned) => poisoned.into_inner(),
                };
            }
        };
        let Some(line) = next else {
            return;
        };
        let mut writer: Box<dyn Write + Send> = {
            let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            match held.writer.take() {
                Some(writer) => writer,
                // No control owns the stream, so the lines are this process's own standard error.
                None => Box::new(std::io::stderr()),
            }
        };
        // What: the bytes of this line the stream has not taken yet.
        // Why:  a line leaves this thread only when all of it is in the stream.
        let mut unwritten: &[u8] = line.as_slice();
        while !unwritten.is_empty() {
            match writer.write(unwritten) {
                // A stream that took nothing is one to ask again later, not one that refused.
                Ok(0) => {}
                Ok(taken) => {
                    unwritten = &unwritten[taken..];
                    continue;
                }
                // A signal ended the call and nothing about the stream changed.
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                // A stream with no room now: the rest of the line waits here.
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                // The stream is gone, and no wait brings a reader back: the rest of this line is dropped and the queue
                //  says so to whoever asks.
                Err(_gone) => {
                    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    held.stream_is_gone = true;
                    drop(held);
                    break;
                }
            }
            // Nothing more can be written now, so this thread waits to be asked again, by the next line handed over or
            //  by the end of the process, and never on the stream itself.
            let held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let _woken = match ready.wait(held) {
                Ok(ready_now) => ready_now,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
        let _flushed = writer.flush();
        let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if held.writer.is_none() {
            held.writer = Some(writer);
        }
        held.writing = false;
        drop(held);
        // A caller waiting at the end of the process is woken by the very line it waits for.
        ready.notify_all();
    }
}

/// What:
///  Hands one line to the thread that writes it,
///  and says whether it was taken.
/// Why:
///   A caller hands its line and goes on;
/// a queue that is full counts the line instead of waiting for room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hand = (line) => { if (queue.length < QUEUED_LINES) { queue.push(line); notify(); return true } dropped++; return false }
/// ```
pub fn hand(line: &str) -> bool {
    return hand_bytes(line.as_bytes().to_vec());
}

/// What:
///  `hand` for a line already in bytes,
///  which is the shape the queue keeps.
/// Why:
///   The line is moved into the queue once and never copied again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handBytes = (line: Uint8Array) => queue.length < QUEUED_LINES ? (queue.push(line), true) : (dropped++, false)
/// ```
fn hand_bytes(line: Vec<u8>) -> bool {
    let (queue, ready) = queue();
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.lines.len() >= QUEUED_LINES {
        held.dropped = held.dropped.saturating_add(1);
        return false;
    }
    held.lines.push_back(line);
    drop(held);
    ready.notify_one();
    return true;
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads and no release path asks for.
#[cfg(test)]
pub fn dropped_so_far() -> u64 {
    let (queue, _ready) = queue();
    return queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).dropped;
}

/// What:
///  Waits,
///  bounded,
///  for every byte of every line handed over to reach the stream,
///  and says whether it did.
/// Why:
///   A process that ends at once after its last line loses that line otherwise:
/// the line is out of the queue and in a thread of its own,
/// and `std::process::exit` ends every thread where it stands.
///  Its answer is the whole of what it promises:
/// `true` means every byte of every line handed over is in the stream,
/// which is why a line a stream has no room for yet is still the thread's when the wait begins,
/// and why the wait asks the thread again at `THE_END_ASKS_AGAIN_EVERY`
/// instead of answering on its first attempt.
///  A stream that is gone is answered `false` at once and not waited for,
/// because every write to it fails the same way and no wait brings a reader back.
/// The wrapper's service mode refuses a start with one line and returns its exit status,
/// and the refusals of 2026-11-14 (a `--git` that is the wrapper,
/// a start with no socket and no address)
/// printed no byte at all in most runs:
/// the person was owed the reason and the way forward,
/// and the words never arrived.
/// The wait belongs only at the end of the process,
/// and never to a caller that hands a line and goes on.
/// A process that never handed a line starts no thread and waits for nothing,
/// so this costs an ordinary wrapper command nothing at all.
/// The bound is the promise:
/// a stream a reader drains is written at once,
/// and a stream nobody drains holds the end of the process for `within` and no longer,
/// which is what keeps this safe where the 7.35 second wait of 2026-11-12 came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeWhatWaits = (within) => (queueEmpty && !writing) || waitedAsLongAs(within)
/// ```
pub fn write_what_waits(within: std::time::Duration) -> bool {
    // No line was ever handed over, so no thread exists to wait for, and starting one here would be the cost this
    //  function exists to avoid.
    let Some((queue, ready)) = QUEUE.get() else { return true };
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    loop {
        // A stream that is gone takes nothing more, so no wait brings a reader back and the answer is owed now.
        if held.stream_is_gone {
            return false;
        }
        if held.lines.is_empty() && !held.writing {
            return true;
        }
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) else {
            return false;
        };
        // The thread keeps what a stream refused and tries again when it is asked to: this is the asking, and the tick
        //  is what makes a reader's room, wherever in the bound it appears, the thread's next attempt.
        ready.notify_all();
        held = match ready.wait_timeout(held, left.min(THE_END_ASKS_AGAIN_EVERY)) {
            Ok((woken, _its_time_was_up)) => woken,
            Err(poisoned) => poisoned.into_inner().0,
        };
    }
}

/// What:
///  Gives the thread a stream of the control's own,
///  once.
/// Why:
///   A control holds the read end of a pipe and never drains it,
/// so the thread waits there and the queue's bound is what the control reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(testWriter)
/// ```
#[cfg(test)]
pub fn give_the_thread(writer: Box<dyn Write + Send>) {
    let (queue, _ready) = queue();
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    held.writer = Some(writer);
    drop(held);
}

/// What:
///  How many lines are waiting for the stream,
///  which a control reads to show the queue is bounded.
/// Why:
///   The bound is the whole of the promise a caller is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queued()
/// ```
#[cfg(test)]
pub fn queued() -> usize {
    let (queue, _ready) = queue();
    return queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).lines.len();
}

/// What the thread does with a full stream is a control in the test module.
#[cfg(test)]
#[path = "service_log_lines_off_the_calling_thread_tests.rs"]
mod tests;
