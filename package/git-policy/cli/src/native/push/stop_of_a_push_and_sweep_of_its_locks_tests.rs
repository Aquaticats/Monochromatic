//! Controls for the sweep after a stopped push:
//! its wait for the rest of the tree ends at the ceiling of a stop.

use super::*;

/// The sweep's wait for the rest of the tree ends at the ceiling of a stop,
/// so a descendant that never ends leaves no task waking for the life of the service.
#[test]
fn the_sweep_waits_for_the_tree_no_longer_than_the_ceiling_of_a_stop() {
    let running: Vec<TreeMember> = i32::try_from(std::process::id()).ok().and_then(member_of).into_iter().collect();
    assert!(!running.is_empty(), "this process is a member of its own tree");
    let never: bool = crate::lazy_runtime::run_on_runtime(wait_until_gone(running.as_slice(), Duration::ZERO)).expect("the runtime runs the wait");
    assert!(!never, "a tree that keeps running is left at the ceiling instead of being waited for for ever");
    // The control: the same wait answers at once for a tree that is gone, so the ceiling above is what ended it.
    let at_once: bool = crate::lazy_runtime::run_on_runtime(wait_until_gone(&[], Duration::ZERO)).expect("the runtime runs the wait");
    assert!(at_once, "a tree that is already gone ends the wait");
}

/// What:
///  The sweep's wait for the child it was handed ends at the ceiling of a stop too,
/// and the child is left to the kernel
///  instead of holding the task.
/// Why:
///   A killed push in uninterruptible I/O is never reaped by the kernel until it is released,
/// so a task that
///  awaited it for as long as that takes is a task that may never end,
/// and the locks of the stopped push would never be
///  swept.
/// The fixture ignores the polite stop and sleeps far past the control's ceiling.
#[test]
fn the_sweep_gives_up_on_a_child_the_kernel_does_not_release() {
    let child: std::process::Child = std::process::Command::new("sh").arg("-c").arg("exec sleep 20").spawn().expect("a child to hand over");
    let id: u32 = child.id();
    let ceiling: Duration = Duration::from_millis(300);
    ceiling_for_a_child_in_a_control(id, ceiling);
    let locks: LockLedger = LockLedger::before_start(&std::process::Command::new("true"));
    let sweeping: tokio::task::JoinHandle<()> = sweep_after_exit(child, Vec::new(), locks).expect("the runtime has a worker to sweep with");
    let began: std::time::Instant = std::time::Instant::now();
    let ended: Result<(), tokio::time::error::Elapsed> = crate::lazy_runtime::run_on_runtime(async move {
        return tokio::time::timeout(Duration::from_secs(10), sweeping).await.map(|_joined| {});
    })
    .expect("the runtime runs the wait");
    let waited: Duration = began.elapsed();
    assert!(ended.is_ok(), "the sweep ended instead of waiting for a child the kernel holds: {waited:?}");
    assert!(waited >= ceiling, "it gave the child its ceiling: {waited:?}");
    // The child the sweep left is this test's own and is ended here; a failure means it already ended.
    // SAFETY: `kill` takes no pointers, and the number is this test's own child, which this test has not reaped.
    unsafe { libc::kill(i32::try_from(id).unwrap_or(0), libc::SIGKILL) };
}

/// What:
///  A child a sweep could not collect is kept,
/// and the next round collects it once the kernel has released it.
/// Why:
///   Dropping the handle reaps nothing:
/// a child that exits after its stop was reported stays a zombie for the life of
///  the service,
/// so the handle is kept and a later round takes the exit with a non-blocking wait.
#[test]
fn a_child_left_behind_is_collected_by_a_later_round() {
    let child: std::process::Child = std::process::Command::new("sh").arg("-c").arg("exec sleep 0.2").spawn().expect("a child to hand over");
    let id: u32 = child.id();
    ceiling_for_a_child_in_a_control(id, Duration::from_millis(50));
    let locks: LockLedger = LockLedger::before_start(&std::process::Command::new("true"));
    let sweeping: tokio::task::JoinHandle<()> = sweep_after_exit(child, Vec::new(), locks).expect("the runtime has a worker to sweep with");
    crate::lazy_runtime::run_on_runtime(async move {
        let _ended: Result<Result<(), tokio::task::JoinError>, tokio::time::error::Elapsed> = tokio::time::timeout(Duration::from_secs(10), sweeping).await;
    })
    .expect("the runtime runs the wait");
    // The child ends after the sweep gave up on it, and nothing has collected it: it is a zombie now.
    std::thread::sleep(Duration::from_millis(400));
    let table: std::path::PathBuf = std::path::PathBuf::from(format!("/proc/{id}"));
    assert!(table.exists(), "the child ended and is still in the process table");
    let collected: usize = reap_abandoned_children();
    assert!(collected >= 1, "a later round collected the child the sweep left");
    assert!(!table.exists(), "so it is no zombie after that");
}
