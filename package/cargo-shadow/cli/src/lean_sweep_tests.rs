//! The tests of mechanism B and of the removal protocol under it:
//! a stale unit is removed,
//! a unit or a folder in use is never removed,
//! and every refusal leaves the files where they were.

/// Files,
/// processes and locks.
use std::{fs, fs::File, os::unix::fs::PermissionsExt, os::unix::fs::symlink, path::{Path, PathBuf}, process::{Child, Command}};

/// The lock a build takes.
use rustix::fs::{FlockOperation, flock};

use crate::lean_identity::Deadline;
use crate::lean_sweep_plan::OlderCompilerSweep;
use crate::lean_test_support::{DAY, HOUR, NOW, add_incremental, add_new_unit, add_old_unit, copy_a_program, make_profile, real_environment};
use crate::lean_unit_removal::{ProfileOutcome, apply_removals};
use crate::test_scratch::scratch;

/// The compiler of the old units.
const OLD_COMPILER: u64 = 111;

/// The compiler of the current units.
const NEW_COMPILER: u64 = 222;

/// The grace period of these tests.
const GRACE: i64 = 2 * DAY;

/// A child process that is stopped when the guard drops,
/// however the test ends.
struct Sleeper {
    /// The running child.
    child: Child,
}

impl Sleeper {
    /// Starts `sleep` with the given working directory.
    fn in_directory(directory: &Path) -> Sleeper {
        let child: Child = Command::new("sleep").arg("120").current_dir(directory).spawn().expect("start sleep");
        return Sleeper { child };
    }
}

impl Drop for Sleeper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
/// Runs the sweep for real on a profile directory.
fn sweep(profile: &Path) -> ProfileOutcome {
    let mut log: Vec<String> = Vec::new();
    return apply_removals(profile, &OlderCompilerSweep { grace_seconds: GRACE }, &real_environment(), NOW, false, &Deadline::after(std::time::Duration::from_secs(60)), &mut log);
}

/// A profile with one stale old-layout unit and one current one.
fn stale_and_current(root: &Path) -> (PathBuf, Vec<PathBuf>, Vec<PathBuf>) {
    let profile: PathBuf = make_profile(root, "debug");
    let stale: Vec<PathBuf> = add_old_unit(&profile, "alpha", "00000000000000aa", OLD_COMPILER, NOW - 10 * DAY);
    let current: Vec<PathBuf> = add_old_unit(&profile, "beta", "00000000000000bb", NEW_COMPILER, NOW - HOUR);
    return (profile, stale, current);
}

#[test]
fn a_stale_unit_is_removed_whole_and_the_current_unit_stays() {
    let root = scratch();
    let (profile, stale, current) = stale_and_current(root.path());
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.skipped, None);
    assert_eq!(outcome.units, 1);
    for path in &stale {
        assert!(!path.exists(), "{} should be gone", path.display());
    }
    for path in &current {
        assert!(path.exists(), "{} should stay", path.display());
    }
    assert!(outcome.bytes > 0, "freed bytes are reported");
    assert!(profile.join(".cargo-build-lock").exists(), "the lock file is never removed");
    assert!(!profile.join(".cargo-shadow-trash").exists(), "the trash is emptied and removed");
}

#[test]
fn a_stale_unit_of_the_new_layout_is_removed_whole() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let stale: PathBuf = add_new_unit(&profile, "gamma", "00000000000000cc", OLD_COMPILER, NOW - 9 * DAY);
    let current: PathBuf = add_new_unit(&profile, "delta", "00000000000000dd", NEW_COMPILER, NOW - HOUR);
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.units, 1);
    assert!(!stale.exists());
    assert!(current.exists());
}

#[test]
fn a_unit_a_process_uses_is_kept_whole_and_the_rest_is_still_swept() {
    let root = scratch();
    let (profile, stale, _current) = stale_and_current(root.path());
    let other_stale: Vec<PathBuf> = add_old_unit(&profile, "epsilon", "00000000000000ee", OLD_COMPILER, NOW - 12 * DAY);
    // A process whose working directory is inside the build directory of the stale unit `alpha`.
    let _holder: Sleeper = Sleeper::in_directory(&stale[1]);
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.kept_in_use, 1, "{outcome:?}");
    for path in &stale {
        assert!(path.exists(), "{} is in use and must stay", path.display());
    }
    for path in &other_stale {
        assert!(!path.exists(), "{} is not in use and goes", path.display());
    }
}

