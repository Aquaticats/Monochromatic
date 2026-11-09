//! Controls for the waiting notice of the commit Git builds:
//! what the line names,
//! when the commit counts as signed,
//! and that a
//!  commit that opens an editor,
//! or that ends at once,
//! prints nothing.

use super::*;

/// The words of a command line as the notice reads them.
fn words(text: &[&str]) -> Vec<String> {
    return text.iter().map(|word| { return (*word).to_string(); }).collect();
}

/// A shared buffer the notice thread writes into.
#[derive(Clone, Default)]
struct Buffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
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

/// The command line can ask for a signature,
/// and `--no-gpg-sign` takes the request back;
/// the arguments after `--` are paths.
#[test]
fn the_command_line_can_ask_for_a_signature_and_take_it_back() {
    assert!(signs_commit(None, &words(&["-S", "-m", "x"])));
    assert!(signs_commit(None, &words(&["--gpg-sign=KEY", "-m", "x"])));
    assert!(signs_commit(None, &words(&["-SKEY", "-m", "x"])));
    assert!(!signs_commit(None, &words(&["-S", "--no-gpg-sign", "-m", "x"])));
    assert!(!signs_commit(None, &words(&["-m", "x", "--", "-S"])));
    assert!(!signs_commit(None, &words(&["-m", "x"])));
}

/// A step that ends before the first notice is due prints nothing,
/// and one that lasts prints at the interval.
#[test]
fn a_step_that_ends_at_once_prints_nothing_and_a_long_one_prints_again() {
    let quick: Buffer = Buffer::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(400), interval: Duration::from_millis(400) };
    let quick_notice: NativeCommitNotice = NativeCommitNotice::start(None, &words(&["-m", "x"]), cadence, Box::new(quick.clone()));
    quick_notice.finish();
    assert_eq!(text_of(&quick), "");
    let slow: Buffer = Buffer::default();
    let notice: NativeCommitNotice = NativeCommitNotice::start(None, &words(&["-m", "x"]), cadence, Box::new(slow.clone()));
    std::thread::sleep(Duration::from_millis(1300));
    notice.finish();
    let said: String = text_of(&slow);
    assert!(said.lines().count() >= 2, "{said}");
    assert_eq!(said.matches("Hint").count(), 1, "only the first line carries the hint: {said}");
}

/// A commit that opens the person's editor prints nothing however long it lasts.
#[test]
fn a_commit_that_opens_an_editor_prints_nothing() {
    let quiet: Buffer = Buffer::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(50), interval: Duration::from_millis(50) };
    let notice: NativeCommitNotice = NativeCommitNotice::start(None, &words(&["-m", "x", "--edit"]), cadence, Box::new(quiet.clone()));
    std::thread::sleep(Duration::from_millis(300));
    notice.finish();
    assert_eq!(text_of(&quiet), "");
}

/// A sleeping child of this test process,
/// which stands for the `git commit` of the step.
fn sleeping_step() -> std::process::Child {
    return std::process::Command::new("sleep").arg("120").spawn().expect("a long-running child");
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
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    assert!(ended.is_some(), "the step was still running after its ceiling");
    let stopped: Option<Duration> = seen.stopped_after;
    assert!(stopped.is_some_and(|after| { return after >= Duration::from_millis(300); }), "{stopped:?}");
    assert!(seen.armed, "a message on the command line arms the ceiling");
    assert!(seen.looks >= 1 && seen.looks_found >= 1, "the notice looked and found the step: {seen:?}");
}

/// A commit that may open the person's editor has no ceiling:
/// the person is the wait,
/// and a message typed into an editor would be lost by a stop.
#[test]
fn a_commit_that_may_open_an_editor_is_never_stopped() {
    let mut step: std::process::Child = sleeping_step();
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    assert!(ended.is_some(), "a step that started after the ceiling's look ran on with no ceiling and was never stopped");
    assert!(stopped.is_some_and(|after| { return after >= Duration::from_millis(200); }), "{stopped:?}");
}

