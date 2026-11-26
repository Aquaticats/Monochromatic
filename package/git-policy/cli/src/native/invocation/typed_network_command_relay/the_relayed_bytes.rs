//! What:
//!  The bytes a relayed child writes to its standard error: read as they arrive, written to this process's standard error, and
//!  kept as a bounded tail for the reading after the wait.
//! Why:
//!  The person reads Git's own words while they are written, and the sentence after a refusal is chosen from the stream's end;
//!  the relay never waits for bytes it cannot see, so a process the child left holding the stream cannot hold the wrapper
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "The end rule").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // copyWhatIsReadable(fd, buf, kept): boolean; drainWhatIsLeft(fd, buf, kept); makeNonBlocking(fd)
//! ```

// What: `AsFd` is a borrow of one open descriptor, which `read` and the flag calls take.
// Why:  Every call below borrows the descriptor without taking it away.
//
// In TS you'd write (pseudocode):
// ```ts
// type AsFd = { fd: number };
// ```
// What: `BorrowedFd` is that borrow as a value; its sibling `OwnedFd` closes its own descriptor.
// Why:  The non-blocking flag call names one explicitly.
//
// In TS you'd write (pseudocode):
// ```ts
// type BorrowedFd = number; // not closed by the holder
// ```
use std::os::fd::BorrowedFd;
// What: `OwnedFd` is one open descriptor whose closing is its value's own doing.
// Why:  The reads take the relay's end, which the caller owns.
//
// In TS you'd write (pseudocode):
// ```ts
// type OwnedFd = number; // closed when the value is dropped
// ```
use std::os::fd::OwnedFd;
// What: `io` is the input and output error type every read here answers with.
use std::io;
// What: `Write` adds `.write_all(..)` and `.flush()` to a stream.
use std::io::Write;
// What: The bound the tail keeps and the chunk one read takes, which the wait above also uses.
// Why:  One constant decides each, so the two halves cannot disagree about the size.
//
// In TS you'd write (pseudocode):
// ```ts
// import { KEPT_TAIL_BYTES } from './git-error-stream-relayed-as-it-arrives';
// ```
use super::git_error_stream_relayed_as_it_arrives::KEPT_TAIL_BYTES;

/// What:
///  Read everything the child's error stream has now, write it to this process's standard error, and keep its tail.
/// Why:
///   One pass per readiness, so a progress update written in one go reaches the person whole.
///   The answer says whether the stream reached its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (;;) { const n = read(fd, buf); if (n === 0) return true; if (n < 0 && errno === EAGAIN) return false; write(2, buf.subarray(0, n)); keep(...); }
/// ```
pub(super) fn copy_what_is_readable(source: &OwnedFd, buffer: &mut [u8], kept: &mut Vec<u8>, out: &mut dyn Write) -> io::Result<bool> {
    loop {
        // What: `read` takes the descriptor and a buffer, and answers how many bytes it wrote there.
        // Why:  The bytes are relayed exactly as the child wrote them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const n = rustix.read(source, buffer);
        // ```
        match rustix::io::read(source, &mut *buffer) {
            // What: `Ok(0)` is end of file: every writer closed its end.
            // Why:  Nothing more arrives, so the caller stops waiting for readiness.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return true;
            // ```
            Ok(0) => return Ok(true),
            Ok(read) => {
                relay_one_chunk(&buffer[..read], kept, out);
            }
            // What: `AGAIN` means "nothing to read now", not a failure.
            // Why:  A non-blocking descriptor answers this at every point where the child wrote nothing.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return false;
            // ```
            Err(rustix::io::Errno::AGAIN) => return Ok(false),
            // A signal interrupted the read; the loop reads again.
            Err(rustix::io::Errno::INTR) => {}
            // A pseudo-terminal's master reports `EIO` once every process holding its slave is gone, which is its end of file.
            Err(_gone) => return Ok(true),
        }
    }
}

/// What:
///  Take whatever the stream still holds once the child ended, without waiting for more.
/// Why:
///   The design's end rule: the relay ends when the child does, after draining what is already readable,
///   so a process the child left holding the stream cannot hold the wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let turn = 0; turn < 16; turn += 1) { const n = readNonBlocking(); if (n <= 0) return; relay(buf.subarray(0, n)); }
/// ```
pub(super) fn drain_what_is_left(source: &OwnedFd, buffer: &mut [u8], kept: &mut Vec<u8>, out: &mut dyn Write) {
    // Sixteen turns each taking `CHUNK_BYTES` is a hundred and twenty-eight kilobytes, and a stream that holds more than that
    //  after the child ended is one this relay stops reading rather than loop on.
    for _turn in 0..16 {
        match rustix::io::read(source, &mut *buffer) {
            Ok(0) => return,
            Ok(read) => {
                relay_one_chunk(&buffer[..read], kept, out);
            }
            // Nothing more is there, or the stream is gone: either way the drain is over.
            Err(_nothing_more) => return,
        }
    }
}

/// What:
///  Write one chunk to this process's standard error and keep its last `KEPT_TAIL_BYTES` bytes.
/// Why:
///   The person reads the bytes as Git writes them, and the reading after the wait needs the stream's end.
///   A write that fails (a closed standard error) keeps the bytes away from the person but never from the reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { write(2, chunk); } catch {} kept = Buffer.concat([kept, chunk]).subarray(-65536);
/// ```
fn relay_one_chunk(chunk: &[u8], kept: &mut Vec<u8>, out: &mut dyn Write) {
    // What: `out` is the stream the caller relays to: this process's standard error in production, and a control's own list
    //       when the relay is measured.
    // Why:   One writer makes the relay testable without writing a hundred kilobytes into the run's log.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // out.write(chunk); out.flush();
    // ```
    let _written: io::Result<()> = out.write_all(chunk);
    let _flushed: io::Result<()> = out.flush();
    kept.extend_from_slice(chunk);
    if kept.len() > KEPT_TAIL_BYTES {
        let drop_this_many: usize = kept.len().saturating_sub(KEPT_TAIL_BYTES);
        kept.drain(..drop_this_many);
    }
}

/// What:
///  Make a descriptor non-blocking, so the relay never waits for bytes it cannot see.
/// Why:
///   The relay waits with `poll` and reads only what is there;
///   a blocking read would hold this process past the child's end whenever a process the child started keeps the stream open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fcntl(fd, F_SETFL, fcntl(fd, F_GETFL) | O_NONBLOCK);
/// ```
pub fn make_non_blocking(descriptor: BorrowedFd<'_>) -> std::io::Result<()> {
    // What: `fcntl_getfl` reads the descriptor's flags and `fcntl_setfl` writes the ones that may change.
    // Why:  `O_NONBLOCK` is set on top of whatever the descriptor already had.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const flags = fcntlGetfl(fd); fcntlSetfl(fd, flags | O_NONBLOCK);
    // ```
    let flags: rustix::fs::OFlags = rustix::fs::fcntl_getfl(descriptor).map_err(std::io::Error::from)?;
    // What: `BorrowedFd` is a copyable borrow, so the second call takes the same descriptor the first one read.
    // Why:  The descriptor stays open here; only its flags change.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fcntlSetfl(fd, flags | O_NONBLOCK);
    // ```
    return rustix::fs::fcntl_setfl(descriptor, flags | rustix::fs::OFlags::NONBLOCK).map_err(std::io::Error::from);
}
