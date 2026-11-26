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

/// The process that sent the signals these controls raise themselves,
/// so a statement about a sender can name it.
#[cfg(unix)]
fn own_process() -> Option<i32> {
    return Some(i32::try_from(std::process::id()).expect("a process ID fits"));
}

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
///  The handlers belong to the process,
///  so the body runs in a helper process of its own,
///  where a death by the signal fails the helper and is named in its verdict (issue 6463).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = watchForAStop(); process.kill(process.pid, 'SIGINT'); expect(stopRequested()).toBe(SIGINT); watch.release();
/// ```
#[cfg(unix)]
#[test]
fn the_first_signal_is_noted_and_the_command_goes_on() {
    if let Some(directory) = crate::helper_process_test_support::helper_directory(FIRST_SIGNAL_CHILD) {
        std::process::exit(crate::helper_process_test_support::verdict_in_a_helper(directory.as_path(), the_first_signal_verdict()));
    }
    let failures: Vec<String> = crate::helper_process_test_support::in_a_helper(FIRST_SIGNAL_CHILD, "service-interrupted-first-signal", "service::service_client_interrupted::tests::the_first_signal_is_noted_and_the_command_goes_on");
    assert!(failures.is_empty(), "{failures:?}");
}

/// The sentences the helper of the first-signal control failed with.
#[cfg(unix)]
fn the_first_signal_verdict() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let watch: StopWatch = watch_for_a_stop();
    if stop().is_some() {
        failures.push(String::from("nothing was sent yet"));
    }
    // SAFETY: `raise` sends one signal to this thread, which is the one whose watch notes it.
    let sent: libc::c_int = unsafe { libc::raise(libc::SIGINT) };
    if sent != 0 {
        failures.push(String::from("the interrupt was sent"));
    }
    if stop() != Some((libc::SIGINT, own_process())) {
        failures.push(String::from("the wait is told which signal came, and this process sent it"));
    }
    drop(watch);
    if stop().is_some() {
        failures.push(String::from("the watch ends with the flag it set"));
    }
    return failures;
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
///  The handlers of the guard are process-wide,
///  so the measurement runs in a helper process of this test binary,
///  which this test starts with `test_support::spawn_owned`,
///  and the signal names that helper and never the test binary (issue 6463).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = thread(block(SIGINT); using awake = wakingTheReadOf(socket); markReady(); readLine());
/// // in the parent: wait for the mark, process.kill(helper.pid, 'SIGINT'), expect the verdict to be empty
/// ```
#[cfg(unix)]
#[test]
fn a_stop_wakes_the_read_the_wait_is_inside() {
    if let Some(directory) = crate::helper_process_test_support::helper_directory(WAKING_READ_CHILD) {
        // What: `exit` ends this helper with the verdict as its status, so the parent reads either the file or the status.
        // Why:  A helper that exited 0 without measuring would pass a parent that only read the status.
        std::process::exit(crate::helper_process_test_support::verdict_in_a_helper(directory.as_path(), waking_read_verdict(directory.as_path())));
    }
    let failures: Vec<String> = crate::helper_process_test_support::in_a_helper_then(
        WAKING_READ_CHILD,
        "service-interrupted-waking-read",
        "service::service_client_interrupted::tests::a_stop_wakes_the_read_the_wait_is_inside",
        |root, helper| {
            crate::helper_process_test_support::wait_for_file_while_a_helper_runs(root.join(READING_MARKER).as_path(), helper);
            // SAFETY: `kill` takes no pointers, and the target is the helper this control started, never this test binary.
            let target: libc::pid_t = libc::pid_t::try_from(helper.id()).expect("a helper pid fits the pid type");
            let sent: libc::c_int = unsafe { libc::kill(target, libc::SIGINT) };
            assert_eq!(sent, 0, "the interrupt was sent to the helper");
        },
    );
    assert!(failures.is_empty(), "{failures:?}");
}

