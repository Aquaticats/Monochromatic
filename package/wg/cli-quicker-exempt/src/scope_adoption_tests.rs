//! Verifies the `busctl` arguments,
//! the confirmation and failure paths against a fake client,
//! and the real manager in an ignored test.

/// The limits a scope carries.
use crate::inherited_limits::InheritedLimits;
/// Adoption under test.
use crate::scope_adoption::{adopt_arguments, ensure_slice_arguments, AdoptError, AdoptRequest, Adopter, BusctlAdopter};
/// The client the adopter runs.
use crate::user_client::UserClient;
/// Executable mode for the fake client.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Slice used by the arguments tests.
const SLICE: &str = "app-monochromatic\\x2dupdates.slice";

/// The adoption call carries the scope name,
/// the slice and every pid in the `au` array.
#[test]
fn the_adoption_arguments_follow_busctl_syntax() {
    let arguments = adopt_arguments("wg-quicker-update-10.scope", SLICE, &[10, 11], &InheritedLimits::default());
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

/// The limits of the cgroup the process left become typed properties of the scope,
/// and the property count covers them.
#[test]
fn the_limits_become_scope_properties() {
    let limits = InheritedLimits { cpu_quota_per_sec_usec: Some(8_000_000), memory_max: Some(1_073_741_824), ..InheritedLimits::default() };
    let arguments = adopt_arguments("wg-quicker-update-10.scope", SLICE, &[10], &limits);
    assert_eq!(arguments[9], "5");
    assert!(arguments.windows(3).any(|window| return window == ["CPUQuotaPerSecUSec", "t", "8000000"]));
    assert!(arguments.windows(3).any(|window| return window == ["MemoryMax", "t", "1073741824"]));
    assert_eq!(arguments.last().map(String::as_str), Some("0"));
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
/// optionally removes the fake process,
/// and exits with `code`.
fn fake_busctl(directory: &Path, proc_dir: &Path, move_pid: bool, remove_process: bool, code: i32) -> PathBuf {
    let path = directory.join("busctl");
    let log = directory.join("log");
    let cgroup_file = proc_dir.join("cgroup");
    let moving = if move_pid {
        format!("printf '%s\\n' '0::/user.slice/user-1000.slice/user@1000.service/app.slice/{SLICE}/x.scope' > '{}'\n", cgroup_file.display())
    } else {
        String::new()
    };
    let removing = if remove_process { format!("rm -r '{}'\n", proc_dir.display()) } else { String::new() };
    let body = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nprintf 'env=%s\\n' \"$XDG_RUNTIME_DIR\" >> '{}'\n{moving}{removing}echo 'boom' >&2\nexit {code}\n",
        log.display(),
        log.display()
    );
    std::fs::write(&path, body).expect("write the fake");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("make the fake executable");
    return path;
}

/// What one fake client does.
struct Behavior {
    /// Whether the fake moves the pid into the slice.
    move_pid: bool,
    /// Whether the fake removes the process directory.
    remove_process: bool,
    /// The exit code.
    code: i32,
}

/// Builds an adopter around the fake client and a fake procfs with one pid.
fn adopter_with(name: &str, behavior: &Behavior) -> (BusctlAdopter, PathBuf, PathBuf) {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-adopt-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    let proc_root = directory.join("proc");
    std::fs::create_dir_all(proc_root.join("77")).expect("make the fake procfs");
    let cgroup_file = proc_root.join("77/cgroup");
    std::fs::write(&cgroup_file, "0::/user.slice/user-1000.slice/user@1000.service/app.slice/old.scope\n").expect("write cgroup");
    let program = fake_busctl(&directory, &proc_root.join("77"), behavior.move_pid, behavior.remove_process, behavior.code);
    let adopter = BusctlAdopter {
        client: UserClient { program: program.to_string_lossy().into_owned(), timeout: Duration::from_secs(5) },
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

/// Builds a request for pid 77 with the given limits.
fn request<'a>(limits: &'a InheritedLimits, pids: &'a [i32]) -> AdoptRequest<'a> {
    let (uid, gid) = own_ids();
    return AdoptRequest { pids, uid, gid, slice: SLICE, limits };
}

/// Runs the client against the user's bus path and confirms the move.
#[test]
fn a_successful_adoption_runs_busctl_with_the_user_bus_and_confirms_the_move() {
    let (adopter, log, _cgroup) = adopter_with("ok", &Behavior { move_pid: true, remove_process: false, code: 0 });
    let (uid, _gid) = own_ids();
    let limits = InheritedLimits { memory_max: Some(4096), ..InheritedLimits::default() };
    adopter.adopt(&request(&limits, &[77])).expect("the move is confirmed");
    let recorded = std::fs::read_to_string(log).expect("read the log");
    assert!(recorded.contains("StartTransientUnit"));
    assert!(recorded.contains("wg-quicker-update-77.scope"));
    assert!(recorded.contains("MemoryMax\nt\n4096"));
    assert!(recorded.contains(&format!("env=/run/user/{uid}")));
}

/// A job the manager accepted but did not apply is reported as a failure.
#[test]
fn an_unconfirmed_move_is_an_error() {
    let (adopter, _log, _cgroup) = adopter_with("unmoved", &Behavior { move_pid: false, remove_process: false, code: 0 });
    let error = adopter.adopt(&request(&InheritedLimits::default(), &[77])).expect_err("not moved");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("is not in")), "{error}");
}

/// A client that fails reports its exit status and message,
/// and an empty request is refused.
#[test]
fn a_failing_client_is_an_error_naming_its_message() {
    let (adopter, _log, _cgroup) = adopter_with("fails", &Behavior { move_pid: false, remove_process: false, code: 1 });
    let error = adopter.adopt(&request(&InheritedLimits::default(), &[77])).expect_err("client fails");
    assert!(matches!(&error, AdoptError::Failed(text) if text.contains("boom")), "{error}");
    assert!(matches!(adopter.adopt(&request(&InheritedLimits::default(), &[])), Err(AdoptError::Failed(_))));
}

/// A process that exits while the manager call fails is a vanish and not a failure.
#[test]
fn a_process_that_exits_during_a_failed_call_is_vanished() {
    let (adopter, _log, _cgroup) = adopter_with("gone-failed", &Behavior { move_pid: false, remove_process: true, code: 1 });
    assert_eq!(adopter.adopt(&request(&InheritedLimits::default(), &[77])), Err(AdoptError::Vanished));
}

/// A process that exits after the manager accepted the job is noticed at once,
/// without waiting out the confirmation timeout.
#[test]
fn a_process_that_exits_after_an_accepted_call_is_vanished_without_waiting() {
    let (adopter, _log, _cgroup) = adopter_with("gone-ok", &Behavior { move_pid: false, remove_process: true, code: 0 });
    let slow = BusctlAdopter { confirmation_timeout: Duration::from_secs(20), ..adopter };
    let started = Instant::now();
    assert_eq!(slow.adopt(&request(&InheritedLimits::default(), &[77])), Err(AdoptError::Vanished));
    assert!(started.elapsed() < Duration::from_secs(5), "the confirmation waited for its timeout");
}

/// A client that outlives its timeout is a timeout failure,
/// which the policy treats as a possibly hung manager.
#[test]
fn a_hung_client_is_a_timeout_failure() {
    let (adopter, _log, _cgroup) = adopter_with("hung", &Behavior { move_pid: false, remove_process: false, code: 0 });
    let sleeper = adopter.client.program.clone();
    std::fs::write(&sleeper, "#!/bin/sh\nexec /usr/bin/sleep 30\n").expect("rewrite the fake");
    let hung = BusctlAdopter { client: UserClient { program: sleeper, timeout: Duration::from_millis(200) }, ..adopter };
    let error = hung.adopt(&request(&InheritedLimits::default(), &[77])).expect_err("the client hangs");
    assert!(matches!(error, AdoptError::TimedOut(_)), "{error}");
}

/// A missing client is an error,
/// not a silent skip.
#[test]
fn a_missing_client_is_an_error() {
    let adopter = BusctlAdopter {
        client: UserClient { program: "/nonexistent/busctl".to_string(), timeout: Duration::from_secs(1) },
        proc_root: PathBuf::from("/proc"),
        confirmation_timeout: Duration::from_millis(10),
    };
    assert!(matches!(adopter.adopt(&request(&InheritedLimits::default(), &[1])), Err(AdoptError::Failed(_))));
}

/// An existing slice directory needs no call.
#[test]
fn an_existing_slice_is_not_requested_again() {
    let (adopter, log, _cgroup) = adopter_with("slice", &Behavior { move_pid: false, remove_process: false, code: 1 });
    let (uid, gid) = own_ids();
    let directory = log.parent().expect("parent").to_path_buf();
    adopter.ensure_slice(uid, gid, SLICE, &directory).expect("an existing directory is enough");
    assert!(!log.exists());
    assert!(adopter.ensure_slice(uid, gid, SLICE, &directory.join("absent")).is_err());
}

/// Adopts a real sleeping child into a throwaway slice through the user's own manager,
/// with a memory limit that the new scope must report.
/// It needs a user manager and its bus.
#[test]
#[ignore = "needs the user systemd manager"]
fn the_real_manager_adopts_a_sleeping_child() {
    let mut child = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the child");
    let pid = i32::try_from(child.id()).expect("pid fits");
    let slice = "app-wgtest\\x2dadopt.slice";
    let limits = InheritedLimits { memory_max: Some(1_073_741_824), cpu_quota_per_sec_usec: Some(2_000_000), tasks_max: Some(64), ..InheritedLimits::default() };
    let started = Instant::now();
    let (uid, gid) = own_ids();
    let result = BusctlAdopter::new().adopt(&AdoptRequest { pids: &[pid], uid, gid, slice, limits: &limits });
    let elapsed = started.elapsed();
    let shown = std::process::Command::new("systemctl")
        .args(["--user", "show", &format!("wg-quicker-update-{pid}.scope"), "--property=MemoryMax,CPUQuotaPerSecUSec,TasksMax"])
        .output()
        .expect("ask the manager");
    let _killed = child.kill();
    let _reaped = child.wait();
    let _stopped = std::process::Command::new("systemctl").args(["--user", "stop", slice]).status();
    result.expect("the manager adopts the child");
    let text = String::from_utf8_lossy(&shown.stdout).into_owned();
    println!("adoption took {elapsed:?}: {text}");
    assert!(text.contains("MemoryMax=1073741824"), "{text}");
    assert!(text.contains("CPUQuotaPerSecUSec=2s"), "{text}");
    assert!(text.contains("TasksMax=64"), "{text}");
}
