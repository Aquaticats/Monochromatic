//! What:
//!  The service's way into the journal:
//! the native journal protocol over a datagram socket when the service's standard
//!  error is the journal,
//! and a plain line on standard error otherwise.
//! Why:
//!  A line on a journal stream socket is stamped by journald when it processes it,
//! so under load the times are seconds
//!  late and out of order with the service manager's own lines.
//! A datagram on the native socket carries the kernel's
//!  time from the moment it was sent (measured in `doc/handover/cli-git-logging.md`),
//! and the entry can carry a
//!  priority and fields a reader filters on (`journalctl CLI_GIT_REQUEST_ID=<id>`).
//! The protocol is at
//!  <https://systemd.io/JOURNAL_NATIVE_PROTOCOL/>,
//! and `$JOURNAL_STREAM` is described in `systemd.exec(5)`.
//!  No lock is taken on the native path:
//! the socket is unconnected and non-blocking,
//! one `sendto` per entry,
//! and an entry
//!  that cannot be sent (journald restarting,
//! a full queue) goes to standard error instead,
//! so a commit never waits for the
//!  journal.
//! A value is carried as written,
//! one with a newline in the protocol's length-prefixed form.
//! On the fallback a line
//!  that finds no room is dropped and counted,
//! never waited for,
//! and a line the stream took only in part is finished before
//!  the next one starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (sameFile(process.env.JOURNAL_STREAM, stderr)) socket.send(`PRIORITY=6\nMESSAGE=${line}\n`); else console.error(line);
//! ```

pub use crate::journald_native_socket_target::Target;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

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

/// The `SYSLOG_IDENTIFIER` of every entry,
/// the name `journalctl` shows after the host.
pub const IDENTIFIER: &str = "monochromatic-git-policy-cli-service";

/// The `SYSLOG_IDENTIFIER` of an entry the wrapper itself sends,
/// as the command a person ran,
/// beside the service's own.
pub const CLIENT_IDENTIFIER: &str = "monochromatic-git-policy-cli";

/// The most characters of a message an entry carries;
/// longer text is cut.
/// The limits count characters,
/// not encoded bytes,
/// and bound no total,
/// so an entry made of very many or very wide fields can still exceed what a socket takes;
/// the send then
/// fails with `EMSGSIZE` and the entry goes to standard error,
/// as every entry that cannot be sent does.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const MESSAGE_LIMIT: usize = 16 * 1024;

/// The most characters of a field value an entry carries.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const FIELD_LIMIT: usize = 1024;

/// What is appended to a cut message.
const CUT_MARK: &str = " [cut]";

/// How serious an entry is,
/// as journald's `PRIORITY` (syslog levels).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    /// Something the service needed failed;
    /// `journalctl --priority=err` shows it.
    Error,
    /// Something did not go as intended and the service went on.
    Warning,
    /// An event of the service's ordinary life.
    Info,
}

impl Priority {
    /// The syslog number journald reads.
    pub fn number(self) -> u8 {
        return match self {
            Priority::Error => 3,
            Priority::Warning => 4,
            Priority::Info => 6,
        };
    }
}

/// What:
///  The text cut to `limit` characters,
/// with the mark appended when it was cut;
/// control characters and newlines are kept.
/// Why:
///   The native protocol carries any bytes (a value with a newline in the length-prefixed form),
/// so a value is cut and not
///  changed;
/// only the line on standard error needs `one_line`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.length > limit ? text.slice(0, limit) + ' [cut]' : text
/// ```
pub fn cut(text: &str, limit: usize) -> String {
    return match text.char_indices().nth(limit) {
        Some((end, _)) => format!("{}{CUT_MARK}", &text[..end]),
        None => text.to_string(),
    };
}

/// What:
///  The text with every control character (a newline included) replaced by a space,
/// and cut to `limit` characters.
/// Why:
///   A line on standard error is one line whatever a client or an error message held:
/// a newline in a message could
///  otherwise start a second line in the stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.replace(/[\x00-\x1f\x7f]/g, ' ').slice(0, limit)
/// ```
pub fn one_line(text: &str, limit: usize) -> String {
    let mut single: String = String::with_capacity(text.len().min(limit + CUT_MARK.len()));
    for (count, character) in text.chars().enumerate() {
        if count >= limit {
            single.push_str(CUT_MARK);
            break;
        }
        single.push(if character.is_control() { ' ' } else { character });
    }
    return single;
}

/// The longest field name journald takes,
/// in characters.
pub const FIELD_NAME_LIMIT: usize = 64;

