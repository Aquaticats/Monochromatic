//! What:
//!  Lines a caller hands over and a thread of its own writes,
//!  so the caller never waits for a stream:
//! the bounded queue,
//! the thread that writes it,
//! the count of the lines it could not take,
//! and the wait the end of a process makes for it.
//! Why:
//!  The wrapper has two such queues,
//!  the diagnostic lines of a command (`diagnostic_lines_off_the_calling_thread.rs`)
//! and the service's own log lines (`service_log_lines_off_the_calling_thread.rs`),
//!  and both are one mechanism:
//! a write to a stream whose reader has stopped waits in the kernel for as long as the reader does not read,
//! and a line said under a held lock would hold that lock for it.
//!  They differ in what the lines are for:
//! whether a caller waits for its own line,
//! which stream they go to when no control gave one,
//! what a gap in them says,
//! and whether the thread ends when the stream does.
//!  Those differences are the `QueuePolicy` a queue is made with;
//! everything else is written here once,
//!  which is what keeps one queue's fix from leaving the other behind
//! (debt ledger of 2026-11-25, item 13).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const queue = startTheQueue(policy); hand(queue, line); writeWhatWaits(queue, 250);
//! ```

/// Writing one line into a stream,
/// and the error it may give.
use std::io::Write;
/// The queue,
/// the condition the thread waits on,
/// and the type both owners keep their one queue in.
use std::sync::{Condvar, Mutex};
/// The lines waiting for the stream.
use std::collections::VecDeque;

/// What:
///  The queue of one process,
/// and the condition its thread waits on,
///  as the value each owner keeps in its own `OnceLock`.
/// Why:
///   The queue, its thread and the callers that hand lines over all reach the same two values,
/// so they travel as one type;
///  each owner keeps its own,
/// and neither reaches the other's.
/// Lock kept:
///  the lines waiting for this queue's stream,
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedQueue = { lines: Pending[]; written: number; dropped: number; writer?: Writable };
/// ```
pub(crate) type SharedQueue = &'static (Mutex<Queue>, Condvar);

/// What:
///  How one queue differs from the other:
///  the words it says,
///  the stream it writes to,
///  and how its callers and its thread wait.
/// Why:
///   The two queues are one mechanism with two purposes,
/// and a difference written as a value here cannot drift into one of them by accident;
///  every field below is a sentence of one file's documentation
///  (`diagnostic_lines_off_the_calling_thread.rs`, `service_log_lines_off_the_calling_thread.rs`)
///  made readable by the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QueuePolicy = { threadName: string; droppedNotice: (n: number) => Uint8Array; defaultStream: () => Writable; callersWait: boolean; retriesOnATick: boolean; endsWhenTheStreamIsGone: boolean; threadRefused?: [string, string]; tag: string };
/// ```
pub(crate) struct QueuePolicy {
    /// The name the thread carries,
    /// which is what a person sees beside the process's threads.
    pub(crate) thread_name: &'static str,
    /// What a line says before it when lines were dropped,
    /// built from the count.
    pub(crate) dropped_notice: fn(u64) -> Vec<u8>,
    /// The stream the lines go to when no control gave one.
    pub(crate) default_stream: fn() -> Box<dyn Write + Send>,
    /// Whether a caller waits for its own line to reach the stream,
    /// and so needs the queue to remember which line the stream has reached.
    pub(crate) callers_wait_for_their_line: bool,
    /// Whether a write the stream refused is tried again on the queue's own tick,
    /// rather than only when a line or the end of the process asks.
    pub(crate) retries_on_a_tick: bool,
    /// Whether the thread ends when the stream takes nothing ever again.
    pub(crate) ends_when_the_stream_is_gone: bool,
    /// What says the thread could not be started,
    /// as the diagnostic tag and the sentence,
    /// or nothing when that is left unsaid.
    pub(crate) thread_refused: Option<(&'static str, &'static str)>,
    /// The tag of this queue's own diagnostic lines.
    pub(crate) tag: &'static str,
}

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
pub(crate) struct Queue {
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
    /// the first caller whose line did not leave its hands within the wait it is given
    ///  set it,
    /// and from then on no caller waits.
    reader_stopped: bool,
    /// Whether the stream takes nothing more:
    /// a write that failed for any reason other than a momentary lack of room
    /// leaves a stream that no wait can bring back.
    stream_is_gone: bool,
    /// Where a control's lines go,
    /// or nothing when they go to the stream this queue defaults to.
    writer: Option<Box<dyn Write + Send>>,
    /// What this queue is for,
    /// which is the only place the two queues differ.
    policy: &'static QueuePolicy,
}

