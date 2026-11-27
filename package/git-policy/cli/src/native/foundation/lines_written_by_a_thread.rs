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
//! whether the thread ends when the stream does,
//! and what the process says where the limit on its processes refuses that thread.
//!  Those differences are the `QueuePolicy` a queue is made with;
//! everything else is written here once,
//!  which is what keeps one queue's fix from leaving the other behind
//! (debt ledger of 2026-11-25,
//! item 13).
//!  A refused thread is one of those differences only in what it says it was for:
//! both queues start their thread the same way,
//! so both meet the same refusal,
//!  and both answer it the same way here (issue 6476).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const queue = startTheQueue(policy); hand(queue, line); writeWhatWaits(queue, 250);
//! ```

/// The lines waiting for the stream.
use std::collections::VecDeque;
/// Writing one line into a stream,
/// and the error it may give.
use std::io::Write;
/// The queue,
/// the condition the thread waits on,
/// and the type both owners keep their one queue in.
use std::sync::{
    Condvar,
    Mutex,
};

/// What:
///  The queue of one process,
/// and the condition its thread waits on,
///  as the value each owner keeps in its own `OnceLock`.
/// Why:
///   The queue,
/// its thread and the callers that hand lines over all reach the same two values,
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
///  (`diagnostic_lines_off_the_calling_thread.rs`,
/// `service_log_lines_off_the_calling_thread.rs`)
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
    /// What the thread this queue starts is for,
    /// said in the sentence a refused thread leaves,
    /// or nothing when the refusal is left unsaid.
    /// Why the refusal is written rather than held:
    ///  a process limit that refused the thread once refuses every later start too,
    ///  so a line kept for a thread of its own would never reach the person (issue 6476).
    pub(crate) refused_thread_purpose: Option<&'static str>,
    /// The tag of this queue's own diagnostic lines.
    pub(crate) tag: &'static str,
}

/// What:
///  Whether the stream took the line a caller handed over.
/// Why:
///   The hand-over answers before the thread has written anything,
///  so a caller that must know whether its words reached the person
///  reads this answer instead of the write's own
///  (`Notice::show`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// enum HandedOver { Written, Unwritten }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandedOver {
    /// Every byte of the line is in the stream.
    Written,
    /// The line is not in the stream,
    /// and no wait of this command puts it there.
    Unwritten,
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
    /// The numbers of the lines a stream refused,
    /// oldest first,
    /// kept until the caller that waits for one reads its answer.
    /// Why the numbers rather than a yes or a no:
    ///  a write that failed for any reason other than a momentary lack of room
    ///  leaves a stream that no wait can bring back,
    ///  and a caller that must know whether its own line reached the person
    ///  asks about that line alone.
    /// Why only a queue whose callers wait keeps them:
    ///  nobody reads an answer in a queue whose callers hand a line and go on,
    ///  and a service that logged for days beside a stream that refused every line
    ///  would keep one number per line for the life of its process.
    lost: VecDeque<u64>,
    /// Whether the system refused the thread that writes these lines,
    /// so every line is written on the thread that says it.
    /// Why the refusal is kept and not retried:
    ///  a process limit that refused the thread once refuses every later start too,
    ///  so a line waiting for a new one would never reach the person (issue 6476).
    thread_refused: bool,
    /// The line that says which limit refused that thread,
    /// or nothing once it is said.
    /// Why it waits here:
    ///  the queue is made before a control's stream is given to it,
    ///  so the sentence is said with the first line the process says.
    thread_refusal: Option<String>,
    /// Where a control's lines go,
    /// or nothing when they go to the stream this queue defaults to.
    writer: Option<Box<dyn Write + Send>>,
    /// Whether `writer` is a control's own stream,
    /// which takes every byte at once,
    /// rather than this queue's default stream,
    /// which has a reader that may stop.
    /// Why it is kept beside the writer:
    ///  the fallback hands its own writer back here after a line,
    ///  and a writer that came back from the default stream is not a control's:
    ///  a line after it must be bounded by the room the stream has,
    ///  as the first one was.
    writer_from_a_control: bool,
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
pub(crate) const THE_END_ASKS_AGAIN_EVERY: std::time::Duration =
    std::time::Duration::from_millis(5);

