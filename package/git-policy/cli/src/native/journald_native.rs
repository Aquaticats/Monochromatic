//! What:
//!  The service's way into the journal: the native journal protocol over a datagram socket when the service's standard
//!  error is the journal, and a plain line on standard error otherwise.
//! Why:
//!  A line on a journal stream socket is stamped by journald when it processes it, so under load the times are seconds
//!  late and out of order with the service manager's own lines. A datagram on the native socket carries the kernel's
//!  time from the moment it was sent (measured in `doc/handover/cli-git-logging.md`), and the entry can carry a
//!  priority and fields a reader filters on (`journalctl CLI_GIT_REQUEST_ID=<id>`). The protocol is at
//!  <https://systemd.io/JOURNAL_NATIVE_PROTOCOL/>, and `$JOURNAL_STREAM` is described in `systemd.exec(5)`.
//!  No lock is taken: the socket is unconnected and non-blocking, one `sendto` per entry, and an entry that cannot be
//!  sent (journald restarting, a full queue) goes to standard error instead, so a commit never waits for the journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (sameFile(process.env.JOURNAL_STREAM, stderr)) socket.send(`PRIORITY=6\nMESSAGE=${line}\n`); else console.error(line);
//! ```

use std::io::Write;
use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

/// The socket journald reads native entries from.
pub const JOURNAL_SOCKET: &str = "/run/systemd/journal/socket";

/// Test-only: names another socket to send entries to, as a path or, after an `@`, as an abstract name (a fixture path
/// can be longer than a socket path may be). Detection by `$JOURNAL_STREAM` still applies, so a test points the
/// variable at the file it gave the process as standard error.
pub const TEST_SOCKET_VARIABLE: &str = "CLI_GIT_TEST_ONLY_JOURNAL_SOCKET";

/// The variable the service manager sets to the device and inode of the stream it connected to the journal.
pub const STREAM_VARIABLE: &str = "JOURNAL_STREAM";

/// The `SYSLOG_IDENTIFIER` of every entry, the name `journalctl` shows after the host.
pub const IDENTIFIER: &str = "cli-git-service";

/// The most characters of a message an entry carries; longer text is cut, so a datagram never exceeds what a socket takes.
pub const MESSAGE_LIMIT: usize = 16 * 1024;

/// The most characters of a field value an entry carries.
pub const FIELD_LIMIT: usize = 1024;

/// What is appended to a cut message.
const CUT_MARK: &str = " [cut]";

/// How serious an entry is, as journald's `PRIORITY` (syslog levels).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    /// Something the service needed failed; `journalctl --priority=err` shows it.
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
///  The text with every control character (a newline included) replaced by a space, and cut to `limit` characters.
/// Why:
///   One entry is one line whatever a client or an error message held: a newline in a message could otherwise start a
///  field of its own in the datagram, or a second line in the standard error stream.
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