/// How often the end of a process asks the thread again for a line a stream refused.
/// Why:
///   A thread that answered `WouldBlock` waits to be told the stream has room,
/// and the end of the process is the last chance to tell it,
///  so the end asks again several times inside its bound rather than once;
///  the same tick is what a refused attempt waits between two tries,
///  where a queue's policy retries on it.
///  Measured against both queues on 2026-11-15:
/// a reader that made room 50 ms into the wait had the line 4.6 ms later,
///  which is one tick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_END_ASKS_AGAIN_EVERY = 5
/// ```
pub(crate) const THE_END_ASKS_AGAIN_EVERY: std::time::Duration = std::time::Duration::from_millis(5);

/// What:
///  The queue of one process,
///  made once and given its thread.
/// Why:
///   The thread waits on the same queue for as long as the process runs,
/// so the value has to outlive the call that made it;
///  a process that says no line starts no thread and pays nothing.
///  A thread the system refuses leaves every line of this process unwritten,
/// which the policy either says once or leaves unsaid
///  (the stream it cannot reach is where it would be said).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startTheQueue = (policy) => { const queue = { lines: [], written: 0, dropped: 0, writer: null, policy }; spawn(policy.threadName, () => writeEveryLine(queue)); return queue };
/// ```
pub(crate) fn start_the_queue(policy: &'static QueuePolicy) -> SharedQueue {
    // What:     `Box::leak(Box::new((..)))` gives the queue and its condition a lifetime the process owns,
    //           which the thread's closure and every later caller borrow.
    // Why:      The thread outlives the call that starts it, so the value it waits on must outlive the call too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const started = { lines: [], nextTicket: 1, written: 0, dropped: 0, writing: false, readerStopped: false, streamIsGone: false, writer: null, policy };
    // ```
    let started: SharedQueue = Box::leak(Box::new((
        Mutex::new(Queue {
            lines: VecDeque::new(),
            // The first line handed over is line 1, so that "the stream has reached this line" is a number only a
            //  written line reaches.
            next_ticket: 1,
            written: 0,
            dropped: 0,
            writing: false,
            reader_stopped: false,
            stream_is_gone: false,
            writer: None,
            policy,
        }),
        Condvar::new(),
    )));
    // What:     `Builder::new().name(...).spawn(...)` starts the one thread that writes this queue for the process's life.
    // Why:      Writing is what may wait for a reader, so it is what must not hold a caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spawned = spawn(policy.threadName, () => writeEveryLine(started));
    // ```
    let spawned: std::io::Result<std::thread::JoinHandle<()>> = std::thread::Builder::new()
        .name(String::from(policy.thread_name))
        .spawn(move || {
            write_every_line(started);
        });
    // What:     `if let Some((tag, sentence)) = policy.thread_refused` says once that the lines will never be written,
    //           where the queue's own policy says it at all.
    // Why:      A caller that hands a line would otherwise believe it was shown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (spawned.kind === 'error' && policy.threadRefused) warn(...policy.threadRefused);
    // ```
    if spawned.is_err()
        && let Some((tag, sentence)) = policy.thread_refused
    {
        crate::diagnostic_log::warn(tag, sentence);
    }
    // What:     `started` is the queue every later call reads and the thread waits on.
    // Why:      One value for both ends of the queue, which is what makes them one queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return started;
    // ```
    return started;
}

