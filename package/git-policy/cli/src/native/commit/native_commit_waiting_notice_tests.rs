//! Controls for the waiting notice of the commit Git builds:
//! what the line names,
//! when the commit counts as signed,
//! and that a
//!  commit that opens an editor,
//! or that ends at once,
//! prints nothing.

use super::*;
use crate::native_commit_waiting_notice_watch::LOOK_INTERVAL;
use crate::process_tree::ProcessUse;
use crate::scratch_step_ceiling_stop::{
    CeilingCame,
    ceiling_came,
};
use crate::scratch_step_look_below::look_below;
use crate::scratch_step_programs_below::{
    SeenBelow,
    credit_between_looks,
    seen_below,
};

/// The words of a command line as the notice reads them.
fn words(text: &[&str]) -> Vec<OsString> {
    return text.iter().map(OsString::from).collect();
}

/// A shared buffer the notice thread writes into.
#[derive(Clone, Default)]
struct Buffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.0.lock().expect("buffer").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The text the buffer holds.
fn text_of(buffer: &Buffer) -> String {
    return String::from_utf8_lossy(buffer.0.lock().expect("buffer").as_slice()).into_owned();
}

/// What:
///  `take_the_windows_a_notice_control_reads()` takes the two windows a notice control reads,
///  in the one order that cannot deadlock a threaded run:
///  the look's window first,
///  then the notice's.
/// Why:
///  Both are one window for the whole process,
///  so a control that plants a ceiling or a look that cannot be made changes the answer of every thread of the run,
///  and a control that reads them without taking them fails by which control ran first (issue 6509):
///  measured,
///  `the_notice_reports_the_programs_that_ran_below_the_step` credited one 250 ms interval of a step it had watched for 1.2 s,
///  because `commit_preparation_tests` planted blind looks while this control's notice looked.
///  Two orders in one process hold each other for ever in a threaded run
///  (`doc/troubleshooting/git-policy-cli-control-windows-taken-in-two-orders.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeTheWindowsANoticeControlReads() { const looks = takeTheLookWindow(); const notice = takeTheNoticeWindow(); return [looks, notice]; }
/// ```
fn take_the_windows_a_notice_control_reads() -> (
    std::sync::MutexGuard<'static, ()>,
    std::sync::MutexGuard<'static, ()>,
) {
    let looks: std::sync::MutexGuard<'static, ()> =
        crate::scratch_step_look_below::the_controls_window::take_the_window();
    let notice: std::sync::MutexGuard<'static, ()> = the_controls_window::take_the_window();
    // The look's guard is held by the answer, so it is let go only when the answer is.
    return (looks, notice);
}

/// The command line can ask for a signature,
/// and `--no-gpg-sign` takes the request back;
/// the arguments after `--` are paths.
#[test]
fn the_command_line_can_ask_for_a_signature_and_take_it_back() {
    assert!(signs_commit(None, &words(&["-S", "-m", "x"])));
    assert!(signs_commit(None, &words(&["--gpg-sign=KEY", "-m", "x"])));
    assert!(signs_commit(None, &words(&["-SKEY", "-m", "x"])));
    assert!(!signs_commit(
        None,
        &words(&["-S", "--no-gpg-sign", "-m", "x"])
    ));
    assert!(!signs_commit(None, &words(&["-m", "x", "--", "-S"])));
    assert!(!signs_commit(None, &words(&["-m", "x"])));
}

/// A step that ends before the first notice is due prints nothing,
/// and one that lasts prints at the interval.
#[test]
fn a_step_that_ends_at_once_prints_nothing_and_a_long_one_prints_again() {
    let quick: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_millis(400),
        interval: Duration::from_millis(400),
    };
    let quick_notice: NativeCommitNotice =
        NativeCommitNotice::start(None, &words(&["-m", "x"]), cadence, Box::new(quick.clone()));
    quick_notice.finish();
    assert_eq!(text_of(&quick), "");
    let slow: Buffer = Buffer::default();
    let notice: NativeCommitNotice =
        NativeCommitNotice::start(None, &words(&["-m", "x"]), cadence, Box::new(slow.clone()));
    std::thread::sleep(Duration::from_millis(1300));
    notice.finish();
    let said: String = text_of(&slow);
    assert!(said.lines().count() >= 2, "{said}");
    assert_eq!(
        said.matches("Hint").count(),
        1,
        "only the first line carries the hint: {said}"
    );
}

/// A commit that opens the person's editor prints nothing however long it lasts.
#[test]
fn a_commit_that_opens_an_editor_prints_nothing() {
    let quiet: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_millis(50),
        interval: Duration::from_millis(50),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start(
        None,
        &words(&["-m", "x", "--edit"]),
        cadence,
        Box::new(quiet.clone()),
    );
    std::thread::sleep(Duration::from_millis(300));
    notice.finish();
    assert_eq!(text_of(&quiet), "");
}

