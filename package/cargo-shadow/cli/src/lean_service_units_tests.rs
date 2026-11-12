//! The tests of the service units:
//! the cleanup keeps running,
//! each pass bounded above the run's own deadline,
//! the next starting after a pause,
//! nothing showing on the desktop,
//! the disk-weight ceiling held,
//! the once-a-day timer never written and an old one removed,
//! and no `systemctl` when the install is asked not to enable.

use std::{fs, path::{Path, PathBuf}};

use crate::lean_policy::{PAUSE_BETWEEN_PASSES_SECONDS, RUN_DEADLINE_SECONDS};
use crate::lean_service_units::{RUNTIME_SLACK_SECONDS, SERVICE_NAME, TIMER_NAME, absent_unit, install_service, render_service};
use crate::test_scratch::scratch;

/// The service this install writes,
/// for a repository at `/repo`.
fn service_text() -> String {
    return render_service(Path::new("/repo"), Path::new("/repo/.cache/toolchain-bin/cargo-shadow"));
}

/// The median time a pass with nothing at all to clean took on this machine,
/// five runs,
/// 2026-11-11:
/// 0.052 s best and 0.054 s median over a world with nothing to clean,
/// and 0.177 s over 500 profile directories it kept.
const MEASURED_IDLE_PASS_MILLISECONDS: u64 = 54;

/// A pause shorter than ten times that pass turns a machine with nothing to clean into a loop.
const _: () = assert!(PAUSE_BETWEEN_PASSES_SECONDS * 1_000 >= 10 * MEASURED_IDLE_PASS_MILLISECONDS);

#[test]
fn the_service_runs_one_pass_after_another_and_never_gives_up_restarting() {
    let service: String = service_text();
    assert!(service.contains("Restart=always"), "a pass that ends must be followed by another:\n{service}");
    assert!(service.contains("RestartSec="), "the next pass waits for a stated pause:\n{service}");
    assert!(service.contains("StartLimitIntervalSec=0"), "a pass that finds nothing must not stop the loop:\n{service}");
    assert!(!service.contains("OnCalendar=") && !service.contains("OnBootSec") && !service.contains("OnUnitActiveSec"));
}

#[test]
fn the_service_waits_a_whole_minute_between_passes_so_an_empty_pass_is_no_busy_loop() {
    let service: String = service_text();
    let pause: String = service
        .lines()
        .find_map(|line| { return line.strip_prefix("RestartSec=").map(str::to_string); })
        .expect("a RestartSec line");
    assert_eq!(pause, format!("{PAUSE_BETWEEN_PASSES_SECONDS}s"));
}

#[test]
fn each_pass_is_bounded_above_the_run_s_own_deadline() {
    let service: String = service_text();
    let bound: u64 = service
        .lines()
        .find_map(|line| { return line.strip_prefix("RuntimeMaxSec=").map(|value| { return value.parse::<u64>().expect("a number of seconds"); }); })
        .expect("a RuntimeMaxSec line");
    // The unit bounds the whole pass above the run's own deadline, so a pass that obeys its deadline is never killed as a timeout.
    assert!(bound > RUN_DEADLINE_SECONDS, "RuntimeMaxSec {bound} is not above the deadline {RUN_DEADLINE_SECONDS}");
    assert_eq!(bound, RUN_DEADLINE_SECONDS + RUNTIME_SLACK_SECONDS);
}

#[test]
fn the_service_starts_when_the_user_manager_does_and_shows_nothing_on_the_desktop() {
    let service: String = service_text();
    assert!(service.contains("[Install]") && service.contains("WantedBy=default.target"), "a continuous service is installed for login:\n{service}");
    assert!(!service.contains("DBUS") && !service.contains("DISPLAY") && !service.contains("notify"));
    assert!(service.contains("ExecStart=/repo/.cache/toolchain-bin/cargo-shadow lean run"));
    assert!(service.contains("WorkingDirectory=/repo"));
    assert!(service.contains("Nice=15") && service.contains("IOSchedulingClass=idle"));
    let weight: u32 = service.lines().find_map(|line| { return line.strip_prefix("IOWeight=").map(|value| { return value.parse::<u32>().expect("a number"); }); }).expect("an IOWeight line");
    assert!(weight <= 10, "{weight}");
    for limit in ["CPUQuota=", "MemoryMax=", "TasksMax="] {
        assert!(service.contains(limit), "{limit}");
    }
}

#[test]
fn installing_without_enabling_writes_the_service_and_removes_the_once_a_day_timer() {
    let root = scratch();
    let binary: PathBuf = root.path().join("cargo-shadow");
    fs::write(&binary, b"").expect("binary");
    let units: PathBuf = root.path().join("units");
    fs::create_dir_all(&units).expect("units");
    fs::write(units.join(TIMER_NAME), "the once-a-day timer of the old install").expect("old timer");
    let lines: Vec<String> = install_service(root.path(), &binary, &units, false).expect("installs");
    assert!(units.join(SERVICE_NAME).is_file());
    assert!(!units.join(TIMER_NAME).exists(), "the once-a-day timer must go: {lines:?}");
    assert!(lines.iter().any(|line| { return line.contains(TIMER_NAME) && line.contains("removed"); }), "the removal must be said: {lines:?}");
    assert!(lines.iter().any(|line| { return line.contains("not enabled"); }), "{lines:?}");
}

#[test]
fn a_systemctl_answer_that_does_not_know_the_timer_is_read_as_the_timer_already_gone() {
    // This host's user manager answers a unit it does not know with exactly this line.
    assert!(absent_unit("Failed to disable unit: Unit cargo-shadow-lean.timer does not exist"));
    assert!(absent_unit("Unit cargo-shadow-lean.timer not loaded."));
    assert!(!absent_unit("Failed to connect to bus: No medium found"), "a manager that did not answer is not a timer that is gone");
}

#[test]
fn installing_writes_the_service_and_no_timer_at_all() {
    let root = scratch();
    let binary: PathBuf = root.path().join("cargo-shadow");
    fs::write(&binary, b"").expect("binary");
    let units: PathBuf = root.path().join("units");
    install_service(root.path(), &binary, &units, false).expect("installs");
    let written: Vec<String> = fs::read_dir(&units)
        .expect("units")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    assert_eq!(written, vec![String::from(SERVICE_NAME)], "the install writes the service and nothing a timer needs");
}

#[test]
fn installing_with_no_installed_binary_is_refused_and_writes_nothing() {
    let root = scratch();
    let units: PathBuf = root.path().join("units");
    let outcome = install_service(root.path(), &root.path().join("missing"), &units, false);
    assert!(outcome.is_err());
    assert!(!units.exists());
}
