//! The child helper:
//! a child started with the standard library is awaited without a thread,
//! owned through every failure after
//! its start,
//! and reaped or handed over,
//! never reported as a failed start.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A shell child that sleeps `seconds` and exits with `code`.
/// A shell is the one program every host has.
fn sleeper(seconds: &str, code: i32) -> std::process::Child {
    return Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("sleep {seconds}; exit {code}"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the shell starts");
}

/// Whether a process with this id still exists (a zombie still does).
fn exists(pid: u32) -> bool {
    return std::path::Path::new(format!("/proc/{pid}").as_str()).exists();
}

/// Every way the helper can fail after the start,
/// plus none.
const ALL_FAILURES: [InjectedFailure; 3] = [InjectedFailure::None, InjectedFailure::PidfdOpen, InjectedFailure::Registration];

/// A child that exits is awaited and its status comes back,
/// whatever failed after its start.
#[test]
fn a_child_that_exits_gives_its_status_after_every_failure_after_the_start() {
    for failure in ALL_FAILURES {
        let status = run_on_runtime(async {
            let mut owned: OwnedChild = adopt_with(sleeper("0.05", 3), failure);
            return owned.wait().await;
        })
        .expect("the runtime")
        .expect("the wait");
        assert_eq!(status.code(), Some(3), "{failure:?}");
    }
}

/// After a failure the helper still owns a live child:
/// it was not reported failed,
/// it is not lost,
/// and it can be reaped.
#[test]
fn a_failure_after_the_start_leaves_the_live_child_owned_and_reapable() {
    for failure in [InjectedFailure::PidfdOpen, InjectedFailure::Registration] {
        let child = sleeper("5", 0);
        let pid: u32 = child.id();
        let mut owned: OwnedChild = adopt_with(child, failure);
        assert!(exists(pid), "{failure:?}: the child must be alive and owned");
        assert_eq!(owned.id(), pid);
        owned.end_and_reap();
        assert!(!exists(pid), "{failure:?}: the child must be reaped, not left a zombie or running");
    }
}

/// Dropping an owned live child ends and reaps it,
/// so the caller's cleanup never runs beside a live child.
#[test]
fn dropping_an_owned_live_child_ends_and_reaps_it() {
    let child = sleeper("5", 0);
    let pid: u32 = child.id();
    drop(adopt_with(child, InjectedFailure::None));
    assert!(!exists(pid));
}

/// A child handed back is the caller's again and is not ended when the helper goes.
#[test]
fn a_handed_over_child_is_not_ended_by_the_helper() {
    let child = sleeper("0.3", 0);
    let pid: u32 = child.id();
    let mut back: std::process::Child = adopt_with(child, InjectedFailure::None).into_std();
    assert!(exists(pid));
    assert!(back.wait().expect("the caller waits").success());
}

/// Waiting uses no thread of its own:
/// many children are awaited at once on a current-thread runtime,
/// in about the time of one.
#[test]
fn many_children_are_awaited_at_once_without_a_thread_each() {
    let started: Instant = Instant::now();
    let count: usize = 8;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let local = tokio::task::LocalSet::new();
    let statuses: Vec<std::process::ExitStatus> = runtime.block_on(local.run_until(async {
        let mut handles = Vec::new();
        for _ in 0..count {
            let mut owned: OwnedChild = adopt_with(sleeper("0.4", 0), InjectedFailure::None);
            handles.push(tokio::task::spawn_local(async move { return owned.wait().await; }));
        }
        let mut done = Vec::new();
        for handle in handles {
            done.push(handle.await.expect("the task").expect("the wait"));
        }
        return done;
    }));
    assert_eq!(statuses.len(), count);
    assert!(started.elapsed() < Duration::from_millis(400) * 4, "the waits ran one after another: {:?}", started.elapsed());
}

use crate::test_support::{fixture, remove};
use std::path::PathBuf;

/// What:
///  Dropping a child that leads no process group ends the helper it started.
/// Why:
///  `OwnedChild::drop` signalled the group `-child_pid`, which names a group only when the child was started as its
/// leader, and then ended the leader alone: an ordinary Git child — a plain `Command`, no `process_group(0)` — kept its
/// helper running after the lane's ceiling dropped the future that owned it (the review of `7a6504e5b6`, finding 3).
///  A Git that detaches its maintenance helper on purpose (`maintenance.autodetach`) leaves that helper in no group of the
/// child's either, so only the tree the child started can be ended.
///  The helper here is a real process the child starts, and the control reads that it is gone;
/// a control that catches the defect kills the helper itself, so a failing run leaves no process behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = Command('/bin/sh', '-c', 'sleep 300 & echo $! > helper; wait').spawn(); const owned = adopt(child);
/// drop(owned); expect(runs(readPid('helper'))).toBe(false)
/// ```
#[test]
fn dropping_a_child_that_leads_no_group_ends_the_helper_it_started() {
    let root: PathBuf = fixture("child-drop-no-group");
    let reported: PathBuf = root.join("helper");
    let child: std::process::Child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("sleep {THE_HELPERS_OWN_SLEEP_SECONDS} & echo $! > '{}'; wait", reported.display()))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a child of this test");
    let began: std::time::Instant = std::time::Instant::now();
    while !reported.exists() && began.elapsed() < crate::parallel_test_support::PATIENCE {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let written: String = std::fs::read_to_string(reported.as_path()).expect("the helper reported its own number");
    let helper: u32 = written.trim().parse().expect("the number the shell wrote is a process id");
    // The drop the lane's ceiling makes: the value that owns the child goes, and the child runs on.
    drop(adopt(child));
    let ended_by: std::time::Instant = std::time::Instant::now();
    while runs_the_helper(helper) && ended_by.elapsed() < THE_GROUP_ENDS_WITHIN {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let survived: bool = runs_the_helper(helper);
    if survived {
        // SAFETY: `kill` takes a process id and a signal number and touches no memory; this control started that process.
        let _ended = unsafe { libc::kill(i32::try_from(helper).unwrap_or(i32::MAX), libc::SIGKILL) };
    }
    assert!(
        !survived,
        "the helper the child started (pid {helper}) was still running after the child that owned it was dropped, and the child led no process group"
    );
    remove(root.as_path());
}

/// How long the helper the child starts would live if nothing ended it:
/// far longer than the control waits, so one that is gone is one something ended.
const THE_HELPERS_OWN_SLEEP_SECONDS: u64 = 300;

/// The longest the control gives the child's tree to end and to be reaped:
/// the grace the normal ceiling path gives a killed child (`STOP_GRACE`) and the margin a loaded machine needs.
const THE_GROUP_ENDS_WITHIN: std::time::Duration = crate::bounded_push::STOP_GRACE.saturating_add(std::time::Duration::from_secs(5));

/// Whether the process `pid` is running: a zombie holds no files and runs nothing, so it counts as ended.
fn runs_the_helper(pid: u32) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat").as_str()) else { return false };
    let Some(after_the_name) = stat.rfind(')') else { return true };
    let state: Option<char> = stat[after_the_name + 1..].trim_start().chars().next();
    return state != Some('Z');
}
