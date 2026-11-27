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
/// `use crate::control_group::retire::retire_unit;` imports the one request that frees a unit,
/// for the warning helper's own clean-up.
/// Why:
/// The helper's stand-in cannot free the unit it refused,
/// and a failed assertion in the parent must not leave it loaded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { retireUnit } from 'crate/control_group/retire';
/// ```
use crate::control_group::retire::retire_unit;
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

/// The control of a run whose stop could not show the scope empty:
/// the run is refused,
/// and its unit is still asked for and freed.
/// The stop is made to fail at once (a failpoint),
/// because a real stop that cannot show a scope empty takes its whole bound and a member that cannot be killed;
/// what this control reads is the retirement,
/// which has to happen whatever the stop answered.
#[test]
fn a_run_whose_stop_is_not_shown_empty_still_asks_for_its_unit() {
    let directory = scratch();
    arm(Point::ContainmentStopError);
    let observed = run_recording(directory.path(), &mut Command::new("true"), RUN_BOUND, true);

    assert!(
        !crate::failpoints::fires(Point::ContainmentStopError),
        "the stop never reached the failure this control arms, so the run was not refused by it"
    );
    assert!(
        matches!(observed.result, Err(RunError::NotQuiescent { .. })),
        "the injected stop failure did not refuse the run"
    );
    assert_stopped_and_gone(&observed);
}

/// The environment variable that tells the warning helper process which run to make.
const WARNING_SCENARIO: &str = "CARGO_SHADOW_RETIRE_WARNING_SCENARIO";
/// The environment variable that gives the warning helper process the directory its stand-ins live in.
const WARNING_DIRECTORY: &str = "CARGO_SHADOW_RETIRE_WARNING_DIRECTORY";

/// What:
/// `fn failing_systemctl(directory: &Path) -> PathBuf` writes a program that logs its arguments,
/// fails a stop request with one line of its own,
/// and runs the real `systemctl` for every other request,
/// and answers the program.
/// Why:
/// The warning this control reads is printed only when the manager refuses the stop,
/// and the same program has to answer the route discovery's own questions from the real manager,
/// or the run would make no scope to leave.
/// It is a stand-in:
/// no claim about a real manager rests on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failingSystemctl(dir: string): string
/// ```
fn failing_systemctl(directory: &Path) -> PathBuf {
    let program: PathBuf = directory.join("failing-systemctl");
    let log: PathBuf = directory.join("failing-systemctl-calls");
    let text: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nif [ \"$3\" = stop ]; then\n  echo 'the stand-in refuses the stop' >&2\n  exit 1\nfi\nexec '{real}' \"$@\"\n",
        log = log.display(),
        real = systemctl_path()
    );
    fs::write(&program, text).expect("write the stand-in");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return program;
}

/// The helper process of the warning control:
/// it makes one run under the scenario its environment names and exits.
/// The warning a run prints goes to the real standard error of the process that made the run,
/// so a fresh process is the only way a test can read it.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_run_whose_retire_is_refused_prints_the_warning_on_its_own_standard_error"]
fn retire_warning_fixture() {
    let (Ok(scenario), Ok(scratch)) = (
        std::env::var(WARNING_SCENARIO),
        std::env::var(WARNING_DIRECTORY),
    ) else {
        return;
    };
    let directory: PathBuf = PathBuf::from(scratch);
    let program: PathBuf = failing_systemctl(&directory);
    let settings: RouteSettings = RouteSettings {
        systemctl: program.display().to_string(),
        ..RouteSettings::default()
    };
    use_route_settings_for_test(Some(settings));
    let _ = take_trace_for_test();
    let result: Result<Output, RunError> = if scenario == "member" {
        // The stop is made to fail before it kills anything, so the program's grandchild is still in the scope when the
        // retirement reads it.
        arm(Point::ContainmentStopError);
        output_within(
            Command::new("sh").args(["-c", "sleep 30 &"]),
            None,
            &bound(RUN_BOUND),
        )
    } else {
        // The look answers the state of the stuck scope of issue 6578, which a run of a real program cannot leave.
        arm(Point::RetireSeesStuckScope);
        output_within(&mut Command::new("true"), None, &bound(RUN_BOUND))
    };
    use_route_settings_for_test(None);
    // What:     `let unit: String = launched_scope(&take_trace_for_test())` is the unit the run made, read from the run's own
    //           trace; the retirement's own request goes through the stand-in, so the unit is freed here with the real manager.
    // Why:      A failed assertion in the parent must not leave a unit behind, and the stand-in cannot free it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const unit = launchedScope(takeTrace())
    // ```
    let unit: String = launched_scope(&take_trace_for_test());
    let _ = retire_unit(
        "systemctl",
        &unit,
        crate::control_group::retire::RETIRE_BOUND,
    );
    assert!(
        result.is_err() || scenario == "refused",
        "the member run was not refused, so its member would have been killed by the stop: {result:?}"
    );
}

/// The control of the words a run prints:
/// the warning a refused retirement leaves on the run's own standard error,
/// read from a fresh process that makes the run,
/// for each of the two states a run can leave a unit in.
/// The trace of the other controls shows which warning a run selects;
/// this shows that it is printed,
/// so a change that removed the printing and kept the answer would fail here and pass there.
#[test]
fn a_run_whose_retire_is_refused_prints_the_warning_on_its_own_standard_error() {
    let directory = scratch();
    let scenarios: [(&str, &str, &str); 2] = [
        (
            "refused",
            "may still be loaded",
            "the stand-in refuses the stop",
        ),
        ("member", "was left loaded", "still has a member"),
    ];
    for (scenario, state, detail) in scenarios {
        let output = Command::new(std::env::current_exe().expect("this test executable"))
            .args([
                "--ignored",
                "--exact",
                "bounded_run_end::retire_tests::retire_warning_fixture",
                "--nocapture",
            ])
            .env(WARNING_SCENARIO, scenario)
            .env(WARNING_DIRECTORY, directory.path())
            .output()
            .expect("the warning helper");

        let stderr: String = String::from_utf8_lossy(&output.stderr).into_owned();
        assert!(
            output.status.success(),
            "{scenario}: the helper failed: {stderr}"
        );
        let printed: Vec<&str> = stderr
            .lines()
            .filter(|line| {
                return line.starts_with("cargo-shadow: warning: the scope unit ");
            })
            .collect();
        assert_eq!(
            printed.len(),
            1,
            "{scenario}: the warning was not printed exactly once on standard error: {stderr}"
        );
        assert!(
            printed[0].contains(state) && printed[0].contains(detail),
            "{scenario}: the printed warning does not say what was read: {}",
            printed[0]
        );
        if scenario == "refused" {
            assert!(
                !printed[0].contains("was left loaded"),
                "the run says the unit was left loaded, where the manager was asked and answered: {}",
                printed[0]
            );
        }
        if scenario == "member" {
            assert!(
                printed[0].contains(
                    "a member that joins after this look is killed by that stop with nothing printed"
                ),
                "the printed warning does not name the window after the look: {}",
                printed[0]
            );
        }
    }
}