/// What:
///  Whether a field name is one journald accepts from a client:
/// 1 to 64 upper-case letters,
/// digits and underscores,
/// not
///  starting with an underscore (the journal's own fields) or a digit.
/// Why:
///   This is systemd's `journal_field_valid` called with `allow_protected` false (`src/libsystemd/sd-journal/journal-file.c`,
///  read from the `main` branch;
/// its comments say names longer than 64 are refused and a digit may not come first),
/// which
///  `manager_process_entry` in `src/journal/journald-native.c` applies to both forms of a field.
/// A refused name is dropped by
///  journald without a word,
/// so the entry is built without it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[A-Z][A-Z0-9_]{0,63}$/.test(name)
/// ```
pub fn valid_field_name(name: &str) -> bool {
    let bytes: &[u8] = name.as_bytes();
    return bytes.len() <= FIELD_NAME_LIMIT
        && bytes.first().is_some_and(u8::is_ascii_uppercase)
        && bytes.iter().all(is_field_name_byte);
}

/// What:
///  Whether a byte may appear in a field name after the first:
/// an uppercase ASCII letter,
/// a digit or an underscore.
/// Why:
///  `valid_field_name` checks every byte,
/// as the journal's native protocol allows no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFieldNameByte = (byte: number) => /[A-Z0-9_]/.test(String.fromCharCode(byte));
/// ```
fn is_field_name_byte(byte: &u8) -> bool {
    return byte.is_ascii_uppercase() || byte.is_ascii_digit() || *byte == b'_';
}

/// Appends one field:
/// `NAME=value\n` when the value has no newline,
/// otherwise the length-prefixed form,
/// `NAME\n`,
/// the byte
/// length as eight little-endian bytes,
/// the value and `\n`.
fn push_field(entry: &mut Vec<u8>, name: &str, value: &str) {
    if value.contains('\n') {
        entry.extend_from_slice(name.as_bytes());
        entry.push(b'\n');
        entry.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
        entry.extend_from_slice(value.as_bytes());
    } else {
        entry.extend_from_slice(name.as_bytes());
        entry.push(b'=');
        entry.extend_from_slice(value.as_bytes());
    }
    entry.push(b'\n');
}

/// What:
///  The bytes of one native entry of the service:
/// see `datagram_as`.
/// Why:
///   Every entry of the service carries the service's identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// datagramAs('cli-git-service', priority, message, fields)
/// ```
pub fn datagram(priority: Priority, message: &str, fields: &[(&str, String)]) -> Vec<u8> {
    return datagram_as(IDENTIFIER, priority, message, fields);
}

/// What:
///  The bytes of one native entry:
/// `PRIORITY`,
/// `SYSLOG_IDENTIFIER`,
/// `MESSAGE`,
/// then each field,
/// each value as written (cut
///  to its limit),
/// in the length-prefixed form when it holds a newline.
/// Why:
///   A newline cannot be written in the simple `NAME=value` form without starting a field,
/// and the length-prefixed form
///  carries any bytes,
/// so a value reads back in the journal as it was sent.
/// A field whose name journald would refuse is left
///  out.
/// The identifier names the sender:
/// the service,
/// or the wrapper a person ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PRIORITY=${n}\nSYSLOG_IDENTIFIER=${identifier}\nMESSAGE=${message}\n${fields.map(([k, v]) => `${k}=${v}\n`).join('')}`
/// ```
pub fn datagram_as(identifier: &str, priority: Priority, message: &str, fields: &[(&str, String)]) -> Vec<u8> {
    let mut entry: Vec<u8> = Vec::new();
    push_field(&mut entry, "PRIORITY", priority.number().to_string().as_str());
    push_field(&mut entry, "SYSLOG_IDENTIFIER", identifier);
    push_field(&mut entry, "MESSAGE", cut(message, MESSAGE_LIMIT).as_str());
    for (name, value) in fields {
        if valid_field_name(name) {
            push_field(&mut entry, name, cut(value, FIELD_LIMIT).as_str());
        }
    }
    return entry;
}

/// Whether `$JOURNAL_STREAM` names the file with this device and inode,
/// which is how `systemd.exec(5)` says to tell that
/// standard error is still the journal:
/// the variable alone is not enough,
/// because a redirection leaves it set.
pub fn stream_matches(variable: Option<&str>, device: u64, inode: u64) -> bool {
    let Some(text) = variable else {
        return false;
    };
    let Some((named_device, named_inode)) = text.split_once(':') else {
        return false;
    };
    return named_device.parse::<u64>().ok() == Some(device) && named_inode.parse::<u64>().ok() == Some(inode);
}

