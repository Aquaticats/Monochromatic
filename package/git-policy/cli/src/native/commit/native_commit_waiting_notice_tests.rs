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

/// The first line names the commit Git builds,
/// says there is no limit and adds the hint;
/// a repeat is the same without the hint;
/// a
///  signed commit says why,
/// and a signer below it is named.
#[test]
fn the_line_names_the_step_the_limit_and_the_signer() {
    let ceiling: Duration = Duration::from_secs(1260);
    let first: String = notice_line(false, Duration::from_secs(12), true, &[], ceiling);
    assert!(first.starts_with("monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, 12 seconds so far;"), "{first}");
    assert!(first.contains("stopped if it lasts 1260 seconds") && first.contains("Hint: it is still running"), "{first}");
    assert!(!first.contains("no time limit"), "{first}");
    assert!(!first.contains("signs"), "{first}");
    let repeat: String = notice_line(true, Duration::from_secs(22), false, &[(7, String::from("ssh-keygen"))], ceiling);
    assert!(!repeat.contains("Hint"), "{repeat}");
    assert!(repeat.contains("signs the commit because `commit.gpgsign` or `-S` asks for it") && repeat.contains("`ssh-keygen` (process 7) runs below it"), "{repeat}");
    assert!(first.ends_with('\n') && repeat.ends_with('\n'));
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

/// The words of a stop say what ended,
/// after how long,
/// why this long,
/// what became of the person's work,
/// and what to run.
#[test]
fn the_words_of_a_stop_say_what_ended_what_is_kept_and_what_to_run() {
    let said: String = stop_words(Duration::from_secs(1261), Duration::from_secs(1260), true);
    assert!(said.starts_with("monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after 1261 seconds"), "{said}");
    assert!(said.contains("1260 seconds") && said.contains("three times"), "{said}");
    assert!(said.contains("signs the commit"), "a signed commit's words name the signer: {said}");
    assert!(said.contains("nothing was committed") && said.contains("index and files are as they were"), "{said}");
    assert!(said.contains("Run the same `git commit` command again"), "{said}");
    assert!(!stop_words(Duration::from_secs(1261), Duration::from_secs(1260), false).contains("signs the commit"));
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
///  A program a look saw once is kept, with a zero lower bound.
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
/// (`read_dir`, then one `stat` per process),
/// and an empty answer was read as the step having ended:
/// the notice returned, the ceiling went with it,
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
    let came: CeilingCame = ceiling_came(own);
    drop(held);
    crate::test_support::restore_limit(before);
    assert!(look.is_none(), "the look was made with no descriptor left free: {look:?}");
    assert_eq!(came, CeilingCame::Unknown, "a look that could not be made was read as the step having ended, which takes the ceiling away");
}

/// What:
///  A step is still stopped at its ceiling after a look that could not be made.
/// Why:
///   This is the harm the review describes:
/// the ceiling's look fails (a descriptor limit refuses the reads of `/proc`),
/// and the old notice ended itself there,
/// so the step ran on with nothing left to stop it while the command still waited.
///  The limit is dropped after the first look has found the step and held past the ceiling,
/// and the next look that can be made stops it.
#[test]
fn a_step_is_still_stopped_after_a_look_that_could_not_be_made() {
    let mut step: std::process::Child = sleeping_step();
    let sink: Buffer = Buffer::default();
    let cadence: Cadence = Cadence { grace: Duration::from_secs(60), interval: Duration::from_secs(60) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(200),
        sink: Box::new(sink.clone()),
    });
    notice.step_started(step_number(&step));
    // The first look finds the step, so the ceiling is armed on it.
    std::thread::sleep(Duration::from_millis(50));
    let baseline: usize = crate::test_support::open_descriptors();
    let before: rustix::process::Rlimit = crate::test_support::set_soft_limit(u64::try_from(baseline).expect("a count") + 16);
    let held: Vec<std::fs::File> = crate::test_support::leave_free(0);
    // Long enough for two looks, both past the ceiling, to fail.
    std::thread::sleep(Duration::from_millis(600));
    let still_running: bool = step.try_wait().expect("status").is_none();
    drop(held);
    crate::test_support::restore_limit(before);
    let ended: Option<i32> = ended_by_signal(&mut step);
    let seen: ScratchStepFacts = notice.finish();
    if ended.is_none() {
        step.kill().expect("cleanup");
        step.wait().expect("reaped");
    }
    assert!(still_running, "the step was stopped while `/proc` could not be read at all");
    assert!(ended.is_some(), "a look that could not be made took the ceiling away: {seen:?}");
    assert!(seen.stopped_after.is_some(), "the stop is recorded: {seen:?}");
    assert!(seen.blind_looks >= 1, "the record says a look could not be made: {seen:?}");
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
///  A ceiling ends the step it watches, and no process beside it.
/// Why:
///   The landing service runs landings of different repositories on worker threads of one process,
/// so another landing's Git is a child of this process at the same moment as this step.
/// The notice read every child of the process and sent `SIGKILL` to all of them,
/// which cuts the other landing's write in the middle,
/// and the owner's answer to that finding (2026-11-09, "B: let a branch write finish")
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
    let ended_number: i32 = step_number(&ended_step);
    ended_step.kill().expect("the step ends");
    ended_step.wait().expect("reaped");
    let mut running_step: std::process::Child = sleeping_step();
    let running_number: i32 = step_number(&running_step);
    let after_the_end: CeilingCame = ceiling_came(ended_number);
    let while_running: CeilingCame = ceiling_came(running_number);
    let ended: Option<i32> = ended_by_signal(&mut running_step);
    if ended.is_none() {
        running_step.kill().expect("cleanup");
        running_step.wait().expect("reaped");
    }
    assert_eq!(after_the_end, CeilingCame::TheStepEnded, "a step that already ended was reported as stopped, which refuses a commit that landed");
    assert_eq!(while_running, CeilingCame::Stopped, "a step that was signalled is reported as stopped");
    assert!(ended.is_some(), "the step the ceiling signalled ended");
}
