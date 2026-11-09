//! Controls for the watcher of waits that block on a child process:
//!  the limit from its variable,
//! the notices at the cadence,
//! the child stopped at the bound,
//!  and the one-shot run that captures a command's output under a watch.
#![cfg(unix)]

use super::*;
use std::os::fd::AsFd as _;
use std::io::Write;
use crate::child_class::CHILD_WAIT_VARIABLE;
use crate::wait_progress::Cadence;
use crate::test_support::{ActionSink, SharedSink};
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::Stdio;

/// A cadence of a 60 ms grace and a 60 ms interval,
/// so a notice comes fast.
const QUICK: Cadence = Cadence {
    grace: Duration::from_millis(60),
    interval: Duration::from_millis(60),
};

/// How many intervals of the quick cadence a control offers a watch that has no wait running,
/// each an end mark read in order:
/// more than two,
/// so a watcher that went on counting a finished wait is due a notice on one of them.
const INTERVALS_OFFERED: usize = 3;

/// Terms of a limit,
/// the quick cadence and a shared sink the test reads.
fn terms(limit: Duration, sink: &SharedSink) -> WatchTerms {
    return WatchTerms {
        limit,
        cadence: QUICK,
        notices: Box::new(sink.clone()),
    };
}

/// The sentence for a stopped wait names it,
/// the window,
/// the variable and what to do next.
#[test]
fn the_timeout_sentence_names_the_wait_the_bound_and_the_next_step() {
    let limit: Duration = ChildClass::Object.default_limit();
    let subject: String = format!("git add -- a.md, {}", ChildClass::Object.label());
    let text: String = watched_timeout_text(subject.as_str(), limit);
    assert!(text.starts_with(format!("{subject} showed no sign of work for ").as_str()), "{text}");
    assert!(text.contains(format!("for {} seconds and was stopped", limit.as_secs()).as_str()), "{text}");
    assert!(text.contains(format!("{CHILD_WAIT_VARIABLE}=").as_str()), "the raise hint spells the variable: {text}");
    assert!(text.contains("run the command again"), "{text}");
    assert!(text.contains("`ps`"), "{text}");
}

/// The object reader names itself with its class,
/// like every other step.
#[test]
fn the_reader_subject_names_the_command_and_its_class() {
    let subject: String = reader_subject();
    assert!(subject.starts_with("git cat-file --batch"), "{subject}");
    assert!(subject.ends_with(ChildClass::Object.label()), "{subject}");
}

/// The terms of a class carry its bound,
/// and a grace that never passes the interval or the bound.
#[test]
fn the_terms_of_a_class_carry_its_bound_and_a_grace_inside_it() {
    for class in [ChildClass::Plumbing, ChildClass::Object] {
        let terms: WatchTerms = WatchTerms::for_class(class);
        assert_eq!(terms.limit, class.limit(), "{class:?}");
        assert!(terms.cadence.grace <= class.notice_grace(), "{class:?}");
        assert!(terms.cadence.grace <= terms.cadence.interval, "{class:?}");
        assert!(terms.cadence.grace < terms.limit, "{class:?}");
    }
}

/// A bound set below a class's grace still gets its first notice before the step is stopped,
///  and a bound above the grace leaves the grace alone.
#[test]
fn a_short_bound_pulls_the_first_notice_inside_it() {
    for class in [ChildClass::Plumbing, ChildClass::Object] {
        let interval: Duration = class.notice_grace() * 2;
        let short: Duration = class.notice_grace() / 2;
        assert!(grace_for(class, interval, short) < short, "{class:?}");
        assert_eq!(grace_for(class, interval, class.default_limit()), class.notice_grace(), "{class:?}");
    }
}

/// A sink that tells a channel each time a line is finished,
/// and keeps the text for the test.
struct LineSignal {
    /// The text written so far.
    shared: SharedSink,
    /// Told once per finished line.
    lines: std::sync::mpsc::Sender<()>,
}

/// Writes are copied and each finished line is signalled.
impl Write for LineSignal {
    /// Copy the bytes and signal the lines they finish.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        Write::write(&mut self.shared, bytes)?;
        for _line in bytes.iter().filter(|byte| { return **byte == b'\n'; }) {
            self.lines.send(()).ok();
        }
        return Ok(bytes.len());
    }

    /// Nothing is buffered.
    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// A wait that crosses the bound is announced,
/// then the child is stopped,
/// which ends the blocked wait.
///  The notice that is due goes out before the bound is checked,
/// so at least one notice is out
///  however slowly the host runs the watcher.
#[test]
fn a_wait_past_the_bound_is_announced_then_the_child_is_stopped() {
    let sink: SharedSink = SharedSink::default();
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30")).expect("sleeper");
    let watch: ChildWatch = ChildWatch::start("the sleeper", child.id(), terms(Duration::from_millis(450), &sink));
    watch.begin();
    let status = child.wait().expect("the stopped child is collected");
    watch.end();
    assert!(watch.expired());
    // A child that does not catch the request ends from it, in the grace, before any kill.
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    let text: String = sink.text();
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty(), "{text}");
    for line in &lines {
        assert!(line.starts_with("monochromatic-git-policy-cli: still waiting for the sleeper (process "), "{text}");
        assert!(line.contains("(CLI_GIT_CHILD_WAIT_SECONDS changes this)"), "{text}");
    }
    assert_eq!(text.matches("Hint:").count(), 1, "{text}");
}

/// A wait that ends inside the grace prints nothing and does not stop the child;
/// an idle watch prints nothing either.
///  The grace is an hour,
/// so no host is slow enough to make the wait outlast it.
#[test]
fn a_wait_that_ends_inside_the_grace_prints_nothing() {
    let sink: SharedSink = SharedSink::default();
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30")).expect("sleeper");
    let watch: ChildWatch = ChildWatch::start(
        "the sleeper",
        child.id(),
        WatchTerms {
            limit: Duration::from_secs(7200),
            cadence: Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            notices: Box::new(sink.clone()),
        },
    );
    std::thread::sleep(Duration::from_millis(50));
    watch.begin();
    std::thread::sleep(Duration::from_millis(50));
    watch.end();
    assert!(!watch.expired());
    assert_eq!(sink.text(), "");
    drop(watch);
    child.kill().expect("kill");
    child.wait().expect("collect");
}

/// A wait that has ended is not announced any more:
/// the watcher goes back to sleeping until the next wait begins.
///  The wait ends on its first notice.
/// Then the ended wait is offered turns of the watcher,
/// each an end mark that is read in order and that resets nothing,
/// and the
///  notices must stay at that count.
/// A watcher that went on counting the finished wait is due a notice on one of those turns,
/// however slowly
///  the host runs it,
/// because the next wait's notice cannot come before every mark above it has been read.
#[test]
fn a_wait_that_has_ended_is_announced_no_more() {
    let shared: SharedSink = SharedSink::default();
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30")).expect("sleeper");
    let watch: ChildWatch = ChildWatch::start(
        "the sleeper",
        child.id(),
        WatchTerms {
            limit: Duration::from_secs(3600),
            cadence: QUICK,
            notices: Box::new(LineSignal { shared: shared.clone(), lines: sender }),
        },
    );
    watch.begin();
    receiver.recv().expect("a notice announces the wait");
    watch.end();
    let settled: usize = shared.text().lines().count();
    assert!(settled > 0, "the waiting notice was announced: {:?}", shared.text());
    // The ended wait is offered more than two intervals of the watcher's cadence: a watcher that went on counting it
    //  prints on one of these turns, and one that went back to sleeping prints nothing.
    for turn in 0..INTERVALS_OFFERED {
        watch.end();
        std::thread::sleep(QUICK.interval);
        assert_eq!(
            shared.text().lines().count(),
            settled,
            "turn {turn} after the wait ended: a finished wait went on being announced: {}",
            shared.text()
        );
    }
    // The next wait is announced from its own start, which the watcher can only do after it has read every mark above.
    watch.begin();
    receiver.recv().expect("the next wait is announced");
    watch.end();
    let later: String = shared.text();
    let expired: bool = watch.expired();
    drop(watch);
    // Collected before the checks, so a failing check does not leave the sleeper running.
    child.kill().expect("kill");
    child.wait().expect("collect");
    assert_eq!(later.lines().count(), settled.saturating_add(1), "a finished wait went on being announced: {later}");
    assert!(!expired);
}

