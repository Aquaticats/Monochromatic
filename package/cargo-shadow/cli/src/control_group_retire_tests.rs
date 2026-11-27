//! What:
//! Controls for the end of a run's scope unit (issue 6578):
//! the last look at the scope,
//! the one question to the manager it may call for,
//! and what the run says when it leaves a unit alone.
//! Why:
//! The leftover unit is rare in nature (a process that is already exiting when the manager writes its id into the scope,
//! 12 of 11360 starts on this host on 2026-11-27),
//! so the controls do not wait for that race:
//! they plant the state it leaves,
//! deterministically,
//! by adopting a zombie process into a scope of their own,
//! which the user manager accepts as attached without moving it.
//! An ordinary run asks the manager nothing,
//! and the control that says so is a scope whose program really ran:
//! the manager ends that unit on the empty change the kernel sent it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('control group retire', () => { it('retires a loaded empty scope', ...) })
//! ```

/// What:
/// `use std::fs;` imports the file system calls that read a process's `stat` file and write the stand-ins.
/// Why:
/// The planted process must be seen as a zombie before the manager is asked to adopt it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports the trait that sets a file's mode.
/// Why:
/// A stand-in program has to be executable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { chmodSync } from 'node:fs';
/// ```
use std::os::unix::fs::PermissionsExt;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The planted scope's directory is owned by the guard,
/// and the helpers that plant and record are lent a directory.
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
/// `use std::process::{Child, Command};` imports a child handle and the command builder.
/// Why:
/// The planted process is a real child this test never reaps until the guard ends,
/// which is what keeps it a zombie at the moment of adoption,
/// and the scope that holds a member starts a real program.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess, spawn } from 'node:child_process';
/// ```
use std::process::{
    Child,
    Command,
};
/// What:
/// `use std::sync::atomic::{AtomicUsize, Ordering};` imports a counter that the parallel tests may share safely.
/// Why:
/// Every planted unit needs its own name,
/// and two tests run at the same time on this suite.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let plantCount = 0
/// ```
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
/// What:
/// `use std::time::{Duration, Instant};` imports a span of time and a point on the clock.
/// Why:
/// The control of the one question's bound measures how long the step took,
/// and every wait here has a bound of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Date.now() and a number of milliseconds
/// ```
use std::time::{
    Duration,
    Instant,
};

/// The shared fixtures of a delegated scope (the route,
/// the waits and the not-run line).
use crate::cgroup_test_support::{
    NOT_RUN_PREFIX,
    delegated_route,
    start_in_scope,
    this_test_needs_a_delegated_scope,
    wait_up_to,
};
/// What:
/// `use crate::control_group::retire::{KILL_REAP_GRACE, RETIRE_BOUND, retire, retire_unit};` imports the step under test, its
/// bound, and the grace a killed request's reap may take.
/// Why:
/// Every control here exercises the same step the run takes at its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RETIRE_BOUND, retire, retireUnit } from 'crate/control_group/retire';
/// ```
use crate::control_group::retire::{
    RETIRE_BOUND,
    retire,
    retire_unit,
};
/// What:
/// `use crate::control_group::{ControlGroup, Membership, Record, Route, View, namespace_id, query, view_of};` imports the scope
/// the step acts on,
/// the record of a scope and the verdict on it,
/// and the reads that build both.
/// Why:
/// A control that asks the step about a scope names that scope itself,
/// and the recovery control builds the record a journal would hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup, Membership, Record, Route, View, namespaceId, query, viewOf } from 'crate/control_group';
/// ```
use crate::control_group::{
    ControlGroup,
    Membership,
    Record,
    Route,
    namespace_id,
    query,
    view_of,
};
/// What:
/// `use crate::control_group_process_identity::parse_stat;` imports the reader of a `stat` file's state letter.
/// Why:
/// The planted process must be a zombie (`Z`) before it is adopted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseStat } from 'crate/control_group_process_identity';
/// ```
use crate::control_group_process_identity::parse_stat;
/// What:
/// `use crate::control_group_route_child_runs::KILL_REAP_GRACE;` imports how long a killed short program is given to be reaped.
/// Why:
/// The control of the one question's bound counts that grace as part of the answer it allows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { KILL_REAP_GRACE } from 'crate/control_group_route_child_runs';
/// ```
use crate::control_group_route_child_runs::KILL_REAP_GRACE;
/// What:
/// `use crate::failpoints::{Point, arm};` imports the test-only switches and how one is armed.
/// Why:
/// A scope's absence cannot be made unverifiable from outside,
/// so a failure is planted instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Point, arm } from 'crate/failpoints';
/// ```
use crate::failpoints::{
    Point,
    arm,
};

