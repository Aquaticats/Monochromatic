//! Verifies the `busctl` arguments,
//! the confirmation and failure paths against a fake client,
//! and the real manager in an ignored test.

/// Adoption under test.
use crate::scope_adoption::{adopt_arguments, client_command, ensure_slice_arguments, AdoptRequest, Adopter, BusctlAdopter};
/// Executable mode for the fake client.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::Duration;

/// Slice used by the arguments tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";

/// The adoption call carries the scope name,
/// the slice and every pid in the `au` array.
#[test]
fn the_adoption_arguments_follow_busctl_syntax() {
    let arguments = adopt_arguments("wg-quicker-update-10.scope", SLICE, &[10, 11]);
    assert_eq!(
        arguments,
        [
            "--user",
            "call",
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
            "StartTransientUnit",
            "ssa(sv)a(sa(sv))",
            "wg-quicker-update-10.scope",
            "fail",
            "3",
            "Slice",
            "s",
            SLICE,
            "PIDs",
            "au",
            "2",
            "10",
            "11",
            "Description",
            "s",
            "wg-quicker update traffic that bypasses the tunnel",
            "0",
        ]
    );
}

/// A root watcher drops to the user through `setpriv`,
/// and an unprivileged one runs the client directly.
#[test]
fn root_drops_to_the_user_before_calling_the_bus() {
    let arguments = vec!["--user".to_string(), "call".to_string()];
    let (program, full) = client_command("busctl", 1000, 1001, &arguments, true);
    assert_eq!(program, "setpriv");
    assert_eq!(
        full,
        ["--reuid", "1000", "--regid", "1001", "--clear-groups", "--no-new-privs", "busctl", "--user", "call"]
    );
    let (direct, plain) = client_command("busctl", 1000, 1001, &arguments, false);
    assert_eq!(direct, "busctl");
    assert_eq!(plain, arguments);
}

/// The slice call has one property and no auxiliary units.
#[test]
fn the_slice_arguments_name_the_slice_with_one_property() {
    let arguments = ensure_slice_arguments(SLICE);
    assert_eq!(arguments[7], SLICE);
    assert_eq!(arguments[9], "1");
    assert_eq!(arguments.last().map(String::as_str), Some("0"));
    assert_eq!(arguments.len(), 14);
}

/// Writes an executable fake `busctl` that appends its arguments to a log,
/// optionally moves a fake pid into the slice,
/// and exits with `code`.
fn fake_busctl(directory: &Path, cgroup_file: &Path, move_pid: bool, code: i32) -> PathBuf {
    let path = directory.join("busctl");
    let log = directory.join("log");
    let moving = if move_pid {
        format!("printf '%s\\n' '0::/user.slice/user-1000.slice/user@1000.service/app.slice/{SLICE}/x.scope' > '{}'\n", cgroup_file.display())
    } else {
        String::new()
    };
    let body = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nprintf 'env=%s\\n' \"$XDG_RUNTIME_DIR\" >> '{}'\n{moving}echo 'boom' >&2\nexit {code}\n",
        log.display(),
        log.display()
    );
    std::fs::write(&path, body).expect("write the fake");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("make the fake executable");
    return path;
}

/// Builds an adopter around the fake client and a fake procfs with one pid.
fn adopter_with(name: &str, move_pid: bool, code: i32) -> (BusctlAdopter, PathBuf, PathBuf) {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-adopt-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    let proc_root = directory.join("proc");
    std::fs::create_dir_all(proc_root.join("77")).expect("make the fake procfs");
    let cgroup_file = proc_root.join("77/cgroup");
    std::fs::write(&cgroup_file, "0::/user.slice/user-1000.slice/user@1000.service/app.slice/old.scope\n").expect("write cgroup");
    let program = fake_busctl(&directory, &cgroup_file, move_pid, code);
    let adopter = BusctlAdopter {
        program: program.to_string_lossy().into_owned(),
        proc_root,
        confirmation_timeout: Duration::from_millis(150),
    };
    return (adopter, directory.join("log"), cgroup_file);
}

