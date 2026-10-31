//! Unit tests of the clearnet updates slice:
//! the placement decision,
//! the `systemd-run` arguments,
//! and the probe-then-run sequence against a fake `systemd-run`.

use std::os::unix::fs::PermissionsExt;

use super::scope::EnvMap;
use super::updates_slice::{
    Facts, GUARD_ENVIRONMENT, OPT_OUT_ENVIRONMENT, TOOLCHAIN_UPDATES_SLICE, inside_updates_slice, run_through_systemd,
    stay_in_place_reason, systemd_run_arguments,
};
use crate::test_scratch::scratch;

/// The cgroup text of a process outside the slice.
const OUTSIDE: &str = "0::/user.slice/user-1000.slice/user@1000.service/app.slice/app-ghostty.scope\n";

#[test]
fn the_slice_name_is_the_one_the_watcher_matches() {
    assert_eq!(TOOLCHAIN_UPDATES_SLICE, "app-monochromatic\\x2dupdates.slice");
    let watcher = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../wg/cli-quicker-exempt/src/application_targets.rs"));
    if let Ok(source) = watcher {
        assert!(source.contains("pub const TOOLCHAIN_UPDATES_SLICE: &str = \"app-monochromatic\\\\x2dupdates.slice\";"));
    }
}

#[test]
fn only_a_whole_segment_counts_as_inside_the_slice() {
    assert!(inside_updates_slice(&format!("0::/app.slice/{TOOLCHAIN_UPDATES_SLICE}/run-p1.scope\n")));
    assert!(inside_updates_slice(&format!("0::/app.slice/{TOOLCHAIN_UPDATES_SLICE}\n")));
    assert!(!inside_updates_slice(&format!("0::/app.slice/{TOOLCHAIN_UPDATES_SLICE}-extra/x\n")));
    assert!(!inside_updates_slice(&format!("1:name=systemd:/{TOOLCHAIN_UPDATES_SLICE}/x\n")));
    assert!(!inside_updates_slice(""));
}

#[test]
fn a_run_moves_only_on_linux_with_a_user_manager_outside_the_slice() {
    let plain = EnvMap::from_pairs(&[("XDG_RUNTIME_DIR", "/run/user/1000")]);
    let ready = Facts { on_linux: true, env: &plain, cgroup_text: OUTSIDE, user_manager_present: true };
    assert!(stay_in_place_reason(&ready).is_none());
    let other_platform = Facts { on_linux: false, ..Facts { on_linux: true, env: &plain, cgroup_text: OUTSIDE, user_manager_present: true } };
    assert!(stay_in_place_reason(&other_platform).is_some());
    let guarded = EnvMap::from_pairs(&[(GUARD_ENVIRONMENT, "1")]);
    assert!(stay_in_place_reason(&Facts { on_linux: true, env: &guarded, cgroup_text: OUTSIDE, user_manager_present: true }).is_some());
    let opted_out = EnvMap::from_pairs(&[(OPT_OUT_ENVIRONMENT, "off")]);
    assert!(stay_in_place_reason(&Facts { on_linux: true, env: &opted_out, cgroup_text: OUTSIDE, user_manager_present: true }).is_some());
    let other_value = EnvMap::from_pairs(&[(OPT_OUT_ENVIRONMENT, "on")]);
    assert!(stay_in_place_reason(&Facts { on_linux: true, env: &other_value, cgroup_text: OUTSIDE, user_manager_present: true }).is_none());
    let inside = format!("0::/app.slice/{TOOLCHAIN_UPDATES_SLICE}/run-p1.scope\n");
    assert!(stay_in_place_reason(&Facts { on_linux: true, env: &plain, cgroup_text: &inside, user_manager_present: true }).is_some());
    assert!(stay_in_place_reason(&Facts { on_linux: true, env: &plain, cgroup_text: OUTSIDE, user_manager_present: false }).is_some());
}

#[test]
fn the_command_runs_in_a_fresh_user_scope_below_the_slice() {
    let arguments = systemd_run_arguments(&["mpm-shadow".to_string(), "sync".to_string()]);
    assert_eq!(
        arguments,
        vec![
            "--user".to_string(),
            "--scope".to_string(),
            "--quiet".to_string(),
            "--collect".to_string(),
            "--slice=app-monochromatic\\x2dupdates.slice".to_string(),
            "--".to_string(),
            "mpm-shadow".to_string(),
            "sync".to_string(),
        ]
    );
}

/// Writes an executable fake `systemd-run` into `directory` and returns its path.
fn fake_systemd_run(directory: &std::path::Path) -> String {
    let path = directory.join("systemd-run");
    std::fs::write(
        &path,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$FAKE_LOG\"\ncase \"$*\" in *--version*) exit \"$FAKE_PROBE_EXIT\";; esac\necho \"guard=$MONOCHROMATIC_TOOLCHAIN_IN_UPDATES_SLICE\" >> \"$FAKE_LOG\"\nexit 3\n",
    )
    .expect("write the fake");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("make the fake executable");
    return path.to_string_lossy().into_owned();
}

#[test]
fn a_good_probe_is_followed_by_the_real_run_under_the_guard() {
    let directory = scratch();
    let program = fake_systemd_run(directory.path());
    let log = directory.path().join("log");
    let env = EnvMap::from_pairs(&[("FAKE_LOG", &log.to_string_lossy()), ("FAKE_PROBE_EXIT", "0")]);
    let outcome = run_through_systemd(&program, "/usr/bin/mpm-shadow", &["sync".to_string(), "--global".to_string()], &env);
    assert_eq!(outcome, Some(3));
    let recorded = std::fs::read_to_string(&log).expect("read the log");
    assert!(recorded.contains("--slice=app-monochromatic\\x2dupdates.slice"));
    assert!(recorded.contains("--version"));
    assert!(recorded.contains("/usr/bin/mpm-shadow\nsync\n--global\n"));
    assert!(recorded.ends_with("guard=1\n"));
}

#[test]
fn a_failed_probe_leaves_the_run_in_place_without_running_the_work() {
    let directory = scratch();
    let program = fake_systemd_run(directory.path());
    let log = directory.path().join("log");
    let env = EnvMap::from_pairs(&[("FAKE_LOG", &log.to_string_lossy()), ("FAKE_PROBE_EXIT", "1")]);
    assert_eq!(run_through_systemd(&program, "/usr/bin/mpm-shadow", &["sync".to_string()], &env), None);
    let recorded = std::fs::read_to_string(&log).expect("read the log");
    assert!(!recorded.contains("guard="));
}

#[test]
fn a_missing_systemd_run_leaves_the_run_in_place() {
    let env = EnvMap::from_pairs(&[]);
    assert_eq!(run_through_systemd("/nonexistent/systemd-run", "/usr/bin/mpm-shadow", &["sync".to_string()], &env), None);
}
