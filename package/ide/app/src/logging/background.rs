//! A log writer that never makes the logging thread wait for the log output.
//!
//! Every formatted record is handed to one writer thread over a queue,
//!  and only that thread
//! writes to the output.
//!  A reader that stops reading the output,
//!  or a disk that stalls an append
//! for seconds,
//!  then blocks the writer thread and nothing else:
//!  the interface thread keeps
//! drawing and handling input.
//!
//! The queue holds at most a fixed number of bytes.
//!  A record that does not fit is dropped and
//! counted,
//!  never waited for.
//!  The next record that does fit carries the count,
//!  and the writer
//! thread writes one warning line with it right before that record,
//!  so the gap is visible where
//! it happened.
//!  At a clean exit,
//!  [`LogFlush`] waits a bounded time for the queue to be written.

/// What:
///  `Write` is the trait of byte outputs (files,
///  pipes,
///  standard error);
///  `io::Result` is a
///       success value or an operating-system error.
/// Why:
///  The writer thread writes to any output,
///  and the logging side presents itself as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Write { write(bytes: Uint8Array): number; flush(): void; }
/// ```
use std::io::{self, Write};
/// What:
///  `Arc` is a thread-safe shared pointer;
///  `Mutex` guards a value that several threads
///       change;
///  `mpsc` is a multi-producer,
///  single-consumer queue between threads.
/// Why:
///  The loss count is shared by every logging thread and the writer thread,
///  and records
///      cross from any thread to the one writer thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const queue: Message[] = []; // shared between workers
/// ```
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
/// What:
///  `thread` starts operating-system threads;
///  `Duration` is a time span.
/// Why:
///  The writer is its own thread,
///  and the exit flush waits a bounded time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writer = new Worker('log-writer');
/// ```
use std::{thread, time::Duration};
/// What:
///  `FormatTime` writes a timestamp;
///  `SystemTime` is the clock the log's other lines use;
///       `Writer` is the text sink a timestamp is written into;
///  `MakeWriter` is how the log
///       subscriber obtains a writer for each record.
/// Why:
///  The loss warning carries the same timestamp format as every other line,
///  and the queue
///      plugs into the subscriber as its output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MakeWriter { makeWriter(): Write; }
/// ```
use tracing_subscriber::fmt::{
    MakeWriter,
    format::Writer,
    time::{FormatTime, SystemTime},
};

/// Bytes of formatted records that may wait for the output at once.
///  A burst of debug records fits;
/// a stalled output fills it within seconds at the most detailed level,
///  and from then on new
/// records are counted as lost instead of growing memory without bound.
pub const QUEUE_BUDGET: usize = 8 * 1024 * 1024;

/// Longest wait at a clean exit for the queued records to reach the output.
///  An output that
/// accepts nothing for this long keeps whatever is still queued;
///  the exit is not held up longer.
pub const FLUSH_GRACE: Duration = Duration::from_secs(1);

/// What:
///  Records that were not written:
///  how many,
///  and how many bytes.
///  `u64` is an unsigned
///       64-bit integer (siblings:
///  `u32`,
///  `usize`);
///  a count over a long session needs the range.
/// Why:
///  The warning that marks a gap says how much is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Loss = { records: number; bytes: number };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Loss {
    /// Records that were dropped.
    pub records: u64,
    /// Their size in bytes.
    pub bytes: u64,
}

/// What:
///  What travels to the writer thread.
///  An `enum` with data is a tagged union.
/// Why:
///  Records and the exit flush share one queue,
///  so a flush comes after every record queued
///      before it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Message = { kind: 'record'; bytes: Uint8Array; lostBefore: Loss } | { kind: 'flush'; done: () => void };
/// ```
enum Message {
    /// One formatted record,
    ///  with the loss that happened right before it.
    Record(
        /// The record's bytes.
        Vec<u8>,
        /// Records dropped since the previous record that was queued.
        Loss,
    ),
    /// Write everything queued so far,
    ///  then report through this sender.
    Flush(
        /// Answered once the queue up to this point was written.
        mpsc::Sender<()>,
    ),
}

/// State that every logging thread and the writer thread share.
struct Shared {
    /// Bytes queued and not yet written.
    queued: AtomicUsize,
    /// Records dropped since the last report.
    lost: Mutex<Loss>,
    /// Largest number of bytes that may be queued.
    budget: usize,
}

/// Counting and taking losses.
impl Shared {
    /// What:
    ///  Add to the loss count.
    ///  `&self` borrows the shared state;
    ///  the lock is held only for
    ///       the addition.
    /// Why:
    ///  Several threads may drop records at the same moment.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lose(loss: Loss) { this.lost.records += loss.records; this.lost.bytes += loss.bytes; }
    /// ```
    fn lose(&self, loss: Loss) {
        // `lock()` returns `Err` only when a thread panicked while holding it; the count is still usable.
        let mut lost = match self.lost.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        lost.records = lost.records.saturating_add(loss.records);
        lost.bytes = lost.bytes.saturating_add(loss.bytes);
    }

