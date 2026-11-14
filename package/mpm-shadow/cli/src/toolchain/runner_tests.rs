//! Unit tests of the program-start seam:
//! the real runner on harmless programs,
//! the toolchain environment,
//! and the
//! recording fake.

use std::collections::BTreeMap;
use std::path::Path;

use super::fakes::FakeRunner;
use super::runner::{ProcessRunner, RunRequest, Runner, toolchain_path_env};
use super::scope::{EnvMap, EXPOSURE_DIR_RELATIVE, repo_scope};
use crate::test_scratch::scratch;

#[test]
fn the_real_runner_captures_output_and_the_exit_code() {
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "echo out; echo err 1>&2; exit 3"]));
    assert_eq!(outcome.stdout, "out\n");
    assert_eq!(outcome.stderr, "err\n");
    assert_eq!(outcome.code, 3);
    assert!(!outcome.succeeded());
    assert_eq!(outcome.failure_text(), "err");
}

#[test]
fn failure_text_falls_back_to_standard_output() {
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "echo only-out; exit 1"]));
    assert_eq!(outcome.failure_text(), "only-out");
}

#[test]
fn a_program_that_cannot_start_reports_code_one_and_the_reason() {
    let outcome = ProcessRunner.run(&RunRequest::new("/nonexistent/definitely-not-a-program", &[]));
    assert_eq!(outcome.code, 1);
    assert!(outcome.stderr.contains("cannot start /nonexistent/definitely-not-a-program"), "{}", outcome.stderr);
}

#[test]
fn an_explicit_environment_replaces_the_inherited_one() {
    let mut env = BTreeMap::new();
    env.insert("ONLY_THIS".to_string(), "yes".to_string());
    env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "echo \"$ONLY_THIS:${HOME:-unset}\""]).with_env(env));
    assert_eq!(outcome.stdout, "yes:unset\n");
}

#[test]
fn the_working_directory_is_honoured() {
    let directory = scratch();
    let outcome = ProcessRunner.run(&RunRequest::new("pwd", &[]).with_cwd(directory.path().to_path_buf()));
    let printed = std::fs::canonicalize(outcome.stdout.trim()).expect("a real directory");
    assert_eq!(printed, std::fs::canonicalize(directory.path()).expect("a real directory"));
}

#[test]
fn the_toolchain_environment_sets_home_and_leads_path_with_the_scope_directories() {
    let env = EnvMap::from_pairs(&[("HOME", "/real/home"), ("PATH", "/usr/bin:/bin"), ("KEEP", "me"), ("CARGO_HOME", "/cargo")]);
    let scope = repo_scope(std::path::Path::new("/work"), &env).expect("a scope");
    let merged = toolchain_path_env(&scope, &env);
    assert_eq!(merged.get("HOME").map(String::as_str), Some("/real/home"));
    assert_eq!(merged.get("KEEP").map(String::as_str), Some("me"));
    let path = merged.get("PATH").expect("PATH");
    let parts: Vec<&str> = path.split(':').collect();
    assert_eq!(parts[0], Path::new("/work").join(EXPOSURE_DIR_RELATIVE).to_str().expect("a path of utf-8 names"));
    assert!(parts.contains(&"/cargo/bin"));
    assert_eq!(&parts[parts.len() - 2..], &["/usr/bin", "/bin"]);
}

#[test]
fn the_fake_records_requests_answers_by_prefix_and_defaults_to_success() {
    let runner = FakeRunner::new();
    runner.answer("cargo binstall", "installed", 0);
    runner.fail("rustup toolchain install", "no such toolchain");
    let installed = runner.run(&RunRequest::new("/some/dir/cargo", &["binstall", "--no-confirm", "fd-find"]));
    assert_eq!(installed.stdout, "installed");
    let failed = runner.run(&RunRequest::new("rustup", &["toolchain", "install", "nightly"]));
    assert_eq!(failed.code, 1);
    assert_eq!(failed.stderr, "no such toolchain");
    let unmatched = runner.run(&RunRequest::new("anything", &["else"]));
    assert!(unmatched.succeeded());
    assert_eq!(
        runner.lines(),
        vec![
            "cargo binstall --no-confirm fd-find".to_string(),
            "rustup toolchain install nightly".to_string(),
            "anything else".to_string()
        ]
    );
    assert_eq!(runner.calls()[0].line(), "/some/dir/cargo binstall --no-confirm fd-find");
}

//region the bound every program runs under

/// What:
/// A program that leaves one of its own children holding its output does not hold the runner:
/// the text it printed is what the runner reports, and it reports it at once.
/// Why:
/// `Command::output` reads both pipes to their end,
/// and a pipe stays open while any process of the program's own tree holds it,
/// so one stand-in that starts a background program of its own held an install for as long as that program lived.
#[test]
fn a_program_that_leaves_a_child_holding_its_output_does_not_hold_the_runner() {
    let started: std::time::Instant = std::time::Instant::now();
    // The shell exits at once; the `sleep` it started holds the same output pipes for 30 s.
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "sleep 30 & echo out; exit 0"]));
    let took: std::time::Duration = started.elapsed();
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.stdout, "out\n");
    assert!(took < std::time::Duration::from_secs(10), "the runner waited {took:?} on a pipe the program's child holds");
}

/// What:
/// A program that outlives its bound is stopped, and the outcome names it, the bound and what it had printed.
/// Why:
/// Nothing may wait forever:
/// a program that hangs on a network read or on a lock of its own ends the command instead of holding it,
/// and the line has to say why the command stopped where it did.
#[test]
fn a_program_that_outlives_its_bound_is_stopped_and_named() {
    let bound: std::time::Duration = std::time::Duration::from_secs(1);
    let started: std::time::Instant = std::time::Instant::now();
    let outcome = ProcessRunner.run(&RunRequest::new("sh", &["-c", "echo started; sleep 30"]).bounded(bound));
    let took: std::time::Duration = started.elapsed();
    assert_eq!(outcome.code, 1, "{}", outcome.stderr);
    assert!(outcome.stderr.contains("sh -c echo started; sleep 30"), "the sentence names the program: {}", outcome.stderr);
    assert!(outcome.stderr.contains("after its bound of 1 s and was stopped"), "the sentence names the bound: {}", outcome.stderr);
    assert!(outcome.stderr.contains("started"), "the sentence keeps what the program had printed: {}", outcome.stderr);
    assert!(took < std::time::Duration::from_secs(20), "the runner waited {took:?} on a program bound to 1 s");
}

//endregion
