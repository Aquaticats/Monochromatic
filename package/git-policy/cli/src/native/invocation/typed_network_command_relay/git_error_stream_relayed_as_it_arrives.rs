//! What:
//!  Run one Git child whose standard error the wrapper relays as it arrives,
//!  keeping a bounded tail for the reading that decides which sentence follows.
//! Why:
//!  A typed network command must still draw what plain Git draws:
//!  the progress display, Git's colour and OpenSSH's touch notice are gated on `isatty(2)`,
//!  and a pipe answers no.
//!  The decision record's K2 gives the child a pseudo-terminal when the wrapper's own standard error is a terminal,
//!  and a pipe when it is not,
//!  so a capture holds plain Git's bytes byte for byte
//!  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`).
//!  The reading happens here and never on a worker:
//!  a blocking wait inside a task holds the very task that would end it
//!  (`source_guard`'s runtime rules).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const pair = isatty(2) ? openpty() : undefined; spawn(git, { stderr: pair?.slave ?? 'pipe' }); copy(pair?.master ?? child.stderr, 2);
//! ```

// What: `AsFd` is a borrow of one open descriptor, which `poll`, `read` and the flag calls all take.
// Why:  Every call below takes a descriptor without taking it away.
//
// In TS you'd write (pseudocode):
// ```ts
// type AsFd = { fd: number }; // not closed by the holder
// ```
use std::os::fd::AsFd;
// What: `OwnedFd` is one open descriptor whose closing is the value's own doing.
// Why:  The pseudo-terminal's ends and the child's error stream are opened here and closed by their values.
//
// In TS you'd write (pseudocode):
// ```ts
// type OwnedFd = number; // closed when the value is dropped
// ```
use std::os::fd::OwnedFd;
// What: `io` is the input and output error type every start, wait and read here answers with.
use std::io;
// What: `Child`, `Command` and `Stdio` are the started program, its description, and how its streams are connected.
use std::process::{
    Child,
    Command,
    Stdio,
};
// What: How the child ended, with the exit-code mapping the wrapper's exit uses.
use crate::forwarding::ChildOutcome;
// What: The guard whose handlers a started child is published to, and the token that points them at it.
use crate::signal_relay::SignalGuard;
use crate::signal_relay_relayed_child::RelayedChild;
// What: The three readings this file's halves are finished by: the terminal the child writes to, the bytes relayed from it, and
//       the descriptor flags the relay needs.
// Why:  The wait, the terminal and the copy each keep their own file under the package's length limit (`rust/max-lines`).
//
// In TS you'd write (pseudocode):
// ```ts
// import { openATerminalPair } from './a-terminal-the-child-writes-its-error-stream-to';
// import { copyWhatIsReadable, drainWhatIsLeft, makeNonBlocking } from './the-relayed-bytes';
// ```
use super::a_terminal_the_child_writes_its_error_stream_to::open_a_terminal_pair;
use super::the_relayed_bytes::{
    copy_what_is_readable,
    drain_what_is_left,
    make_non_blocking,
};

/// What:
///  How much of the child's error stream the relay keeps for the reading after the wait.
/// Why:
///   A clone prints megabytes of progress, and the words the reading needs are at the end;
///   keeping the whole stream would let one command hold all of it.
///   Sixty-four kilobytes is the bound the design names, and it holds a refusal's lines whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KEPT_TAIL_BYTES = 65536;
/// ```
pub(super) const KEPT_TAIL_BYTES: usize = 65536;

/// What:
///  How long one wait for the child's error stream lasts before the relay looks at the child again.
/// Why:
///   `poll` returns the moment bytes arrive, so this slice is only how often an idle stream is looked at;
///   it also bounds how long the relay takes to notice that the child ended while a process it started still holds the
///   stream, which the design's end rule says must not hold the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POLL_SLICE = { tv_sec: 0, tv_nsec: 100_000_000 }; // 100 ms
/// ```
pub(super) const POLL_SLICE: rustix::event::Timespec = rustix::event::Timespec {
    tv_sec: 0,
    tv_nsec: 100_000_000,
};

/// What:
///  One read's worth of the child's error stream.
/// Why:
///   A refusal's transcript is far smaller than this, and a progress update is not larger,
///   so one buffer reads a whole update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHUNK_BYTES = 8192;
/// ```
pub(super) const CHUNK_BYTES: usize = 8192;

