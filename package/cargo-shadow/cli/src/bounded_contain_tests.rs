//! What:
//! Round 6 tests of what a bounded run contains:
//! descendants that leave the process group,
//! the group record made before the child
//! can run,
//! and the scope of that record.
//! Why:
//! Fifth review 5a F1,
//! F2,
//! 5b2 6 and 5c N2:
//! a process group is not an operation boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('containment of a bounded run', () => { it(...) })
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::bounded_run::{Bound, RunError, output_within};
use crate::crates::LockSnapshot;
use crate::journal::begin;
use crate::journal_support::journal_dirs;
use crate::test_scratch::scratch;

/// A bound for the commands of these tests.
fn bound(limit: Duration) -> Bound {
    return Bound { limit, what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// The script of a parent that starts a writer in a session of its own (it also closes every standard stream,
/// so no pipe reaches it),
/// waits until the writer says it escaped (its process id is in a file, then a ready file exists),
/// and then does `after`.
/// The writer lives on until it is stopped.
fn script_with_escaped_writer(directory: &Path, after: &str) -> String {
    let ready = directory.join("ready");
    let pid_file = directory.join("pid");
    return format!(
        "setsid sh -c 'exec </dev/null >/dev/null 2>&1; echo $$ > {pid}; : > {ready}; exec sleep 600' & \
         while [ ! -e {ready} ]; do sleep 0.02; done; {after}",
        ready = ready.display(),
        pid = pid_file.display()
    );
}

/// The process id the escaped writer recorded.
fn writer_pid(directory: &Path) -> i32 {
    return crate::contain_counterexample_tests::read_pid(&directory.join("pid"));
}

/// Round 6 (5a F1,
/// 5c N2):
/// a descendant that left the process group and closed the pipes is stopped before a normal exit returns.
#[test]
fn a_writer_in_another_session_is_stopped_before_a_normal_exit_returns() {
    let directory = scratch();
    let script = script_with_escaped_writer(directory.path(), "echo done");

    let output = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(20))).expect("runs");
    let writer = writer_pid(directory.path());

    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    assert!(!crate::contain_counterexample_tests::is_running(writer), "a descendant is still running after the run had returned");
}

/// Round 6 (5a F1,
/// 5c N2):
/// the same at the bound.
#[test]
fn a_writer_in_another_session_is_stopped_at_the_bound() {
    let directory = scratch();
    let script = script_with_escaped_writer(directory.path(), "sleep 30");

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(1)));
    let writer = writer_pid(directory.path());

    assert!(matches!(result, Err(RunError::TimedOut { .. })), "{result:?}");
    assert!(!crate::contain_counterexample_tests::is_running(writer), "a descendant is still running after the run had returned");
}

/// Round 6 (5a P1,
/// 5b2 2):
/// a child whose group cannot be recorded is not started,
/// because a killed wrapper could then leave a writer that no recovery knows about.
#[test]
fn a_child_whose_group_cannot_be_recorded_is_not_started() {
    let root = scratch();
    fs::create_dir_all(root.path().join("package/c/one")).expect("dir");
    fs::write(root.path().join("package/c/one/Cargo.lock"), "lock").expect("lock");
    let transaction = begin(root.path(), &[LockSnapshot { crate_dir: "package/c/one".to_string(), source: "lock".to_string() }], false).expect("begin");
    let journal = journal_dirs(root.path()).into_iter().next().expect("a journal");
    fs::remove_file(journal.join("groups")).expect("remove the record");
    fs::create_dir(journal.join("groups")).expect("a directory where the record was, so appending to it fails");
    let marker = root.path().join("ran");
    let script = format!(": > {}", marker.display());

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(20)));
    std::thread::sleep(Duration::from_millis(300));

    assert!(result.is_err(), "a child ran with no record of its group");
    assert!(!marker.exists(), "the child ran before its group was recorded");
    drop(transaction);
}

/// Round 6 (5b2 6):
/// dropping an inner transaction does not stop the recording of the outer one,
/// and dropping the outer one first does not stop the inner one's.
#[test]
fn nested_transactions_keep_their_own_records() {
    let first = scratch();
    let second = scratch();
    for root in [&first, &second] {
        fs::create_dir_all(root.path().join("package/c/one")).expect("dir");
        fs::write(root.path().join("package/c/one/Cargo.lock"), "lock").expect("lock");
    }
    let snapshot = LockSnapshot { crate_dir: "package/c/one".to_string(), source: "lock".to_string() };
    let outer = begin(first.path(), std::slice::from_ref(&snapshot), false).expect("outer");
    let inner = begin(second.path(), std::slice::from_ref(&snapshot), false).expect("inner");
    drop(inner);

    output_within(&mut Command::new("true"), None, &bound(Duration::from_secs(20))).expect("runs");

    let outer_record = fs::read_to_string(journal_dirs(first.path()).into_iter().next().expect("journal").join("groups")).expect("groups");
    // One run records its scope and then its end.
    let lines: Vec<&str> = outer_record.lines().collect();
    assert_eq!(lines.len(), 2, "the outer transaction stopped recording: {outer_record:?}");
    assert!(lines[0].starts_with("scope ") && lines[1].starts_with("settled "), "{outer_record:?}");
    drop(outer);
}

/// Round 6 (5a F2):
/// a run that returns an error does not leave a worker reading the pipe.
#[test]
fn a_timeout_returns_promptly_even_when_the_child_ignores_its_input() {
    let input: Vec<u8> = vec![b'x'; 4_000_000];
    let started = Instant::now();

    let result = output_within(Command::new("sleep").arg("30"), Some(&input), &bound(Duration::from_millis(300)));

    assert!(matches!(result, Err(RunError::TimedOut { .. })), "{result:?}");
    assert!(started.elapsed() < Duration::from_secs(8), "{:?}", started.elapsed());
}

/// Round 7 (retires the round 6 refusal of a second run):
/// two runs of one process are each contained,
/// and the second neither waits for the first nor is refused,
/// because only the spawn of each is serialized.
#[test]
fn two_runs_in_one_process_are_contained_each_and_neither_waits_for_the_other() {
    let first = std::thread::spawn(|| { return output_within(Command::new("sleep").arg("2"), None, &bound(Duration::from_secs(20))); });
    std::thread::sleep(Duration::from_millis(400));
    let started = Instant::now();

    let second = output_within(&mut Command::new("true"), None, &bound(Duration::from_secs(20)));

    assert!(second.is_ok(), "the second run was refused: {second:?}");
    assert!(started.elapsed() < Duration::from_millis(1500), "the second run waited for the first: {:?}", started.elapsed());
    assert!(first.join().expect("first run thread").is_ok());
}
