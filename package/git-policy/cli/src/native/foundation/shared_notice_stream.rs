//! What:
//!  One stream of a command's notices, written by two threads:
//!  every line is rendered first, then written whole while the stream's own lock is held,
//!  and the lock is given back before either thread goes on with its work.
//! Why:
//!  A command's report belongs on the stream the command was given,
//! and a step that runs on a thread of its own beside the command
//!  must reach that same stream,
//! or the person reads half the report and a caller that handed in its own stream reads
//!  none of that half.
//! A stream is one mutable borrow,
//! so the two threads reach it through one small lock,
//! and the lock covers
//!  one line:
//! a line is rendered before the lock is taken,
//! and neither thread ever waits for the other's file work,
//!  because the lock is given back before that work starts.
//! A line the other thread writes can therefore never come inside this
//!  one.
//! A panic of either thread poisons the lock,
//! and a report is not worth losing to that,
//! so the poison is taken as
//!  the stream.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shared = new SharedNoticeStream({ held: mutex(stream), quiet: false });
//! await Promise.all([dispose(shared), repair(shared)]); // each line: lock, write whole, unlock
//! ```

use std::io::Write;
use std::sync::{Mutex, MutexGuard};

/// What:
///  The stream a command was given,
/// behind the one lock that the command and the step beside it write through.
/// Why:
///   Both threads must name the same stream,
/// which one mutable borrow cannot be lent to two of them without.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NoticeStream = Mutex<&mut (dyn Write + Send)>;
/// ```
pub(crate) type NoticeStream<'a> = Mutex<&'a mut (dyn Write + Send)>;

/// What:
///  A writer over one shared stream that puts each line on it whole,
/// and drops every line when the person asked for silence.
/// Why:
///   The repair of a new worktree and the emptying of its trash both report to the person,
/// and only one of them
///  can hold the stream's borrow:
/// this is the writer both are given instead of a second guess at where the person's stream is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SharedNoticeStream = { held: NoticeStream; quiet: boolean };
/// ```
pub(crate) struct SharedNoticeStream<'a, 'b> {
    /// The stream,
    /// held by the lock only while a line is written.
    pub(crate) held: &'a NoticeStream<'b>,
    /// Whether the person's `--quiet` silences this stream.
    pub(crate) quiet: bool,
}

impl<'a, 'b> SharedNoticeStream<'a, 'b> {
    /// Take the stream for one write,
    /// whether or not a panic poisoned the lock:
    /// an earlier report is no reason to lose this one.
    fn locked(&self) -> MutexGuard<'a, &'b mut (dyn Write + Send)> {
        return match self.held.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
    }
}

impl Write for SharedNoticeStream<'_, '_> {
    /// Write one piece of a line that the caller builds,
    /// whole,
    /// with the lock held.
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        if self.quiet {
            return Ok(buffer.len());
        }
        self.locked().write_all(buffer)?;
        return Ok(buffer.len());
    }

    /// Write one piece in one locked call,
    /// so the other thread's line never comes inside it.
    fn write_all(&mut self, buffer: &[u8]) -> std::io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        return self.locked().write_all(buffer);
    }

    /// Render the whole line before the lock is taken,
    /// then write it in one locked call.
    fn write_fmt(&mut self, arguments: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        let mut line: String = String::new();
        if std::fmt::Write::write_fmt(&mut line, arguments).is_err() {
            return Err(std::io::Error::other("the line could not be rendered"));
        }
        return self.write_all(line.as_bytes());
    }

    /// Push what the stream holds to where it goes,
    /// with the lock held.
    fn flush(&mut self) -> std::io::Result<()> {
        if self.quiet {
            return Ok(());
        }
        return self.locked().flush();
    }
}

/// Notice-stream controls stay out of the release executable.
#[cfg(test)]
#[path = "shared_notice_stream_tests.rs"]
mod tests;
