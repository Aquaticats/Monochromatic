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

use super::support::{
    ENDED_WITH_SIGKILL, Fixture, SERVICE_LOG_FILE, a_filling_that_did_not_run_to_its_end, end_the_fills_clean_within, executable, fixture,
};
use git_policy_cli::work_item::{FAILED, PENDING, WorkItem, WorkKind, enqueue, state_directory};
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

/// What:
///  A filling whose record stands in `failed` is a failure that quotes the service's own words,
///  and one that settled in `done` is none.
/// Why:
///   This is the sentence a person reads instead of an empty folder,
/// so it must name where the record stands and carry the line the service wrote about this worktree:
/// a sentence that named the bytes without the reason is the failure the gate of 2026-11-15T06:00-05:00 printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('failed'); expect(said).toContain('could not be filled'); move(record, 'done'); expect(said).toBeUndefined();
/// ```
#[test]
fn a_filling_that_did_not_run_to_its_end_quotes_where_it_stands_and_what_the_service_said() {
    let fixture: Fixture = fixture("filling-outcome");
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the repository's Git directory");
    let worktree: PathBuf = fixture.root.join("deeper/stale");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    // The record an add hands the service for this worktree's filling.
    let item: WorkItem = WorkItem {
        kind: WorkKind::Prewarm,
        repository: worktree.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(git_policy_cli::package_repair_prewarm::PREWARM_REFERENCE),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::from("[]"),
        created: git_policy_cli::iso_time::now_iso(),
        tries: 0,
    };
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("the record is queued");
    let id: String = item.id();
    // The service's own line about this filling, as its log keeps it.
    let line: String = format!(
        "monochromatic-git-policy-cli-service: the Cargo build directory of {} could not be filled: the copy of {}/target.monochromatic-prewarm could not be cleaned (stopped by signal 9: ) at_ms=1\n",
        worktree.join("target").display(),
        worktree.display()
    );
    std::fs::write(fixture.root.join(SERVICE_LOG_FILE), line.as_bytes()).expect("the service's log");
    // The state a consumer that gave up on a filling leaves it in.
    std::fs::create_dir_all(state_directory(work.as_path(), FAILED).as_path()).expect("the failed state");
    std::fs::rename(state_directory(work.as_path(), PENDING).join(id.as_str()), state_directory(work.as_path(), FAILED).join(id.as_str())).expect("the record moves to failed");
    let said: String = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path()).expect("a filling whose record failed did not run to its end");
    assert!(said.contains(FAILED), "the sentence names where the record stands: {said}");
    assert!(said.contains("could not be filled"), "the sentence quotes the service's own reason: {said}");
    assert!(said.contains("stopped by signal 9"), "and how the clean ended: {said}");
    // The same record settled in `done` is a filling that ran to its end, and a control reads its bytes.
    std::fs::create_dir_all(state_directory(work.as_path(), git_policy_cli::work_item::DONE).as_path()).expect("the done state");
    std::fs::rename(state_directory(work.as_path(), FAILED).join(id.as_str()), state_directory(work.as_path(), git_policy_cli::work_item::DONE).join(id.as_str())).expect("the record settles");
    assert!(
        a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path()).is_none(),
        "a record that settled in done is a filling that ran to its end"
    );
}
