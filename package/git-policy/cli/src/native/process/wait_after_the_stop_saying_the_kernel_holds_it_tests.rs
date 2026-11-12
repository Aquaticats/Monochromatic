//! What:
//!  Controls for the wait after a stop:
//!  that the command says the kernel has not released the step,
//!  that it says so again while the wait lasts,
//!  that a look which failed or a line a stopped reader could not take is tried again,
//!  and that the line names the process the kernel is holding.
//! Why:
//!   No process this host can start stays in uninterruptible sleep,
//!  so a control states the kernel's answer about a process's state,
//!  and states that the step's end has not been collected:
//!  those two are the whole of what a real hold gives the wait
//!  (`scratch_step_look_below_control_window.rs`,
//!  `wait_after_the_stop_saying_the_kernel_holds_it_control_window.rs`).
//!  Every control here starts a real `sleep` as the step,
//!  and the wait ends it for real.
#![cfg(target_os = "linux")]

use super::*;
use crate::child_awaited_through_a_pidfd::adopt;
use crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::{Ceilings, wait_until_it_exits_the_stop_or_the_ceiling};
use crate::forwarding::ChildOutcome;
use crate::lazy_runtime::run_on_runtime;
use crate::native_commit_waiting_notice::{NativeCommitNotice, NoticeStart, refuse_the_next_notice_thread};
use crate::scratch_step_look_below::the_controls_window as the_look_controls;
use crate::test_support::SharedSink;
use crate::wait_progress::Cadence;
use std::ffi::OsString;
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// The words of a commit command line,
/// as the notice reads them.
fn words(text: &[&str]) -> Vec<OsString> {
    return text.iter().map(OsString::from).collect();
}

/// The `git commit` of the step,
/// standing in for the one the commit runs in its scratch repository.
fn sleeping_step() -> std::process::Child {
    return std::process::Command::new("sleep").arg("120").spawn().expect("a long-running child");
}

/// The number of a child.
fn number_of(child: &std::process::Child) -> i32 {
    return i32::try_from(child.id()).expect("a process number");
}

/// How long a control gives the wait to say the line:
///  longer than the pause before its first look and one look interval of the wait it gives,
/// and only a control pays it.
const A_LINE_BOUND: Duration = Duration::from_secs(10);

/// How long a control holds a step the kernel is said to be holding,
/// so the wait has said its line and,
/// where the control counts lines,
/// said it more than once.
const A_HOLD_BOUND: Duration = Duration::from_millis(1500);

/// What:
///  The text `read` answers once it contains `needle`,
///  waited for no longer than `bound`,
///  so a command that never says it fails the control instead of hanging it.
/// Why:
///  A control reads the line while the wait still runs,
///  and a wait that is not going to say it must fail,
///  never block the run.
fn until_the_text(read: &dyn Fn() -> String, bound: Duration, needle: &str) -> String {
    let deadline: Instant = Instant::now() + bound;
    loop {
        let said: String = read();
        if said.contains(needle) || Instant::now() >= deadline {
            return said;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// What:
///  Run the wait on `step` with the notice's own queue,
///  and answer what `read` says once the line was said.
/// Why:
///   Every control here runs the same wait with the same words,
///  and differs only in what the kernel answers and in how the reader behaves.
fn says_while_the_wait_lasts(lines: Option<&SyncSender<String>>, step: std::process::Child, read: &dyn Fn() -> String) -> String {
    let the_lines: &SyncSender<String> = lines.expect("the notice's writer thread runs");
    // The two bounds a control pays: the ceiling comes at once, and the bound a step the kernel holds is waited for
    //  is short, so a control never waits the five minutes the shipped bound allows.
    let ceilings: Ceilings = Ceilings { usually: Duration::from_millis(300), while_the_kernel_holds_it: Duration::from_millis(600) };
    let (outcome, a_stop): (ChildOutcome, Option<crate::child_awaited_until_it_exits_the_stop_or_the_ceiling::CeilingStop>) =
        run_on_runtime(wait_until_it_exits_the_stop_or_the_ceiling(adopt(step), None, ceilings, "a step", Some(the_lines)))
            .expect("the runtime")
            .expect("the wait answers");
    assert_eq!(outcome, ChildOutcome::Signaled(libc::SIGKILL), "the ceiling ended the step the wait owns");
    assert!(a_stop.is_some(), "the ceiling's stop is recorded");
    return until_the_text(read, A_LINE_BOUND, "has not released");
}

/// What:
///  A reader that takes the first line it is given and stops taking more until a control lets it go,
///  standing for a standard error whose reader stopped reading.
/// Why:
///   A reader that stopped reading holds the writer thread and fills the queue behind it,
///  which is where a line that explains a wait used to be lost.
struct AReaderThatStops {
    /// Set once a write was entered,
    /// so a control knows the line reached the sink.
    entered: Arc<AtomicBool>,
    /// Set by the control to let the first write return.
    released: Arc<AtomicBool>,
    /// What was written.
    written: Arc<std::sync::Mutex<Vec<u8>>>,
}

impl Write for AReaderThatStops {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.entered.store(true, Ordering::SeqCst);
        while !self.released.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(5));
        }
        self.written.lock().unwrap_or_else(std::sync::PoisonError::into_inner).extend_from_slice(bytes);
        return Ok(bytes.len());
    }

    fn flush(&mut self) -> std::io::Result<()> {
        return Ok(());
    }
}