/// What:
///  Writes the lines of the queue for as long as the process runs,
///  and keeps whatever a stream did not take.
/// Why:
///   This thread may wait for the stream as long as the stream takes;
/// the caller that handed a line does not.
///  A line leaves this thread only when all of its bytes are in the stream,
/// because the caller that waits for it,
///  where a queue's callers wait,
/// waits for exactly that much,
///  and a write can end short.
///  A stream that is gone takes nothing ever again,
/// so the rest of the line is dropped and the queue answers from then on,
///  and only the queue whose policy says so ends its thread there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const { line, dropped } = take(); const writer = stream(); if (dropped) writeAll(notice(dropped)); const failed = writeAll(line); markWritten(line.ticket, failed); if (failed && policy.endsWhenTheStreamIsGone) return }
/// ```
fn write_every_line(started: SharedQueue) {
    let (queue, ready) = started;
    // What:     `policy` is the value this queue was made with, read once for the thread's whole life.
    // Why:      The words and the two ends' rules never change, so the thread reads them before its first line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const policy = queue.policy;
    // ```
    let policy: &'static QueuePolicy = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).policy;
    loop {
        // What:     `(line, dropped)` is the next line and the count of lines dropped before it,
        //           read under the one lock that takes the line.
        // Why:      A line and the gap before it belong together, so no caller's line can slide between them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const { line, dropped } = takeALine();
        // ```
        let (line, dropped): (Pending, u64) = take_a_line(queue, ready);
        // What:     `take_the_stream(...)` is the control's stream, or this queue's default one.
        // Why:      The writer is handed out for one line and given back, so a control can replace it between lines.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const writer = takeTheStream();
        // ```
        let mut writer: Box<dyn Write + Send> = take_the_stream(queue);
        // What:     `if dropped > 0` writes the gap's sentence before the line.
        // Why:      A gap in the lines is a gap the lines state, and only before the line that follows it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (dropped > 0) { const notice = policy.droppedNotice(dropped); const failed = writeWhole(writer, notice); if (failed) { giveBack(writer, line.ticket, true); if (policy.endsWhenTheStreamIsGone) return; continue } }
        // ```
        if dropped > 0 {
            // What:     `notice` is this queue's own sentence about the gap, with the count in it.
            // Why:      The two queues word a gap differently, and the policy is where that is written.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const notice = policy.droppedNotice(dropped);
            // ```
            let notice: Vec<u8> = (policy.dropped_notice)(dropped);
            if write_whole(&mut writer, notice.as_slice(), queue, ready).is_err() {
                give_back(queue, ready, writer, line.ticket, true);
                if policy.ends_when_the_stream_is_gone {
                    return;
                }
                continue;
            }
        }
        // What:     `write_whole(...)` writes every byte of the line, or answers the failure that stopped it.
        // Why:      A caller that waits for its line waits for all of it, and a half-written line must not be followed by another.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const failed = writeWhole(writer, line.line);
        // ```
        let written: std::io::Result<()> = write_whole(&mut writer, line.line.as_slice(), queue, ready);
        // What:     `writer.flush()` pushes what the stream buffers, and the failure of a flush is not the line's.
        // Why:      A line already in the stream is written; a flush that fails changes nothing about what was taken.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // writer.flush();
        // ```
        let _flushed = writer.flush();
        // What:     `give_back(...)` puts the writer back, moves the number the stream has reached and wakes the callers.
        // Why:      Exactly one movement of that number ends the wait of the caller whose line it is.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // giveBack(writer, line.ticket, failed);
        // ```
        give_back(queue, ready, writer, line.ticket, written.is_err());
        if written.is_err() && policy.ends_when_the_stream_is_gone {
            return;
        }
    }
}

/// What:
///  The next line to write and the count of lines dropped before it,
///  waiting for the thread until a line is handed over.
/// Why:
///   A queue is empty between two lines and the thread must wait there rather than end;
///  the pop and the count are one step, so a gap is stated by the line right after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeALine = () => { lock(); for (;;) { const line = lines.shift(); if (line) { writing = true; return { line, dropped: take(dropped) } } wait(); } }
/// ```
fn take_a_line(queue: &'static Mutex<Queue>, ready: &'static Condvar) -> (Pending, u64) {
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    loop {
        // What:     `pop_front()` takes the oldest line, which is the order the callers handed them over in.
        // Why:      The lines are written in the order they were said, and the queue is what keeps that order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const line = held.lines.shift();
        // ```
        if let Some(line) = held.lines.pop_front() {
            // What:     `std::mem::take(&mut held.dropped)` reads the count and leaves zero for the next gap.
            // Why:      A drop is said once, by the next line that gets through.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const dropped = held.dropped; held.dropped = 0;
            // ```
            let dropped: u64 = std::mem::take(&mut held.dropped);
            // What:     `held.writing = true` says the line left the queue and is not yet in the stream.
            // Why:      A caller that waits at the end of the process must tell the two apart.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // held.writing = true;
            // ```
            held.writing = true;
            // What:     The line and its gap are the answer, and the loop ends.
            // Why:      Nothing else may be read under this lock.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return { line, dropped };
            // ```
            return (line, dropped);
        }
        // What:     `ready.wait(held)` sleeps until a caller hands a line or the end of the process asks again.
        // Why:      The thread must not spin while the queue is empty, and a poisoned lock still holds a usable queue.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // held = await ready.wait(held);
        // ```
        held = match ready.wait(held) {
            Ok(ready_now) => ready_now,
            Err(poisoned) => poisoned.into_inner(),
        };
    }
}