/// Each wait gets its own clock:
/// a second wait on the same child is announced from its own start,
/// with its own hint.
///  Each wait ends on its first notice,
/// so every wait has at least one notice,
/// however slowly the host runs the watcher.
#[test]
fn each_wait_is_announced_from_its_own_start() {
    let shared: SharedSink = SharedSink::default();
    let (sender, receiver) = std::sync::mpsc::channel::<()>();
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30")).expect("sleeper");
    let watch: ChildWatch = ChildWatch::start(
        "the sleeper",
        child.id(),
        WatchTerms {
            limit: Duration::from_secs(3600),
            cadence: QUICK,
            notices: Box::new(LineSignal { shared: shared.clone(), lines: sender }),
        },
    );
    for _wait in 0..2 {
        watch.begin();
        receiver.recv().expect("a notice announces the wait");
        watch.end();
    }
    let text: String = shared.text();
    assert_eq!(text.matches("Hint:").count(), 2, "the hint opens each wait: {text}");
    assert!(!watch.expired());
    drop(watch);
    child.kill().expect("kill");
    child.wait().expect("collect");
}

/// A command that runs long has its output collected untouched while the wait is announced.
///  The command waits for a release file that the second notice creates,
/// so the wait ends on that event and at least
///  two notices are out,
/// however slowly the host runs the watcher.
#[test]
fn a_command_that_runs_long_is_announced_and_its_output_is_kept() {
    let shared: SharedSink = SharedSink::default();
    let root: PathBuf = crate::test_support::fixture("child-watch-long");
    let release: PathBuf = root.join("release");
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("while [ ! -e \"$1\" ] && {}; do sleep 0.02; done; printf out; printf err >&2", crate::test_support::while_test_lives(root.as_path())))
        .arg("sh")
        .arg(&release)
        .stdin(Stdio::null());
    let release_file: PathBuf = release.clone();
    let counted: ActionSink = ActionSink::new(&shared, 2, move || {
        std::fs::write(&release_file, b"").expect("release the command");
    });
    let output: Output = output_watched(
        &mut command,
        "the slow command",
        WatchTerms { limit: Duration::from_secs(3600), cadence: QUICK, notices: Box::new(counted) },
    )
    .expect("finished");
    assert_eq!((output.stdout, output.stderr), (b"out".to_vec(), b"err".to_vec()));
    assert!(output.status.success());
    let text: String = shared.text();
    assert!(text.lines().count() >= 2, "{text}");
    assert!(text.starts_with("monochromatic-git-policy-cli: still waiting for the slow command (process "), "{text}");
    crate::test_support::remove(root.as_path());
}

/// A command that outlasts the bound is stopped and reported as a timeout that names it,
/// after at least one notice.
#[test]
fn a_command_past_the_bound_is_stopped_and_reported() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sleep");
    command.arg("30").stdin(Stdio::null());
    let failure: std::io::Error = output_watched(&mut command, "git rev-parse HEAD", terms(Duration::from_millis(400), &sink))
        .expect_err("stopped at the bound");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut);
    assert!(failure.to_string().starts_with("git rev-parse HEAD showed no sign of work for 0 seconds and was stopped"), "{failure}");
    assert!(sink.text().lines().count() >= 1, "{}", sink.text());
}

/// Whether `pid` is still the `sleep` of `seconds` that a control started:
/// the number may have been given to another process
/// since,
/// and only the helper is signalled.
fn is_the_helper(pid: i32, seconds: u64) -> bool {
    let Ok(line) = std::fs::read(format!("/proc/{pid}/cmdline")) else { return false };
    let words: Vec<&[u8]> = line.split(|byte| { return *byte == 0; }).filter(|word| { return !word.is_empty(); }).collect();
    return words == [b"sleep".as_slice(), seconds.to_string().as_bytes()];
}

/// Whether the process `pid` still runs.
/// A process that was killed and is waiting to be reaped is a zombie,
/// and `kill` with signal 0 succeeds for it.
/// On a host where the process that adopts an orphan is the user's service manager,
/// which reaps a moment after the signal landed,
/// the helper of the control below was a zombie when it was looked at.
/// The process table says what the process is:
/// a zombie has exited (`process_birth_identity` answers nothing for one),
/// so the observation does not depend on how fast the reaper is.
fn helper_is_running(pid: i32) -> bool {
    return !matches!(crate::process_identity::process_birth_identity(i64::from(pid)), Ok(None));
}

/// A killed process that nobody has reaped yet is not running,
/// though `kill` with signal 0 still reaches it,
/// and a process that runs is running.
/// This is the observation the control below makes of the helper a stopped step started.
/// The host's reaper makes the zombie window wide enough to fall into,
/// and this makes it deterministic.
#[test]
fn a_killed_process_that_is_not_yet_reaped_is_not_running() {
    let mut child: std::process::Child = crate::test_support::spawn_owned(Command::new("sleep").arg("30").stdin(Stdio::null())).expect("a process");
    let pid: i32 = i32::try_from(child.id()).expect("a process ID");
    assert!(helper_is_running(pid), "a sleeping process runs");
    // SAFETY: `kill` takes two integers and touches no memory of this process.
    unsafe { libc::kill(pid, libc::SIGKILL) };
    let began: Instant = Instant::now();
    while std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|text| { return !text.contains(") Z "); }) {
        assert!(began.elapsed() < Duration::from_secs(10), "the process never became a zombie");
        std::thread::sleep(Duration::from_millis(2));
    }
    // SAFETY: `kill` with signal 0 only checks that the process exists.
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0, "the premise: a zombie still answers signal 0");
    assert!(!helper_is_running(pid), "a zombie has exited");
    child.wait().expect("reaped");
    assert!(!helper_is_running(pid), "a reaped process is gone");
}

/// What a command that started a process of its own must not leave behind when it is stopped at the bound:
/// a step of a commit
///  (`git gc`,
/// a `git push` with its transport) has helpers,
/// and a helper that outlives its step holds the step's output open,
///  so the wait for the output outlasts the bound by as long as the helper lives.
#[test]
fn a_command_past_the_bound_is_stopped_with_the_processes_it_started() {
    let root: PathBuf = crate::test_support::fixture("child-watch-tree");
    let helper_file: PathBuf = root.join("helper-pid");
    let helper_seconds: u64 = 25;
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("sleep {helper_seconds} & echo $! > '{}'; wait", helper_file.display()))
        .stdin(Stdio::null());
    let started: Instant = Instant::now();
    let failure: std::io::Error = output_watched(&mut command, "a step with a helper", terms(Duration::from_millis(600), &sink))
        .expect_err("stopped at the bound");
    let waited: Duration = started.elapsed();
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut);
    let helper: i32 = std::fs::read_to_string(&helper_file).expect("the helper's process ID").trim().parse().expect("a process ID");
    // A process that was killed stays a zombie until whoever inherited it has waited for it, and a zombie is not running
    //  (`helper_is_running`). The kill lands before the process is scheduled to die, so the control waits for that event, with a
    //  patience that only a helper that was never stopped reaches.
    let running = || -> bool { return helper_is_running(helper) && is_the_helper(helper, helper_seconds); };
    let patience: Instant = Instant::now();
    while running() && patience.elapsed() < Duration::from_secs(60) {
        std::thread::sleep(Duration::from_millis(20));
    }
    let helper_alive: bool = running();
    // The assertions come after the cleanup, so a failure leaves no helper running. Only a process that is still the helper is
    //  signalled: its number may have been given to another process since.
    if helper_alive {
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        unsafe { libc::kill(helper, libc::SIGKILL) };
    }
    assert!(
        waited < Duration::from_secs(helper_seconds / 2),
        "the stop took {waited:?}, as long as the helper that kept the output open lived"
    );
    assert!(!helper_alive, "the helper the step started is still running after the step was stopped");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A root nothing could be pinned for is not signalled at all,
