//! What:
//!  The control that a build shaped like the release,
//!  made without the `test-only-switches` feature,
//!  reads none of the crate's test-only variables:
//!  every value a control would arm is set,
//!  and every switch's reader still answers what a released run answers.
//! Why:
//!  A unit's `Environment=` line can set a test-only variable,
//!  and one that arms a fault can kill or stall the commit service,
//!  so the release must hold no read of them at all
//!  (`doc/decision/cli-git-test-only-switches-compiled-out-of-release.md`).
//!  A build with the feature on cannot show that,
//!  because it reads every one of them by design,
//!  so this control is compiled only where the feature is off and does its work in a child of the test binary,
//!  as `start_probe/probe_deadline_tests.rs` asks its own question of a build without its feature.
//!  A plant that removes a gating `cfg` makes the child's answer differ here,
//!  naming the switch that was read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! spawnSync(binary, ['--exact', name], { env: { ...everySwitchSet } }); // the child asserts each shipped answer
//! ```
#![cfg(not(feature = "test-only-switches"))]

use crate::add_route_plan::{HOLD_STEP_VARIABLE, HOLD_UNTIL_VARIABLE, STOP_AFTER_VARIABLE, Stops};
use crate::account_companion::companion_deadline::{COMPANION_DEADLINE, TEST_DEADLINE_VARIABLE, the_deadline};
use crate::account_companion::companion_test_switch::{Switch, TEST_SWITCH_VARIABLE as COMPANION_SWITCH_VARIABLE, from_environment as the_companion_switch_from_environment};
use crate::journald_native::TEST_SOCKET_VARIABLE;
use crate::lazy_runtime::{SLOW_BUILD_VARIABLE, the_pause_a_control_asks_for};
use crate::lock_birth_times::{BIRTH_TIME_VARIABLE, BirthTimes, times_in_force};
use crate::start_probe::probe_test_switch::{
    KEEP_DESCRIPTOR_VARIABLE, TEST_SWITCH_VARIABLE as PROBE_SWITCH_VARIABLE, keep_descriptor_from_environment,
    test_switch_from_environment,
};
use crate::start_probe::TestSwitch;
use crate::push_deadline_and_patience_settings::{CONNECTED_WINDOW_VARIABLE, connected_window};
use crate::runtime_probe::{RUNTIME_PROBE_VARIABLE, is_requested};
use crate::service_fault::{FAULT_VARIABLE, FIRST_BATCH_VARIABLE, QUEUED_DIRECTORY_VARIABLE, pinned_first_batch, queued, reach};
use crate::service_warm_request_control_window::{PARK_VARIABLE, the_answer_waits_where_a_control_says};
use crate::transaction_phase::{PHASE_SIGNAL_VARIABLE, Phase, reach_transaction_phase};
use crate::wait_ceilings::CONNECTED_PUSH_QUIET_WINDOW;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The variable that tells the child it is the child.
/// Why:
///  The control runs the same test name in a child,
///  and the child must do the child's one body rather than start another child.
const THE_CHILD: &str = "CLI_GIT_TEST_ONLY_SWITCH_CHILD";

/// This control's own test path,
/// which the child is started on.
const THE_TEST: &str = "testing::no_switch_is_read_without_its_feature_tests::a_build_without_the_feature_reads_no_test_only_switch";

/// The fault point the fault-point switch names,
/// which a build with the feature ends the process at.
const A_FAULT_POINT: &str = "after-intent";

/// The phase and the action the phase-marker switch names,
/// which a build with the feature ends the process at.
const A_KILLED_PHASE: &str = "merge-intent:kill";

/// The name of the parking point inside the folder this control's parent gives it,
/// where a build with the feature leaves its `.answered` file.
const THE_PARK_NAME: &str = "park";

/// What:
///  The file a parking control leaves beside the point it named.
/// Why:
///  The control reads the same sibling name the park itself writes,
///  so a build that read the switch and wrote one is caught by a file that exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function answeredBeside(park: Path): Path { return park.withName(`${park.name}.answered`) }
/// ```
fn answered_beside(point: &Path) -> PathBuf {
    // `with_extension` replaces what follows the last dot of the name, and the point has none, so the name gains the
    // suffix the park writes.
    return point.with_extension("answered");
}

