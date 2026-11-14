//! Controls for what the kernel's report of a signal says:
//! whether a process sent it,
//! and which process that was.
//! Why:
//!   The fourth word of the report is the sender,
//!   and the record of a stopped commit names it;
//!   the layout is stated here once,
//! against reports built in it.

use super::*;

/// What:
///  A report a process sent names that process,
///  and a terminal's names none.
/// Why:
///  The handler may call only what the kernel allows,
///  so the sender has to come from the report it is handed;
///  a terminal's interrupt and a tool's bound are the same signal number,
///  and only this fact tells them apart (`service_client_interrupted.rs`,
///  `sender`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readReport(report(SIGTERM, SI_USER, 4242))).toEqual([true, 4242]);
/// expect(readReport(report(SIGINT, SI_KERNEL, 0))).toEqual([false, undefined]);
/// ```
#[cfg(unix)]
#[test]
fn a_report_a_process_sent_names_that_process_and_a_terminals_names_none() {
    let mut from_process: libc::siginfo_t = report_of(libc::SIGTERM, libc::SI_USER, 4_242);
    assert_eq!(read_report(&raw mut from_process), (true, Some(4_242)), "a process's own signal names that process");
    // A terminal's interrupt carries a sender code that is no process's, and macOS reports no code at all for one, so
    //  only the systems that name the code say anything here.
    #[cfg(target_os = "linux")]
    {
        let mut from_the_terminal: libc::siginfo_t = report_of(libc::SIGINT, libc::SI_KERNEL, 0);
        assert_eq!(read_report(&raw mut from_the_terminal), (false, None), "a terminal's interrupt names no process");
    }
}

/// What:
///  A report the kernel named no sender in is a process's signal with no sender,
///  never a terminal's.
/// Why:
///  `kill` always names its sender,
///  so the two facts are read apart:
///  whether a process sent the signal decides relaying,
///  and the sender is what the record states.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readReport(report(SIGTERM, SI_USER, 0))).toEqual([true, undefined]);
/// ```
#[cfg(unix)]
#[test]
fn a_process_signal_with_no_sender_is_still_a_process_signal() {
    let mut unnamed: libc::siginfo_t = report_of(libc::SIGTERM, libc::SI_USER, 0);
    assert_eq!(read_report(&raw mut unnamed), (true, None), "the signal is a process's and names none");
    let nothing: *mut libc::siginfo_t = std::ptr::null_mut();
    assert_eq!(read_report(nothing), (false, None), "a handler given no report has nothing to read");
}

/// A kernel report of one signal,
/// in the layout every Unix shares:
/// the signal number,
/// the error number,
/// the sender code,
/// then the sender's process ID.
#[cfg(unix)]
fn report_of(signal: libc::c_int, code: libc::c_int, sender: libc::c_int) -> libc::siginfo_t {
    // SAFETY: all zero bytes are a valid `siginfo_t`, and the four integers written below are its first four words on
    //  every Unix this program builds for.
    let mut report: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let words: *mut libc::c_int = (&raw mut report).cast();
    // SAFETY: `words` points at the local report, whose first four integers are the words written here.
    unsafe {
        *words = signal;
        *words.add(1) = 0;
        *words.add(2) = code;
        *words.add(3) = sender;
    }
    return report;
}