/// What:
///  A notice thread the system refused leaves the wait that keeps waiting to say the kernel has not released the step.
/// Why:
///   A notice thread is what made a step end at its ceiling before the wait that owns the child did,
///  and a process at its thread limit starts none.
///  The line that explains the wait was said only from that thread,
///  so a commit whose thread was refused waited for the kernel with nothing said
///  (finding 1 of the 2026-11-10 review of the held-step notice).
///  The stop here is real and so is the step;
///  what the control states is the kernel's answer about the step's state and the step's unreaped end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // refuseTheNextNoticeThread(); theKernelHolds(pid); holdTheReap(); await wait(...); expect(text).toContain(`(process ${pid})`)
/// ```
#[test]
fn a_wait_whose_notice_thread_was_refused_still_says_the_kernel_holds_the_step() {
    let _look_window = the_look_controls::take_the_window();
    let _reap_window = the_controls_window::take_the_window();
    let sink: SharedSink = SharedSink::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(50), interval: Duration::from_millis(200) };
    refuse_the_next_notice_thread(1);
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(sink.clone()),
    });
    let step: std::process::Child = sleeping_step();
    let number: i32 = number_of(&step);
    the_look_controls::the_kernel_holds(number, "git");
    the_controls_window::hold_the_reap();
    let release = std::thread::spawn(|| {
        std::thread::sleep(A_HOLD_BOUND);
        the_controls_window::let_the_reap_go();
    });
    let said: String = says_while_the_wait_lasts(notice.lines_for_the_wait(), step, &|| { return sink.text(); });
    release.join().expect("the control's thread");
    let _facts = notice.finish();
    assert!(
        said.contains(format!("(process {number})").as_str()),
        "the command named no process it says the kernel has not released, so the person cannot tell what the wait is: {said:?}"
    );
    assert!(said.contains("has not released"), "the command said nothing of the kernel holding the step: {said:?}");
}

