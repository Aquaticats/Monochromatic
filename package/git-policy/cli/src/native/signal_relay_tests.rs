//! Controls for the signal guard.
//!
//! Handlers are process-wide, so every test that installs one runs in a helper process:
//! the test executable re-runs this file's `helper_entry` test with a mode in the environment,
//! as `scanner_test_support.rs` re-runs the executable.
//! Outside that mode `helper_entry` returns at once.

use super::*;
use crate::test_support::{executable, fixture, isolated_overlay, remove};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

/// Environment variable naming the helper scenario.
const MODE_VAR: &str = "CLI_GIT_SIGNAL_GUARD_TEST_MODE";
/// Environment variable naming the helper's scratch directory.
const DIR_VAR: &str = "CLI_GIT_SIGNAL_GUARD_TEST_DIR";

/// How long a test waits for a helper step before it fails.
const STEP_TIMEOUT: Duration = Duration::from_secs(20);

/// Helper entry for every guard scenario.
/// Outside the helper environment this test does nothing.
#[test]
fn helper_entry() {
    let Some(mode) = std::env::var_os(MODE_VAR) else {
        return;
    };
    let directory: PathBuf = PathBuf::from(
        std::env::var_os(DIR_VAR).expect("helper directory is set with the mode"),
    );
    let guard: SignalGuard = SignalGuard::install().expect("guard installs");
    std::fs::write(directory.join("installed"), b"").expect("installed marker");
    let mut command: Command = helper_command(mode.to_string_lossy().as_ref(), directory.as_path());
    let outcome: ChildOutcome = guard.run_child(&mut command).expect("guarded child runs");
    std::fs::write(directory.join("outcome"), describe(outcome)).expect("outcome record");
    drop(guard);
    // Written after the guard is gone, so the test sees that the helper continued past the wait.
    std::fs::write(directory.join("cleanup"), b"").expect("cleanup marker");
}

/// The child each helper scenario runs under the guard.
fn helper_command(mode: &str, directory: &Path) -> Command {
    let mut command: Command = match mode {
        // A direct `sleep`, with no shell in front of it: the relay reaches the process that will wait.
        // A shell that runs a foreground command first ignores SIGINT until that command ends,
        // so an early relay could be absorbed and the child would then `exec` without it.
        "early" => {
            let mut built: Command = Command::new("sleep");
            built.arg("30");
            built
        }
        // Long-lived child: records its start, then waits to be signaled.
        "relay" => {
            let mut built: Command = Command::new("/bin/sh");
            built.arg("-c").arg(format!(
                "touch '{}/started'; exec sleep 30",
                directory.display()
            ));
            built
        }
        // Counts every SIGINT it receives, then waits; the terminal case counts deliveries here.
        "terminal" => {
            let mut built: Command = Command::new("/bin/sh");
            built.arg("-c").arg(format!(
                "trap 'echo interrupt >> \"{dir}/interrupts\"' INT; touch '{dir}/started'; while :; do sleep 0.05; done",
                dir = directory.display()
            ));
            built
        }
        // Real Git committing into a repository whose pre-commit hook holds the commit open.
        "commit" => {
            let mut built: Command = Command::new(crate::test_support::REAL_GIT);
            built
                .arg("commit")
                .arg("--allow-empty")
                .arg("--message")
                .arg("held")
                .current_dir(directory.join("repo"));
            built
        }
        other => panic!("unknown helper mode {other}"),
    };
    if mode != "commit" {
        command.current_dir(directory);
    }
    for (name, value) in isolated_overlay() {
        command.env(name, value);
    }
    return command;
}

/// The text a helper writes for a child's outcome.
fn describe(outcome: ChildOutcome) -> String {
    match outcome {
        ChildOutcome::Exited(code) => return format!("exited {code}"),
        ChildOutcome::Signaled(signal) => return format!("signaled {signal}"),
    }
}

