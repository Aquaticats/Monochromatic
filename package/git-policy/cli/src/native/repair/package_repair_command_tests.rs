//! Finding a program on a search path and running it with a limit:
//! found,
//! missing,
//! not executable,
//! succeeded,
//! failed with
//!  its output,
//! stopped at the limit,
//! run in the named directory with the added variables.

use super::*;
use crate::test_support::executable;
use crate::test_support::{fixture, remove};

/// A limit long enough that no control reaches it unless the control is about the limit.
const GENEROUS: Duration = Duration::from_secs(60);

/// A ceiling no control reaches unless the control is about the ceiling.
const ROOMY: Duration = Duration::from_secs(3600);

/// The limits of a control that is about the quiet window only.
fn limits(window: Duration) -> ToolLimits {
    return ToolLimits { window, ceiling: ROOMY };
}

/// A limit short enough to wait out.
const BRIEF: Duration = Duration::from_millis(300);

/// The status the failing fixture exits with.
const FAILING_STATUS: i32 = 7;

/// How long the slow fixture would sleep if nothing stopped it;
/// far past any limit used here.
const SLOW_SECONDS: u64 = 30;

/// The first matching executable wins,
/// in the order of the search path.
#[test]
fn finds_the_first_executable_in_search_order() {
    let root: PathBuf = fixture("finds-first");
    let first: PathBuf = root.join("first");
    let second: PathBuf = root.join("second");
    std::fs::create_dir_all(&first).expect("first");
    std::fs::create_dir_all(&second).expect("second");
    executable(first.join("tool").as_path(), b"#!/bin/sh\nexit 0\n");
    executable(second.join("tool").as_path(), b"#!/bin/sh\nexit 0\n");
    let search: OsString = std::env::join_paths([&first, &second]).expect("search path");
    assert_eq!(find_program("tool", Some(search.as_os_str())), Some(first.join("tool")));
    remove(&root);
}

/// A name that is absent,
/// a file that is not executable and an absent search path are all "not found".
#[test]
fn reports_missing_and_non_executable_programs_as_absent() {
    let root: PathBuf = fixture("finds-absent");
    std::fs::write(root.join("plain"), b"data").expect("plain file");
    let search: OsString = root.clone().into_os_string();
    assert_eq!(find_program("absent", Some(search.as_os_str())), None);
    assert_eq!(find_program("plain", Some(search.as_os_str())), None);
    assert_eq!(find_program("plain", None), None);
    remove(&root);
}

/// The program runs in the named directory with the added variable,
/// and a zero status is a success.
#[test]
fn runs_in_the_directory_with_the_variables_and_reports_success() {
    let root: PathBuf = fixture("runs-success");
    let program: PathBuf = root.join("tool");
    let record: PathBuf = root.join("record");
    executable(
        program.as_path(),
        format!("#!/bin/sh\npwd -P > '{}'\nprintf '%s' \"$REPAIR_PROBE\" >> '{}'\n", record.display(), record.display()).as_bytes(),
    );
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: vec![(OsString::from("REPAIR_PROBE"), OsString::from("kept"))] };
    let run: ToolRun = run_tool(program.as_path(), &command, limits(GENEROUS));
    assert_eq!(run.end, ToolEnd::Succeeded);
    let recorded: String = std::fs::read_to_string(&record).expect("record");
    assert!(recorded.starts_with(root.to_str().expect("utf8 root")), "ran in {recorded}");
    assert!(recorded.ends_with("kept"), "variable missing in {recorded}");
    remove(&root);
}

/// A failing status names itself and carries the tail of the output the program printed.
#[test]
fn reports_a_failure_with_its_status_and_output() {
    let root: PathBuf = fixture("runs-failure");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), format!("#!/bin/sh\necho 'reason printed on stderr' >&2\nexit {FAILING_STATUS}\n").as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, limits(GENEROUS));
    let ToolEnd::Failed(text) = run.end else { panic!("expected a failure, got {:?}", run.end) };
    assert!(text.contains(FAILING_STATUS.to_string().as_str()), "status missing in {text}");
    assert!(text.contains("reason printed on stderr"), "output missing in {text}");
    remove(&root);
}

