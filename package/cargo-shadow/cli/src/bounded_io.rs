//! What:
//! Pipe reading and writing that a caller can cancel.
//! Why:
//! Round 6 (fifth review 5a F2):
//! a reader blocked in `read_to_end` cannot be stopped,
//! so an error return left a thread and a
//! growing buffer behind.
//! Here the pipe is non-blocking and each worker looks at a cancel flag between attempts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function collect(stream: Readable, cancel: AbortSignal): Promise<Buffer>
//! export function feed(stream: Writable, bytes: Buffer, cancel: AbortSignal): Promise<void>
//! ```

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::os::fd::OwnedFd` for this file.
use std::os::fd::OwnedFd;

/// Imports `std::sync::Arc` for this file.
use std::sync::Arc;

/// Imports `std::sync::atomic::{AtomicBool, Ordering}` for this file.
use std::sync::atomic::{AtomicBool, Ordering};

/// Imports `std::sync::mpsc::{self, Receiver}` for this file.
use std::sync::mpsc::{self, Receiver};

/// Imports `std::thread` for this file.
use std::thread;

/// Imports `std::time::Duration` for this file.
use std::time::Duration;

/// Imports `rustix::fs::{OFlags, fcntl_setfl}` for this file.
use rustix::fs::{OFlags, fcntl_setfl};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// How long a worker waits before it asks a pipe again.
/// Delegated pick of round 6,
/// open to veto:
/// short against the 10 ms process polling of the runner,
/// so a cancelled worker ends
/// within one runner poll.
const RETRY_INTERVAL: Duration = Duration::from_millis(5);

/// The size of one read.
/// Delegated pick of round 6,
/// open to veto:
/// the default pipe capacity of Linux (64 KiB).
const CHUNK_BYTES: usize = 64 * 1024;

/// What:
/// `pub(crate) fn collect(stream: OwnedFd, cancel: Arc<AtomicBool>) -> Receiver<io::Result<Vec<u8>>>` reads a pipe to its end on its own thread.
/// Why:
/// The result is complete bytes,
/// or an error when the pipe failed or the run was cancelled first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(stream: Readable, cancel: AbortSignal): Promise<Buffer>
/// ```
pub(crate) fn collect(stream: OwnedFd, cancel: Arc<AtomicBool>) -> Receiver<io::Result<Vec<u8>>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let outcome: io::Result<Vec<u8>> = read_all(&stream, &cancel);
        let _ = sender.send(outcome);
    });
    return receiver;
}

/// What:
/// `fn read_all(stream: &OwnedFd, cancel: &AtomicBool) -> io::Result<Vec<u8>>` reads until end of file,
/// until an error,
/// or until cancelled.
/// Why:
/// Cancellation is looked at between reads,
/// which are non-blocking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readAll(stream: Readable, cancel: AbortSignal): Buffer
/// ```
fn read_all(stream: &OwnedFd, cancel: &AtomicBool) -> io::Result<Vec<u8>> {
    fcntl_setfl(stream, OFlags::NONBLOCK).map_err(io::Error::from)?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut chunk: Vec<u8> = vec![0; CHUNK_BYTES];
    loop {
        match rustix::io::read(stream, &mut chunk) {
            Ok(0) => return Ok(bytes),
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(Errno::INTR) => {}
            Err(Errno::AGAIN) => {
                if cancel.load(Ordering::Relaxed) {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "the read was cancelled"));
                }
                thread::sleep(RETRY_INTERVAL);
            }
            Err(error) => return Err(io::Error::from(error)),
        }
    }
}

/// What:
/// `pub(crate) fn feed(stream: OwnedFd, bytes: Vec<u8>, cancel: Arc<AtomicBool>) -> Receiver<io::Result<()>>` writes all of `bytes` on its own thread.
/// Why:
/// The result says whether every byte was written;
/// a closed pipe is `BrokenPipe`,
/// and a cancelled write is `Interrupted`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function feed(stream: Writable, bytes: Buffer, cancel: AbortSignal): Promise<void>
/// ```
pub(crate) fn feed(stream: OwnedFd, bytes: Vec<u8>, cancel: Arc<AtomicBool>) -> Receiver<io::Result<()>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let outcome: io::Result<()> = write_all(&stream, &bytes, &cancel);
        let _ = sender.send(outcome);
    });
    return receiver;
}

/// What:
/// `fn write_all(stream: &OwnedFd, bytes: &[u8], cancel: &AtomicBool) -> io::Result<()>` writes every byte,
/// or fails.
/// Why:
/// Cancellation is looked at between writes,
/// which are non-blocking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeAll(stream: Writable, bytes: Buffer, cancel: AbortSignal): void
/// ```
fn write_all(stream: &OwnedFd, bytes: &[u8], cancel: &AtomicBool) -> io::Result<()> {
    fcntl_setfl(stream, OFlags::NONBLOCK).map_err(io::Error::from)?;
    let mut written: usize = 0;
    while written < bytes.len() {
        match rustix::io::write(stream, &bytes[written..]) {
            Ok(count) => written += count,
            Err(Errno::INTR) => {}
            Err(Errno::AGAIN) => {
                if cancel.load(Ordering::Relaxed) {
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "the write was cancelled"));
                }
                thread::sleep(RETRY_INTERVAL);
            }
            Err(error) => return Err(io::Error::from(error)),
        }
    }
    return Ok(());
}
