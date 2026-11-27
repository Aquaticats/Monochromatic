//! What:
//! The tests of a launcher,
//! the per-tree program that starts a repository-published tool:
//! that a record naming the launcher itself is refused,
//! that two records naming each other are refused,
//! that a record naming the launcher of a real program still reaches that program,
//! and that a record naming a shadow is the arrangement `install` makes for `mpm`.
//! Why:
//! A launcher starts whatever its record names,
//! so a record that leads back to a launcher is the same never-ending start as a shadow starting itself,
//! and these runs are bounded by the same shared `run_bounded`.
//! A record that names a shadow is not one of those loops:
//! `install` links `mpm` to a launcher whose record names the installed shadow,
//! so the words for "this program is a shadow" belong to the shadow's own start check and not to a launcher.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // never_start_another_shadow/launchers-that-must-not-start-themselves.ts
//! test('a launcher whose record names itself stops at once', ...);
//! test('two launchers whose records name each other stop at once', ...);
//! test('a launcher whose record names a shadow runs the shadow of mpm', ...);
//! ```

use std::fs;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

use super::fixtures_and_bounded_runs::{
    PASS_THROUGH_ARGUMENTS,
    SHADOW_EXECUTABLE,
    SHADOW_FILE,
    TOOL,
    copy_program,
    expected_first_line,
    finished,
    run_bounded,
    scratch,
    write_real_script,
};
/// A launcher:
/// a copy of the built program with a record beside it that names the program it starts.
fn make_launcher(
    path: &Path,
    real: &Path,
) {
    copy_program(Path::new(SHADOW_EXECUTABLE), path);
    let record = mpm_shadow::toolchain::launcher::LaunchRecord {
        tool_id: "cargo:monochromatic-demo-tool".to_string(),
        lock_file: PathBuf::from("/dev/null"),
        real: real.to_path_buf(),
        scope: mpm_shadow::toolchain::scope::ScopeKind::Repo,
    };
    fs::write(
        mpm_shadow::toolchain::launcher::record_path(path),
        mpm_shadow::toolchain::launcher::render_record(&record),
    )
    .expect("write the record");
}

/// The command that starts a launcher with a bare environment.
fn command_for_launcher(launcher: &Path) -> Command {
    let mut command: Command = Command::new(launcher);
    command.arg("argument");
    command.env_clear();
    command.env("PATH", "/usr/bin:/bin");
    return command;
}

#[test]
fn a_launcher_whose_record_names_the_launcher_itself_stops_at_once_with_a_message() {
    let root = scratch();
    let launcher: PathBuf = root.path().join("launchers").join("tool");
    make_launcher(&launcher, &launcher);

    let output = finished(run_bounded(command_for_launcher(&launcher)));

    assert!(
        !output.status.success(),
        "a launcher that starts itself must not succeed"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("itself"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn two_launchers_whose_records_name_each_other_stop_at_once_with_a_message() {
    let root = scratch();
    let (first, second) = (
        root.path().join("launchers").join("first"),
        root.path().join("launchers").join("second"),
    );
    make_launcher(&first, &second);
    make_launcher(&second, &first);

    let output = finished(run_bounded(command_for_launcher(&first)));

    assert!(
        !output.status.success(),
        "launchers that start each other must not succeed"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("launcher"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_launcher_that_names_a_launcher_of_a_real_program_still_reaches_the_program() {
    let root = scratch();
    let (outer, inner, real) = (
        root.path().join("launchers").join("outer"),
        root.path().join("launchers").join("inner"),
        root.path().join("real").join("tool"),
    );
    // The shadow starts this script as the real program, so a child process writes it.
    crate::through_a_child_process::write_program(
        real.as_path(),
        b"#!/bin/sh\nprintf 'REAL %s\\n' \"$*\"\n",
    );
    make_launcher(&inner, &real);
    make_launcher(&outer, &inner);

    let output = finished(run_bounded(command_for_launcher(&outer)));

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "REAL argument\n");
}

/// What:
/// A launcher whose record names a shadow runs the shadow of `mpm`,
/// which reaches the real `mpm` behind it:
/// this is the arrangement `install` makes for `mpm`,
/// so it is not a loop and no launcher refuses it.
/// Why:
/// The words for a program that is a shadow belong to the check the shadow makes on the open file just before its start,
/// and a launcher makes no such check.
/// A launcher taught to refuse a shadow would refuse the `mpm` link `install` places itself.
/// Measured 2026-11-27:
/// a launcher named `mpm` whose record names the shadow binary,
/// with a real `mpm` script after it on PATH,
/// printed that script's output and exited 0.
#[test]
fn a_launcher_whose_record_names_a_shadow_runs_the_shadow_of_mpm() {
    let root = scratch();
    let (launchers, shadows, real) = (
        root.path().join("launchers"),
        root.path().join("shadows"),
        root.path().join("real"),
    );
    // The launcher carries the tool's name, so the shadow it reaches is started as that tool and runs in shadow mode.
    let launcher: PathBuf = launchers.join(TOOL);
    copy_program(Path::new(SHADOW_EXECUTABLE), &shadows.join(SHADOW_FILE));
    make_launcher(&launcher, &shadows.join(SHADOW_FILE));
    write_real_script(&real);

    let mut command: Command = Command::new(&launcher);
    command.args(PASS_THROUGH_ARGUMENTS);
    command.env_clear();
    command.env("PATH", format!("{}", real.display()));
    let output = finished(run_bounded(command));

    let stdout: String = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout.lines().next(),
        Some(expected_first_line().as_str()),
        "the real mpm behind the shadow did not run: {stdout}"
    );
}