/// A program that has shown no sign of work for its window is stopped,
/// and the run says so.
#[test]
fn stops_a_program_that_shows_no_sign_of_work_for_its_window() {
    let root: PathBuf = fixture("runs-limit");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), format!("#!/bin/sh\nexec sleep {SLOW_SECONDS}\n").as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, limits(BRIEF));
    assert_eq!(run.end, ToolEnd::TimedOut);
    assert!(run.millis < u128::from(SLOW_SECONDS) * 1000, "the run waited {} ms for a program stopped at its limit", run.millis);
    remove(&root);
}

/// A program that cannot start is a failure that names the cause,
/// never a panic.
#[test]
fn reports_a_program_that_cannot_start() {
    let root: PathBuf = fixture("runs-unstartable");
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(root.join("absent").as_path(), &command, limits(GENEROUS));
    assert!(matches!(run.end, ToolEnd::Failed(_)), "got {:?}", run.end);
    remove(&root);
}

/// Lines that only say the tool waited for another process are dropped from a failure's text;
/// the reason stays.
#[test]
fn a_failure_text_drops_wait_notices() {
    let text: String = tail_of(b"Blocking waiting for file lock on package cache\nerror: the real reason\n");
    assert_eq!(text, "error: the real reason");
}

/// The quiet window of the control below:
/// longer than the sampling interval of the progress meter.
const WORKING_WINDOW: Duration = Duration::from_millis(1200);

/// A tool that keeps printing is never stopped however long it runs:
/// the limit judges quiet time,
/// not wall time.
///  The fixture prints a line every pause for a multiple of the window,
/// with no busy loop.
#[test]
fn a_program_that_keeps_working_is_never_stopped_however_long_it_runs() {
    let root: PathBuf = fixture("runs-working");
    let program: PathBuf = root.join("tool");
    let rounds: u32 = 15;
    executable(
        program.as_path(),
        format!("#!/bin/sh\nn=0\nwhile [ $n -lt {rounds} ]; do echo progress; sleep 0.2; n=$((n + 1)); done\n").as_bytes(),
    );
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, limits(WORKING_WINDOW));
    assert_eq!(run.end, ToolEnd::Succeeded);
    assert!(run.millis > WORKING_WINDOW.as_millis() * 2, "the fixture ran only {} ms", run.millis);
    remove(&root);
}

/// The ceiling of the control below:
/// well past the quiet window,
/// well short of the fixture's length.
const SHORT_CEILING: Duration = Duration::from_secs(2);

/// Whether the process of this number exists and is not a zombie waiting to be reaped.
fn is_running(pid: i32) -> bool {
    // SAFETY: signal 0 only asks whether the process exists.
    if unsafe { libc::kill(pid, 0) } != 0 {
        return false;
    }
    return std::fs::read_to_string(format!("/proc/{pid}/stat")).map_or(true, |stat| { return !stat.rsplit(')').next().unwrap_or("").trim_start().starts_with('Z'); });
}

/// Waits a moment for a process to be gone:
/// the stop reaches it with a signal,
/// and its end is seen a moment later.
fn is_gone_soon(pid: i32) -> bool {
    let waiting_since: Instant = Instant::now();
    while is_running(pid) {
        if waiting_since.elapsed() > Duration::from_secs(10) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    return true;
}

/// Owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)":
/// a tool that keeps printing is ended at the ceiling,
/// with its whole
/// process group.
/// The fixture starts two helpers,
/// one that prints and one that has its output redirected away,
/// and the control
/// records each by the number the shell gives the background job (`$!`),
/// which names the helper and not the shell that started it
/// (sol-2:
/// `$$` in a subshell is the parent's number),
/// then checks that both are gone.
#[test]
fn a_program_that_keeps_working_is_ended_at_its_ceiling_with_its_whole_process_group() {
    let root: PathBuf = fixture("runs-ceiling");
    let program: PathBuf = root.join("tool");
    let printing_record: PathBuf = root.join("printing-helper-pid");
    let silent_record: PathBuf = root.join("silent-helper-pid");
    let rounds: u32 = 100;
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\n(n=0; while [ $n -lt {rounds} ]; do echo helper; sleep 0.2; n=$((n + 1)); done) &\necho $! > '{printing}'\n(sleep 60) >/dev/null 2>&1 &\necho $! > '{silent}'\nn=0\nwhile [ $n -lt {rounds} ]; do echo progress; sleep 0.2; n=$((n + 1)); done\n",
            printing = printing_record.display(),
            silent = silent_record.display()
        )
        .as_bytes(),
    );
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, ToolLimits { window: WORKING_WINDOW, ceiling: SHORT_CEILING });
    assert_eq!(run.end, ToolEnd::CeilingReached(SHORT_CEILING), "ran {} ms", run.millis);
    assert!(run.millis >= SHORT_CEILING.as_millis() && run.millis < u128::from(SLOW_SECONDS) * 1000 / 2, "the run lasted {} ms against a ceiling of {SHORT_CEILING:?}", run.millis);
    for record in [&printing_record, &silent_record] {
        let pid: i32 = std::fs::read_to_string(record).expect("the helper's number").trim().parse().expect("a process number");
        assert!(is_gone_soon(pid), "the helper {pid} of the tool outlived the stop of its group");
    }
    remove(&root);
}