/// What:
///  The line is said once,
///  however long the wait lasts.
/// Why:
///   A person who met the same sentence three times in one failed commit read nothing new the second time
///  (2026-11-11,
///  a commit the installed wrapper failed in a disk wait),
///  and the command has already said how long the step lasted before it stopped it:
///  one stop is one sentence.
///  The wait here outlasts several notice intervals,
///  so a line said again would be counted again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // theKernelHolds(pid); holdTheReap(1500); expect(count(text, 'has not released')).toBe(1)
/// ```
#[test]
fn the_line_is_said_once_however_long_the_wait_lasts() {
    let _look_window = the_look_controls::take_the_window();
    let _reap_window = the_controls_window::take_the_window();
    let sink: SharedSink = SharedSink::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(50), interval: Duration::from_millis(200) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(sink.clone()),
    });
    let step: std::process::Child = sleeping_step();
    the_look_controls::the_kernel_holds(number_of(&step), "git");
    the_controls_window::hold_the_reap();
    let release = std::thread::spawn(|| {
        std::thread::sleep(A_HOLD_BOUND);
        the_controls_window::let_the_reap_go();
    });
    let said: String = says_while_the_wait_lasts(notice.lines_for_the_wait(), step, &|| { return sink.text(); });
    release.join().expect("the control's thread");
    let _facts = notice.finish();
    let times: usize = said.matches("has not released").count();
    assert_eq!(times, 1, "a wait that outlasted several notice intervals said the kernel holds the step {times} time(s): {said:?}");
}

/// What:
///  A look that could not be made is made again,
///  never read as the kernel having let go.
/// Why:
///   `/proc` can refuse a read for a moment (a descriptor limit,
///   a walk that named processes and let none of them be read),
///  and reading that as "the step is not held" lost the whole explanation for a wait that then lasted
///  (finding 2 of the 2026-11-10 review of the held-step notice).
///  The step still is held in this control,
///  and only the first look fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // plantOneLookThatCannotBeMade(); theKernelHolds(pid); holdTheReap(); await wait(...); expect(text).toContain('has not released')
/// ```
#[test]
fn a_look_that_could_not_be_made_is_made_again() {
    let _look_window = the_look_controls::take_the_window();
    let _reap_window = the_controls_window::take_the_window();
    let sink: SharedSink = SharedSink::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(50), interval: Duration::from_millis(200) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(sink.clone()),
    });
    let step: std::process::Child = sleeping_step();
    let number: i32 = number_of(&step);
    the_look_controls::the_kernel_holds(number, "git");
    the_look_controls::plant_looks_that_cannot_be_made(1);
    the_controls_window::hold_the_reap();
    let release = std::thread::spawn(|| {
        std::thread::sleep(A_HOLD_BOUND);
        the_controls_window::let_the_reap_go();
    });
    let said: String = says_while_the_wait_lasts(notice.lines_for_the_wait(), step, &|| { return sink.text(); });
    release.join().expect("the control's thread");
    let _facts = notice.finish();
    assert!(
        said.contains(format!("(process {number})").as_str()),
        "one look that could not be made took the whole explanation away from a wait that lasted: {said:?}"
    );
}

/// What:
///  A line the writer could not take is said again as soon as the writer can take it.
/// Why:
///   The queue holds one line,
///   and a reader that stopped reading fills it with an older advisory line,
///  so the line that explains a wait of unknown length was dropped and never handed over again
///  (finding 3 of the 2026-11-10 review of the held-step notice).
///  The reader here takes nothing until it is let go halfway through the wait,
///  and the interval the wait is given is far longer than the wait itself,
///  so only an immediate retry brings the line in time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // readerStops(); theKernelHolds(pid); holdTheReap(2200); releaseReader(1200); expect(text).toContain('has not released')
/// ```
#[test]
fn a_line_the_writer_could_not_take_is_said_again_when_it_can() {
    let _look_window = the_look_controls::take_the_window();
    let _reap_window = the_controls_window::take_the_window();
    let entered: Arc<AtomicBool> = Arc::default();
    let released: Arc<AtomicBool> = Arc::default();
    let written: Arc<std::sync::Mutex<Vec<u8>>> = Arc::default();
    let reader: AReaderThatStops = AReaderThatStops { entered: entered.clone(), released: released.clone(), written: written.clone() };
    // A line every tenth of a second, so the queue behind the stopped reader is full before the ceiling comes.
    let cadence: Cadence = Cadence { grace: Duration::from_millis(100), interval: Duration::from_millis(100) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(reader),
    });
    let writing: Instant = Instant::now() + A_LINE_BOUND;
    while !entered.load(Ordering::SeqCst) && Instant::now() < writing {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(entered.load(Ordering::SeqCst), "the notice never wrote a line, so no reader ever stopped");
    // Two more intervals: the writer holds the first line, and the line behind it fills the one queue place.
    std::thread::sleep(Duration::from_millis(250));
    let step: std::process::Child = sleeping_step();
    the_look_controls::the_kernel_holds(number_of(&step), "git");
    the_controls_window::hold_the_reap();
    let release = {
        let letting_go: Arc<AtomicBool> = released.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(1200));
            letting_go.store(true, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(1000));
            the_controls_window::let_the_reap_go();
        })
    };
    let read = {
        let written_by_the_reader: Arc<std::sync::Mutex<Vec<u8>>> = written.clone();
        move || -> String {
            return String::from_utf8_lossy(written_by_the_reader.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_slice()).into_owned();
        }
    };
    // Far longer than the wait lasts, so a line that waited for the next interval never arrives in it.
    let said: String = says_while_the_wait_lasts(notice.lines_for_the_wait(), step, &read);
    release.join().expect("the control's thread");
    let _facts = notice.finish();
    assert!(
        said.contains("has not released"),
        "the line that explains a wait of unknown length waited for a whole notice interval after the reader resumed: {said:?}"
    );
}

