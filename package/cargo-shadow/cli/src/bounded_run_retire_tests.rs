//! What:
//! Controls that every path a bounded run controls asks the manager to stop the run's scope unit,
//! and leaves it unloaded
//! (issue 6578):
//! a normal end,
//! a bound that passes,
//! a refusal after the program started,
//! and a launcher that is gone.
//! Why:
//! The unit is left `active (running)` only when the manager missed the kernel's empty notification,
//! which is rare,
//! so a control that waited for that race could not tell a fixed run from a lucky one.
//! These controls read the request the run made instead:
//! the `systemctl` they give the route records every call and then runs the real one,
//! so the manager's state after the run is real,
//! and the request is the guard.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('a run retires its scope unit', () => { it('on a normal end', ...) })
//! ```

/// What:
/// `use std::fs;` imports the write of the recording stand-in and the read of its log.
/// Why:
/// The one file that stands between the run and the manager is written and then read back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports the trait that sets a file's mode.
/// Why:
/// The recording program has to be executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs';
/// ```
use std::os::unix::fs::PermissionsExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// Each control owns a scratch directory and lends it to the writer of the recording program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::{
    Path,
    PathBuf,
};
/// What:
/// `use std::process::{Command, Output};` imports the command builder and the collected output of a run.
/// Why:
/// The run under test is a command,
/// and its output is what a normal end returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnSync } from 'node:child_process';
/// ```
use std::process::{
    Command,
    Output,
};
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type (its sibling `Arc` is the one that is safe across threads).
/// Why:
/// The hooks of a run are set on this one thread and shared with the run's own calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
use std::rc::Rc;
/// What:
/// `use std::time::Duration;` imports a span of time.
/// Why:
/// Each run gets a bound,
/// and the guard that waits for the unit to be gone gets one too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// The bound a run of these controls gets unless it is the point of the control.
const RUN_BOUND: Duration = Duration::from_secs(30);
/// How long a run's unit is given to leave the manager.
const GONE_BOUND: Duration = Duration::from_secs(10);

/// What:
/// `use crate::bounded_run::{Bound, RunError, after_launch_for_test, on_startup_for_test, output_within, take_trace_for_test, use_route_settings_for_test};`
/// imports the run under test,
/// its bound and error,
/// the hooks that make its later paths,
/// and the trace that names its scope.
/// Why:
/// Every control here ends a real run and reads the unit it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Bound, RunError, outputWithin, ... } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    Bound,
    RunError,
    after_launch_for_test,
    on_startup_for_test,
    output_within,
    take_trace_for_test,
    use_route_settings_for_test,
};
/// What:
/// `use crate::cgroup_test_support::wait_up_to;` imports the bounded wait of the scope controls.
/// Why:
/// The unit is collected by the manager's next pass,
/// which this control waits for instead of sleeping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { waitUpTo } from 'crate/cgroup_test_support';
/// ```
use crate::cgroup_test_support::wait_up_to;
/// What:
/// `use crate::control_group::RouteSettings;` imports the settings that make the run ask the recording program instead of
/// `systemctl`.
/// Why:
/// That substitution is how the request is observed without a stand-in for the manager's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RouteSettings } from 'crate/control_group';
/// ```
use crate::control_group::RouteSettings;
/// What:
/// `use crate::round_sixteen_tests::launched_scope;` imports the reader of the scope name from a run's own trace.
/// Why:
/// The unit the run made is the name its own trace carries,
/// not a name a control guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { launchedScope } from 'crate/round_sixteen_tests';
/// ```
use crate::round_sixteen_tests::launched_scope;
/// What:
/// `use crate::test_scratch::scratch;` imports the disposable directory of one control.
/// Why:
/// The recording program and its log live in a fixture of the control's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { scratch } from 'crate/test_scratch';
/// ```
use crate::test_scratch::scratch;
/// What:
/// `use crate::failpoints::{Point, arm};` imports the test-only switches and how one is armed.
/// Why:
/// A run of a real program cannot leave the stuck scope of issue 6578 (a scope that held a process keeps `pids.peak`),
/// so the last look is made to read that state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Point, arm } from 'crate/failpoints';
/// ```
use crate::failpoints::{
    Point,
    arm,
};

