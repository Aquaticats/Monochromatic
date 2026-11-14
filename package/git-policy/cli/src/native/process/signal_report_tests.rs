//! Controls for what the kernel's report of a signal says:
//! whether a process sent it,
//! and which process that was.
//! Why:
//!   The report's sender is the one fact that tells a tool's bound from a person's Ctrl-C,
//!   and the record of a stopped commit names it.
//!   A structure of this crate's own that names the wrong word of the report
//!   agrees with every report this crate builds
//!   and names no process for every report the kernel writes,
//!   so these controls read reports the kernel wrote.

use super::*;

/// The report the last delivery handed the capture handler,
/// and the signal it carried.
static CAPTURED: std::sync::Mutex<Option<(libc::c_int, Vec<u8>)>> = std::sync::Mutex::new(None);

/// The handler that keeps the report the kernel hands it,
/// so a control can read the layout the kernel really used.
extern "C" fn capture(signal: libc::c_int, info: *mut libc::siginfo_t, _context: *mut std::ffi::c_void) {
    if info.is_null() {
        return;
    }
    // SAFETY: with `SA_SIGINFO` the kernel passes a valid `siginfo_t` of `size_of::<siginfo_t>()` bytes, and this copies
    //  those bytes to the heap; a poisoned lock cannot be helped from a handler, so a failure to take it drops the report.
    let bytes: Vec<u8> = unsafe { std::slice::from_raw_parts(info.cast::<u8>(), std::mem::size_of::<libc::siginfo_t>()) }.to_vec();
    if let Ok(mut held) = CAPTURED.lock() {
        *held = Some((signal, bytes));
    }
}

/// What:
///  One report the kernel wrote,
///  by sending this thread `SIGTERM` under a handler that keeps what it is handed.
/// Why:
///  Every control of the report's layout starts from a report the kernel wrote,
///  never from one this crate built:
///  a built report agrees with its own reader whatever the layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = await kernelReportOf(SIGTERM); // the bytes the kernel delivered
/// ```
#[cfg(target_os = "linux")]
fn report_the_kernel_wrote() -> Vec<u8> {
    // SAFETY: an empty action and a handler that copies one report are what `sigaction` takes; what was in place is kept and
    //  put back before this control returns.
    let mut before: libc::sigaction = unsafe { std::mem::zeroed() };
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = capture as *const () as usize;
        action.sa_flags = libc::SA_SIGINFO;
        libc::sigemptyset(&mut action.sa_mask);
        assert_eq!(libc::sigaction(libc::SIGTERM, &action, &mut before), 0, "the capture handler is installed");
    }
    // SAFETY: `raise` sends one signal to this thread, whose handler copies the report and returns.
    let raised: libc::c_int = unsafe { libc::raise(libc::SIGTERM) };
    // SAFETY: the disposition read above is this process's own; putting it back touches no memory.
    let _restored = unsafe { libc::sigaction(libc::SIGTERM, &before, std::ptr::null_mut()) };
    assert_eq!(raised, 0, "the stop was sent");
    let captured: Option<(libc::c_int, Vec<u8>)> = CAPTURED.lock().expect("the captured report").take();
    let (signal, bytes): (libc::c_int, Vec<u8>) = captured.expect("the handler kept the report the kernel delivered");
    assert_eq!(signal, libc::SIGTERM, "the report is the signal that was sent");
    assert_eq!(bytes.len(), std::mem::size_of::<libc::siginfo_t>(), "the report is one `siginfo_t`");
    return bytes;
}

/// What:
///  Those bytes as a report the reader takes.
/// Why:
///  The reader is handed a `siginfo_t`,
///  and a control reads a report the kernel wrote by copying its bytes into one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = asReport(bytes);
/// ```
#[cfg(target_os = "linux")]
fn as_report(bytes: &[u8]) -> libc::siginfo_t {
    // SAFETY: all zero bytes are a valid `siginfo_t`, and `bytes` is exactly its size.
    let mut report: libc::siginfo_t = unsafe { std::mem::zeroed() };
    // SAFETY: `report` is the local `siginfo_t` above, and `bytes` is exactly its size.
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), (&raw mut report).cast::<u8>(), bytes.len()) };
    return report;
}

/// What:
///  The sender the report names sits where the report is read on this platform,
///  and the kernel's own accessor agrees.
/// Why:
///  A view of `siginfo_t` that names the wrong word states no sender for every stop the kernel sends,
///  while a control that builds its own report agrees with it either way:
///  this one asks the kernel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readReport(kernelReportOf(SIGTERM))).toEqual([true, process.pid]);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn the_kernel_puts_the_sender_where_the_report_reads_it() {
    let bytes: Vec<u8> = report_the_kernel_wrote();
    let mut report: libc::siginfo_t = as_report(bytes.as_slice());
    let mine: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    assert_eq!(read_report(&raw mut report), (true, Some(mine)), "the report names this process, where the report is read");
    // SAFETY: the same report, read through the platform's own accessor, which is the reading this one must agree with.
    assert_eq!(unsafe { report.si_pid() }, mine, "the kernel's own accessor names the same process");
}

/// What:
///  A report the kernel named no sender in is a process's signal with no sender,
///  never a terminal's.
/// Why:
///  `kill` always names its sender,
///  so the two facts are read apart:
///  whether a process sent the signal decides relaying,
///  and the sender is what the record states.
///  The sender is cleared where the kernel puts it,
///  which the bytes of a real report say and no guess of this crate's does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readReport(withoutTheSender(kernelReportOf(SIGTERM)))).toEqual([true, undefined]);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_process_signal_with_no_sender_is_still_a_process_signal() {
    let mut bytes: Vec<u8> = report_the_kernel_wrote();
    let mine: i32 = i32::try_from(std::process::id()).expect("a process ID fits");
    let needle: [u8; 4] = mine.to_ne_bytes();
    let found: Vec<usize> = bytes.windows(4).enumerate().filter(|(_, window)| *window == needle).map(|(at, _)| { return at; }).collect();
    assert_eq!(found.len(), 1, "the sender appears once in the report: {found:?}");
    let at: usize = found[0];
    bytes[at..at + 4].fill(0);
    let mut report: libc::siginfo_t = as_report(bytes.as_slice());
    assert_eq!(read_report(&raw mut report), (true, None), "the signal is a process's and names none");
    let nothing: *mut libc::siginfo_t = std::ptr::null_mut();
    assert_eq!(read_report(nothing), (false, None), "a handler given no report has nothing to read");
}

/// What:
///  A sender code no process sent names no process,
///  so the relay does not relay a terminal's signal and the record states no sender.
/// Why:
///  A terminal's interrupt, a kernel fault and a timer are not a process's bound,
///  and relaying one is what the relay's own rule forbids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(readReport(withCode(SI_KERNEL))).toEqual([false, undefined]);
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_sender_code_no_process_sent_names_no_process() {
    let bytes: Vec<u8> = report_the_kernel_wrote();
    let mut report: libc::siginfo_t = as_report(bytes.as_slice());
    report.si_code = libc::SI_KERNEL;
    assert_eq!(read_report(&raw mut report), (false, None), "a terminal's report is no process's");
}
