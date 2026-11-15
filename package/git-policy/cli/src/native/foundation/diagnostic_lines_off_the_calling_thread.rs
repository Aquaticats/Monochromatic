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

/// Writing one line into the stream,
/// and the error it may give.
use std::io::Write;
/// The queue,
/// the condition the thread waits on,
/// and the one-time start.
use std::sync::{Condvar, Mutex, OnceLock};
/// The lines waiting for the stream.
use std::collections::VecDeque;
/// The bound of one write,
/// measured beside that constant.
use crate::wait_ceilings::WARNING_WRITE_CEILING;
/// The stream that is this process's standard error without the shared lock.
use crate::standard_error_descriptor::StandardErrorDescriptor;

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

/// What the queue says before the next line when lines were dropped.
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

/// One line waiting for the stream,
/// with the number the caller waits for.
struct Pending {
    /// Which line this is,
    /// in the order the callers handed their lines over.
    ticket: u64,
    /// The bytes of the line,
    /// which are written whole or not at all.
    line: Vec<u8>,
}

/// What the queue of one process holds,
/// and where the lines of a control go.
struct Queue {
    /// The lines waiting for the stream,
    /// oldest first.
    lines: VecDeque<Pending>,
    /// The number the next line handed over takes.
    next_ticket: u64,
    /// The number of the last line whose every byte is in the stream.
    written: u64,
    /// How many lines found no room since the last one that got through.
    dropped: u64,
    /// Whether the thread holds a line it has not written yet.
    writing: bool,
    /// Whether a reader has stopped taking lines:
    /// the first caller whose line did not leave its hands within `WARNING_WRITE_CEILING` set it,
    /// and from then on no caller waits.
    reader_stopped: bool,
    /// Whether the stream takes nothing more:
    /// a write that failed for any reason other than a momentary lack of room
    /// leaves a stream that no wait can bring back.
    stream_is_gone: bool,
    /// Where a control's lines go,
    /// or nothing when they go to this process's standard error.
    writer: Option<Box<dyn Write + Send>>,
}

/// The queue of this process and the condition its thread waits on,
/// as the one value the process keeps,
/// and the one lock of this file.
/// Lock kept:
///  the lines waiting for the diagnostic stream,
///  how many found no room,
///  and which line the stream has reached,
///  because a caller hands a line and goes on while one thread takes lines and waits on the stream,
///  so the hand and the take are the two ends of one queue and neither may see the other's half.
///  Under it one line is pushed or popped and one number is moved;
///  the write itself,
///  which may wait for the reader as long as the reader takes,
///  runs outside it,
///  and no Git process,
///  no file and no other lock runs under it.
type TheQueueOfThisProcess = (Mutex<Queue>, Condvar);

/// The one queue of this process,
/// made on the first line.
static QUEUE: OnceLock<&'static TheQueueOfThisProcess> = OnceLock::new();

/// The queue,
/// started on the first line.
fn queue() -> &'static TheQueueOfThisProcess {
    return QUEUE.get_or_init(start_the_queue);
}

/// What:
///  The queue of this process,
///  made once and given its thread.
/// Why:
///   The thread waits on the same queue for as long as the process runs,
/// so the value has to outlive the call that made it.
///  A process that says no diagnostic line starts no thread and pays nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startTheQueue = () => { const queue = { lines: [], written: 0, dropped: 0 }; spawn(() => writeEveryLine(queue)); return queue }
/// ```
fn start_the_queue() -> &'static TheQueueOfThisProcess {
    let started: &'static TheQueueOfThisProcess = Box::leak(Box::new((
        Mutex::new(Queue {
            lines: VecDeque::new(),
            // The first line handed over is line 1, so that "the stream has reached this line" is a number that only a
            //  written line reaches.
            next_ticket: 1,
            written: 0,
            dropped: 0,
            writing: false,
            reader_stopped: false,
            stream_is_gone: false,
            writer: None,
        }),
        Condvar::new(),
    )));
    // The thread holds the queue for as long as this process runs. A refusal leaves every line of this process unwritten,
    //  which is the one state this file cannot report on the stream it cannot reach, so it is not reported at all.
    let _spawned = std::thread::Builder::new().name(String::from("cli-git-diagnostic-lines")).spawn(move || {
        write_every_line(started);
    });
    return started;
}

