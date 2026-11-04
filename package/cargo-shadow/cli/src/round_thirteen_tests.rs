//! What:
//! Round 13 tests: what a changed process number in procfs does and does not show (12a), and the false greens of part 12b.
//! Why:
//! The review of round 12 (`cargo-shadow-sol-12a.txt`, `-12b.txt`) named schedules inside existing rows of the acceptance list.
//! The owner's answer of 2026-11-04 ("Out of scope, keep cheap guards") puts a program that deliberately changes mounts or `/proc`
//! while a run or a recovery is in progress outside the contract (assumption a), so the remount schedules of 12a are not tested here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round thirteen', () => { it('a changed positive number is unknown', ...) })
//! ```

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::time::Duration;

use rustix::process::{Pid, PidfdFlags, pidfd_open};

use crate::contain_counterexample_tests::wait_for_file;
use crate::constants::INSTALL_FAILED_EXIT_CODE;
use crate::control_group::{StartOf, missing_entry_verdict};
use crate::crates::LockSnapshot;
use crate::journal::{begin, recover};
use crate::journal_support::journal_dirs;
use crate::journal_transaction::begin_step_for_test;
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/n/one";

/// The live text of the lockfile a killed run left.
const LEFT_BY_THE_KILLED_RUN: &str = "unchecked output of the killed run";

/// The environment variable that gives the helper process its workspace.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_ROUND_THIRTEEN_FIXTURE_ROOT";

/// The environment variable that names the step of the preparation at which the helper stops.
const FIXTURE_STEP: &str = "CARGO_SHADOW_ROUND_THIRTEEN_FIXTURE_STEP";

/// A judge that refuses the live lockfile, so that a recovery which goes ahead restores the snapshot.
fn refusing(_before: &[LockSnapshot], _live: &[LockSnapshot]) -> Vec<String> {
    return vec!["dep@1.0.1 refused".to_string()];
}

/// The text of the live lockfile.
fn live_lock(root: &Path) -> String {
    return fs::read_to_string(root.join(CRATE_DIR).join("Cargo.lock")).expect("lock");
}

/// Acceptance rows A1, V2 (12a, the missing-entry inference):
/// after a failed lookup of a process's directory, only the value that the kernel prints for an ended, reaped process (`-1`) is gone;
/// the same number is a hidden process, and any other number (another positive id, `0`) is unknown, never gone.
#[test]
fn only_the_ended_process_number_shows_a_gone_process() {
    let pid: i32 = 4242;
    assert_eq!(missing_entry_verdict(-1, pid, "r"), StartOf::Gone, "the sentinel of an ended process was not read as gone");
    for (label, number) in [("the same number", i64::from(pid)), ("another positive id", i64::from(pid) + 1), ("a process this procfs does not show", 0), ("another negative number", -2)] {
        assert!(matches!(missing_entry_verdict(number, pid, "r"), StartOf::Unknown(_)), "{label}: read as gone");
    }
}

/// The kernel's own statement, which the rule above rests on (measured here, not recalled):
/// the `Pid:` line of the fdinfo of a pidfd is this process's number for the target while the target exists (a zombie that is not
/// reaped yet included), and `-1` once the target has ended and been reaped.
#[test]
fn the_fdinfo_of_a_pidfd_says_minus_one_only_for_a_reaped_process() {
    let mut child = Command::new("true").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts");
    let id: i32 = i32::try_from(child.id()).expect("a process id");
    let pidfd = pidfd_open(Pid::from_raw(id).expect("a pid"), PidfdFlags::empty()).expect("a pidfd");
    let number = |descriptor: &std::os::fd::OwnedFd| -> i64 {
        use std::os::fd::AsRawFd;
        let info: String = fs::read_to_string(format!("/proc/self/fdinfo/{}", descriptor.as_raw_fd())).expect("the fdinfo");
        return info.lines().find_map(|line| { return line.strip_prefix("Pid:"); }).expect("a Pid line").trim().parse::<i64>().expect("a number");
    };
    crate::cgroup_test_support::wait_until("the process to end and wait for its reaping", &|| { return fs::read_to_string(format!("/proc/{id}/stat")).is_ok_and(|text| { return text.rsplit(')').next().is_some_and(|rest| { return rest.trim_start().starts_with('Z'); }); }); });
    let unreaped: i64 = number(&pidfd);
    let _ = child.wait();
    let reaped: i64 = number(&pidfd);

    assert_eq!(unreaped, i64::from(id), "an unreaped zombie is still numbered");
    assert_eq!(reaped, -1, "the sentinel of an ended process is not -1 on this kernel");
}