/// The leader exits and a helper it started keeps the tool's output open (sol-2,
/// A):
/// the run ends with the leader's status,
/// and does
/// not wait for the helper to close what it inherited.
/// The control has a deadline of its own around the run.
#[test]
fn a_helper_that_keeps_the_output_open_after_the_leader_exits_does_not_hold_the_run() {
    let root: PathBuf = fixture("runs-helper-holds-output");
    let program: PathBuf = root.join("tool");
    let record: PathBuf = root.join("helper-pid");
    executable(program.as_path(), format!("#!/bin/sh\nsleep 40 &\necho $! > '{}'\necho leader-output\n", record.display()).as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let (sender, receiver) = std::sync::mpsc::channel();
    let started_in: PathBuf = program.clone();
    std::thread::spawn(move || { let _sent = sender.send(run_tool(started_in.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: GENEROUS })); });
    let run: ToolRun = receiver.recv_timeout(Duration::from_secs(15)).expect("the run ended without waiting for the helper to close the output");
    assert_eq!(run.end, ToolEnd::Succeeded);
    assert!(String::from_utf8_lossy(run.stdout.as_slice()).contains("leader-output"));
    let pid: i32 = std::fs::read_to_string(&record).expect("the helper's number").trim().parse().expect("a process number");
    // The helper is left alone on purpose (sol-2 round 2): a run that ended it would also pass the deadline above, so the control
    //  asks the helper itself before it cleans up.
    assert!(is_running(pid), "the helper of a tool that succeeded is left running");
    // SAFETY: the control ends the helper it started.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    remove(&root);
}

/// The leader exits and a helper it started keeps writing to the tool's output (sol-1 and sol-2 round 2):
/// the run ends with the
/// leader's status and a finite snapshot,
/// and the helper is left running.
/// The control has a deadline of its own around the run.
#[test]
fn a_helper_that_keeps_writing_after_the_leader_exits_does_not_hold_the_run() {
    let root: PathBuf = fixture("runs-helper-keeps-writing");
    let program: PathBuf = root.join("tool");
    let record: PathBuf = root.join("helper-pid");
    executable(
        program.as_path(),
        format!("#!/bin/sh\n(while :; do echo helper-output; sleep 0.01; done) &\necho $! > '{}'\necho leader-output\nsleep 0.3\n", record.display()).as_bytes(),
    );
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let (sender, receiver) = std::sync::mpsc::channel();
    let started_in: PathBuf = program.clone();
    std::thread::spawn(move || { let _sent = sender.send(run_tool(started_in.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: GENEROUS })); });
    let run: ToolRun = receiver.recv_timeout(Duration::from_secs(15)).expect("the run ended without chasing the helper's output");
    assert_eq!(run.end, ToolEnd::Succeeded);
    assert!(String::from_utf8_lossy(run.stdout.as_slice()).contains("leader-output"));
    let pid: i32 = std::fs::read_to_string(&record).expect("the helper's number").trim().parse().expect("a process number");
    assert!(is_running(pid), "the helper that keeps writing is left running");
    // SAFETY: the control ends the helper it started.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    remove(&root);
}