/// What:
///  Writes the lines of the queue for as long as the process runs,
///  and keeps whatever a stream did not take.
/// Why:
///   This thread may wait for the stream as long as the stream takes;
/// the caller that said the line does not.
///  A line leaves this thread only when all of its bytes are in the stream,
/// because the caller that waits for it waits for exactly that much,
/// and a write can end short.
///  A stream that is gone takes nothing ever again,
/// so the rest of the line is dropped and the queue answers from then on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const line = take(); while (line.hasBytes) { const wrote = tryWrite(line.bytes); if (gone) return } markWritten(line.ticket); wakeCallers() }
/// ```
fn write_every_line(started: &'static TheQueueOfThisProcess) {
    let (queue, ready) = started;
    loop {
        let next: Option<Pending> = {
            let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                if let Some(line) = held.lines.pop_front() {
                    held.writing = true;
                    break Some(line);
                }
                held = match ready.wait(held) {
                    Ok(ready_now) => ready_now,
                    Err(poisoned) => poisoned.into_inner(),
                };
            }
        };
        let Some(line) = next else {
            return;
        };
        let (mut writer, reported): (Box<dyn Write + Send>, u64) = {
            let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let dropped: u64 = std::mem::take(&mut held.dropped);
            let writer: Box<dyn Write + Send> = match held.writer.take() {
                Some(writer) => writer,
                // No control owns the stream, so the lines are this process's own standard error.
                None => Box::new(StandardErrorDescriptor),
            };
            (writer, dropped)
        };
        if reported > 0 {
            let notice: String = format!("{DROPPED_NOTICE}{reported}{DROPPED_NOTICE_TAIL}");
            if write_whole(&mut writer, notice.as_bytes(), queue, ready).is_err() {
                give_back(queue, ready, writer, line.ticket, true);
                return;
            }
        }
        let outcome: std::io::Result<()> = write_whole(&mut writer, line.line.as_slice(), queue, ready);
        let _flushed = writer.flush();
        give_back(queue, ready, writer, line.ticket, outcome.is_err());
        if outcome.is_err() {
            return;
        }
    }
}

/// What:
///  Puts the stream back where the next line takes it,
///  moves the number the stream has reached,
///  and wakes every caller waiting for a line.
/// Why:
///   A caller waits for the number of its own line,
/// so exactly one movement of that number is what ends the wait,
///  and the wake has to come after the stream took the line rather than before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBack = (writer, ticket, gone) => { lock(); queue.writer ??= writer; queue.written = ticket; queue.writing = false; if (gone) queue.stream_is_gone = true; unlock(); notifyAll() }
/// ```
fn give_back(queue: &'static Mutex<Queue>, ready: &'static Condvar, writer: Box<dyn Write + Send>, ticket: u64, stream_is_gone: bool) {
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.writer.is_none() {
        held.writer = Some(writer);
    }
    held.written = ticket;
    held.writing = false;
    if stream_is_gone {
        held.stream_is_gone = true;
    }
    drop(held);
    ready.notify_all();
}