/// The entry the notice kept for the program `name`,
/// or a failure that names what it kept instead.
fn kept_below<'kept>(below: &'kept [crate::scratch_commit_step::SeenProgram], name: &str) -> &'kept crate::scratch_commit_step::SeenProgram {
    for program in below {
        if program.name == name {
            return program;
        }
    }
    panic!("the notice kept no entry for `{name}`: {below:?}");
}

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
    let mut step: std::process::Child = std::process::Command::new("sleep").arg("2").spawn().expect("a long-running child");
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_secs(120),
        sink: Box::new(Buffer::default()),
    });
    notice.step_started(step_number(&step));
    std::thread::sleep(Duration::from_millis(1200));
    let seen: ScratchStepFacts = notice.finish();
    step.kill().expect("cleanup");
    step.wait().expect("reaped");
    assert!(seen.armed && seen.looks_found >= 1, "the notice looked and found the step: {seen:?}");
    assert_eq!(seen.stopped_after, None, "the ceiling never came: {seen:?}");
    let found: &crate::scratch_commit_step::SeenProgram = kept_below(seen.below.as_slice(), "sleep");
    assert!(found.seen >= Duration::from_millis(750), "the sleep ran while the notice looked: {seen:?}");
    assert!(found.seen <= Duration::from_millis(1200), "a program cannot be seen longer than it ran: {seen:?}");
}

