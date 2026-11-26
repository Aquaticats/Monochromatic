//! What:
//!  This process's standard error as a stream whose every whole line is handed to the thread
//!  that writes it,
//!  so the thread that says the line never waits for the reader.
//! Why:
//!  A write to a pipe or a terminal whose reader has stopped draining waits in the kernel until
//!  it resumes,
//!  and the commit says its notes on its own thread:
//!  the policy events of a landing,
//!  the refusal Git answered with,
//!  the warning about conflict markers,
//!  the note about the builds that did not land.
//!  A reader that stopped reading must cost those lines and never the commit,
//!  whose exit status already carries the result (issue 6421).
//!  So they go where every diagnostic line goes
//!  (`diagnostic_lines_off_the_calling_thread.rs`):
//!  a bounded queue whose own thread writes this process's standard error descriptor,
//!  which takes no lock the shared handle would hold,
//!  and a caller that waits only while the reader keeps up.
//!  The queue takes whole lines,
//!  so this stream keeps the bytes of the line being built until its newline arrives,
//!  and hands the rest over at a block,
//!  because a program below the commit may print a line with no end.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = new StandardErrorOffTheCallingThread(); stream.write(line); stream.flush();
//! ```

/// The door every line goes through.
use crate::diagnostic_lines_off_the_calling_thread::write_bytes;
/// Writing one line into the thread that owns the stream,
/// and the error a stream may give.
use std::io::Write;

/// How long a stretch with no newline may grow before it is handed over as it stands.
/// Why:
///  A program below the commit can write bytes with no newline at all,
///  and a line that never ends is not a reason to hold them in this buffer:
///  one pipe block is what a reader takes at a time,
///  so a stretch of this many bytes goes on to the reader rather than waiting here.
///  Measured:
///  the longest line of the wrapper's own notes is the policy event of a finding,
///  a few hundred bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LONGEST_STRETCH_WITHOUT_AN_END = 4096
/// ```
const LONGEST_STRETCH_WITHOUT_AN_END: usize = 4096;

/// What:
///  This process's standard error,
///  as a stream that hands each whole line to the thread that writes it and returns at once.
/// Why:
///  A note of the commit's own path is a line a person can live without,
///  so the reader's pace must not reach the command that says it.
///  The stream is a value of its own and holds no lock,
///  because the queue is the one place the lines wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StandardErrorOffTheCallingThread implements Write { #buffered = new Uint8Array(0); write(bytes) {} flush() {} }
/// ```
pub struct StandardErrorOffTheCallingThread<H: FnMut(&[u8])> {
    /// The bytes of the line being built,
    /// which a formatting call may hand over in several pieces.
    buffered: Vec<u8>,
    /// Where a whole line goes,
    /// which is a control's own collector in a test and the diagnostic thread in a release.
    hand: H,
}

impl StandardErrorOffTheCallingThread<fn(&[u8])> {
    /// What:
    ///  The stream of this process's standard error.
    /// Why:
    ///   A caller writes its notes here instead of asking for the shared handle,
    ///  whose lock a reader that stopped draining would hold.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new StandardErrorOffTheCallingThread()
    /// ```
    pub fn new() -> Self {
        return Self {
            buffered: Vec::new(),
            hand: write_bytes,
        };
    }
}

impl Default for StandardErrorOffTheCallingThread<fn(&[u8])> {
    /// What:
    ///  The same stream `new` makes,
    ///  so a caller that takes the default gets this process's standard error too.
    /// Why:
    ///  A stream with a constructor and no default is a lint finding,
    ///  and the two must never disagree about where a line goes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const byDefault = () => new StandardErrorOffTheCallingThread();
    /// ```
    fn default() -> Self {
        return Self::new();
    }
}

impl<H: FnMut(&[u8])> StandardErrorOffTheCallingThread<H> {
    /// What:
    ///  The stream that hands every whole line to `hand`,
    ///  and a line a control reads back.
    /// Why:
    ///   The thread that writes the line is this process's own,
    ///  and a control must read the lines without touching that thread or the state a second
    ///  control changes beside it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new StandardErrorOffTheCallingThread({ hand })
    /// ```
    #[cfg(test)]
    fn handing_to(hand: H) -> Self {
        return Self {
            buffered: Vec::new(),
            hand,
        };
    }

    /// What:
    ///  Hands over every whole line the buffer now holds,
    ///  then a stretch with no end that has grown past a block,
    ///  and keeps the rest.
    /// Why:
    ///  The queue takes one line whole,
    ///  so two lines of a commit and a diagnostic beside it never arrive interleaved,
    ///  and a line whose end has not arrived is said once rather than in pieces.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// let taken = 0; for (const [index, byte] of this.#buffered.entries()) { if (byte !== 0x0a) continue; this.#hand(this.#buffered.subarray(taken, index + 1)); taken = index + 1 } this.#buffered = this.#buffered.subarray(taken)
    /// if (this.#buffered.length >= LONGEST_STRETCH_WITHOUT_AN_END) this.#hand(whatIsLeft())
    /// ```
    fn hand_over(&mut self) {
        let mut taken: usize = 0;
        for (index, byte) in self.buffered.iter().enumerate() {
            if *byte != b'\n' {
                continue;
            }
            (self.hand)(&self.buffered[taken..=index]);
            taken = index + 1;
        }
        if taken > 0 {
            self.buffered.drain(..taken);
        }
        if self.buffered.len() >= LONGEST_STRETCH_WITHOUT_AN_END {
            self.hand_over_what_is_left();
        }
    }

    /// Hand over the bytes of the line no newline ended,
    /// and empty the buffer.
    fn hand_over_what_is_left(&mut self) {
        if self.buffered.is_empty() {
            return;
        }
        let rest: Vec<u8> = std::mem::take(&mut self.buffered);
        (self.hand)(rest.as_slice());
    }
}

impl<H: FnMut(&[u8])> Write for StandardErrorOffTheCallingThread<H> {
    /// Append the bytes,
    /// and hand over each whole line they complete.
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.buffered.extend_from_slice(bytes);
        self.hand_over();
        return Ok(bytes.len());
    }

    /// Hand over the bytes that no newline ended yet.
    fn flush(&mut self) -> std::io::Result<()> {
        self.hand_over_what_is_left();
        return Ok(());
    }
}

/// The stream's own behaviour is a control in the test module.
#[cfg(test)]
#[path = "standard_error_off_the_calling_thread_tests.rs"]
mod tests;