/// Whether a field name is one journald accepts from a client: upper-case letters, digits and underscores, not starting
/// with an underscore (those are the journal's own) or a digit.
pub fn valid_field_name(name: &str) -> bool {
    let mut characters = name.chars();
    return characters.next().is_some_and(|first| { return first.is_ascii_uppercase(); })
        && characters.all(|character| { return character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'; });
}

/// What:
///  The bytes of one native entry: `PRIORITY`, `SYSLOG_IDENTIFIER`, `MESSAGE`, then each field as `NAME=value` and a newline.
/// Why:
///   Every value is a single line (`one_line`), so the simple form of the protocol is enough and no value can start a
///  field. A field whose name journald would refuse is left out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PRIORITY=${n}\nSYSLOG_IDENTIFIER=cli-git-service\nMESSAGE=${message}\n${fields.map(([k, v]) => `${k}=${v}\n`).join('')}`
/// ```
pub fn datagram(priority: Priority, message: &str, fields: &[(&str, String)]) -> Vec<u8> {
    let mut text: String = format!("PRIORITY={}\nSYSLOG_IDENTIFIER={IDENTIFIER}\nMESSAGE={}\n", priority.number(), one_line(message, MESSAGE_LIMIT));
    for (name, value) in fields {
        if valid_field_name(name) {
            text.push_str(&format!("{name}={}\n", one_line(value, FIELD_LIMIT)));
        }
    }
    return text.into_bytes();
}

/// Whether `$JOURNAL_STREAM` names the file with this device and inode, which is how `systemd.exec(5)` says to tell that
/// standard error is still the journal: the variable alone is not enough, because a redirection leaves it set.
pub fn stream_matches(variable: Option<&str>, device: u64, inode: u64) -> bool {
    let Some((named_device, named_inode)) = variable.and_then(|text| { return text.split_once(':'); }) else {
        return false;
    };
    return named_device.parse::<u64>().ok() == Some(device) && named_inode.parse::<u64>().ok() == Some(inode);
}

/// The device and inode of standard error, or nothing when it cannot be read.
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
///  The address as the C structure `sendto` takes, and its length, or nothing when the name does not fit.
/// Why:
///   A signal handler may not build it (the standard library allocates), so it is made once, ahead of the signal.
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
    // A path ends in a zero byte that `zeroed` already supplies; an abstract name starts with one.
    if offset + name.len() >= raw.sun_path.len() {
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
    /// As a line on standard error.
    Stderr,
}

/// Where the journal listens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// A socket file.
    Path(String),
    /// An abstract name, which a test uses.
    Abstract(String),
}

impl Target {
    /// The journal's own socket.
    pub fn journal() -> Self {
        return Self::Path(String::from(JOURNAL_SOCKET));
    }

    /// Reads a test-only setting: `@name` is abstract, anything else a path.
    pub fn parse(text: &str) -> Self {
        return text.strip_prefix('@').map_or_else(|| { return Self::Path(text.to_string()); }, |name| { return Self::Abstract(name.to_string()); });
    }

    /// The socket address, or nothing when the name does not fit one.
    pub fn address(&self) -> Option<SocketAddr> {
        return match self {
            Self::Path(path) => SocketAddr::from_pathname(path).ok(),
            Self::Abstract(name) => SocketAddr::from_abstract_name(name.as_bytes()).ok(),
        };
    }
}

/// Where the service's entries go, decided once.
#[derive(Debug)]
pub struct Sink {
    /// The unconnected, non-blocking socket, when standard error is the journal and a socket could be made.
    native: Option<UnixDatagram>,
    /// The journal's socket.
    address: Option<SocketAddr>,
    /// Whether standard error is the journal's stream, so that a line there may carry a `<priority>` prefix.
    stream: bool,
}

impl Sink {
    /// What:
    ///  The sink for a process whose `$JOURNAL_STREAM` is `variable`, whose standard error is the file `stderr`, and
    ///  whose journal socket is at `socket`.
    /// Why:
    ///   The native socket is used only when standard error is the journal; a terminal, a file or a pipe keeps its plain
    ///  lines, and a service run by hand reads as before.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Sink.choose(process.env.JOURNAL_STREAM, stderrIdentity(), '/run/systemd/journal/socket')
    /// ```
    pub fn choose(variable: Option<&str>, stderr: Option<(u64, u64)>, socket: &Target) -> Self {
        let stream: bool = stderr.is_some_and(|(device, inode)| { return stream_matches(variable, device, inode); });
        let native: Option<UnixDatagram> = if stream { Self::open() } else { None };
        return Self { native, address: socket.address(), stream };
    }

    /// An unconnected datagram socket that never blocks, or nothing.
    fn open() -> Option<UnixDatagram> {
        let socket: UnixDatagram = UnixDatagram::unbound().ok()?;
        socket.set_nonblocking(true).ok()?;
        return Some(socket);
    }

    /// The sink of this process: the environment's `$JOURNAL_STREAM`, the real standard error, and the real socket or the
    /// test-only one.
    pub fn from_environment() -> Self {
        let variable: Option<String> = std::env::var(STREAM_VARIABLE).ok();
        let socket: Target = std::env::var(TEST_SOCKET_VARIABLE).map_or_else(|_| { return Target::journal(); }, |text| { return Target::parse(text.as_str()); });
        return Self::choose(variable.as_deref(), stderr_identity(), &socket);
    }

    /// Whether this sink sends datagrams.
    pub fn is_native(&self) -> bool {
        return self.native.is_some();
    }

    /// The raw descriptor and socket address a signal handler sends its entry with, when this sink is native.
    pub fn signal_target(&self) -> Option<(std::os::fd::RawFd, &SocketAddr)> {
        use std::os::fd::AsRawFd;
        let socket: &UnixDatagram = self.native.as_ref()?;
        return self.address.as_ref().map(|address| { return (socket.as_raw_fd(), address); });
    }

    /// What:
    ///  Sends one entry, and gives where it went: a datagram when the sink is native and the socket takes it, otherwise
    ///  `line` on `stderr`.
    /// Why:
    ///   A journal that is restarting, absent or full is no reason for the service to wait or to lose the entry. Standard
    ///  error is the journal's stream when the sink was chosen for it, and a `<priority>` prefix there keeps the level.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// try { socket.send(datagram) } catch { stderr.write(prefix + line) }
    /// ```
    pub fn deliver(&self, priority: Priority, message: &str, fields: &[(&str, String)], line: &str, stderr: &mut dyn Write) -> Delivery {
        if let Some(socket) = self.native.as_ref()
            && let Some(address) = self.address.as_ref()
            && socket.send_to_addr(datagram(priority, message, fields).as_slice(), address).is_ok()
        {
            return Delivery::Native;
        }
        let prefix: String = if self.stream { format!("<{}>", priority.number()) } else { String::new() };
        // The result is dropped on purpose: a closed standard error is no reason to fail the work the entry describes.
        let _written = stderr.write_all(format!("{prefix}{line}").as_bytes());
        return Delivery::Stderr;
    }
}

/// The datagram, the detection, the fallback.
#[cfg(test)]
#[path = "journald_native_tests.rs"]
mod tests;
