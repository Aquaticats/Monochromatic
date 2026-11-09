//! The captured run:
//! a command started with the standard library,
//! awaited through its pidfd,
//! its output read from anonymous files.

use super::*;
use crate::lazy_runtime::run_on_runtime;
use std::process::Command;
use std::time::{Duration, Instant};

/// A shell command line as a command.
fn shell(line: &str) -> Command {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(line);
    return command;
}

/// The output and status of a command that succeeds come back,
/// with both streams kept apart.
#[test]
fn a_run_gives_status_and_both_streams() {
    let output = run_on_runtime(output_of(&mut shell("printf out; printf err >&2; exit 0"))).expect("the runtime").expect("the run");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"out");
    assert_eq!(output.stderr, b"err");
}

/// A failing command's exit code comes back as the status,
/// not as an error of the run.
#[test]
fn a_failing_command_is_a_status_not_an_error() {
    let output = run_on_runtime(output_of(&mut shell("exit 7"))).expect("the runtime").expect("the run");
    assert_eq!(output.status.code(), Some(7));
}

/// A program that does not exist is the error of the start,
/// as `Command::output` reports it.
#[test]
fn a_program_that_does_not_exist_is_a_start_error() {
    let mut command = Command::new("/nonexistent/program-for-the-captured-run");
    let error = run_on_runtime(output_of(&mut command)).expect("the runtime").expect_err("the start fails");
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

/// A command that writes more than the pipe holds still finishes:
/// the output goes to a file,
/// so nothing waits for a reader.
#[test]
fn output_larger_than_a_pipe_does_not_block_the_child() {
    let output = run_on_runtime(output_of(&mut shell("head -c 300000 /dev/zero"))).expect("the runtime").expect("the run");
    assert_eq!(output.stdout.len(), 300_000);
}

/// What:
///  A synchronous run whose ceiling nothing crosses gives its status and both streams.
/// Why:
///  The queries of the wrapper take this form (`repo_read/git_metadata.rs`),
/// so its happy path has to be proven.
#[test]
fn a_synchronous_run_within_its_ceiling_gives_its_output() {
    let output = output_of_within_bound(&mut shell("printf hello; printf there >&2"), Duration::from_secs(30)).expect("the run");
    assert_eq!((output.stdout, output.stderr), (b"hello".to_vec(), b"there".to_vec()));
}

/// What:
///  A synchronous run that does not end is ended at its ceiling and says so.
/// Why:
///   A read-only query has no watch and no meter,
/// so a program that never answers would hold the command for good
///  (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 5).
/// The stand-in outlives the control's own bound if nothing ends it,
/// so a run that outlasts the bound shows.
#[test]
fn a_synchronous_run_past_its_ceiling_is_ended_and_says_so() {
    let root: std::path::PathBuf = crate::test_support::fixture("captured-run-ceiling");
    let pid_file: std::path::PathBuf = root.join("pid");
    let mut command: Command = Command::new("/bin/sh");
    command.arg("-c").arg(format!("echo $$ > '{}'; sleep 6", pid_file.display()));
    let began: Instant = Instant::now();
    let failure = output_of_within_bound(&mut command, Duration::from_millis(700)).expect_err("the ceiling ends it");
    let waited: Duration = began.elapsed();
    let pid: i32 = std::fs::read_to_string(&pid_file).expect("the run recorded its process").trim().parse().expect("a process number");
    // The ceiling's own kill ends the stand-in, and the thread that waited for it reaps it; this waits for that end, bounded,
    //  because a process that was killed stays a zombie that signal 0 still reaches until it is reaped.
    let ending: Instant = Instant::now();
    while runs(pid) && ending.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    // Nothing is signalled here: the stand-in ends by itself if the ceiling failed to end it (`sleep 6`), and a number read from
    //  a file may name an unrelated process once the stand-in was reaped.
    let still_running: bool = runs(pid);
    crate::test_support::remove(root.as_path());
    assert!(waited < Duration::from_secs(3), "the run outlasted its ceiling by {waited:?}");
    assert_eq!(failure.kind(), std::io::ErrorKind::TimedOut, "{failure}");
    assert!(failure.to_string().contains("did not answer within"), "{failure}");
    assert!(!still_running, "the query the ceiling ended is not left running");
}

/// Whether the process `pid` still runs.
/// A process that was killed and waits to be reaped is a zombie,
/// which `kill` with signal 0 still reaches,
/// so the process table is what says it has exited.
fn runs(pid: i32) -> bool {
    return !matches!(crate::process_identity::process_birth_identity(i64::from(pid)), Ok(None));
}

/// The two Linux callers that used to block on `Command::output` use the captured run instead.
#[test]
fn the_one_shot_git_questions_no_longer_block_a_thread_on_command_output() {
    let sources: [(&str, &str); 2] = [
        ("repo_read/git_system_config.rs", include_str!("../repo_read/git_system_config.rs")),
        ("invocation/management_service_repository_selection.rs", include_str!("../invocation/management_service_repository_selection.rs")),
    ];
    for (name, source) in sources {
        let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
        assert!(!code.contains(".output()"), "{name} still blocks a thread on `Command::output`");
        assert!(code.contains("output_of_within_bound"), "{name} does not use the captured run");
    }
}

/// What:
///  Neither query path signals a process by its number.
/// Why:
///  A number is a safe address only while its process is this process's own unreaped child,
/// and the thread that waits for
///  a query reaps it as soon as it ends,
/// so the number read at the start may name another process by the time a ceiling
///  is reached.
/// Nothing on these paths signals a number at all:
/// the one-shot query is waited for,
/// and ended,
/// by the thread that
///  owns it (`wait_within`),
/// and the batch query is read by the runtime,
/// which owns its child for the same reason
///  (`read_until_the_child_ends`).
#[test]
fn neither_query_path_signals_by_number() {
    let sources: [(&str, &str); 2] = [
        ("process/captured_child_run_to_its_end.rs", include_str!("captured_child_run_to_its_end.rs")),
        ("push/manual_push_range.rs", include_str!("../push/manual_push_range.rs")),
    ];
    for (name, source) in sources {
        let code: String = source.lines().filter(|line| { return !line.trim_start().starts_with("//"); }).collect::<Vec<&str>>().join("\n");
        assert!(!code.contains("libc::kill"), "{name} still signals a process by its number");
        assert!(
            code.contains("wait_within") || code.contains("read_until_the_child_ends"),
            "{name} does not wait for its child with a ceiling that its own thread keeps"
        );
    }
}