/// A sleeping child of this test process,
/// which stands for the `git commit` of the step,
/// started by `test_support::spawn_owned` so the test thread's fixtures end it however the test ends.
fn sleeping_step() -> std::process::Child {
    return crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("120"))
        .expect("a long-running child");
}

/// The number of a child,
/// as the commit path tells it to the notice once the step has started.
fn step_number(child: &std::process::Child) -> i32 {
    return i32::try_from(child.id()).expect("a process number");
}

/// The signal that ended a child,
/// waiting at most a few seconds so a child that was never stopped fails the control instead of hanging it.
fn ended_by_signal(child: &mut std::process::Child) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    let give_up: Instant = Instant::now() + Duration::from_secs(10);
    while Instant::now() < give_up {
        if let Some(status) = child.try_wait().expect("status") {
            return status.signal();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return None;
}

/// A commit whose message came with the command is stopped at its ceiling:
/// the `git commit` below this process is ended,
/// and the notice says it did so.
#[test]
fn a_commit_whose_message_came_with_the_command_is_stopped_at_its_ceiling() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(sink.clone()),
    });
    notice.step_started(step_number(&step));
    let ended: Option<i32> = ended_by_signal(&mut step);
    let seen: ScratchStepFacts = notice.finish();
    if ended.is_none() {
        step.kill().expect("cleanup");
    }
    assert!(
        ended.is_some(),
        "the step was still running after its ceiling"
    );
    let stopped: Option<Duration> = seen.stopped_after;
    assert!(
        stopped.is_some_and(|after| {
            return after >= Duration::from_millis(300);
        }),
        "{stopped:?}"
    );
    assert!(seen.armed, "a message on the command line arms the ceiling");
    assert!(
        seen.looks >= 1 && seen.looks_found >= 1,
        "the notice looked and found the step: {seen:?}"
    );
    // A `sleep` acts on the signal at once, so no look of this stop finds it held, and the notice says nothing of a kernel holding it.
    let said: String = text_of(&sink);
    assert!(
        !said.contains("has not released"),
        "a step that acted on its stop is said to be held by the kernel: {said}"
    );
}

/// A commit that may open the person's editor has no ceiling:
/// the person is the wait,
/// and a message typed into an editor would be lost by a stop.
#[test]
fn a_commit_that_may_open_an_editor_is_never_stopped() {
    let mut step: std::process::Child = sleeping_step();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x", "--edit"]),
        cadence,
        ceiling: Duration::from_millis(100),
        sink: Box::new(Buffer::default()),
    });
    std::thread::sleep(Duration::from_millis(700));
    let still_running: bool = step.try_wait().expect("status").is_none();
    let stopped: Option<Duration> = notice.finish().stopped_after;
    step.kill().expect("cleanup");
    step.wait().expect("reaped");
    assert!(still_running, "an editor commit was stopped");
    assert_eq!(stopped, None);
}

/// A step that has ended when the ceiling comes is not stopped,
/// and so reports no stop:
/// a commit that landed is never turned into a failure.
#[test]
fn a_step_that_ended_before_its_ceiling_reports_no_stop() {
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(100),
        sink: Box::new(Buffer::default()),
    });
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(notice.finish().stopped_after, None);
}

/// What:
///  A step that was not there yet when the ceiling's look happened is still stopped once it runs.
/// Why:
///   A start the host held up (a full process table,
/// memory pressure,
/// a scheduling stall) puts the process below this one after the ceiling has passed,
/// and a look that finds nothing is then a statement about the start,
/// not about the step.
///  A commit whose step ran 36,651 ms landed under a 3,006 ms ceiling on 2026-11-08,
/// and the record could not say which of the two it was;
/// this control holds the reading that a missing process never takes the ceiling away.
#[test]
fn a_step_that_appeared_after_the_ceiling_look_is_still_stopped() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(200),
        sink: Box::new(Buffer::default()),
    });
    // The start is held up past the ceiling, as a start on a host with no process to spare is.
    std::thread::sleep(Duration::from_millis(400));
    let mut step: std::process::Child = sleeping_step();
    notice.step_started(step_number(&step));
    let ended: Option<i32> = ended_by_signal(&mut step);
    let stopped: Option<Duration> = notice.finish().stopped_after;
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert!(
        ended.is_some(),
        "a step that started after the ceiling's look ran on with no ceiling and was never stopped"
    );
    assert!(
        stopped.is_some_and(|after| {
            return after >= Duration::from_millis(200);
        }),
        "{stopped:?}"
    );
}

/// The entry the notice kept for the program `name`,
/// or a failure that names what it kept instead.
fn kept_below<'kept>(
    below: &'kept [crate::scratch_commit_step::SeenProgram],
    name: &str,
) -> &'kept crate::scratch_commit_step::SeenProgram {
    for program in below {
        if program.name == name {
            return program;
        }
    }
    panic!("the notice kept no entry for `{name}`: {below:?}");
}

