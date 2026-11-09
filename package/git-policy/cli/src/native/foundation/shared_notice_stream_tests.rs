//! What one stream written by two threads does:
//! each line lands whole,
//! `--quiet` drops every line,
//! a panic of one thread does not cost the other its report,
//! and a flush reaches the stream.

use super::*;
use std::sync::Arc;

/// A stream that records every write it is given,
/// so a control can see where one line ended and the next began.
#[derive(Default)]
struct Recorded {
    /// Each write the stream was given,
    /// in the order it arrived.
    pieces: Vec<Vec<u8>>,
    /// Whether the stream was asked to flush.
    flushed: bool,
}

/// The stream's recording,
/// shared with the thread a control starts.
type Recording = Arc<Mutex<Recorded>>;

/// A writer over the recording that keeps each call separate.
struct Recorder(Recording);

impl Write for Recorder {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("recording").pieces.push(buffer.to_vec());
        return Ok(buffer.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().expect("recording").flushed = true;
        return Ok(());
    }
}

/// One line written through a shared stream is one write to the stream,
/// so the other thread's line can never come inside it,
/// and a flush reaches the stream.
#[test]
fn a_line_reaches_the_stream_whole() {
    let recording: Recording = Arc::new(Mutex::new(Recorded::default()));
    let mut writer = Recorder(Arc::clone(&recording));
    let stream: NoticeStream<'_> = Mutex::new(&mut writer);
    let mut shared = SharedNoticeStream { held: &stream, quiet: false };
    shared.write_all(b"first\n").expect("write");
    writeln!(shared, "the {} line", 2).expect("write");
    shared.flush().expect("flush");
    let recorded = recording.lock().expect("recording");
    assert_eq!(recorded.pieces, vec![b"first\n".to_vec(), b"the 2 line\n".to_vec()], "one write per line");
    assert!(recorded.flushed, "the flush reaches the stream");
}

/// Two threads write their lines through the same stream,
/// and every line arrives whole.
#[test]
fn two_threads_write_lines_that_never_mix() {
    let recording: Recording = Arc::new(Mutex::new(Recorded::default()));
    let mut writer = Recorder(Arc::clone(&recording));
    let stream: NoticeStream<'_> = Mutex::new(&mut writer);
    let held: &NoticeStream<'_> = &stream;
    std::thread::scope(|scope| {
        for each in 0..2 {
            scope.spawn(move || {
                let mut line_writer = SharedNoticeStream { held, quiet: false };
                for index in 0..16 {
                    writeln!(line_writer, "{each}:{index}").expect("write");
                }
            });
        }
    });
    let recorded = recording.lock().expect("recording");
    assert_eq!(recorded.pieces.len(), 32, "one write per line");
    for piece in &recorded.pieces {
        let text: String = String::from_utf8(piece.clone()).expect("utf8");
        assert_eq!(text.matches(':').count(), 1, "a line from one thread is not split by the other's: {text:?}");
        assert!(text.ends_with('\n'), "and it is complete: {text:?}");
    }
}

/// A stream told the person asked for silence drops every line and flushes nothing.
#[test]
fn quiet_drops_every_line() {
    let recording: Recording = Arc::new(Mutex::new(Recorded::default()));
    let mut writer = Recorder(Arc::clone(&recording));
    let stream: NoticeStream<'_> = Mutex::new(&mut writer);
    let mut shared = SharedNoticeStream { held: &stream, quiet: true };
    shared.write_all(b"never\n").expect("write");
    writeln!(shared, "never either").expect("write");
    assert_eq!(shared.write(b"nor this").expect("write"), b"nor this".len(), "the caller is told its bytes were taken");
    shared.flush().expect("flush");
    let recorded = recording.lock().expect("recording");
    assert!(recorded.pieces.is_empty(), "{:?}", recorded.pieces);
    assert!(!recorded.flushed, "nothing is flushed to a stream that is silenced");
}

/// A thread that panicked while holding the stream does not cost the other thread its report.
#[test]
fn a_poisoned_lock_still_takes_the_next_line() {
    let recording: Recording = Arc::new(Mutex::new(Recorded::default()));
    let mut writer = Recorder(Arc::clone(&recording));
    let stream: NoticeStream<'_> = Mutex::new(&mut writer);
    let held: &NoticeStream<'_> = &stream;
    std::thread::scope(|scope| {
        let panicked = scope.spawn(move || {
            let _guard = held.lock().expect("the stream");
            panic!("a thread that held the stream stopped here");
        });
        assert!(panicked.join().is_err(), "the control's own thread panicked while holding the stream");
    });
    let mut shared = SharedNoticeStream { held: &stream, quiet: false };
    shared.write_all(b"the line after the panic\n").expect("a poisoned stream still takes a report");
    let recorded = recording.lock().expect("recording");
    assert_eq!(recorded.pieces, vec![b"the line after the panic\n".to_vec()], "{:?}", recorded.pieces);
}
