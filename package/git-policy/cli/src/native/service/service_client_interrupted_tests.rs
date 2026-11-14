//! Controls for the signals a waiting commit answers:
//! the first one is noted and the command goes on to withdraw its request,
//! the second one ends this process at once,
//! and the actions that were in place before come back when the watch ends.

use super::*;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt as _;
use std::path::PathBuf;

/// The variable that makes a run of this test the child whose death is the assertion.
const SECOND_SIGNAL_CHILD: &str = "CLI_GIT_INTERRUPTED_SECOND_SIGNAL_CHILD";

/// What is in place for `signal` now,
/// as `sigaction` reports it.
#[cfg(unix)]
fn action_of(signal: std::os::raw::c_int) -> libc::sigaction {
    // SAFETY: a null `act` only reads what is in place, into this local.
    return unsafe {
        let mut current: libc::sigaction = std::mem::zeroed();
        let _read = libc::sigaction(signal, std::ptr::null(), &mut current);
        current
    };
}

/// What:
///  The first signal of a wait is noted,
/// not acted on:
///  the command must stay alive to withdraw the request it sent.
/// Why:
///   A default action ends this process where the signal arrives,
///  and the request it sent would then land with nobody to read the answer.
///  This control dies of the interrupt if the watch is not installed,
/// which the test harness reports as a death by signal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = watchForAStop(); process.kill(process.pid, 'SIGINT'); expect(stopRequested()).toBe(SIGINT); watch.release();
/// ```
#[cfg(unix)]
#[test]
fn the_first_signal_is_noted_and_the_command_goes_on() {
    let watch: StopWatch = watch_for_a_stop();
    assert_eq!(stop_requested(), None, "nothing was sent yet");
    // SAFETY: `raise` sends one signal to this thread, which is the one whose watch notes it.
    let sent: libc::c_int = unsafe { libc::raise(libc::SIGINT) };
    assert_eq!(sent, 0, "the interrupt was sent");
    assert_eq!(stop_requested(), Some(libc::SIGINT), "the wait is told which signal came");
    drop(watch);
    assert_eq!(stop_requested(), None, "the watch ends with the flag it set");
}

/// What:
///  A stop delivered while the wait is inside a read wakes that read,
///  so the withdrawal starts at once and not a silence bound later.
/// Why:
///   The kernel hands a process-directed signal to any thread of the process;
///  a handler that only wrote a flag would leave the waiting thread in its read until the service spoke or the read
///  timed out,
///  and a commit that lands in that window would be reported as landed although the person interrupted it.
///  The reading thread blocks the signal here,
///  so delivery has to find another thread:
///  the wake through the declared connection is what this control measures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = thread(block(SIGINT); using awake = wakingTheReadOf(socket); readLine()); sleep(200); process.kill(pid, 'SIGINT'); expect(readReturnedWithin(2s)).toBe(true);
/// ```
#[cfg(unix)]
#[test]
fn a_stop_wakes_the_read_the_wait_is_inside() {
    use std::io::BufRead as _;
    let (client, _service) = std::os::unix::net::UnixStream::pair().expect("a connected pair");
    let _timeout = client.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let watch: StopWatch = watch_for_a_stop();
    let reading = std::thread::spawn(move || {
        // The waiting thread holds the signal, so the stop has to be delivered to another one.
        // SAFETY: `sigemptyset` and `sigaddset` write only this local set, and `pthread_sigmask` reads it and touches no other
        //  memory of this process.
        unsafe {
            let mut held: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut held);
            libc::sigaddset(&mut held, libc::SIGINT);
            let _held = libc::pthread_sigmask(libc::SIG_BLOCK, &held, std::ptr::null_mut());
        }
        let began: std::time::Instant = std::time::Instant::now();
        let mut reader: std::io::BufReader<std::os::unix::net::UnixStream> = std::io::BufReader::new(client);
        let mut line: String = String::new();
        let awake = waking_the_read_of(reader.get_ref());
        let read = reader.read_line(&mut line);
        drop(awake);
        return (read.is_ok(), began.elapsed());
    });
    std::thread::sleep(std::time::Duration::from_millis(200));
    // SAFETY: `kill` takes no pointers and the target is this test's own process, whose watch notes the signal and wakes the
    //  read the other thread is inside.
    let sent: libc::c_int = unsafe { libc::kill(libc::getpid(), libc::SIGINT) };
    assert_eq!(sent, 0, "the interrupt was sent");
    let (read_answered, waited): (bool, std::time::Duration) = reading.join().expect("the reading thread ends");
    assert!(read_answered, "the read ended with an answer, not with its timeout");
    assert!(waited < std::time::Duration::from_secs(2), "the read ended {} ms after it began", waited.as_millis());
    assert_eq!(stop_requested(), Some(libc::SIGINT), "the stop is noted for the wait");
    drop(watch);
}

