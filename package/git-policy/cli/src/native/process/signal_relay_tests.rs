//! Controls for the signal guard.
//!
//! Handlers are process-wide,
//! so every test that installs one runs in a helper process:
//! the test executable re-runs this file's `helper_entry` test with a mode in the environment,
//! as `scanner_test_support.rs` re-runs the executable.
//! Outside that mode `helper_entry` returns at once.

use super::*;

use crate::test_support::{executable, fixture, isolated_overlay, remove};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

/// The sender codes the tests feed in;
/// macOS relays every signal whatever its code,
/// so its tests need none.
#[cfg(all(unix, not(target_os = "macos")))]
use libc::{SI_QUEUE, SI_USER};

/// A code that no process sends:
/// the kernel's own on Linux.
#[cfg(target_os = "linux")]
const KERNEL_CODE: std::ffi::c_int = libc::SI_KERNEL;

/// A code that no process sends:
/// the empty code on Unix systems other than Linux and macOS.
#[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
const KERNEL_CODE: std::ffi::c_int = 0;

/// Environment variable naming the helper scenario.
const MODE_VAR: &str = "CLI_GIT_SIGNAL_GUARD_TEST_MODE";
/// Environment variable naming the helper's scratch directory.
const DIR_VAR: &str = "CLI_GIT_SIGNAL_GUARD_TEST_DIR";

