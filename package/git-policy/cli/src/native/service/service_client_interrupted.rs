//! What:
//!  The signals that end a commit while it waits for the commit service,
//!  and the flag they set for the wait.
//! Why:
//!  A commit the person interrupts is not made:
//!  native Git makes none when it is interrupted,
//!  and the wrapper's promise for a commit that is not made is that nothing lands later by itself
//!  (`SPEC.md`,
//! "The commit protocol").
//!  The request a commit already sent is the service's,
//! so ending this process where the signal arrives would leave it
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

use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};

/// What:
///  The stop the wait answers,
///  as one word:
///  the signal in the low half and one more than its sender in the high half,
///  so zero means no stop,
///  in the encoding `stop_word` owns.
/// Why:
///  A signal handler may run on any thread at any instruction,
///  and two words let a stop that arrives between the wait's two reads rename the first stop's sender:
///  a record would then pair one stop's signal with another stop's process,
///  and the command would end by a signal whose sender it never named
///  (`stop_word.rs`,
/// and the same hazard the relay's word closes).
///  One word cannot be read half written.
static STOP: AtomicU64 = AtomicU64::new(0);

/// The connection whose read is running,
/// and minus one while none is:
/// a handler may call only what the kernel allows,
/// and shutting this read down is how a stop that arrives on another thread reaches the wait.
static READING: AtomicI32 = AtomicI32::new(-1);

/// What:
///  The signals a waiting commit answers by withdrawing its request:
///  the interrupt a terminal sends (Ctrl-C),
///  the stop a caller sends at its own bound,
///  the hang-up a closing terminal and a session that ends send,
///  and the quit a person sends.
/// Why:
///  Each one means the person or the caller is gone,
///  and a commit that is not made must say why it was not:
///  the default action of a hang-up and of a quit ends the process where it arrives,
///  so a commit that was waiting left no record at all
///  (a closing terminal's hang-up and `loginctl terminate-session`'s are the same signal).
///  The command still ends by the signal afterwards,
///  so a quit keeps its core file where the system would make one.
///  A `SIGKILL` cannot be caught at all;
///  its request is dropped by the service when the connection closes
///  (`service_server_land_group.rs`).
const WATCHED: [std::os::raw::c_int; 4] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

/// What:
///  The watched signals whose caller's own `SIG_IGN` the wait leaves in place:
///  the hang-up a closing terminal or a session that ends sends,
///  and a quit.
/// Why:
///   `nohup` shields the program it starts by setting `SIGHUP` to `SIG_IGN`,
///  and a caller that does means it to survive a closing terminal:
///  native Git lands such a commit,
///  and this wrapper landed it before the wait answered a hang-up at all.
///  Replacing the caller's ignore withdrew the request and ended the process by the signal the caller shielded it from,
///  which is a commit that fails where real Git lands one,
///  and the preparation's own relay already keeps an inherited ignore for the same reason
///  (`process/signal_relay.rs`,
/// "A signal the parent ignores stays ignored",
///  and `SPEC.md`,
/// "Merges and commits never fail").
///  The interrupt and the stop are not of these:
///  a shell that ignores the interrupt for a background job while a person still means a Ctrl-C to end the command
///  is what the wait answered before this branch,
/// and that is a question of its own.
const KEPT_IGNORED: [std::os::raw::c_int; 2] = [libc::SIGHUP, libc::SIGQUIT];