/// How long a planted unit is given to appear in the manager.
const PLANT_BOUND: Duration = Duration::from_secs(10);
/// How long a scope whose program ended is given to become empty.
const EMPTY_BOUND: Duration = Duration::from_secs(10);

/// The number the last planted unit was given,
/// so each name is unique across the tests that run at the same time.
static PLANTS: AtomicUsize = AtomicUsize::new(0);

/// What:
/// `fn not_run(reason: &str) -> !` fails the test with the line the suite's task reads as "this test did not run",
/// naming the test and the reason.
/// Why:
/// A host whose manager cannot adopt a zombie cannot run these controls,
/// and a test that returned quietly would be counted as passed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function notRun(reason: string): never { throw new Error(reason) }
/// ```
fn not_run(reason: &str) -> ! {
    // What:     `std::thread::current().name()` gives the name the test runner gave this thread; `map(str::to_string)` turns the
    //           borrowed name into an owned `String` (the sibling of `&str`), and `unwrap_or_else` supplies a name when there is none.
    // Why:      The task leaves a test out by the exact name it printed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const test = currentTestName() ?? '<a test the runner did not name>'
    // ```
    let test: String = std::thread::current()
        .name()
        .map(str::to_string)
        .unwrap_or_else(|| {
            return "<a test the runner did not name>".to_string();
        });
    panic!(
        "{NOT_RUN_PREFIX}{test}]: this host cannot adopt a zombie into a scope ({reason}); it runs where a user manager answers on a cgroup v2 mount"
    );
}

/// What:
/// `fn show_property(unit: &str, property: &str) -> String` asks the manager for one property of a unit,
/// trimmed.
/// Why:
/// The controls judge the unit's real state,
/// not a stand-in's idea of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showProperty(unit: string, property: string): string
/// ```
fn show_property(
    unit: &str,
    property: &str,
) -> String {
    let output = Command::new("systemctl")
        .args([
            "--user",
            "--no-ask-password",
            "show",
            unit,
            &format!("--property={property}"),
            "--value",
        ])
        .output()
        .unwrap_or_else(|error| {
            not_run(&format!("systemctl could not be started ({error})"));
        });
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// What:
/// `fn pids_peak(path: &Path) -> String` is the `pids.peak` count of a control group.
/// Why:
/// The gate rests on that count,
/// so a control that judges it reads it from the kernel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pidsPeak(path: string): string
/// ```
fn pids_peak(path: &Path) -> String {
    return fs::read_to_string(path.join("pids.peak"))
        .map(|text| {
            return text.trim().to_string();
        })
        .unwrap_or_else(|error| {
            return format!("(unreadable: {error})");
        });
}

/// What:
/// `fn populated(path: &Path) -> String` is the `populated` line of a control group's `cgroup.events`.
/// Why:
/// The planted unit is only the leftover this issue is about when the kernel says no thread is in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function populated(path: string): string
/// ```
fn populated(path: &Path) -> String {
    let text: String = fs::read_to_string(path.join("cgroup.events")).unwrap_or_default();
    return text
        .lines()
        .find(|line| {
            return line.starts_with("populated ");
        })
        .unwrap_or("populated ?")
        .trim()
        .to_string();
}

/// What:
/// `fn group_of(route: &Route, unit: &str) -> ControlGroup` is the scope a run of this test would have made under `unit`.
/// Why:
/// The step under test is given a scope, not a name,
/// and the fields it reads are the ones the planner fills in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function groupOf(route: Route, unit: string): ControlGroup
/// ```
fn group_of(
    route: &Route,
    unit: &str,
) -> ControlGroup {
    let Some(name) = unit.strip_suffix(".scope") else {
        panic!("{unit} is not a scope unit name");
    };
    return ControlGroup {
        name: name.to_string(),
        directory: route.slice_dir.join(unit),
        parent: route.slice_dir.clone(),
        parent_view: view_of(&route.slice_dir).expect("the slice's view"),
        namespace: namespace_id().expect("this cgroup namespace"),
    };
}

/// What:
/// `fn recording_systemctl(directory: &Path) -> (String, PathBuf)` writes a program that appends its arguments to a log and
/// answers that it cannot stop anything,
/// and answers the program and the log.
/// Why:
/// A control that requires the manager to be asked nothing cannot see that from the manager's state;
/// it sees it from a log that stays empty.
/// The program is a stand-in and no claim about a real manager rests on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recordingSystemctl(dir: string): [string, string]
/// ```
fn recording_systemctl(directory: &Path) -> (String, PathBuf) {
    let program: PathBuf = directory.join("recording-systemctl");
    let log: PathBuf = directory.join("systemctl-calls");
    let text: String = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nexit 1\n",
        log = log.display()
    );
    fs::write(&program, text).expect("write the stand-in");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("make it executable");
    return (program.display().to_string(), log);
}