/// The ceiling ends the run even when a helper left the tool's process group while it holds the output (sol-2,
/// A):
/// the stop of the
/// group does not reach it,
/// and the run must not wait for it either.
#[test]
fn a_helper_outside_the_process_group_that_holds_the_output_does_not_hold_the_ceiling() {
    let root: PathBuf = fixture("runs-helper-outside-group");
    let program: PathBuf = root.join("tool");
    let record: PathBuf = root.join("helper-pid");
    executable(
        program.as_path(),
        format!("#!/bin/sh\nsetsid sleep 40 &\necho $! > '{}'\nn=0\nwhile [ $n -lt 100 ]; do echo progress; sleep 0.2; n=$((n + 1)); done\n", record.display()).as_bytes(),
    );
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let (sender, receiver) = std::sync::mpsc::channel();
    let started_in: PathBuf = program.clone();
    std::thread::spawn(move || { let _sent = sender.send(run_tool(started_in.as_path(), &command, ToolLimits { window: WORKING_WINDOW, ceiling: SHORT_CEILING })); });
    let run: ToolRun = receiver.recv_timeout(Duration::from_secs(15)).expect("the ceiling ended the run without waiting for the helper");
    assert_eq!(run.end, ToolEnd::CeilingReached(SHORT_CEILING));
    let pid: i32 = std::fs::read_to_string(&record).expect("the helper's number").trim().parse().expect("a process number");
    // SAFETY: the control ends the helper it started.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    remove(&root);
}

/// A tool that finishes before the ceiling is not touched by it.
#[test]
fn a_program_that_finishes_before_its_ceiling_succeeds() {
    let root: PathBuf = fixture("runs-under-ceiling");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), b"#!/bin/sh\necho done\n");
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: GENEROUS });
    assert_eq!(run.end, ToolEnd::Succeeded);
    remove(&root);
}

/// A leader that does not die when the group is stopped (a process in uninterruptible I/O ends only when the I/O does) is not waited
/// for without a bound (sol-2 round 2):
/// the reaping gives up after its budget,
/// reports that it did,
/// and hands the child to a thread
/// that reaps it whenever it does end.
#[test]
fn a_leader_that_does_not_end_is_handed_to_a_reaper_after_the_budget() {
    let mut command: Command = Command::new("sleep");
    command.arg("30");
    let child: Child = command.spawn().expect("the leader");
    let pid: i32 = i32::try_from(child.id()).expect("a process number");
    let began: Instant = Instant::now();
    assert!(!run_on_runtime(reap_within(child, Duration::from_millis(300))).expect("the runtime runs the reaping"), "the leader had not ended");
    assert!(began.elapsed() < Duration::from_secs(10), "the reaping gave up after its budget");
    assert!(is_running(pid), "the leader is still there: nothing waited for it");
    // SAFETY: the control ends the process it started.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    assert!(is_gone_soon(pid), "the thread the leader was handed to reaped it when it ended");
}

/// A leader that ends is reaped within the budget.
#[test]
fn a_leader_that_ends_is_reaped_within_the_budget() {
    let mut command: Command = Command::new("true");
    let child: Child = command.spawn().expect("the leader");
    assert!(run_on_runtime(reap_within(child, Duration::from_secs(10))).expect("the runtime runs the reaping"));
}

/// A tool that writes more than the snapshot keeps,
/// then fails:
/// the person is told the start of its output is left out (sol-1 round 3,
/// P2 E:
/// the notes of the snapshot were dropped).
/// A tool that succeeds with output that could not be taken whole says so in
/// `notes`,
/// which a parser of that output must look at.
#[test]
fn output_past_the_snapshot_is_disclosed_to_the_person() {
    let root: PathBuf = fixture("runs-output-past-the-limit");
    let program: PathBuf = root.join("tool");
    let too_much: usize = crate::output_capture::STEP_OUTPUT_LIMIT + 1024;
    executable(program.as_path(), format!("#!/bin/sh\nhead --bytes={too_much} /dev/zero | tr '\\0' 'x'\necho the-reason >&2\nexit {FAILING_STATUS}\n").as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: GENEROUS });
    let ToolEnd::Failed(text) = &run.end else { panic!("{run:?}") };
    assert!(text.contains("the-reason"), "the tail of the error output is shown: {text}");
    assert!(text.contains("left out"), "the person is told output was left out: {text}");
    let succeeding: PathBuf = root.join("succeeding");
    executable(succeeding.as_path(), format!("#!/bin/sh\nhead --bytes={too_much} /dev/zero | tr '\\0' 'x'\n").as_bytes());
    let succeeded: ToolRun = run_tool(succeeding.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: GENEROUS });
    assert_eq!(succeeded.end, ToolEnd::Succeeded);
    assert!(succeeded.notes.iter().any(|note| { return note.contains("left out"); }), "output that was cut is not a complete result: {:?}", succeeded.notes);
    remove(&root);
}