/// The file the helper writes once its reading thread is about to block,
/// which is the moment the parent may send the stop.
const READING_MARKER: &str = "reading";

/// The variable that makes a run of this test the helper whose read the parent's signal must wake,
/// carrying the fixture directory.
const WAKING_READ_CHILD: &str = "CLI_GIT_INTERRUPTED_WAKING_READ_CHILD";

/// The wait's own stop word and the relayed one belong to the process,
/// so the controls that read or write them run one at a time,
/// and the lock is taken through the poison so one control's panic cannot fail the others.
#[cfg(unix)]
static THE_STOP_WORD: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// What:
///  `fn hold_the_stop_word() -> MutexGuard<'static, ()>` takes the lock that makes one control's use of the stop word its
///  own.
///  `MutexGuard` is the proof the lock is held, released when the value drops (sibling: `()` alone, which would release it
///  at once).
/// Why:
///  `stop_requested_for_test` writes one word of the process and `stop()` reads it,
///  so two controls running as threads of one process would read each other's stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdTheStopWord(): Disposable // using held = holdTheStopWord()
/// ```
#[cfg(unix)]
fn hold_the_stop_word() -> std::sync::MutexGuard<'static, ()> {
    return THE_STOP_WORD.lock().unwrap_or_else(|poisoned| return poisoned.into_inner());
}

/// The variable that makes a run of the first-signal control its helper.
const FIRST_SIGNAL_CHILD: &str = "CLI_GIT_INTERRUPTED_FIRST_SIGNAL_CHILD";

/// The variable that makes a run of the hang-up control its helper.
const HANGUP_CHILD: &str = "CLI_GIT_INTERRUPTED_HANGUP_CHILD";

/// The variable that makes a run of the ignored-signal control its helper.
const IGNORE_CHILD: &str = "CLI_GIT_INTERRUPTED_IGNORE_CHILD";

/// The variable that makes a run of the disposition control its helper.
const DISPOSITION_CHILD: &str = "CLI_GIT_INTERRUPTED_DISPOSITION_CHILD";

/// What:
///  `fn waking_read_verdict(directory: &Path) -> Vec<String>` makes the measurement in the helper process and answers the
///  sentences it failed with.
/// Why:
///  The stop's handlers are process-wide, so the measurement runs where no other control of a threaded run can see them,
///  and a control that failed says why in one file rather than through a panic whose text a parent never prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wakingReadVerdict(directory: string): string[] { /* measure the woken read and return what failed */ }
/// ```
#[cfg(unix)]
fn waking_read_verdict(directory: &std::path::Path) -> Vec<String> {
    use std::io::BufRead as _;
    let mut failures: Vec<String> = Vec::new();
    let (client, _service) = std::os::unix::net::UnixStream::pair().expect("a connected pair");
    let _timeout = client.set_read_timeout(Some(std::time::Duration::from_secs(10)));
    let watch: StopWatch = watch_for_a_stop();
    let marker: PathBuf = directory.join(READING_MARKER);
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
        // The marker is written after the descriptor is declared, so the parent's stop finds the wake already in place.
        std::fs::write(&marker, b"").expect("the reading marker");
        let read = reader.read_line(&mut line);
        drop(awake);
        return (read.is_ok(), began.elapsed());
    });
    let (read_answered, waited): (bool, std::time::Duration) = reading.join().expect("the reading thread ends");
    if !read_answered {
        failures.push(String::from("the read ended with an answer, not with its timeout"));
    }
    if waited >= std::time::Duration::from_secs(2) {
        failures.push(format!("the read ended {} ms after it began", waited.as_millis()));
    }
    // What: `getppid` is the process that started this helper, which is the test binary the parent ran the control in.
    // Why:  The kernel names the sender, and the parent is the only process that sent anything.
    let sender: i32 = i32::try_from(unsafe { libc::getppid() }).expect("a parent process ID fits");
    let stopped: Option<(i32, Option<i32>)> = stop();
    if stopped != Some((libc::SIGINT, Some(sender))) {
        failures.push(format!("the stop is noted for the wait, with the sending process as its sender: {stopped:?}"));
    }
    drop(watch);
    return failures;
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
    let _stop_word: std::sync::MutexGuard<'_, ()> = hold_the_stop_word();
    stop_requested_for_test(libc::SIGTERM, Some(4_242));
    assert_eq!(stop(), Some((libc::SIGTERM, Some(4_242))), "the process that sent the stop is kept");
    stop_requested_for_test(libc::SIGINT, None);
    assert_eq!(stop(), Some((libc::SIGINT, None)), "a keyboard's interrupt names no process");
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
///  The handlers belong to the process,
///  so the body runs in a helper process of its own.
#[cfg(unix)]
#[test]
fn the_wait_answers_a_hangup_and_a_quit_too() {
    if let Some(directory) = crate::helper_process_test_support::helper_directory(HANGUP_CHILD) {
        std::process::exit(crate::helper_process_test_support::verdict_in_a_helper(directory.as_path(), the_hangup_verdict()));
    }
    let failures: Vec<String> = crate::helper_process_test_support::in_a_helper(HANGUP_CHILD, "service-interrupted-hangup", "service::service_client_interrupted::tests::the_wait_answers_a_hangup_and_a_quit_too");
    assert!(failures.is_empty(), "{failures:?}");
}