/// What:
/// `fn calls(log: &Path) -> String` is what the recording program logged.
/// Why:
/// An empty log is the claim that no question was asked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function calls(log: string): string
/// ```
fn calls(log: &Path) -> String {
    return fs::read_to_string(log).unwrap_or_default();
}

/// A scope the test planted:
/// the manager adopted a zombie,
/// so the unit is `active (running)` and its control group is empty.
pub(crate) struct PlantedScope {
    /// The unit name the manager knows.
    pub(crate) unit: String,
    /// The scope's directory in the cgroup file system.
    pub(crate) path: PathBuf,
    /// The process that was adopted and is still this test's child.
    zombie: Child,
}

/// How the guard cleans up.
impl Drop for PlantedScope {
    /// What:
    /// `fn drop(&mut self)` asks the manager to stop the planted unit and reaps the zombie.
    /// Why:
    /// A failed assertion must leave neither a unit nor a process behind,
    /// and it stops only the unit its own test planted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void { stopPlantedUnit(); reapZombie() }
    /// ```
    fn drop(&mut self) {
        let _ = retire_unit("systemctl", &self.unit, RETIRE_BOUND);
        let _ = self.zombie.wait();
    }
}

/// What:
/// `pub(crate) fn plant_loaded_empty_scope(route: &Route) -> PlantedScope` adopts a zombie process into a new scope in `route`'s
/// slice through the user manager,
/// and returns once the manager shows the unit `active` with `populated 0`.
/// Why:
/// That is exactly the state a run leaves when the manager attaches a process that is already exiting:
/// the real leftover,
/// made on purpose and in a unit of the test's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function plantLoadedEmptyScope(route: Route): PlantedScope
/// ```
pub(crate) fn plant_loaded_empty_scope(route: &Route) -> PlantedScope {
    let count: usize = PLANTS.fetch_add(1, Ordering::SeqCst);
    let unit: String = format!("cargo-shadow-test-{}-{count}.scope", std::process::id());
    let zombie: Child = Command::new("true").spawn().unwrap_or_else(|error| {
        not_run(&format!(
            "a process to adopt could not be started ({error})"
        ));
    });
    let zombie_pid: i32 = i32::try_from(zombie.id()).unwrap_or(0);
    let stat_path: PathBuf = PathBuf::from(format!("/proc/{zombie_pid}/stat"));
    // What:     `state_of` is a closure that returns the process's state letter, or `?` when the file cannot be read. `&stat_path`
    //           lends the path to the read, and `.unwrap_or('?')` substitutes the question mark for a failed parse.
    // Why:      The wait asks the same question until the process shows `Z`, so it is written once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stateOf = (): string => { try { return readStat(statePath) } catch { return '?' } }
    // ```
    let state_of = || -> char {
        let text: String = fs::read_to_string(&stat_path).unwrap_or_default();
        return parse_stat(zombie_pid, &text)
            .map(|found| {
                return found.1;
            })
            .unwrap_or('?');
    };
    assert!(
        wait_up_to(PLANT_BOUND, &|| {
            return state_of() == 'Z';
        }),
        "the process to adopt was not a zombie in time"
    );
    // The property count is three (`PIDs`, `Description`, `CollectMode`), the signature is the manager's own
    // (`busctl introspect org.freedesktop.systemd1 /org/freedesktop/systemd1`).
    let adopted = Command::new("busctl")
        .args([
            "--user",
            "call",
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "StartTransientUnit",
            "ssa(sv)a(sa(sv))",
            &unit,
            "fail",
            "3",
            "PIDs",
            "au",
            "1",
            &zombie_pid.to_string(),
            "Description",
            "s",
            "cargo-shadow test",
            "CollectMode",
            "s",
            "inactive-or-failed",
            // The empty auxiliary-units array of the signature (its last argument).
            "0",
        ])
        .output()
        .unwrap_or_else(|error| {
            not_run(&format!("busctl could not be started ({error})"));
        });
    if !adopted.status.success() {
        not_run(&format!(
            "the manager refused the adoption ({}; {})",
            adopted.status,
            String::from_utf8_lossy(&adopted.stderr).trim()
        ));
    }
    let path: PathBuf = route.slice_dir.join(&unit);
    assert!(
        wait_up_to(PLANT_BOUND, &|| {
            return show_property(&unit, "ActiveState") == "active"
                && populated(&path) == "populated 0";
        }),
        "the planted unit did not become an active empty scope ({}; {})",
        show_property(&unit, "ActiveState"),
        populated(&path)
    );
    return PlantedScope {
        unit,
        path,
        zombie,
    };
}