/// The stop path hands the killed leader to the reaper the run was given,
/// after the captured pages were released (sol-1 round 3,
/// P2 E and the test review:
/// nothing showed that `run_tool` reaps within a budget when a killed leader does not end).
/// The tool says
/// when it has written,
/// and the control checks that before it checks that the pages were released,
/// so a tool that had not run yet
/// cannot make the emptiness hold by itself.
#[test]
fn a_stopped_tool_is_reaped_through_the_budgeted_reaper_after_its_output_is_released() {
    let root: PathBuf = fixture("runs-stop-reaper");
    let program: PathBuf = root.join("tool");
    let written: PathBuf = root.join("written");
    executable(program.as_path(), format!("#!/bin/sh\necho before-the-stop\n: > '{}'\nsleep {SLOW_SECONDS}\n", written.display()).as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let calls = std::cell::Cell::new(0_u32);
    let verdict = std::cell::RefCell::new(String::new());
    let run: ToolRun = run_on_runtime(run_tool_reaping(program.as_path(), &command, ToolLimits { window: ROOMY, ceiling: Duration::from_secs(3) }, &async |mut leader: Child, captured: &Capture| -> bool {
        calls.set(calls.get() + 1);
        let tool_had_written: bool = written.exists();
        let emptied: bool = captured.snapshot(1024).stdout.is_empty();
        *verdict.borrow_mut() = format!("written={tool_had_written} emptied={emptied}");
        // A leader that does not end is the reaper's to wait for: this one is reaped here.
        let _ended = leader.wait();
        return false;
    })).expect("the runtime runs the tool");
    assert_eq!(run.end, ToolEnd::CeilingReached(Duration::from_secs(3)));
    assert_eq!(calls.get(), 1, "the stop path reaps through the reaper it was given");
    assert_eq!(verdict.borrow().as_str(), "written=true emptied=true", "the tool had written, and what it wrote was released before the reaper ran");
    remove(&root);
}

/// A failure to wait for the tool is a stop like any other:
/// the process group is stopped,
/// the pages are released and the leader goes
/// to the reaper (sol-1 round 4:
/// the wait error returned before any of that,
/// and a descendant holding the files kept the pages).
/// The
/// seam makes the wait fail once the tool says it has written.
#[test]
fn a_wait_that_fails_stops_the_tool_and_releases_its_output() {
    let root: PathBuf = fixture("runs-wait-error");
    let program: PathBuf = root.join("tool");
    let written: PathBuf = root.join("written");
    executable(program.as_path(), format!("#!/bin/sh\necho before-the-failure\n: > '{}'\nsleep {SLOW_SECONDS}\n", written.display()).as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    WAIT_FAILS_ONCE_THIS_FILE_EXISTS.with(|seam| { *seam.borrow_mut() = Some(written.clone()); });
    let calls = std::cell::Cell::new(0_u32);
    let verdict = std::cell::RefCell::new(String::new());
    let run: ToolRun = run_on_runtime(run_tool_reaping(program.as_path(), &command, ToolLimits { window: ROOMY, ceiling: ROOMY }, &async |mut leader: Child, captured: &Capture| -> bool {
        calls.set(calls.get() + 1);
        *verdict.borrow_mut() = format!("emptied={}", captured.snapshot(1024).stdout.is_empty());
        let _ended = leader.wait();
        return false;
    })).expect("the runtime runs the tool");
    WAIT_FAILS_ONCE_THIS_FILE_EXISTS.with(|seam| { *seam.borrow_mut() = None; });
    let ToolEnd::Failed(text) = &run.end else { panic!("{run:?}") };
    assert!(text.contains("could not be waited for"), "{text}");
    assert_eq!(calls.get(), 1, "the leader goes to the reaper");
    assert_eq!(verdict.borrow().as_str(), "emptied=true", "the output was released");
    remove(&root);
}

/// The wait for a tool is the runtime's:
/// a task beside it runs while the tool does,
/// and the run still reports its end.
#[test]
fn the_wait_for_a_running_tool_lets_other_tasks_run() {
    let root: PathBuf = fixture("runs-wait-beside-a-ticker");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), b"#!/bin/sh\nsleep 0.3\n");
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let (run, ticks): (ToolRun, u32) = crate::ticker_beside_a_wait::with_a_ticker_beside(run_tool_reaping(
        program.as_path(),
        &command,
        ToolLimits { window: GENEROUS, ceiling: GENEROUS },
        &async |leader: Child, _captured: &Capture| -> bool { return reap_within(leader, Duration::from_secs(1)).await; },
    ));
    assert_eq!(run.end, ToolEnd::Succeeded);
    assert!(ticks > 0, "the wait for the tool held the thread: the task beside it never ran");
    remove(&root);
}