/// and says so.
/// Why:
///  A record that could not be read says nothing about which process holds the number,
/// and a number that was not
///  pinned may name an unrelated process by now.
/// The stop says which signal the step was not sent,
/// and finding 1 of
///  the third review was a signal sent to whatever held the root's number *after* the root was reaped.
#[cfg(target_os = "linux")]
#[test]
fn a_root_that_could_not_be_pinned_is_not_signalled() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a sleeper");
    let pid: i32 = i32::try_from(sleeper.id()).expect("a process number");
    assert!(helper_is_running(pid), "the premise: the process runs");
    // The state of a stop whose read proved nothing about the root.
    let delivered: crate::member_signalled_through_its_pidfd::Delivered = crate::child_watch_clock_marks_and_stopping::send_to_the_root(
        &crate::member_signalled_through_its_pidfd::Pinned::Unproven,
        pid,
        libc::SIGKILL,
    );
    std::thread::sleep(Duration::from_millis(300));
    let survived: bool = helper_is_running(pid);
    sleeper.kill().expect("the control ends its sleeper");
    sleeper.wait().expect("the sleeper is collected");
    assert_eq!(delivered, crate::member_signalled_through_its_pidfd::Delivered::Unproven, "a signal nothing was pinned for is not taken");
    assert!(survived, "a number nothing was pinned for was signalled: it may have named an unrelated process");
}

/// What:
///  The root's pin is taken where the child starts,
/// and no stop pins a record it reads at stopping time.
/// Why:
///  A pin taken at stopping time pins whoever holds the number then,
/// and the thread that waits for the child reaps it as soon as it
///  ends,
/// so the child's number can name another command by the moment the stop begins
/// (the third review's finding 1).
/// The watch takes the pin at the start,
/// while the child is this process's own unreaped
///  child,
/// and every signal of the stop goes through that pin.
#[test]
fn the_root_is_pinned_when_its_child_starts() {
    let watch: &str = include_str!("child_watch.rs");
    let stop: &str = include_str!("child_watch_clock_marks_and_stopping.rs");
    let watch_code: String = watch.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    let stop_code: String = stop.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
    assert!(watch_code.contains("pin_at_start("), "the watch does not pin its child where the child starts");
    // The needle is built at run time, so this control's own text does not contain it.
    let at_stop: String = format!("pin_of_the{}", "_root");
    assert!(!stop_code.contains(at_stop.as_str()), "a stop still pins a record it reads at stopping time");
}

/// What:
///  A step's root is pinned when the child starts,
/// and that pin never follows the number.
/// Why:
///  The third review's finding 1 was a pin taken at stopping time:
/// by then the child can have ended and been reaped,
/// and the number can name another command,
///  which the stop would then signal.
/// A pin taken while the child is this process's own unreaped child names that
///  child for good,
/// so once the child has ended the pin sends nothing,
/// whatever holds the number afterwards.
#[cfg(target_os = "linux")]
#[test]
fn a_root_pinned_at_its_start_never_follows_the_number() {
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a child of this control");
    let pid: u32 = child.id();
    let started: i32 = i32::try_from(pid).expect("a process number");
    // The pin is taken while this process's child is running and has not been reaped: the state of a watch at its start.
    let pin: crate::child_watch_clock_marks_and_stopping::RootPin = crate::child_watch_clock_marks_and_stopping::pin_at_start(pid);
    assert!(
        matches!(pin, crate::member_signalled_through_its_pidfd::Pinned::Held(_)),
        "the child that was just started was not pinned"
    );
    // The child ends and is reaped, so its number is free for another process; the pin still names the child that ended.
    child.kill().expect("the control ends its child");
    child.wait().expect("the child is collected");
    let delivered: crate::member_signalled_through_its_pidfd::Delivered =
        crate::child_watch_clock_marks_and_stopping::send_to_the_root(&pin, started, libc::SIGKILL);
    assert_eq!(
        delivered,
        crate::member_signalled_through_its_pidfd::Delivered::Gone,
        "the pin of a child that has ended signalled something"
    );
}

/// What:
///  A step's output that a program the step started still holds open ends the wait one drain after the step,
///  and the failure names the program.
/// Why:
///   A pipe ends at end of file,
/// which needs every write end closed,
/// so a program that inherited the step's streams and outlived
///  it kept the read open for as long as it lived,
/// past the step's ceiling,
/// with the command holding its locks and its
///  transaction meanwhile.
/// The holder lives longer than the bound of the control (eight seconds against three),
///  so a wait that outlasts the step shows.
#[test]
fn a_pipe_a_program_the_step_started_holds_ends_the_wait_and_is_named() {
    let root: PathBuf = crate::test_support::fixture("child-watch-held-output");
    let holder_file: PathBuf = root.join("holder-pid");
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg(format!("sleep 8 & echo $! > '{}'", holder_file.display())).stdin(Stdio::null());
    let began: Instant = Instant::now();
    let failure: std::io::Error = output_watched_within(
        &mut command,
        "a step with a holder",
        "a step with a holder",
        terms(Duration::from_secs(3600), &sink),
        true,
        StepBound::EndAt(Duration::from_secs(60)),
    )
    .expect_err("the output a program held open is not the step's answer");
    let waited: Duration = began.elapsed();
    let holder: i32 = std::fs::read_to_string(&holder_file).expect("the holder's process ID").trim().parse().expect("a process ID");
    // The assertions come after the cleanup, so a failure leaves nothing running. Only the helper the step started is signalled.
    if helper_is_running(holder) && is_the_helper(holder, 8) {
        // SAFETY: `kill` takes two integers and touches no memory of this process.
        unsafe { libc::kill(holder, libc::SIGKILL) };
    }
    assert!(
        crate::step_output_read_whole_through_pipes::is_output_held_open(&failure),
        "the wait names the program that held the output open: {failure}"
    );
    assert!(failure.to_string().contains("still holds the step's output open"), "{failure}");
    assert!(failure.to_string().contains(format!("process {holder}").as_str()), "the holder is named by its number: {failure}");
    assert!(waited < Duration::from_secs(3), "the wait outlasted the step by {waited:?}, as long as the holder lived");
    crate::test_support::remove(root.as_path());
}

/// What:
///  A step that printed its whole answer and ended succeeds though a program it started still holds its standard error open,
/// and what it printed to standard error before the drain ended comes back with it.
/// Why:
///  A step's answer is its exit status and its standard output,
/// so a program that inherited only the standard error must not
///  decide that the step failed and take the answer away with it.
/// A program that holds the standard output open still yields no
///  answer at all,
/// which the control above this one checks.
/// The holder ends by itself (`sleep 4`),
/// so nothing here is signalled.
#[test]
fn a_step_whose_standard_error_is_held_open_returns_its_answer() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    // The helper inherits the step's standard error alone: its standard output is the step's answer and closes with the step.
    command.arg("-c").arg("sleep 4 >/dev/null & printf 'answer\\n'; printf 'warned\\n' >&2").stdin(Stdio::null());
    let began: Instant = Instant::now();
    let output: std::process::Output = output_watched_within(
        &mut command,
        "a step with a helper on its standard error",
        "a step with a helper on its standard error",
        terms(Duration::from_secs(3600), &sink),
        true,
        StepBound::EndAt(Duration::from_secs(60)),
    )
    .expect("the step's exit status and standard output are its answer");
    let waited: Duration = began.elapsed();
    assert!(output.status.success(), "the step succeeded: {:?}", output.status);
    assert_eq!(output.stdout, b"answer\n".to_vec(), "the whole answer comes back");
    assert_eq!(output.stderr, b"warned\n".to_vec(), "what the step printed to standard error before the drain ended comes back");
    assert!(waited < Duration::from_secs(3), "the wait outlasted the step by {waited:?}, as long as the holder lived");
}

