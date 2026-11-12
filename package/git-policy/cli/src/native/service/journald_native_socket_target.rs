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

    /// What:
    ///  Whether something is there to receive an entry at this address.
    /// Why:
    ///   A service the wrapper starts has its output in its own file, not on the journal's stream,
    /// so the journal is asked for by this look at its socket rather than by the environment.
    /// An abstract name has no file to look at, and a test that names one knows it bound it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// target.exists() // Path -> existsSync(path), Abstract -> true
    /// ```
    pub fn exists(&self) -> bool {
        return match self {
            Self::Path(path) => std::path::Path::new(path).exists(),
            Self::Abstract(_name) => true,
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
