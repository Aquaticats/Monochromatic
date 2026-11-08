//! What:
//!  The sinks a control reads a notice from:
//! a buffer shared with the reader and a buffer that runs an action once a counted line is finished.
//! Why:
//!  Several controls watch a wait's notices without a timer,
//!  so the wait stops on an event and not on the host's speed;
//!  `test_support.rs` re-exports them and stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sink = new SharedBuffer(); const counted = new ActionBuffer(sink, 3, () => done());
//! ```

/// A sink a test reads after the code under test wrote its notices to a clone of it.
#[derive(Clone, Default)]
pub struct SharedSink(pub(crate) std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl SharedSink {
    /// Everything written so far,
    /// as text.
    pub fn text(&self) -> String {
        let bytes: Vec<u8> = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        return String::from_utf8_lossy(bytes.as_slice()).into_owned();
    }
}

/// Writes go to the shared buffer.
impl std::io::Write for SharedSink {
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
///  A sink that keeps what is written in a shared buffer and runs an action once,
/// when the `after`th line is finished.
/// Why:
///   A control of a repeated notice ends the wait it watches from inside the notice that counts:
///  the wait stops on that event and not on a timer,
///  so the count of notices holds however slowly a loaded host runs the waiting thread.
pub struct ActionSink {
    /// The buffer the test reads.
    shared: SharedSink,
    /// How many lines were finished.
    lines: usize,
    /// The line after which the action runs.
    after: usize,
    /// The action,
    /// until it has run.
    action: Option<Box<dyn FnOnce() + Send>>,
}

impl ActionSink {
    /// A sink that copies into `shared` and runs `action` once the `after`th line is finished.
    pub fn new(shared: &SharedSink, after: usize, action: impl FnOnce() + Send + 'static) -> Self {
        return Self { shared: shared.clone(), lines: 0, after, action: Some(Box::new(action)) };
    }
}

/// Writes are copied,
/// and the action runs when the counted line is finished.
impl std::io::Write for ActionSink {
    /// Append the bytes,
    /// count finished lines,
    /// run the action at the counted one.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        std::io::Write::write(&mut self.shared, bytes)?;
        self.lines += bytes.iter().filter(|byte| { return **byte == b'\n'; }).count();
        if self.lines >= self.after && let Some(action) = self.action.take() {
            action();
        }
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}