/// What:
///  What the relay saw: how the child ended, its process number, and the tail of its error stream.
/// Why:
///   The caller prints the sentence from that tail and ends the way Git ended,
///   and the record of a lock Git may hold needs the child's number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelayedErrorStream = { outcome: ChildOutcome; childNumber: number; kept: Buffer };
/// ```
pub struct RelayedErrorStream {
    /// How the child ended.
    pub outcome: ChildOutcome,
    /// The child's process number, for the record of a lock it may hold.
    pub child_number: u32,
    /// The last bytes the child wrote to its error stream, and no more than `KEPT_TAIL_BYTES` of them.
    pub kept: Vec<u8>,
}

/// What:
///  Which stream the child's standard error is connected to.
/// Why:
///   K2 is decided once, by one `isatty` call on the wrapper's own standard error,
///   and both shapes then relay the same bytes to the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ErrorStreamShape = 'terminal' | 'pipe';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorStreamShape {
    /// The wrapper's standard error is a terminal: the child gets a pseudo-terminal.
    Terminal,
    /// The wrapper's standard error is not a terminal: the child gets a pipe.
    Pipe,
}

/// What:
///  The shape this process's child gets, from one `isatty` call on the wrapper's own standard error.
/// Why:
///   The design's K2 turns on exactly this call, and a second call could answer differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isatty(2) ? 'terminal' : 'pipe'
/// ```
pub fn shape_of_this_process() -> ErrorStreamShape {
    // What: `isatty` asks the kernel whether one descriptor is a terminal.
    // Why:  Plain Git draws its progress and OpenSSH its touch notice only when this answer is yes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (tty.isatty(2)) return 'terminal'; return 'pipe';
    // ```
    if rustix::termios::isatty(std::io::stderr()) {
        return ErrorStreamShape::Terminal;
    }
    return ErrorStreamShape::Pipe;
}

/// What:
///  A child already started with its error stream connected, and the descriptor the relay reads it from.
/// Why:
///   The stream is decided before the start (a terminal must be the child's `stderr` from its first instruction),
///   and read after it,
///   so the two halves are one value and no caller can start a child the relay cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RelayedStart { constructor(readonly child: Child, readonly source: OwnedFd) {} }
/// ```
pub struct RelayedStart {
    /// The child itself, owned here until the read that ends it, or nothing once it is taken.
    child: Option<Child>,
    /// The end the relay reads: the pseudo-terminal's master, or the pipe's reading end, or nothing once it is taken.
    source: Option<OwnedFd>,
    /// The token that points the signal relay at the child until the read is over, or nothing when no guard was given.
    relayed: Option<RelayedChild>,
}