/// What:
///  Notes the stop the person asked for,
///  or ends this process at once for a second one.
/// Why:
///   A signal handler may call only what the kernel allows,
///  so this writes one word and nothing else:
///  `signal`,
/// `raise` and a lock-free atomic store are all on that list.
///  The second signal restores the default action first,
///  so the process ends the way the shell expects a Ctrl-C to end it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.on(signal, () => { if (alreadyAsked) { signal(SIG_DFL); raise(signal) } else { asked = signal } });
/// ```
extern "C" fn note_the_stop(signal: std::os::raw::c_int, info: *mut libc::siginfo_t, _context: *mut std::ffi::c_void) {
    // What: The signal and the process the kernel named as its sender go in as one word, written once.
    // Why:  The record states which stop ended the commit and who sent it, and a handler that wrote the two apart would let
    //       a reader take half of one stop beside half of another; `si_pid` is on the list of what a handler may read, and a
    //       terminal's interrupt and a timer carry no sender, which the word states as zero.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (stop.swap(pack(signal, senderOf(info))) === 0) { wake(); return }
    // ```
    if STOP.swap(crate::stop_word::packed(signal, crate::signal_report::sender_in_report(info)), Ordering::SeqCst) == 0 {
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

/// What:
///  The stop a waiting commit answers,
///  as the one fact it is:
///  the signal,
///  and the process the kernel named as its sender.
/// Why:
///  A signal handler may run on any thread at any instruction,
///  and a wait that read the signal and then,
/// a step later,
/// the sender could pair one stop's signal with the
///  sender of a stop that arrived between the two reads.
///  The command ends by the signal it names and its record states that sender,
///  so the two have to be one fact read once (`stop_word.rs`).
///  The relay's word is the second place a stop can come from,
///  and it is one word too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stop() // [SIGTERM, 4242], or undefined
/// ```
pub(crate) fn stop() -> Option<(i32, Option<i32>)> {
    // What: One read of the wait's own word, and then the relayed one if the wait's is empty.
    // Why:  Each read carries a stop whole, so no delivery can rename the sender of the stop this answers; the word is not
    //       cleared here, because the handler tells a second signal from a first by it.
    if let Some(stopped) = crate::stop_word::unpacked(STOP.load(Ordering::SeqCst)) {
        return Some(stopped);
    }
    return crate::signal_relay_relayed_stop::take();
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
    /// The actions that were in place before this watch,
    /// in the order of `WATCHED`.
    previous: Vec<(std::os::raw::c_int, libc::sigaction)>,
}

/// What:
///  Watches the signals a waiting commit answers,
///  for as long as the value lives.
/// Why:
///  An interrupt or a stop the process started with ignored is replaced so the commit is told,
///  and put back when the value is dropped;
///  a hang-up or a quit the process started with ignored is left ignored,
///  so a commit `nohup` shielded lands (`KEPT_IGNORED`).
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
    STOP.store(0, Ordering::SeqCst);
    let mut previous: Vec<(std::os::raw::c_int, libc::sigaction)> = Vec::new();
    for signal in WATCHED {
        // SAFETY: a null new action only reads what is in place, into this local.
        let mut before: libc::sigaction = unsafe { std::mem::zeroed() };
        let read: std::os::raw::c_int = unsafe { libc::sigaction(signal, std::ptr::null(), &mut before) };
        // What: A hang-up or a quit the caller ignores is the caller's own choice, and the wait leaves it alone.
        // Why:  `nohup` shields a commit that way, and a shielded commit lands: answering the signal here would withdraw a
        //       request the caller asked to be shielded from. Nothing is unblocked either, so the process's own signal mask
        //       is left as the caller set it.
        if read == 0 && before.sa_sigaction == libc::SIG_IGN && KEPT_IGNORED.contains(&signal) {
            continue;
        }
        // SAFETY: an empty mask and a handler that writes one word, shuts a descriptor down and raises a number are what
        //  `sigaction` takes; what was in place is kept and put back when the watch ends.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = note_the_stop as *const () as usize;
            // What: `SA_SIGINFO` carries the sender of the signal into the handler; `SA_RESTART` is left off.
            // Why:  The record names who stopped the commit, and a read this process is inside must come back when the signal
            //       arrives.
            action.sa_flags = libc::SA_SIGINFO;
            libc::sigemptyset(&mut action.sa_mask);
            let mut replaced: libc::sigaction = std::mem::zeroed();
            if libc::sigaction(signal, &action, &mut replaced) == 0 {
                previous.push((signal, replaced));
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
        STOP.store(0, Ordering::SeqCst);
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
///  so a control can drive the wait without the signal,
///  and states the sender the kernel would have named.
/// Why:
///  A control of the client that is interrupted would otherwise have to send itself a signal it must not die of,
///  and a control of what the record says about who sent it must be able to state that process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopRequestedForTest(2, sender)
/// ```
#[cfg(test)]
pub(crate) fn stop_requested_for_test(signal: i32, sender: Option<i32>) {
    // One word, as the handler writes it: the seam stands for a delivery and never for half of one.
    STOP.store(crate::stop_word::packed(signal, sender), Ordering::SeqCst);
}

/// The watch and its flag are controlled in the test module.
#[cfg(test)]
#[path = "service_client_interrupted_tests.rs"]
mod tests;