/// What:
///  The stretch a host under load put between two looks of the notice,
///  which this control states so its figures are asked for under the schedule that failed a gate.
/// Why:
///  The gate run of 2026-11-27 that failed this control held `looks: 4, looks_found: 3` and credited the
///  `sleep` with 639.46873 ms, which is two intervals of 319.734 ms each (issue 6592).
///  A look interval the host stretched to 319.7 ms is the one this control means,
///  and the stretch planted is longer than the 69.7 ms load added:
///  the figures this control asserted before the fix needed a look that the 1,200 ms window held only while
///  the notice waited exactly `LOOK_INTERVAL`, so the look is lost on every run between 350 ms and 500 ms
///  and by which thread woke first outside that, and a control that states it is never wrong for the race.
const A_STRETCH_A_LOADED_HOST_PUT_BETWEEN_TWO_LOOKS: Duration = Duration::from_millis(450);

/// What:
///  The notice reports which programs ran below the step and for how long.
/// Why:
///   Git starts the signing program below itself and the record held no figure for it,
/// so a commit whose step ran 36,651 ms on 2026-11-08 could not be told from one whose signing program waited,
/// and the owner asked for the logging that tells them apart (2026-11-08,
/// "About 16 seconds").
///  The child here is a `sleep` below this process,
/// named as the kernel names it.
#[test]
fn the_notice_reports_the_programs_that_ran_below_the_step() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    let mut step: std::process::Child =
        crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("2"))
            .expect("a long-running child");
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_secs(120),
        sink: Box::new(Buffer::default()),
    });
    notice.step_started(step_number(&step));
    // What: The stretch a loaded host put between this notice's looks, planted for the step it watches.
    // Why:  The gate's own figures show each look interval stretched to 319.7 ms (issue 6592),
    //       and this control states that schedule instead of waiting for a load to happen.
    the_controls_window::a_look_of_this_step_waits(
        step_number(&step),
        A_STRETCH_A_LOADED_HOST_PUT_BETWEEN_TWO_LOOKS,
    );
    std::thread::sleep(Duration::from_millis(1200));
    let seen: ScratchStepFacts = notice.finish();
    step.kill().expect("cleanup");
    step.wait().expect("reaped");
    assert!(
        seen.armed && seen.looks_found >= 1,
        "the notice looked and found the step: {seen:?}"
    );
    assert_eq!(seen.stopped_after, None, "the ceiling never came: {seen:?}");
    let found: &crate::scratch_commit_step::SeenProgram =
        kept_below(seen.below.as_slice(), "sleep");
    assert!(
        found.seen >= Duration::from_millis(750),
        "the sleep ran while the notice looked: {seen:?}"
    );
    assert!(
        found.seen <= Duration::from_millis(1200),
        "a program cannot be seen longer than it ran: {seen:?}"
    );
}

/// One process a look saw below the step,
/// as `look_below` reports it.
fn seen(
    pid: i32,
    started: u64,
    name: &str,
) -> (i32, ProcessUse) {
    return (
        pid,
        ProcessUse {
            name: String::from(name),
            started,
            state: 'S',
            cpu: Duration::ZERO,
        },
    );
}

/// What:
///  A program's time is credited only when the same process was below the step at both looks.
/// Why:
///   Two programs can carry one name:
/// a signing program can end after one look and another of the same name start before the next,
/// and a name alone then credits the interval to a program that was not there for any of it.
///  The two looks here name the same program and hold two processes,
/// so a name match credits the interval and an identity match credits nothing.
#[test]
fn the_same_program_must_be_seen_at_both_looks_to_be_credited() {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(Duration::from_secs(60)));
    let before: Vec<SeenBelow> = seen_below(&[seen(100, 5, "ssh-keygen")]);
    let now: Vec<(i32, ProcessUse)> = vec![seen(200, 9, "ssh-keygen")];
    credit_between_looks(
        &mut facts,
        before.as_slice(),
        now.as_slice(),
        Duration::from_millis(250),
    );
    let kept: &crate::scratch_commit_step::SeenProgram =
        kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(
        kept.seen,
        Duration::ZERO,
        "a program that was not below the step at the look before it was credited with time no process ran: {facts:?}"
    );
}

/// What:
///  A program that takes the place of another between two looks is credited only from its own look.
/// Why:
///   Finding 7 of the second review:
/// a process number and its start time survive an `exec`,
/// so a shell that becomes the signing program keeps the identity the look before it recorded,
/// and the interval before the `exec` is credited to a program that was not running then.
///  One number and one start time hold two names here,
/// which is exactly what an `exec` leaves.
#[test]
fn a_program_that_takes_another_place_between_looks_is_credited_only_from_its_own() {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(Duration::from_secs(60)));
    let before: Vec<SeenBelow> = seen_below(&[seen(100, 5, "sh")]);
    let now: Vec<(i32, ProcessUse)> = vec![seen(100, 5, "ssh-keygen")];
    credit_between_looks(
        &mut facts,
        before.as_slice(),
        now.as_slice(),
        Duration::from_millis(250),
    );
    let kept: &crate::scratch_commit_step::SeenProgram =
        kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(
        kept.seen,
        Duration::ZERO,
        "a program was credited with an interval in which another program held its process: {facts:?}"
    );
}