/// What:
///  A step whose watch could not start still ends at its ceiling,
/// and the failure says no thread could watch it.
/// Why:
///   `spawn_on_runtime` returning nothing left the step with no task at all:
/// nothing announced it,
/// nothing ended it,
///  and the wait was unbounded,
/// so a step that keeps quiet was waited for until it ended by itself,
/// with nothing saying why
///  (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 8).
/// The fixture would outlive the control's own bound if the ceiling were not carried,
/// so a wait that outlasts the step shows.
#[test]
fn a_step_whose_watch_had_no_task_ends_at_its_ceiling_and_says_so() {
    let refused: crate::lazy_runtime::RefusalForTheControl = crate::lazy_runtime::a_control_refuses_every_task();
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sleep");
    command.arg("6").stdin(Stdio::null());
    let began: Instant = Instant::now();
    let failure: std::io::Error = output_watched_within(
        &mut command,
        "a step nobody watches",
        "a step nobody watches",
        terms(Duration::from_secs(3600), &sink),
        true,
        StepBound::EndAt(Duration::from_millis(700)),
    )
    .expect_err("the ceiling ended it");
    let waited: Duration = began.elapsed();
    drop(refused);
    // The positive control: a worker can run the watch, and the step ends at the ceiling through it.
    let mut watched: Command = Command::new("sleep");
    watched.arg("6").stdin(Stdio::null());
    let began_watched: Instant = Instant::now();
    let through_the_watch: std::io::Error = output_watched_within(
        &mut watched,
        "a step a watch follows",
        "a step a watch follows",
        terms(Duration::from_secs(3600), &sink),
        true,
        StepBound::EndAt(Duration::from_millis(700)),
    )
    .expect_err("the ceiling ended it");
    let waited_watched: Duration = began_watched.elapsed();
    assert!(is_ceiling(&through_the_watch), "the watch ends the step at its ceiling: {through_the_watch}");
    assert!(waited_watched < Duration::from_secs(3), "the watched step ended at its ceiling after {waited_watched:?}");
    assert!(waited < Duration::from_secs(3), "a step nobody could watch outlasted its ceiling by {waited:?}");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(failure.to_string().contains("refused a thread"), "the failure says no thread could watch the step: {failure}");
    assert!(failure.to_string().contains("a step nobody watches"), "the failure names the step: {failure}");
}

/// A command that finishes at once prints nothing;
/// the grace is an hour,
/// so no host is slow enough to outlast it.
#[test]
fn a_command_that_finishes_at_once_prints_nothing() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("true");
    command.stdin(Stdio::null());
    let output: Output = output_watched(
        &mut command,
        "true",
        WatchTerms {
            limit: Duration::from_secs(7200),
            cadence: Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            notices: Box::new(sink.clone()),
        },
    )
    .expect("finished");
    assert!(output.status.success());
    assert_eq!(sink.text(), "");
}

/// The quiet window of the controls below:
/// past the sampling interval of the meter,
/// so a child that works shows its work
///  at a sample before the window runs out.
const WORKING_WINDOW: Duration = Duration::from_millis(1200);

/// How often the working fixture shows a sign of work:
/// well inside the window.
const WORKING_PAUSE_SECONDS: &str = "0.2";

/// How many times the working fixture works,
/// so that it runs for a multiple of the window.
const WORKING_ROUNDS: u32 = 15;

/// A shell command that writes a byte and pauses,
/// `WORKING_ROUNDS` times:
/// a slow child that keeps making progress,
/// with no busy loop.
fn working_command() -> Command {
    let mut command: Command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("n=0; while [ $n -lt {WORKING_ROUNDS} ]; do printf x; sleep {WORKING_PAUSE_SECONDS}; n=$((n + 1)); done"))
        .stdin(Stdio::null());
    return command;
}

/// A child that keeps making progress is never ended however long it runs:
/// the window judges quiet time,
/// not wall time.
///  The fixture runs for a multiple of the window and shows a sign of work every pause.
#[test]
fn a_child_that_keeps_working_is_never_ended_however_long_it_runs() {
    let sink: SharedSink = SharedSink::default();
    let started: Instant = Instant::now();
    let output: Output = output_watched(&mut working_command(), "a working step", terms(WORKING_WINDOW, &sink))
        .expect("a step that keeps working is never ended");
    assert!(started.elapsed() > WORKING_WINDOW * 2, "the fixture ran only {:?}", started.elapsed());
    assert_eq!(output.stdout.len(), usize::try_from(WORKING_ROUNDS).expect("rounds fit"));
    assert!(output.status.success());
}

/// A silent child is ended once it has shown no sign of work for the window,
/// and the processes beside it are left alone:
///  the stop reaches the child and what it started,
/// never a sibling.
#[test]
fn a_silent_child_is_ended_after_the_quiet_window_and_a_sibling_is_left_alone() {
    let sink: SharedSink = SharedSink::default();
    let mut sibling: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("31").stdin(Stdio::null())).expect("sibling");
    let mut command: Command = Command::new("sleep");
    command.arg("30").stdin(Stdio::null());
    let failure: std::io::Error = output_watched(&mut command, "a silent step", terms(WORKING_WINDOW, &sink))
        .expect_err("a silent step is ended");
    let sibling_ended: bool = sibling.try_wait().expect("look at the sibling").is_some();
    sibling.kill().expect("kill the sibling");
    sibling.wait().expect("collect the sibling");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut);
    assert!(failure.to_string().contains("showed no sign of work for 1 seconds"), "{failure}");
    assert!(!sibling_ended, "the stop reached a process beside the step");
}

/// A command that keeps showing work (it writes a little every moment) is ended at the ceiling,
/// though the quiet window is an hour:
/// the step that works is no longer waited for however long it takes.
/// The error is the ceiling's,
/// which a step that may be run
/// again is not run again after.
#[test]
fn a_step_that_keeps_working_is_ended_at_its_ceiling() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.args(["-c", "while true; do echo working; sleep 0.05; done"]);
    let began: Instant = Instant::now();
    let error = output_watched_within(&mut command, "a working step", "a working step", terms(Duration::from_secs(3600), &sink), true, StepBound::EndAt(Duration::from_millis(700)))
        .expect_err("the ceiling ended it");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(is_ceiling(&error), "{error}");
    assert!(error.to_string().contains("a working step ran for") && error.to_string().contains("was ended"), "{error}");
    assert!(began.elapsed() >= Duration::from_millis(700) && began.elapsed() < Duration::from_secs(30), "{:?}", began.elapsed());
}

/// A step that may not be ended for being quiet (a reference update) is ended at the ceiling all the same.
#[test]
fn a_step_that_is_never_ended_for_being_quiet_is_ended_at_its_ceiling() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sleep");
    command.arg("600");
    let began: Instant = Instant::now();
    let error = output_watched_within(&mut command, "a quiet step", "a quiet step", terms(Duration::from_millis(200), &sink), false, StepBound::EndAt(Duration::from_millis(700)))
        .expect_err("the ceiling ended it");
    assert!(is_ceiling(&error), "{error}");
    assert!(began.elapsed() < Duration::from_secs(30), "{:?}", began.elapsed());
}

/// A step with no ceiling (a write in place that is not safe to run twice) is waited for as long as it lives,
/// though a ceiling of the
/// length that ended the control above would have ended it,
/// and though it was quiet for longer than the window:
/// it is never ended
/// for being quiet either,
/// because it may not be run twice.
#[test]
fn a_step_with_no_ceiling_is_waited_for_until_it_ends_by_itself() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.args(["-c", "sleep 2; echo done"]);
    let output = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_millis(300), &sink), false, StepBound::Unbounded).expect("waited for");
    assert!(String::from_utf8_lossy(output.stdout.as_slice()).contains("done"));
}

