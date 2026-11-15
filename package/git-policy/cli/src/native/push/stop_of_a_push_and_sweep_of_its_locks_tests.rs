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
/// so this control runs in about a second instead of eighteen;
/// what the waiting for the held member may use is the ceiling less the parts kept back for the reap and the sweep,
/// and the
///  stop's own answer still lies inside the ceiling.
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
    // The part of the ceiling the waiting for a held member may use: the ceiling less the parts the stop keeps for the reap
    //  it needs after the kill and for the sweep of the locks it leaves.
    let escalation_part: Duration = CEILING.saturating_sub(REAP_AFTER_THE_KILL).saturating_sub(SWEEP_AFTER_THE_STOP);
    assert!(
        took >= escalation_part,
        "the stop gave a member that cannot be interrupted the whole of the bound its escalation may use ({escalation_part:?}): {took:?}"
    );
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

//region A stop keeps its child and its ledger whatever the runtime does

/// What:
///  Whether `id` is a zombie of this process in the next `bound`.
/// Why:
///  A child whose handle was dropped is not reaped when it ends,
/// so it stays in the process table as a zombie for the life of the
///  process,
/// and that is what the two findings of the review of `374f34c4b9` left behind.
fn is_zombie(id: u32, bound: Duration) -> bool {
    let watching: Instant = Instant::now();
    while watching.elapsed() < bound {
        match std::fs::read_to_string(format!("/proc/{id}/stat")) {
            Ok(stat) => {
                let after_name: &str = stat.rfind(')').and_then(|at| { return stat.get(at + 2..); }).unwrap_or_default();
                if after_name.starts_with("Z ") {
                    return true;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return false,
            Err(_) => {}
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    return false;
}

/// What:
///  A ledger over one disposable Git directory with an orphan staging file,
/// whose judgement looks at an empty process table of its own.
/// Why:
///   The sweep's own removal is what a control must see,
/// and the real judgement refuses to decide from the processes of the whole
///  machine (a Git of another account vetoes it),
/// so the control gives it a table with nothing in it.
fn ledger_over_an_orphan_staging_file(root: &std::path::Path) -> (LockLedger, std::path::PathBuf) {
    let git_dir: std::path::PathBuf = root.join("git-dir");
    let proc_root: std::path::PathBuf = root.join("proc");
    std::fs::create_dir_all(git_dir.as_path()).expect("a disposable Git directory");
    std::fs::create_dir_all(proc_root.as_path()).expect("an empty disposable process table");
    let staging: std::path::PathBuf = git_dir.join("packed-refs.new");
    std::fs::write(staging.as_path(), b"an orphan staging file of a killed Git").expect("the staging file");
    let mut command: std::process::Command = std::process::Command::new("true");
    command.env("GIT_DIR", git_dir.as_os_str());
    let judge: crate::stale_ref_lock_judge::Judge = crate::stale_ref_lock_judge::Judge {
        proc_root,
        margin: Duration::ZERO,
        boot: None,
        local_file_systems: ["btrfs", "ext4", "xfs", "f2fs", "tmpfs", "overlay", "zfs", "bcachefs"].into_iter().map(String::from).collect(),
        scan_scope: Some(Vec::new()),
    };
    return (LockLedger::before_start(&command).with_judge(judge), staging);
}

/// What:
///  A stop whose bound is spent and whose runtime has no worker still reaps its child and still judges the locks it left.
/// Why:
///   The last handle of the child and the ledger used to travel into the sweeping task's future,
/// and `spawn_on_runtime`
///  drops that future where the runtime has no worker,
/// which a thread or a memory limit leaves (finding 1 of the review of
///  `374f34c4b9`:
/// `zombie=true, staging_left=true, abandoned_round_collected=0`).
/// Both are kept by the stop
///  itself here:
/// the child is reaped out of the part of the bound kept for the reap,
/// and the sweep runs inline out of the part
///  kept for it,
/// so neither depends on a task the runtime may refuse.
#[test]
fn a_stop_with_no_worker_still_reaps_its_child_and_judges_the_locks_it_left() {
    const CEILING: Duration = Duration::from_millis(100);
    const GRACE: Duration = Duration::from_millis(25);
    const SLACK: Duration = Duration::from_millis(400);
    const ZOMBIE_BOUND: Duration = Duration::from_secs(10);
    let _rules = crate::stop_waits_while_a_member_cannot_be_interrupted::rules_for_this_thread(
        crate::stop_waits_while_a_member_cannot_be_interrupted::ControlRules { blocked: held_for_ever, ceiling: CEILING },
    );
    // What a thread or a memory limit leaves: a runtime whose tasks cannot run.
    let _no_worker = crate::lazy_runtime::a_control_refuses_every_task();
    let root: std::path::PathBuf = crate::test_support::fixture("stop-with-no-worker");
    let (locks, staging): (LockLedger, std::path::PathBuf) = ledger_over_an_orphan_staging_file(root.as_path());
    let owed_before: usize = sweeps_owed();
    let child: std::process::Child = std::process::Command::new("sh").args(["-c", "trap '' TERM; exec sleep 30"]).spawn().expect("a child that ignores the polite stop");
    let id: u32 = child.id();
    let began: Instant = Instant::now();
    crate::lazy_runtime::run_on_runtime(stop_push(adopt(child), GRACE, locks)).expect("the runtime runs the stop");
    let took: Duration = began.elapsed();
    assert!(took < CEILING + SLACK, "the stop answered inside its bound: {took:?}");
    assert!(!is_zombie(id, ZOMBIE_BOUND), "the child of a stop with no worker was reaped, not left a zombie for the life of this process");
    assert!(!staging.exists(), "the sweep ran inside the stop without a task of the runtime");
    assert_eq!(sweeps_owed(), owed_before, "and no sweep was recorded as owed, because none was");
    let _collected: usize = reap_abandoned_children();
    crate::test_support::remove(root.as_path());
}

/// What:
///  A stop answers inside its bound while the blocking pool cannot run the sweep,
/// and the sweep it could not run is recorded as owed.
/// Why:
///   The inline sweep read `run_blocking_job`'s answer with no bound of its own,
/// so a pool held by jobs that have not returned held the stop
///  past its ceiling for as long as that job lasted
/// (finding 2 of the review of `374f34c4b9`:
/// `stop_push` was still waiting
///  at 301 ms under a ceiling of 100 ms).
/// The part of the bound kept for the sweep is what its answer is awaited for,
/// and what does not run
///  inside it is said to be owed (`sweeps_owed`),
/// because the judgement before the next fetch or push clears what it left.
#[test]
fn a_stop_answers_inside_its_bound_while_the_blocking_pool_cannot_sweep() {
    const CEILING: Duration = Duration::from_millis(100);
    const GRACE: Duration = Duration::from_millis(25);
    const OUTER_BOUND: Duration = Duration::from_millis(300);
    const BLOCKER_BOUND: Duration = Duration::from_secs(10);
    crate::lazy_runtime::cap_workers(1);
    let _rules = crate::stop_waits_while_a_member_cannot_be_interrupted::rules_for_this_thread(
        crate::stop_waits_while_a_member_cannot_be_interrupted::ControlRules { blocked: held_for_ever, ceiling: CEILING },
    );
    let (holding_send, holding_receive) = std::sync::mpsc::channel();
    let (release_send, release_receive) = std::sync::mpsc::channel();
    let holder = crate::lazy_runtime::run_on_runtime(async move {
        return tokio::task::spawn_blocking(move || {
            holding_send.send(()).expect("the control waits for the holder");
            let _held = release_receive.recv_timeout(BLOCKER_BOUND);
        });
    })
    .expect("the runtime starts the job that holds the pool");
    holding_receive.recv_timeout(BLOCKER_BOUND).expect("the only blocking thread is held");
    let owed_before: usize = sweeps_owed();
    // The child answers the polite stop, so the stop reaches its sweep with the whole of its bound still to spend.
    let child: std::process::Child = std::process::Command::new("sleep").arg("30").spawn().expect("a child that accepts the polite stop");
    let began: Instant = Instant::now();
    let answered = crate::lazy_runtime::run_on_runtime(async {
        return tokio::time::timeout(OUTER_BOUND, stop_push(adopt(child), GRACE, LockLedger::none())).await;
    })
    .expect("the runtime runs the bounded outer control");
    let took: Duration = began.elapsed();
    let owed_after: usize = sweeps_owed();
    release_send.send(()).expect("the holder still waits");
    crate::lazy_runtime::run_on_runtime(async {
        let _joined = tokio::time::timeout(BLOCKER_BOUND, holder).await;
    })
    .expect("the runtime releases the holder");
    assert!(answered.is_ok(), "the stop answered inside its bound of {CEILING:?} and did not wait for the queued sweep: {took:?}");
    assert!(owed_after > owed_before, "the sweep the pool could not run is recorded as owed ({owed_before} before, {owed_after} after)");
}

//endregion

//region A place that comes late is inside the same bound

/// What:
///  A place of the blocking budget that comes late leaves the sweep the rest of the stop's bound, so the stop answers
///  inside it.
/// Why:
///   The inline sweep is awaited for what is left of `STOP_CEILING`,
/// and the helper that awaited it spent that duration
///  twice once,
/// a place of the budget and then the job,
/// so a place that came late let the stop pass its ceiling: the review
///  of `2da23e4478` measured `1.351937327 s` under a ceiling of one second.
/// The place is freed by the ceiling of the job that holds it and the only blocking thread is held for the whole stop,
/// so what the control catches is the doubled wait and not the machine's lateness.
#[test]
fn a_place_that_comes_late_does_not_let_the_stop_pass_its_ceiling() {
    const CEILING: Duration = Duration::from_millis(1000);
    const GRACE: Duration = Duration::from_millis(25);
    // The place the sweep waits for is freed by the ceiling of the job that holds it.
    const THE_PLACE_IS_FREED_AFTER: Duration = Duration::from_millis(400);
    // What a stop that ends by its ceiling may take past it, as in the control beside this one.
    const SLACK: Duration = Duration::from_millis(250);
    const HOLD_BOUND: Duration = Duration::from_secs(20);
    // The places the control gives the process: one for the held thread and one for the place that comes late.
    const THE_PLACES: usize = 2;
    crate::lazy_runtime::cap_workers(1);
    crate::blocking_jobs_within_the_budget::hold_blocking_jobs_to(THE_PLACES);
    let _rules = crate::stop_waits_while_a_member_cannot_be_interrupted::rules_for_this_thread(
        crate::stop_waits_while_a_member_cannot_be_interrupted::ControlRules { blocked: held_for_ever, ceiling: CEILING },
    );
    let (holding, held) = std::sync::mpsc::channel::<()>();
    let (release, releasing) = std::sync::mpsc::channel::<()>();
    let thread_holder: tokio::task::JoinHandle<crate::blocking_jobs_within_the_budget::TheAnswerOfAJobAwaited<()>> = crate::lazy_runtime::spawn_on_runtime(async move {
        return crate::blocking_jobs_within_the_budget::run_a_job_once_awaited_no_longer_than(HOLD_BOUND, move || {
            let _told = holding.send(());
            let _ = releasing.recv_timeout(HOLD_BOUND);
        })
        .await;
    })
    .expect("the runtime has a worker, so the only blocking thread can be held");
    assert!(held.recv_timeout(HOLD_BOUND).is_ok(), "the only blocking thread is held before the stop asks for a place");
    let place_holder: tokio::task::JoinHandle<crate::blocking_jobs_within_the_budget::TheAnswerOfAJobAwaited<()>> = crate::lazy_runtime::spawn_on_runtime(async move {
        return crate::blocking_jobs_within_the_budget::run_a_job_once_awaited_no_longer_than(THE_PLACE_IS_FREED_AFTER, || {
            panic!("the queued job must not reach the held thread");
        })
        .await;
    })
    .expect("the runtime has a worker, so the second place can be held");
    // The place is taken at the first poll of the job that asks for it, so the control waits on the place itself:
    //  a stop that asked before that poll would find the place free and test nothing.
    let asking: Instant = Instant::now();
    while crate::blocking_jobs_within_the_budget::free_places_in_this_control() > 0 {
        assert!(asking.elapsed() < HOLD_BOUND, "the second place is taken before the stop asks for one");
        std::thread::sleep(Duration::from_millis(1));
    }
    // The child answers the polite stop, so the stop reaches its sweep with the whole of its bound still to spend.
    let child: std::process::Child = std::process::Command::new("sleep").arg("30").spawn().expect("a child that accepts the polite stop");
    let owed_before: usize = sweeps_owed();
    let began: Instant = Instant::now();
    let answered = crate::lazy_runtime::run_on_runtime(async {
        return tokio::time::timeout(HOLD_BOUND, stop_push(adopt(child), GRACE, LockLedger::none())).await;
    })
    .expect("the runtime runs the bounded outer control");
    let took: Duration = began.elapsed();
    let owed_after: usize = sweeps_owed();
    let _let_go = release.send(());
    crate::lazy_runtime::run_on_runtime(async {
        let _thread: Result<_, _> = tokio::time::timeout(HOLD_BOUND, thread_holder).await;
        let _place: Result<_, _> = tokio::time::timeout(HOLD_BOUND, place_holder).await;
    })
    .expect("the runtime releases both holders");
    assert!(answered.is_ok(), "the stop answered, and the control's own bound is not what it met: {took:?}");
    assert!(
        took < CEILING + SLACK,
        "the stop spent what was left of its bound once waiting for a place and again waiting for the job: {took:?}, ceiling {CEILING:?}"
    );
    assert!(owed_after > owed_before, "the sweep the pool could not run is recorded as owed ({owed_before} before, {owed_after} after)");
}

//endregion
