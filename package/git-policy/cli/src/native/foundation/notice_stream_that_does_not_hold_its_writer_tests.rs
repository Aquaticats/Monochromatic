//! Controls for the notice stream that does not hold its writer:
//! a reader that takes every line,
//! a reader that stopped draining,
//! and the ceiling of the wait for the writer at the end.

use super::*;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

/// What:
///  How long a control waits for something a healthy run does in microseconds.
/// Why:
///   A loaded host adds scheduling time and never work,
/// so only a hang reaches it.
const PATIENCE: Duration = Duration::from_secs(10);

/// What:
///  A sink that keeps what it is given,
/// shared with the control that reads it.
/// Why:
///   The writer is a thread of its own,
/// so the control reads the lines through a lock rather than through a borrow the thread would need.
#[derive(Clone, Default)]
struct KeptLines(Arc<Mutex<Vec<u8>>>);

impl KeptLines {
    /// Everything written so far, as text.
    fn text(&self) -> String {
        let bytes: Vec<u8> = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        return String::from_utf8_lossy(bytes.as_slice()).into_owned();
    }
}

impl Write for KeptLines {
    /// Append the bytes.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A sink that takes one line and then stops,
/// as a pipe whose reader is open and no longer draining does.
/// Why:
///   This is the reader the stream must not wait for;
/// it says when a line reached it and waits to be let go.
struct StoppedReader {
    /// Says that a line reached the sink and is held there.
    held: mpsc::Sender<()>,
    /// Where the sink waits until the control lets the line go.
    release: mpsc::Receiver<()>,
    /// What the sink took once it was let go.
    kept: KeptLines,
    /// How many lines it took.
    took: usize,
}

impl Write for StoppedReader {
    /// Announce the line, wait to be released, and keep it.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let _told = self.held.send(());
        let _let_go = self.release.recv();
        self.kept.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(bytes);
        self.took = self.took.saturating_add(1);
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The lines a reader that takes them receives, in the order they were said.
#[test]
fn a_reader_that_takes_every_line_receives_them_in_order() {
    let kept: KeptLines = KeptLines::default();
    let mut stream = NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(kept.clone()));
    let _first = write!(stream, "This commit is next.\n");
    let _second = write!(stream, "Waiting for 2 earlier commits.\n");
    stream.finish();
    assert_eq!(kept.text(), "This commit is next.\nWaiting for 2 earlier commits.\n");
    assert_eq!(stream.dropped_lines(), 0, "nothing is dropped when the reader takes every line");
}

/// A line a formatting call hands over in pieces reaches the reader as one line,
/// and the piece no newline ended is handed over by `flush`.
#[test]
fn a_line_handed_over_in_pieces_reaches_the_reader_whole() {
    let kept: KeptLines = KeptLines::default();
    let mut stream = NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(kept.clone()));
    let _head = write!(stream, "Waiting for ");
    let _tail = write!(stream, "3 earlier commits.");
    let _flushed = stream.flush();
    let _next = write!(stream, "\nThis commit is next.\n");
    stream.finish();
    assert_eq!(kept.text(), "Waiting for 3 earlier commits.\nThis commit is next.\n");
}

/// What:
///  A reader that stopped taking lines holds neither the writer nor the end of the command,
/// and the lines it could not take are counted.
/// Why:
///   This is the whole bound:
/// a line of a wait is advisory,
/// and a reader that stopped draining waits in the kernel with no end.
#[test]
fn a_reader_that_stopped_taking_lines_holds_neither_the_writer_nor_the_end() {
    let kept: KeptLines = KeptLines::default();
    let (held_sender, held) = mpsc::channel::<()>();
    let (release_sender, release) = mpsc::channel::<()>();
    let reader: StoppedReader = StoppedReader { held: held_sender, release, kept: kept.clone(), took: 0 };
    let mut stream = NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(reader));
    // Far more lines than the stream may hold waiting for the writer, so a line is dropped.
    let saying: Instant = Instant::now();
    for number in 0..(LINES_WAITING_FOR_THE_WRITER * 4) {
        let _line = writeln!(stream, "Waiting for {number} earlier commits.");
    }
    assert!(saying.elapsed() < PATIENCE, "saying the lines does not wait for the reader: {:?}", saying.elapsed());
    assert!(held.recv_timeout(PATIENCE).is_ok(), "a line reached the reader, which then held it");
    let ending: Instant = Instant::now();
    stream.finish();
    let took: Duration = ending.elapsed();
    assert!(took < WARNING_WRITE_CEILING + PATIENCE, "the end waits no longer than the ceiling of an optional line: {took:?}");
    assert!(stream.dropped_lines() > 0, "the lines the writer could not take are counted");
    // The reader is let go, and what it took is what was said while it was reading.
    drop(release_sender);
    let began: Instant = Instant::now();
    while kept.text().is_empty() && began.elapsed() < PATIENCE {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(kept.text().starts_with("Waiting for 0 earlier commits."), "the first line said is the first written: {:?}", kept.text());
}

/// What:
///  A stream whose writer the system refused drops its lines and counts them,
/// and the code that says them is not held.
/// Why:
///   A thread the system refuses is "no writer" and not "write it yourself":
/// the command's own thread must never be
///  the one a reader can hold.
#[test]
fn a_stream_with_no_writer_says_nothing_and_counts_every_line() {
    let mut stream: NoticeStreamThatDoesNotHoldItsWriter = NoticeStreamThatDoesNotHoldItsWriter { lines: None, buffered: Vec::new(), dropped: 0, finished: std::sync::mpsc::channel::<()>().1 };
    let saying: Instant = Instant::now();
    for _line in 0..8 {
        let _said = writeln!(stream, "This commit is next.");
    }
    assert!(saying.elapsed() < PATIENCE, "{:?}", saying.elapsed());
    stream.finish();
    assert_eq!(stream.dropped_lines(), 8, "every line a stream with no writer is given is dropped and counted");
}

/// What:
///  The commit command hands its waiting notices to this stream and to nothing else.
/// Why:
///   The bound is only real where the stream is,
/// and this is the one place the commit path chooses it:
/// a later change that goes back to
///  writing those lines straight to standard error would hold the commit again,
/// and this control names that.
#[test]
fn the_commit_command_hands_its_waiting_notices_to_this_stream() {
    let commit_path: &str = include_str!("../landing/service_commit.rs");
    assert!(commit_path.contains("NoticeStreamThatDoesNotHoldItsWriter::start(Box::new(std::io::stderr()))"), "the commit path starts the stream of its waiting notices");
    assert!(commit_path.contains("submit_traced(&waiting_config, &sent, &mut notices, &mut timing.submit)"), "the stream is what the client writes its notices to");
    assert!(!commit_path.contains("submit_traced(&waiting_config, &sent, &mut std::io::stderr()"), "no notice of the wait goes straight to standard error");
}