/// What:
/// `fn bound(limit: Duration) -> Bound` is the bound of a run in these controls.
/// Why:
/// `Bound` carries what the caller was doing and how to repeat it,
/// which the refusal text names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bound(limit: number): Bound { return { limit, what: '...', rerun: '...' } }
/// ```
fn bound(limit: Duration) -> Bound {
    return Bound {
        limit,
        what: "the contained command".to_string(),
        rerun: "contained-command".to_string(),
    };
}

/// What:
/// `fn systemctl_path() -> String` is the first `systemctl` on `PATH`.
/// Why:
/// The recording program has to run the real one,
/// and the run's own environment finds it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function systemctlPath(): string
/// ```
fn systemctl_path() -> String {
    let search: String = std::env::var("PATH").unwrap_or_default();
    for directory in search.split(':') {
        let candidate: PathBuf = Path::new(directory).join("systemctl");
        if directory.is_empty() {
            continue;
        }
        if crate::control_group::is_executable_file(&candidate) {
            return candidate.display().to_string();
        }
    }
    panic!("no systemctl is on PATH, so this host has no user manager to ask");
}

/// What:
/// `fn recording_systemctl(directory: &Path) -> (PathBuf, PathBuf)` writes a program that appends its arguments to a log and then
/// runs the real `systemctl`,
/// and answers the program and the log.
/// Why:
/// The run's stop request is then readable while every answer still comes from the real manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordingSystemctl(dir: string): [string, string]
/// ```
fn recording_systemctl(directory: &Path) -> (PathBuf, PathBuf) {
    let program: PathBuf = directory.join("recording-systemctl");
    let log: PathBuf = directory.join("systemctl-calls");
    let text: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexec '{real}' \"$@\"\n",
        log = log.display(),
        real = systemctl_path()
    );
    fs::write(&program, text).expect("write the recording program");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return (program, log);
}

/// What:
/// `fn load_state(unit: &str) -> String` asks the real manager whether a unit is loaded,
/// trimmed.
/// Why:
/// The control's claim is about the manager's own state,
/// not the trace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function loadState(unit: string): string
/// ```
fn load_state(unit: &str) -> String {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "--no-ask-password",
            "show",
            unit,
            "--property=LoadState",
            "--value",
        ])
        .output()
        .expect("the real systemctl runs");
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// What:
/// `struct Observed` is what one control's run made:
/// its own answer,
/// its unit and the log of its calls.
struct Observed {
    /// The run's answer.
    result: Result<Output, RunError>,
    /// The unit the run made,
    /// from its own trace.
    unit: String,
    /// The file the recording program wrote every call into.
    log: PathBuf,
}

/// What:
/// `fn run_recording(directory: &Path, command: &mut Command, limit: Duration, stuck_scope: bool) -> Observed` runs one bounded
/// run whose route asks the recording program,
/// and answers the run,
/// its unit and its call log.
/// Why:
/// All four path controls differ only in the command and the hook,
/// so this is the one place a run is made.
/// `stuck_scope` makes the last look at the run's scope read the leftover state of issue 6578 (`pids.peak` 0),
/// which a real run of a program cannot produce:
/// a scope whose program ran has held a process,
/// and the control that runs without it shows what an ordinary run asks instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runRecording(dir: string, command: Command, limit: number, stuckScope: boolean): Observed
/// ```
fn run_recording(
    directory: &Path,
    command: &mut Command,
    limit: Duration,
    stuck_scope: bool,
) -> Observed {
    let (program, log) = recording_systemctl(directory);
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    use_route_settings_for_test(Some(settings));
    let _ = take_trace_for_test();
    if stuck_scope {
        arm(Point::RetireSeesStuckScope);
    }
    let result: Result<Output, RunError> = output_within(command, None, &bound(limit));
    let trace: Vec<String> = take_trace_for_test();
    use_route_settings_for_test(None);
    return Observed {
        result,
        unit: launched_scope(&trace),
        log,
    };
}

/// What:
/// `fn assert_stopped_and_gone(observed: &Observed)` requires that the run asked for its own unit to be stopped and that the manager
/// no longer loads it.
/// Why:
/// This is the shape of every path control's claim:
/// the request is the guard (a run without the retire step leaves no line in the log),
/// and the manager's answer is the effect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertStoppedAndGone(observed: Observed): void
/// ```
fn assert_stopped_and_gone(observed: &Observed) {
    let calls: String = fs::read_to_string(&observed.log).unwrap_or_default();
    let wanted: String = format!("--user --no-ask-password stop {}", observed.unit);
    let stops: Vec<&str> = stop_requests(&calls);
    assert_eq!(
        stops,
        vec![wanted.as_str()],
        "the run's stop requests were not the one request this step makes: {calls:?}"
    );
    assert!(
        !calls.contains("LoadState"),
        "the run asked whether the unit is still loaded as well as stopping it: {calls:?}"
    );
    assert!(
        wait_up_to(GONE_BOUND, &|| {
            return load_state(&observed.unit) == "not-found";
        }),
        "the run left {} loaded as {}",
        observed.unit,
        load_state(&observed.unit)
    );
}