/// How long a test waits for a helper step before it fails.
const STEP_TIMEOUT: Duration = Duration::from_secs(60);

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
    if mode == "service" {
        leave_signals_to_the_service();
    }
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
        // The child of a commit prepared inside a service: records its pid and its start, then waits.
        "service" => {
            let mut built: Command = Command::new("/bin/sh");
            built.arg("-c").arg(format!(
                "echo $$ > '{dir}/child-pid'; touch '{dir}/started'; exec sleep 30",
                dir = directory.display()
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

/// Wait until `path` exists,
/// or fail after the step timeout.
fn wait_for(path: &Path) {
    let started: Instant = Instant::now();
    while !path.exists() {
        assert!(started.elapsed() < STEP_TIMEOUT, "{} never appeared", path.display());
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Start this test executable as the helper for one scenario.
fn spawn_helper(mode: &str, directory: &Path) -> Child {
    let executable_path: PathBuf = crate::test_support::this_test_binary().expect("test executable");
    // Owned, so a helper that waits for a signal a failing control never sent does not outlive the control.
    return crate::test_support::spawn_owned(
        Command::new(executable_path)
            .args(["--exact", "process::signal_relay::tests::helper_entry", "--nocapture", "--test-threads=1"])
            .env(MODE_VAR, mode)
            .env(DIR_VAR, directory)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null()),
    )
    .expect("helper starts");
}

/// Send one signal from this test process,
/// so the kernel records it as sent by a process.
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

/// The incumbent-facing contract:
/// a process sender is relayed,
/// the terminal is not.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn only_process_senders_are_relayed() {
    assert!(is_process_sender(SI_USER));
    assert!(is_process_sender(SI_QUEUE));
    assert!(!is_process_sender(KERNEL_CODE));
}

/// macOS reports `si_code` 0 for a real `kill`,
/// so every code counts as a process sender there.
#[cfg(target_os = "macos")]
#[test]
fn every_signal_is_relayed_on_macos() {
    assert!(is_process_sender(0));
    assert!(is_process_sender(-1));
    assert!(is_process_sender(0x10001));
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

/// The handler marks a signal for relay only when a process sent it,
/// so a terminal sender leaves the slot empty.
///
/// The handler runs directly with no child recorded,
/// so nothing is sent and the slot shows the decision.
#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn handler_marks_a_signal_only_for_a_process_sender() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // SAFETY: a zeroed `siginfo_t` is a valid buffer; only its header fields are written below.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let header: *mut SignalHeader = (&mut info as *mut libc::siginfo_t).cast::<SignalHeader>();
    // SAFETY: `header` points into the live `info` buffer above.
    unsafe { (*header).code = KERNEL_CODE };
    relay_handler(libc::SIGINT, &mut info, std::ptr::null_mut());
    assert_eq!(
        PENDING[0].load(std::sync::atomic::Ordering::SeqCst),
        0,
        "a terminal sender is not marked for relay"
    );
    // SAFETY: as above.
    unsafe { (*header).code = SI_USER };
    relay_handler(libc::SIGINT, &mut info, std::ptr::null_mut());
    assert_eq!(
        PENDING[0].load(std::sync::atomic::Ordering::SeqCst),
        libc::SIGINT,
        "a process sender is marked for relay"
    );
    PENDING[0].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  A stop signal a child was relayed is kept for a wait that asks,
///  and taken once.
/// Why:
///   A Git child a person's interrupt is relayed to may already be gone,
///  so the relay is the last thing that knows the person stopped the command:
///  a commit that waited afterwards must withdraw instead of landing
///  (`service_client_interrupted.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// relay_handler(SIGINT, infoWith(SI_USER)); expect(takeRelayedStop()).toBe(SIGINT); expect(takeRelayedStop()).toBe(undefined);
/// ```
#[cfg(unix)]
#[test]
fn a_relayed_stop_is_kept_for_the_wait_that_asks() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    let _cleared = crate::signal_relay_relayed_stop::take();
    // SAFETY: a zeroed `siginfo_t` is a valid buffer; only its header fields are written below.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    let header: *mut SignalHeader = (&mut info as *mut libc::siginfo_t).cast::<SignalHeader>();
    // SAFETY: `header` points into the live `info` buffer above.
    unsafe { (*header).code = SI_USER };
    relay_handler(libc::SIGTERM, &mut info, std::ptr::null_mut());
    assert_eq!(crate::signal_relay_relayed_stop::take(), Some(libc::SIGTERM), "the wait is told which signal the person sent");
    assert_eq!(crate::signal_relay_relayed_stop::take(), None, "one stop is applied once");
    PENDING[1].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// On macOS the handler marks a signal for relay whatever `si_code` it carries,
/// including the 0 a real `kill` gives.
#[cfg(target_os = "macos")]
#[test]
fn handler_marks_a_signal_with_the_code_macos_gives_a_kill() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // SAFETY: a zeroed `siginfo_t` is a valid buffer; its header code stays 0, as the kernel delivers it for `kill`.
    let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
    relay_handler(libc::SIGINT, &mut info, std::ptr::null_mut());
    assert_eq!(
        PENDING[0].load(std::sync::atomic::Ordering::SeqCst),
        libc::SIGINT,
        "a signal with code 0 is marked for relay"
    );
    PENDING[0].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// Serializes the in-process tests that touch the relay statics (`CHILD_PID`,
/// `PENDING`).
#[cfg(unix)]
static RELAY_STATICS: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A signal caught before the child exists is relayed the moment the child's PID is published.
///
/// The pending slot is set directly,
/// as the handler would set it,
/// and no child is recorded yet.
/// Without the relay after publication the child runs its full 30 seconds and exits normally,
/// so the assertion names the outcome:
/// a child that was not signaled exits normally after its sleep.
#[cfg(unix)]
#[test]
fn a_signal_pending_before_spawn_is_relayed_when_the_pid_is_published() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // Slot 1 is SIGTERM; the handler test uses slot 0, so the two tests never read each other's slot.
    PENDING[1].store(libc::SIGTERM, std::sync::atomic::Ordering::SeqCst);
    let mut command: Command = Command::new("sleep");
    command.arg("30");
    let told: crate::child_number_told_at_start::ChildNumber = crate::child_number_told_at_start::ChildNumber::not_started();
    let outcome: ChildOutcome = run_on_runtime(run_relayed(&mut command, true, CeilingDuty { told: &told, bound: None, while_held: None })).expect("the runtime runs the child").expect("child runs").0;
    assert_eq!(
        outcome,
        ChildOutcome::Signaled(libc::SIGTERM),
        "the pending SIGTERM reaches the child"
    );
    PENDING[1].store(0, std::sync::atomic::Ordering::SeqCst);
}

/// The first guard of a process owns the handlers;
/// a guard made meanwhile is made all the same and relays nothing,
/// and the handlers
///  are free for the next owner once the first is dropped.
#[test]
fn a_second_guard_is_made_and_relays_nothing_while_the_first_owns_the_handlers() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    let first: SignalGuard = SignalGuard::install().expect("first guard");
    assert!(first.relaying, "the first guard owns the handlers");
    let second: SignalGuard = SignalGuard::install().expect("a second guard is not refused");
    assert!(!second.relaying, "the second guard relays nothing while the first owns the handlers");
    drop(second);
    assert!(RELAYING.load(std::sync::atomic::Ordering::SeqCst), "dropping the second guard leaves the first one's handlers");
    drop(first);
    let again: SignalGuard = SignalGuard::install().expect("guard after drop");
    assert!(again.relaying, "the handlers are free for the next owner");
    drop(again);
}

/// The child of a guard that owns no handlers is never published to the handler,
/// so a signal of the process cannot reach it.
#[cfg(unix)]
#[test]
fn the_child_of_a_guard_without_handlers_is_never_published() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    let owner: SignalGuard = SignalGuard::install().expect("first guard");
    let other: SignalGuard = SignalGuard::install().expect("second guard");
    let watching: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    let published_once: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            while watching.load(std::sync::atomic::Ordering::SeqCst) {
                if CHILD_PUBLISHED.load(std::sync::atomic::Ordering::SeqCst) {
                    published_once.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        });
        let mut command: Command = Command::new("sleep");
        command.arg("0.3");
        let outcome: ChildOutcome = other.run_child(&mut command).expect("the second guard's child runs");
        watching.store(false, std::sync::atomic::Ordering::SeqCst);
        assert_eq!(outcome, ChildOutcome::Exited(0));
    });
    assert!(!published_once.load(std::sync::atomic::Ordering::SeqCst), "the second guard's child was published to the handler");
    drop(other);
    drop(owner);
}

