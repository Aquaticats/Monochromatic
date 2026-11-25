//! What:
//!  Controls of the rule that turns a missing `strace` from a skipped control into a failed one,
//! for two runs that differ only in what they state.
//! Why:
//!  The rule is a few lines in a test-support file,
//! so nothing else would fail if it stopped reading the run's statement,
//! and every traced control in both test crates would pass again while checking nothing.
//!  A control cannot change this process's environment beside the other controls of its binary,
//! so the arms that read the variable,
//! including the empty value that states nothing,
//! run in children of the test binary
//!  (`child_test_process`).
//! The tracer itself is never removed from the host:
//!  both arms search a fixture folder that holds no program of that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runWith({ PATH: empty }).status).not.toBe(0);
//! expect(runWith({ PATH: empty, CLI_GIT_STRACE_ABSENT_ACKNOWLEDGED: '1' }).stderr).toContain('STRACE CONTROL NOT RUN');
//! ```

use super::strace_the_controls_run_under::{ABSENT_ACKNOWLEDGED_VARIABLE, decided, strace_available_or_acknowledged};
use crate::test_support::child_test_process::run_with_environment;
use crate::test_support::{fixture, remove};
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  The name this file's own control has in the test binary,
/// which the child of that control is started on.
/// Why:
///   A child started on any other name would run nothing and pass,
/// and a control that proves nothing must not read as green.
const THIS_CONTROL: &str = "testing::strace_the_controls_run_under_tests::the_absence_the_environment_states_decides_whether_a_control_fails";

/// What:
///  Whether this process is one of the children of [`THIS_CONTROL`].
/// Why:
///   The child runs the same body and must not start children of its own,
/// so its presence is the marker that says this is the arm and not the parent.
const CHILD_VARIABLE: &str = "CLI_GIT_STRACE_CONTROL_CHILD";

/// What:
///  A run that states nothing fails the control whose `PATH` holds no tracer,
/// and the failure names the variable that would have passed it.
/// Why:
///   This is the promise the decision makes:
/// a control whose cut or count cannot be made fails rather than passing unchecked,
/// so a machine or an image that lost `strace` is red instead of silently empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(() => decided('the control', undefined, false)).toThrow('CLI_GIT_STRACE_ABSENT_ACKNOWLEDGED');
/// ```
#[test]
#[should_panic(expected = "CLI_GIT_STRACE_ABSENT_ACKNOWLEDGED")]
fn a_run_that_states_nothing_fails_the_control_that_needs_the_tracer() {
    let _answer: Option<PathBuf> = decided("the rule's own control", None, false);
}

/// What:
///  A run that states the absence is known passes the control,
/// which names itself and checks nothing.
/// Why:
///   The other half of the decision:
/// a person on a machine without the tracer can say so and read which controls did not run,
/// and every other control still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decided('the control', undefined, true)).toBeUndefined();
/// ```
#[test]
fn an_acknowledged_absence_passes_the_control_and_names_it() {
    assert_eq!(
        decided("the rule's own control", None, true),
        None,
        "a run that acknowledged the absence passes the control, which prints the line naming itself"
    );
}

/// What:
///  A tracer that is there is handed to the control,
/// whether or not the run stated anything.
/// Why:
///   The statement decides only the case the tracer cannot run:
/// where it can,
/// the control must trace the command rather than skip it,
/// and the statement is no reason to skip a control that could run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decided('the control', '/fixture/strace', false)).toBe('/fixture/strace');
/// ```
#[test]
fn a_tracer_that_is_there_is_handed_to_the_control() {
    let found: PathBuf = PathBuf::from("/a/fixture/folder/strace");
    assert_eq!(
        decided("the rule's own control", Some(found.clone()), false),
        Some(found.clone()),
        "the tracer found is the one the control runs its command under"
    );
    assert_eq!(
        decided("the rule's own control", Some(found.clone()), true),
        Some(found),
        "a run that acknowledged the absence still traces a command it can trace"
    );
}

/// What:
///  The same decision from the environment:
/// a `PATH` that holds no tracer fails the control unless the run states the absence is known,
/// an empty value states nothing,
/// and the acknowledged arm prints the line that names the control.
/// Why:
///   The rule reads the process's environment,
/// so the arm that fails and the arm that passes must differ only in that variable.
/// The `PATH` of every arm names one fixture folder with no program of that name,
/// so a host that has `strace` in `/usr/bin` cannot answer for the arm that must find none,
/// and the host's program is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(failed.status.success).toBe(false); expect(acknowledged.stderr).toContain('STRACE CONTROL NOT RUN');
/// ```
#[test]
fn the_absence_the_environment_states_decides_whether_a_control_fails() {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        // The arm: the tracer is absent from the list this child was given, and only the statement decides the answer.
        let _answered: bool = strace_available_or_acknowledged("the acknowledgement of this run");
        return;
    }
    // The folder every arm searches, made before any child starts and by the parent alone.
    let empty: PathBuf = fixture("strace-acknowledged");
    let search: OsString = empty.clone().into_os_string();
    // The arm that states nothing: the control fails, and the failure names the variable.
    let unstated: std::process::Output = run_with_environment(THIS_CONTROL, &[
        (OsString::from(CHILD_VARIABLE), OsString::from("1")),
        // An empty value states nothing, as the btrfs and ZFS acknowledgements read theirs, so a value the parent's
        //  environment holds cannot pass this arm.
        (OsString::from(ABSENT_ACKNOWLEDGED_VARIABLE), OsString::new()),
        (OsString::from("PATH"), search.clone()),
    ]);
    let unstated_said: String =
        format!("{}{}", String::from_utf8_lossy(unstated.stdout.as_slice()), String::from_utf8_lossy(unstated.stderr.as_slice()));
    assert!(!unstated.status.success(), "a run that states nothing fails the control: {unstated_said}");
    assert!(unstated_said.contains(ABSENT_ACKNOWLEDGED_VARIABLE), "the failure names the variable that would pass it: {unstated_said}");
    assert!(unstated_said.contains("1 failed"), "the child ran the control and it failed: {unstated_said}");
    // The control of that arm: the same absent tracer, acknowledged, passes and says which control did not run.
    let acknowledged: std::process::Output = run_with_environment(THIS_CONTROL, &[
        (OsString::from(CHILD_VARIABLE), OsString::from("1")),
        (OsString::from(ABSENT_ACKNOWLEDGED_VARIABLE), OsString::from("1")),
        (OsString::from("PATH"), search),
    ]);
    let acknowledged_said: String = format!(
        "{}{}",
        String::from_utf8_lossy(acknowledged.stdout.as_slice()),
        String::from_utf8_lossy(acknowledged.stderr.as_slice())
    );
    assert!(acknowledged.status.success(), "a run that acknowledged the absence passes the control: {acknowledged_said}");
    assert!(
        acknowledged_said.contains("STRACE CONTROL NOT RUN: the acknowledgement of this run"),
        "the pass names the control that did not run: {acknowledged_said}"
    );
    remove(empty.as_path());
}
