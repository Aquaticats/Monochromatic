//! What:
//!  The wrapper's diagnostic lines on standard error,
//!  written by a thread of their own,
//!  and never a wait for the caller that says them.
//! Why:
//!  A write to a pipe whose reader has stopped draining waits in the kernel until it resumes,
//!  and a diagnostic is said at whatever moment the code reaches it:
//!  inside a landing that holds the index lock,
//!  inside a push that holds its claim,
//!  inside a block that holds a place in the blocking pool.
//!  A caller that waited there would hold that thing for as long as the reader does not read,
//!  and every command waiting for the thing would wait with it
//!  (`doc/planning/cli-git-no-unbounded-waits.md`, "Writes to the caller's own streams").
//!  So the line is handed to a queue of bounded length and written on a thread that may wait as long as it likes;
//!  a line that finds no room is dropped and counted,
//!  and the next line that gets through says how many were dropped before it,
//!  so a gap in the diagnostics is a gap the diagnostics state.
//!  The caller waits for its own line only while the reader is still taking lines,
//!  and only up to `WARNING_WRITE_CEILING`,
//!  which keeps the order of the diagnostics among the lines written by hand beside them;
//!  a reader that was not taking a line by then is not taking any,
//!  so from the first such bound on the caller never waits again,
//!  and the lines wait in the queue for a reader that comes back or for the end of the process.
//!  The stream's own descriptor is written rather than the shared handle,
//!  because `std::io::stderr()` is one lock for the whole process:
//!  a thread blocked in a write through it would hold every other writer of this process
//!  (`standard_error_descriptor.rs`).
//!  The queue and its thread are `foundation/lines_written_by_a_thread.rs`,
//!  which the service's own log lines share;
//!  this file is the policy that makes that mechanism the diagnostics' queue.
//!  A process about to end waits,
//!  once and bounded,
//!  for the lines it handed over,
//!  because `std::process::exit` ends the thread where it stands
//!  and the last lines of a refusal are the whole of what a person gets.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! hand(`[warn] ...\n`); // waits for the line to reach the stream, at most WARNING_WRITE_CEILING
//! ```

/// The bounded queue, its thread and its control hooks,
/// which this file gives the diagnostics' words and rules.
use crate::lines_written_by_a_thread::{QueuePolicy, SharedQueue, hand, start_the_queue, wait_until_written, write_what_waits as queue_write_what_waits};
/// The queue's control hooks,
/// which only a control reads.
#[cfg(test)]
use crate::lines_written_by_a_thread::{
    dropped_so_far as queue_dropped_so_far, give_the_thread as queue_give_the_thread, queued as queue_queued, the_reader_stopped as queue_the_reader_stopped,
};
/// The bound of one write,
/// measured beside that constant.
use crate::wait_ceilings::WARNING_WRITE_CEILING;
/// The stream that is this process's standard error without the shared lock.
use crate::standard_error_descriptor::StandardErrorDescriptor;
/// Writing one line into the stream,
/// which a control of the queue gives the thread.
use std::io::Write;
/// The one-time start of the queue.
use std::sync::OnceLock;

/// The tag of this module's own lines.
const TAG: &str = "diagnosticLines";

/// The most lines that wait for the stream.
///
/// Measured on 2026-11-15 over this repository's commit timing log and the runs of the package's own gates:
/// a wrapper command writes no diagnostic line at all,
///  a verbose command writes a handful,
///  and the busiest `git worktree add` of the measurement wrote thirteen,
/// so a queue of this length holds every line a command says while a reader that never drains costs it no wait.
/// The figures are in `doc/planning/cli-git-no-unbounded-waits.md`,
/// "Writes to the caller's own streams".
pub const QUEUED_LINES: usize = 256;

/// What a line says before the next line when lines were dropped.
const DROPPED_NOTICE: &str = "monochromatic-git-policy-cli: diagnostic lines were dropped because the reader of standard error was not taking them: ";

/// What it says after the count.
const DROPPED_NOTICE_TAIL: &str = "\n";

/// How long the end of a process waits for the diagnostic lines it handed over.
/// Why:
///   The lines of a process that is ending are the last chance the person gets to read them,
/// and `std::process::exit` ends their thread where it stands;
///  the bound keeps the promise the stream was given in the first place,
/// which is that no caller waits for the reader,
///  and a process that said no line waits for nothing at all.
///  It is the same 250 ms the service's own log lines wait at the end of their process
///  (`service_log_lines_off_the_calling_thread.rs`, `WAIT_BEFORE_AN_EXIT`),
///  measured there on 2026-11-14 over sixty rounds of one refusal:
/// 3.16 ms at the median with the wait and 3.05 ms without it,
///  inside the 2.8 to 3.7 ms spread of either arm alone,
///  while a process that never handed over a line starts no thread and pays nothing.
///  So the two ends of one executable answer to one figure,
/// and a reader that is keeping up is always served inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WAIT_BEFORE_AN_EXIT = 250
/// ```
pub const WAIT_BEFORE_AN_EXIT: std::time::Duration = WARNING_WRITE_CEILING;

/// What a gap in the diagnostics says, built from the count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const droppedNotice = (dropped) => new TextEncoder().encode(DROPPED_NOTICE + dropped + DROPPED_NOTICE_TAIL);
/// ```
fn dropped_notice(dropped: u64) -> Vec<u8> {
    // What:     `format!` builds the sentence with the count inside it, and `into_bytes()` takes the UTF-8 bytes the
    //           queue keeps.
    // Why:      The queue writes bytes, and the person reads how many lines the gap holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return encoder.encode(`${DROPPED_NOTICE}${dropped}${DROPPED_NOTICE_TAIL}`);
    // ```
    return format!("{DROPPED_NOTICE}{dropped}{DROPPED_NOTICE_TAIL}").into_bytes();
}