    /// What:
    ///  Take the loss count and reset it.
    ///  `std::mem::take` returns the value and leaves the
    ///       type's default (zero) in its place.
    /// Why:
    ///  Each loss is reported once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// takeLoss(): Loss { const loss = this.lost; this.lost = { records: 0, bytes: 0 }; return loss; }
    /// ```
    fn take_loss(&self) -> Loss {
        let mut lost = match self.lost.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        return std::mem::take(&mut *lost);
    }
}

/// What:
///  The log subscriber's output:
///  hands every record to the writer thread.
///  `Clone` copies
///       the queue's sending end and the shared pointer.
/// Why:
///  Logging from any thread costs one queue push and never waits for the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BackgroundWriter implements MakeWriter { makeWriter() { return { write: bytes => this.enqueue(bytes) }; } }
/// ```
#[derive(Clone)]
pub struct BackgroundWriter {
    /// Sending end of the queue to the writer thread.
    sender: mpsc::Sender<Message>,
    /// Queue size and loss count.
    shared: Arc<Shared>,
}

/// Queuing one record.
impl BackgroundWriter {
    /// What:
    ///  Queue one record,
    ///  or count it as lost when the queue is full or the writer thread is
    ///       gone.
    ///  `bytes.to_vec()` copies the borrowed bytes into an owned list.
    /// Why:
    ///  The caller is any logging thread,
    ///  the interface thread included;
    ///  it must never wait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// enqueue(bytes: Uint8Array) { if (queued + bytes.length > budget) { lose(bytes); return; } queue.push({ bytes, lostBefore: takeLoss() }); }
    /// ```
    fn enqueue(&self, bytes: &[u8]) {
        let size = bytes.len();
        // `fetch_add` adds and returns the earlier value in one step, so two threads never both
        // see room for the same bytes.
        let before = self.shared.queued.fetch_add(size, Ordering::SeqCst);
        if before.saturating_add(size) > self.shared.budget {
            self.shared.queued.fetch_sub(size, Ordering::SeqCst);
            self.shared.lose(Loss {
                records: 1,
                bytes: size as u64,
            });
            return;
        }
        let lost_before = self.shared.take_loss();
        // What: `send` on this queue never waits; it fails only when the writer thread has ended,
        //       and then hands the message back inside the error.
        // Why: A record nobody will write is counted, with the loss it carried.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!queue.push(message)) lose(message);
        // ```
        if let Err(mpsc::SendError(_message)) = self
            .sender
            .send(Message::Record(bytes.to_vec(), lost_before))
        {
            self.shared.queued.fetch_sub(size, Ordering::SeqCst);
            self.shared.lose(Loss {
                records: lost_before.records.saturating_add(1),
                bytes: lost_before.bytes.saturating_add(size as u64),
            });
        }
    }
}

/// What:
///  The writer handed out for one record.
///  `'owner` is a lifetime:
///  the record writer only
///       borrows the background writer and cannot outlive it.
/// Why:
///  The log subscriber asks for a writer per record and writes the whole record through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordWriter = { write(bytes: Uint8Array): number };
/// ```
pub struct RecordWriter<'owner> {
    /// The queue the record goes to.
    owner: &'owner BackgroundWriter,
}

/// Every write is queued whole and reported as fully written.
impl Write for RecordWriter<'_> {
    /// Queue the bytes;
    ///  claiming them all keeps the subscriber from splitting the record.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.owner.enqueue(bytes);
        return Ok(bytes.len());
    }

    /// Nothing is held here;
    ///  the writer thread writes in order.
    fn flush(&mut self) -> io::Result<()> {
        return Ok(());
    }
}

/// The log subscriber obtains one record writer per record.
impl<'owner> MakeWriter<'owner> for BackgroundWriter {
    /// The per-record writer,
    ///  borrowing this background writer.
    type Writer = RecordWriter<'owner>;

    /// Lend this background writer to one record.
    fn make_writer(&'owner self) -> Self::Writer {
        return RecordWriter { owner: self };
    }
}

/// What:
///  Kept alive until a clean exit;
///  dropping it waits up to its grace for the queue to be
///       written.
///  `impl Drop` below runs that wait.
/// Why:
///  Records written just before exit,
///  such as the language shutdown,
///  must reach the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using flush = logFlush; // [Symbol.dispose]() waits up to the grace
/// ```
pub struct LogFlush {
    /// Sending end of the same queue,
    ///  for the flush request.
    sender: mpsc::Sender<Message>,
    /// Longest wait.
    grace: Duration,
}