/// A step left running:
/// the command stops waiting at its bound plus the extra wait and gets the error that says the result is not
/// known,
/// the step is not ended,
/// and it is reaped when it ends by itself.
#[test]
fn a_step_that_outlasts_its_bound_and_the_extra_wait_is_left_running() {
    let sink: SharedSink = SharedSink::default();
    let directory: PathBuf = crate::test_support::fixture("child-watch-left-running");
    let pid_file: PathBuf = directory.join("pid");
    let mut command: Command = Command::new("sh");
    command.args(["-c", format!("echo $$ > '{}'; sleep 20", pid_file.display()).as_str()]);
    let bound: StepBound = StepBound::LeaveRunning { after: Duration::from_millis(500), extra: Duration::from_millis(500) };
    let began: Instant = Instant::now();
    let error = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_secs(3600), &sink), false, bound).expect_err("the command stopped waiting");
    let waited: Duration = began.elapsed();
    assert!(is_not_known(&error) && error.kind() == std::io::ErrorKind::TimedOut, "{error}");
    assert!(error.to_string().contains("a write") && error.to_string().contains("left running"), "{error}");
    assert!(waited >= Duration::from_secs(1) && waited < Duration::from_secs(10), "{waited:?}");
    let pid: i32 = std::fs::read_to_string(&pid_file).expect("the step recorded its process").trim().parse().expect("a process number");
    // SAFETY: signal 0 only asks whether the process exists.
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0, "the step was not ended");
    // SAFETY: the control ends the process it started, by the number it recorded.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    crate::test_support::remove(directory.as_path());
}

/// A step that ends within the extra wait is not an error:
/// its output is the answer.
#[test]
fn a_step_that_ends_within_the_extra_wait_returns_its_output() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.args(["-c", "sleep 1; echo done"]);
    let bound: StepBound = StepBound::LeaveRunning { after: Duration::from_millis(500), extra: Duration::from_secs(20) };
    let output = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_secs(3600), &sink), false, bound).expect("finished within the extra wait");
    assert!(String::from_utf8_lossy(output.stdout.as_slice()).contains("done"));
}

/// A quiet step is ended for being quiet,
/// and that error is not the ceiling's.
#[test]
fn a_quiet_step_is_not_reported_as_the_ceiling() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sleep");
    command.arg("600");
    let error = output_watched_within(&mut command, "a quiet step", "a quiet step", terms(Duration::from_millis(300), &sink), true, StepBound::EndAt(Duration::from_secs(600)))
        .expect_err("the quiet window ended it");
    assert!(!is_ceiling(&error), "{error}");
}

/// The variable that makes `left_running_helper` do its work:
/// the test that spawns it names a directory.
const HELPER_DIRECTORY: &str = "CLI_GIT_LEFT_RUNNING_DIRECTORY";

/// The command that leaves a step running and then exits,
/// run in a process of its own by the control below.
/// A test run without the
/// variable does nothing,
/// so this is a no-op in a normal run.
#[test]
fn left_running_helper() {
    let Some(named) = std::env::var_os(HELPER_DIRECTORY) else { return };
    let directory: PathBuf = PathBuf::from(named);
    let script: PathBuf = directory.join("step.sh");
    let mut command: Command = Command::new("sh");
    command.arg(script.as_path());
    let sink: SharedSink = SharedSink::default();
    let bound: StepBound = StepBound::LeaveRunning { after: Duration::from_millis(300), extra: Duration::from_millis(300) };
    let error = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_secs(3600), &sink), false, bound).expect_err("the command stopped waiting");
    assert!(is_not_known(&error), "{error}");
    // The command exits now, with the step still running, as the wrapper does after it reports that the result is not known.
    std::process::exit(0);
}