/// What:
///  A program a look saw once is kept,
/// with a zero lower bound.
/// Why:
///   A signing program that runs and ends between two looks was below the step,
/// and a record that leaves it out says the step ran no signing program at all.
///  The look before it saw nothing,
/// so no interval can be credited to it,
/// and the entry it keeps is the lower bound of nothing.
#[test]
fn a_program_seen_once_is_kept_with_a_zero_lower_bound() {
    let mut facts: ScratchStepFacts = ScratchStepFacts::unarmed(Some(Duration::from_secs(60)));
    let before: Vec<SeenBelow> = Vec::new();
    let now: Vec<(i32, ProcessUse)> = vec![seen(300, 9, "ssh-keygen")];
    credit_between_looks(
        &mut facts,
        before.as_slice(),
        now.as_slice(),
        Duration::from_millis(250),
    );
    let kept: &crate::scratch_commit_step::SeenProgram =
        kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(
        kept.seen,
        Duration::ZERO,
        "a program seen once is kept with a zero lower bound: {facts:?}"
    );
}

/// What:
///  A look that could not be made keeps the ceiling armed instead of reading as "the step ended".
/// Why:
///   A descriptor or memory limit refuses both reads a look makes of `/proc`
/// (`read_dir`,
/// then one `stat` per process),
/// and an empty answer was read as the step having ended:
/// the notice returned,
/// the ceiling went with it,
/// and the step ran on while the command still waited for it.
///  The limit belongs to a whole process,
/// so it is lowered in a child process of this test's own,
/// and no other test of the run opens under it.
#[test]
fn a_look_that_could_not_be_made_keeps_the_ceiling_armed() {
    if !crate::descriptor_limit_child_test_support::this_process_is_the_child() {
        crate::descriptor_limit_child_test_support::started_this_test_in_a_child(
            module_path!(),
            "a_look_that_could_not_be_made_keeps_the_ceiling_armed",
            None,
        );
        return;
    }
    let baseline: usize = crate::test_support::open_descriptors();
    let before: rustix::process::Rlimit =
        crate::test_support::set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
    let held: Vec<std::fs::File> = crate::test_support::leave_free(0);
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
    let look: Option<Vec<(i32, ProcessUse)>> = look_below(own);
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(own);
    let came: CeilingCame = ceiling_came(&watched).came;
    drop(held);
    crate::test_support::restore_limit(before);
    assert!(
        look.is_none(),
        "the look was made with no descriptor left free: {look:?}"
    );
    assert_eq!(
        came,
        CeilingCame::Unknown,
        "a look that could not be made was read as the step having ended, which takes the ceiling away"
    );
}

/// The longest a control waits for the notice to reach a state it means to test.
///  Longer than several look intervals (`LOOK_INTERVAL`),
/// so a starved thread delays the control rather than failing it.
#[cfg(all(test, target_os = "linux"))]
const A_STATE_BOUND: Duration = Duration::from_secs(30);

/// The ceiling this control gives the notice,
///  long enough that no setup before the window can reach it:
/// the window holds the notice past it and this control plants the ceiling that comes.
#[cfg(all(test, target_os = "linux"))]
const A_CEILING_NO_SETUP_CAN_REACH: Duration = Duration::from_secs(600);

/// The ceiling this control plants once its window is blind,
///  already behind the look that found the step,
/// so the next look counts as past it.
#[cfg(all(test, target_os = "linux"))]
const A_CEILING_ALREADY_COME: Duration = Duration::from_millis(1);

