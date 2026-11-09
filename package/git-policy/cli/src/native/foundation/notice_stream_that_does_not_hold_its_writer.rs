//! What:
//!  A stream of a command's notices that is written by a thread of its own,
//! so the code that says its lines never waits for the reader of them.
//! Why:
//!  A stream whose reader has stopped draining waits in the kernel until it resumes,
//! and the command's own thread must go on
//!  reading the service's answer,
//! withdrawing a request that reached its ceiling and keeping its scratch directory fresh.
//!  A reader that is not taking lines may not hold any of that,
//! so a line is handed to the writer thread and dropped when the line before it is
//!  still being written;
//! the same bargain `WARNING_WRITE_CEILING` makes for one warning line (`wait_ceilings.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const notices = new NoticeStreamThatDoesNotHoldItsWriter(sink); notices.write(line); notices.finish();
//! ```

use crate::diagnostic_log::debug;
use crate::wait_ceilings::WARNING_WRITE_CEILING;
use std::io::Write;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, channel, sync_channel};

/// The tag of this module's diagnostic lines.
const TAG: &str = "NoticeStreamThatDoesNotHoldItsWriter";

/// The thread name the writer carries in a process listing,
/// so a writer held by a reader is recognisable.
const WRITER_THREAD_NAME: &str = "cli-git-notice-lines";

/// What:
///  How many lines may be waiting for the writer before a further line is dropped.
/// Why:
///   Every line of a wait is advisory beside the command's own result,
/// and a reader that has stopped draining must cost the
///  command nothing,
/// so the queue is short.
///  Measured:
/// this repository's commit timing log (both generations,
/// 640 records that state it,
/// 2026-11-09)
///  holds no more than one waiting notice for any commit (0 in 634 records,
/// 1 in 6),
/// and writing one line to a pipe whose
///  reader drains it took 0.006 ms at the median and 3.6 ms at the slowest of 200
/// (`wait_ceilings.rs`,
///  `WARNING_WRITE_CEILING`,
/// measured on this host,
/// 2026-11-09),
/// so four lines are 14.4 ms of the
///  writer's work at its slowest measured pace.
pub const LINES_WAITING_FOR_THE_WRITER: usize = 4;

/// What:
///  A notice stream whose lines one thread writes while the command goes on,
/// and which drops a line rather than hold the writer.
/// Why:
///  The command's thread says its lines between two reads of the service's answer,
/// and a reader that stopped
///  draining must not be able to stop that reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NoticeStreamThatDoesNotHoldItsWriter implements Write { write(bytes): number; finish(): void }
/// ```
pub struct NoticeStreamThatDoesNotHoldItsWriter {
    /// Where whole lines go,
    /// and nothing once the stream is finished or the writer could not be started.
    lines: Option<SyncSender<Vec<u8>>>,
    /// The bytes of the line being written,
    /// which a formatting call may hand over in several pieces.
    buffered: Vec<u8>,
    /// How many lines were dropped because the writer was still writing the one before.
    dropped: usize,
    /// The writer's last word,
    /// waited for no longer than `WARNING_WRITE_CEILING` by `finish`.
    finished: Receiver<()>,
}

impl NoticeStreamThatDoesNotHoldItsWriter {
    /// What:
    ///  Starts the stream that writes `sink`'s lines on a thread of its own.
    /// Why:
    ///   The thread owns the sink,
    /// so a line still waiting when the command ends is the thread's to finish or to leave,
    /// and the
    ///  command is never the one waiting for a reader.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const notices = NoticeStreamThatDoesNotHoldItsWriter.start(sink)
    /// ```
    pub fn start(sink: Box<dyn Write + Send>) -> Self {
        let (sender, lines): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel::<Vec<u8>>(LINES_WAITING_FOR_THE_WRITER);
        let (told, finished): (std::sync::mpsc::Sender<()>, Receiver<()>) = channel::<()>();
        let started = std::thread::Builder::new().name(String::from(WRITER_THREAD_NAME)).spawn(move || {
            write_lines(sink, &lines, told);
        });
        return match started {
            Ok(_detached) => Self { lines: Some(sender), buffered: Vec::new(), dropped: 0, finished },
            // The system refused the thread: the lines of a wait are advisory, so they are dropped rather than written
            //  by the command's own thread, which a reader that stopped draining could then hold.
            Err(error) => {
                debug(TAG, format!("the system refused the thread for the notices ({error}), so no notice line is written").as_str());
                Self { lines: None, buffered: Vec::new(), dropped: 0, finished }
            }
        };
    }