/// The helper process of the preparation cut test:
/// it begins a transaction and stops (announcing it) after the step it is given, until it is killed.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_wrapper_killed_after_each_step_of_the_preparation_is_recovered_conservatively"]
fn wrapper_held_after_a_preparation_step_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let target: String = std::env::var(FIXTURE_STEP).expect("a step");
    begin_step_for_test(Some(Rc::new(move |step: &str| {
        if step == target {
            fs::write(root.join("reached"), step).expect("announce the step");
            std::thread::sleep(Duration::from_secs(300));
        }
    })));
    let root_again: PathBuf = PathBuf::from(std::env::var(FIXTURE_ROOT).expect("root"));
    let _ = begin(&root_again, &[LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() }], false);
}

/// Acceptance rows V3, V4 (12b, real cuts): a real wrapper killed after each step of the preparation of its journal is recovered
/// conservatively, with the live lockfile untouched until the journal can restore it:
/// before any snapshot the journal holds nothing a child could depend on and is removed; with a snapshot and no index, or an index and
/// no ready record, the only copy of the output is kept; with the ready record the recovery restores the snapshot and completes.
#[test]
fn a_wrapper_killed_after_each_step_of_the_preparation_is_recovered_conservatively() {
    for (step, removed, restored) in [
        ("directory", true, false),
        ("groups", true, false),
        ("view", true, false),
        ("owner", true, false),
        ("lock-0", false, false),
        ("index", false, false),
        ("ready", true, true),
    ] {
        let root = scratch();
        fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
        fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), LEFT_BY_THE_KILLED_RUN).expect("lock");
        let mut wrapper: Child = Command::new(std::env::current_exe().expect("this test executable"))
            .args(["--ignored", "--exact", "round_thirteen_tests::wrapper_held_after_a_preparation_step_fixture", "--nocapture"])
            .env(FIXTURE_ROOT, root.path())
            .env(FIXTURE_STEP, step)
            .spawn()
            .expect("the wrapper helper");
        wait_for_file(&root.path().join("reached"));
        wrapper.kill().expect("kill the wrapper at the step");
        let _ = wrapper.wait();
        let before: usize = journal_dirs(root.path()).len();

        let messages: Vec<String> = recover(root.path(), &refusing);

        assert_eq!(before, 1, "{step}: the cut left {before} journals");
        assert_eq!(journal_dirs(root.path()).is_empty(), removed, "{step}: the journal's fate is not the conservative one: {messages:?}");
        let expected: &str = if restored { "original" } else { LEFT_BY_THE_KILLED_RUN };
        assert_eq!(live_lock(root.path()), expected, "{step}: the live lockfile is not the expected one: {messages:?}");
    }
}

/// The fake cargo of the shim test:
/// `update` makes the lock satisfied, `metadata` prints an empty package list, and anything else is the user's command, which logs
/// itself and creates the sentinel file.
const FAKE_CARGO: &str = "#!/bin/sh\n\
echo \"$@\" >> \"$FAKE_CARGO_LOG\"\n\
if [ \"$1\" = update ]; then\n\
  case \" $* \" in\n\
    *\" --locked \"*) head -n 1 Cargo.lock | grep -q satisfied; exit $? ;;\n\
  esac\n\
  printf '# satisfied\\nversion = 4\\n' > Cargo.lock\n\
  exit 0\n\
fi\n\
if [ \"$1\" = metadata ]; then\n\
  printf '{\"workspace_root\":\"%s\",\"packages\":[]}' \"$PWD\"\n\
  exit 0\n\
fi\n\
: > \"$FAKE_CARGO_SENTINEL\"\n\
echo \"fake-cargo-ran $@\"\n\
exit 0\n";