/// "Left running" survives the exit of the command (sol-1:
/// the pipes the command held closed with it,
/// and the step died of SIGPIPE
/// when it wrote):
/// the step is started with output that outlives the command,
/// the command is a real process that exits,
/// and the step
/// writes after that and finishes.
/// The control runs the command as a separate process of this test binary.
#[test]
fn a_step_left_running_survives_the_exit_of_the_command_that_left_it() {
    let directory: PathBuf = crate::test_support::fixture("child-watch-survives-exit");
    let release: PathBuf = directory.join("release");
    let finished: PathBuf = directory.join("finished");
    let pid_file: PathBuf = directory.join("pid");
    std::fs::write(
        directory.join("step.sh"),
        format!(
            "echo $$ > '{pid}'\nwhile [ ! -e '{release}' ] && [ -d '{directory}' ]; do sleep 0.05; done\necho late-output || exit 3\necho late-error >&2 || exit 4\n: > '{finished}'\n",
            pid = pid_file.display(),
            release = release.display(),
            directory = directory.display(),
            finished = finished.display()
        ),
    )
    .expect("the step");
    // The test binary is the program this process was started as (the test harness starts it by its path).
    let test_binary: PathBuf = PathBuf::from(std::env::args_os().next().expect("the program this test runs as"));
    let mut running = Command::new(test_binary)
        .args(["--exact", "push::child_watch::tests::left_running_helper", "--nocapture"])
        .env(HELPER_DIRECTORY, directory.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the command runs");
    // A command that waits for the step it left (the defect under test) must fail this test and not hang it (sol-1 round 2).
    let command_began: Instant = Instant::now();
    let helper = loop {
        if let Some(status) = running.try_wait().expect("ask the command") {
            break status;
        }
        if command_began.elapsed() > Duration::from_secs(30) {
            let _ended = running.kill();
            let _reaped = running.wait();
            panic!("the command did not exit after it left the step running");
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(helper.success(), "the command that left the step running exited cleanly: {helper:?}");
    let pid: i32 = std::fs::read_to_string(&pid_file).expect("the step recorded its process").trim().parse().expect("a process number");
    // SAFETY: signal 0 only asks whether the process exists.
    assert_eq!(unsafe { libc::kill(pid, 0) }, 0, "the step outlived the command");
    std::fs::write(&release, b"").expect("let the step go on");
    let waiting_since: Instant = Instant::now();
    while !finished.exists() {
        assert!(waiting_since.elapsed() < Duration::from_secs(20), "the step died when it wrote after the command had exited");
        std::thread::sleep(Duration::from_millis(50));
    }
    crate::test_support::remove(directory.as_path());
}

/// What a step wrote is released once it is read (sol-1 round 3,
/// P2 E):
/// a descendant that keeps the files open cannot keep the pages
/// the reader no longer needs.
#[test]
fn the_output_of_a_step_is_released_once_it_is_read() {
    let capture: Capture = Capture::new().expect("the files");
    let (out, err) = capture.stdio().expect("the streams");
    let status = Command::new("sh").args(["-c", "echo kept-for-the-reader; echo and-this >&2"]).stdout(out).stderr(err).status().expect("the child");
    let output: Output = output_of(status, Some(&capture));
    assert!(String::from_utf8_lossy(output.stdout.as_slice()).contains("kept-for-the-reader"));
    let again: crate::output_capture::Captured = capture.snapshot(1024);
    assert!(again.stdout.is_empty() && again.stderr.is_empty(), "the files were emptied after the read");
}

/// What:
///  The wait after the kill ends,
/// and it still gives the grace the stop promises.
/// Why:
///  The wait after the kill had no bound of its own,
/// so a member whose record could not be read kept the stop waiting for as
///  long as the process lived against a kill that was never sent.
/// The three answers this has to give are the wait inside the grace,
/// the wait
///  past the grace with nothing in uninterruptible sleep,
/// and the wait past the stop's ceiling.
#[cfg(target_os = "linux")]
#[test]
fn the_wait_after_the_kill_is_bounded() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a sleeper");
    let pid: i32 = i32::try_from(sleeper.id()).expect("a process number");
    let member: crate::process_tree::TreeMember = match crate::process_tree::member_read(pid) {
        crate::process_tree::MemberRead::Found { member, .. } => member,
        other => panic!("the control's process is found: {other:?}"),
    };
    let tree: [crate::process_tree::TreeMember; 1] = [member];
    let inside: bool = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(Duration::from_millis(1), tree.as_slice());
    let past_the_grace: bool = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(
        crate::bounded_push::STOP_GRACE + Duration::from_secs(1),
        tree.as_slice(),
    );
    let past_the_ceiling: bool = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(
        crate::stop_waits_while_a_member_cannot_be_interrupted::STOP_CEILING + Duration::from_secs(1),
        tree.as_slice(),
    );
    let nothing: bool = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(Duration::from_millis(1), &[]);
    sleeper.kill().expect("the control ends its sleeper");
    sleeper.wait().expect("the sleeper is collected");
    assert!(inside, "a stop inside the grace after the kill waited for nothing");
    assert!(!past_the_grace, "a stop waited past its grace with nothing left to wait for");
    assert!(!past_the_ceiling, "a stop waited past its own ceiling");
    assert!(!nothing, "a stop with nothing left in its tree waited");
}

/// What:
///  A step whose watch cannot prove which process to stop is still ended at its ceiling,
/// and says so.
/// Why:
///  A watch exists,
/// so the wait for the output used to carry no ceiling of its own:
/// when the watch's own signals
///  proved nothing,
/// the step ran for as long as it lived,
/// which is the third review's finding 2.
/// Whatever the
///  watch could not do,
/// the wait for the output keeps the step's ceiling.
///  The watch's descriptor is refused here,
/// which is the state its signals cannot leave:
/// `refuse_the_next_descriptors` is
///  armed on this thread,
/// where the pin of the child is taken.
/// The step runs on a thread of its own and is taken with a
///  deadline,
/// so a wait that never ends is reported here rather than hanging the run.
#[test]
fn a_step_the_watch_cannot_end_is_ended_at_its_ceiling() {
    let sink: SharedSink = SharedSink::default();
    let ceiling: Duration = Duration::from_millis(300);
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("sleep 30").stdin(Stdio::null());
    let (sender, receiver) = std::sync::mpsc::channel::<std::io::Result<std::process::Output>>();
    let running = std::thread::Builder::new()
        .name(String::from("child-watch-ceiling-control"))
        .spawn(move || {
            // The descriptor of the pin the watch takes at the child's start is refused: nothing can prove which process to
            //  stop, so the watch cannot end this step.
            crate::member_signalled_through_its_pidfd::refuse_the_next_descriptors(1);
            let _abandoned = sender.send(output_watched_within(
                &mut command,
                "a step the watch cannot end",
                "a step the watch cannot end",
                terms(ceiling, &sink),
                true,
                StepBound::EndAt(ceiling),
            ));
        })
        .expect("the control's thread");
    let began: Instant = Instant::now();
    let answer: Result<std::io::Result<std::process::Output>, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(Duration::from_secs(20));
    let waited: Duration = began.elapsed();
    let _joined = running.join();
    let failure: std::io::Error = answer
        .expect("the step ended; a step the watch cannot end ran until this control's deadline")
        .expect_err("a step past its ceiling is not an answer");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    // The step reached its ceiling and was ended there; whether the sentence names the watch or the wait that carried the
    //  ceiling, it must say which limit the step met.
    assert!(failure.to_string().contains("the longest the wrapper lets one step run"), "{failure}");
    assert!(waited < Duration::from_secs(10), "the step outlasted its ceiling by {waited:?}");
}

/// What:
///  A member in uninterruptible I/O cannot keep the stop waiting past its ceiling,
/// and one that can act on its signal is not waited for after the grace at all.
/// Why:
///  After the escalation ceiling and the kill,
/// a loop with no elapsed-time check of its own waited for as long as a
///  member sat in uninterruptible I/O,
/// which is the third review's finding 2 in one place and the locks-and-transactions
///  review's finding in another:
/// the stop waited without a bound after the last signal it could send.
/// The state cannot be
///  produced on this host,
/// so the control says which members count as uninterruptible and how long the
///  ceiling is (`stop_waits_while_a_member_cannot_be_interrupted`).
#[cfg(target_os = "linux")]
#[test]
fn a_member_in_uninterruptible_io_cannot_hold_the_stop_past_its_ceiling() {
    let mut sleeper: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a sleeper");
    let pid: i32 = i32::try_from(sleeper.id()).expect("a process number");
    let member: crate::process_tree::TreeMember = match crate::process_tree::member_read(pid) {
        crate::process_tree::MemberRead::Found { member, .. } => member,
        other => panic!("the control's process is found: {other:?}"),
    };
    let tree: [crate::process_tree::TreeMember; 1] = [member];
    // The stop's ceiling is a multiple of the longest delay measured, so it sits above the grace; a control keeps that order.
    let ceiling: Duration = crate::bounded_push::STOP_GRACE * 2;
    let inside_grace: bool;
    let after_the_grace: bool;
    let past_the_ceiling: bool;
    {
        let _rules = crate::stop_waits_while_a_member_cannot_be_interrupted::rules_for_this_thread(
            crate::stop_waits_while_a_member_cannot_be_interrupted::ControlRules {
                blocked: |_member| { return true; },
                ceiling,
            },
        );
        inside_grace = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(Duration::ZERO, tree.as_slice());
        after_the_grace =
            crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(crate::bounded_push::STOP_GRACE, tree.as_slice());
        past_the_ceiling = crate::child_watch_clock_marks_and_stopping::the_wait_after_the_kill(ceiling + Duration::from_millis(1), tree.as_slice());
    }
    sleeper.kill().expect("the control ends its sleeper");
    sleeper.wait().expect("the sleeper is collected");
    assert!(inside_grace, "the stop did not wait at all after the kill");
    assert!(after_the_grace, "a member in uninterruptible I/O was given up at the grace rather than at the ceiling");
    assert!(!past_the_ceiling, "a member in uninterruptible I/O held the stop past its ceiling");
}

/// What:
///  A step that finished inside its ceiling is answered,
/// though a program it started still holds its standard error open.
/// Why:
///  The fallback ceiling — the one this wait carries when no watch could run — wrapped the wait for the child
/// *and* the
///  reads of its streams,
/// so a step that ended inside the ceiling was reported as ended at it when a program held its
///  standard error past the ceiling
/// (the third review's finding 5).
/// The ceiling bounds the wait for the child;
/// the
///  drain after its end is bounded by `OUTPUT_DRAIN_AFTER_EXIT`,
/// which is much shorter.
/// The ceiling here is below
///  that drain,
/// and no watch runs,
/// so the state the review describes is the one this step is in.
#[test]
fn a_step_that_finishes_inside_its_ceiling_is_answered() {
    let refused: crate::lazy_runtime::RefusalForTheControl = crate::lazy_runtime::a_control_refuses_every_task();
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    // The helper holds the standard error alone, and the step finishes at once with its whole answer on the standard output.
    command.arg("-c").arg("sleep 6 >/dev/null & printf answer; exit 0").stdin(Stdio::null());
    let began: Instant = Instant::now();
    let output: std::process::Output = output_watched_within(
        &mut command,
        "a step with a holder on its standard error",
        "a step with a holder on its standard error",
        terms(Duration::from_secs(3600), &sink),
        true,
        StepBound::EndAt(Duration::from_millis(500)),
    )
    .expect("the step's exit status and standard output are its answer");
    let waited: Duration = began.elapsed();
    drop(refused);
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(output.stdout, b"answer".to_vec(), "the step's answer comes back");
    assert!(waited < Duration::from_secs(3), "the step was reported at its ceiling after {waited:?}");
}

/// What:
///  A watched step that finished inside its ceiling is answered,
/// though the ceiling passes while its standard error is still held open.
/// Why:
///  A watcher exists here,
/// so the watch's own ceiling fires while the wait is in its post-exit drain,
/// and
///  `ending_of_a_watched_step` replaced the collected status with `CeilingReached`
/// (the fourth review's finding 2).
///  A status already collected decides:
/// the pin of the child reports it has ended,
/// so nothing of the stop
///  replaces its answer.
#[test]
fn a_watched_step_that_finished_inside_its_ceiling_is_answered() {
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.arg("-c").arg("sleep 6 >/dev/null & printf answer; exit 0").stdin(Stdio::null());
    let began: Instant = Instant::now();
    let output: std::process::Output = output_watched_within(
        &mut command,
        "a watched step with a holder on its standard error",
        "a watched step with a holder on its standard error",
        terms(Duration::from_secs(3600), &sink),
        true,
        // Below the one second the read of a held stream may go on after the step ended: the ceiling passes during that drain.
        StepBound::EndAt(Duration::from_millis(300)),
    )
    .expect("the step's exit status and standard output are its answer");
    let waited: Duration = began.elapsed();
    assert!(output.status.success(), "{:?}", output.status);
    assert_eq!(output.stdout, b"answer".to_vec(), "the step's answer comes back");
    assert!(waited < Duration::from_secs(3), "the step was reported at its ceiling after {waited:?}");
}

/// What:
///  Each stream of a step is looked for through its own pipe,
/// so a program holding only the standard error is found there.
/// Why:
///  The identity of the standard output's pipe was given to the reader of the standard error too,
/// so a program holding only
///  the standard error was never found,
/// and the sentence said it no longer shows in the process table
/// (the third review's finding 7).
/// The first half of this control reads the module's own wiring;
/// the second shows that
///  the identity of a pipe is what finds the program holding it,
/// so the reader that is given the wrong pipe cannot
///  find that program at all.
#[cfg(target_os = "linux")]
#[test]
fn each_stream_is_found_by_its_own_pipe() {
    let module: String = include_str!("../process/step_output_read_whole_through_pipes.rs")
        .lines()
        .filter(|line| { return !line.trim_start().starts_with("//"); })
        .collect::<Vec<&str>>()
        .join("\n");
    assert!(module.contains("standard_output.as_fd()"), "the standard output's pipe identity is not taken from it");
    assert!(module.contains("standard_error.as_fd()"), "the standard error's pipe identity is not taken from it");
    // A program of this control holds the write end of one pipe as its standard error and writes nothing into it.
    let (error_read, error_write) = std::io::pipe().expect("a pipe");
    let mut holder: Child = Command::new("sleep")
        .arg("6")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(error_write))
        .spawn()
        .expect("a holder of the pipe");
    let holder_pid: i32 = i32::try_from(holder.id()).expect("a process number");
    let identity: Option<(u64, u64)> = crate::step_output_read_whole_through_pipes::pipe_identity(error_read.as_fd());
    let named: Vec<i64> = match identity {
        Some((mount, inode)) => {
            crate::process_use::processes_using(&crate::process_use::UseTarget::Mounted { mount, inode }).users.iter().map(|user| { return user.pid; }).collect()
        }
        None => Vec::new(),
    };
    let _ended = holder.kill();
    let _reaped = holder.wait();
    assert!(named.contains(&i64::from(holder_pid)), "the program holding the pipe was not found through that pipe: {named:?}");
}

/// What:
///  A root the pin reports gone has no tree read for it,
/// and a root that is still there has one.
/// Why:
///  Once the root is reaped its number may name another process,
/// and the children of that process are not this step's;
/// enumerating them would signal a command of the person's that has nothing to do with the stop
/// (the fourth review's finding 1).
/// The control process below stands for whatever took the number.
#[cfg(target_os = "linux")]
#[test]
fn the_tree_of_a_root_that_has_ended_is_not_read() {
    // A control process with a child of its own: the tree a reused number could offer.
    let mut shell: Child = {
        use std::os::unix::process::CommandExt as _;
        Command::new("sh")
        .args(["-c", "sleep 600 & echo $!; wait"])
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the control's process")
    };
    let mut line: String = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(shell.stdout.take().expect("piped stdout")), &mut line).expect("the child's number");
    let shell_pid: i32 = i32::try_from(shell.id()).expect("a process number");
    // A child of this control, pinned at its start and then ended and reaped: its pin names a process that has gone.
    let mut gone_child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a child");
    let gone_pin: crate::child_watch_clock_marks_and_stopping::RootPin =
        crate::child_watch_clock_marks_and_stopping::pin_at_start(gone_child.id());
    gone_child.kill().expect("the control ends its child");
    gone_child.wait().expect("the child is collected");
    // A root that is still there is the positive control: its tree is read.
    let live_pin: crate::child_watch_clock_marks_and_stopping::RootPin =
        crate::child_watch_clock_marks_and_stopping::pin_at_start(u32::try_from(shell_pid).expect("a process number"));
    let for_a_gone_root: Vec<crate::process_tree::TreeMember> =
        crate::child_watch_clock_marks_and_stopping::descendants_of_a_root_that_still_runs(&gone_pin, shell_pid);
    let for_a_live_root: Vec<crate::process_tree::TreeMember> =
        crate::child_watch_clock_marks_and_stopping::descendants_of_a_root_that_still_runs(&live_pin, shell_pid);
    // SAFETY: `kill` takes two integers and touches no memory; the number is this control's own process group leader.
    let _ended = unsafe { libc::kill(-shell_pid, libc::SIGKILL) };
    let _collected = shell.wait();
    assert!(for_a_gone_root.is_empty(), "the tree of a root that has ended was read: {for_a_gone_root:?}");
    assert_eq!(for_a_live_root.len(), 1, "the tree of the root that is still there was not read: {for_a_live_root:?}");
}