/// Our own uid and gid,
/// so the privilege drop is a no-op in an unprivileged test.
fn own_ids() -> (u32, u32) {
    // SAFETY: getuid and getgid have no preconditions.
    return unsafe { (libc::getuid(), libc::getgid()) };
}

/// Runs the client against the user's bus path and confirms the move.
#[test]
fn a_successful_adoption_runs_busctl_with_the_user_bus_and_confirms_the_move() {
    let (adopter, log, _cgroup) = adopter_with("ok", true, 0);
    let (uid, gid) = own_ids();
    let request = AdoptRequest { pids: &[77], uid, gid, slice: SLICE };
    adopter.adopt(&request).expect("the move is confirmed");
    let recorded = std::fs::read_to_string(log).expect("read the log");
    assert!(recorded.contains("StartTransientUnit"));
    assert!(recorded.contains("wg-quicker-update-77.scope"));
    assert!(recorded.contains(&format!("env=/run/user/{uid}")));
}

/// A job the manager accepted but did not apply is reported as a failure.
#[test]
fn an_unconfirmed_move_is_an_error() {
    let (adopter, _log, _cgroup) = adopter_with("unmoved", false, 0);
    let (uid, gid) = own_ids();
    let error = adopter.adopt(&AdoptRequest { pids: &[77], uid, gid, slice: SLICE }).expect_err("not moved");
    assert!(error.to_string().contains("is not in"));
}

/// A client that fails reports its exit status and message.
#[test]
fn a_failing_client_is_an_error_naming_its_message() {
    let (adopter, _log, _cgroup) = adopter_with("fails", false, 1);
    let (uid, gid) = own_ids();
    let error = adopter.adopt(&AdoptRequest { pids: &[77], uid, gid, slice: SLICE }).expect_err("client fails");
    assert!(error.to_string().contains("boom"));
    assert!(adopter.adopt(&AdoptRequest { pids: &[], uid, gid, slice: SLICE }).is_err());
}

/// A missing client is an error,
/// not a silent skip.
#[test]
fn a_missing_client_is_an_error() {
    let adopter = BusctlAdopter {
        program: "/nonexistent/busctl".to_string(),
        proc_root: PathBuf::from("/proc"),
        confirmation_timeout: Duration::from_millis(10),
    };
    let (uid, gid) = own_ids();
    assert!(adopter.adopt(&AdoptRequest { pids: &[1], uid, gid, slice: SLICE }).is_err());
}

/// An existing slice directory needs no call.
#[test]
fn an_existing_slice_is_not_requested_again() {
    let (adopter, log, _cgroup) = adopter_with("slice", false, 1);
    let (uid, gid) = own_ids();
    let directory = log.parent().expect("parent").to_path_buf();
    adopter.ensure_slice(uid, gid, SLICE, &directory).expect("an existing directory is enough");
    assert!(!log.exists());
    assert!(adopter.ensure_slice(uid, gid, SLICE, &directory.join("absent")).is_err());
}

/// Adopts a real sleeping child into a throwaway slice through the user's own manager.
/// It needs a user manager and its bus.
#[test]
#[ignore = "needs the user systemd manager"]
fn the_real_manager_adopts_a_sleeping_child() {
    let mut child = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the child");
    let pid = i32::try_from(child.id()).expect("pid fits");
    let (uid, gid) = own_ids();
    let slice = "app-wgtest\\x2dadopt.slice";
    let result = BusctlAdopter::new().adopt(&AdoptRequest { pids: &[pid], uid, gid, slice });
    let _killed = child.kill();
    let _reaped = child.wait();
    let _stopped = std::process::Command::new("systemctl").args(["--user", "stop", slice]).status();
    result.expect("the manager adopts the child");
}
