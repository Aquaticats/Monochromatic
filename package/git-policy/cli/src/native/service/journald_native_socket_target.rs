//! What:
//!  The address the native journal entries are sent to:
//! the journal's own socket,
//! or a path or abstract name set by a test.
//! Why:
//!  `journald_native.rs` sends datagrams to it;
//! naming the address needs no socket,
//! so it is a file `journald_native.rs` uses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target = Target.journal(); // or Target.parse('@name') in a test
//! ```

use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::SocketAddr;
/// The socket journald reads native entries from.
pub const JOURNAL_SOCKET: &str = "/run/systemd/journal/socket";

/// Where the journal listens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// A socket file.
    Path(String),
    /// An abstract name,
    /// which a test uses.
    Abstract(String),
}

impl Target {
    /// The journal's own socket.
    pub fn journal() -> Self {
        return Self::Path(String::from(JOURNAL_SOCKET));
    }

    /// Reads a test-only setting:
    /// `@name` is abstract,
    /// anything else a path.
    pub fn parse(text: &str) -> Self {
        return match text.strip_prefix('@') {
            Some(name) => Self::Abstract(name.to_string()),
            None => Self::Path(text.to_string()),
        };
    }

    /// The socket address,
    /// or nothing when the name does not fit one.
    pub fn address(&self) -> Option<SocketAddr> {
        return match self {
            Self::Path(path) => SocketAddr::from_pathname(path).ok(),
            Self::Abstract(name) => SocketAddr::from_abstract_name(name.as_bytes()).ok(),
        };
    }
}