/// A unit a process is running a program from stays,
/// even when the profile directory is named through a symbolic link.
/// Why:
/// `/proc` names the program a process runs with every link resolved,
/// so a root reached through a link matched none of those names and the sweep removed the unit
/// (measured 2026-11-11 in a throwaway build folder: a unit whose `out/` held the program a live process was running
/// went with `0 uses under the roots`, and the same fixture named without the link kept it).
#[test]
fn a_unit_a_process_runs_a_program_from_is_kept_even_through_a_link() {
    let root = scratch();
    let real: PathBuf = root.path().join("real");
    fs::create_dir_all(&real).expect("the real directory");
    let linked: PathBuf = root.path().join("link");
    symlink(&real, &linked).expect("a link to the real directory");
    let profile: PathBuf = make_profile(&real, "debug");
    let stale: PathBuf = add_new_unit(&profile, "alpha", "00000000000000aa", OLD_COMPILER, NOW - 10 * DAY);
    add_new_unit(&profile, "beta", "00000000000000bb", NEW_COMPILER, NOW - HOUR);
    let program: PathBuf = copy_a_program(&stale.join("out"));
    let child: Child = Command::new(&program).arg("120").spawn().expect("start the program the unit holds");
    let _holder: Sleeper = Sleeper { child };
    let outcome: ProfileOutcome = sweep(&linked.join("debug"));
    assert_eq!(outcome.units, 0, "a unit a process runs a program from was removed: {outcome:?}");
    assert_eq!(outcome.kept_in_use, 1, "{outcome:?}");
    assert!(stale.exists(), "{} holds a running program and must stay", stale.display());
    assert!(program.exists(), "{} is running and must stay", program.display());
}

#[test]
fn a_profile_whose_build_lock_is_held_is_skipped_and_nothing_is_removed() {
    let root = scratch();
    let (profile, stale, _current) = stale_and_current(root.path());
    let lock: File = File::open(profile.join(".cargo-build-lock")).expect("open the lock");
    flock(&lock, FlockOperation::NonBlockingLockExclusive).expect("take the lock like a build");
    let outcome: ProfileOutcome = sweep(&profile);
    assert!(outcome.skipped.as_deref().is_some_and(|reason| { return reason.starts_with("busy"); }), "{outcome:?}");
    for path in &stale {
        assert!(path.exists(), "{} must stay while a build holds the lock", path.display());
    }
    drop(lock);
    assert_eq!(sweep(&profile).units, 1, "once the build ends the sweep goes ahead");
}

#[test]
fn a_directory_with_no_lock_file_is_skipped() {
    let root = scratch();
    let (profile, stale, _current) = stale_and_current(root.path());
    fs::remove_file(profile.join(".cargo-build-lock")).expect("remove the lock file");
    let outcome: ProfileOutcome = sweep(&profile);
    assert!(outcome.skipped.is_some());
    assert!(stale[0].exists());
}

#[test]
fn one_compiler_group_alone_is_left_untouched_however_old() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let only: Vec<PathBuf> = add_old_unit(&profile, "alpha", "00000000000000aa", OLD_COMPILER, NOW - 40 * DAY);
    add_incremental(&profile, "alpha-1", NOW - 40 * DAY);
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!((outcome.units, outcome.incremental), (0, 0));
    assert!(only[0].exists());
}

#[test]
fn an_older_compiler_within_the_grace_period_is_kept() {
    let root = scratch();
    let profile: PathBuf = make_profile(root.path(), "debug");
    let recent_old: Vec<PathBuf> = add_old_unit(&profile, "alpha", "00000000000000aa", OLD_COMPILER, NOW - DAY);
    add_old_unit(&profile, "beta", "00000000000000bb", NEW_COMPILER, NOW - HOUR);
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.units, 0);
    assert!(recent_old[0].exists(), "a second toolchain used a day ago may still be in use");
}

#[test]
fn incremental_state_older_than_the_newest_compiler_goes_and_newer_stays() {
    let root = scratch();
    let (profile, _stale, _current) = stale_and_current(root.path());
    let old_incremental: PathBuf = add_incremental(&profile, "alpha-1", NOW - 20 * DAY);
    let new_incremental: PathBuf = add_incremental(&profile, "beta-1", NOW - HOUR);
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.incremental, 1);
    assert!(!old_incremental.exists());
    assert!(new_incremental.exists());
}