/// The reaping of a stopped leader is the runtime's too:
/// a task beside it runs through the whole budget when the leader does not end.
#[test]
fn the_reaping_of_a_leader_that_does_not_end_lets_other_tasks_run() {
    let mut command: Command = Command::new("sleep");
    command.arg("30");
    let child: Child = command.spawn().expect("the leader");
    let pid: i32 = i32::try_from(child.id()).expect("a process number");
    let (reaped, ticks): (bool, u32) = crate::ticker_beside_a_wait::with_a_ticker_beside(reap_within(child, Duration::from_millis(300)));
    assert!(!reaped, "the leader had not ended");
    assert!(ticks > 0, "the reaping held the thread: the task beside it never ran");
    // SAFETY: the control ends the process it started.
    let _ended = unsafe { libc::kill(pid, libc::SIGKILL) };
    assert!(is_gone_soon(pid), "the task the leader was handed to reaped it when it ended");
}

/// What:
///  A tool that is still running when the notice bound has passed says once which tool it is,
/// where it runs,
/// and what ends it.
/// Why:
///  An add whose `pnpm` never ended printed its last line at two seconds and then nothing for 613 s,
/// though the quiet window that
///  ends such a tool and the ceiling above it were both running
/// (`wait_ceilings.rs`,
/// `REPAIR_TOOL_NOTICE_AFTER`):
/// a wait nobody can see is indistinguishable from a hang.
/// The line is said once,
/// because a line every few seconds buries the output a person
///  reads for everything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const run = runTool(slow, command, limits); expect(noticesSaid().filter((line) => line.includes(root))).toHaveLength(1);
/// ```
#[test]
fn a_tool_that_outlasts_the_notice_bound_says_once_what_it_waits_for() {
    let root: PathBuf = fixture("says-what-it-waits-for");
    let program: PathBuf = root.join("pnpm");
    // Long past the notice bound, and short of the quiet window so the tool ends by itself.
    executable(program.as_path(), b"#!/bin/sh\necho working\nsleep 3\n");
    let command: ToolCommand = ToolCommand { arguments: vec![OsString::from("install"), OsString::from("--offline")], directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: ROOMY });
    assert_eq!(run.end, ToolEnd::Succeeded);
    let said: Vec<String> = notices_said().into_iter().filter(|line| { return line.contains(root.display().to_string().as_str()); }).collect();
    assert_eq!(said.len(), 1, "one line, and one only: {said:?}");
    let line: &String = &said[0];
    assert!(line.contains("pnpm"), "the line names the tool: {line}");
    assert!(line.contains("install --offline"), "the line names what it runs: {line}");
    assert!(line.contains("60 seconds") && line.contains("60 minutes"), "the line names both bounds that end it: {line}");
    remove(&root);
}

/// What:
///  A tool that ends before the notice bound says nothing about scaling.
/// Why:
///   Every add starts several tools,
/// and a line for each of them would bury the one line that says what became of the add;
/// a tool that works quickly
///  is what the person expects and needs no telling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runTool(quick, command, limits); expect(noticesSaid().filter((line) => line.includes(root))).toHaveLength(0);
/// ```
#[test]
fn a_tool_that_ends_before_the_notice_bound_says_nothing() {
    let root: PathBuf = fixture("says-nothing-when-quick");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), b"#!/bin/sh\nexit 0\n");
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, ToolLimits { window: GENEROUS, ceiling: ROOMY });
    assert_eq!(run.end, ToolEnd::Succeeded);
    let said: Vec<String> = notices_said().into_iter().filter(|line| { return line.contains(root.display().to_string().as_str()); }).collect();
    assert!(said.is_empty(), "a quick tool says nothing: {said:?}");
    remove(&root);
}