/// What:
///  A stop a process sent keeps the sender the kernel named,
///  and one the terminal raised keeps none.
/// Why:
///  The record's `sender` is the only fact that separates a keyboard's interrupt from the stop a caller sends at its own
///  bound,
/// so a control states both and reads them back:
///  a number the wrapper guessed would make the two look alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopRequestedForTest(SIGTERM, 4242); expect(stopSender()).toBe(4242); stopRequestedForTest(SIGINT, undefined); expect(stopSender()).toBeUndefined();
/// ```
#[cfg(unix)]
#[test]
fn a_stop_that_names_its_sender_is_kept() {
    stop_requested_for_test(libc::SIGTERM, Some(4_242));
    assert_eq!(stop_sender(), Some(4_242), "the process that sent the stop is kept");
    stop_requested_for_test(libc::SIGINT, None);
    assert_eq!(stop_sender(), None, "a keyboard's interrupt names no process");
}

/// What:
///  A closed terminal's hang-up and a quit are answers the wait gives too,
///  and the signal it names is the one that came.
/// Why:
///  A caller that hangs up on a slow commit sends `SIGHUP`,
///  and a person's quit is `SIGQUIT`,
///  so either one ending the process where it arrives loses the record of a commit that was not made:
///  the default action of both ends the process,
///  and nothing on disk then says why the commit did not land.
///  `loginctl terminate-session` and every closing terminal send the first of them.
#[cfg(unix)]
#[test]
fn the_wait_answers_a_hangup_and_a_quit_too() {
    // No core file for the quit this control sends a process that may not answer it yet.
    // SAFETY: `setrlimit` takes a number, a `rlimit` of this local and the value of the constant; it touches no other
    //  memory and this is the one process the call changes.
    let bounded: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    let _limited = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &bounded) };
    for signal in [libc::SIGHUP, libc::SIGQUIT] {
        let watch: StopWatch = watch_for_a_stop();
        // SAFETY: `raise` sends one signal to this thread, which is the one whose watch notes it.
        let sent: libc::c_int = unsafe { libc::raise(signal) };
        assert_eq!(sent, 0, "signal {signal} was sent");
        assert_eq!(stop_requested(), Some(signal), "the wait is told which signal came");
        drop(watch);
        assert_eq!(stop_requested(), None, "the watch ends with the flag it set");
    }
}