/// The helper process of the shim test:
/// it is the shim (`run_shim`) for `cargo check`, started in a catalog repository with a stale lock and a fake cargo on `PATH`, and the
/// first stop of a scope is made to report a member that cannot be stopped.
/// It writes the exit code the shim returns. Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_run_that_cannot_show_quiescence_stops_the_shim_before_the_users_command"]
fn shim_with_a_scope_that_cannot_be_stopped_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    // The first run of the install step is the `--locked` check, which fails on the stale lock; the second is the update that writes the
    // lock, and that one cannot be stopped.
    let starts: std::cell::Cell<u32> = std::cell::Cell::new(0);
    let counted: Rc<std::cell::Cell<u32>> = Rc::new(starts);
    let seen: Rc<std::cell::Cell<u32>> = Rc::clone(&counted);
    crate::bounded_run::on_start_for_test(Some(Rc::new(move || {
        seen.set(seen.get() + 1);
        if seen.get() == 2 {
            crate::failpoints::arm(crate::failpoints::Point::ContainmentStopStuck);
        }
    })));
    let code: i32 = crate::shim::run_shim(vec![OsString::from("check")]);
    fs::write(PathBuf::from(root_text).join("shim-exit-code"), code.to_string()).expect("write the exit code");
}

/// Acceptance rows A2, R4 (12b, the shim and a user-command sentinel):
/// the shim, started for a resolving command in a catalog repository with a stale lock, runs the install step; a run of that step that
/// cannot show its scope empty stops the command: the shim exits with the install-failed code, the user's command (a fake cargo that
/// creates a sentinel file) is never started, the lockfile cargo wrote is neither rolled back nor accepted, and the journal is kept.
#[test]
fn a_run_that_cannot_show_quiescence_stops_the_shim_before_the_users_command() {
    if crate::cgroup_test_support::delegated_route().is_none() {
        return;
    }
    let root = scratch();
    let crate_root: PathBuf = root.path().join("repository").join(CRATE_DIR);
    let fake_directory: PathBuf = root.path().join("fake");
    fs::create_dir_all(&crate_root).expect("dir");
    fs::create_dir_all(&fake_directory).expect("dir");
    fs::write(root.path().join("repository").join("rust-catalog.toml"), "trustPolicy = \"no-downgrade\"\nminimumReleaseAge = 0\n\n[catalog]\n").expect("catalog");
    fs::write(crate_root.join("Cargo.toml"), "[package]\nname = \"y\"\nversion = \"0.1.0\"\n\n[workspace]\n").expect("manifest");
    fs::write(crate_root.join("Cargo.lock"), "# stale\nversion = 4\n").expect("lock");
    let fake: PathBuf = fake_directory.join("cargo");
    fs::write(&fake, FAKE_CARGO).expect("fake cargo");
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).expect("executable");
    let sentinel: PathBuf = root.path().join("users-command-ran");
    let log: PathBuf = root.path().join("fake-cargo.log");

    let status = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "round_thirteen_tests::shim_with_a_scope_that_cannot_be_stopped_fixture", "--nocapture"])
        .current_dir(&crate_root)
        .env(FIXTURE_ROOT, root.path())
        // The fake cargo comes first; the rest of `PATH` keeps `systemd-run` reachable, so the run is contained in a scope.
        .env("PATH", format!("{}:{}", fake_directory.display(), std::env::var("PATH").unwrap_or_default()))
        .env("CI", "")
        .env("FAKE_CARGO_LOG", &log)
        .env("FAKE_CARGO_SENTINEL", &sentinel)
        .status()
        .expect("the shim helper");

    assert!(status.success(), "the helper failed: {status:?}");
    let code: i32 = fs::read_to_string(root.path().join("shim-exit-code")).expect("the exit code").trim().parse::<i32>().expect("a number");
    let calls: String = fs::read_to_string(&log).unwrap_or_default();
    let journals: Vec<PathBuf> = journal_dirs(&root.path().join("repository"));
    assert_eq!(code, INSTALL_FAILED_EXIT_CODE, "the shim did not stop the command: {calls:?}");
    assert!(!sentinel.exists(), "the user's command was started: {calls:?}");
    assert!(!calls.lines().any(|line| { return line.starts_with("check"); }), "the user's command reached cargo: {calls:?}");
    assert!(calls.lines().any(|line| { return line.starts_with("update"); }), "the install step never ran cargo update, so the schedule was not made: {calls:?}");
    assert_eq!(fs::read_to_string(crate_root.join("Cargo.lock")).expect("lock"), "# satisfied\nversion = 4\n", "the lockfile cargo wrote was rolled back or replaced: {calls:?}");
    assert_eq!(journals.len(), 1, "the journal was not kept: {journals:?}");
}