/// The issue's first control:
/// a unit the manager left active with an empty control group that no process was ever counted in is stopped by the retire step,
/// and the manager then reports it `not-found` (collected).
/// The kernel's `pids.peak` of the planted group is 0,
/// which is what the step reads to know the manager was never told this group emptied.
#[test]
fn a_scope_that_never_held_a_process_is_retired() {
    let route: Route = delegated_route();
    let planted = plant_loaded_empty_scope(&route);
    let group: ControlGroup = group_of(&route, &planted.unit);
    assert_eq!(
        show_property(&planted.unit, "LoadState"),
        "loaded",
        "the planted unit should be loaded before the retire"
    );
    assert_eq!(
        pids_peak(&planted.path),
        "0",
        "the planted group must have held no process, or it is not the leftover this step exists for"
    );

    let word: Option<String> = retire(&group, &route.systemctl, RETIRE_BOUND);

    assert_eq!(word, None, "a retired unit is not a warning");
    assert!(
        wait_up_to(PLANT_BOUND, &|| {
            return show_property(&planted.unit, "LoadState") == "not-found";
        }),
        "the retire left the unit loaded as {}",
        show_property(&planted.unit, "LoadState")
    );
}

/// A second control:
/// a scope whose program really ran is left to the manager,
/// which ends the unit on the empty change the kernel sent it,
/// so the step asks the manager nothing.
#[test]
fn a_scope_that_held_a_process_is_left_to_the_manager() {
    this_test_needs_a_delegated_scope();
    let directory = crate::test_scratch::scratch();
    let (systemctl, log) = recording_systemctl(directory.path());
    let scope = start_in_scope(Command::new("sh").args(["-c", "sleep 0.1"]));
    let unit: String = scope.group.unit();
    assert!(
        wait_up_to(EMPTY_BOUND, &|| {
            return scope
                .group
                .populated()
                .map(|populated| {
                    return !populated;
                })
                .unwrap_or(true);
        }),
        "the scope did not become empty"
    );
    assert_ne!(
        pids_peak(scope.group.directory()),
        "0",
        "the scope held a process, so the manager was told it emptied"
    );

    let word: Option<String> = retire(&scope.group, &systemctl, RETIRE_BOUND);

    assert_eq!(word, None, "nothing is said about a unit left to the manager");
    assert_eq!(
        calls(&log),
        "",
        "the step asked the manager about a unit the manager was already told about"
    );
    assert!(
        wait_up_to(EMPTY_BOUND, &|| {
            return show_property(&unit, "LoadState") == "not-found";
        }),
        "the manager did not collect {}: it reads {}",
        unit,
        show_property(&unit, "LoadState")
    );
}

