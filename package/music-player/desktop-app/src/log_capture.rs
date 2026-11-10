//! Capturing the records the installed logging pipeline would print,
//! for the tests that assert on them.
//!
//! What:
//! `Capture` is a writer that collects a subscriber's records into a buffer,
//! and `capture_records` runs one closure under a
//! subscriber built from the application's own filter.
//! Why:
//! the `log` bridge and the filter's target directives are only observable through what reaches a sink,
//! and no test
//! may install the pipeline globally in a test binary that runs tests in parallel.

/// What:
/// `Arc` shares the buffer with every writer the subscriber makes,
/// and `Mutex` lets the test thread read it.
/// Why:
/// A subscriber hands out a fresh writer per record,
/// so the buffer must outlive each one.
use std::sync::{Arc, Mutex};

/// What:
/// `Capture` is the writer the subscriber writes each record through.
/// Why:
/// It collects the record's bytes instead of printing them.
#[derive(Clone)]
pub(crate) struct Capture {
    /// The bytes written so far,
    /// shared with the test.
    pub(crate) buffer: Arc<Mutex<Vec<u8>>>,
}

/// What:
/// `Write` appends a record's bytes to the shared buffer.
/// Why:
/// That is the sink's whole job.
impl std::io::Write for Capture {
    /// What:
    /// `write` appends `bytes` and accepts the whole slice.
    /// Why:
    /// The formatter writes each record in several calls.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.buffer.lock().expect("the capture buffer is readable").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    /// What:
    /// `flush` does nothing,
    /// because a buffer needs no flushing.
    /// Why:
    /// `Write` requires it.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
/// `MakeWriter` hands the subscriber one `Capture` per record.
/// Why:
/// The formatter asks for a writer for every record it emits.
impl<'writer> tracing_subscriber::fmt::MakeWriter<'writer> for Capture {
    /// The writer type,
    /// which is this same capturing type.
    type Writer = Capture;

    /// What:
    /// `make_writer` returns another handle on the shared buffer.
    /// Why:
    /// Every record must land in the one buffer the test reads.
    fn make_writer(&'writer self) -> Self::Writer {
        return self.clone();
    }
}

/// What:
/// `capture_records` runs `emit` with a subscriber that has the application's filter and a buffer in place of stderr,
/// and returns what was written.
/// Why:
/// A test needs the records the installed pipeline would print,
/// and the pipeline itself must not be installed globally.
pub(crate) fn capture_records(rust_log: Option<&str>, emit: impl FnOnce()) -> String {
    let buffer: Arc<Mutex<Vec<u8>>> = Arc::default();
    let sink = Capture {
        buffer: Arc::clone(&buffer),
    };
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(crate::logging::filter(rust_log))
        .with_ansi(false)
        .with_writer(sink)
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    emit();
    drop(guard);
    return String::from_utf8_lossy(&buffer.lock().expect("the captured records are readable")).into_owned();
}
