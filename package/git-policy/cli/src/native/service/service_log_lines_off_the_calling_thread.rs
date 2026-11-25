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
//!  The queue and its thread are `foundation/lines_written_by_a_thread.rs`,
//!  which the command's diagnostic lines share;
//!  this file is the policy that makes that mechanism the service's log:
//!  its callers never wait,
//!  a stream that refused a write is asked again by the next line or the end of the process,
//!  and a stream that is gone leaves the thread taking the lines that still come.
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

/// The bounded queue, its thread and its control hooks,
/// which this file gives the service's log words and rules.
use crate::lines_written_by_a_thread::{QueuePolicy, SharedQueue, hand as queue_hand, start_the_queue, write_what_waits as queue_write_what_waits};
/// The queue's control hooks,
/// which only a control reads.
#[cfg(test)]
use crate::lines_written_by_a_thread::{
    dropped_so_far as queue_dropped_so_far, give_the_thread as queue_give_the_thread, queued as queue_queued,
};
/// Writing one line into the stream,
/// which a control of the queue gives the thread.
use std::io::Write;
/// The one-time start of the queue.
use std::sync::OnceLock;

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

/// What a gap in the log says, built from the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const droppedNotice = (dropped) => new TextEncoder().encode(DROPPED_NOTICE_HEAD + dropped + DROPPED_NOTICE_TAIL);
/// ```
fn dropped_notice(dropped: u64) -> Vec<u8> {
    // What:     `format!` builds the sentence with the count inside it, and `into_bytes()` takes the UTF-8 bytes the
    //           queue keeps.
    // Why:      The queue writes bytes, and the person reads how many entries the gap holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return encoder.encode(`${DROPPED_NOTICE_HEAD}${dropped}${DROPPED_NOTICE_TAIL}`);
    // ```
    return format!("{DROPPED_NOTICE_HEAD}{dropped}{DROPPED_NOTICE_TAIL}").into_bytes();
}

/// The stream the lines go to when no control gave one:
/// this process's standard error, which is the journal under the service's unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const defaultStream = () => process.stderr;
/// ```
fn default_stream() -> Box<dyn Write + Send> {
    // What:     `Box::new(std::io::stderr())` puts the process's error handle behind the one shape the queue writes
    //           through: an owning writer that may travel to the thread.
    // Why:      The thread writes the lines, so the stream must be its own value rather than a borrow of the caller's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return process.stderr;
    // ```
    return Box::new(std::io::stderr());
}

/// The service's log queue:
///  its callers never wait,
///  a refused write waits for the next ask rather than a tick,
///  and a stream that is gone leaves the thread running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_POLICY = { threadName: 'service log lines', droppedNotice, defaultStream, callersWaitForTheirLine: false, retriesOnATick: false, endsWhenTheStreamIsGone: false, threadRefused: ['serviceLog', '...'], tag: 'serviceLog' };
/// ```
static THE_POLICY: QueuePolicy = QueuePolicy {
    thread_name: "service log lines",
    dropped_notice,
    default_stream,
    callers_wait_for_their_line: false,
    retries_on_a_tick: false,
    ends_when_the_stream_is_gone: false,
    thread_refused: Some((
        "serviceLog",
        "the thread that writes the service's log lines could not be started",
    )),
    tag: "serviceLog",
};

/// The one queue of this process,
/// made on the first line.
static QUEUE: OnceLock<SharedQueue> = OnceLock::new();

/// The queue,
/// started on the first line.
fn queue() -> SharedQueue {
    return QUEUE.get_or_init(|| start_the_queue(&THE_POLICY));
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
    // What:     `hand(...)` queues the line's own bytes and answers its number, or nothing when the queue had no room.
    // Why:      The service's callers never wait, so the number is not read; whether it came back is the whole answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return handBytes(line) !== undefined;
    // ```
    return queue_hand(queue(), line.as_bytes().to_vec(), QUEUED_LINES).is_some();
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads and no release path asks for.
#[cfg(test)]
pub fn dropped_so_far() -> u64 {
    return queue_dropped_so_far(queue());
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
    let Some(queue) = QUEUE.get() else { return true };
    return queue_write_what_waits(queue, within);
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
    queue_give_the_thread(queue(), writer);
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
    return queue_queued(queue());
}

/// What the thread does with a full stream is a control in the test module.
#[cfg(test)]
#[path = "service_log_lines_off_the_calling_thread_tests.rs"]
mod tests;
