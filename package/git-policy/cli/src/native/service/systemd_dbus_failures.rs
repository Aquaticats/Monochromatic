//! What:
//!  The failures of a bus exchange as the call's own error:
//! the time it was given running out,
//! a socket that refused or timed out,
//! a reply of another shape,
//! and a `zbus` failure read as the state the caller can act on.
//! Why:
//!  A caller tells a refusal of the manager,
//! a failed login,
//! a missing bus and a timeout apart,
//! and each has its own words for a person (`systemd_dbus.rs`).
//! The reading of a `zbus` error is here,
//! so the client that makes the calls holds only what it does with them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const callError = (error: ZbusError): CallError => ...; // one error per state
//! ```

use crate::systemd_dbus_wire_format::CallError;
use std::time::{
    Duration,
    Instant,
};

/// The time left of the exchange,
/// or an error when it ran out.
pub(crate) fn remaining(deadline: Instant) -> Result<Duration, CallError> {
    let left: Duration = deadline.saturating_duration_since(Instant::now());
    return if left.is_zero() {
        Err(CallError::TimedOut)
    } else {
        Ok(left)
    };
}

/// running out of the read or write time is a timeout,
/// anything else a broken connection.
fn socket_failure(error: &std::io::Error) -> CallError {
    return match error.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => CallError::TimedOut,
        _ => CallError::Connection(error.to_string()),
    };
}

/// A reply that did not have the expected shape,
/// as the call's `Malformed` error.
pub(crate) fn malformed(error: impl std::fmt::Display) -> CallError {
    return CallError::Malformed(error.to_string());
}

/// What:
///  A `zbus` failure as the call's error:
/// a method error keeps its name and text,
/// a handshake error is an authentication error,
/// and
///  running out of time on the socket is a timeout.
/// Why:
///  The callers tell a refusal of the manager,
/// a failed login,
/// a missing bus and a timeout apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const callError = (error: ZbusError): CallError => ...
/// ```
pub(crate) fn zbus_failure(error: zbus::Error) -> CallError {
    return match error {
        zbus::Error::MethodError(name, text, _message) => CallError::Manager {
            name: name.to_string(),
            text: text.unwrap_or_default(),
        },
        zbus::Error::InputOutput(io) => socket_failure(&io),
        zbus::Error::Handshake(text) => CallError::Authentication(text),
        zbus::Error::Address(text) => CallError::Connection(text),
        other if other.to_string().starts_with("failed to connect") => {
            CallError::Connection(other.to_string())
        }
        other => CallError::Malformed(other.to_string()),
    };
}