/// What:
///  Start `command` with its standard error connected for the relay, published to this guard's handlers.
/// Why:
///   A signal a process sends to the wrapper must reach Git
///   (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`, "the child is published to the existing
///   signal relay"),
///   and the child's error stream must be the terminal or the pipe K2 chose before it starts.
///   The command is taken here rather than borrowed,
///   so this process's own handle on that stream is closed once the child
///   is running:
///   a wrapper that kept the terminal's slave open held its master from ever reporting the child's end,
///   so the read noticed Git's exit only at its next `poll` slice,
///   which cost about 94 ms on every typed ssh command at a terminal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pair = shape === 'terminal' ? openpty() : undefined;
/// command.stderr = pair?.slave ?? 'pipe';
/// const relayed = await guard.startRelayed(command);
/// command[Symbol.dispose](); // the wrapper's own handle on the child's stream, closed before the read
/// const source = pair?.master ?? relayed.child.stderr;
/// return { child, source, relayed };
/// ```
pub fn start_relaying(
    shape: ErrorStreamShape,
    guard: Option<&SignalGuard>,
    mut command: Command,
) -> io::Result<RelayedStart> {
    // What: `Option<OwnedFd>` is "a descriptor or nothing": the pseudo-terminal's master when K2 chose a terminal.
    // Why:  A pipe's reading end belongs to the child value, so only one shape opens a descriptor here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let master: number | undefined;
    // ```
    let mut master: Option<OwnedFd> = None;
    match shape {
        ErrorStreamShape::Terminal => {
            let pair: (OwnedFd, OwnedFd) = open_a_terminal_pair()?;
            master = Some(pair.0);
            command.stderr(Stdio::from(pair.1));
        }
        // What: `Stdio::piped()` gives the child a pipe whose reading end the `Child` value holds.
        // Why:   When this process's standard error is not a terminal, plain Git would draw nothing, and a pipe presents
        //        exactly that.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // command.stderr = 'pipe';
        // ```
        ErrorStreamShape::Pipe => {
            command.stderr(Stdio::piped());
        }
    }
    // What: A `RelayedChild` is a started child the signal relay points at until the token is dropped.
    // Why:  The token's own start publishes the child and hands it back still published.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relayed = guard.startRelayedNow(command);
    // ```
    let (mut child, relayed): (Child, Option<RelayedChild>) = match guard {
        Some(owning) => {
            let mut token: RelayedChild = owning.start_relayed_now(&mut command)?;
            // What: `Some(child)` is the present variant of `Option`; no trailing `;` would make it a return, and the token
            //       keeps pointing at the child while the read waits.
            // Why:  The read that ends the child must own it, and the relay must cover the whole wait.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const child = token.takeChild()!;
            // ```
            match token.take_child() {
                Some(taken) => (taken, Some(token)),
                None => {
                    return Err(io::Error::other("the relayed child was already taken"));
                }
            }
        }
        // What: `None` is no guard at all: a caller that has none (a control of the bytes alone) starts the child plainly.
        // Why:  The publication is one concern and the bytes another, and a control of the bytes must change no signal's
        //       disposition of its own process.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (await spawnRetrying(command), undefined)
        // ```
        None => (
            crate::start_retry::RetryStart::spawn_retrying(&mut command)?,
            None,
        ),
    };
    // What: `OwnedFd::from` takes the pipe's reading end out of the child value, which then holds no stream.
    // Why:  Both shapes are then read through one descriptor, and the child value is only the wait.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = master ?? child.stderr!;
    // ```
    let source: OwnedFd = match master {
        Some(descriptor) => descriptor,
        None => {
            let Some(piped) = child.stderr.take() else {
                return Err(io::Error::other(
                    "the child's error stream is not the wrapper's",
                ));
            };
            OwnedFd::from(piped)
        }
    };
    // The relay waits with `poll` and reads only what is there; a blocking read would hold this process past the child's end.
    make_non_blocking(source.as_fd())?;
    // What: `drop(command)` closes the wrapper's own handles on the child's streams, which is the terminal's slave here.
    // Why:  A slave this process still held would keep the master from reporting the child's end, so the read would wait a
    //        whole poll slice after Git ended (measured: about 94 ms on a typed ssh command at a terminal).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // command[Symbol.dispose]();
    // ```
    drop(command);
    // What: The three values together, as the present variant of the result.
    // Why:  The caller reads the stream and then ends the child.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { child, source, relayed };
    // ```
    return Ok(RelayedStart {
        child: Some(child),
        source: Some(source),
        relayed,
    });
}

/// What:
///  Read the child's error stream to the child's end, relaying every byte to this process's standard error.
/// Why:
///   The relay ends when the child does, after draining what is already readable,
///   so a process the child left holding the stream cannot hold the wrapper
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "The end rule").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { await pollOne(source, 100ms); drain(); const s = child.tryWait(); if (s) { drain(); return { outcome: s, kept }; } }
/// ```
pub fn read_to_the_child_s_end(
    start: RelayedStart,
    out: &mut dyn std::io::Write,
) -> io::Result<RelayedErrorStream> {
    // `mut` lets the wait take the child's status and the tail grow.
    // What: `mut start` lets the child and the token be taken out of the start, whose own drop then finds nothing to end.
    // Why:   The read is the one owner of the child.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const owned = start.takeChild()!;
    // ```
    let mut held: RelayedStart = start;
    let mut owned: Child = match held.child.take() {
        Some(taken) => taken,
        None => {
            return Err(io::Error::other("the relayed child was already taken"));
        }
    };
    // What: `take` moves the token out, so the start's drop retires nothing twice.
    // Why:   One owner points the relay away, and it is this function's token.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const token = start.takeToken();
    // ```
    let relayed: Option<RelayedChild> = held.relayed.take();
    // What: `source` is the descriptor the read takes from the start, which then owns no stream.
    // Why:   The relay reads exactly one descriptor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = start.takeSource();
    // ```
    let source: OwnedFd = match held.source.take() {
        Some(taken) => taken,
        None => {
            return Err(io::Error::other("the relayed descriptor was already taken"));
        }
    };
    // What: `map_or` answers zero when no guard was given, which is a caller that records no child.
    // Why:   The number is a fact about the token, and there may be none.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const childNumber = relayed?.childNumber() ?? 0;
    // ```
    let child_number: u32 = relayed.as_ref().map_or(0, RelayedChild::child_number);
    // What: `Vec::new()` is an empty growable list; the siblings are `&[u8]`, a borrowed view, and `[u8; N]`, a fixed array.
    // Why:  The kept tail grows as the child writes and is trimmed to its bound.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept: number[] = [];
    // ```
    let mut kept: Vec<u8> = Vec::new();
    let mut buffer: [u8; CHUNK_BYTES] = [0u8; CHUNK_BYTES];
    loop {
        // What: `PollFd` is one descriptor and the events asked of it; `PollFlags::IN` is "readable".
        // Why:  The relay waits for bytes without blocking this thread, so it can also look at the child.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // poll([{ fd: source, events: IN }], 100ms);
        // ```
        let mut watched: [rustix::event::PollFd<'_>; 1] = [rustix::event::PollFd::new(
            &source,
            rustix::event::PollFlags::IN,
        )];
        let _ready: rustix::io::Result<usize> =
            rustix::event::poll(&mut watched, Some(&POLL_SLICE));
        // What: The answer says whether the stream reached its end, which a writer's closing brings.
        // Why:  After that nothing more arrives, and the wait left is the child's own.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const eof = copyWhatIsReadable(source, buffer, kept);
        // ```
        let eof: bool = copy_what_is_readable(&source, &mut buffer, &mut kept, out)?;
        if eof {
            // One last look at whatever the stream still holds, and then the child's own wait: no process holds this wrapper
            //  past the child's end.
            drain_what_is_left(&source, &mut buffer, &mut kept, out);
            let status: std::process::ExitStatus = owned.wait()?;
            return finish(status, child_number, kept, relayed);
        }
        // What: `try_wait` asks whether the child has ended without waiting for it.
        // Why:  Once it has, the relay takes what is still in the stream and stops.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const status = child.tryWait(); if (status) { drain(); return { status, kept }; }
        // ```
        if let Some(status) = owned.try_wait()? {
            drain_what_is_left(&source, &mut buffer, &mut kept, out);
            return finish(status, child_number, kept, relayed);
        }
    }
}