/// The sentences the helper of the hang-up control failed with.
#[cfg(unix)]
fn the_hangup_verdict() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    // No core file for the quit this control sends a process that may not answer it yet.
    // SAFETY: `setrlimit` takes a number, a `rlimit` of this local and the value of the constant; it touches no other
    //  memory and this is the one process the call changes.
    let bounded: libc::rlimit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    let _limited = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &bounded) };
    for signal in [libc::SIGHUP, libc::SIGQUIT] {
        let watch: StopWatch = watch_for_a_stop();
        // SAFETY: `raise` sends one signal to this thread, which is the one whose watch notes it.
        let sent: libc::c_int = unsafe { libc::raise(signal) };
        if sent != 0 {
            failures.push(format!("signal {signal} was sent"));
        }
        if stop() != Some((signal, own_process())) {
            failures.push(format!("the wait is told which signal came, and this control sent it: {signal}"));
        }
        drop(watch);
        if stop().is_some() {
            failures.push(format!("the watch ends with the flag it set: {signal}"));
        }
    }
    return failures;
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
    // The death by signal is the assertion, so this asks for the raw ending rather than for a passing test.
    let environment: [(&str, &std::ffi::OsStr); 1] = [(SECOND_SIGNAL_CHILD, std::ffi::OsStr::new("1"))];
    let output: std::process::Output = crate::child_test_process::finished(crate::child_test_process::ChildTest {
        exact_test: "service::service_client_interrupted::tests::a_second_signal_ends_the_process_by_it",
        environment: environment.as_slice(),
        ..crate::child_test_process::ChildTest::default()
    })
    .expect("the child ends within the deadline");
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
///  A disposition belongs to the process,
///  so the body runs in a helper process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = actionOf(SIGTERM); const watch = watchForAStop(); watch.release(); expect(actionOf(SIGTERM)).toEqual(before);
/// ```
#[cfg(unix)]
#[test]
fn the_actions_in_place_come_back_when_the_watch_ends() {
    if let Some(directory) = crate::helper_process_test_support::helper_directory(DISPOSITION_CHILD) {
        std::process::exit(crate::helper_process_test_support::verdict_in_a_helper(directory.as_path(), the_actions_come_back_verdict()));
    }
    let failures: Vec<String> = crate::helper_process_test_support::in_a_helper(DISPOSITION_CHILD, "service-interrupted-disposition", "service::service_client_interrupted::tests::the_actions_in_place_come_back_when_the_watch_ends");
    assert!(failures.is_empty(), "{failures:?}");
}