/// One process a look saw below the step,
/// as `look_below` reports it.
fn seen(pid: i32, started: u64, name: &str) -> (i32, ProcessUse) {
    return (pid, ProcessUse { name: String::from(name), started, state: 'S', cpu: Duration::ZERO });
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
    credit_between_looks(&mut facts, before.as_slice(), now.as_slice(), Duration::from_millis(250));
    let kept: &crate::scratch_commit_step::SeenProgram = kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(kept.seen, Duration::ZERO, "a program that was not below the step at the look before it was credited with time no process ran: {facts:?}");
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
    credit_between_looks(&mut facts, before.as_slice(), now.as_slice(), Duration::from_millis(250));
    let kept: &crate::scratch_commit_step::SeenProgram = kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(kept.seen, Duration::ZERO, "a program was credited with an interval in which another program held its process: {facts:?}");
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
    credit_between_looks(&mut facts, before.as_slice(), now.as_slice(), Duration::from_millis(250));
    let kept: &crate::scratch_commit_step::SeenProgram = kept_below(facts.below.as_slice(), "ssh-keygen");
    assert_eq!(kept.seen, Duration::ZERO, "a program seen once is kept with a zero lower bound: {facts:?}");
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
///  The limit is this test's own process,
/// which nextest runs alone for each test.
#[test]
fn a_look_that_could_not_be_made_keeps_the_ceiling_armed() {
    let baseline: usize = crate::test_support::open_descriptors();
    let before: rustix::process::Rlimit = crate::test_support::set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
    let held: Vec<std::fs::File> = crate::test_support::leave_free(0);
    let own: i32 = i32::try_from(std::process::id()).unwrap_or(0);
    let look: Option<Vec<(i32, ProcessUse)>> = look_below(own);
    let watched: ChildNumber = ChildNumber::not_started_pinned();
    watched.publish(own);
    let came: CeilingCame = ceiling_came(&watched).came;
    drop(held);
    crate::test_support::restore_limit(before);
    assert!(look.is_none(), "the look was made with no descriptor left free: {look:?}");
    assert_eq!(came, CeilingCame::Unknown, "a look that could not be made was read as the step having ended, which takes the ceiling away");
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
    let _window: std::sync::MutexGuard<'_, ()> = the_controls_window::take_the_window();
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    let before: rustix::process::Rlimit = crate::test_support::set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
    let held: Vec<std::fs::File> = crate::test_support::leave_free(0);
    assert!(std::fs::read_dir("/proc").is_err(), "the window this control makes is not blind: `/proc` was read with no descriptor left free");
    the_controls_window::plant_the_ceiling(A_CEILING_ALREADY_COME);
    let ended: Option<i32> = ended_by_signal(&mut step);
    drop(held);
    crate::test_support::restore_limit(before);
    let seen: ScratchStepFacts = notice.finish();
    if step.try_wait().expect("status").is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert_eq!(ended, Some(libc::SIGKILL), "a look that could not be made took the ceiling away, so the step ran on: {seen:?}");
    assert!(seen.stopped_after.is_some(), "the stop is recorded: {seen:?}");
    assert!(seen.below_unlisted, "a walk that could not be made leaves what the step started unnamed, and the record says so: {seen:?}");
}

/// What:
///  The signal a process ended with,
///  or nothing when it was still running at `bound`.
/// Why:
///   A control that shows a process was left alone must give a signal on its way the same bound,
/// so "alive" is an answer and not a race.
fn ended_within(child: &mut std::process::Child, bound: Duration) -> Option<i32> {
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

/// A child of this process that is nobody's step.
fn a_process_beside_the_step() -> std::process::Child {
    return std::process::Command::new("sleep").arg("600").stdin(std::process::Stdio::null()).spawn().expect("a long-running child");
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
    let mut beside: std::process::Child = a_process_beside_the_step();
    let mut step: std::process::Child = sleeping_step();
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
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
    assert!(ended.is_some(), "the ceiling did not end the step it watches: {seen:?}");
    assert!(beside_ended.is_none(), "the ceiling ended a process beside the step ({beside_ended:?}); only the step it watches may be ended");
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
    assert_eq!(after_the_end, CeilingCame::TheStepEnded, "a step that already ended was reported as stopped, which refuses a commit that landed");
    assert_eq!(while_running, CeilingCame::Stopped, "a step that was signalled is reported as stopped");
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
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.entered.store(true, std::sync::atomic::Ordering::SeqCst);
        while !self.released.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        self.written.lock().expect("written").extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// How long a control gives a commit to return after its step ended:
///  longer than the notice's own bound,
/// and far shorter than a reader that never reads again.
const COMMIT_RETURN_BOUND: Duration = Duration::from_secs(2);

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
    let sink: StoppedReaderSink = StoppedReaderSink { entered: entered.clone(), released: released.clone(), written: written.clone() };
    let cadence: Cadence = Cadence { grace: Duration::from_millis(100), interval: Duration::from_millis(100) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_secs(60),
        sink: Box::new(sink),
    });
    // Bounded: the first line is due 100 ms after the start.
    let writing_started: Instant = Instant::now() + Duration::from_secs(10);
    while !entered.load(std::sync::atomic::Ordering::SeqCst) && Instant::now() < writing_started {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(entered.load(std::sync::atomic::Ordering::SeqCst), "the notice never wrote a line");
    // Three cadences: the writer holds the first line, the queue holds the second, and the third cannot be handed over.
    std::thread::sleep(cadence.interval.saturating_mul(3));
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
        None => told.recv_timeout(Duration::from_secs(30)).expect("the finish returns once the reader takes the line"),
    };
    finishing.join().expect("the finishing thread");
    assert!(answered_in_time, "the commit was still waiting for a reader that stopped reading {waited:?} after its step ended: {facts:?}");
    assert!(facts.lines_dropped >= 1, "the record says a line was dropped because the reader was not taking it: {facts:?}");
    assert_eq!(facts.looks_found, 0, "no step ran, so no look found one: {facts:?}");
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
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let after_the_first: bool = !self.written.lock().expect("written").is_empty();
        if after_the_first {
            self.blocking.store(true, std::sync::atomic::Ordering::SeqCst);
            while !self.released.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        self.written.lock().expect("written").push(String::from_utf8_lossy(bytes).into_owned());
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
    let sink: LateSink = LateSink { written: written.clone(), blocking: blocking.clone(), released: released.clone() };
    let cadence: Cadence = Cadence { grace: Duration::from_millis(60), interval: Duration::from_millis(60) };
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
    assert!(blocking.load(std::sync::atomic::Ordering::SeqCst), "the sink never had a second line to wait on");
    // Two more cadences: the writer still waits, so one line is queued behind it and the later ones are dropped.
    std::thread::sleep(Duration::from_millis(150));
    let _facts: ScratchStepFacts = notice.finish();
    let when_over: usize = written.lock().expect("written").len();
    assert_eq!(when_over, 1, "one line was taken before the command ended");
    released.store(true, std::sync::atomic::Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(300));
    let after: usize = written.lock().expect("written").len();
    assert!(after <= when_over + 1, "a line was printed after the command was over: {} lines, {when_over} of them before it ended", after);
}