/// What:
///  The parking point this control's parent gave the child,
///  read from the environment as the control's own bookkeeping rather than as the code under test reads it.
/// Why:
///  The child's fixture folders are its own,
///  and this is how a path the parent made reaches the child that must check it;
///  the value the *crate* would read is the same variable, which the assertions below show it never does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theParkGiven(parkFolder: Path): Path { return join(parkFolder, 'park') }
/// ```
fn the_park_a_control_names(folder: &Path) -> PathBuf {
    return folder.join(THE_PARK_NAME);
}

/// Says which switch is about to be asked,
/// so a child that a switch ends names it in its output.
fn asking(variable: &str) {
    eprintln!("a build without the feature asks {variable} and expects no answer");
}

/// What:
///  Asks every switch's reader while the value a control arms is set,
///  and asserts the answer a released run gives.
/// Why:
///  A read that survived in the release answers the armed value here,
///  which no assertion below accepts,
///  so the control fails naming the switch rather than passing quietly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(reach('after-intent')).toBeUndefined(); // this process is still alive
/// ```
fn the_answers_a_build_without_the_feature_gives() {
    asking(FAULT_VARIABLE);
    // The fault point would end this process; reaching the next line is the answer.
    reach(A_FAULT_POINT);

    asking(QUEUED_DIRECTORY_VARIABLE);
    // The queue directory the parent made for this run, read as the control's own name for it.
    let queue: PathBuf = PathBuf::from(std::env::var_os(QUEUED_DIRECTORY_VARIABLE).expect("the queue directory this control set"));
    queued("request-1");
    let left: usize = std::fs::read_dir(queue.as_path()).expect("the queue directory").count();
    assert_eq!(left, 0, "a release build wrote {left} queued-request files for {QUEUED_DIRECTORY_VARIABLE}");

    asking(FIRST_BATCH_VARIABLE);
    assert_eq!(pinned_first_batch(), None, "a release build read {FIRST_BATCH_VARIABLE}");

    asking(TEST_SOCKET_VARIABLE);
    // The address an entry goes to: a released run reaches nothing here, because a unit-test build that was given no
    // switch names no socket at all, and a build that read the switch would name the one this run was given.
    assert_eq!(crate::client_journal::target(), None, "a release build read {TEST_SOCKET_VARIABLE}");

    asking(PARK_VARIABLE);
    let park: PathBuf = the_park_a_control_names(Path::new(&std::env::var_os(PARK_VARIABLE).expect("the park folder this control set")));
    let found: bool = the_answer_waits_where_a_control_says(true);
    assert!(found, "a released run answers the look's own answer, true");
    assert!(!answered_beside(park.as_path()).exists(), "a release build read {PARK_VARIABLE} and wrote its answered file");

    asking(STOP_AFTER_VARIABLE);
    // No switch at all, which is what every add of a released run reads.
    assert_eq!(Stops::from_environment(), Ok(Stops::none()), "a release build read an add switch");

    asking(RUNTIME_PROBE_VARIABLE);
    // The environment a command was started with, holding the variable a control sets.
    let probe_environment: Vec<(OsString, OsString)> = vec![(OsString::from(RUNTIME_PROBE_VARIABLE), OsString::from("1"))];
    assert!(!is_requested(probe_environment.as_slice()), "a release build read {RUNTIME_PROBE_VARIABLE}");

    asking(PROBE_SWITCH_VARIABLE);
    assert_eq!(test_switch_from_environment(), TestSwitch::None, "a release build read {PROBE_SWITCH_VARIABLE}");

    asking(KEEP_DESCRIPTOR_VARIABLE);
    assert!(!keep_descriptor_from_environment(), "a release build read {KEEP_DESCRIPTOR_VARIABLE}");

    asking(CONNECTED_WINDOW_VARIABLE);
    let window_environment: Vec<(OsString, OsString)> = vec![(OsString::from(CONNECTED_WINDOW_VARIABLE), OsString::from("1"))];
    assert_eq!(connected_window(window_environment.as_slice()), CONNECTED_PUSH_QUIET_WINDOW, "a release build read {CONNECTED_WINDOW_VARIABLE}");

    asking(SLOW_BUILD_VARIABLE);
    assert_eq!(the_pause_a_control_asks_for(), Duration::ZERO, "a release build read {SLOW_BUILD_VARIABLE}");

    asking(PHASE_SIGNAL_VARIABLE);
    // The marker would end this process; returning at all is the answer.
    let reached: Result<(), crate::transaction_phase::PhaseMarkerError> = reach_transaction_phase(Phase::MergeIntent, None);
    assert!(reached.is_ok(), "a release build read {PHASE_SIGNAL_VARIABLE}: {reached:?}");

    asking(BIRTH_TIME_VARIABLE);
    assert_eq!(times_in_force(), BirthTimes::Recorded, "a release build read {BIRTH_TIME_VARIABLE}");

    asking(TEST_DEADLINE_VARIABLE);
    assert_eq!(the_deadline(true), COMPANION_DEADLINE, "a release build read {TEST_DEADLINE_VARIABLE}");

    asking(COMPANION_SWITCH_VARIABLE);
    assert_eq!(the_companion_switch_from_environment(), Switch::None, "a release build read {COMPANION_SWITCH_VARIABLE}");
}

