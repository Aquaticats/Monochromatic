//! What:
//!  One failed call of the probe:
//!  the call's name and the errno it answered.
//! Why:
//!   An arm's outcome is a call and an errno,
//!  and the record carries the errno's name,
//!  so the two travel together from the syscall to the status line and the
//!  record.
//!  A number this build names nowhere stays a number:
//!  inventing a word for it would make the record read as if the call answered
//!  something it did not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CallError = { call: string; errno: number };
//! ```

use super::probe_errno_names::errno_of;

/// What:
///  One call that failed:
///  which call,
///  and the errno it answered.
/// Why:
///   The record names the arm,
///  the step and the errno,
///  so the call and its number travel together from the syscall to the status
///  line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CallError = { call: string; errno: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallError {
    /// The call's own name,
    /// as the refusal line and the record use it.
    pub call: &'static str,
    /// The errno the call answered.
    pub errno: i32,
}

impl CallError {
    /// The errno's name,
    /// where this build names it.
    pub fn errno_name(&self) -> Option<&'static str> {
        return errno_of(self.errno);
    }
}

/// What one call's failure is,
/// read from the thread's own error the moment the call returned.
pub(super) fn failed(call: &'static str) -> CallError {
    return CallError { call, errno: std::io::Error::last_os_error().raw_os_error().unwrap_or(libc::EIO) };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_call_error_tests.rs"]
mod tests;