/// The stream the lines go to when no control gave one:
/// this process's standard error written through its own descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const defaultStream = () => new StandardErrorDescriptor();
/// ```
fn default_stream() -> Box<dyn Write + Send> {
    // What:     `Box::new(StandardErrorDescriptor)` puts the descriptor writer behind the one shape the queue writes
    //           through: an owning writer that may travel to the thread.
    // Why:      The thread writes the lines, so the stream must be its own and must not be the process's shared handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new StandardErrorDescriptor();
    // ```
    return Box::new(StandardErrorDescriptor);
}

/// The diagnostics' queue:
///  its caller waits for its own line,
///  its thread retries a refused write on a tick,
///  and a stream that is gone ends the thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_POLICY = { threadName: 'cli-git-diagnostic-lines', droppedNotice, defaultStream, callersWaitForTheirLine: true, retriesOnATick: true, endsWhenTheStreamIsGone: true, threadRefused: undefined, tag: TAG };
/// ```
static THE_POLICY: QueuePolicy = QueuePolicy {
    thread_name: "cli-git-diagnostic-lines",
    dropped_notice,
    default_stream,
    callers_wait_for_their_line: true,
    retries_on_a_tick: true,
    ends_when_the_stream_is_gone: true,
    // A refusal leaves every line of this process unwritten, which is the one state this file cannot report on the stream
    //  it cannot reach, so it is not reported at all.
    thread_refused: None,
    tag: TAG,
};

/// The one queue of this process,
/// made on the first line.
static QUEUE: OnceLock<SharedQueue> = OnceLock::new();

/// The queue,
/// started on the first line.
fn queue() -> SharedQueue {
    // What:     `get_or_init` makes the queue on the first line and answers the one the process keeps afterwards.
    // Why:      A second start would give the process two threads writing two queues, so the queue is made once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return QUEUE.value ??= startTheQueue(THE_POLICY);
    // ```
    return QUEUE.get_or_init(|| return start_the_queue(&THE_POLICY));
}

/// What:
///  Hands one rendered diagnostic line to the thread that writes it,
///  and waits for it to reach the stream only while the reader is still taking lines.
/// Why:
///   The caller must not wait for a reader that has stopped draining,
/// and the lines must keep their order among the ones written by hand beside them,
///  so the wait lasts while the reader keeps up (`WARNING_WRITE_CEILING`)
/// and ends for good at the first line that reader did not take.
///  A line the reader has not taken is still the thread's,
/// so it reaches the person when the reader comes back,
///  and the lines after it wait behind it in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const write = (line) => { const ticket = hand(line); if (ticket !== undefined) waitFor(ticket, WARNING_WRITE_CEILING) }
/// ```
pub fn write(line: &str) {
    write_bytes(line.as_bytes());
}

/// What:
///  Hands one rendered line's bytes to the thread that writes it,
///  and waits for it to reach the stream only while the reader is still taking lines.
/// Why:
///   A line is not always text a `&str` can hold:
///  a Git a commit asked for its own refusal prints bytes a path or a message of the person's may put
///  outside UTF-8,
///  and decoding them would change what they read,
///  so the queue keeps bytes and this face takes them as they came.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeBytes = (line: Uint8Array) => { const ticket = hand(line); if (ticket !== undefined) waitFor(ticket, WARNING_WRITE_CEILING) }
/// ```
pub fn write_bytes(line: &[u8]) {
    let queue: SharedQueue = queue();
    // What:     `hand(...)` queues the line's own bytes and answers its number, or nothing when the queue had no room.
    // Why:      A line the queue could not take is counted there, and the next line states the gap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ticket = hand(queue, line.slice(), QUEUED_LINES);
    // ```
    let Some(ticket) = hand(queue, line.to_vec(), QUEUED_LINES) else {
        return;
    };
    // What:     `wait_until_written(...)` waits, bounded, for exactly this line to reach the stream.
    // Why:      A later line's write must not be taken for this one, and the caller waits for its own line alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // waitUntilWritten(queue, ticket, WARNING_WRITE_CEILING);
    // ```
    wait_until_written(queue, ticket, WARNING_WRITE_CEILING);
}

/// What:
///  Waits,
///  bounded,
///  for every byte of every line handed over to reach the stream,
///  and says whether it did.
/// Why:
///   A process that ends at once after its last line loses that line otherwise:
/// the line is in a thread of its own,
///  and `std::process::exit` ends every thread where it stands.
///  A process that never handed over a line starts no thread and waits for nothing,
/// which is every ordinary wrapper command.
/// The wait belongs only at the end of the process,
/// and never to a caller that says a line and goes on.
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

/// How many lines are waiting for the stream,
/// which a control reads to show the queue is bounded.
#[cfg(test)]
pub fn queued() -> usize {
    return queue_queued(queue());
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads.
#[cfg(test)]
pub fn dropped_so_far() -> u64 {
    return queue_dropped_so_far(queue());
}

/// What:
///  Gives the thread a stream of the control's own,
///  and puts back the wait a caller gives a reader that keeps up.
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

/// Whether a caller of this process stopped waiting for the reader,
/// which a control reads and no release path asks for.
#[cfg(test)]
pub fn the_reader_stopped() -> bool {
    return queue_the_reader_stopped(queue());
}

/// The queue's own behaviour is a control in the test module.
#[cfg(test)]
#[path = "diagnostic_lines_off_the_calling_thread_tests.rs"]
mod tests;
