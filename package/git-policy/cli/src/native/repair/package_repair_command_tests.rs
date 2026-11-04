//! Finding a program on a search path and running it with a limit: found, missing, not executable, succeeded, failed with
//!  its output, stopped at the limit, run in the named directory with the added variables.

use super::*;
use crate::test_support::executable;
use crate::test_support::{fixture, remove};

/// A limit long enough that no control reaches it unless the control is about the limit.
const GENEROUS: Duration = Duration::from_secs(60);

/// A limit short enough to wait out.
const BRIEF: Duration = Duration::from_millis(300);

/// The status the failing fixture exits with.
const FAILING_STATUS: i32 = 7;

/// How long the slow fixture would sleep if nothing stopped it; far past any limit used here.
const SLOW_SECONDS: u64 = 30;

/// The first matching executable wins, in the order of the search path.
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

/// A name that is absent, a file that is not executable and an absent search path are all "not found".
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

/// The program runs in the named directory with the added variable, and a zero status is a success.
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
    let run: ToolRun = run_tool(program.as_path(), &command, GENEROUS);
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
    let run: ToolRun = run_tool(program.as_path(), &command, GENEROUS);
    let ToolEnd::Failed(text) = run.end else { panic!("expected a failure, got {:?}", run.end) };
    assert!(text.contains(FAILING_STATUS.to_string().as_str()), "status missing in {text}");
    assert!(text.contains("reason printed on stderr"), "output missing in {text}");
    remove(&root);
}

/// A program that has shown no sign of work for its window is stopped, and the run says so.
#[test]
fn stops_a_program_that_shows_no_sign_of_work_for_its_window() {
    let root: PathBuf = fixture("runs-limit");
    let program: PathBuf = root.join("tool");
    executable(program.as_path(), format!("#!/bin/sh\nexec sleep {SLOW_SECONDS}\n").as_bytes());
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(program.as_path(), &command, BRIEF);
    assert_eq!(run.end, ToolEnd::TimedOut);
    assert!(run.millis < u128::from(SLOW_SECONDS) * 1000, "the run waited {} ms for a program stopped at its limit", run.millis);
    remove(&root);
}

/// A program that cannot start is a failure that names the cause, never a panic.
#[test]
fn reports_a_program_that_cannot_start() {
    let root: PathBuf = fixture("runs-unstartable");
    let command: ToolCommand = ToolCommand { arguments: Vec::new(), directory: root.clone(), set: Vec::new() };
    let run: ToolRun = run_tool(root.join("absent").as_path(), &command, GENEROUS);
    assert!(matches!(run.end, ToolEnd::Failed(_)), "got {:?}", run.end);
    remove(&root);
}

/// Lines that only say the tool waited for another process are dropped from a failure's text; the reason stays.
#[test]
fn a_failure_text_drops_wait_notices() {
    let text: String = tail_of(b"Blocking waiting for file lock on package cache\nerror: the real reason\n");
    assert_eq!(text, "error: the real reason");
}

/// The quiet window of the control below: longer than the sampling interval of the progress meter.
const WORKING_WINDOW: Duration = Duration::from_millis(1200);

/// A tool that keeps printing is never stopped however long it runs: the limit judges quiet time, not wall time.
///  The fixture prints a line every pause for a multiple of the window, with no busy loop.
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
    let run: ToolRun = run_tool(program.as_path(), &command, WORKING_WINDOW);
    assert_eq!(run.end, ToolEnd::Succeeded);
    assert!(run.millis > WORKING_WINDOW.as_millis() * 2, "the fixture ran only {} ms", run.millis);
    remove(&root);
}
