//! What:
//!  The line a process whose process limit refused the queue's thread writes on the thread that says it:
//!  as much of it as the stream takes at once,
//!  one page at a time,
//!  and never a wait for a reader.
//! Why:
//!   A process whose process limit refuses a queue's thread still has to say why it failed:
//! the refusal of a commit at its process limit reached nobody on 2026-11-25 (issue 6476),
//!  because every line waited in a queue that nothing would ever write.
//!  The queue is `the_queue_and_its_thread.rs`,
//!  which hands a line here once its thread is refused;
//!  the write a stream with a reader takes is the thread's,
//!  and this one is the caller's,
//!  so it is bounded by the room the stream reports now rather than by a reader's patience.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeWhereItIsSaid(queue, line); // at most one pipe page a write, only while poll reports room
//! ```

/// The queue a refused thread's lines are taken from,
/// and the streams they go to.
use crate::the_queue_and_its_thread::SharedQueue;
/// Writing one line into a stream,
/// which a control of the queue gives the thread.
use std::io::Write;

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
pub(crate) fn write_where_it_is_said(
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
///  a terminal and a pipe being drained all answer that they have room,
///  so the ordinary case is written and the stopped reader costs the line.
///  Both queues' policies write this process's standard error when no control gave a stream,
/// which is descriptor 2,
///  so the room that was asked of here is the room the line goes to.
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
///  it is written and waited for.
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