#[test]
fn a_fingerprint_with_no_readable_compiler_is_never_removed() {
    let root = scratch();
    let (profile, _stale, _current) = stale_and_current(root.path());
    let odd: PathBuf = profile.join(".fingerprint").join("odd-00000000000000ff");
    fs::create_dir_all(&odd).expect("odd fingerprint");
    fs::write(odd.join("lib-odd.json"), b"not json").expect("odd json");
    let outcome: ProfileOutcome = sweep(&profile);
    assert_eq!(outcome.units, 1);
    assert!(odd.exists());
}

#[test]
fn a_directory_in_no_known_layout_is_left_untouched() {
    let root = scratch();
    let profile: PathBuf = root.path().join("strange");
    fs::create_dir_all(&profile).expect("directory");
    fs::write(profile.join(".cargo-build-lock"), b"").expect("lock");
    fs::write(profile.join("keep.txt"), b"data").expect("data");
    let outcome: ProfileOutcome = sweep(&profile);
    assert!(outcome.skipped.as_deref().is_some_and(|reason| { return reason.starts_with("unknown layout"); }));
    assert!(profile.join("keep.txt").exists());
}

#[test]
fn a_dry_run_removes_nothing_and_reports_what_would_go() {
    let root = scratch();
    let (profile, stale, _current) = stale_and_current(root.path());
    let mut log: Vec<String> = Vec::new();
    let outcome: ProfileOutcome = apply_removals(&profile, &OlderCompilerSweep { grace_seconds: GRACE }, &real_environment(), NOW, true, &Deadline::after(std::time::Duration::from_secs(60)), &mut log);
    assert_eq!(outcome.units, 1);
    assert!(outcome.bytes > 0);
    for path in &stale {
        assert!(path.exists());
    }
}

#[test]
fn each_removal_logs_the_lock_and_process_proof() {
    let root = scratch();
    let (profile, _stale, _current) = stale_and_current(root.path());
    let mut log: Vec<String> = Vec::new();
    apply_removals(&profile, &OlderCompilerSweep { grace_seconds: GRACE }, &real_environment(), NOW, false, &Deadline::after(std::time::Duration::from_secs(60)), &mut log);
    let line: &String = log.iter().find(|line| { return line.starts_with("removed unit alpha"); }).expect("a removal line for the stale unit");
    assert!(line.contains("exclusive flock on"), "{line}");
    assert!(line.contains("/proc scan:"), "{line}");
}

#[test]
fn what_an_earlier_stopped_run_left_in_the_trash_is_deleted_by_the_next_sweep() {
    let root = scratch();
    let (profile, _stale, _current) = stale_and_current(root.path());
    let leftover: PathBuf = profile.join(".cargo-shadow-trash").join("123-456-1-unit");
    fs::create_dir_all(&leftover).expect("a claimed unit that was never deleted");
    fs::write(leftover.join("file"), b"claimed earlier").expect("file");
    sweep(&profile);
    assert!(!leftover.exists());
}

#[test]
fn a_leftover_with_a_directory_this_user_may_not_write_is_removed_by_the_next_sweep() {
    let root = scratch();
    let (profile, _stale, _current) = stale_and_current(root.path());
    let leftover: PathBuf = profile.join(".cargo-shadow-trash").join("123-456-1-unit");
    let closed: PathBuf = leftover.join("build").join("closed");
    fs::create_dir_all(closed.join("inner")).expect("a claimed unit that was never deleted");
    fs::write(closed.join("inner").join("file"), b"claimed earlier").expect("file");
    fs::write(closed.join("file"), b"claimed earlier").expect("file");
    fs::set_permissions(&closed, fs::Permissions::from_mode(0o555)).expect("mode");
    fs::set_permissions(closed.join("inner"), fs::Permissions::from_mode(0o500)).expect("mode");
    let mut log: Vec<String> = Vec::new();
    let outcome: ProfileOutcome = apply_removals(&profile, &OlderCompilerSweep { grace_seconds: GRACE }, &real_environment(), NOW, false, &Deadline::after(std::time::Duration::from_secs(60)), &mut log);
    assert!(outcome.refusals.is_empty(), "{outcome:?}");
    assert!(!leftover.exists(), "the sweep finished what the stopped run could not");
    let line: &String = log
        .iter()
        .find(|line| { return line.starts_with("finished an earlier run's deletion in ") && line.contains("mode changed on 2 directories"); })
        .unwrap_or_else(|| panic!("no line names the changed modes: {log:#?}"));
    assert!(line.contains(&leftover.display().to_string()), "{line}");
}