/// The sentences the helper of the disposition control failed with.
#[cfg(unix)]
fn the_actions_come_back_verdict() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    let before: libc::sigaction = action_of(libc::SIGTERM);
    let watch: StopWatch = watch_for_a_stop();
    if action_of(libc::SIGTERM).sa_sigaction == before.sa_sigaction {
        failures.push(String::from("the watch is in place while it lasts"));
    }
    drop(watch);
    if action_of(libc::SIGTERM).sa_sigaction != before.sa_sigaction {
        failures.push(String::from("what was in place is back"));
    }
    return failures;
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
/// and the preparation's own relay:
/// "A signal the parent ignores stays ignored",
/// `process/signal_relay.rs`).
///  A disposition belongs to the process,
///  so the body runs in a helper process of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ignore(SIGHUP); const watch = watchForAStop(); raise(SIGHUP); expect(stopRequested()).toBeUndefined();
/// expect(actionOf(SIGHUP)).toEqual(ignore);
/// ```
#[cfg(unix)]
#[test]
fn a_signal_the_caller_ignored_stays_ignored_and_is_never_a_stop() {
    if let Some(directory) = crate::helper_process_test_support::helper_directory(IGNORE_CHILD) {
        std::process::exit(crate::helper_process_test_support::verdict_in_a_helper(directory.as_path(), the_ignored_signal_verdict()));
    }
    let failures: Vec<String> = crate::helper_process_test_support::in_a_helper(IGNORE_CHILD, "service-interrupted-ignored", "service::service_client_interrupted::tests::a_signal_the_caller_ignored_stays_ignored_and_is_never_a_stop");
    assert!(failures.is_empty(), "{failures:?}");
}

/// The sentences the helper of the ignored-signal control failed with.
#[cfg(unix)]
fn the_ignored_signal_verdict() -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    // A stop another control of this process relayed is no stop of this one.
    let _stale = crate::signal_relay_relayed_stop::take();
    let ignored: libc::sigaction = action_of(libc::SIGHUP);
    let quit: libc::sigaction = action_of(libc::SIGQUIT);
    // SAFETY: `signal` takes a number and a disposition that outlives this statement; both are the caller's own choice of
    //  `SIG_IGN`, put back at the end of this control.
    let set_hup: usize = unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN) };
    let set_quit: usize = unsafe { libc::signal(libc::SIGQUIT, libc::SIG_IGN) };
    if (set_hup, set_quit) != (0, 0) {
        failures.push(String::from("the ignore is in place before the watch"));
    }
    let watch: StopWatch = watch_for_a_stop();
    if action_of(libc::SIGHUP).sa_sigaction != libc::SIG_IGN {
        failures.push(String::from("the caller's ignore of the hang-up is left in place"));
    }
    if action_of(libc::SIGQUIT).sa_sigaction != libc::SIG_IGN {
        failures.push(String::from("the caller's ignore of the quit is left in place"));
    }
    // SAFETY: `raise` sends one signal to this thread; both signals are ignored, so neither can end this control.
    let raised: (libc::c_int, libc::c_int) = unsafe { (libc::raise(libc::SIGHUP), libc::raise(libc::SIGQUIT)) };
    if raised != (0, 0) {
        failures.push(String::from("the shielded signals were sent"));
    }
    if stop().is_some() {
        failures.push(String::from("a signal the caller ignored is no stop of the commit"));
    }
    // The other watched signals are still answered: the ignore is what is honoured, not the whole watch.
    // SAFETY: `raise` sends one signal to this thread, whose watch notes it.
    let stopped: libc::c_int = unsafe { libc::raise(libc::SIGTERM) };
    if stopped != 0 {
        failures.push(String::from("the stop was sent"));
    }
    if stop() != Some((libc::SIGTERM, own_process())) {
        failures.push(String::from("a signal the caller did not ignore is still a stop, and this control sent it"));
    }
    drop(watch);
    // SAFETY: the two dispositions read at the start are the process's own; putting them back touches no memory.
    let _hup = unsafe { libc::sigaction(libc::SIGHUP, &ignored, std::ptr::null_mut()) };
    let _quit = unsafe { libc::sigaction(libc::SIGQUIT, &quit, std::ptr::null_mut()) };
    return failures;
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
    let _stop_word: std::sync::MutexGuard<'_, ()> = hold_the_stop_word();
    let _stale = crate::signal_relay_relayed_stop::take();
    stop_requested_for_test(0, None);
    crate::signal_relay_relayed_stop::note(libc::SIGTERM, Some(4_242));
    assert_eq!(stop(), Some((libc::SIGTERM, Some(4_242))), "the wait learns the signal the relay kept, with the process the kernel named");
    // A relayed stop with no sender is no sender, and never the one the last stop named.
    crate::signal_relay_relayed_stop::note(libc::SIGTERM, None);
    assert_eq!(stop(), Some((libc::SIGTERM, None)), "the second stop is taken too, and names no process");
}

