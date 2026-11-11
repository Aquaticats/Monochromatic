//! The tests of the timer units:
//! they fire once a day at a stated time,
//! never at boot or login,
//! show nothing on the desktop,
//! stay within the disk-weight ceiling,
//! and install without touching systemd when asked not to enable.

use std::{fs, path::{Path, PathBuf}};

use crate::lean_policy::RUN_DEADLINE_SECONDS;
use crate::lean_timer_units::{SCHEDULE, SERVICE_NAME, TIMER_NAME, install_timer, render_service, render_timer};
use crate::test_scratch::scratch;

#[test]
fn the_timer_fires_daily_at_the_stated_time_and_never_catches_up_at_boot() {
    let timer: String = render_timer();
    assert!(timer.contains(&format!("OnCalendar={SCHEDULE}")));
    assert!(timer.contains("Persistent=false"));
    assert!(!timer.contains("OnBootSec") && !timer.contains("OnStartupSec") && !timer.contains("OnUnitActiveSec"));
    assert!(timer.contains(&format!("Unit={SERVICE_NAME}")));
}

#[test]
fn the_service_has_no_install_section_shows_nothing_on_the_desktop_and_is_bounded() {
    let service: String = render_service(Path::new("/repo"), Path::new("/repo/.cache/toolchain-bin/cargo-shadow"));
    assert!(!service.contains("[Install]"), "only the timer may start it");
    assert!(!service.contains("DBUS") && !service.contains("DISPLAY") && !service.contains("notify"));
    assert!(service.contains("ExecStart=/repo/.cache/toolchain-bin/cargo-shadow lean run"));
    assert!(service.contains("WorkingDirectory=/repo"));
    assert!(service.contains("TimeoutStartSec="));
    let weight: u32 = service.lines().find_map(|line| { return line.strip_prefix("IOWeight=").map(|value| { return value.parse::<u32>().expect("a number"); }); }).expect("an IOWeight line");
    assert!(weight <= 10, "{weight}");
    for limit in ["CPUQuota=", "MemoryMax=", "TasksMax="] {
        assert!(service.contains(limit), "{limit}");
    }
}

#[test]
fn installing_without_enabling_writes_both_units_and_starts_nothing() {
    let root = scratch();
    let binary: PathBuf = root.path().join("cargo-shadow");
    fs::write(&binary, b"").expect("binary");
    let units: PathBuf = root.path().join("units");
    let lines: Vec<String> = install_timer(root.path(), &binary, &units, false).expect("installs");
    assert!(units.join(SERVICE_NAME).is_file() && units.join(TIMER_NAME).is_file());
    assert!(lines.iter().any(|line| { return line.contains("not enabled"); }));
}

#[test]
fn installing_with_no_installed_binary_is_refused_and_writes_nothing() {
    let root = scratch();
    let units: PathBuf = root.path().join("units");
    let outcome = install_timer(root.path(), &root.path().join("missing"), &units, false);
    assert!(outcome.is_err());
    assert!(!units.exists());
}

#[test]
fn the_service_start_timeout_is_above_the_run_s_own_deadline() {
    let service: String = render_service(Path::new("/repo"), Path::new("/repo/.cache/toolchain-bin/cargo-shadow"));
    let timeout: u64 = service
        .lines()
        .find_map(|line| { return line.strip_prefix("TimeoutStartSec=").map(|value| { return value.parse::<u64>().expect("a number of seconds"); }); })
        .expect("a TimeoutStartSec line");
    // The unit bounds the whole run above the run's own deadline, so a run that obeys its deadline is never killed as a timeout.
    assert!(timeout > RUN_DEADLINE_SECONDS, "TimeoutStartSec {timeout} is not above the deadline {RUN_DEADLINE_SECONDS}");
}
