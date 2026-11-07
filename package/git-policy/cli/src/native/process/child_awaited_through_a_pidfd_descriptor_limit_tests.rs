//! The child helper under a descriptor limit:
//! each sweep lowers the soft `RLIMIT_NOFILE` of the test's own process (nextest runs
//! every test in a process of its own),
//! so the limit reaches `pidfd_open`,
//! the pipes of a start and the files of a capture in turn.
//! The invariants are the plan's ("A child is started twice,
//! or a started child is reported as failed"):
//! a start the limit refuses
//! leaves no child,
//! a start that worked is never reported failed,
//! the status comes back whichever way the wait is done,
//! and
//! nothing is left behind (no descriptor,
//! no child).

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::fs::File;
use std::process::{Command, Stdio};

/// The most descriptors a start needs beyond the process's baseline:
/// the standard library's error-report pipe,
/// and a margin.
/// A sweep over `0..=ROOM_SWEPT` therefore covers every limit from "the start is refused" to "the start works with room".
const ROOM_SWEPT: usize = 8;

/// How many descriptors are left free for the pidfd in the sweep of the pidfd's own limit:
/// none (the open fails),
/// one (it works).
const FREE_FOR_THE_PIDFD: [usize; 2] = [0, 1];

/// What the open descriptors are,
/// for a failure message.
fn describe_descriptors() -> Vec<String> {
    return std::fs::read_dir("/proc/self/fd")
        .expect("the descriptor listing")
        .flatten()
        .map(|entry| { return format!("{} -> {:?}", entry.file_name().to_string_lossy(), std::fs::read_link(entry.path())); })
        .collect();
}

/// The number of descriptors the process holds now,
/// the directory handle of the listing itself not counted.
fn open_descriptors() -> usize {
    let listed: usize = std::fs::read_dir("/proc/self/fd").expect("the descriptor listing").count();
    return listed - 1;
}

/// Sets the soft descriptor limit and returns the one it replaced,
/// so the caller restores it.
fn set_soft_limit(soft: u64) -> rustix::process::Rlimit {
    let before: rustix::process::Rlimit = rustix::process::getrlimit(rustix::process::Resource::Nofile);
    let lowered: rustix::process::Rlimit = rustix::process::Rlimit { current: Some(soft), maximum: before.maximum };
    rustix::process::setrlimit(rustix::process::Resource::Nofile, lowered).expect("a soft limit below the hard one can be set");
    return before;
}

/// Puts a limit back.
fn restore_limit(before: rustix::process::Rlimit) {
    rustix::process::setrlimit(rustix::process::Resource::Nofile, before).expect("the limit can be raised back to the hard one");
}

/// Opens descriptors until `free` are left under the limit;
/// returns them,
/// to be dropped when the step under test is done.
fn leave_free(free: usize) -> Vec<File> {
    let mut held: Vec<File> = Vec::new();
    loop {
        match File::open("/dev/null") {
            Ok(file) => held.push(file),
            Err(error) => {
                assert_eq!(error.raw_os_error(), Some(libc::EMFILE), "only the limit stops the opens: {error}");
                break;
            }
        }
    }
    // Give back `free` of them, so that exactly that many opens still work.
    for _ in 0..free {
        held.pop();
    }
    return held;
}

/// Whether the process still has a child it has not reaped:
/// `waitpid` with no child answers `ECHILD`.
fn has_unreaped_child() -> bool {
    // SAFETY: `waitpid` with a null status pointer and `WNOHANG` touches no memory of this process.
    let reaped: libc::pid_t = unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::WNOHANG) };
    return reaped != -1 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ECHILD);
}

/// The runtime is built before any limit is lowered:
/// the build itself needs descriptors,
/// and its own limits are the subject of
/// `binary_runtime_build_under_limits_tests.rs`.
fn warm_runtime() {
    run_on_runtime(async {}).expect("the runtime");
}