/// What:
///  The stop the wait answers is one fact:
///  the signal and the process the kernel named as its sender are taken together,
///  so a stop that arrives while the first is being answered never renames the first one's sender.
/// Why:
///  A person's Ctrl-C and a tool's `SIGTERM` are the same number,
///  and the sender is the only fact that tells them apart,
///  so a record that pairs one stop's signal with another stop's sender names a process that did not send
///  what ended the command (`commit_stop_reason.rs`,
/// `client-signal`).
///  The handler may run on any thread at any instruction,
///  and the call site used to read the signal and then,
/// a step later,
/// the sender.
///  This control puts a second stop exactly where that step was:
///  it is the shape the call site has,
///  and it states what the wait took.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = whatTheWaitTook(() => noteAStop(SIGINT, B)); expect(taken).toEqual([SIGTERM, A]);
/// ```
#[cfg(unix)]
#[test]
fn a_stop_that_arrives_while_the_first_is_answered_never_renames_its_sender() {
    let _stop_word: std::sync::MutexGuard<'_, ()> = hold_the_stop_word();
    let _stale = crate::signal_relay_relayed_stop::take();
    stop_requested_for_test(0, None);
    const FIRST_SENDER: i32 = 4_242;
    const SECOND_SENDER: i32 = 9_191;
    // A stop from the first process arrives during the preparation, as a relayed one.
    crate::signal_relay_relayed_stop::note(libc::SIGTERM, Some(FIRST_SENDER));
    // The wait takes that stop, and the second process's interrupt arrives at the moment the sender used to be read.
    let taken: Option<(i32, Option<i32>)> = what_the_wait_took(|| {
        // The second process's interrupt, noted the way the handler notes one: the same words, in the same atomics.
        stop_requested_for_test(libc::SIGINT, Some(SECOND_SENDER));
    });
    assert_eq!(
        taken,
        Some((libc::SIGTERM, Some(FIRST_SENDER))),
        "the record names the stop the wait answered and the process that sent that stop"
    );
    // And the shape the handler writes reads back as one fact: both halves in one call.
    stop_requested_for_test(0, None);
    stop_requested_for_test(libc::SIGTERM, Some(FIRST_SENDER));
    assert_eq!(stop(), Some((libc::SIGTERM, Some(FIRST_SENDER))), "a stop's signal and sender come back together");
}

/// What:
///  The stop the wait answers,
/// taken the way its call site takes it,
///  with the moment between the call site's two reads given to the control.
/// Why:
///  The call site read the signal and then the sender,
///  and the control has to put another thread's delivery exactly there;
///  naming that moment is what makes the hazard reachable in a control at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = whatTheWaitTook(between); // [signal, sender] as the call site read them
/// ```
#[cfg(unix)]
fn what_the_wait_took(between: impl FnOnce()) -> Option<(i32, Option<i32>)> {
    // One call, one read: what the second delivery does afterwards cannot change what was taken.
    let taken: Option<(i32, Option<i32>)> = stop();
    between();
    return taken;
}
