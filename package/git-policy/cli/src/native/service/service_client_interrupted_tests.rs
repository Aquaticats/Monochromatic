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
///  The first signal of a wait is noted, not acted on:
///  the command must stay alive to withdraw the request it sent.
/// Why:
///   A default action ends this process where the signal arrives,
///  and the request it sent would then land with nobody to read the answer.
///  This control dies of the interrupt if the watch is not installed, which the test harness reports as a death by signal.
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
        stop_requested_for_test(libc::SIGINT);
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