/// What:
///  The queue of one process,
///  made once and given its thread.
/// Why:
///   The thread waits on the same queue for as long as the process runs,
/// so the value has to outlive the call that made it;
///  a process that says no line starts no thread and pays nothing.
///  A thread the system refuses leaves every line of this process unwritten if nothing answers it,
/// which is how the refusal of a commit at its process limit was lost on 2026-11-25 (issue 6476),
///  so the refusal is kept here and every line is written where it is said from then on.
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
            lost: VecDeque::new(),
            thread_refused: false,
            thread_refusal: None,
            writer: None,
            writer_from_a_control: false,
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
    // What:     `if let Err(refusal) = spawned` keeps the refusal of the one thread that writes this queue,
    //           and builds the sentence that says which limit refused it,
    //           where the queue's own policy says the refusal at all.
    // Why:      A caller that hands a line would otherwise believe it was shown,
    //           and a refusal is the whole of what a command at its process limit has to say (issue 6476).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (spawned.kind === 'error' && policy.refusedThreadPurpose) { queue.threadRefused = true; queue.threadRefusal = `monochromatic-git-policy-cli: ${refusedThreadSentence(policy.refusedThreadPurpose, spawned.error)}. Every line is written on the thread that says it from now on, so nothing this process says is lost while this limit stands.\n`; }
    // ```
    if let Err(refusal) = spawned
        && let Some(purpose) = policy.refused_thread_purpose
    {
        // What:     `said` is the one sentence this process says about the refused thread,
        //           built from the purpose the policy names and the system's own error.
        // Why:      The person reads which limit refused the thread and what to change,
        //           not the queue's own vocabulary.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const said = `monochromatic-git-policy-cli: ${refusedThreadSentence(purpose, refusal)}. Every line is written on the thread that says it from now on, so nothing this process says is lost while this limit stands.\n`;
        // ```
        let said: String = format!(
            "monochromatic-git-policy-cli: {}. Every line is written on the thread that says it from now on, so nothing this \
             process says is lost while this limit stands.\n",
            crate::scoped_start::refused_thread_sentence(purpose, &refusal)
        );
        let mut held = started
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        held.thread_refused = true;
        held.thread_refusal = Some(said);
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
    let policy: &'static QueuePolicy = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .policy;
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
        let written: std::io::Result<()> =
            write_whole(&mut writer, line.line.as_slice(), queue, ready);
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
///  the pop and the count are one step,
/// so a gap is stated by the line right after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeALine = () => { lock(); for (;;) { const line = lines.shift(); if (line) { writing = true; return { line, dropped: take(dropped) } } wait(); } }
/// ```
fn take_a_line(
    queue: &'static Mutex<Queue>,
    ready: &'static Condvar,
) -> (Pending, u64) {
    let mut held = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
    let mut held = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
///  keeps a line the stream refused,
///  and wakes every caller waiting for a line.
/// Why:
///   A caller waits for the number of its own line,
/// so exactly one movement of that number is what ends the wait,
///  and the wake comes after the stream took the line rather than before.
///  A line the stream refused is kept by its number,
/// because the number alone cannot say whether this line or an earlier one failed,
///  and only a queue whose callers wait for an answer keeps them,
///  so the list is bounded by the callers a queue can hold at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBack = (writer, ticket, gone) => { lock(); queue.writer ??= writer; queue.written = ticket; queue.writing = false; if (gone) { queue.streamIsGone = true; queue.lost.push(ticket) } unlock(); notifyAll() }
/// ```
fn give_back(
    queue: &'static Mutex<Queue>,
    ready: &'static Condvar,
    writer: Box<dyn Write + Send>,
    ticket: u64,
    stream_is_gone: bool,
) {
    let mut held = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.writer.is_none() {
        held.writer = Some(writer);
    }
    held.written = ticket;
    held.writing = false;
    if stream_is_gone {
        held.stream_is_gone = true;
        if held.policy.callers_wait_for_their_line {
            held.lost.push_back(ticket);
        }
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
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                wait_to_be_asked_again(queue, ready)
            }
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
/// one that does not waits to be asked,
/// by the next line or the end of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await policy.retriesOnATick ? sleep(THE_END_ASKS_AGAIN_EVERY) : nextAsk()
/// ```
fn wait_to_be_asked_again(
    queue: &'static Mutex<Queue>,
    ready: &'static Condvar,
) {
    let held = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
///  Writes one line on the thread that says it,
///  where the system refused the thread that would have written it,
///  and answers the number of the line where the call settled it.
/// Why:
///   A process whose process limit refuses a queue's thread still has to say why it failed:
/// the refusal of a commit at its process limit reached nobody on 2026-11-25 (issue 6476),
///  because every line waited in a queue that nothing would ever write.
///  The line is written without waiting for a reader:
/// a write is made only where the stream has room now,
/// and of no more than one pipe page,
///  because the room `POLLOUT` reports is one free slot and not the whole of a long line,
/// so a stream nobody drains costs the line and never the caller,
///  which is the same promise the queue keeps when its room runs out.
///  A control's own stream is written as it stands,
/// because a control's stream is the control's memory and never a reader.
///  A line the stream does not take whole is settled here all the same,
/// so the number moves at once,
/// the caller that waits for its line waits for nothing,
/// and the part that did not fit is counted as a dropped line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeWhereItIsSaid = (line) => { const [writer, fromAControl] = takeTheStream(); const took = writeWhatFitsNow(writer, line, fromAControl); if (!fromAControl) dropIf(took < line.length); writer.flush(); giveBack(writer); const ticket = next++; written = ticket; return ticket }
/// ```
fn write_where_it_is_said(
    queue: SharedQueue,
    line: &[u8],
) -> Option<u64> {
    let (held_queue, _ready) = queue;
    // What:     `from_a_control` says the stream came from a control rather than from this queue's policy,
    //           and a control's stream is memory that takes every byte at once.
    // Why:      The room a write waits for is a reader's, and only a stream of this process has a reader.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [writer, fromAControl] = held.writer ? [held.writer, true] : [(held.policy.defaultStream)(), false];
    // ```
    let (mut writer, from_a_control): (Box<dyn Write + Send>, bool) = {
        let mut held = held_queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match held.writer.take() {
            // What: Whether the writer that was here is a control's own, read beside it.
            // Why:  The fallback put its own stream back after the last line, and that stream has a reader.
            Some(known) => {
                let it_is_a_control = held.writer_from_a_control;
                (known, it_is_a_control)
            }
            None => ((held.policy.default_stream)(), false),
        }
    };
    // What:     `took` is how much of the line the stream took without waiting for a reader,
    //           which is every byte of a control's line and at most what its room holds of this process's line.
    // Why:      A write to a stream with no room now waits for the reader, which is the wait a caller must never pay.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const took = writeWhatFitsNow(writer, line, fromAControl);
    // ```
    let took: usize = write_what_fits_now(&mut writer, line, from_a_control);
    let _flushed: std::io::Result<()> = writer.flush();
    let mut held = held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if held.writer.is_none() {
        held.writer = Some(writer);
        held.writer_from_a_control = from_a_control;
    }
    // What:     A line the stream did not take whole is counted as dropped, so the next line that gets through says the gap.
    // Why:      Part of a line reached the person and the rest could not be said without waiting;
    //           the count is what the queue says about every line it could not say.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (took < line.length) held.dropped = held.dropped + 1;
    // ```
    if took < line.len() {
        held.dropped = held.dropped.saturating_add(1);
    }
    // What:     `ticket` is the number this line takes, and `held.written = ticket` says the stream has reached it.
    // Why:      A caller that waits for its own line asks for this number, and what this call could write is the whole of
    //           the answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ticket = held.nextTicket++; held.written = ticket; return ticket;
    // ```
    let ticket: u64 = held.next_ticket;
    held.next_ticket = held.next_ticket.saturating_add(1);
    held.written = ticket;
    return Some(ticket);
}

/// What:
///  The most a pipe takes in one write once a poll reports room:
///  one page,
///  which is `PIPE_BUF` on Linux and is what a pipe holding one free slot takes whole.
/// Why:
///   `POLLOUT` promises a free slot and not the whole of a longer line,
/// and a blocking write of more than that free room waits for the reader:
/// measured on this host,
/// a pipe with one page free held a blocking write of two pages for as long as the control waited (3 s),
///  while the same pipe took one page at once.
///  So every write on the thread that says a line is bounded by this figure,
///  and a line that does not fit costs its tail rather than the caller.
///  The service's journal writes are bounded by the same figure for the same reason,
///  the `PIPE_CHUNK` of `service/journald_native.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM = 4096
/// ```
pub(crate) const THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM: usize = 4096;

/// What:
///  Writes as much of `bytes` as `writer` takes without waiting for a reader,
///  and answers how many bytes that was.
/// Why:
///   The thread that says a line has no one to finish it,
/// so it makes only the writes a stream takes at once and leaves the rest;
///  a control's stream is memory,
///  which takes every byte at once.
///  Each write is bounded by one pipe page,
///  because the room a poll reports is one free slot,
/// and a line longer than that would wait for the reader inside the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeWhatFitsNow = (writer, bytes, fromAControl) => { if (fromAControl) return writer.writeAll(bytes); let done = 0; while (done < bytes.length && roomNow()) { const took = writer.write(bytes.slice(done, done + 4096)); if (took === 0) break; done += took } return done }
/// ```
fn write_what_fits_now(
    writer: &mut Box<dyn Write + Send>,
    bytes: &[u8],
    from_a_control: bool,
) -> usize {
    // What:     A control's stream is written whole: it is the control's memory and never a reader that can stop.
    // Why:      The bound exists for a stream with a reader, and a control's stream has none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fromAControl) return bytes.length;
    // ```
    if from_a_control {
        return match writer.write_all(bytes) {
            Ok(()) => bytes.len(),
            Err(_) => 0,
        };
    }
    let mut done: usize = 0;
    // What:     Each turn asks whether the stream has room now, and writes no more than one page of what is left.
    // Why:      A turn without room ends the line here, which is what keeps the caller from waiting for a reader;
    //           every turn that writes moves `done`, so the loop ends with the line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (done < bytes.length && roomNow()) { ... }
    // ```
    while done < bytes.len() && the_stream_has_room_now() {
        let left: &[u8] = &bytes[done..];
        let piece: &[u8] = &left[..left.len().min(THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM)];
        match writer.write(piece) {
            // What:     A stream that took nothing at once has no room after all, so the line's rest is left unsaid.
            // Why:      Asking again would be a loop of this process's own, and the promise here is one settling call.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (took === 0) break;
            // ```
            Ok(0) => break,
            Ok(taken) => done += taken,
            // What:     A signal ended the call and nothing about the stream changed, so the rest is tried again.
            // Why:      A signal must not cost the person the rest of a line the stream would still take.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // else if (error.kind === 'Interrupted') continue;
            // ```
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            // What:     Any other refusal is the stream's answer, and the rest of the line is left unsaid.
            // Why:      A stream that refused a write either has no room now or is gone, and neither is a reason to wait.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // else break;
            // ```
            Err(_) => break,
        }
    }
    return done;
}

/// What:
///  Whether this process's standard error takes a write now,
///  asked without waiting for the reader.
/// Why:
///   A line written where it is said must still keep the promise that no caller waits for a reader,
/// and a stream with no room now is one a write would wait on;
///  a regular file,
/// a terminal and a pipe being drained all answer that they have room,
///  so the ordinary case is written and the stopped reader costs the line.
///  Both queues' policies write this process's standard error when no control gave a stream,
/// which is descriptor 2,
/// so the room that was asked of here is the room the line goes to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const roomNow = () => poll([{ fd: 2, events: POLLOUT }], 0) > 0
/// ```
#[cfg(unix)]
fn the_stream_has_room_now() -> bool {
    // What: one descriptor watched for room to write in it, and for no time at all.
    let mut watched: libc::pollfd = libc::pollfd {
        fd: libc::STDERR_FILENO,
        events: libc::POLLOUT,
        revents: 0,
    };
    // SAFETY: `poll` reads and writes only the one structure given to it here, and waits for no time.
    let answered: libc::c_int = unsafe { libc::poll(&mut watched, 1, 0) };
    // What: whether the stream reported room, and not an error or a hang-up instead.
    return answered > 0 && watched.revents & libc::POLLOUT != 0;
}

/// What:
///  Whether this process's standard error takes a write now,
///  on a platform whose descriptors carry no such question:
/// it is written and waited for.
/// Why:
///   Every platform this program is released on has the question above;
/// this answer keeps the build of the other platforms and is measured by no control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const roomNow = () => true
/// ```
#[cfg(not(unix))]
fn the_stream_has_room_now() -> bool {
    return true;
}

/// What:
///  Hands one line's bytes to the thread that writes it,
///  answers the number of the line when the queue had room,
///  and writes the line where it is said when the system refused that thread.
/// Why:
///   A queue that is full counts the line instead of waiting for room,
/// so the caller never waits for the reader;
///  the number is what a caller that waits for its own line waits for,
///  and a queue whose callers do not wait never reads it.
///  A refused thread leaves no one to take the line,
/// so the line is written on the thread that says it instead of waiting for ever (issue 6476).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hand = (line) => { lock(); if (refused) { const said = refusal.shift(); unlock(); if (said) writeWhereItIsSaid(said); return writeWhereItIsSaid(line) } if (queue.length >= bound) { dropped++; return undefined } queue.push({ ticket: next++, line }); unlock(); notify(); return ticket }
/// ```
pub(crate) fn hand(
    queue: SharedQueue,
    line: Vec<u8>,
    bound: usize,
) -> Option<u64> {
    let (held_queue, ready) = queue;
    let mut held = held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // What:     `if held.thread_refused` is the queue of a process whose limit refused the thread that writes it,
    //           where `held.thread_refusal.take()` takes the one sentence about that refusal.
    // Why:      Nothing will ever take a line from this queue, so the line is written where it is said,
    //           and the sentence is said with the first line the process says, once (issue 6476).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (held.threadRefused) { const said = held.threadRefusal; held.threadRefusal = undefined; drop(held); if (said) writeWhereItIsSaid(queue, said); return writeWhereItIsSaid(queue, line); }
    // ```
    if held.thread_refused {
        let sentence: Option<String> = held.thread_refusal.take();
        drop(held);
        if let Some(said) = sentence {
            let _said_where_it_stands: Option<u64> = write_where_it_is_said(queue, said.as_bytes());
        }
        return write_where_it_is_said(queue, line.as_slice());
    }
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
    held.lines.push_back(Pending {
        ticket,
        line,
    });
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
///  answers whether it does,
///  and remembers a line the reader did not take so that no later caller waits.
/// Why:
///   One reader that was not taking a line by the bound is a reader that is not taking lines,
/// so the wait is paid once per process and never again,
///  which is what keeps a command that says twenty lines under a held lock to one bound rather than twenty.
///  A queue whose callers do not wait reads the answer its line already has and waits for nothing,
///  and nothing here waits for a stream that is already gone.
///  The answer is the whole point for a caller that must know whether its words reached the person:
/// a line the stream refused is answered `Unwritten`,
///  and a line still waiting when the bound passes is answered `Unwritten` too,
///  because a command never waits for a reader past its bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitUntilWritten = (ticket, within) => { for (;;) { if (lost.includes(ticket)) return 'Unwritten'; if (written >= ticket) return 'Written'; if (streamIsGone || readerStopped) return 'Unwritten'; if (deadlinePassed()) { readerStopped = true; return 'Unwritten' } waitUntil(deadline) } }
/// ```
pub(crate) fn wait_until_written(
    queue: SharedQueue,
    ticket: u64,
    within: std::time::Duration,
) -> HandedOver {
    let (held_queue, ready) = queue;
    // What:     `Instant::now() + within` is the moment the caller stops waiting for the reader.
    // Why:      A reader that was not taking the line by then is not taking lines at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const deadline = Date.now() + within;
    // ```
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // What:     `if !held.policy.callers_wait_for_their_line` leaves at once for a queue whose callers hand a line and go on.
    // Why:      The wait is a promise only the diagnostics' queue makes; the service's log never holds its callers,
    //           and it still reads the answer its line has now.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!held.policy.callersWaitForTheirLine) return takenNow(held, ticket);
    // ```
    if !held.policy.callers_wait_for_their_line {
        return the_answer_the_queue_holds(&mut held, ticket).unwrap_or(HandedOver::Unwritten);
    }
    loop {
        // What:     `the_answer_the_queue_holds(...)` is the answer this queue holds now, where one it holds is decided.
        // Why:      A line the stream refused and a line it wrote are both decided;
        //           a line still waiting is not, and it waits on.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const answered = theAnswerTheQueueHolds(held, ticket); if (answered !== undefined) return answered;
        // ```
        if let Some(answered) = the_answer_the_queue_holds(&mut held, ticket) {
            return answered;
        }
        // A stream that is gone takes nothing more, so no wait brings a reader back.
        if held.stream_is_gone || held.reader_stopped {
            return HandedOver::Unwritten;
        }
        // What:     `left` is how long the caller may still wait, or nothing once the bound has passed.
        // Why:      A caller that waited its bound remembers the stopped reader instead of waiting again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const left = deadline - now(); if (left <= 0) { readerStopped = true; log(...); return 'Unwritten'; }
        // ```
        let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) else {
            // Said once per process, to the log this command keeps rather than to the stream it cannot reach.
            held.reader_stopped = true;
            let tag: &'static str = held.policy.tag;
            drop(held);
            crate::diagnostic_log::debug(tag, format!("the reader of this queue's stream took no line within {within:?}, so the command stops waiting for it").as_str());
            return HandedOver::Unwritten;
        };
        held = match ready.wait_timeout(held, left) {
            Ok((ready_now, _its_time_was_up)) => ready_now,
            Err(poisoned) => poisoned.into_inner().0,
        };
    }
}

/// What:
///  The answer the queue holds for line `ticket` now,
///  where a refusal of that line is read and taken out,
///  and nothing while the stream has not answered for it yet.
/// Why:
///   A line the stream refused is kept by its number,
///  and the caller that asks for it is the one that reads the refusal;
///  a line the stream wrote is answered by the number the queue moved to,
///  which is exact because each line moves it to its own number and one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theAnswerTheQueueHolds = (held, ticket) => { if (held.lost.includes(ticket)) { held.lost = held.lost.filter((each) => each !== ticket); return 'Unwritten' } if (held.written >= ticket) return 'Written'; return undefined }
/// ```
fn the_answer_the_queue_holds(
    held: &mut Queue,
    ticket: u64,
) -> Option<HandedOver> {
    if let Some(at) = held.lost.iter().position(|lost| return *lost == ticket) {
        let _read: Option<u64> = held.lost.remove(at);
        return Some(HandedOver::Unwritten);
    }
    if held.written >= ticket {
        return Some(HandedOver::Written);
    }
    return None;
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
pub(crate) fn write_what_waits(
    queue: SharedQueue,
    within: std::time::Duration,
) -> bool {
    let (held_queue, ready) = queue;
    let deadline: std::time::Instant = std::time::Instant::now() + within;
    let mut held = held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
    return held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .lines
        .len();
}

/// How many lines have been dropped since the last one that got through,
/// which a control reads.
#[cfg(test)]
pub(crate) fn dropped_so_far(queue: SharedQueue) -> u64 {
    let (held_queue, _ready) = queue;
    return held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .dropped;
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
pub(crate) fn give_the_thread(
    queue: SharedQueue,
    writer: Box<dyn Write + Send>,
) {
    let (held_queue, _ready) = queue;
    let mut held = held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    held.writer = Some(writer);
    // What:     `held.writer_from_a_control = true` says the stream here is the control's memory, which takes every byte.
    // Why:      The fallback bounds a line by the room of a stream with a reader, and a control's stream has none.
    held.writer_from_a_control = true;
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
    return held_queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .reader_stopped;
}