/// What:
///  Turn the child's status and kept bytes into the answer, retiring the relay's descriptor once the child is reaped.
/// Why:
///   A descriptor left pointing at a reaped number must never take a signal meant for a later child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relayed[Symbol.dispose](); return { outcome: outcomeOf(status), childNumber, kept };
/// ```
fn finish(
    status: std::process::ExitStatus,
    child_number: u32,
    kept: Vec<u8>,
    relayed: Option<RelayedChild>,
) -> io::Result<RelayedErrorStream> {
    // The token goes here, after the child was reaped, so the relay stops pointing at a number the kernel may hand again.
    drop(relayed);
    // What: `outcome_of` turns the operating system's status into "exited with this code" or "ended by this signal".
    // Why:  The caller reports Git's end as plain Git's caller sees it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = status.code() !== null ? { exited: status.code() } : { signaled: status.signal() };
    // ```
    let outcome: ChildOutcome = crate::forwarding::outcome_of(status);
    // What: `Ok(..)` is the success variant of the result, carrying what the caller reads.
    // Why:  A non-zero Git status is Git's answer and not an error of this function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { outcome, childNumber, kept };
    // ```
    return Ok(RelayedErrorStream {
        outcome,
        child_number,
        kept,
    });
}

/// What:
///  End a child the read never took, so a panic between the start and the read leaves no Git running.
/// Why:
///  The wrapper dies with a panic sometimes, and a Git child it started would go on holding the repository's locks with nobody
///  left to reap it;
///  the token's own drop retires the relay first, so the kill reaches only this child.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose]() { if (this.child) { this.child.kill(); this.child.wait(); } }
/// ```
impl Drop for RelayedStart {
    /// What:
    ///  Kill and reap the child this start still owns, and point the relay away from it.
    /// Why:
    ///  A child nobody read is a child nobody will wait for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose]() { const child = this.child; if (!child) return; child.kill(); child.wait(); }
    /// ```
    fn drop(&mut self) {
        // What: `as_mut` borrows the child when the start still owns one, and answers nothing once it was taken.
        // Why:   A taken child is the read's to end, and this drop must not kill a child the read is waiting for.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const owned = this.child; if (owned) { owned.kill(); owned.wait(); }
        // ```
        let Some(owned) = self.child.as_mut() else {
            return;
        };
        let _killed: std::io::Result<()> = owned.kill();
        let _reaped: std::io::Result<std::process::ExitStatus> = owned.wait();
    }
}

/// The relay's readings are controlled in the test module.
#[cfg(test)]
#[path = "git_error_stream_relayed_as_it_arrives_tests.rs"]
mod tests;
