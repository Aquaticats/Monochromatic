//! What:
//!  Controls for running one child with a time bound and output caps.
//! Why:
//!  A child that hangs,
//!  floods its output,
//!  ignores its input or dies by a signal must
//!      come back as its own failure,
//!  quickly,
//!  and must not be left running.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runBoundedChild({ executable: sleeper, limits: { timeoutMs: 200 } })).rejects.toEqual({ kind: 'timed-out' });
//! ```

/// The runner and its pure helpers.
use super::{
    ChildExit, ChildFailure, ChildLimits, ChildRequest, FinishedChild, deadline_reached, exceeds,
    next_pause, run_bounded_child, run_bounded_child_noticing, sleep_for,
};
use crate::test_support::{ActionSink, SharedSink, executable, fixture, remove};
use crate::wait_progress::Cadence;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Bounds generous enough that no ordinary stand-in reaches them.
const ROOMY: ChildLimits = ChildLimits {
    timeout: Duration::from_secs(60),
    max_stdout: 1 << 20,
    max_stderr: 1 << 20,
};

/// The environment of a stand-in that needs only system programs.
fn path_only() -> Vec<(OsString, OsString)> {
    return vec![(OsString::from("PATH"), OsString::from("/usr/bin:/bin"))];
}

/// Write a `/bin/sh` stand-in named `name` in `root` and return its path.
fn stand_in(root: &Path, name: &str, script: &str) -> PathBuf {
    let path: PathBuf = root.join(name);
    executable(path.as_path(), format!("#!/bin/sh\n{script}\n").as_bytes());
    return path;
}

/// Run `program` in `root` with `input`,
///  the given environment and limits.
fn run(
    root: &Path,
    program: &Path,
    input: &[u8],
    environment: &[(OsString, OsString)],
    limits: ChildLimits,
) -> Result<FinishedChild, ChildFailure> {
    let arguments: Vec<OsString> = vec![OsString::from("first"), OsString::from("sec ond")];
    return run_bounded_child(&ChildRequest {
        executable: program,
        arguments: arguments.as_slice(),
        directory: root,
        environment,
        input,
        scratch: root,
        limits,
        waiting_for: "the stand-in program",
        cadence: Cadence::standard(),
    });
}

/// The stand-in reads all its input,
///  sees exactly the given environment,
///  arguments and
/// directory,
///  and its exit code and both outputs come back exactly.
#[test]
fn a_child_reads_its_input_and_returns_both_outputs() {
    let root: PathBuf = fixture("bounded-echo");
    let program: PathBuf = stand_in(
        root.as_path(),
        "echo",
        "cat\nprintf '%s|%s|%s|%s|%s' \"$1\" \"$2\" \"$ONLY\" \"${HOME-unset}\" \"$(pwd)\" >&2\nexit 3",
    );
    let input: Vec<u8> = (0..=255).cycle().take(70_000).collect();
    let environment: Vec<(OsString, OsString)> = vec![
        (OsString::from("ONLY"), OsString::from("value")),
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
    ];
    let finished: FinishedChild = run(
        root.as_path(),
        program.as_path(),
        input.as_slice(),
        environment.as_slice(),
        ROOMY,
    )
    .expect("finished");
    assert_eq!(finished.exit, ChildExit::Code(3));
    assert_eq!(finished.stdout, input);
    assert_eq!(
        String::from_utf8(finished.stderr).expect("text"),
        format!("first|sec ond|value|unset|{}", root.display())
    );
    remove(root.as_path());
}

/// A child that never reads its input,
///  of any size,
///  ends normally and quickly.
#[test]
fn a_child_that_ignores_its_input_ends_quickly() {
    let root: PathBuf = fixture("bounded-ignore");
    let program: PathBuf = stand_in(root.as_path(), "ignore", "printf done");
    let started: Instant = Instant::now();
    let finished: FinishedChild = run(
        root.as_path(),
        program.as_path(),
        vec![b'x'; 4 << 20].as_slice(),
        path_only().as_slice(),
        ROOMY,
    )
    .expect("finished");
    assert_eq!(
        (finished.exit, finished.stdout),
        (ChildExit::Code(0), b"done".to_vec())
    );
    // Far below the 60-second bound: the parent never sleeps out the bound.
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    remove(root.as_path());
}

/// A child ended by a signal reports that signal.
#[test]
fn a_child_ended_by_a_signal_reports_it() {
    let root: PathBuf = fixture("bounded-signal");
    let program: PathBuf = stand_in(root.as_path(), "signal", "kill -TERM $$");
    let finished: FinishedChild = run(
        root.as_path(),
        program.as_path(),
        b"",
        path_only().as_slice(),
        ROOMY,
    )
    .expect("finished");
    assert_eq!(finished.exit, ChildExit::Signal(15));
    remove(root.as_path());
}

/// Whether a process with this identifier still exists,
///  by `/proc`.
fn running(pid: &str) -> bool {
    return Path::new("/proc").join(pid).exists();
}