/// A child whose status is `code`,
/// started with inherited streams so the start needs only its error-report pipe.
fn quick_child(code: i32) -> std::io::Result<std::process::Child> {
    return Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("sleep 0.05; exit {code}"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// When the limit leaves no descriptor for `pidfd_open`,
/// the child that started is still awaited,
/// by the timer,
/// and its status
/// comes back;
/// with one descriptor free,
/// the pidfd is used.
/// The two cases prove the limit really reached `pidfd_open` (a sweep
/// whose every run used the pidfd would test nothing).
#[test]
fn a_limit_that_refuses_the_pidfd_changes_the_wait_and_nothing_else() {
    warm_runtime();
    let baseline: usize = open_descriptors();
    let mut seen: Vec<(usize, bool)> = Vec::new();
    for free in FREE_FOR_THE_PIDFD {
        let before = set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
        let child: std::process::Child = quick_child(3).expect("a start with room");
        let held: Vec<File> = leave_free(free);
        // The registration with the reactor needs the runtime, so the child is adopted inside it.
        let (used_the_pidfd, waited) = run_on_runtime(async {
            let mut owned: OwnedChild = adopt(child);
            let used: bool = owned.waits_on_a_pidfd();
            return (used, owned.wait().await);
        })
        .expect("the runtime");
        seen.push((free, used_the_pidfd));
        let status = waited.expect("the wait");
        assert_eq!(status.code(), Some(3), "{free} free: the status of the child that started");
        drop(held);
        restore_limit(before);
        assert_eq!(open_descriptors(), baseline, "{free} free: no descriptor stays open");
        assert!(!has_unreaped_child(), "{free} free: the child was reaped by the wait");
    }
    assert_eq!(seen, vec![(0, false), (1, true)], "the limit reached `pidfd_open` exactly when it left none");
}

/// A start the limit refuses is an error of the start and leaves no child;
/// a start that worked is awaited to its status;
/// and
/// whichever it is,
/// no descriptor and no child stays.
/// The limit sweeps from one that refuses the start to one with room.
#[test]
fn a_descriptor_limit_never_leaves_a_child_or_a_descriptor() {
    warm_runtime();
    let baseline: usize = open_descriptors();
    let mut refused: usize = 0;
    let mut finished: usize = 0;
    for room in 0..=ROOM_SWEPT {
        let before = set_soft_limit(u64::try_from(baseline + room).expect("a count"));
        let started: std::io::Result<std::process::Child> = quick_child(5);
        match started {
            Err(error) => {
                refused += 1;
                assert_eq!(error.raw_os_error(), Some(libc::EMFILE), "room {room}: the limit is the reason: {error}");
            }
            Ok(child) => {
                finished += 1;
                let status = run_on_runtime(async { return adopt(child).wait().await; }).expect("the runtime").expect("the wait");
                assert_eq!(status.code(), Some(5), "room {room}: a start that worked is awaited to its status");
            }
        }
        restore_limit(before);
        assert_eq!(open_descriptors(), baseline, "room {room}: no descriptor stays open");
        assert!(!has_unreaped_child(), "room {room}: no child stays, whether the start was refused or worked");
    }
    assert!(refused > 0 && finished > 0, "the sweep must cross the limit: {refused} refused, {finished} finished");
}

/// The captured run,
/// whose output goes to anonymous files,
/// under the same sweep:
/// either the command's output comes back whole or
/// the run ends with the descriptor limit as its plain error;
/// no descriptor and no child stays either way.
#[test]
fn a_captured_run_under_a_descriptor_limit_is_whole_or_a_plain_error_and_leaves_nothing() {
    warm_runtime();
    let baseline: usize = open_descriptors();
    let mut refused: usize = 0;
    let mut whole: usize = 0;
    for room in 0..=ROOM_SWEPT + ROOM_SWEPT {
        let before = set_soft_limit(u64::try_from(baseline + room).expect("a count"));
        let mut command: Command = Command::new("/bin/sh");
        command.arg("-c").arg("printf out; printf err >&2; exit 4").stdin(Stdio::null());
        let outcome = run_on_runtime(crate::captured_child_run_to_its_end::output_of(&mut command)).expect("the runtime");
        // The command holds the streams it was given until it is dropped, so the descriptor count is taken after that.
        drop(command);
        match outcome {
            Err(error) => {
                refused += 1;
                assert_eq!(error.raw_os_error(), Some(libc::EMFILE), "room {room}: the limit is the reason: {error}");
            }
            Ok(output) => {
                whole += 1;
                assert_eq!(output.status.code(), Some(4), "room {room}");
                assert_eq!(output.stdout, b"out", "room {room}");
                assert_eq!(output.stderr, b"err", "room {room}");
            }
        }
        restore_limit(before);
        assert_eq!(open_descriptors(), baseline, "room {room}: no descriptor stays open: {:?}", describe_descriptors());
        assert!(!has_unreaped_child(), "room {room}: no child stays");
    }
    assert!(refused > 0 && whole > 0, "the sweep must cross the limit: {refused} refused, {whole} whole");
}
