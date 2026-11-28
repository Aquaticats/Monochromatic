//! What:
//!  A caller's side of a queue of lines a thread writes:
//!  the answer a handed line gets,
//!  the hand-over that gives the line its number,
//!  the wait for that line to reach the stream,
//!  and the wait the end of a process makes for every line it handed over.
//! Why:
//!   A caller must never wait for a reader that has stopped draining,
//!  so a line is handed to a queue of bounded length and written on a thread that may wait as long as it likes;
//!  a line that finds no room is dropped and counted,
//!  and the next line that gets through says how many were dropped before it,
//!  so a gap in the lines is a gap the lines state.
//!  The caller waits for its own line only while the reader is still taking lines,
//!  and only up to the bound its queue gives it,
//!  which keeps the order of the lines among the ones written by hand beside them;
//!  a reader that was not taking a line by then is not taking any,
//!  so from the first such bound on the caller never waits again,
//!  and the lines wait in the queue for a reader that comes back or for the end of the process.
//!  A process about to end waits,
//!  once and bounded,
//!  for the lines it handed over,
//!  because `std::process::exit` ends the thread where it stands
//!  and the last lines of a refusal are the whole of what a person gets.
//!  The queue,
//!  its thread and the state a control reads are `the_queue_and_its_thread.rs`;
//!  the line a process whose thread was refused writes where it is said is
//!  `the_line_a_refused_thread_writes_where_it_is_said.rs`.
//!  This file names that file's and the queue's items again under this file's own name,
//!  so the code that reached them here before a split reaches them here still.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ticket = hand(queue, line, bound); return ticket === undefined ? 'Unwritten' : waitUntilWritten(queue, ticket, within);
//! ```

/// The most one write of a line a refused thread says takes,
/// which a control of that write reads.
#[cfg(test)]
pub(crate) use crate::the_line_a_refused_thread_writes_where_it_is_said::THE_MOST_A_PIPE_TAKES_WHEN_IT_SHOWS_ROOM;
/// The line a refused thread writes where it is said,
/// which the hand-over below writes when that thread is gone.
use crate::the_line_a_refused_thread_writes_where_it_is_said::write_where_it_is_said;
/// The queue's own items,
/// named again here because this file's callers reached them here before the split:
/// the policy and the value both ends of a queue share,
/// the entry a line waits in,
/// and the queue itself.
pub(crate) use crate::the_queue_and_its_thread::{
    Pending,
    Queue,
    QueuePolicy,
    SharedQueue,
    THE_END_ASKS_AGAIN_EVERY,
    start_the_queue,
    the_reader_stopped,
};
/// The queue's control hooks,
/// which only a control reads,
/// named again here for the controls that read them at this path.
#[cfg(test)]
pub(crate) use crate::the_queue_and_its_thread::{
    dropped_so_far,
    give_the_thread,
    queued,
};

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
            // What:     `held.reader_stopped = true` is this process remembering that a reader is not taking lines,
            //           so no later caller waits for one.
            // Why:      The wait is paid once per process and never again, which is what keeps a command that says twenty
            //           lines under a held lock to one bound rather than twenty. The line the person is owed about it is
            //           said by the caller that reads this state after the wait, through the one write a diagnostic line
            //           takes (`diagnostic_lines_off_the_calling_thread.rs`).
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const left = deadline - now(); if (left <= 0) { readerStopped = true; return 'Unwritten'; }
            // ```
            held.reader_stopped = true;
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
