//! What:
//!  What the kernel's own report of a signal says:
//!  whether a process sent it,
//!  and which process that was.
//! Why:
//!  The relay of the preparation and the wait of a commit both answer a signal,
//!  and both need the same two facts from the `siginfo_t` the kernel hands a handler:
//!  a terminal's interrupt carries a sender code no process sent,
//!  and a process's `SIGTERM` names the process,
//!  which is the one fact that tells a tool's bound from a person's Ctrl-C
//!  (`service_client_interrupted.rs`,
//! `sender`).
//!  It lives beside the relay rather than inside it because the relay is at the line bound the repository
//!  linter enforces (`rust/max-lines`),
//! and because the layout it reads is a fact of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [fromProcess, sender] = readReport(info);
//! ```

/// `c_int` is the C integer type signal numbers and sender codes use.
#[cfg(unix)]
use std::ffi::c_int;
/// Whether a sender code says a process sent the signal.
#[cfg(unix)]
use crate::signal_relay::is_process_sender;

/// What:
///  The first four fields of `siginfo_t` on every Unix system:
/// signal number,
/// error number,
/// sender code,
/// and the process that sent the signal.
/// Why:
///   The C struct's fields differ in name and union layout across systems;
/// these four share one layout,
///  and reading them through this view avoids the platform accessors.
///  The fourth word is `si_pid` in the union the sender codes this module relays through use (`kill`,
///  `sigqueue` and `tkill`),
/// which is what the record needs to name the process that stopped a commit
/// (`service_client_interrupted.rs`,
///  `sender`).
#[cfg(unix)]
#[repr(C)]
pub(crate) struct SignalHeader {
    /// Signal number.
    pub(crate) signal: c_int,
    /// Error number,
    /// unused here.
    pub(crate) error: c_int,
    /// Sender code,
    /// `SI_USER` for `kill` and so on.
    pub(crate) code: c_int,
    /// The process that sent the signal,
    /// in the union of the sender codes this module relays;
    /// zero where the kernel named none.
    pub(crate) sender: c_int,
}

/// What:
///  Whether the kernel attributes this report to a process,
///  and which process it names.
/// Why:
///   The handler may call only what the kernel allows,
///  so the sender has to come from the report it is handed;
///  this reads the report once for both answers,
///  and the record of a stopped commit names that process
///  (`service_client_interrupted.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [fromProcess, sender] = readReport(info);
/// ```
#[cfg(unix)]
pub(crate) fn read_report(info: *mut libc::siginfo_t) -> (bool, Option<i32>) {
    if info.is_null() {
        return (false, None);
    }
    // SAFETY: with `SA_SIGINFO` the kernel passes a valid `siginfo_t`, whose first four integers are the header read here.
    let header: SignalHeader = unsafe { std::ptr::read(info.cast::<SignalHeader>()) };
    if !is_process_sender(header.code) {
        return (false, None);
    }
    return (true, if header.sender > 0 { Some(header.sender) } else { None });
}

/// The layout this file reads is controlled beside it.
#[cfg(all(unix, test))]
#[path = "signal_report_tests.rs"]
mod tests;