/// While a child runs the relay descriptor is its pidfd,
/// and once it is retired the descriptor is `/dev/null` again,
/// so a signal after
///  the child's end reaches nothing and the number is never another process's.
#[cfg(target_os = "linux")]
#[test]
fn the_relay_descriptor_names_the_child_while_it_runs_and_nothing_after() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    ensure_relay_descriptor().expect("the relay descriptor is made");
    let number: std::os::fd::RawFd = *RELAY_FD.get().expect("the relay descriptor exists");
    let target = || { return std::fs::read_link(format!("/proc/self/fd/{number}")).expect("the descriptor is open").display().to_string(); };
    let mut command: Command = Command::new("sleep");
    command.arg("30");
    let mut child: Child = command.spawn().expect("sleep starts");
    let published: bool = publish(child.id());
    assert!(published, "the child is published");
    assert!(target().contains("pidfd"), "the descriptor names the child's pidfd while it runs: {}", target());
    retire(published);
    assert_eq!(target(), "/dev/null", "the descriptor names nothing once the child is retired");
    child.kill().expect("the control ends its child");
    child.wait().expect("the child is reaped");
}

/// A process that is a service leaves its stop signals to the service:
/// no guard takes the handlers there,
/// so a stop reaches the service's
///  own handling (here the default,
/// which ends the helper) and the relay does not signal the child of the commit.
/// Ending the Gits a service waits on is the service's own shutdown,
/// not the relay's (`binary_service_stop_ends_its_gits_tests.rs`).
#[cfg(unix)]
#[test]
fn a_stop_of_a_service_is_not_relayed_to_the_child_of_a_commit() {
    let root: PathBuf = fixture("signal-service-owned");
    let helper: Child = spawn_helper("service", root.as_path());
    let pid: u32 = helper.id();
    wait_for(root.join("started").as_path());
    let child_pid: libc::pid_t = std::fs::read_to_string(root.join("child-pid")).expect("the child's pid").trim().parse().expect("a pid");
    send(pid, libc::SIGTERM);
    let status: ExitStatus = finish(helper);
    assert_eq!(std::os::unix::process::ExitStatusExt::signal(&status), Some(libc::SIGTERM), "the stop reached the service's own handling: {status:?}");
    // SAFETY: signal 0 only asks whether the process exists.
    let alive: bool = unsafe { libc::kill(child_pid, 0) } == 0;
    // SAFETY: the control ends the child it started.
    unsafe { libc::kill(child_pid, libc::SIGKILL) };
    assert!(alive, "the stop was not relayed to the commit's child");
    remove(root.as_path());
}

/// A process signal reaching the waiting wrapper ends the child with that signal,
/// and the wrapper continues.
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

/// A terminal interrupt reaches the child once,
/// from the terminal,
/// and is not relayed again.
///
/// The helper runs its child on a pseudo-terminal,
/// and the test writes the interrupt character to the
/// terminal's master side;
/// the kernel then sends `SIGINT` to the foreground group with a kernel sender.
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
    let executable_path: PathBuf = crate::test_support::this_test_binary().expect("test executable");
    let mut command: Command = Command::new(executable_path);
    command
        .args(["--exact", "process::signal_relay::tests::helper_entry", "--nocapture", "--test-threads=1"])
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
    // The `setsid` above makes the helper the leader of a group of its own; `spawn_owned` cannot start it, so only its group is owned.
    crate::test_support::own_group(&helper);
    return (master_file, helper);
}

/// Acceptance in miniature:
/// a held pre-commit ends Git on `SIGTERM` to the wrapper,
/// the wrapper reports 143 and continues.
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

/// The handler address `signal` currently has,
/// read without changing it.
#[cfg(unix)]
fn disposition_of(signal: libc::c_int) -> libc::sighandler_t {
    // SAFETY: a zeroed `sigaction` is a valid buffer for the kernel to fill.
    let mut current: libc::sigaction = unsafe { std::mem::zeroed() };
    // SAFETY: a null new action only reads the current disposition into `current`.
    unsafe { libc::sigaction(signal, std::ptr::null(), &mut current) };
    return current.sa_sigaction;
}

/// A handler that does nothing,
/// standing in for a disposition the process had before the guard.
#[cfg(unix)]
extern "C" fn earlier_handler(_signal: libc::c_int) {}

/// Dropping the guard puts back the disposition each signal had before,
/// not the default.
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