/// What:
///  The second signal ends the process at once,
///  and the death is the one the shell expects from a Ctrl-C,
///  so a command that is stuck withdrawing can always be ended.
/// Why:
///  The body runs in a child,
/// because the death is the assertion,
/// as the control of `end_as_git_ended` does (`forwarding_tests.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // in a child: watch(); noteOneStop(); kill(pid, SIGINT); expect(parent).toSeeSignal('SIGINT');
/// ```
#[cfg(unix)]
#[test]
fn a_second_signal_ends_the_process_by_it() {
    if std::env::var_os(SECOND_SIGNAL_CHILD).is_some() {
        let _watch: StopWatch = watch_for_a_stop();
        stop_requested_for_test(libc::SIGINT, None);
        // SAFETY: `raise` sends one signal to this thread, which is the one whose watch sees a second signal and ends the
        //  process by it.
        let _sent: libc::c_int = unsafe { libc::raise(libc::SIGINT) };
        // Only reached when the second signal did not end this process: the parent names that status.
        std::process::exit(0);
    }
    let binary: PathBuf = crate::test_support::this_test_binary().expect("the test binary");
    let output: std::process::Output = std::process::Command::new(binary)
        .args([
            "--exact",
            "service::service_client_interrupted::tests::a_second_signal_ends_the_process_by_it",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(SECOND_SIGNAL_CHILD, "1")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("the child test runs");
    assert_eq!(
        output.status.signal(),
        Some(libc::SIGINT),
        "the child ended with {:?} instead of the death by signal {}: {}",
        output.status.code(),
        libc::SIGINT,
        String::from_utf8_lossy(output.stderr.as_slice())
    );
}

/// What:
///  The watch puts back what was in place for both signals,
///  so a command's watch never outlives its wait.
/// Why:
///  A `SIG_IGN` a shell set before the command is the person's choice and comes back with the watch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = actionOf(SIGTERM); const watch = watchForAStop(); watch.release(); expect(actionOf(SIGTERM)).toEqual(before);
/// ```
#[cfg(unix)]
#[test]
fn the_actions_in_place_come_back_when_the_watch_ends() {
    let before: libc::sigaction = action_of(libc::SIGTERM);
    let watch: StopWatch = watch_for_a_stop();
    assert_ne!(action_of(libc::SIGTERM).sa_sigaction, before.sa_sigaction, "the watch is in place while it lasts");
    drop(watch);
    assert_eq!(action_of(libc::SIGTERM).sa_sigaction, before.sa_sigaction, "what was in place is back");
}

/// What:
///  A signal this process started with ignored stays ignored for the whole wait,
///  and is never taken for a stop.
/// Why:
///  `nohup` shields the command it starts by setting `SIGHUP` to `SIG_IGN`,
///  and a caller that shields a commit that way means it to survive a closing terminal:
///  native Git lands such a commit,
///  and the wrapper landed it before the wait answered a hang-up at all.
///  Replacing the caller's ignore with a handler withdrew the request and ended the process by the signal the caller
///  shielded it from,
///  which is a commit that fails where real Git lands one
///  (`SPEC.md`,
/// "Merges and commits never fail",
/// and the preparation's own relay: "A signal the parent ignores stays ignored",
/// `process/signal_relay.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ignore(SIGHUP); const watch = watchForAStop(); raise(SIGHUP); expect(stopRequested()).toBeUndefined();
/// expect(actionOf(SIGHUP)).toEqual(ignore);
/// ```
#[cfg(unix)]
#[test]
fn a_signal_the_caller_ignored_stays_ignored_and_is_never_a_stop() {
    // A stop another control of this process relayed is no stop of this one.
    let _stale = crate::signal_relay_relayed_stop::take();
    let ignored: libc::sigaction = action_of(libc::SIGHUP);
    let quit: libc::sigaction = action_of(libc::SIGQUIT);
    // SAFETY: `signal` takes a number and a disposition that outlives this statement; both are the caller's own choice of
    //  `SIG_IGN`, put back at the end of this control.
    let set_hup: usize = unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN) };
    let set_quit: usize = unsafe { libc::signal(libc::SIGQUIT, libc::SIG_IGN) };
    assert_eq!((set_hup, set_quit), (0, 0), "the ignore is in place before the watch");
    let watch: StopWatch = watch_for_a_stop();
    assert_eq!(action_of(libc::SIGHUP).sa_sigaction, libc::SIG_IGN, "the caller's ignore of the hang-up is left in place");
    assert_eq!(action_of(libc::SIGQUIT).sa_sigaction, libc::SIG_IGN, "the caller's ignore of the quit is left in place");
    // SAFETY: `raise` sends one signal to this thread; both signals are ignored, so neither can end this control.
    let raised: (libc::c_int, libc::c_int) = unsafe { (libc::raise(libc::SIGHUP), libc::raise(libc::SIGQUIT)) };
    assert_eq!(raised, (0, 0), "the shielded signals were sent");
    assert_eq!(stop_requested(), None, "a signal the caller ignored is no stop of the commit");
    // The other watched signals are still answered: the ignore is what is honoured, not the whole watch.
    // SAFETY: `raise` sends one signal to this thread, whose watch notes it.
    let stopped: libc::c_int = unsafe { libc::raise(libc::SIGTERM) };
    assert_eq!(stopped, 0, "the stop was sent");
    assert_eq!(stop_requested(), Some(libc::SIGTERM), "a signal the caller did not ignore is still a stop");
    drop(watch);
    // SAFETY: the two dispositions read at the start are the process's own; putting them back touches no memory.
    let _hup = unsafe { libc::sigaction(libc::SIGHUP, &ignored, std::ptr::null_mut()) };
    let _quit = unsafe { libc::sigaction(libc::SIGQUIT, &quit, std::ptr::null_mut()) };
}
/// What:
///  A stop relayed during the preparation keeps its signal and the process the kernel named as its sender,
///  so the record of a commit a tool's signal ended names that tool and not a keyboard's interrupt.
/// Why:
///  A tool that gives up on a slow command sends `SIGTERM`,
///  which is the signal a person's own stop sends;
///  the wait learns of either through the relay when the signal arrives while a Git child of the preparation runs,
///  and a record that says only the number cannot tell the two apart (`commit_stop_reason.rs`,
/// `client-signal`).
///  The relay is handed the kernel's `siginfo_t`,
///  so the sender is one word of the report it already reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relayNote(SIGTERM, 4242); expect(stopRequested()).toBe(SIGTERM); expect(stopSender()).toBe(4242);
/// ```
#[cfg(unix)]
#[test]
fn a_stop_relayed_during_the_preparation_keeps_the_sender_the_kernel_named() {
    let _stale = crate::signal_relay_relayed_stop::take();
    stop_requested_for_test(0, None);
    crate::signal_relay_relayed_stop::note(libc::SIGTERM, Some(4_242));
    assert_eq!(stop_requested(), Some(libc::SIGTERM), "the wait learns the signal the relay kept");
    assert_eq!(stop_sender(), Some(4_242), "and the process the kernel named as its sender");
    // A relayed stop with no sender is no sender, and never the one the last stop named.
    crate::signal_relay_relayed_stop::note(libc::SIGTERM, None);
    assert_eq!(stop_requested(), Some(libc::SIGTERM), "the second stop is taken too");
    assert_eq!(stop_sender(), None, "a signal no process sent names no process");
}

