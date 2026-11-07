//! The captured run:
//! a command started with the standard library,
//! awaited through its pidfd,
//! its output read from anonymous files.

use super::*;
use std::process::Command;

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

/// The blocking face of the run refuses,
/// with an error,
/// when called from inside the runtime,
/// where it would deadlock.
#[test]
fn the_blocking_face_refuses_inside_the_runtime() {
    let inner = run_on_runtime(async { return output_of_blocking(&mut shell("exit 0")).map(|_| ()); }).expect("the runtime");
    assert!(inner.is_err());
}

/// The blocking face works from plain synchronous code.
#[test]
fn the_blocking_face_runs_a_command_from_synchronous_code() {
    let output = output_of_blocking(&mut shell("printf hello")).expect("the run");
    assert_eq!(output.stdout, b"hello");
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
        assert!(code.contains("output_of_blocking"), "{name} does not use the captured run");
    }
}
