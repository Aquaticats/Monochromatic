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
//!  A write answers whether the line it handed over reached the stream:
//!  the thread that writes the line is the only place that knows,
//!  so the hand-over answers (`HandedOver`),
//!  and a line it did not put in the stream is an error of the write that completed it.
//!  A caller that says its line and goes on ignores that error,
//!  which is every note of the commit path;
//!  a caller that must not lose what its line says keeps it
//!  (`Notice::show`),
//!  and the record of a failed push or a kept worktree then stays for the next command.
//!  The wait behind that answer is the queue's own (`WARNING_WRITE_CEILING`),
//!  paid once per process and never past the first line a stopped reader did not take,
//!  so a reader that stops draining still holds no command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stream = new StandardErrorOffTheCallingThread(); stream.write(line); stream.flush();
//! ```

/// The door every line goes through.
use crate::diagnostic_lines_off_the_calling_thread::write_bytes;
/// What the door answers about one line,
/// which is what a write of this stream answers through its error.
use crate::lines_written_by_a_thread::HandedOver;
/// Writing one line into the thread that owns the stream,
/// and the error a stream may give.
use std::io::Write;

/// What:
///  The hand every line of a release's standard error goes through:
///  the queue's own thread,
///  which answers whether the line reached the stream.
/// Why:
///  A site that names the stream names this hand as its type,
///  so the answer is part of the type and no site can be built around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StandardErrorHand = (line: Uint8Array) => 'Written' | 'Unwritten';
/// ```
pub type StandardErrorHand = fn(&[u8]) -> HandedOver;

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
///  A line is not the same as the record behind it:
///  a write answers an error when the line it completed did not reach the stream,
///  so the notice whose words those are keeps its record for the next command
///  (`Notice::show`),
///  while every note that says nothing a record holds leaves that error to its caller's silence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class StandardErrorOffTheCallingThread implements Write { #buffered = new Uint8Array(0); write(bytes) { this.#handOver(); this.#answer() } flush() { this.#handOverTheRest(); this.#answer() } }
/// ```
pub struct StandardErrorOffTheCallingThread<H: FnMut(&[u8]) -> HandedOver> {
    /// The bytes of the line being built,
    /// which a formatting call may hand over in several pieces.
    buffered: Vec<u8>,
    /// Where a whole line goes,
    /// which is a control's own collector in a test and the diagnostic thread in a release.
    hand: H,
    /// Whether a line handed over during the call now running did not reach the stream.
    /// Why it is kept until the call answers:
    ///  one write may complete more than one line,
    ///  and the caller reads one answer for the bytes it gave.
    unwritten: bool,
}

impl StandardErrorOffTheCallingThread<StandardErrorHand> {
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
            unwritten: false,
        };
    }
}

impl Default for StandardErrorOffTheCallingThread<StandardErrorHand> {
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

impl<H: FnMut(&[u8]) -> HandedOver> StandardErrorOffTheCallingThread<H> {
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
            unwritten: false,
        };
    }

    /// What:
    ///  Hands over every whole line the buffer now holds,
    ///  then a stretch with no end that has grown past a block,
    ///  keeps the answer to each,
    ///  and keeps the rest.
    /// Why:
    ///  The queue takes one line whole,
    ///  so two lines of a commit and a diagnostic beside it never arrive interleaved,
    ///  and a line whose end has not arrived is said once rather than in pieces.
    ///  The answer is kept here rather than returned,
    ///  because the caller reads one answer for the bytes of one write,
    ///  which may complete more than one line.
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
            if (self.hand)(&self.buffered[taken..=index]) == HandedOver::Unwritten {
                self.unwritten = true;
            }
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
        if (self.hand)(rest.as_slice()) == HandedOver::Unwritten {
            self.unwritten = true;
        }
    }

    /// What:
    ///  The answer for the line the call that just ended handed over:
    ///  nothing when every line it completed is in the stream,
    ///  and an error when one of them is not.
    /// Why:
    ///  A caller that must not lose what its line says reads this error
    ///  and keeps the record the line was about;
    ///  the thread that writes the line is the only place that knows,
    ///  so the write itself could not answer it.
    ///  The kind is the one a closed terminal gives,
    ///  because a stream that took no line is gone for every later line as well.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const answer = () => { if (!this.#unwritten) return; this.#unwritten = false; throw brokenPipe('a line this command said did not reach standard error') }
    /// ```
    fn answer(&mut self) -> std::io::Result<()> {
        if !std::mem::take(&mut self.unwritten) {
            return Ok(());
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "a line this command said did not reach standard error",
        ));
    }
}

impl<H: FnMut(&[u8]) -> HandedOver> Write for StandardErrorOffTheCallingThread<H> {
    /// Append the bytes,
    /// hand over each whole line they complete,
    /// and answer an error when a line they completed did not reach the stream.
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.buffered.extend_from_slice(bytes);
        self.hand_over();
        self.answer()?;
        return Ok(bytes.len());
    }

    /// Hand over the bytes that no newline ended yet,
    /// and answer an error when they did not reach the stream.
    fn flush(&mut self) -> std::io::Result<()> {
        self.hand_over_what_is_left();
        return self.answer();
    }
}

/// The stream's own behaviour is a control in the test module.
#[cfg(test)]
#[path = "standard_error_off_the_calling_thread_tests.rs"]
mod tests;