/// What:
///  The ceiling still ends the step when the look below it cannot be made,
///  and the record says the programs below it could not be listed.
/// Why:
///   The harm this control exists for is the ceiling going away with a look that failed,
/// so the step runs on with nothing left to stop it while the command still waits.
///  What this control measured before it was ordered:
/// the look interval is 250 ms,
/// so a 50 ms sleep after the number is published never contains the look that finds the step.
/// In the runs that passed,
/// the arm that starts the ceiling landed inside the descriptor-free window,
/// so nothing was armed there,
/// and the step was found and stopped only after the window ended:
/// the control passed without ever making a look fail at the ceiling.
/// When the setup slipped in front of that look instead,
/// the arm happened before the window and the ceiling came inside it,
/// which is the failure this control reported.
///  The order is the control's now:
///  it waits for the look that finds the step,
///  makes `/proc` unreadable,
///  plants the ceiling so it comes at the next look,
///  and reads the answer in the record.
///  A loaded host makes the control slower and never wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // await untilALookFindsTheStep(); holdEveryDescriptor(); plantTheCeiling(); const seen = notice.finish(); expect(seen.stoppedAfterMs).toBeDefined(); expect(seen.belowUnlisted).toBe(true)
/// ```
#[cfg(all(test, target_os = "linux"))]
#[test]
fn a_step_is_still_stopped_after_a_look_that_could_not_be_made() {
    if !crate::descriptor_limit_child_test_support::this_process_is_the_child() {
        crate::descriptor_limit_child_test_support::started_this_test_in_a_child(
            module_path!(),
            "a_step_is_still_stopped_after_a_look_that_could_not_be_made",
            None,
        );
        return;
    }
    let _window: std::sync::MutexGuard<'_, ()> = the_controls_window::take_the_window();
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: A_CEILING_NO_SETUP_CAN_REACH,
        sink: Box::new(sink.clone()),
    });
    notice.step_started(step_number(&step));
    assert!(
        the_controls_window::until_a_look_finds_the_step(A_STATE_BOUND),
        "no look found the step within {A_STATE_BOUND:?}, so the ceiling was never armed"
    );
    // The ceiling comes now, and `/proc` cannot be read at all, so the look that carries it out fails.
    let baseline: usize = crate::test_support::open_descriptors();
    let before: rustix::process::Rlimit =
        crate::test_support::set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
    let held: Vec<std::fs::File> = crate::test_support::leave_free(0);
    assert!(
        std::fs::read_dir("/proc").is_err(),
        "the window this control makes is not blind: `/proc` was read with no descriptor left free"
    );
    the_controls_window::plant_the_ceiling(A_CEILING_ALREADY_COME);
    let ended: Option<i32> = ended_by_signal(&mut step);
    drop(held);
    crate::test_support::restore_limit(before);
    let seen: ScratchStepFacts = notice.finish();
    if step.try_wait().expect("status").is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(
        ended,
        Some(libc::SIGKILL),
        "a look that could not be made took the ceiling away, so the step ran on: {seen:?}"
    );
    assert!(
        seen.stopped_after.is_some(),
        "the stop is recorded: {seen:?}"
    );
    assert!(
        seen.below_unlisted,
        "a walk that could not be made leaves what the step started unnamed, and the record says so: {seen:?}"
    );
}

/// What:
///  The signal a process ended with,
///  or nothing when it was still running at `bound`.
/// Why:
///   A control that shows a process was left alone must give a signal on its way the same bound,
/// so "alive" is an answer and not a race.
fn ended_within(
    child: &mut std::process::Child,
    bound: Duration,
) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    let deadline: Instant = Instant::now() + bound;
    loop {
        if let Some(status) = child.try_wait().expect("status") {
            return Some(status.signal().unwrap_or(0));
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// How long a control waits before it calls a process it did not signal alive:
///  longer than a `SIGKILL` on its way takes,
/// and only a control pays it.
const STILL_RUNNING_BOUND: Duration = Duration::from_millis(500);

/// A child of this process that is nobody's step,
/// started by `test_support::spawn_owned` so the test thread's fixtures end it however the test ends.
fn a_process_beside_the_step() -> std::process::Child {
    return crate::test_support::spawn_owned(
        std::process::Command::new("sleep")
            .arg("600")
            .stdin(std::process::Stdio::null()),
    )
    .expect("a long-running child");
}

/// What:
///  A ceiling ends the step it watches,
/// and no process beside it.
/// Why:
///   The landing service runs landings of different repositories on worker threads of one process,
/// so another landing's Git is a child of this process at the same moment as this step.
/// The notice read every child of the process and sent `SIGKILL` to all of them,
/// which cuts the other landing's write in the middle,
/// and the owner's answer to that finding (2026-11-09,
/// "B:
/// let a branch write finish")
/// is that a ceiling reaches only its own step.
#[test]
fn a_ceiling_ends_the_step_and_not_a_process_beside_it() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    let mut beside: std::process::Child = a_process_beside_the_step();
    let mut step: std::process::Child = sleeping_step();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(Buffer::default()),
    });
    notice.step_started(step_number(&step));
    let ended: Option<i32> = ended_by_signal(&mut step);
    let beside_ended: Option<i32> = ended_within(&mut beside, STILL_RUNNING_BOUND);
    let seen: ScratchStepFacts = notice.finish();
    if beside_ended.is_none() {
        beside.kill().expect("cleanup");
        beside.wait().expect("reaped");
    }
    assert!(
        ended.is_some(),
        "the ceiling did not end the step it watches: {seen:?}"
    );
    assert!(
        beside_ended.is_none(),
        "the ceiling ended a process beside the step ({beside_ended:?}); only the step it watches may be ended"
    );
}