/// A child still running at its bound is killed and collected,
///  and the run says so.
#[test]
fn a_child_past_its_bound_is_killed() {
    let root: PathBuf = fixture("bounded-timeout");
    let pid_file: PathBuf = root.join("pid");
    let program: PathBuf = stand_in(
        root.as_path(),
        "sleeper",
        format!("echo $$ > '{}'\nexec sleep 30", pid_file.display()).as_str(),
    );
    let started: Instant = Instant::now();
    let failure: ChildFailure = run(
        root.as_path(),
        program.as_path(),
        b"",
        path_only().as_slice(),
        ChildLimits {
            timeout: Duration::from_millis(300),
            ..ROOMY
        },
    )
    .expect_err("timed out");
    assert_eq!(failure, ChildFailure::TimedOut(Duration::from_millis(300)));
    assert!(started.elapsed() >= Duration::from_millis(300));
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    let pid: String = std::fs::read_to_string(&pid_file)
        .expect("pid")
        .trim()
        .to_owned();
    assert!(!running(pid.as_str()), "process {pid} is still running");
    remove(root.as_path());
}

/// Output past either cap stops the child;
///  output exactly at the cap is kept whole.
#[test]
fn output_past_a_cap_stops_the_child() {
    let root: PathBuf = fixture("bounded-cap");
    let flood: PathBuf = stand_in(root.as_path(), "flood", "exec head -c 50000000 /dev/zero");
    let failure: ChildFailure = run(
        root.as_path(),
        flood.as_path(),
        b"",
        path_only().as_slice(),
        ChildLimits {
            max_stdout: 1000,
            ..ROOMY
        },
    )
    .expect_err("too large");
    assert_eq!(
        failure,
        ChildFailure::TooLarge {
            stream: "standard output",
            limit: 1000
        }
    );
    let errors: PathBuf = stand_in(
        root.as_path(),
        "errors",
        "exec head -c 50000000 /dev/zero >&2",
    );
    assert_eq!(
        run(
            root.as_path(),
            errors.as_path(),
            b"",
            path_only().as_slice(),
            ChildLimits {
                max_stderr: 10,
                ..ROOMY
            }
        ),
        Err(ChildFailure::TooLarge {
            stream: "standard error",
            limit: 10
        })
    );
    let exact: PathBuf = stand_in(root.as_path(), "exact", "printf 12345\nprintf abc >&2");
    let at_cap: FinishedChild = run(
        root.as_path(),
        exact.as_path(),
        b"",
        path_only().as_slice(),
        ChildLimits {
            timeout: ROOMY.timeout,
            max_stdout: 5,
            max_stderr: 3,
        },
    )
    .expect("at the cap");
    assert_eq!(
        (at_cap.stdout, at_cap.stderr),
        (b"12345".to_vec(), b"abc".to_vec())
    );
    // A child that already ended is still reported when it wrote one byte too many.
    assert_eq!(
        run(
            root.as_path(),
            exact.as_path(),
            b"",
            path_only().as_slice(),
            ChildLimits {
                timeout: ROOMY.timeout,
                max_stdout: 4,
                max_stderr: 3,
            }
        ),
        Err(ChildFailure::TooLarge {
            stream: "standard output",
            limit: 4
        })
    );
    remove(root.as_path());
}

/// A program that does not exist,
///  and a scratch directory that does not exist,
///  fail
/// before anything runs,
///  each with its own cause.
#[test]
fn a_child_that_cannot_start_reports_why() {
    let root: PathBuf = fixture("bounded-start");
    let missing: PathBuf = root.join("missing");
    match run(
        root.as_path(),
        missing.as_path(),
        b"",
        path_only().as_slice(),
        ROOMY,
    ) {
        Err(ChildFailure::Start(reason)) => assert!(reason.contains("No such file"), "{reason}"),
        other => panic!("{other:?}"),
    }
    let program: PathBuf = stand_in(root.as_path(), "fine", "exit 0");
    let nowhere: PathBuf = root.join("no-scratch");
    let arguments: Vec<OsString> = Vec::new();
    match run_bounded_child(&ChildRequest {
        executable: program.as_path(),
        arguments: arguments.as_slice(),
        directory: root.as_path(),
        environment: &[],
        input: b"",
        scratch: nowhere.as_path(),
        limits: ROOMY,
        waiting_for: "the stand-in program",
        cadence: Cadence::standard(),
    }) {
        Err(ChildFailure::Files(reason)) => assert!(reason.contains("No such file"), "{reason}"),
        other => panic!("{other:?}"),
    }
    remove(root.as_path());
}

/// The cap comparison keeps a size equal to the cap;
///  the bound is reached at equality.
#[test]
fn caps_and_bounds_are_inclusive_where_they_should_be() {
    assert!(!exceeds(5, 5));
    assert!(exceeds(6, 5));
    assert!(!exceeds(0, 0));
    assert!(exceeds(u64::MAX, usize::MAX - 1));
    let five: Duration = Duration::from_millis(5);
    assert!(deadline_reached(five, five));
    assert!(!deadline_reached(Duration::from_millis(4), five));
    assert!(deadline_reached(Duration::from_millis(6), five));
}