    /// How many lines were dropped because the writer still had the one before.
    pub fn dropped_lines(&self) -> usize {
        return self.dropped;
    }

    /// What:
    ///  Hands over the line still being built,
    /// ends the writer when that line is done,
    /// and waits for its last word no longer than `WARNING_WRITE_CEILING`.
    /// Why:
    ///   A command that exits before the writer reached the reader loses the last lines,
    /// and a reader that stopped draining
    ///  must not become a wait with no end,
    /// so the wait is the ceiling of an optional warning line and no more.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// notices.finish() // waits at most WARNING_WRITE_CEILING for the writer
    /// ```
    pub fn finish(&mut self) {
        let rest: Vec<u8> = std::mem::take(&mut self.buffered);
        if !rest.is_empty() {
            self.hand_over(rest);
        }
        // The writer ends when every line is written and the stream that fed it is gone.
        drop(self.lines.take());
        if self.finished.recv_timeout(WARNING_WRITE_CEILING).is_err() {
            debug(TAG, format!("the writer had not written its last line within {WARNING_WRITE_CEILING:?}").as_str());
        }
    }

    /// Hands one whole line to the writer,
    /// or drops it and counts it when the writer still has the one before.
    fn hand_over(&mut self, line: Vec<u8>) {
        let Some(lines) = self.lines.as_ref() else {
            self.dropped = self.dropped.saturating_add(1);
            return;
        };
        match lines.try_send(line) {
            Ok(()) => {}
            Err(TrySendError::Full(_dropped_line)) | Err(TrySendError::Disconnected(_dropped_line)) => {
                // Said once: a reader that stopped draining drops every line after the first, and one line says why.
                if self.dropped == 0 {
                    debug(TAG, "a notice line was dropped because the reader was not taking the one before it");
                }
                self.dropped = self.dropped.saturating_add(1);
            }
        }
    }
}

/// What:
///  Writes every line the command hands over,
/// and says so at the end.
/// Why:
///   The thread is the only writer of the sink,
/// so two lines never interleave;
/// a stream that refuses a line ends the
///  thread,
/// because every later line would be refused too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of lines) { if (sink.writeSync(line).error) break; } finished.send();
/// ```
fn write_lines(mut sink: Box<dyn Write + Send>, lines: &Receiver<Vec<u8>>, finished: std::sync::mpsc::Sender<()>) {
    for line in lines {
        // A stream that refuses a line ends the thread: every line after it would be refused too.
        if sink.write_all(line.as_slice()).is_err() {
            break;
        }
        if sink.flush().is_err() {
            break;
        }
    }
    // A command that stopped waiting is gone with its receiver, and the word goes with the send.
    let _told = finished.send(());
}

/// Whether `byte` ends a line,
/// which is where a whole line of the stream ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0x0a
/// ```
fn ends_a_line(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Appends the bytes to the line being built,
/// and hands over each whole line it completes.
impl Write for NoticeStreamThatDoesNotHoldItsWriter {
    /// Append the bytes,
    /// and hand over each line it completes.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffered.extend_from_slice(bytes);
        while let Some(end) = self.buffered.iter().position(ends_a_line) {
            let line: Vec<u8> = self.buffered.drain(..=end).collect();
            self.hand_over(line);
        }
        return Ok(bytes.len());
    }

    /// Hand over the bytes that no newline ended yet.
    fn flush(&mut self) -> std::io::Result<()> {
        let rest: Vec<u8> = std::mem::take(&mut self.buffered);
        if !rest.is_empty() {
            self.hand_over(rest);
        }
        return Ok(());
    }
}

/// The notice stream is controlled in the test module.
#[cfg(test)]
#[path = "notice_stream_that_does_not_hold_its_writer_tests.rs"]
mod tests;
