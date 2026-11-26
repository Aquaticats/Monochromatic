//! What:
//!  Controls of the decision that turns a missing `git-lfs` from a skipped control into a failed one,
//! and of the one thing this file copies from the wrapper.
//! Why:
//!  The decision itself is a line in a test file,
//! so nothing else would fail if it stopped reading the run's statement,
//! or if the program named here stopped being the program the wrapper runs.
//! A control cannot change this process's environment beside the other controls of this binary,
//! so the arms that read the variable run in children of the test binary
//!  (`child_test_process`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runWith({ CLI_GIT_REQUIRE_REAL_GIT_LFS: '1', PATH: empty }).status).not.toBe(0);
//! expect(runWith({ CLI_GIT_REQUIRE_REAL_GIT_LFS: '', PATH: empty }).stderr).toContain('REAL GIT LFS CONTROL NOT RUN');
//! ```

use super::real_git_lfs_the_run_requires::{PROGRAM, REQUIRED_VARIABLE, required};
use crate::child_test_process::{ChildTest, finished};
use crate::test_support::{fixture, remove};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// What:
///  The name this file's own control has in the test binary,
/// which the child of that control is started on.
/// Why:
///   A child started on any other name would run nothing and pass,
/// and a control that proves nothing must not read as green.
const THIS_CONTROL: &str = "testing::real_git_lfs_the_run_requires_tests::the_requirement_from_the_environment_decides_whether_a_control_fails";

/// What:
///  Whether this process is one of the children of [`THIS_CONTROL`].
/// Why:
///   The child runs the same body and must not start children of its own,
/// so its presence is the marker that says this is the arm and not the parent.
const CHILD_VARIABLE: &str = "CLI_GIT_REAL_GIT_LFS_CONTROL_CHILD";

/// What:
///  The program named here is the program the wrapper runs.
/// Why:
///   The name is written in this test-support file because it is compiled into two targets and imports nothing of the crate,
/// and a copy that no control reads is a second answer waiting to be wrong:
/// a release that renamed the program would leave these controls looking for a file nothing installs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(PROGRAM).toBe(LFS_PROGRAM);
/// ```
#[test]
fn the_program_the_controls_look_for_is_the_one_the_wrapper_runs() {
    assert_eq!(PROGRAM, crate::lfs_pre_push::LFS_PROGRAM, "the test support names the program the wrapper runs");
}

/// What:
///  A run that states the real `git-lfs` must be there fails a control whose `PATH` finds none,
///  a run that states nothing prints the line and passes,
///  and the two differ only in that variable.
/// Why:
///   This is the promise the container gate relies on:
/// an image that lost `git-lfs` fails instead of passing,
/// and a developer whose host has none reads a line rather than a failure.
/// The `PATH` of both arms names one empty folder,
/// so a host or image that does have `git-lfs` in `/usr/bin` cannot answer for the arm that must find none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(requiredArm.status.success).toBe(false); expect(notRequiredArm.stdout).toContain('REAL GIT LFS CONTROL NOT RUN');
/// ```
#[test]
fn the_requirement_from_the_environment_decides_whether_a_control_fails() {
    if std::env::var_os(CHILD_VARIABLE).is_some() {
        // The arm: the program is absent from the list this child was given, and only the statement decides the answer.
        let _answer: Option<PathBuf> = required("the requirement of this run");
        return;
    }
    // The empty folder both arms search, made before either child starts and by the parent alone.
    let empty: PathBuf = fixture("real-git-lfs-required");
    let search: OsString = empty.clone().into_os_string();
    // The arm that states the program must be there: the control fails, and the failure names the variable.
    let environment: [(&str, &OsStr); 3] = [(CHILD_VARIABLE, OsStr::new("1")), (REQUIRED_VARIABLE, OsStr::new("1")), ("PATH", search.as_os_str())];
    let stated: std::process::Output = finished(ChildTest { exact_test: THIS_CONTROL, environment: environment.as_slice(), ..ChildTest::default() })
        .expect("the child control ends within the deadline");
    let said: String = format!("{}{}", String::from_utf8_lossy(stated.stdout.as_slice()), String::from_utf8_lossy(stated.stderr.as_slice()));
    assert!(!stated.status.success(), "a run that states the program must be there fails the control: {said}");
    assert!(said.contains(REQUIRED_VARIABLE), "the failure names the variable that made the absence a failure: {said}");
    assert!(said.contains("1 failed"), "the child ran the control and it failed: {said}");
    // The control of that arm: with the same absent program and no statement, the control passes and says it did not run.
    let unstated_environment: [(&str, &OsStr); 3] = [
        (CHILD_VARIABLE, OsStr::new("1")),
        // An empty value states nothing, as the btrfs and end-to-end acknowledgements read theirs, so the gate's own
        // value cannot leak into this arm through the parent's environment.
        (REQUIRED_VARIABLE, OsStr::new("")),
        ("PATH", search.as_os_str()),
    ];
    let unstated: std::process::Output =
        finished(ChildTest { exact_test: THIS_CONTROL, environment: unstated_environment.as_slice(), ..ChildTest::default() })
            .expect("the child control ends within the deadline");
    let unstated_said: String =
        format!("{}{}", String::from_utf8_lossy(unstated.stdout.as_slice()), String::from_utf8_lossy(unstated.stderr.as_slice()));
    assert!(unstated.status.success(), "a run that states nothing passes the control: {unstated_said}");
    assert!(unstated_said.contains("REAL GIT LFS CONTROL NOT RUN"), "the pass says the control did not run: {unstated_said}");
    assert!(
        unstated_said.contains("nothing states that this run must have it"),
        "the line says the run made no statement rather than blaming the variable: {unstated_said}"
    );
    remove(empty.as_path());
}