/// What:
///  A watcher whose child has ended,
/// and whose ceiling has passed,
/// waits for a mark instead of taking the ceiling
///  branch again.
/// Why:
///  The branch that stopped for a root the pin reports gone went round the loop without receiving a mark and without
///  sleeping,
/// while the wait stayed begun and the ceiling stayed passed,
/// so every turn took the same branch again:
///  the watcher held a worker of the runtime and never received the end mark queued behind it
///  (the fifth review's finding 3).
///  The sender is dropped,
/// which is how a dropped watch ends the loop,
/// and the task must answer that.
#[cfg(target_os = "linux")]
#[test]
fn a_watcher_whose_child_has_ended_waits_for_a_mark_instead_of_taking_its_ceiling_again() {
    // A child this control started and then ended and reaped: the pin taken at its start reports it gone, which is the state
    //  a step that finished before its ceiling leaves.
    let mut child: Child = crate::test_support::spawn_owned(Command::new("sleep").arg("600").stdin(Stdio::null())).expect("a child of this control");
    let pid: u32 = child.id();
    let pin: crate::child_watch_clock_marks_and_stopping::RootPin = crate::child_watch_clock_marks_and_stopping::pin_at_start(pid);
    child.kill().expect("the control ends its child");
    child.wait().expect("the child is collected");
    assert!(
        !crate::child_watch_clock_marks_and_stopping::the_root_is_still_there(&pin),
        "the control's child ended, so its pin must report it gone"
    );
    let answering: Result<Result<(), tokio::task::JoinError>, tokio::time::error::Elapsed> = crate::lazy_runtime::run_on_runtime(async move {
        let sink: SharedSink = SharedSink::default();
        let flags: WatchFlags = WatchFlags {
            expired: Arc::new(AtomicBool::new(false)),
            stoppable: Arc::new(AtomicBool::new(true)),
            longest: Arc::new(AtomicU64::new(0)),
            // A ceiling of nothing: it has passed at the first look.
            ceiling: Arc::new(AtomicU64::new(0)),
            ceiling_hit: Arc::new(AtomicBool::new(false)),
            stopped: Arc::new(AtomicBool::new(false)),
        };
        let terms: WatchTerms = WatchTerms {
            limit: Duration::from_secs(3600),
            // No notice comes due, so a loop that took its ceiling branch again would have nothing to await.
            cadence: Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
            notices: Box::new(sink),
        };
        let (marks, incoming) = unbounded_channel::<Mark>();
        let task: tokio::task::JoinHandle<()> = tokio::spawn(watch_loop(
            incoming,
            flags,
            String::from("a step that already ended"),
            pid,
            terms,
            LockLedger::none(),
            None,
            pin,
        ));
        let _began = marks.send(Mark::Begin(Instant::now()));
        // The watch is dropped, as a command that stopped waiting drops it, and the watcher reads that as its last mark.
        drop(marks);
        return tokio::time::timeout(Duration::from_secs(5), task).await;
    })
    .expect("the runtime");
    assert!(answering.is_ok(), "a watcher whose child had ended spun on its ceiling branch instead of waiting for the mark that ends it");
    answering.expect("the task answered").expect("the watcher ended normally");
}

