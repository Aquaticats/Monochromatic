//! What:
//! Controls for the end of a run's scope unit (issue 6578):
//! a unit the manager left `active (running)` with an empty control group is stopped,
//! a unit it already collected is not a refusal,
//! a refused stop is reported,
//! and a recovery reads a loaded but empty scope as gone.
//! Why:
//! The leftover unit is rare in nature (a process that is already exiting when the manager writes its id into the scope,
//! 12 of 11360 starts on this host on 2026-11-27),
//! so the controls do not wait for that race:
//! they plant the state it leaves,
//! deterministically,
//! by adopting a zombie process into a scope of their own,
//! which the user manager accepts as attached without moving it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('control group retire', () => { it('retires a loaded empty scope', ...) })
//! ```

/// What:
/// `use std::fs;` imports the file system calls that read a process's `stat` file.
/// Why:
/// The planted process must be seen as a zombie before the manager is asked to adopt it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// The planted scope's directory is owned by the guard,
/// and the helper that writes a stand-in is lent a directory.
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
/// which is what keeps it a zombie at the moment of adoption.
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

/// The shared fixtures of a delegated scope (the route,
/// the waits and the not-run line).
use crate::cgroup_test_support::{
    NOT_RUN_PREFIX,
    delegated_route,
    wait_up_to,
};
/// What:
/// `use crate::control_group::retire::{RETIRE_BOUND, retire_unit};` imports the step under test and its bound.
/// Why:
/// Every control here exercises the same step the run takes at its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RETIRE_BOUND, retireUnit } from 'crate/control_group/retire';
/// ```
use crate::control_group::retire::{
    RETIRE_BOUND,
    retire_unit,
};
/// What:
/// `use crate::control_group::{Membership, Record, Route, View, namespace_id, query, view_of};` imports the record of a scope,
/// the verdict on it and the reads that build both.
/// Why:
/// The recovery control builds the record a journal would hold and asks the same question a recovery asks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Membership, Record, Route, View, namespaceId, query, viewOf } from 'crate/control_group';
/// ```
use crate::control_group::{
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

/// How long a planted unit is given to appear in the manager.
const PLANT_BOUND: std::time::Duration = std::time::Duration::from_secs(10);

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
/// a unit the manager left active with an empty control group is stopped by the retire step,
/// and the manager then reports it `not-found` (collected).
#[test]
fn retiring_a_loaded_empty_scope_unit_removes_it() {
    let route: Route = delegated_route();
    let planted = plant_loaded_empty_scope(&route);
    assert_eq!(
        show_property(&planted.unit, "LoadState"),
        "loaded",
        "the planted unit should be loaded before the retire"
    );

    retire_unit(&route.systemctl, &planted.unit, RETIRE_BOUND).expect("the planted unit retires");

    assert!(
        wait_up_to(PLANT_BOUND, &|| {
            return show_property(&planted.unit, "LoadState") == "not-found";
        }),
        "the retire left the unit loaded as {}",
        show_property(&planted.unit, "LoadState")
    );
}

/// A second control:
/// a unit the manager already collected is not a refusal,
/// because the stop of an unknown unit fails and the state question
/// answers `not-found`.
#[test]
fn retiring_a_unit_the_manager_already_collected_is_not_a_refusal() {
    let route: Route = delegated_route();
    let absent: String = format!("cargo-shadow-test-{}-absent.scope", std::process::id());
    assert_eq!(show_property(&absent, "LoadState"), "not-found");
    retire_unit(&route.systemctl, &absent, RETIRE_BOUND).expect("an absent unit is not a refusal");
}

/// A third control:
/// a manager that refuses the stop and still loads the unit is reported with both facts,
/// so the warning a person reads names the unit's state.
/// The `systemctl` here is a stand-in that answers no program's real output;
/// no claim about a real manager rests on it.
#[test]
fn a_retire_the_manager_refuses_is_reported() {
    let directory = crate::test_scratch::scratch();
    let stand_in: PathBuf = directory.path().join("refusing-systemctl");
    fs::write(
        &stand_in,
        "#!/bin/sh\necho 'the stand-in refuses' >&2\nexit 1\n",
    )
    .expect("write the stand-in");
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&stand_in, fs::Permissions::from_mode(0o755))
            .expect("make it executable");
    }
    let reason: String = retire_unit(
        &stand_in.display().to_string(),
        "cargo-shadow-test-x.scope",
        RETIRE_BOUND,
    )
    .expect_err("a refused retire is reported");
    assert!(reason.contains("cargo-shadow-test-x.scope"), "{reason}");
    assert!(reason.contains("the stand-in refuses"), "{reason}");
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
