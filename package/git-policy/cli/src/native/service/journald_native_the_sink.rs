//! What:
//!  Where the service's entries go,
//!  decided once:
//!  as a datagram on the journal's native socket,
//!  as a line on standard error,
//!  or nowhere,
//!  with the count of the entries a full stream dropped.
//! Why:
//!   A line on a journal stream socket is stamped by journald when it processes it,
//!  so under load the times are seconds late and out of order with the service manager's own lines;
//!  a datagram on the native socket carries the kernel's time from the moment it was sent,
//!  and no lock is taken on that path:
//!  one `sendto` per entry,
//!  and an entry that cannot be sent goes to standard error instead,
//!  so a commit never waits for the journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (sameFile(process.env.JOURNAL_STREAM, stderr)) socket.send(`PRIORITY=6\nMESSAGE=${line}\n`); else console.error(line);
//! ```

use crate::journald_native_socket_target::Target;
use crate::journald_native_the_fields_of_one_entry::datagram;
use crate::journald_native_the_words_of_one_entry::Priority;
use crate::journald_native_where_an_entry_goes::{
    stderr_identity,
    stream_matches,
};
use std::io::Write;
use std::os::unix::net::{
    SocketAddr,
    UnixDatagram,
};
use std::sync::Mutex;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};
/// Test-only:
/// names another socket to send entries to,
/// as a path or,
/// after an `@`,
/// as an abstract name (a fixture path
/// can be longer than a socket path may be).
/// Detection by `$JOURNAL_STREAM` still applies,
/// so a test points the
/// variable at the file it gave the process as standard error.
pub const TEST_SOCKET_VARIABLE: &str = "CLI_GIT_TEST_ONLY_JOURNAL_SOCKET";

/// The variable the service manager sets to the device and inode of the stream it connected to the journal.
pub const STREAM_VARIABLE: &str = "JOURNAL_STREAM";

/// Where an entry went.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// As a datagram on the native socket.
    Native,
    /// As a line on standard error (all of it,
    /// or the start of it with the rest kept to finish before the next line).
    Stderr,
    /// Nowhere:
    /// the journal's socket refused it and standard error had no room,
    /// so the entry was dropped and counted.
    Dropped,
}

/// Where the service's entries go,
/// decided once.
#[derive(Debug)]
pub struct Sink {
    /// The unconnected,
    /// non-blocking socket,
    /// when standard error is the journal and a socket could be made.
    native: Option<UnixDatagram>,
    /// The journal's socket.
    address: Option<SocketAddr>,
    /// Whether standard error is the journal's stream,
    /// so that a line there may carry a `<priority>` prefix.
    stream: bool,
    /// The rest of a line the stream took only in part,
    /// to be written before any other line.
    ///
    /// Lock kept:
    /// every thread of the service logs,
    /// the journal's stream is a byte stream that may take a line in parts,
    /// and
    /// two lines must never run together,
    /// so the unfinished rest of a line and the write that finishes it are one critical
    /// section.
    /// The section is one non-blocking write of what fits (the stream is never waited for),
    /// so a thread that finds
    /// the lock held waits for one system call and never for the journal.
    /// A design without the lock needs a thread and a
    /// queue for every entry,
    /// or a rule that drops an entry whenever another thread is mid-line;
    /// both give up the guarantee
    /// that an entry is lost only when the stream is full.
    /// The lock covers one sink's pending bytes,
    /// the smallest unit there is.
    pending: Mutex<Vec<u8>>,
    /// How many entries were dropped since one last said so.
    pub(crate) lost: AtomicU64,
}

impl Sink {
    /// What:
    ///  The sink for a process whose `$JOURNAL_STREAM` is `variable`,
    /// whose standard error is the file `stderr`,
    /// and
    ///  whose journal socket is at `socket`.
    /// Why:
    ///   The native socket is used only when standard error is the journal;
    /// a terminal,
    /// a file or a pipe keeps its plain
    ///  lines,
    /// and a service run by hand reads as before.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Sink.choose(process.env.JOURNAL_STREAM, stderrIdentity(), '/run/systemd/journal/socket')
    /// ```
    pub fn choose(
        variable: Option<&str>,
        stderr: Option<(u64, u64)>,
        socket: &Target,
    ) -> Self {
        let stream: bool = match stderr {
            Some((device, inode)) => stream_matches(variable, device, inode),
            None => false,
        };
        let native: Option<UnixDatagram> = if stream { Self::open() } else { None };
        return Self {
            native,
            address: socket.address(),
            stream,
            pending: Mutex::new(Vec::new()),
            lost: AtomicU64::new(0),
        };
    }

    /// An unconnected datagram socket that never blocks,
    /// or nothing.
    fn open() -> Option<UnixDatagram> {
        let socket: UnixDatagram = UnixDatagram::unbound().ok()?;
        socket.set_nonblocking(true).ok()?;
        return Some(socket);
    }

    /// The sink of this process:
    /// the environment's `$JOURNAL_STREAM`,
    /// the real standard error,
    /// and the real socket or the
    /// test-only one.
    pub fn from_environment() -> Self {
        let variable: Option<String> = std::env::var(STREAM_VARIABLE).ok();
        // A test-only setting that is not set (or not text) leaves the journal's own socket; no release build reads it.
        let socket: Target = match crate::test_only_switches::text(TEST_SOCKET_VARIABLE) {
            Some(text) => Target::parse(text.as_str()),
            None => Target::journal(),
        };
        return Self::choose(variable.as_deref(), stderr_identity(), &socket);
    }

    /// Whether standard error is the journal's stream,
    /// so that a write there may carry a `<priority>` prefix and must not wait.
    pub fn on_stream(&self) -> bool {
        return self.stream;
    }