/// The device and inode of standard error,
/// or nothing when it cannot be read.
pub fn stderr_identity() -> Option<(u64, u64)> {
    let mut status: libc::stat = unsafe { std::mem::zeroed() };
    // SAFETY: `status` is valid for the call to write one `stat`.
    let result: libc::c_int = unsafe { libc::fstat(libc::STDERR_FILENO, &raw mut status) };
    if result != 0 {
        return None;
    }
    return Some((u64::from(status.st_dev), u64::from(status.st_ino)));
}

/// What:
///  The address as the C structure `sendto` takes,
/// and its length,
/// or nothing when the name does not fit.
/// Why:
///   A signal handler may not build it (the standard library allocates),
/// so it is made once,
/// ahead of the signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rawAddress(address) // { family: AF_UNIX, path: '\0name' }
/// ```
pub fn raw_address(address: &SocketAddr) -> Option<(libc::sockaddr_un, libc::socklen_t)> {
    use std::os::unix::ffi::OsStrExt;
    // SAFETY: `sockaddr_un` is a plain C structure for which all zero bytes is a valid value.
    let mut raw: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    raw.sun_family = libc::sa_family_t::try_from(libc::AF_UNIX).ok()?;
    let (offset, name): (usize, &[u8]) = match (address.as_pathname(), address.as_abstract_name()) {
        (Some(path), _) => (0, path.as_os_str().as_bytes()),
        (None, Some(name)) => (1, name),
        (None, None) => return None,
    };
    // A path needs room for the zero byte that ends it; an abstract name starts with one, so it may fill the whole field.
    if offset + name.len() + usize::from(offset == 0) > raw.sun_path.len() {
        return None;
    }
    for (slot, byte) in raw.sun_path[offset..].iter_mut().zip(name) {
        *slot = libc::c_char::from_ne_bytes([*byte]);
    }
    let length: usize = std::mem::size_of::<libc::sa_family_t>() + offset + name.len() + usize::from(offset == 0);
    return Some((raw, libc::socklen_t::try_from(length).ok()?));
}

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
    lost: AtomicU64,
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
    pub fn choose(variable: Option<&str>, stderr: Option<(u64, u64)>, socket: &Target) -> Self {
        let stream: bool = match stderr {
            Some((device, inode)) => stream_matches(variable, device, inode),
            None => false,
        };
        let native: Option<UnixDatagram> = if stream { Self::open() } else { None };
        return Self { native, address: socket.address(), stream, pending: Mutex::new(Vec::new()), lost: AtomicU64::new(0) };
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
        // A test-only setting that is not set (or not text) leaves the journal's own socket.
        let socket: Target = match std::env::var(TEST_SOCKET_VARIABLE) {
            Ok(text) => Target::parse(text.as_str()),
            Err(_unset) => Target::journal(),
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
    pub fn deliver(&self, priority: Priority, message: &str, fields: &[(&str, String)], line: &str, stderr: &mut dyn Write) -> Delivery {
        let lost: u64 = self.lost.load(Ordering::Relaxed);
        if let Some(socket) = self.native.as_ref()
            && let Some(address) = self.address.as_ref()
        {
            let mut all: Vec<(&str, String)> = fields.to_vec();
            if lost > 0 {
                all.push((LOST_FIELD, lost.to_string()));
            }
            if socket.send_to_addr(datagram(priority, message, all.as_slice()).as_slice(), address).is_ok() {
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
    fn forgive(&self, count: u64) {
        let mut current: u64 = self.lost.load(Ordering::Relaxed);
        while let Err(seen) = self.lost.compare_exchange_weak(current, current.saturating_sub(count), Ordering::Relaxed, Ordering::Relaxed) {
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
    fn fall_back(&self, priority: Priority, line: &str, stderr: &mut dyn Write) -> Delivery {
        // Only the journal's stream carries a `<priority>` prefix; any other standard error gets its plain line, and the rest of
        //  it is kept for the next line like the stream's, because a writer that never waits may take a line only in part.
        let mut pending = self.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
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
            bytes.extend_from_slice(format!("<{}>{LOST_NOTICE_HEAD}{lost}{LOST_NOTICE_TAIL}\n", Priority::Warning.number()).as_bytes());
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
const LOST_NOTICE_TAIL: &str = " log entries were dropped before this one because the log stream had no room";

/// Writes as much of `bytes` as `stream` takes without waiting,
/// and gives how much it took.
fn write_what_fits(stream: &mut dyn Write, bytes: &[u8]) -> usize {
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

/// What:
///  A writer to the journal's stream (standard error as the service manager connected it) that never waits for room.
/// Why:
///   `O_NONBLOCK` on the datagram socket does not make standard error non-blocking,
/// and a journal that stops reading would
///  fill the stream and block the service,
/// and so the commits that wait for it.
/// Each write is `MSG_DONTWAIT` on a socket,
/// or,
///  on a pipe or file a test or a redirection put there,
/// a write made only after a poll shows room and of no more than a
///  pipe takes whole;
/// a write that finds no room is refused with `WouldBlock`,
/// and one that takes part of the bytes says how
///  many,
/// so the caller keeps the rest (`Sink::deliver` does) instead of the writer waiting for room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { socket.sendNonBlocking(line) } catch (error) { if (error.code === 'EAGAIN') report(error) }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct StreamWriter {
    /// The descriptor of the stream.
    descriptor: std::os::fd::RawFd,
}

/// The most a pipe takes in one write once a poll shows room:
/// a page,
/// which a pipe holding a free slot takes whole.
pub const PIPE_CHUNK: usize = 4096;

/// What:
///  Sends bytes to a descriptor without ever waiting for room,
/// and gives the count taken or the error number.
/// Why:
///   One rule for every write the service makes to standard error,
/// including the signal handler's,
/// which may call only
///  functions that `signal-safety(7)` lists (read from <https://man7.org/linux/man-pages/man7/signal-safety.7.html>):
/// `send`,
///  `poll` and `write` are,
/// and this uses nothing else,
/// allocates nothing and takes no lock.
/// A socket is written with
///  `MSG_DONTWAIT`.
/// A pipe or file a test or a redirection put there is written only after a poll shows room,
/// and in a piece
///  of at most a page,
/// which a pipe holding a free slot takes whole;
/// so the write cannot block.
/// No room is `EAGAIN`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(await pollWritable(fd, 0))) throw errno('EAGAIN'); return write(fd, bytes.slice(0, 4096));
/// ```
///
/// # Errors
/// The error number of the failed call,
/// `EAGAIN` when there is no room.
pub fn send_without_waiting(descriptor: std::os::fd::RawFd, bytes: &[u8]) -> Result<usize, libc::c_int> {
    // SAFETY: `bytes` is valid for its length for the call, and `send` is safe in a signal handler.
    let sent: isize = unsafe { libc::send(descriptor, bytes.as_ptr().cast(), bytes.len(), libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL) };
    if let Ok(count) = usize::try_from(sent) {
        return Ok(count);
    }
    // SAFETY: the location is the calling thread's own error number, valid for a read.
    let failure: libc::c_int = unsafe { *libc::__errno_location() };
    if failure != libc::ENOTSOCK {
        return Err(failure);
    }
    // Not a socket (a file or a pipe a test or a redirection put there): an ordinary write, made only when the descriptor
    //  shows room and small enough for a pipe to take whole, so it cannot block.
    let mut request: libc::pollfd = libc::pollfd { fd: descriptor, events: libc::POLLOUT, revents: 0 };
    // SAFETY: `request` is valid for the call, which reads and writes one `pollfd`, and `poll` is safe in a signal handler.
    let ready: libc::c_int = unsafe { libc::poll(&raw mut request, 1, 0) };
    if ready <= 0 || request.revents & libc::POLLOUT == 0 {
        return Err(libc::EAGAIN);
    }
    let chunk: &[u8] = &bytes[..bytes.len().min(PIPE_CHUNK)];
    // SAFETY: `chunk` is valid for its length for the call, and `write` is safe in a signal handler.
    let written: isize = unsafe { libc::write(descriptor, chunk.as_ptr().cast(), chunk.len()) };
    return match usize::try_from(written) {
        Ok(count) => Ok(count),
        Err(_negative) => {
            // SAFETY: the location is the calling thread's own error number, valid for a read.
            Err(unsafe { *libc::__errno_location() })
        }
    };
}

impl StreamWriter {
    /// A writer to standard error.
    pub fn stderr() -> Self {
        return Self { descriptor: libc::STDERR_FILENO };
    }

    /// A writer to a stream socket descriptor.
    pub fn on(descriptor: std::os::fd::RawFd) -> Self {
        return Self { descriptor };
    }
}

impl Write for StreamWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        return send_without_waiting(self.descriptor, bytes).map_err(std::io::Error::from_raw_os_error);
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The datagram,
/// the detection,
/// the fallback.
#[cfg(test)]
#[path = "journald_native_tests.rs"]
mod tests;