/// A third control:
/// a scope that still holds a member keeps its unit,
/// and the run says why.
#[test]
fn a_scope_that_holds_a_member_keeps_its_unit_and_the_run_says_so() {
    this_test_needs_a_delegated_scope();
    let directory = crate::test_scratch::scratch();
    let (systemctl, log) = recording_systemctl(directory.path());
    let scope = start_in_scope(Command::new("sleep").arg("30"));
    let unit: String = scope.group.unit();

    let word: Option<String> = retire(&scope.group, &systemctl, RETIRE_BOUND);

    let said: String = word.expect("a unit left loaded is reported");
    assert!(said.contains(&unit), "{said}");
    assert!(said.contains("still has a member"), "{said}");
    assert_eq!(
        calls(&log),
        "",
        "the step asked the manager to stop a scope that holds a member"
    );
    assert_eq!(
        show_property(&unit, "ActiveState"),
        "active",
        "the unit of a scope with a member must stay loaded"
    );
}

/// A fourth control:
/// an absence that cannot be verified is not read as emptiness,
/// so the unit is kept and the run says so.
/// The absence is planted (a failpoint) because no run can make this process see another cgroup view.
#[test]
fn an_absence_that_cannot_be_verified_keeps_the_unit_and_the_run_says_so() {
    let route: Route = delegated_route();
    let directory = crate::test_scratch::scratch();
    let (systemctl, log) = recording_systemctl(directory.path());
    let absent: String = format!("cargo-shadow-test-{}-absent.scope", std::process::id());
    let group: ControlGroup = group_of(&route, &absent);
    arm(Point::AbsenceUnverifiable);

    let word: Option<String> = retire(&group, &systemctl, RETIRE_BOUND);

    assert!(
        !crate::failpoints::fires(Point::AbsenceUnverifiable),
        "the check of the scope's absence was never reached"
    );
    let said: String = word.expect("a unit kept because its absence was not shown is reported");
    assert!(said.contains(&absent), "{said}");
    assert!(said.contains("absence could not be read as emptiness"), "{said}");
    assert_eq!(
        calls(&log),
        "",
        "the step asked the manager to stop a unit whose emptiness it had not shown"
    );
}

/// A fifth control:
/// a scope whose `populated` line cannot be read is reported,
/// and its unit is kept:
/// nothing was shown about its members.
#[test]
fn a_populated_read_that_fails_is_reported() {
    let route: Route = delegated_route();
    let directory = crate::test_scratch::scratch();
    let (systemctl, log) = recording_systemctl(directory.path());
    // A path that is a file, not a group directory: its `cgroup.events` cannot be read, and the error is neither a missing
    // group nor a removed one.
    let not_a_group: PathBuf = directory.path().join("not-a-group");
    fs::write(&not_a_group, "").expect("write the file the look is pointed at");
    let absent: String = format!("cargo-shadow-test-{}-unreadable.scope", std::process::id());
    let mut group: ControlGroup = group_of(&route, &absent);
    group.directory = not_a_group;

    let word: Option<String> = retire(&group, &systemctl, RETIRE_BOUND);

    let said: String = word.expect("a scope that cannot be read is reported");
    assert!(said.contains("`populated` line could not be read"), "{said}");
    assert_eq!(calls(&log), "", "an unreadable scope is not stopped");
}