/// Wait until `path` exists, or fail after the step timeout.
fn wait_for(path: &Path) {
    let started: Instant = Instant::now();
    while !path.exists() {
        assert!(started.elapsed() < STEP_TIMEOUT, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Start this test executable as the helper for one scenario.
fn spawn_helper(mode: &str, directory: &Path) -> Child {
    let executable_path: PathBuf = std::env::current_exe().expect("test executable");
    return Command::new(executable_path)
        .args(["--exact", "signal_relay::tests::helper_entry", "--nocapture", "--test-threads=1"])
        .env(MODE_VAR, mode)
        .env(DIR_VAR, directory)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("helper starts");
}

/// Send one signal from this test process, so the kernel records it as sent by a process.
fn send(pid: u32, signal: libc::c_int) {
    let target: libc::pid_t = libc::pid_t::try_from(pid).expect("helper pid fits");
    // SAFETY: `kill` names a child this test started and sends one signal; it reads no memory.
    let status: libc::c_int = unsafe { libc::kill(target, signal) };
    assert_eq!(status, 0, "kill delivered");
}

/// Wait for the helper to exit and require a normal exit status.
fn finish(mut helper: Child) -> ExitStatus {
    let started: Instant = Instant::now();
    loop {
        if let Some(status) = helper.try_wait().expect("helper status") {
            return status;
        }
        assert!(started.elapsed() < STEP_TIMEOUT, "helper did not exit");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The incumbent-facing contract: a process sender is relayed, the terminal is not.
#[cfg(unix)]
#[test]
fn only_process_senders_are_relayed() {
    assert!(is_process_sender(libc::SI_USER));
    assert!(is_process_sender(libc::SI_QUEUE));
    assert!(!is_process_sender(libc::SI_KERNEL));
}

/// On Linux a thread-directed kill is a process sender too.
#[cfg(target_os = "linux")]
#[test]
fn thread_directed_kill_is_a_process_sender_on_linux() {
    assert!(is_process_sender(libc::SI_TKILL));
}

/// The guard relays exactly the four signals the wrapper catches.
#[test]
fn guard_catches_the_four_signals() {
    assert_eq!(
        RELAYED_SIGNALS,
        [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT]
    );
}

/// The handler marks a signal for relay only when a process sent it, so a terminal sender leaves the slot empty.
///
/// The handler runs directly with no child recorded, so nothing is sent and the slot shows the decision.
#[cfg(unix)]
#[test]
fn handler_marks_a_signal_only_for_a_process_sender() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // SAFETY: a zeroed `siginfo_t` is a valid buffer; only its header fields are written below.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let header: *mut SignalHeader = (&mut info as *mut libc::siginfo_t).cast::<SignalHeader>();
    // SAFETY: `header` points into the live `info` buffer above.
    unsafe { (*header).code = libc::SI_KERNEL };
    relay_handler(libc::SIGINT, &mut info, std::ptr::null_mut());
    assert_eq!(
        PENDING[0].load(std::sync::atomic::Ordering::SeqCst),
        0,
        "a terminal sender is not marked for relay"
    );
    // SAFETY: as above.
    unsafe { (*header).code = libc::SI_USER };
    relay_handler(libc::SIGINT, &mut info, std::ptr::null_mut());
    assert_eq!(
        PENDING[0].load(std::sync::atomic::Ordering::SeqCst),
        libc::SIGINT,
        "a process sender is marked for relay"
    );
    PENDING[0].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// Serializes the in-process tests that touch the relay statics (`CHILD_PID`, `PENDING`).
#[cfg(unix)]
static RELAY_STATICS: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A signal caught before the child exists is relayed the moment the child's PID is published.
///
/// The pending slot is set directly, as the handler would set it, and no child is recorded yet.
/// Without the relay after publication the child runs its full 30 seconds and exits normally,
/// so the assertion names the outcome and the bound instead of waiting for the sleep to end.
#[cfg(unix)]
#[test]
fn a_signal_pending_before_spawn_is_relayed_when_the_pid_is_published() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // Slot 1 is SIGTERM; the handler test uses slot 0, so the two tests never read each other's slot.
    PENDING[1].store(libc::SIGTERM, std::sync::atomic::Ordering::SeqCst);
    let mut command: Command = Command::new("sleep");
    command.arg("30");
    let started: Instant = Instant::now();
    let outcome: ChildOutcome = run_relayed(&mut command).expect("child runs");
    assert_eq!(
        outcome,
        ChildOutcome::Signaled(libc::SIGTERM),
        "the pending SIGTERM reaches the child"
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the child was signaled at once, not after its sleep"
    );
    PENDING[1].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// One guard may be active at a time in a process.
#[test]
fn a_second_guard_is_refused_until_the_first_is_dropped() {
    let first: SignalGuard = SignalGuard::install().expect("first guard");
    assert!(SignalGuard::install().is_err());
    drop(first);
    let again: SignalGuard = SignalGuard::install().expect("guard after drop");
    drop(again);
}

/// A process signal reaching the waiting wrapper ends the child with that signal, and the wrapper continues.
#[cfg(unix)]
#[test]
fn process_signal_is_relayed_and_the_child_outcome_is_reported() {
    let root: PathBuf = fixture("signal-relay");
    let helper: Child = spawn_helper("relay", root.as_path());
    let pid: u32 = helper.id();
    wait_for(root.join("started").as_path());
    send(pid, libc::SIGTERM);
    let status: ExitStatus = finish(helper);
    assert!(status.success(), "the wrapper survived the signal: {status:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("outcome")).expect("outcome"),
        format!("signaled {}", libc::SIGTERM)
    );
    assert!(root.join("cleanup").exists(), "the wrapper continued after the child");
    remove(root.as_path());
}

/// A signal sent before the child is started is relayed once the child exists.
#[cfg(unix)]
#[test]
fn signal_during_start_is_relayed_after_the_child_starts() {
    let root: PathBuf = fixture("signal-early");
    let helper: Child = spawn_helper("early", root.as_path());
    let pid: u32 = helper.id();
    wait_for(root.join("installed").as_path());
    send(pid, libc::SIGINT);
    let status: ExitStatus = finish(helper);
    assert!(status.success(), "the wrapper survived the signal: {status:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("outcome")).expect("outcome"),
        format!("signaled {}", libc::SIGINT)
    );
    remove(root.as_path());
}

/// A terminal interrupt reaches the child once, from the terminal, and is not relayed again.
///
/// The helper runs its child on a pseudo-terminal, and the test writes the interrupt character to the
/// terminal's master side; the kernel then sends `SIGINT` to the foreground group with a kernel sender.
#[cfg(target_os = "linux")]
#[test]
fn terminal_interrupt_is_not_relayed_a_second_time() {
    let root: PathBuf = fixture("signal-terminal");
    let (mut master, helper): (std::fs::File, Child) = spawn_terminal_helper(root.as_path());
    wait_for(root.join("started").as_path());
    std::io::Write::write_all(&mut master, b"\x03").expect("interrupt character written");
    wait_for(root.join("interrupts").as_path());
    // Long enough for a second, relayed delivery to reach the child's trap.
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        std::fs::read_to_string(root.join("interrupts")).expect("interrupt log").lines().count(),
        1,
        "the interrupt reached the child exactly once"
    );
    send(helper.id(), libc::SIGTERM);
    let status: ExitStatus = finish(helper);
    assert!(status.success(), "the wrapper survived: {status:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("outcome")).expect("outcome"),
        format!("signaled {}", libc::SIGTERM)
    );
    remove(root.as_path());
}

/// Start the terminal helper with a pseudo-terminal as its controlling terminal and return the master side.
#[cfg(target_os = "linux")]
fn spawn_terminal_helper(directory: &Path) -> (std::fs::File, Child) {
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::os::unix::process::CommandExt;
    let mut master: libc::c_int = -1;
    let mut slave: libc::c_int = -1;
    // SAFETY: `openpty` writes one descriptor per out-pointer; the null name, termios and window pointers are allowed.
    let opened: libc::c_int = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(opened, 0, "pseudo-terminal opens");
    // SAFETY: both descriptors were just returned by `openpty` and nothing else owns them.
    let master_file: std::fs::File = unsafe { std::fs::File::from_raw_fd(master) };
    // SAFETY: as above; the slave is owned once and cloned into the helper's standard streams.
    let slave_owned: OwnedFd = unsafe { OwnedFd::from_raw_fd(slave) };
    let executable_path: PathBuf = std::env::current_exe().expect("test executable");
    let mut command: Command = Command::new(executable_path);
    command
        .args(["--exact", "signal_relay::tests::helper_entry", "--nocapture", "--test-threads=1"])
        .env(MODE_VAR, "terminal")
        .env(DIR_VAR, directory)
        .stdin(std::process::Stdio::from(slave_owned.try_clone().expect("slave clone")))
        .stdout(std::process::Stdio::from(slave_owned.try_clone().expect("slave clone")))
        .stderr(std::process::Stdio::from(slave_owned));
    // SAFETY: between fork and exec only async-signal-safe calls run: `setsid` and `ioctl` on descriptor 0.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            libc::ioctl(0, libc::TIOCSCTTY, 0);
            return Ok(());
        });
    }
    let helper: Child = command.spawn().expect("terminal helper starts");
    return (master_file, helper);
}

/// Acceptance in miniature: a held pre-commit ends Git on `SIGTERM` to the wrapper, the wrapper reports 143 and continues.
#[cfg(unix)]
#[test]
fn term_during_a_held_pre_commit_ends_git_and_reports_143() {
    let root: PathBuf = fixture("signal-commit");
    let repo: PathBuf = crate::test_support::repository(root.as_path(), "repo");
    executable(
        repo.join(".git").join("hooks").join("pre-commit").as_path(),
        format!("#!/bin/sh\ntouch '{}/started'\nsleep 5\n", root.display()).as_bytes(),
    );
    let helper: Child = spawn_helper("commit", root.as_path());
    let pid: u32 = helper.id();
    wait_for(root.join("started").as_path());
    send(pid, libc::SIGTERM);
    let status: ExitStatus = finish(helper);
    assert!(status.success(), "the wrapper survived: {status:?}");
    assert_eq!(
        std::fs::read_to_string(root.join("outcome")).expect("outcome"),
        format!("signaled {}", libc::SIGTERM)
    );
    assert_eq!(128 + libc::SIGTERM, 143);
    assert!(root.join("cleanup").exists());
    remove(root.as_path());
}

/// The handler address `signal` currently has, read without changing it.
#[cfg(unix)]
fn disposition_of(signal: libc::c_int) -> libc::sighandler_t {
    // SAFETY: a zeroed `sigaction` is a valid buffer for the kernel to fill.
    let mut current: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: a null new action only reads the current disposition into `current`.
    unsafe { libc::sigaction(signal, std::ptr::null(), &mut current) };
    return current.sa_sigaction;
}

/// A handler that does nothing, standing in for a disposition the process had before the guard.
#[cfg(unix)]
extern "C" fn earlier_handler(_signal: libc::c_int) {}

/// Dropping the guard puts back the disposition each signal had before, not the default.
#[cfg(unix)]
#[test]
fn dropping_the_guard_restores_the_previous_dispositions() {
    let earlier: libc::sighandler_t = earlier_handler as extern "C" fn(libc::c_int) as libc::sighandler_t;
    // SAFETY: installs a handler that does nothing for a valid signal number.
    unsafe { libc::signal(libc::SIGHUP, earlier) };
    let guard: SignalGuard = SignalGuard::install().expect("guard");
    assert_eq!(
        disposition_of(libc::SIGHUP),
        relay_handler as *const () as libc::sighandler_t,
        "the guard replaces the earlier handler while it lives"
    );
    drop(guard);
    assert_eq!(
        disposition_of(libc::SIGHUP),
        earlier,
        "the earlier handler is back after the drop"
    );
    // SAFETY: returns the signal to its default so no later test sees the stand-in.
    unsafe { libc::signal(libc::SIGHUP, libc::SIG_DFL) };
}

/// The wait returns when the child has exited and leaves the child unreaped, so its PID still names it.
#[cfg(target_os = "linux")]
#[test]
fn waiting_ends_at_the_childs_exit_and_leaves_it_unreaped() {
    let mut command: Command = Command::new("sleep");
    command.arg("1");
    let mut child: Child = command.spawn().expect("sleep starts");
    let pid: libc::pid_t = libc::pid_t::try_from(child.id()).expect("pid fits");
    let started: Instant = Instant::now();
    wait_without_reaping(pid).expect("the wait succeeds");
    assert!(
        started.elapsed() >= Duration::from_millis(800),
        "the wait lasted until the child exited"
    );
    let state: String = std::fs::read_to_string(format!("/proc/{pid}/stat")).expect("the child is still listed");
    assert!(state.contains(") Z"), "the child is a zombie, not reaped: {state}");
    assert!(child.wait().expect("the child is still waitable").success());
}

/// A PID that is not this process's child has nothing to wait for, which is success, not an error.
#[cfg(target_os = "linux")]
#[test]
fn waiting_for_a_process_that_is_not_a_child_ends_without_error() {
    wait_without_reaping(libc::pid_t::MAX).expect("no such child means nothing to wait for");
}

/// A signal that interrupts the wait does not end it; the wait goes on until the child exits.
///
/// The test handler is installed without `SA_RESTART`, so the interrupted `waitid` reports `EINTR`.
#[cfg(target_os = "linux")]
#[test]
fn an_interrupted_wait_continues_until_the_child_exits() {
    // SAFETY: a zeroed `sigaction` is a valid starting value; the fields used below are set first.
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = earlier_handler as extern "C" fn(libc::c_int) as libc::sighandler_t;
    action.sa_flags = 0;
    // SAFETY: `sigemptyset` initializes the mask in place.
    unsafe { libc::sigemptyset(&mut action.sa_mask) };
    // SAFETY: installs a handler that does nothing for a valid signal number.
    unsafe { libc::sigaction(libc::SIGUSR1, &action, std::ptr::null_mut()) };
    let mut command: Command = Command::new("sleep");
    command.arg("1");
    // The clock starts before the child does, so a host that starts the waiting late cannot shorten what is measured.
    let started: Instant = Instant::now();
    let mut child: Child = command.spawn().expect("sleep starts");
    let pid: libc::pid_t = libc::pid_t::try_from(child.id()).expect("pid fits");
    // SAFETY: `pthread_self` has no preconditions.
    let waiting_thread: libc::pthread_t = unsafe { libc::pthread_self() };
    let interrupter = std::thread::spawn(move || {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(150));
            // SAFETY: the waiting thread outlives these signals because the test joins this thread before it ends.
            unsafe { libc::pthread_kill(waiting_thread, libc::SIGUSR1) };
        }
    });
    wait_without_reaping(pid).expect("interrupts do not end the wait");
    assert!(
        started.elapsed() >= Duration::from_millis(800),
        "the wait lasted until the child exited"
    );
    interrupter.join().expect("interrupter finishes");
    assert!(child.wait().expect("the child is still waitable").success());
}

/// The refusal of a second guard carries its type, and nothing else is taken for it.
#[cfg(unix)]
#[test]
fn only_the_refusal_of_a_second_guard_is_recognized_as_such() {
    let first: SignalGuard = SignalGuard::install().expect("first guard");
    let refusal: io::Error = SignalGuard::install().err().expect("a second guard is refused");
    assert!(is_guard_active_refusal(&refusal));
    assert_eq!(refusal.to_string(), "a signal guard is already active in this process");
    drop(first);
    assert!(!is_guard_active_refusal(&io::Error::other("a signal guard is already active in this process")));
    assert!(!is_guard_active_refusal(&io::Error::from_raw_os_error(libc::EAGAIN)));
}
