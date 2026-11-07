//! The child helper: a child started with the standard library is awaited without a thread, owned through every failure after
//! its start, and reaped or handed over, never reported as a failed start.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A shell child that sleeps `seconds` and exits with `code`. A shell is the one program every host has.
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

/// Every way the helper can fail after the start, plus none.
const ALL_FAILURES: [InjectedFailure; 3] = [InjectedFailure::None, InjectedFailure::PidfdOpen, InjectedFailure::Registration];

/// A child that exits is awaited and its status comes back, whatever failed after its start.
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

/// After a failure the helper still owns a live child: it was not reported failed, it is not lost, and it can be reaped.
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

/// Dropping an owned live child ends and reaps it, so the caller's cleanup never runs beside a live child.
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

/// Waiting uses no thread of its own: many children are awaited at once on a current-thread runtime, in about the time of one.
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