/// A signal the process already ignores stays ignored while the guard lives and after it is dropped,
/// so the Git child inherits the ignore the wrapper was started with.
#[cfg(unix)]
#[test]
fn an_inherited_ignore_is_not_replaced_by_the_guard() {
    let _statics: std::sync::MutexGuard<'_, ()> = RELAY_STATICS.lock().expect("relay statics lock");
    // SAFETY: installs the ignore disposition for a valid signal number.
    unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN) };
    let guard: SignalGuard = SignalGuard::install().expect("guard");
    assert_eq!(
        disposition_of(libc::SIGHUP),
        libc::SIG_IGN,
        "an ignored signal is not replaced while the guard lives"
    );
    drop(guard);
    assert_eq!(
        disposition_of(libc::SIGHUP),
        libc::SIG_IGN,
        "an ignored signal is still ignored after the drop"
    );
    // SAFETY: returns the signal to its default so no later test sees the ignore.
    unsafe { libc::signal(libc::SIGHUP, libc::SIG_DFL) };
}

/// The wait ends when the child has exited,
/// reports its status and leaves it reaped.
#[cfg(target_os = "linux")]
#[test]
fn waiting_ends_at_the_childs_exit_and_reports_it() {
    let mut command: Command = Command::new("sleep");
    command.arg("1");
    // The clock starts before the child does, so a host that starts the waiting late cannot shorten what is measured.
    let started: Instant = Instant::now();
    let told: crate::child_number_told_at_start::ChildNumber = crate::child_number_told_at_start::ChildNumber::not_started();
    let duty: CeilingDuty<'_> = CeilingDuty { told: &told, bound: None, while_held: None };
    let outcome: ChildOutcome = run_on_runtime(run_relayed(&mut command, false, duty)).expect("the runtime runs the child").expect("the child runs").0;
    assert!(started.elapsed() >= Duration::from_millis(800), "the wait lasted until the child exited");
    assert_eq!(outcome, ChildOutcome::Exited(0));
}

/// A signal that interrupts the waiting thread does not end the wait;
/// the wait goes on until the child exits.
///
/// The test handler is installed without `SA_RESTART`,
/// so a blocking system call of the waiting thread would report `EINTR`.
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
    // SAFETY: `pthread_self` has no preconditions.
    let waiting_thread: libc::pthread_t = unsafe { libc::pthread_self() };
    let interrupter = std::thread::spawn(move || {
        for _ in 0..3 {
            std::thread::sleep(Duration::from_millis(150));
            // SAFETY: the waiting thread outlives these signals because the test joins this thread before it ends.
            unsafe { libc::pthread_kill(waiting_thread, libc::SIGUSR1) };
        }
    });
    let told: crate::child_number_told_at_start::ChildNumber = crate::child_number_told_at_start::ChildNumber::not_started();
    let duty: CeilingDuty<'_> = CeilingDuty { told: &told, bound: None, while_held: None };
    let outcome: ChildOutcome = run_on_runtime(run_relayed(&mut command, false, duty)).expect("the runtime runs the child").expect("interrupts do not end the wait").0;
    assert!(started.elapsed() >= Duration::from_millis(800), "the wait lasted until the child exited");
    interrupter.join().expect("interrupter finishes");
    assert_eq!(outcome, ChildOutcome::Exited(0));
}

/// A child of a commit step that is running when the service is stopped is ended by the service's shutdown,
/// and the caller is told the stop did it;
/// the shutdown finds nothing left unfinished.
/// (The stop is process-wide state,
/// so this control runs in a process of its own,
/// as every control does under nextest.)
#[cfg(unix)]
#[test]
fn a_commit_step_running_when_the_service_is_stopped_is_ended_by_the_stop() {
    use crate::stop_of_the_service_noticed_by_its_children::{Settled, prepare_to_notice, record_from_a_signal_handler, tell_the_stop_and_wait_for_the_work, wake_the_shutdown_from_a_signal_handler};
    prepare_to_notice().expect("the wake pipe");
    let guard: SignalGuard = SignalGuard::install().expect("a guard");
    let mut command: Command = Command::new("sleep");
    command.arg("60");
    let (outcome, settled): (io::Result<ChildOutcome>, Settled) = run_on_runtime(async {
        return tokio::join!(guard.run_child_awaiting(&mut command), async {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            assert!(record_from_a_signal_handler(libc::SIGTERM));
            assert!(wake_the_shutdown_from_a_signal_handler());
            return tell_the_stop_and_wait_for_the_work(std::time::Duration::from_secs(60)).await;
        });
    })
    .expect("the runtime");
    let error: io::Error = outcome.expect_err("a stopped step has no outcome");
    assert!(crate::child_watch::is_ended_by_stop(&error), "{error}");
    assert_eq!(settled, Settled::Completely);
}