/// The longest a start may take before the run ends:
/// short, so a control reaches it in milliseconds.
const BRIEF_START: Duration = Duration::from_millis(300);

/// How long the control below holds the child before its own `exec`:
/// well past the start bound and well inside this control's own deadline.
const HELD_BEFORE_EXEC: Duration = Duration::from_secs(4);

/// The numbers of this process's own children,
/// read from every thread of it,
/// because a child the blocking pool forked is listed under that thread.
fn children_of_this_process() -> Vec<i32> {
    let mut found: Vec<i32> = Vec::new();
    let Ok(threads) = std::fs::read_dir("/proc/self/task") else { return found };
    for thread in threads.flatten() {
        let Ok(listed) = std::fs::read_to_string(thread.path().join("children")) else { continue };
        found.extend(listed.split_whitespace().filter_map(|number| { return number.parse::<i32>().ok(); }));
    }
    return found;
}

/// What:
///  A start that gives no answer ends the run at its own bound,
///  and the child that finally appears is stopped instead of rewriting the new worktree.
/// Why:
///  `Command::spawn` returns only after the child's own `exec`:
///  the standard library reads the pipe `exec` closes,
///  and that `exec` can wait in the kernel with no end,
///  so a start that does not answer waits for ever while the quiet window,
///  the ceiling and the notice are all read from a clock the run has not reached.
///  A child that appears after the bound must not run,
///  because the run has already told the person the tool did not start and named the command to run by hand.
///  The seam holds the child before its own `exec`,
///  which is the only place a control can make a start itself take time;
///  on 6777017fc4 the run waited the whole hold and then ran the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdBeforeExec(4_000); const run = await runToolReapingStartedWithin(tool, command, limits, 300, reap);
/// expect(run.end).toMatchObject({ kind: 'failed' }); await expect(theHeldChild).toBeGone();
/// ```
#[test]
fn a_start_that_gives_no_answer_ends_the_run_at_its_bound_and_stops_the_child_it_started_late() {
    let root: PathBuf = fixture("runs-start-bound");
    let program: PathBuf = root.join("tool");
    let ran: PathBuf = root.join("ran");
    executable(program.as_path(), format!("#!/bin/sh\nsleep 2\n: > '{}'\nexec sleep {SLOW_SECONDS}\n", ran.display()).as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    crate::package_repair_start::HOLD_BEFORE_EXEC.with(|seam| seam.set(Some(HELD_BEFORE_EXEC)));
    let began: Instant = Instant::now();
    let run: ToolRun = run_on_runtime(run_tool_reaping_started_within(program.as_path(), &command, ToolLimits { window: BRIEF, ceiling: SHORT_CEILING }, BRIEF_START, &reap_within_stop_grace))
        .expect("the runtime runs the tool");
    let waited: Duration = began.elapsed();
    crate::package_repair_start::HOLD_BEFORE_EXEC.with(|seam| seam.set(None));
    // The held child is this process's own, so the control can see it leave once the hold ends and the child execs.
    let held: Vec<i32> = children_of_this_process();
    assert!(waited < HELD_BEFORE_EXEC, "the run waited {waited:?} for a start it bounds at {BRIEF_START:?}");
    assert!(run.millis < HELD_BEFORE_EXEC.as_millis(), "the run reported {} ms, past the {HELD_BEFORE_EXEC:?} the child was held", run.millis);
    let ToolEnd::Failed(text) = &run.end else { panic!("a start that gave no answer is a failure of the start, not {:?}", run.end) };
    assert!(text.contains("could not start"), "the reason says the tool never started here: {text}");
    assert!(text.contains("gave no answer"), "the reason names the start that did not answer: {text}");
    assert!(!held.is_empty(), "the control's own child was held before its exec when the run ended");
    for pid in held {
        assert!(is_gone_soon(pid), "the child {pid} the run left started outlived the answer that it did not start");
    }
    assert!(!ran.exists(), "the tool ran in the worktree after the run said it did not start");
    remove(&root);
}
