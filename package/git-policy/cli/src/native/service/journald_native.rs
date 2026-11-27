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
//! The parts of this way into the journal,
//! each a file of this folder,
//! in the order they build on each other:
//!
//! - `journald_native_the_bounds_of_one_entry.rs`:
//!   the most characters of a message,
//!   of a field value and of a field name.
//! - `journald_native_the_words_of_one_entry.rs`:
//!   how serious an entry is,
//!   and the text of one entry,
//!   cut and made one line.
//! - `journald_native_the_fields_of_one_entry.rs`:
//!   the fields of one entry,
//!   written as the protocol carries them.
//! - `journald_native_where_an_entry_goes.rs`:
//!   whether standard error is the journal,
//!   and the address a datagram is sent to.
//! - `journald_native_the_sink.rs`:
//!   where the entries of one process go,
//!   decided once.
//! - `journald_native_the_stream_writer.rs`:
//!   a write to the stream that never waits for room.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (sameFile(process.env.JOURNAL_STREAM, stderr)) socket.send(`PRIORITY=6\nMESSAGE=${line}\n`); else console.error(line);
//! ```

/// The address native journal entries are sent to.
pub use crate::journald_native_socket_target::Target;
/// The most characters of a message,
/// of a field value and of a field name.
pub use crate::journald_native_the_bounds_of_one_entry::{
    FIELD_LIMIT,
    FIELD_NAME_LIMIT,
    MESSAGE_LIMIT,
};
/// The sender's identifier,
/// the fields of one entry,
/// and the entry itself.
pub use crate::journald_native_the_fields_of_one_entry::{
    IDENTIFIER,
    datagram,
    datagram_as,
    valid_field_name,
};
/// Where the entries of one process go,
/// decided once,
/// and the count of the entries a full stream dropped.
pub use crate::journald_native_the_sink::{
    Delivery,
    LOST_FIELD,
    STREAM_VARIABLE,
    Sink,
    TEST_SOCKET_VARIABLE,
};
/// A write to the journal's stream that never waits for room.
pub use crate::journald_native_the_stream_writer::{
    PIPE_CHUNK,
    StreamWriter,
    send_without_waiting,
};
/// The mark a cut message ends with,
/// named here for the controls of this module.
#[cfg(test)]
pub(crate) use crate::journald_native_the_words_of_one_entry::CUT_MARK;
/// How serious an entry is,
/// and the text of one entry.
pub use crate::journald_native_the_words_of_one_entry::{
    Priority,
    cut,
    one_line,
};
/// Whether standard error is the journal,
/// and the C address a datagram is sent to.
pub use crate::journald_native_where_an_entry_goes::{
    raw_address,
    stderr_identity,
    stream_matches,
};

/// The `SYSLOG_IDENTIFIER` of an entry the wrapper itself sends,
/// as the command a person ran,
/// beside the service's own.
pub const CLIENT_IDENTIFIER: &str = "monochromatic-git-policy-cli";

/// The datagram,
/// the detection,
/// the fallback.
#[cfg(test)]
#[path = "journald_native_tests.rs"]
mod tests;