/// What:
///  A program below the step that the kernel is holding is the process the line names,
///  and the step itself is not said to be in uninterruptible sleep when it is not.
/// Why:
///   A `git commit` that started a signing program is blocked by that program's call,
///  not by a call of its own,
///  and a line that names the step states a specific false fact about it
///  (finding 4 of the 2026-11-10 review of the held-step notice).
///  The step here is a live `sleep` and the held process is another real one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // theKernelHolds(helper, 'ssh-keygen'); holdTheReap(); await wait(...); expect(text).toContain(`(process ${helper})`)
/// ```
#[test]
fn a_helper_held_below_the_step_is_the_process_the_line_names() {
    let _look_window = the_look_controls::take_the_window();
    let _reap_window = the_controls_window::take_the_window();
    let sink: SharedSink = SharedSink::default();
    let cadence: Cadence = Cadence { grace: Duration::from_millis(50), interval: Duration::from_millis(200) };
    let notice: NativeCommitNotice = NativeCommitNotice::start_within(NoticeStart {
        snapshot: None,
        commit_arguments: &words(&["-m", "x"]),
        cadence,
        ceiling: Duration::from_millis(300),
        sink: Box::new(sink.clone()),
    });
    // The step starts a helper of its own and prints its number, so the helper a control holds is really below the step:
    //  a control marks a process the look found and never adds one
    //  (`scratch_step_look_below_control_window.rs`).
    let mut command: std::process::Command = std::process::Command::new("sh");
    command.arg("-c").arg("sleep 600 & echo $!; wait").stdout(std::process::Stdio::piped());
    let mut step: std::process::Child = crate::test_support::spawn_owned(&mut command).expect("a step with a helper");
    let helper_number: i32 = {
        use std::io::{BufRead, BufReader};
        let printed = step.stdout.take().expect("the helper's number");
        let mut line: String = String::new();
        BufReader::new(printed).read_line(&mut line).expect("a number");
        line.trim().parse::<i32>().expect("a process number")
    };
    let step_number: i32 = number_of(&step);
    the_look_controls::the_kernel_holds(helper_number, "ssh-keygen");
    the_controls_window::hold_the_reap();
    let release = std::thread::spawn(|| {
        std::thread::sleep(A_HOLD_BOUND);
        the_controls_window::let_the_reap_go();
    });
    let said: String = says_while_the_wait_lasts(notice.lines_for_the_wait(), step, &|| { return sink.text(); });
    release.join().expect("the control's thread");
    let _facts = notice.finish();
    assert!(
        said.contains(format!("(process {helper_number})").as_str()) && said.contains("ssh-keygen"),
        "the line did not name the program the kernel is holding: {said:?}"
    );
    assert!(
        !(said.contains(format!("(process {step_number})").as_str()) && said.contains("it is in uninterruptible sleep")),
        "the line said the step itself is in uninterruptible sleep, which the look never found: {said:?}"
    );
}