/// Waiting for the queue at exit.
impl Drop for LogFlush {
    /// Queue a flush request behind every record and wait for its answer,
    ///  at most the grace.
    fn drop(&mut self) {
        let (done, answered) = mpsc::channel();
        if self.sender.send(Message::Flush(done)).is_err() {
            // The writer thread has ended, so nothing queued can be written any more.
            return;
        }
        // What: `recv_timeout` waits for the answer or gives up after the grace; `Err` names which.
        // Why: An output that accepts nothing must not hold the exit; there is also no output left
        //      to report that to, because it is that output which is not accepting.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await Promise.race([answered, sleep(grace)]);
        // ```
        let _unwritten_records_have_no_reader = answered.recv_timeout(self.grace);
    }
}

/// What:
///  Write the warning line that marks a gap,
///  when there is one.
///  `&mut O` lends the output
///       for writing;
///  `O: Write` accepts any output.
/// Why:
///  A dropped record must leave a trace where it would have been.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function report(output: Write, loss: Loss): boolean { if (loss.records === 0) return true; return tryWrite(output, line(loss)); }
/// ```
fn report<O: Write>(output: &mut O, loss: Loss) -> bool {
    if loss.records == 0 {
        return true;
    }
    let mut line = String::new();
    // `SystemTime` formats the clock the way the subscriber does; a failure only leaves the time out.
    if SystemTime.format_time(&mut Writer::new(&mut line)).is_err() {
        line.clear();
    }
    line.push_str(&format!(
        "  WARN ide_app::logging: {} log records ({} bytes) were not written because the log output did not accept them in time; the log has a gap here\n",
        loss.records, loss.bytes
    ));
    return output.write_all(line.as_bytes()).is_ok();
}

/// What:
///  The writer thread's loop:
///  write each record in order,
///  mark gaps,
///  answer flushes.
///  The loop
///       ends when every sending end is gone.
/// Why:
///  Only this thread ever waits for the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const message of queue) { if (message.kind === 'record') { report(output, message.lostBefore); output.write(message.bytes); } else { output.flush(); message.done(); } }
/// ```
fn drain<O: Write>(receiver: &mpsc::Receiver<Message>, output: &mut O, shared: &Shared) {
    // `iter()` yields messages until the queue is closed.
    for message in receiver.iter() {
        match message {
            Message::Record(bytes, lost_before) => {
                if !report(output, lost_before) {
                    shared.lose(lost_before);
                }
                // A record the output refused is counted, so a later report includes it.
                if output.write_all(&bytes).is_err() {
                    shared.lose(Loss {
                        records: 1,
                        bytes: bytes.len() as u64,
                    });
                }
                shared.queued.fetch_sub(bytes.len(), Ordering::SeqCst);
            }
            Message::Flush(done) => {
                let lost = shared.take_loss();
                if !report(output, lost) {
                    shared.lose(lost);
                }
                // An output that cannot flush has nothing more to give; the flush is still answered.
                let _flush_failure_has_no_reader = output.flush();
                // The waiting side may have given up already; then nobody needs the answer.
                let _answer_may_be_unwanted = done.send(());
            }
        }
    }
}

/// What:
///  Start the writer thread for `output` with the default budget and grace.
///       `O: Write + Send + 'static` means the output can move to another thread and lives on its own.
/// Why:
///  The application hands standard output to this;
///  tests hand an output they control.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function background(output: Write): [BackgroundWriter, LogFlush]
/// ```
pub fn background<O: Write + Send + 'static>(
    output: O,
) -> io::Result<(BackgroundWriter, LogFlush)> {
    return background_with(output, QUEUE_BUDGET, FLUSH_GRACE);
}

/// What:
///  Start the writer thread for `output` with an explicit budget and exit grace.
///       The tuple `(BackgroundWriter, LogFlush)` returns two values at once.
/// Why:
///  Tests use a small budget to reach the full queue quickly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function backgroundWith(output: Write, budget: number, grace: number): [BackgroundWriter, LogFlush]
/// ```
pub fn background_with<O: Write + Send + 'static>(
    mut output: O,
    budget: usize,
    grace: Duration,
) -> io::Result<(BackgroundWriter, LogFlush)> {
    let (sender, receiver) = mpsc::channel();
    // `Arc::new` puts the shared state behind a thread-safe shared pointer.
    let shared = Arc::new(Shared {
        queued: AtomicUsize::new(0),
        lost: Mutex::new(Loss::default()),
        budget,
    });
    // `clone` copies the shared pointer for the writer thread.
    let for_thread = Arc::clone(&shared);
    // What: `spawn` starts the named thread; `move` hands it the receiver, the output, and its pointer.
    // Why: The thread owns the output from now on; nothing else touches it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // new Worker(() => drain(receiver, output, shared), { name: 'ide-log-writer' });
    // ```
    thread::Builder::new()
        .name("ide-log-writer".to_string())
        .spawn(move || {
            drain(&receiver, &mut output, &for_thread);
        })?;
    let writer = BackgroundWriter {
        sender: sender.clone(),
        shared,
    };
    let flush = LogFlush { sender, grace };
    return Ok((writer, flush));
}

/// Queuing,
///  dropping,
///  gap reports,
///  and the exit flush with outputs the tests control.
#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
