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
