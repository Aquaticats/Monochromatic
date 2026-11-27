//! What:
//!  The sinks a control reads a notice from:
//! a buffer shared with the reader,
//! a buffer that runs an action once a counted line is finished,
//! and a stream that refuses every byte,
//! with the release's own standard error given one.
//! Why:
//!  Several controls watch a wait's notices without a timer,
//!  so the wait stops on an event and not on the host's speed;
//!  `test_support.rs` re-exports them and stays under the file-size limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sink = new SharedBuffer(); const counted = new ActionBuffer(sink, 3, () => done());
//! giveTheThread(new RefusingSink()); const notes = new StandardErrorOffTheCallingThread();
//! ```

/// The stream a release's notes are shown through,
/// and the answer its hand gives about one line.
/// Why:
///  A control shows a notice through the release stream
///  to read what a record does whose line never reached the person.
use crate::standard_error_off_the_calling_thread::{
    StandardErrorHand,
    StandardErrorOffTheCallingThread,
};

/// A sink a test reads after the code under test wrote its notices to a clone of it.
#[derive(Clone, Default)]
pub struct SharedSink(pub(crate) std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl SharedSink {
    /// Everything written so far,
    /// as text.
    pub fn text(&self) -> String {
        let bytes: Vec<u8> = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        return String::from_utf8_lossy(bytes.as_slice()).into_owned();
    }
}

/// Writes go to the shared buffer.
impl std::io::Write for SharedSink {
    /// Append the bytes.
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(bytes);
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
    pub fn new(
        shared: &SharedSink,
        after: usize,
        action: impl FnOnce() + Send + 'static,
    ) -> Self {
        return Self {
            shared: shared.clone(),
            lines: 0,
            after,
            action: Some(Box::new(action)),
        };
    }
}

/// Writes are copied,
/// and the action runs when the counted line is finished.
impl std::io::Write for ActionSink {
    /// Append the bytes,
    /// count finished lines,
    /// run the action at the counted one.
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        std::io::Write::write(&mut self.shared, bytes)?;
        self.lines += bytes
            .iter()
            .filter(|byte| {
                return **byte == b'\n';
            })
            .count();
        if self.lines >= self.after
            && let Some(action) = self.action.take()
        {
            action();
        }
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A stream that refuses every byte it is given,
///  as a closed terminal does.
/// Why:
///  A control whose subject is a notice the person could not be shown
///  needs a stream that says so,
///  both where `Notice::show` is handed a writer directly
///  and where the thread that writes standard error is given one.
///  One stand-in answers both,
///  so a control reads the same refusal either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AStreamThatRefusesEveryByte { write() { throw brokenPipe } flush() { throw brokenPipe } }
/// ```
pub(crate) struct AStreamThatRefusesEveryByte;

impl std::io::Write for AStreamThatRefusesEveryByte {
    /// Refuse the bytes,
    /// as a stream nobody reads does.
    fn write(
        &mut self,
        _bytes: &[u8],
    ) -> std::io::Result<usize> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }

    /// Refuse the flush,
    /// for the same reason.
    fn flush(&mut self) -> std::io::Result<()> {
        return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
    }
}

/// What:
///  This process's standard error as the stream the notices are shown through,
///  with the thread that writes it given a stream that refuses every byte.
/// Why:
///   A control that shows a notice through the release stream meets the answer the thread gives
///  when the line did not reach the person,
///  which is the answer a notice's record outlives the line by;
///  a writer handed to `show` directly cannot show that.
///  The queue and its thread are one for the whole process,
///  so the control that calls this runs its body in a child of its own
///  (`run_the_body_in_a_child_of_its_own`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// giveTheThread(new AStreamThatRefusesEveryByte()); return new StandardErrorOffTheCallingThread()
/// ```
pub(crate) fn standard_error_whose_writer_refuses_every_line()
-> StandardErrorOffTheCallingThread<StandardErrorHand> {
    crate::diagnostic_lines_off_the_calling_thread::give_the_thread(Box::new(
        AStreamThatRefusesEveryByte,
    ));
    return StandardErrorOffTheCallingThread::new();
}
