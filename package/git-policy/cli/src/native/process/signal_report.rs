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
///  Whether the kernel attributes this report to a process,
///  and which process it names.
/// Why:
///   The handler may call only what the kernel allows,
///  so the sender has to come from the report it is handed;
///  this reads the report once for both answers,
///  and the record of a stopped commit names that process
///  (`service_client_interrupted.rs`,
///  `sender`).
///  The two facts are read with the platform's own accessors,
///  never through a structure of this crate's:
///  a hand-written view of `siginfo_t` has to guess where the union starts,
///  and a view one word off states no sender for every stop while agreeing with any report this crate
///  builds itself
///  (measured on this host with a report the kernel wrote:
///  the hand-written view read the padding and named no process,
///  `signal_report_tests.rs`).
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
    // SAFETY: with `SA_SIGINFO` the kernel passes a valid `siginfo_t`; `si_code` is a plain read of it.
    let code: c_int = unsafe { (*info).si_code };
    if !is_process_sender(code) {
        return (false, None);
    }
    return (true, sender_in_report(info));
}

/// What:
///  The process the kernel names as a signal's sender,
///  where it names one.
/// Why:
///  `si_pid` is defined for the signals a process sends (`kill`,
/// `sigqueue`,
/// `tkill`);
///  a terminal's interrupt,
/// a kernel fault signal and a timer carry no sender,
///  and the kernel's own definition of who sent it is the only honest answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// senderInReport(info) // info ? (info.fromProcess ? info.pid : undefined) : undefined
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn sender_in_report(info: *mut libc::siginfo_t) -> Option<i32> {
    if info.is_null() {
        return None;
    }
    // SAFETY: the same `siginfo_t`; `si_pid` is the sender's number inside it.
    let pid: i32 = unsafe { (*info).si_pid() };
    if pid <= 0 {
        return None;
    }
    return Some(pid);
}

/// The systems whose kernel names a signal's sender,
/// and the systems this one is not:
/// without the accessor the record says the signal and no sender.
#[cfg(not(target_os = "linux"))]
pub(crate) fn sender_in_report(_info: *mut libc::siginfo_t) -> Option<i32> {
    return None;
}

/// The layout this file reads is controlled beside it.
#[cfg(all(unix, test))]
#[path = "signal_report_tests.rs"]
mod tests;