    /// Whether this sink sends datagrams.
    pub fn is_native(&self) -> bool {
        return self.native.is_some();
    }

    /// The raw descriptor and socket address a signal handler sends its entry with,
    /// when this sink is native.
    pub fn signal_target(&self) -> Option<(std::os::fd::RawFd, &SocketAddr)> {
        use std::os::fd::AsRawFd;
        let socket: &UnixDatagram = self.native.as_ref()?;
        let address: &SocketAddr = self.address.as_ref()?;
        return Some((socket.as_raw_fd(), address));
    }

    /// What:
    ///  Sends one entry,
    /// and gives where it went:
    /// a datagram when the sink is native and the socket takes it,
    /// otherwise
    ///  `line` on `stderr`,
    /// otherwise nowhere.
    /// Why:
    ///   A journal that is restarting,
    /// absent or full is no reason for the service to wait or to lose the entry,
    /// and standard
    ///  error that has no room is no reason to wait either:
    /// that entry is dropped and counted,
    /// and the next one that gets
    ///  through says how many were.
    /// On the journal's stream a `<priority>` prefix keeps the level,
    /// and a line taken in part is
    ///  finished before another starts,
    /// so two lines never run together.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { socket.send(datagram) } catch { stderr.write(prefix + line) ?? count(dropped) }
    /// ```
    pub fn deliver(
        &self,
        priority: Priority,
        message: &str,
        fields: &[(&str, String)],
        line: &str,
        stderr: &mut dyn Write,
    ) -> Delivery {
        let lost: u64 = self.lost.load(Ordering::Relaxed);
        if let Some(socket) = self.native.as_ref()
            && let Some(address) = self.address.as_ref()
        {
            let mut all: Vec<(&str, String)> = fields.to_vec();
            if lost > 0 {
                all.push((LOST_FIELD, lost.to_string()));
            }
            if socket
                .send_to_addr(
                    datagram(priority, message, all.as_slice()).as_slice(),
                    address,
                )
                .is_ok()
            {
                self.forgive(lost);
                return Delivery::Native;
            }
        }
        return self.fall_back(priority, line, stderr);
    }

    /// Takes `count` off the dropped-entry count without going below zero.
    ///
    /// Two threads can each have said the same count (one on the native path,
    /// one on the stream),
    /// so the second
    /// subtraction must find less than it expects and stop at zero instead of wrapping.
    pub(crate) fn forgive(
        &self,
        count: u64,
    ) {
        let mut current: u64 = self.lost.load(Ordering::Relaxed);
        while let Err(seen) = self.lost.compare_exchange_weak(
            current,
            current.saturating_sub(count),
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            current = seen;
        }
    }

    /// What:
    ///  Writes the line to `stderr` without waiting:
    /// finishes what an earlier line left unwritten first,
    /// says how many
    ///  entries were dropped before this one,
    /// and keeps the rest of a line the stream takes only in part.
    /// Why:
    ///   Only the journal's stream needs this care;
    /// any other standard error gets its plain line as it always did.
    /// Entries
    ///  that find no room are counted (`lost`) and reported with the next that gets through,
    /// because a dropped entry nobody
    ///  hears of is a gap nobody can explain.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (pending.length) flush(pending); if (!pending.length) write(note + line) else lost++
    /// ```
    fn fall_back(
        &self,
        priority: Priority,
        line: &str,
        stderr: &mut dyn Write,
    ) -> Delivery {
        // Only the journal's stream carries a `<priority>` prefix; any other standard error gets its plain line, and the rest of
        //  it is kept for the next line like the stream's, because a writer that never waits may take a line only in part.
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !pending.is_empty() {
            let finished: usize = write_what_fits(stderr, pending.as_slice());
            pending.drain(..finished);
            if !pending.is_empty() {
                self.lost.fetch_add(1, Ordering::Relaxed);
                return Delivery::Dropped;
            }
        }
        let lost: u64 = self.lost.load(Ordering::Relaxed);
        let mut bytes: Vec<u8> = Vec::new();
        if lost > 0 {
            bytes.extend_from_slice(
                format!(
                    "<{}>{LOST_NOTICE_HEAD}{lost}{LOST_NOTICE_TAIL}\n",
                    Priority::Warning.number()
                )
                .as_bytes(),
            );
        }
        if self.stream {
            bytes.extend_from_slice(format!("<{}>{line}", priority.number()).as_bytes());
        } else {
            bytes.extend_from_slice(line.as_bytes());
        }
        let written: usize = write_what_fits(stderr, bytes.as_slice());
        if written == 0 {
            self.lost.fetch_add(1, Ordering::Relaxed);
            return Delivery::Dropped;
        }
        // The note, if any, is now in the stream or in `pending` ahead of the rest of the line, so it is not owed again.
        self.forgive(lost);
        pending.extend_from_slice(&bytes[written..]);
        return Delivery::Stderr;
    }
}

/// The field a native entry carries the count of earlier dropped entries in.
pub const LOST_FIELD: &str = "CLI_GIT_LOG_DROPPED";

/// What a line on standard error says before the count of entries dropped before it.
const LOST_NOTICE_HEAD: &str = "monochromatic-git-policy-cli-service: ";

/// What it says after the count.
const LOST_NOTICE_TAIL: &str =
    " log entries were dropped before this one because the log stream had no room";

/// Writes as much of `bytes` as `stream` takes without waiting,
/// and gives how much it took.
fn write_what_fits(
    stream: &mut dyn Write,
    bytes: &[u8],
) -> usize {
    let mut done: usize = 0;
    while done < bytes.len() {
        match stream.write(&bytes[done..]) {
            Ok(0) => break,
            Ok(count) => done += count,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => break,
        }
    }
    return done;
}