/// Pauses double from one millisecond and settle at sixteen;
///  a sleep never passes the bound.
#[test]
fn pauses_double_and_never_pass_the_bound() {
    let mut pauses: Vec<u128> = Vec::new();
    let mut pause: Duration = Duration::from_millis(1);
    for _ in 0..7 {
        pauses.push(pause.as_millis());
        pause = next_pause(pause);
    }
    assert_eq!(pauses, [1, 2, 4, 8, 16, 16, 16]);
    assert_eq!(
        sleep_for(Duration::from_millis(16), Duration::from_millis(3)),
        Duration::from_millis(3)
    );
    assert_eq!(
        sleep_for(Duration::from_millis(2), Duration::from_millis(3)),
        Duration::from_millis(2)
    );
}

/// Run `program` with the given cadence and return the result; the notices the wait printed go to `notices`.
fn run_noticing(
    root: &Path,
    program: &Path,
    limits: ChildLimits,
    cadence: Cadence,
    notices: &mut dyn std::io::Write,
) -> Result<FinishedChild, ChildFailure> {
    let arguments: Vec<OsString> = Vec::new();
    return run_bounded_child_noticing(
        &ChildRequest {
            executable: program,
            arguments: arguments.as_slice(),
            directory: root,
            environment: path_only().as_slice(),
            input: b"",
            scratch: root,
            limits,
            waiting_for: "the stand-in program to check a.md",
            cadence,
        },
        notices,
    );
}

/// A cadence of a 20 ms grace and a 20 ms interval, so a notice comes fast.
const QUICK: Cadence = Cadence {
    grace: Duration::from_millis(20),
    interval: Duration::from_millis(20),
};

/// A child that runs long is waited for with a notice that names what the wait is for, the child's process and the
///  bound, repeated at every interval; the child's output stays untouched.
///  The child waits for a release file that the third notice creates, so the wait ends on that event and at least
///  three notices are out, however slowly the host runs the parent.
#[test]
fn a_child_that_runs_long_is_waited_for_with_notices() {
    let root: PathBuf = fixture("bounded-notices");
    let program: PathBuf = stand_in(root.as_path(), "slow", "while [ ! -e release ]; do sleep 0.02; done\nprintf done");
    let shared: SharedSink = SharedSink::default();
    let release: PathBuf = root.join("release");
    let mut sink: ActionSink = ActionSink::new(&shared, 3, move || {
        std::fs::write(&release, b"").expect("release the child");
    });
    let result = run_noticing(root.as_path(), program.as_path(), ROOMY, QUICK, &mut sink);
    assert_eq!(result.expect("finished").stdout, b"done".to_vec());
    let notices: String = shared.text();
    let lines: Vec<&str> = notices.lines().collect();
    assert!(lines.len() >= 3, "{notices}");
    for line in &lines {
        assert!(
            line.starts_with("cli-git: still waiting for the stand-in program to check a.md (process "),
            "{notices}"
        );
        assert!(line.contains("stopping it after 60 seconds"), "{notices}");
    }
    assert_eq!(notices.matches("Hint:").count(), 1, "{notices}");
    remove(root.as_path());
}

/// A child that finishes inside the grace prints nothing.
///  The grace is an hour, so no host is slow enough to make the first look late.
#[test]
fn a_child_that_finishes_inside_the_grace_prints_nothing() {
    let root: PathBuf = fixture("bounded-quiet");
    let program: PathBuf = stand_in(root.as_path(), "quick", "printf done");
    let mut notices: Vec<u8> = Vec::new();
    let result = run_noticing(
        root.as_path(),
        program.as_path(),
        ROOMY,
        Cadence { grace: Duration::from_secs(3600), interval: Duration::from_secs(3600) },
        &mut notices,
    );
    assert_eq!(result.expect("finished").stdout, b"done".to_vec());
    assert!(notices.is_empty());
    remove(root.as_path());
}

/// A child that outlasts its bound is announced and then stopped:
///  the notice that is due goes out before the bound is checked, so at least one notice is out however slowly the
///  host runs the parent.
#[test]
fn a_child_that_outlasts_its_bound_is_announced_then_stopped() {
    let root: PathBuf = fixture("bounded-notices-timeout");
    let program: PathBuf = stand_in(root.as_path(), "stuck", "sleep 30");
    let mut notices: Vec<u8> = Vec::new();
    let result = run_noticing(
        root.as_path(),
        program.as_path(),
        ChildLimits { timeout: Duration::from_millis(450), ..ROOMY },
        QUICK,
        &mut notices,
    );
    assert_eq!(result.expect_err("stopped"), ChildFailure::TimedOut(Duration::from_millis(450)));
    let text: String = String::from_utf8(notices).expect("text");
    assert!(text.lines().count() >= 1, "{text}");
    remove(root.as_path());
}
