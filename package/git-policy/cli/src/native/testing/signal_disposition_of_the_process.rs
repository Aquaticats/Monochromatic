//! What:
//!  Reads the action in place for a signal in this process,
//!  names the three actions a control compares against,
//!  and puts back the ignore a Rust program starts with.
//! Why:
//!  The disposition of a signal belongs to the whole process,
//!  so a control that asks which action is in place asks about every thread of the run.
//!  Reading it changes nothing (`sigaction` with a null new action only answers),
//!  and a control whose subject is a call that changes one runs its body in a child of its own
//!  (`testing/process_wide_state_child_test_support.rs`).
//!  `std` sets `SIGPIPE` to its ignore at the start of every Rust program,
//!  and a start that cannot run its program leaves it at the default in the process that tried
//!  (`library/std/src/sys/process/unix/unix.rs:374`;
//!  `doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`, issue 6604).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const before = theDispositionOf(SIGPIPE);  // theWordFor(before) === "SIG_IGN"
//! ```

/// What:
///  `the_disposition_of(signal)` answers the action in place for `signal` in this process.
///  `libc::sigaction` is the C call that reads it,
///  and `std::ptr::null()` where the new action goes is what makes the call a read alone.
/// Why:
///  A control says what a call left in place by reading before it and after it,
///  and a read that changed the action would be the very defect the control is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theDispositionOf(signal: number): Sigaction { return sigaction(signal, null, out); }
/// ```
pub(crate) fn the_disposition_of(signal: libc::c_int) -> libc::sigaction {
    // What: `std::mem::zeroed()` is an all-zero `sigaction`, which the kernel overwrites whole.
    // Why:  The call writes the action into this variable, so its first value never matters.
    // SAFETY: every field of `sigaction` is an integer or a set of signals, for which all zeroes are valid.
    let mut in_place: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: a null new action only reads the action in place into `in_place`, and touches no other memory.
    let read: libc::c_int = unsafe { libc::sigaction(signal, std::ptr::null(), &mut in_place) };
    assert_eq!(
        read,
        0,
        "reading the action in place for signal {signal} failed: {}",
        std::io::Error::last_os_error()
    );
    return in_place;
}

/// What:
///  The word for `the_disposition`:
///  `"SIG_DFL"` for the signal's default action,
///  `"SIG_IGN"` for its ignore,
///  and `"a handler"` for a function this process installed.
///  Its siblings a reader might expect are the two other actions the kernel knows,
///  which are exactly the three this answers.
/// Why:
///  A control's failure should read as the action it found,
///  not as the numbers of a C struct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theWordFor(theDisposition: Sigaction): string { return theDisposition.sa_sigaction === SIG_DFL ? "SIG_DFL" : theDisposition.sa_sigaction === SIG_IGN ? "SIG_IGN" : "a handler"; }
/// ```
pub(crate) fn the_word_for(the_disposition: &libc::sigaction) -> &'static str {
    // What: `sa_sigaction` holds the action itself: one of the two numbers `SIG_DFL`/`SIG_IGN`, or a handler's address.
    // Why:  It is the field `sigaction` reads and writes, so it is the one a control compares.
    let action: libc::sighandler_t = the_disposition.sa_sigaction;
    if action == libc::SIG_DFL {
        return "SIG_DFL";
    }
    if action == libc::SIG_IGN {
        return "SIG_IGN";
    }
    return "a handler";
}

/// What:
///  Puts the ignore a Rust program starts with back for `signal`,
///  which is the one direction a control may change: towards the state the program began in.
/// Why:
///  A child that read a call leaving a signal at its default is a process whose next write to a pipe
///  nobody reads would end it,
///  so the body puts the ignore back before it returns and asserts what it left.
/// Gotcha:
///  Only a child of a control calls this:
///  the action belongs to the whole process, and a test run holds its controls as threads of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putTheIgnoreBack(signal: number): void { signal(signal, SIG_IGN); }
/// ```
pub(crate) fn put_the_ignore_back(signal: libc::c_int) {
    // SAFETY: restoring the ignore takes one signal number and one action number and touches no memory; it is the
    //  direction a Rust program's own start already took.
    let previous: libc::sighandler_t = unsafe { libc::signal(signal, libc::SIG_IGN) };
    assert_ne!(
        previous,
        libc::SIG_ERR,
        "putting back the ignore for signal {signal} failed: {}",
        std::io::Error::last_os_error()
    );
}