/// What:
/// `fn stop_requests(log: &str) -> Vec<&str>` is the lines of a call log that are stop requests.
/// Why:
/// The route discovery asks the manager its own questions through the same program,
/// so a control that counts questions has to count the stop requests and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopRequests(log: string): string[]
/// ```
fn stop_requests(log: &str) -> Vec<&str> {
    return log
        .lines()
        .filter(|line| {
            return line.contains(" stop ");
        })
        .collect();
}

/// A normal end:
/// the program exits by itself,
/// and its scope unit is stopped and collected.
#[test]
fn a_run_that_ends_by_itself_retires_its_scope_unit() {
    let directory = scratch();
    let observed = run_recording(directory.path(), &mut Command::new("true"), RUN_BOUND, true);
    observed.result.as_ref().expect("the program runs");
    assert_stopped_and_gone(&observed);
}

/// A bound that passes:
/// the program outlives its bound and is killed,
/// and its scope unit is stopped and collected.
#[test]
fn a_run_whose_bound_passes_retires_its_scope_unit() {
    let directory = scratch();
    let observed = run_recording(
        directory.path(),
        Command::new("sleep").arg("600"),
        Duration::from_millis(300),
        true,
    );
    assert!(
        matches!(observed.result, Err(RunError::TimedOut { .. })),
        "the run did not end at its bound"
    );
    assert_stopped_and_gone(&observed);
}

/// A refusal after the program started:
/// the startup completion fails,
/// the run answers that failure,
/// and its scope unit is stopped and collected.
#[test]
fn a_refused_run_retires_its_scope_unit() {
    let directory = scratch();
    on_startup_for_test(Some(Rc::new(|| {
        return true;
    })));
    let observed = run_recording(directory.path(), &mut Command::new("true"), RUN_BOUND, true);
    on_startup_for_test(None);
    let refused: &RunError = observed
        .result
        .as_ref()
        .expect_err("the injected refusal ends the run");
    assert!(
        format!("{refused}").contains("injected failure to complete the startup"),
        "{refused}"
    );
    assert_stopped_and_gone(&observed);
}

/// A launcher that is gone:
/// the launcher is killed right after it is spawned,
/// and whatever the manager made of that start is stopped and collected.
#[test]
fn a_run_whose_launcher_is_gone_retires_its_scope_unit() {
    let directory = scratch();
    after_launch_for_test(Some(Rc::new(|pid: u32| {
        let Some(target) = rustix::process::Pid::from_raw(i32::try_from(pid).unwrap_or(0)) else {
            return;
        };
        let _ = rustix::process::kill_process(target, rustix::process::Signal::KILL);
    })));
    let observed = run_recording(
        directory.path(),
        Command::new("sleep").arg("600"),
        RUN_BOUND,
        true,
    );
    after_launch_for_test(None);
    assert_stopped_and_gone(&observed);
}

/// The control of the cost:
/// a run whose program really ran asks the manager nothing,
/// because the manager was told by the kernel that the scope emptied and collects the unit by itself.
/// This is what makes the retirement free on the ordinary path:
/// the look reads two files and starts no program.
#[test]
fn an_ordinary_run_asks_the_manager_nothing() {
    let directory = scratch();
    let observed = run_recording(
        directory.path(),
        &mut Command::new("true"),
        RUN_BOUND,
        false,
    );
    observed.result.as_ref().expect("the program runs");
    let calls: String = fs::read_to_string(&observed.log).unwrap_or_default();
    assert_eq!(
        stop_requests(&calls),
        Vec::<&str>::new(),
        "the run asked the manager to stop a unit the manager was already told about: {calls:?}"
    );
    assert!(
        !calls.contains("LoadState"),
        "the run asked whether the unit is still loaded: {calls:?}"
    );
    assert!(
        wait_up_to(GONE_BOUND, &|| {
            return load_state(&observed.unit) == "not-found";
        }),
        "the manager did not collect {} by itself: it reads {}",
        observed.unit,
        load_state(&observed.unit)
    );
}