/// What:
///  Every test-only variable a control arms,
///  with the value a control gives it,
///  and the two folders this run made for the switches that name a path.
/// Why:
///  One list is what the child is started with,
///  so a variable this list lacks is a variable this control does not show a release ignoring;
///  each value is one a build with the feature acts on, never one it could not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const armed: [string, string][] = [[FAULT_VARIABLE, 'after-intent'], /* … */];
/// ```
fn every_switch_a_control_arms(queue: &Path, park_folder: &Path) -> Vec<(OsString, OsString)> {
    // One pair per test-only variable, as the name and the value the child's environment carries.
    let pairs: Vec<(&str, OsString)> = vec![
        (THE_CHILD, OsString::from("1")),
        (FAULT_VARIABLE, OsString::from(A_FAULT_POINT)),
        (QUEUED_DIRECTORY_VARIABLE, queue.as_os_str().to_os_string()),
        (FIRST_BATCH_VARIABLE, OsString::from("7")),
        (TEST_SOCKET_VARIABLE, OsString::from("@the-socket-this-run-names")),
        (PARK_VARIABLE, park_folder.as_os_str().to_os_string()),
        (STOP_AFTER_VARIABLE, OsString::from("claim")),
        (HOLD_STEP_VARIABLE, OsString::from("slot")),
        (HOLD_UNTIL_VARIABLE, OsString::from("the-file-this-run-waits-for")),
        (RUNTIME_PROBE_VARIABLE, OsString::from("1")),
        (PROBE_SWITCH_VARIABLE, OsString::from("disabled")),
        (KEEP_DESCRIPTOR_VARIABLE, OsString::from("1")),
        (CONNECTED_WINDOW_VARIABLE, OsString::from("1")),
        (SLOW_BUILD_VARIABLE, OsString::from("60000")),
        (PHASE_SIGNAL_VARIABLE, OsString::from(A_KILLED_PHASE)),
        (BIRTH_TIME_VARIABLE, OsString::from("none")),
        (TEST_DEADLINE_VARIABLE, OsString::from("1")),
        (COMPANION_SWITCH_VARIABLE, OsString::from("disabled")),
    ];
    let mut environment: Vec<(OsString, OsString)> = Vec::new();
    for (variable, value) in pairs {
        // The pair the child's environment carries: an owned name and the owned value beside it.
        environment.push((OsString::from(variable), value));
    }
    return environment;
}

/// A build made without the feature reads no test-only switch,
/// whatever the environment holds.
#[test]
fn a_build_without_the_feature_reads_no_test_only_switch() {
    if std::env::var_os(THE_CHILD).is_some() {
        the_answers_a_build_without_the_feature_gives();
        return;
    }
    // Two folders this run owns: the queue a control would watch, and the folder the parking point sits in. A build that
    // read either switch writes into them, which the child then finds.
    let queue: PathBuf = crate::test_support::fixture("release-ignores-switches-queue");
    let park_folder: PathBuf = crate::test_support::fixture("release-ignores-switches-park");
    // The release file a parking control would wait for, left at once so a build that parked its answer here returns
    // rather than holding the control open.
    let release: PathBuf = answered_beside(the_park_a_control_names(park_folder.as_path()).as_path()).with_extension("release");
    std::fs::write(release.as_path(), b"").expect("the control's release file");
    let output: std::process::Output =
        crate::test_support::child_test_process::run_with_environment(THE_TEST, every_switch_a_control_arms(queue.as_path(), park_folder.as_path()).as_slice());
    let said: String = format!("{}{}", String::from_utf8_lossy(output.stdout.as_slice()), String::from_utf8_lossy(output.stderr.as_slice()));
    assert!(output.status.success(), "a build without `test-only-switches` answered a switch it was given, so a release build would too:\n{said}");
    assert!(said.contains("1 passed"), "the child ran no test:\n{said}");
}