/// What:
///  A ceiling that finds the step already ended reports no stop,
///  and one that finds it running does.
/// Why:
///   A commit that landed is refused as "stopped" when the notice records a stop it never made:
/// the step can finish between the look that armed the ceiling and the signal.
///  The ended step here really is a reaped process and the running one really is live,
/// so the two arms differ in nothing but whether a signal was taken.
#[test]
fn a_ceiling_reports_a_stop_only_when_the_step_was_signalled() {
    let mut ended_step: std::process::Child = sleeping_step();
    // The step is published while it is alive, which is the only moment the kernel makes its descriptor, and ends after.
    let already_over: ChildNumber = ChildNumber::not_started_pinned();
    already_over.publish(step_number(&ended_step));
    ended_step.kill().expect("the step ends");
    ended_step.wait().expect("reaped");
    let mut running_step: std::process::Child = sleeping_step();
    let running: ChildNumber = ChildNumber::not_started_pinned();
    running.publish(step_number(&running_step));
    let after_the_end: CeilingCame = ceiling_came(&already_over).came;
    let while_running: CeilingCame = ceiling_came(&running).came;
    let ended: Option<i32> = ended_by_signal(&mut running_step);
    if ended.is_none() {
        running_step.kill().expect("cleanup");
        running_step.wait().expect("reaped");
    }
    assert_eq!(
        after_the_end,
        CeilingCame::TheStepEnded,
        "a step that already ended was reported as stopped, which refuses a commit that landed"
    );
    assert_eq!(
        while_running,
        CeilingCame::Stopped,
        "a step that was signalled is reported as stopped"
    );
    assert!(ended.is_some(), "the step the ceiling signalled ended");
}

/// A sink that stops taking bytes until it is let go,
/// standing for a standard error whose reader stopped reading:
/// a pager a person stopped,
/// or a terminal stopped with the flow control key.
struct StoppedReaderSink {
    /// Set once a write was entered,
    /// so a control knows the line reached the sink.
    entered: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Set by the control to let the write return.
    released: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// What was written before the write returned.
    written: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

impl Write for StoppedReaderSink {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        self.entered
            .store(true, std::sync::atomic::Ordering::SeqCst);
        while !self.released.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        self.written
            .lock()
            .expect("written")
            .extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// The cadence the control gives the notice.
///  Short on purpose:
/// the notice hands over one line per look,
/// so every line a stopped reader costs is offered within the first few of them,
/// and a control that waited out the shipped ten-second cadence would be waiting on the notice's own wakeups.
const A_STALLED_READER_CADENCE: Cadence = Cadence {
    grace: Duration::from_millis(20),
    interval: Duration::from_millis(20),
};

/// What:
///  How long the control waits for the finish of a notice whose reader stopped reading,
///  before it counts the commit as held by that reader.
/// Why:
///   This is not the notice's promise.
///  `finish` waits no longer than twice `NOTICE_BOUND`,
/// once for the notice's answer and once for the line it handed over,
/// and this is how long a control waits before it stops waiting.
///  Measured on 2026-11-26 from this control's own record over six runs,
/// `finish` answered in 500.1 to 501.6 ms
/// (three runs alone,
/// one of its module,
/// one of the `commit::` module,
/// and one of `commit::` at 24 threads),
/// and the thread that ran it started within 0.99 ms of the window opening.
///  The recorded failure of 2026-11-26 lost more than a second of wall clock that way and reported a commit held
/// at two seconds,
/// which is twice the product's own maximum and nothing like a wide window.
///  Thirty times `NOTICE_BOUND` is thirty times the measured answer and fifteen times what `finish` may honestly
/// wait,
/// so a host that wakes a thread late delays the control instead of failing it,
/// and a `finish` that really waits for the reader still fails,
/// naming the reader.
/// Gotcha:
///  A wider window is a weaker guard,
/// so it stays far below the reader's own stall:
/// the reader takes nothing until this control lets its line go,
/// which is after this window has passed.
const COMMIT_RETURN_BOUND: Duration = NOTICE_BOUND.saturating_mul(30);

/// What:
///  A standard error whose reader stopped reading keeps neither the notice nor the commit.
/// Why:
///   Finding 3 of the process and child lifetime audit on main:
/// the notice wrote its line on the thread that enforces the ceiling,
/// and `finish` joined that thread with no bound,
/// so a full standard error pipe held the commit after the commit itself had ended,
/// with its index lock and its transaction still held.
///  The sink here is the reader that stopped reading:
/// it says when a write was entered and takes nothing until the control lets it go.
#[test]
fn a_notice_whose_sink_stopped_reading_does_not_hold_the_commit() {
    let entered: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::default();
    let released: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::default();
    let written: std::sync::Arc<std::sync::Mutex<Vec<u8>>> = std::sync::Arc::default();
    let sink: StoppedReaderSink = StoppedReaderSink {
        entered: entered.clone(),
        released: released.clone(),
        written: written.clone(),
    };
    let cadence: Cadence = A_STALLED_READER_CADENCE;
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_secs(60),
        sink: Box::new(sink),
    });
    // Bounded, and by the event: the first line is due one cadence after the start, so this waits for that write and not for a time.
    let writing_started: Instant = Instant::now() + Duration::from_secs(10);
    while !entered.load(std::sync::atomic::Ordering::SeqCst) && Instant::now() < writing_started {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        entered.load(std::sync::atomic::Ordering::SeqCst),
        "the notice never wrote a line"
    );
    // What: one line offered into the queue the writer takes its lines from, which the writer cannot take yet.
    // Why:  the state the record calls a reader that stopped taking lines is the writer holding one line
    //       while the queue cannot hold another, and one send reaches it.
    //       Waiting for the notice to reach it by its own looks took a wall clock the notice's own wakeups decide:
    //       measured 2026-11-26, a notice thread starved by a threaded `commit::` run made three looks in 400 ms,
    //       so the third line the control used to sleep out was never offered and the record counted no drop.
    let queued: Result<(), std::sync::mpsc::TrySendError<String>> = notice
        .lines_for_the_wait()
        .expect("a notice whose writer thread was started")
        .try_send(String::new());
    assert!(
        !matches!(&queued, Err(std::sync::mpsc::TrySendError::Disconnected(_))),
        "the writer's queue ended before the notice was over, so no line of it could be dropped: {queued:?}"
    );
    // One look interval is the longest the notice goes between two looks, so after one of those it has looked again
    // and offered the line the full queue costs. A starved thread makes that look later and never skips it, and the
    // record says what it offered.
    std::thread::sleep(LOOK_INTERVAL);
    let (sent, told) = std::sync::mpsc::channel::<ScratchStepFacts>();
    let began: Instant = Instant::now();
    let finishing: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let facts: ScratchStepFacts = notice.finish();
        let _told = sent.send(facts);
    });
    let heard: Option<ScratchStepFacts> = told.recv_timeout(COMMIT_RETURN_BOUND).ok();
    let waited: Duration = began.elapsed();
    // Let the line through, so a finish that is still waiting for it (the code this control exists for) returns and ends the test.
    released.store(true, std::sync::atomic::Ordering::SeqCst);
    let answered_in_time: bool = heard.is_some();
    let facts: ScratchStepFacts = match heard {
        Some(answered) => answered,
        None => told
            .recv_timeout(Duration::from_secs(30))
            .expect("the finish returns once the reader takes the line"),
    };
    finishing.join().expect("the finishing thread");
    assert!(
        answered_in_time,
        "the commit was still waiting for a reader that stopped reading {waited:?} after its step ended: {facts:?}"
    );
    assert!(
        facts.lines_dropped >= 1,
        "the record says a line was dropped because the reader was not taking it: {facts:?}"
    );
    assert_eq!(
        facts.looks_found, 0,
        "no step ran, so no look found one: {facts:?}"
    );
}