/// A sixth control:
/// the one question the step asks is bounded,
/// and the program it killed is reaped before the answer is built.
/// The `systemctl` here is a stand-in that never answers and not a real one;
/// no claim about a real manager rests on it.
#[test]
fn the_one_question_is_bounded_and_its_killed_program_is_reaped() {
    let directory = crate::test_scratch::scratch();
    let stand_in: PathBuf = directory.path().join("silent-systemctl");
    let pid_file: PathBuf = directory.path().join("pid");
    fs::write(
        &stand_in,
        format!(
            "#!/bin/sh\necho $$ > '{pid}'\nexec sleep 30\n",
            pid = pid_file.display()
        ),
    )
    .expect("write the stand-in");
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755)).expect("make it executable");
    let bound: Duration = Duration::from_millis(200);
    let began: Instant = Instant::now();

    let refused: String = retire_unit(
        &stand_in.display().to_string(),
        "cargo-shadow-test-silent.scope",
        bound,
    )
    .expect_err("a request that does not answer is a warning");

    let elapsed: Duration = began.elapsed();
    assert!(
        elapsed >= bound,
        "the question answered after {elapsed:?}, before its bound of {bound:?}"
    );
    assert!(
        elapsed < bound + KILL_REAP_GRACE + Duration::from_millis(1000),
        "the question answered after {elapsed:?}: more than one bound and its reap grace of {KILL_REAP_GRACE:?}"
    );
    assert!(
        refused.contains("--no-ask-password"),
        "the warning does not name the command that ran: {refused}"
    );
    assert!(
        !refused.contains("could not be reaped"),
        "the killed stand-in was reaped, so the warning must not say otherwise: {refused}"
    );
    let pid: String = fs::read_to_string(&pid_file)
        .expect("the stand-in recorded its process id")
        .trim()
        .to_string();
    assert!(
        !PathBuf::from(format!("/proc/{pid}")).exists(),
        "the killed stand-in ({pid}) is still a process of this account, so it was not reaped"
    );
}

/// A seventh control:
/// a manager that refuses the stop is reported with the command that ran,
/// so the warning a person reads names the unit and can be pasted to see the same refusal.
/// The `systemctl` here is a stand-in that answers no program's real output;
/// no claim about a real manager rests on it.
#[test]
fn a_retire_the_manager_refuses_is_reported_with_the_command_that_ran() {
    let directory = crate::test_scratch::scratch();
    let stand_in: PathBuf = directory.path().join("refusing-systemctl");
    fs::write(
        &stand_in,
        "#!/bin/sh\necho 'the stand-in refuses' >&2\nexit 1\n",
    )
    .expect("write the stand-in");
    fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755))
        .expect("make it executable");
    let reason: String = retire_unit(
        &stand_in.display().to_string(),
        "cargo-shadow-test-x.scope",
        RETIRE_BOUND,
    )
    .expect_err("a refused retire is reported");
    assert!(reason.contains("cargo-shadow-test-x.scope"), "{reason}");
    assert!(
        reason.contains("--user --no-ask-password stop cargo-shadow-test-x.scope"),
        "the warning does not name the command that ran: {reason}"
    );
    assert!(reason.contains("the stand-in refuses"), "{reason}");
}

/// An eighth control:
/// a unit the manager already collected is not a refusal,
/// because the stop of a unit it does not load answers exit status 5.
#[test]
fn retiring_a_unit_the_manager_already_collected_is_not_a_refusal() {
    let route: Route = delegated_route();
    let absent: String = format!("cargo-shadow-test-{}-absent.scope", std::process::id());
    assert_eq!(show_property(&absent, "LoadState"), "not-found");
    retire_unit(&route.systemctl, &absent, RETIRE_BOUND).expect("an absent unit is not a refusal");
}

/// The issue's last control:
/// a recovery that meets a loaded unit whose control group is empty reads the scope as `Gone`,
/// because it asks the kernel's `populated` line and not the manager's active state.
#[test]
fn a_recovery_reads_a_loaded_empty_scope_as_gone() {
    let route: Route = delegated_route();
    let planted = plant_loaded_empty_scope(&route);
    let record: Record = Record::Scope {
        namespace: namespace_id().expect("this cgroup namespace"),
        parent_view: view_of(&route.slice_dir).expect("the slice's view"),
        path: planted.path.clone(),
    };
    let Record::Scope {
        namespace,
        parent_view,
        path,
    } = record
    else {
        panic!("the record of a scope was read as another kind");
    };

    let verdict: Membership = query(namespace, parent_view, &path);

    assert_eq!(
        show_property(&planted.unit, "ActiveState"),
        "active",
        "the unit should still be active while the recovery reads it"
    );
    assert_eq!(
        verdict,
        Membership::Gone,
        "a loaded but empty scope is gone for a recovery"
    );
}
