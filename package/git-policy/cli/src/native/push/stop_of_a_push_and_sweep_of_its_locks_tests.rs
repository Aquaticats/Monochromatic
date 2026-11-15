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
    // The child ends after the sweep gave up on it. A round collects it, here or in another test of this process, so what is
    //  asked is the state a later round leaves, not which round did it.
    std::thread::sleep(Duration::from_millis(400));
    let table: std::path::PathBuf = std::path::PathBuf::from(format!("/proc/{id}"));
    let collected: usize = reap_abandoned_children();
    assert!(
        !table.exists(),
        "the child the sweep left was collected by a round instead of staying a zombie ({collected} were collected here)"
    );
}

//region The whole stop is bounded by the ceiling of a stop

/// Every member counts as one that cannot be interrupted,
/// so the stop under the control waits its ceiling without a process of this machine having to sit in a disk call.
fn held_for_ever(_member: &TreeMember) -> bool {
    return true;
}

/// What:
///  A member the kernel does not release after the stop kills it does not hold the stop past its ceiling.
/// Why:
///   A process of a stopped push can sit in an uninterruptible call for longer than any stop may take,
/// a file system writing back or a
///  stalled disk,
/// and the kernel then releases it whenever that call returns,
/// never on the signal.
/// The stop waits its ceiling for it,
/// and a stop that waited a
///  second time after the kill took the ceiling twice from the person's side:
/// the gate of 2026-11-14 showed a push whose stop reported
///  `push_ms=30443` where the connected window ends the push at 7 seconds and a stop of a push otherwise takes about a tenth of
///  a second (`/home/user/temp/agent/after-commit-work-settles-push-fail-pnpm-fail-2026-11-14/keep/report-2-push-minimum-rate.md`).
/// The member here is one the kernel does not release:
/// `strace` traces it and its tracer is stopped,
/// so the `SIGKILL` that
///  ends it leaves it held until its tracer runs again,
/// which is the wait a process in an uninterruptible call leaves behind it.
/// The ceiling and the grace are the control's own,
/// as in
///  `a_member_held_for_ever_is_ended_at_the_ceiling_and_its_lock_is_left`,
/// so this control runs in about a second instead of eighteen.
#[test]
fn a_member_the_kernel_does_not_release_does_not_hold_the_stop_past_its_ceiling() {
    const CEILING: Duration = Duration::from_millis(1000);
    const GRACE: Duration = Duration::from_millis(300);
    // What a stop that ends within its ceiling may take past it: the poll it ends on and the kill.
    const SLACK: Duration = Duration::from_millis(200);
    // How long the control waits for its tracer to reach the member, and for the member to be released afterwards.
    const REACH_BOUND: Duration = Duration::from_secs(10);
    // The wait the sweep task gives this member's own child, so the task is not held for the shipped ceiling.
    const SWEEP_CEILING: Duration = Duration::from_millis(100);
    let _rules = crate::stop_waits_while_a_member_cannot_be_interrupted::rules_for_this_thread(
        crate::stop_waits_while_a_member_cannot_be_interrupted::ControlRules { blocked: held_for_ever, ceiling: CEILING },
    );
    let child: std::process::Child = std::process::Command::new("sleep").arg("30").spawn().expect("a child to stop");
    let id: u32 = child.id();
    let number: i32 = i32::try_from(id).expect("a process number");
    let member: TreeMember = member_of(number).expect("the child is a member of its own tree");
    // The tracer attaches with a stop of its tracee; once it is stopped in turn, a killing signal leaves the tracee held in
    //  `ptrace_stop` until the tracer runs, which is the state the control needs and cannot reach another way.
    let mut tracer: std::process::Child = std::process::Command::new("strace")
        .args(["--attach", number.to_string().as_str(), "--output", "/dev/null"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("a tracer of the member");
    let tracer_number: i32 = i32::try_from(tracer.id()).expect("a process number");
    let reached: Instant = Instant::now();
    while !traced_by(number, tracer_number) {
        assert!(reached.elapsed() < REACH_BOUND, "the tracer attached to the member it was started for");
        std::thread::sleep(Duration::from_millis(10));
    }
    // SAFETY: `kill` takes no pointers and writes no memory; the number is this test's own child, which has not been reaped.
    unsafe { libc::kill(tracer_number, libc::SIGSTOP) };
    ceiling_for_a_child_in_a_control(id, SWEEP_CEILING);
    let began: Instant = Instant::now();
    crate::lazy_runtime::run_on_runtime(stop_push(adopt(child), GRACE, LockLedger::none())).expect("the runtime runs the stop");
    let took: Duration = began.elapsed();
    assert!(took >= CEILING, "the stop gave a member that cannot be interrupted its whole ceiling: {took:?}");
    assert!(
        took < CEILING + SLACK,
        "the stop ended within its ceiling instead of waiting for the member it killed once more: {took:?}"
    );
    assert!(still_running(&member), "the member the kernel held was still there when the stop ended: {took:?}");
    // The member is released and collected here, so this control leaves no process of its own behind.
    // SAFETY: as above; the number is this test's own tracer, which it kills and then reaps.
    unsafe { libc::kill(tracer_number, libc::SIGKILL) };
    let _strace_ended: Option<std::process::ExitStatus> = tracer.wait().ok();
    let releasing: Instant = Instant::now();
    while member_of(number).is_some() {
        assert!(releasing.elapsed() < REACH_BOUND, "the member the kernel held was released once its tracer ended");
        std::thread::sleep(Duration::from_millis(10));
    }
    let _collected: usize = reap_abandoned_children();
}

/// Whether `tracer` is the process tracing `pid`,
/// read from the member's own record.
/// Why:
///   `strace --attach` answers once it is attached,
/// and a control that measured before that would hold nothing:
/// the member's record names the
///  tracer while the trace lasts.
fn traced_by(pid: i32, tracer: i32) -> bool {
    let Ok(status) = std::fs::read_to_string(format!("/proc/{pid}/status")) else {
        return false;
    };
    return status.lines().any(|line| { return line == format!("TracerPid:\t{tracer}"); });
}

//endregion