/// What:
///  The stream the next line goes to:
/// the control's if one was given,
/// and otherwise the one this queue's policy defaults to.
/// Why:
///   A control holds the read end of a pipe and reads what the thread wrote;
/// a process without one writes where it always wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeTheStream = () => held.writer ?? policy.defaultStream();
/// ```
fn take_the_stream(queue: &'static Mutex<Queue>) -> Box<dyn Write + Send> {
    let mut held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    // What:     `held.writer.take()` gives this line the control's stream and leaves none behind, so it is handed back below.
    // Why:      One writer travels with one line, which is what lets a control replace it between lines.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const own = held.writer; held.writer = undefined;
    // ```
    return match held.writer.take() {
        Some(writer) => writer,
        // What:     `(policy.default_stream)()` makes the stream this queue writes to by itself.
        // Why:      A process that no control watches must still show its lines.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // None: (held.policy.defaultStream)();
        // ```
        None => (held.policy.default_stream)(),
    };
}

/// What:
///  Puts the stream back where the next line takes it,
///  moves the number the stream has reached,
///  and wakes every caller waiting for a line.
/// Why:
///   A caller waits for the number of its own line,
/// so exactly one movement of that number is what ends the wait,
///  and the wake comes after the stream took the line rather than before.
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
///  A stream that takes nothing at once is asked again after a tick where the policy retries on one,
///  and otherwise waits for the next line or the end of the process to ask;
///  a stream with no room now takes the rest when the reader makes room,
///  which is what a blocking descriptor does by itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// while (bytes.length > 0) { const took = write(bytes); if (took === 0) await nextAsk(); bytes = bytes.slice(took) }
/// ```
fn write_whole(
    writer: &mut Box<dyn Write + Send>,
    bytes: &[u8],
    queue: &'static Mutex<Queue>,
    ready: &'static Condvar,
) -> std::io::Result<()> {
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
            // A stream with no room now takes the rest of the line when the reader makes room.
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => wait_to_be_asked_again(queue, ready),
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

/// What:
///  Waits on the queue's own condition until the stream may be asked again,
///  and goes on.
/// Why:
///   The thread keeps a line a stream refused and tries again,
/// and between two attempts it must sleep somewhere that is not a loop of its own.
///  A queue whose policy retries on a tick sleeps for one and tries again by itself;
/// one that does not waits to be asked, by the next line or the end of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await policy.retriesOnATick ? sleep(THE_END_ASKS_AGAIN_EVERY) : nextAsk()
/// ```
fn wait_to_be_asked_again(queue: &'static Mutex<Queue>, ready: &'static Condvar) {
    let held = queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let retries_on_a_tick: bool = held.policy.retries_on_a_tick;
    let _woken = match retries_on_a_tick {
        true => match ready.wait_timeout(held, THE_END_ASKS_AGAIN_EVERY) {
            Ok((ready_now, _its_time_was_up)) => ready_now,
            Err(poisoned) => poisoned.into_inner().0,
        },
        false => match ready.wait(held) {
            Ok(ready_now) => ready_now,
            Err(poisoned) => poisoned.into_inner(),
        },
    };
}

/// What:
///  Hands one line's bytes to the thread that writes it,
///  and answers the number of the line when the queue had room.
/// Why:
///   A queue that is full counts the line instead of waiting for room,
/// so the caller never waits for the reader;
///  the number is what a caller that waits for its own line waits for,
///  and a queue whose callers do not wait never reads it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hand = (line) => { lock(); if (queue.length >= bound) { dropped++; return undefined } queue.push({ ticket: next++, line }); unlock(); notify(); return ticket }
/// ```
pub(crate) fn hand(queue: SharedQueue, line: Vec<u8>, bound: usize) -> Option<u64> {
    let (held_queue, ready) = queue;
    let mut held = held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    // What:     `held.lines.len() >= bound` is the moment the queue says a line was dropped.
    // Why:      A bound is the whole of the promise a caller is given: it costs no wait, a line beyond it costs that line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (held.lines.length >= bound) { held.dropped = held.dropped + 1; return undefined; }
    // ```
    if held.lines.len() >= bound {
        held.dropped = held.dropped.saturating_add(1);
        return None;
    }
    // What:     `ticket` is this line's number, taken before it is queued.
    // Why:      The caller waits for the line it handed over, and no other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ticket = held.nextTicket;
    // ```
    let ticket: u64 = held.next_ticket;
    held.next_ticket = held.next_ticket.saturating_add(1);
    held.lines.push_back(Pending { ticket, line });
    drop(held);
    ready.notify_one();
    // What:     `Some(ticket)` tells the caller the queue took the line, and which one.
    // Why:      A queue that dropped a line says nothing about it; a queue that took one owes the caller its number.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return ticket;
    // ```
    return Some(ticket);
}

/// What:
///  Waits,
///  up to the wait it is given,
///  until the stream holds every byte of line `ticket`,
///  and remembers a line the reader did not take so that no later caller waits.
/// Why:
///   One reader that was not taking a line by the bound is a reader that is not taking lines,
/// so the wait is paid once per process and never again,
///  which is what keeps a command that says twenty lines under a held lock to one bound rather than twenty.
///  A queue whose callers do not wait never calls this,
///  and nothing here waits for a stream that is already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitUntilWritten = (ticket, within) => { for (;;) { if (written >= ticket) return; if (deadlinePassed()) { readerStopped = true; return } waitUntil(deadline) } }
/// ```
pub(crate) fn wait_until_written(queue: SharedQueue, ticket: u64, within: std::time::Duration) {
    let (held_queue, ready) = queue;
    // What:     `Instant::now() + within` is the moment the caller stops waiting for the reader.
    // Why:      A reader that was not taking the line by then is not taking lines at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const deadline = Date.now() + within;
    // ```
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    // What:     `if !held.policy.callers_wait_for_their_line` leaves at once for a queue whose callers hand a line and go on.
    // Why:      The wait is a promise only the diagnostics' queue makes; the service's log never holds its callers.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!held.policy.callersWaitForTheirLine) return;
    // ```
    if !held.policy.callers_wait_for_their_line {
        return;
    }
    loop {
        // A stream that is gone takes nothing more, so no wait brings a reader back.
        if held.stream_is_gone || held.reader_stopped || held.written >= ticket {
            return;
        }
        // What:     `left` is how long the caller may still wait, or nothing once the bound has passed.
        // Why:      A caller that waited its bound remembers the stopped reader instead of waiting again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const left = deadline - now(); if (left <= 0) { readerStopped = true; log(...); return; }
        // ```
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) else {
            // Said once per process, to the log this command keeps rather than to the stream it cannot reach.
            held.reader_stopped = true;
            let tag: &'static str = held.policy.tag;
            drop(held);
            crate::diagnostic_log::debug(tag, format!("the reader of this queue's stream took no line within {within:?}, so the command stops waiting for it").as_str());
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
///  for every byte of every line handed over to reach the stream before the process ends,
///  and says whether it did.
/// Why:
///   A process that ends at once after its last line loses that line otherwise:
/// the line is in a thread of its own,
///  and `std::process::exit` ends every thread where it stands.
///  A process that never handed over a line starts no thread and waits for nothing,
/// which is every ordinary wrapper command;
///  and the wait belongs only at the end of the process,
/// never to a caller that says a line and goes on.
///  Its answer is the whole of what it promises:
/// `true` means every byte of every line handed over is in the stream,
/// which is why the wait asks the thread again at `THE_END_ASKS_AGAIN_EVERY`
/// instead of answering on its first attempt,
///  and why a stream that is gone is answered `false` at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeWhatWaits = (within) => (queueEmpty && !writing) || waitedAsLongAs(within)
/// ```
pub(crate) fn write_what_waits(queue: SharedQueue, within: std::time::Duration) -> bool {
    let (held_queue, ready) = queue;
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
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

/// How many lines are waiting for the stream,
/// which a control reads to show the queue is bounded.
#[cfg(test)]
pub(crate) fn queued(queue: SharedQueue) -> usize {
    let (held_queue, _ready) = queue;
    return held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).lines.len();
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads.
#[cfg(test)]
pub(crate) fn dropped_so_far(queue: SharedQueue) -> u64 {
    let (held_queue, _ready) = queue;
    return held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).dropped;
}

/// What:
///  Gives the thread a stream of the control's own.
/// Why:
///   A control holds the read end of a pipe and never drains it,
/// so the thread waits there and the queue's bound is what the control reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(testWriter)
/// ```
#[cfg(test)]
pub(crate) fn give_the_thread(queue: SharedQueue, writer: Box<dyn Write + Send>) {
    let (held_queue, _ready) = queue;
    let mut held = held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    held.writer = Some(writer);
    // What:     `held.reader_stopped = false` puts back the wait a caller gives a reader, for a control that gives a fresh
    //           stream after the reader that had stopped was replaced.
    // Why:      A queue whose callers wait would otherwise never wait again in a control that hands a new stream.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // held.readerStopped = false;
    // ```
    held.reader_stopped = false;
}

/// Whether a caller of this process stopped waiting for the reader,
/// which a control reads and no release path asks for.
#[cfg(test)]
pub(crate) fn the_reader_stopped(queue: SharedQueue) -> bool {
    let (held_queue, _ready) = queue;
    return held_queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner).reader_stopped;
}
