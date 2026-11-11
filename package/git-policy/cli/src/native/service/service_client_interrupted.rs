//! What:
//!  The signals that end a commit while it waits for the commit service,
//!  and the flag they set for the wait.
//! Why:
//!  A commit the person interrupts is not made:
//!  native Git makes none when it is interrupted,
//!  and the wrapper's promise for a commit that is not made is that nothing lands later by itself
//!  (`SPEC.md`, "The commit protocol").
//!  The request a commit already sent is the service's, so ending this process where the signal arrives would leave it
//!  to land with nobody to read the answer;
//!  the signal is noted instead,
//!  the wait asks at each step,
//!  the request is withdrawn,
//!  and the person is told what became of it.
//! A second signal ends the command at once:
//!  the first one already asked the service and started waiting for its answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchForAStop(); try { await submit() } finally { watch.release() }
//! ```

use std::sync::atomic::{AtomicI32, Ordering};

/// The signal the person sent,
/// and zero while none was seen:
/// an `AtomicI32` because a handler may only write one word.
static STOP_REQUESTED: AtomicI32 = AtomicI32::new(0);

/// The connection whose read is running,
/// and minus one while none is:
/// a handler may call only what the kernel allows,
/// and shutting this read down is how a stop that arrives on another thread reaches the wait.
static READING: AtomicI32 = AtomicI32::new(-1);

/// The signals a waiting commit answers by withdrawing its request:
/// the interrupt a terminal sends (Ctrl-C) and the one a stop sends.
/// A `SIGKILL` cannot be caught at all;
/// its request is dropped by the service when the connection closes
/// (`service_server_land_group.rs`).
const WATCHED: [std::os::raw::c_int; 2] = [libc::SIGINT, libc::SIGTERM];