/// What:
///  The relay's handler reads the sender out of the kernel's report,
///  and takes none where the kernel named none.
/// Why:
///  The handler may call only what the kernel allows,
///  so the sender it keeps has to come from the report it is handed and never from a lookup:
///  `si_pid` is the one fact that tells a tool's `SIGTERM` from a terminal's.
///  The report is built here in the layout the kernel uses on every Unix (`si_signo`, `si_errno`, `si_code`, then the
///  sender's process ID in the union those codes use),
///  which is the layout the handler reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(senderIn(report(SIGTERM, SI_USER, 4242))).toBe(4242);
/// expect(senderIn(report(SIGINT, SI_KERNEL, 0))).toBeUndefined();
/// ```
#[cfg(unix)]
#[test]
fn the_relay_reads_the_sender_of_a_signal_the_kernel_attributes() {
    let mut from_process: libc::siginfo_t = report_of(libc::SIGTERM, libc::SI_USER, 4_242);
    assert_eq!(
        crate::signal_relay::sender_in_report(&raw mut from_process),
        Some(4_242),
        "a process's own signal names that process"
    );
    // A terminal's interrupt carries a sender code that is no process's, and macOS reports no code at all for one, so
    //  only the systems that name the code say anything here.
    #[cfg(target_os = "linux")]
    {
        let mut from_the_terminal: libc::siginfo_t = report_of(libc::SIGINT, libc::SI_KERNEL, 0);
        assert_eq!(
            crate::signal_relay::sender_in_report(&raw mut from_the_terminal),
            None,
            "a terminal's interrupt names no process"
        );
    }
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
