//! What:
//!  The two raw descriptor calls a binary-level control needs:
//!  a pipe whose ends are closed on `exec`,
//!  and the path of a pseudo-terminal's slave side.
//! Why:
//!  A control that hands a child one of these needs it to behave the same on Linux and on the macOS
//!  the package also builds for,
//!  and each of the two has a different call on each of the two systems,
//!  so the pairs live together in a file of their own rather than beside the fixture helpers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [read, write] = closeOnExecPipe(); const slave = pseudoTerminalSlaveName(master);
//! ```

/// A pipe whose two ends are closed on `exec`:
/// `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`,
/// so there the flag is set on each end right after `pipe`.
#[cfg(target_os = "linux")]
pub fn close_on_exec_pipe() -> [libc::c_int; 2] {
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe2` fills the two integers it is given.
    let made: libc::c_int = unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) };
    assert_eq!(made, 0, "pipe2 failed: {}", std::io::Error::last_os_error());
    return ends;
}

/// A pipe whose two ends are closed on `exec`:
/// `[read end, write end]`.
/// Linux makes both ends close-on-exec in the one `pipe2` call;
///  macOS has no `pipe2`,
/// so there the flag is set on each end right after `pipe`.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn close_on_exec_pipe() -> [libc::c_int; 2] {
    let mut ends: [libc::c_int; 2] = [0, 0];
    // SAFETY: `pipe` fills the two integers it is given.
    let made: libc::c_int = unsafe { libc::pipe(ends.as_mut_ptr()) };
    assert_eq!(made, 0, "pipe failed: {}", std::io::Error::last_os_error());
    for end in ends {
        // SAFETY: `end` was just created here; `fcntl` with `F_SETFD` only changes that descriptor's flags.
        let flagged: libc::c_int = unsafe { libc::fcntl(end, libc::F_SETFD, libc::FD_CLOEXEC) };
        assert_eq!(
            flagged,
            0,
            "fcntl failed: {}",
            std::io::Error::last_os_error()
        );
    }
    return ends;
}

/// The path of the slave side of the pseudo-terminal whose master is `descriptor`.
/// Linux has `ptsname_r`;
///  macOS has none,
/// and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
#[cfg(target_os = "linux")]
pub fn pseudo_terminal_slave_name(descriptor: libc::c_int) -> std::ffi::CString {
    let mut buffer: [libc::c_char; 256] = [0; 256];
    // SAFETY: `ptsname_r` writes at most the buffer's length.
    let named: libc::c_int =
        unsafe { libc::ptsname_r(descriptor, buffer.as_mut_ptr(), buffer.len()) };
    assert_eq!(named, 0, "the slave has a name");
    // SAFETY: the call above wrote a NUL-terminated name into `buffer`.
    return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned();
}

/// The path of the slave side of the pseudo-terminal whose master is `descriptor`.
/// Linux has `ptsname_r`;
///  macOS has none,
/// and `TIOCPTYGNAME` returns the name into the caller's buffer just as safely.
#[cfg(all(unix, not(target_os = "linux")))]
pub fn pseudo_terminal_slave_name(descriptor: libc::c_int) -> std::ffi::CString {
    let mut buffer: [libc::c_char; 128] = [0; 128];
    // SAFETY: `TIOCPTYGNAME` writes a NUL-terminated name of at most 128 bytes into the buffer it is given.
    let named: libc::c_int = unsafe {
        libc::ioctl(
            descriptor,
            libc::TIOCPTYGNAME as libc::c_ulong,
            buffer.as_mut_ptr(),
        )
    };
    assert_eq!(named, 0, "the slave has a name");
    // SAFETY: the call above wrote a NUL-terminated name into `buffer`.
    return unsafe { std::ffi::CStr::from_ptr(buffer.as_ptr()) }.to_owned();
}