/// A sink that takes the first line at once and waits to be let go before every later one.
struct LateSink {
    /// What it took.
    written: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    /// Set once a later line is waiting to be let go.
    blocking: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Set by the control to let the waiting line through.
    released: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Write for LateSink {
    fn write(
        &mut self,
        bytes: &[u8],
    ) -> std::io::Result<usize> {
        let after_the_first: bool = !self.written.lock().expect("written").is_empty();
        if after_the_first {
            self.blocking
                .store(true, std::sync::atomic::Ordering::SeqCst);
            while !self.released.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        self.written
            .lock()
            .expect("written")
            .push(String::from_utf8_lossy(bytes).into_owned());
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A command that ended prints nothing more.
/// Why:
///   Finding 5 of the second review:
/// the writer thread is detached and can be inside a write when the command ends,
/// so the line it holds and the line queued behind it were printed after the command had returned,
/// and a reader that resumes late reads "still waiting" for a step that is over.
///  The lines here are due every 60 ms,
/// the first is taken at once,
/// the second waits,
/// and the control ends the notice while it waits.
#[test]
fn a_finished_command_prints_nothing_more() {
    let written: std::sync::Arc<std::sync::Mutex<Vec<String>>> = std::sync::Arc::default();
    let blocking: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::default();
    let released: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::default();
    let sink: LateSink = LateSink {
        written: written.clone(),
        blocking: blocking.clone(),
        released: released.clone(),
    };
    let cadence: Cadence = Cadence {
        grace: Duration::from_millis(60),
        interval: Duration::from_millis(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_secs(60),
        sink: Box::new(sink),
    });
    // Bounded: the first line is due 60 ms after the start, and the second 60 ms after that.
    let deadline: Instant = Instant::now() + Duration::from_secs(10);
    while !blocking.load(std::sync::atomic::Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        blocking.load(std::sync::atomic::Ordering::SeqCst),
        "the sink never had a second line to wait on"
    );
    // Two more cadences: the writer still waits, so one line is queued behind it and the later ones are dropped.
    std::thread::sleep(Duration::from_millis(150));
    let _facts: ScratchStepFacts = notice.finish();
    let when_over: usize = written.lock().expect("written").len();
    assert_eq!(when_over, 1, "one line was taken before the command ended");
    released.store(true, std::sync::atomic::Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(300));
    let after: usize = written.lock().expect("written").len();
    assert!(
        after <= when_over + 1,
        "a line was printed after the command was over: {} lines, {when_over} of them before it ended",
        after
    );
}

/// The ceiling a control gives a step that outlasts it:
/// short enough that a loaded host still reaches it.
const A_CEILING_A_STEP_OUTLASTS: Duration = Duration::from_millis(200);

/// The bound a control plants for a step the kernel holds:
/// no test waits the five minutes the shipped bound allows.
const A_STALL_BOUND: Duration = Duration::from_secs(30);

/// What:
///  The ceiling stands aside once for a step the kernel holds in a disk wait,
///  and the step runs on.
/// Why:
///   Ending a step that waits for the disk fails a commit that would have landed:
///  on 2026-11-11 two commits failed that way,
///  their steps held for 37,153 and 47,978 ms under a 16,332 ms ceiling.
///  The step here is a real child of this process;
///  what a control states is the kernel's answer about its state,
///  which no run on this host can make real (`scratch_step_look_below_control_window.rs`).
///  The record says the ceiling stood aside and names the bound the step then ran under,
///  so the next reading of the log can tell a slow step from a stopped one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // plantTheStallCeiling(30_000); const step = sh('sleep 120'); theKernelHolds(step.pid); await sleep(1_000); expect(endedWithin(step)).toBeUndefined(); expect(seen.stoppedAfter).toBeUndefined(); expect(seen.stallCeilingMs).toBe(30_000)
/// ```
#[cfg(all(test, target_os = "linux"))]
#[test]
fn the_ceiling_stands_aside_for_a_step_the_kernel_holds() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    the_controls_window::plant_the_stall_ceiling(A_STALL_BOUND);
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: A_CEILING_A_STEP_OUTLASTS,
        sink: Box::new(sink.clone()),
    });
    crate::scratch_step_look_below::the_controls_window::the_kernel_holds(
        step_number(&step),
        "git",
    );
    notice.step_started(step_number(&step));
    std::thread::sleep(A_CEILING_A_STEP_OUTLASTS * 5);
    let ended: Option<i32> = ended_within(&mut step, A_CEILING_A_STEP_OUTLASTS);
    let seen: ScratchStepFacts = notice.finish();
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(
        ended, None,
        "the ceiling ended a step the kernel held past it: {seen:?}"
    );
    assert_eq!(
        seen.stopped_after, None,
        "the ceiling stood aside, so no stop is recorded: {seen:?}"
    );
    assert!(
        seen.held_after_ceiling.is_some_and(|waited| {
            return waited >= A_CEILING_A_STEP_OUTLASTS;
        }),
        "the record does not say when the ceiling stood aside: {seen:?}"
    );
    assert_eq!(
        seen.stall_ceiling,
        Some(A_STALL_BOUND),
        "the record names the bound the step then ran under: {seen:?}"
    );
}

/// What:
///  The thread that watches a step the kernel holds prints no line of its own about the stand-aside.
/// Why:
///   Finding 3 of the 2026-11-12 review of the stand-aside:
///  both enforcers stood aside for the same step and each handed the same sentence to the one writer,
///  so the person read it twice in one commit.
///  The line belongs to the wait that owns the child, which always runs and is the one that lasts
///  (`process/child_awaited_until_it_exits_the_stop_or_the_ceiling.rs`);
///  this thread says the later bound in its waiting lines and nothing more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // plantTheStallCeiling(30_000); const step = sh('sleep 120'); theKernelHolds(step.pid); await sleep(1_000); expect(sink).not.toContain('reached its ceiling')
/// ```
#[cfg(all(test, target_os = "linux"))]
#[test]
fn the_ceiling_that_stood_aside_prints_no_line_of_its_own() {
    let _windows: (
        std::sync::MutexGuard<'static, ()>,
        std::sync::MutexGuard<'static, ()>,
    ) = take_the_windows_a_notice_control_reads();
    the_controls_window::plant_the_stall_ceiling(A_STALL_BOUND);
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence {
        grace: Duration::from_secs(60),
        interval: Duration::from_secs(60),
    };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: A_CEILING_A_STEP_OUTLASTS,
        sink: Box::new(sink.clone()),
    });
    crate::scratch_step_look_below::the_controls_window::the_kernel_holds(
        step_number(&step),
        "git",
    );
    notice.step_started(step_number(&step));
    std::thread::sleep(A_CEILING_A_STEP_OUTLASTS * 5);
    let seen: ScratchStepFacts = notice.finish();
    step.kill().expect("cleanup");
    step.wait().expect("reaped");
    assert_eq!(
        seen.stall_ceiling,
        Some(A_STALL_BOUND),
        "the ceiling stood aside: {seen:?}"
    );
    let said: String = text_of(&sink);
    assert!(
        !said.contains("reached its ceiling"),
        "the thread that watches the step printed the wait's line: {said}"
    );
    assert!(
        !said.contains("waits for it instead of failing the commit"),
        "the stand-aside is said by the wait that owns the step, once: {said}"
    );
}