/// What:
///  A step ended at the ceiling carried by its own wait takes the processes it started with it.
/// Why:
///  The fallback ceiling ended the leader alone,
/// and a helper the step started kept running after the wrapper
///  reported the step ended,
/// though `StepBound::EndAt` says a step ends with its whole process tree
///  (the fifth review's finding 2).
///  The step's own shell waits for a helper that outlives every control here,
/// so a helper still alive at the end of
///  this control shows that nothing reached it.
#[cfg(target_os = "linux")]
#[test]
fn a_step_ended_at_the_ceiling_its_wait_carried_takes_the_processes_it_started_with_it() {
    let refused: crate::lazy_runtime::RefusalForTheControl = crate::lazy_runtime::a_control_refuses_every_task();
    let directory: PathBuf = crate::test_support::fixture("child-watch-tree-at-the-ceiling");
    let helper_file: PathBuf = directory.join("helper");
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.args(["-c", format!("sleep 600 & echo $! > '{}'; wait", helper_file.display()).as_str()]).stdin(Stdio::null());
    let failure: std::io::Error =
        output_watched_within(&mut command, "a step with a helper", "a step with a helper", terms(Duration::from_secs(3600), &sink), true, StepBound::EndAt(Duration::from_millis(400)))
            .expect_err("the ceiling ended the step");
    drop(refused);
    let helper: i32 = std::fs::read_to_string(&helper_file).expect("the helper's process ID").trim().parse().expect("a process number");
    // A process that has ended is collected by the system's init, or waits as a zombie of a subreaper; neither is a
    //  process that still runs, and `member_read` answers both with `Gone`.
    let looked_from: Instant = Instant::now();
    while looked_from.elapsed() < Duration::from_secs(2) && crate::process_tree::member_read(helper) != crate::process_tree::MemberRead::Gone {
        std::thread::sleep(Duration::from_millis(20));
    }
    let gone: bool = crate::process_tree::member_read(helper) == crate::process_tree::MemberRead::Gone;
    // The assertions come after the cleanup, so a failing control leaves nothing running.
    if !gone {
        // SAFETY: `kill` takes two integers and touches no memory; the number is the helper this control started.
        let _ended = unsafe { libc::kill(helper, libc::SIGKILL) };
    }
    crate::test_support::remove(directory.as_path());
    assert!(failure.to_string().contains("the longest the wrapper lets one step run"), "the step ended at its ceiling: {failure}");
    assert!(gone, "the process the step started was left running after the step was reported ended at its ceiling");
}

/// What:
///  A step left running whose waiter thread the system refuses is still collected when it ends.
/// Why:
///  The child was moved into the closure of the refused start,
/// and the start's failure dropped that closure and the child
///  with it.
/// A dropped `Child` is neither ended nor reaped,
/// so the step the command had just started ran on with an owner,
///  and became a zombie of this process for the rest of its life once it ended
///  (the fifth review's finding 3).
///  The step ends when this control releases it,
/// and nothing may keep it a zombie afterwards.
#[cfg(target_os = "linux")]
#[test]
fn a_step_left_running_whose_waiter_thread_is_refused_is_still_collected() {
    let directory: PathBuf = crate::test_support::fixture("child-watch-refused-waiter");
    let pid_file: PathBuf = directory.join("pid");
    let release: PathBuf = directory.join("release");
    let script: PathBuf = directory.join("step.sh");
    std::fs::write(
        &script,
        format!("echo $$ > '{pid}'\nwhile [ ! -e '{release}' ]; do sleep 0.02; done\n", pid = pid_file.display(), release = release.display()),
    )
    .expect("the step");
    crate::child_watch::refuse_the_next_left_running_waiter(1);
    let sink: SharedSink = SharedSink::default();
    let mut command: Command = Command::new("sh");
    command.arg(script.as_path());
    let bound: StepBound = StepBound::LeaveRunning { after: Duration::from_millis(300), extra: Duration::from_millis(300) };
    let failure: std::io::Error = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_secs(3600), &sink), false, bound)
        .expect_err("no thread could be started to wait for the step");
    // The step was started before the thread was refused, and it writes its number as it begins.
    let waited_for_the_number: Instant = Instant::now();
    while waited_for_the_number.elapsed() < Duration::from_secs(5) && !pid_file.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    let pid: i32 = std::fs::read_to_string(&pid_file).expect("the step recorded its process").trim().parse().expect("a process number");
    // The start time proves which process the number names, so a number that passed to another one is not read as the step.
    let started: u64 = crate::process_tree::use_of(pid).expect("the step runs as it waits").started;
    // The step ends now, and its end must be collected by whoever owns it.
    std::fs::write(&release, b"").expect("let the step end");
    let looked_from: Instant = Instant::now();
    let mut collected: bool = false;
    while !collected && looked_from.elapsed() < Duration::from_secs(10) {
        match crate::process_tree::use_of(pid) {
            // The step is gone, or its number names a process that started later: this one's end was collected.
            None => collected = true,
            Some(seen) if seen.started != started => collected = true,
            // A process that is still there is the step, waiting as a zombie for a wait that never comes.
            Some(_still_there) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
    // The assertions come after the cleanup, so a failing control leaves nothing running.
    if !collected {
        // SAFETY: `kill` takes two integers and touches no memory; the number is the step this control started, and a zombie
        //  ignores it.
        let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    }
    crate::test_support::remove(directory.as_path());
    assert!(failure.to_string().contains("no thread could be started to wait for it"), "the refusal is what this control armed: {failure}");
    assert!(collected, "the step that was left running stayed a zombie after it ended: nothing collected it");
}

/// What:
///  A step left running whose waiter thread is refused ends the wait with a sentence that names the limit.
/// Why:
///  The step has already started, so the refusal costs the command the answer and nothing else: the step runs on and
///  a reaper collects it. A person who meets this has to be told which limit refused the thread, because nothing in
///  the command can be changed to work around it.
/// The refusal is armed rather than made of a real limit, and it has to be: the step is a process of this very
/// process, which `RLIMIT_NPROC` refuses as well, and the waiter's start comes a moment after the child starts, so a
/// limit lowered once the child exists (measured: the step's number is visible 30 ms after it starts) never arrives
/// before the start it has to refuse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // arm the refusal, then: expect(watched(...)).rejects.toContain('ulimit -u');
/// ```
#[cfg(target_os = "linux")]
#[test]
fn a_step_left_running_whose_waiter_is_refused_names_the_limit() {
    let directory: PathBuf = crate::test_support::fixture("child-watch-refused-waiter-names-the-limit");
    let pid_file: PathBuf = directory.join("pid");
    let release: PathBuf = directory.join("release");
    let script: PathBuf = directory.join("step.sh");
    std::fs::write(
        &script,
        format!("echo $$ > '{pid}'\nwhile [ ! -e '{release}' ]; do sleep 0.02; done\n", pid = pid_file.display(), release = release.display()),
    )
    .expect("the step");
    crate::child_watch::refuse_the_next_left_running_waiter(1);
    let mut command: Command = Command::new("sh");
    command.arg(script.as_path());
    let sink: SharedSink = SharedSink::default();
    let bound: StepBound = StepBound::LeaveRunning { after: Duration::from_millis(300), extra: Duration::from_millis(300) };
    let failure: std::io::Error = output_watched_within(&mut command, "a write", "a write", terms(Duration::from_secs(3600), &sink), false, bound)
        .expect_err("no thread could be started to wait for the step");
    let waited_for_the_number: Instant = Instant::now();
    while waited_for_the_number.elapsed() < Duration::from_secs(5) && !pid_file.exists() {
        std::thread::sleep(Duration::from_millis(20));
    }
    std::fs::write(&release, b"").expect("let the step end");
    crate::test_support::remove(directory.as_path());
    let said: String = failure.to_string();
    assert!(said.contains("no thread could be started to wait for it"), "the refusal is what this control made: {said}");
    assert!(said.contains("ulimit -u") && said.contains("pids.max"), "the refusal names the limits that refused the thread: {said}");
}
