//! What:
//!  Controls of the ending a run may give a filling's clean,
//!  and of the reader that says how a filling ended.
//! Why:
//!   A control that shows what a failed filling is reported as rests on the stand-in ending the clean exactly as the runner
//! words an ending from outside its own bounds,
//! and on the reader naming the state and the words the filling came to.
//! A stand-in that ended the wrong run,
//!  or one that passed every run through and ended nothing,
//! would make such a control pass for no reason at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(signalOf(standIn, ['clean'], { CARGO_BUILD_BUILD_DIR: staging })).toBe(9); expect(run(standIn, ['--version']).stdout).toContain('real');
//! ```

use super::support::{ENDED_WITH_SIGKILL, Fixture, end_the_fills_clean_within, executable, fixture};
use std::os::unix::process::ExitStatusExt as _;
use std::path::PathBuf;
use std::process::Command;

/// What a stand-in's real program is in this control:
/// it says which of its runs reached it,
/// so a run the stand-in should have passed through is told apart from one it ended.
const A_REAL_PROGRAM: &str = "#!/bin/sh\nprintf 'reached the real program: %s\\n' \"$*\"\n";

/// The fixture's own `cargo` name,
/// which the stand-in takes over.
fn cargo_in(fixture: &Fixture) -> PathBuf {
    return fixture.root.join("bin/cargo");
}

/// What:
///  The end of a run of a stand-in `cargo`,
///  with standard error kept for the failure.
/// # Panics
///  When the program cannot be started.
fn run_cargo(fixture: &Fixture, arguments: &[&str], build_directory: Option<&str>) -> std::process::Output {
    let mut command: Command = Command::new(cargo_in(fixture));
    command.args(arguments).env_clear().env("PATH", "/usr/bin:/bin").env("HOME", fixture.root.join("home"));
    if let Some(staging) = build_directory {
        command.env(git_policy_cli::package_repair_prewarm::BUILD_DIRECTORY_VARIABLE, staging);
    }
    return command.output().expect("the stand-in cargo runs");
}

/// What:
///  A stand-in ends a filling's clean with `SIGKILL`, fails the resolution that follows it,
///  and is the real program for every other run.
/// Why:
///   This is the ending the gate's line names (`stopped by signal 9`) and the state it leaves (nothing displaced),
/// so each of the three runs is asserted:
/// the stand-in's own words end by the signal whatever it was doing,
/// a run of the real program is passed through untouched before that,
/// and the resolution after a killed clean is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['--version']).stdout).toContain('reached the real program'); expect(signalOf(run(['clean'], staging))).toBe(9); expect(run(['metadata']).status).toBe(101);
/// ```
#[test]
fn a_fillings_clean_ends_with_the_signal_and_every_other_run_reaches_the_real_program() {
    let fixture: Fixture = fixture("filling-clean-ended");
    let real: PathBuf = fixture.root.join("real-cargo");
    executable(real.as_path(), A_REAL_PROGRAM.as_bytes());
    std::fs::create_dir_all(fixture.root.join("bin")).expect("the fixture's bin directory");
    std::os::unix::fs::symlink(real.as_path(), cargo_in(&fixture)).expect("the fixture's Cargo link");
    end_the_fills_clean_within(&fixture, Some(ENDED_WITH_SIGKILL));
    // A run that is not the filling's clean is the real program's, and it says so.
    let passed: std::process::Output = run_cargo(&fixture, &["--version"], None);
    assert!(passed.status.success(), "a run that is not a filling's clean reaches the real program: {:?}", passed.status);
    assert!(
        String::from_utf8_lossy(passed.stdout.as_slice()).contains("reached the real program: --version"),
        "the real program is the one that ran: {}",
        String::from_utf8_lossy(passed.stdout.as_slice())
    );
    // A clean of a build directory that is not a filling's staging copy is passed through as well.
    let elsewhere: std::process::Output = run_cargo(&fixture, &["clean", "--locked"], Some(fixture.root.join("worktree/target").to_str().expect("utf-8")));
    assert!(elsewhere.status.success(), "a clean of a worktree's own build directory is the real program's: {:?}", elsewhere.status);
    // The filling's clean is the one that ends by the signal, which is the ending the runner words `stopped by signal 9`.
    let staging: PathBuf = fixture.root.join("deeper/stale").join(format!("target{}", git_policy_cli::package_repair_prewarm::STAGING_SUFFIX));
    let killed: std::process::Output = run_cargo(&fixture, &["clean", "--locked", "--package", "app"], Some(staging.to_str().expect("utf-8")));
    assert_eq!(killed.status.signal(), Some(libc::SIGKILL), "the filling's clean ends with the signal the runner reports as its own kind of ending: {:?}", killed.status);
    // The resolution after that clean is refused, which is the second half of the gate's line: nothing displaced, nothing kept.
    let resolved: std::process::Output = run_cargo(&fixture, &["metadata", "--offline", "--format-version", "1"], None);
    assert_eq!(resolved.status.code(), Some(101), "the resolution after a killed clean is refused: {:?} {}", resolved.status, String::from_utf8_lossy(resolved.stderr.as_slice()));
}

/// What:
///  A run that asks for no ending keeps the fixture's Cargo exactly as the fixture linked it.
/// Why:
///   Every run of the suite whose variable is unset must reach the toolchain's Cargo,
/// and a stand-in written by accident would end the clean of a filling some other control waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endTheFillsCleanWithin(fixture, undefined); expect(readlink(bin/cargo)).toBe(real);
/// ```
#[test]
fn a_run_that_asks_for_no_ending_keeps_the_real_cargo_linked() {
    let fixture: Fixture = fixture("filling-clean-not-ended");
    let real: PathBuf = fixture.root.join("real-cargo");
    executable(real.as_path(), A_REAL_PROGRAM.as_bytes());
    std::fs::create_dir_all(fixture.root.join("bin")).expect("the fixture's bin directory");
    std::os::unix::fs::symlink(real.as_path(), cargo_in(&fixture)).expect("the fixture's Cargo link");
    end_the_fills_clean_within(&fixture, None);
    assert_eq!(std::fs::read_link(cargo_in(&fixture)).expect("the fixture's Cargo is still the link it was"), real, "a run that asks for no ending leaves the real Cargo linked");
    let passed: std::process::Output = run_cargo(&fixture, &["--version"], None);
    assert!(passed.status.success(), "the real program still answers: {:?}", passed.status);
}