/// What:
///  Notes the stop the person asked for,
///  or ends this process at once for a second one.
/// Why:
///   A signal handler may call only what the kernel allows,
///  so this writes one word and nothing else:
///  `signal`, `raise` and a lock-free atomic store are all on that list.
///  The second signal restores the default action first,
///  so the process ends the way the shell expects a Ctrl-C to end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.on(signal, () => { if (alreadyAsked) { signal(SIG_DFL); raise(signal) } else { asked = signal } });
/// ```
extern "C" fn note_the_stop(signal: std::os::raw::c_int) {
    if STOP_REQUESTED.swap(signal, Ordering::SeqCst) == 0 {
        // The read the wait is inside is woken here, because the kernel may deliver this signal to any thread of the process,
        //  and a thread that is not the waiting one only writes the flag the waiter reads after its own read gives up.
        let reading: i32 = READING.load(Ordering::SeqCst);
        if reading >= 0 {
            // SAFETY: the descriptor is the read end of the connection the wait declared it is inside, kept from the moment
            //  before that read to the moment after it; `shutdown` takes one number and touches no memory, and it is on the
            //  list of calls a handler may make.
            unsafe {
                libc::shutdown(reading, libc::SHUT_RD);
            }
        }
        return;
    }
    // SAFETY: `signal` and `SIG_DFL` are numbers, `raise` sends that number to this thread, and every call here is
    //  async-signal-safe; a second signal is the person asking for the command to end now.
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

/// The signal a waiting commit stops for,
/// and nothing while none was sent.
pub(crate) fn stop_requested() -> Option<i32> {
    let signal: i32 = STOP_REQUESTED.load(Ordering::SeqCst);
    return if signal == 0 { None } else { Some(signal) };
}

/// What:
///  The connection a stop must wake,
///  declared for as long as this value lives.
/// Why:
///   The descriptor is declared around the one blocking read and retracted before the connection can be closed and its number
///  reused,
///  so a stop that arrives after the read can never shut down a descriptor another part of this process now owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using awake = wakingTheReadOf(socket); const heard = await listen(reader);
/// ```
pub(crate) struct ReadAwake;

/// What:
///  Declares `stream` as the connection a stop wakes.
/// Why:
///  The wait reads one line at a time and calls this before each read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wakingTheReadOf(socket) // the value's drop retracts it
/// ```
pub(crate) fn waking_the_read_of(stream: &std::os::unix::net::UnixStream) -> ReadAwake {
    use std::os::unix::io::AsRawFd as _;
    READING.store(stream.as_raw_fd(), Ordering::SeqCst);
    return ReadAwake;
}

impl Drop for ReadAwake {
    fn drop(&mut self) {
        READING.store(-1, Ordering::SeqCst);
    }
}

/// What:
///  The signals of a waiting commit,
///  watched from the making of this value until it is dropped.
/// Why:
///   The watch covers the wait for the service's answer and nothing else,
///  so a Ctrl-C before the commit exists,
///  or after it landed,
///  keeps the ending it has.
///  The action that was in place before is put back when the wait ends,
///  so one command's watch is never another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = watchForAStop(); ... watch.release();
/// ```
pub(crate) struct StopWatch {
    /// The actions that were in place before this watch, in the order of `WATCHED`.
    previous: Vec<(std::os::raw::c_int, libc::sigaction)>,
}

/// What:
///  Watches the signals a waiting commit answers,
///  for as long as the value lives.
/// Why:
///  A `SIG_IGN` in place before (a shell that ignored the interrupt) is replaced so the commit is told,
///  and put back when the value is dropped.
///  A signal the process started with blocked stays pending and no handler runs,
///  and a harness that blocks the interrupt before it runs the wrapper would then make a Ctrl-C wait for the whole bound:
///  the two signals are unblocked for the thread that waits,
///  and the mask stays as the wait leaves it because the command ends after it.
///  The kernel may deliver the signal to any thread of the process,
///  so the handler wakes the read itself (`READING`) instead of relying on the interrupt to land in the waiting one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchForAStop() // the release is the value's drop
/// ```
pub(crate) fn watch_for_a_stop() -> StopWatch {
    STOP_REQUESTED.store(0, Ordering::SeqCst);
    let mut previous: Vec<(std::os::raw::c_int, libc::sigaction)> = Vec::new();
    for signal in WATCHED {
        // SAFETY: an empty mask and a handler that writes one word, shuts a descriptor down and raises a number are what
        //  `sigaction` takes; what was in place is kept and put back when the watch ends.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = note_the_stop as *const () as usize;
            // No `SA_RESTART`: a read this process is inside must come back when the signal arrives.
            action.sa_flags = 0;
            libc::sigemptyset(&mut action.sa_mask);
            let mut before: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(signal, &action, &mut before) == 0 {
                previous.push((signal, before));
            }
            let mut unblocked: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut unblocked);
            libc::sigaddset(&mut unblocked, signal);
            let _unblocked = libc::pthread_sigmask(libc::SIG_UNBLOCK, &unblocked, std::ptr::null_mut());
        }
    }
    return StopWatch { previous };
}

impl Drop for StopWatch {
    fn drop(&mut self) {
        STOP_REQUESTED.store(0, Ordering::SeqCst);
        READING.store(-1, Ordering::SeqCst);
        for (signal, before) in std::mem::take(&mut self.previous) {
            // SAFETY: `before` is the action this watch replaced, read back by `sigaction`; putting it back touches no
            //  other memory of this process.
            let _restored = unsafe { libc::sigaction(signal, &before, std::ptr::null_mut()) };
        }
    }
}

/// What:
///  Notes a stop without a process to receive a signal,
///  so a control can drive the wait without the signal.
/// Why:
///  A control of the client that is interrupted would otherwise have to send itself a signal it must not die of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopRequestedForTest(2)
/// ```
#[cfg(test)]
pub(crate) fn stop_requested_for_test(signal: i32) {
    STOP_REQUESTED.store(signal, Ordering::SeqCst);
}

/// The watch and its flag are controlled in the test module.
#[cfg(test)]
#[path = "service_client_interrupted_tests.rs"]
mod tests;