/// What:
///  Writes every byte of `bytes`,
///  and answers the first failure that is not a signal.
/// Why:
///   A caller's wait is over only when all of its line is in the stream,
/// and a short write leaves the rest here rather than in the middle of the next line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (bytes.length > 0) { const took = write(bytes); if (took === 0) await nextAsk(); bytes = bytes.slice(took) }
/// ```
fn write_whole(writer: &mut Box<dyn Write + Send>, bytes: &[u8], queue: &'static Mutex<Queue>, ready: &'static Condvar) -> std::io::Result<()> {
    let mut left: &[u8] = bytes;
    while !left.is_empty() {
        match writer.write(left) {
            // A stream that took nothing at once is asked again after a tick rather than in a loop of its own: this process's
            //  standard error is a blocking descriptor and never answers this, so it is a stream of a control, and a control's
            //  full pipe must not become load this process puts on the machine.
            Ok(0) => wait_to_be_asked_again(queue, ready),
            Ok(taken) => left = &left[taken..],
            // A signal ended the call and nothing about the stream changed.
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            // A stream with no room now takes the rest of the line when the reader makes room, which is what a blocking
            //  descriptor does by itself; a control's stream that answers this enjoys the same tick.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => wait_to_be_asked_again(queue, ready),
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// What:
///  Waits on the queue's own condition for the tick the end of a process asks at,
///  and goes on.
/// Why:
///   The thread keeps a line a stream refused and tries again,
/// and between two attempts it must sleep somewhere that is not a loop of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sleep(THE_END_ASKS_AGAIN_EVERY)
/// ```
fn wait_to_be_asked_again(queue: &'static Mutex<Queue>, ready: &'static Condvar) {
    let held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let _woken = match ready.wait_timeout(held, THE_END_ASKS_AGAIN_EVERY) {
        Ok((ready_now, _its_time_was_up)) => ready_now,
        Err(poisoned) => poisoned.into_inner().0,
    };
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
/// const write = (line) => { const ticket = hand(line); if (ticket !== undefined && !stopped) waitFor(ticket, WARNING_WRITE_CEILING) }
/// ```
pub fn write(line: &str) {
    let (queue, ready) = queue();
    let ticket: u64 = {
        let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let ticket: u64 = held.next_ticket;
        if held.lines.len() >= QUEUED_LINES {
            held.dropped = held.dropped.saturating_add(1);
            return;
        }
        held.next_ticket = held.next_ticket.saturating_add(1);
        held.lines.push_back(Pending { ticket, line: line.as_bytes().to_vec() });
        drop(held);
        ready.notify_one();
        ticket
    };
    wait_until_written(queue, ready, ticket);
}

/// What:
///  Waits,
///  up to `WARNING_WRITE_CEILING`,
///  until the stream holds every byte of line `ticket`,
///  and remembers a line the reader did not take so that no later caller waits.
/// Why:
///   One reader that was not taking a line by the bound is a reader that is not taking lines,
/// so the wait is paid once per process and never again,
///  which is what keeps a command that says twenty lines under a held lock to one bound rather than twenty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitUntilWritten = (ticket) => { for (;;) { if (written >= ticket) return; if (deadlinePassed()) { readerStopped = true; return } waitUntil(deadline) } }
/// ```
fn wait_until_written(queue: &'static Mutex<Queue>, ready: &'static Condvar, ticket: u64) {
    let deadline: std::time::Instant = std::time::Instant::now() + WARNING_WRITE_CEILING;
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    loop {
        // A stream that is gone takes nothing more, so no wait brings a reader back.
        if held.stream_is_gone || held.reader_stopped || held.written >= ticket {
            return;
        }
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) else {
            // Said once per process, to the log this command keeps rather than to the stream it cannot reach.
            held.reader_stopped = true;
            drop(held);
            crate::diagnostic_log::debug(TAG, format!("the reader of standard error took no diagnostic line within {WARNING_WRITE_CEILING:?}, so the command stops waiting for it").as_str());
            return;
        };
        held = match ready.wait_timeout(held, left) {
            Ok((ready_now, _its_time_was_up)) => ready_now,
            Err(poisoned) => poisoned.into_inner().0,
        };
    }
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
    let Some((queue, ready)) = QUEUE.get() else { return true };
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    loop {
        if held.stream_is_gone {
            return false;
        }
        if held.lines.is_empty() && !held.writing {
            return true;
        }
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) else {
            return false;
        };
        // The thread keeps what a stream refused and tries again when it is asked to: this is the asking, and the tick is
        //  what makes a reader's room, wherever in the bound it appears, the thread's next attempt.
        ready.notify_all();
        held = match ready.wait_timeout(held, left.min(THE_END_ASKS_AGAIN_EVERY)) {
            Ok((woken, _its_time_was_up)) => woken,
            Err(poisoned) => poisoned.into_inner().0,
        };
    }
}

/// How often the end of a process asks the thread again for a line a stream refused.
/// Why:
///   A thread that answered `WouldBlock` waits to be told the stream has room,
/// and the end of the process is the last chance to tell it,
///  so the end asks again several times inside its bound rather than once.
///  The same tick is what a refused attempt waits between two tries,
/// so neither the thread nor the end of the process spins.
///  Measured against the queue of `service_log_lines_off_the_calling_thread`,
///  whose end waits the same way:
/// a reader that made room 50 ms into the wait had the line 4.6 ms later,
///  which is one tick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_END_ASKS_AGAIN_EVERY = 5
/// ```
const THE_END_ASKS_AGAIN_EVERY: std::time::Duration = std::time::Duration::from_millis(5);

/// How many lines are waiting for the stream,
/// which a control reads to show the queue is bounded.
#[cfg(test)]
pub fn queued() -> usize {
    let (queue, _ready) = queue();
    return queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).lines.len();
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads.
#[cfg(test)]
pub fn dropped_so_far() -> u64 {
    let (queue, _ready) = queue();
    return queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).dropped;
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
    let (queue, _ready) = queue();
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    held.writer = Some(writer);
    held.reader_stopped = false;
}

/// Whether a caller of this process stopped waiting for the reader,
/// which a control reads and no release path asks for.
#[cfg(test)]
pub fn the_reader_stopped() -> bool {
    let (queue, _ready) = queue();
    return queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).reader_stopped;
}

/// The queue's own behaviour is a control in the test module.
#[cfg(test)]
#[path = "diagnostic_lines_off_the_calling_thread_tests.rs"]
mod tests;
