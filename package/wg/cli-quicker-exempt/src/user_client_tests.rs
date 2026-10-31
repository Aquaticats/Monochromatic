//! Verifies the privilege drop,
//! the cleared environment,
//! the timeout kill,
//! and the failure variants of the user client against fake client scripts.

/// Client under test.
use crate::user_client::{client_command, ClientError, UserClient};
/// Executable mode for the fake client.
use std::os::unix::fs::PermissionsExt;
/// Fixture paths.
use std::path::{Path, PathBuf};
/// Bounded waits.
use std::time::{Duration, Instant};

/// Our own uid and gid,
/// so the privilege drop is a no-op in an unprivileged test.
fn own_ids() -> (u32, u32) {
    // SAFETY: getuid and getgid have no preconditions.
    return unsafe { (libc::getuid(), libc::getgid()) };
}

/// Makes an empty scratch directory for one test.
fn scratch(name: &str) -> PathBuf {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-client-{name}-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the scratch directory");
    return directory;
}

/// Writes an executable script and returns its path.
fn script(directory: &Path, body: &str) -> PathBuf {
    let path = directory.join("client");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write the fake client");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("make the fake client executable");
    return path;
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

/// The client sees only `PATH` and the user's runtime directory of the watcher's environment.
#[test]
fn the_client_environment_is_cleared() {
    // The watcher's own environment must hold something that could leak.
    assert!(std::env::vars().count() > 2, "the test process has a nearly empty environment");
    let directory = scratch("env");
    let recorded = directory.join("env.txt");
    let fake = script(&directory, &format!("/usr/bin/env > '{}'", recorded.display()));
    let client = UserClient { program: fake.to_string_lossy().into_owned(), timeout: Duration::from_secs(5) };
    let (uid, gid) = own_ids();
    client.run(uid, gid, &[]).expect("the client succeeds");
    let text = std::fs::read_to_string(recorded).expect("read the recorded environment");
    assert!(text.lines().any(|line| return line == "PATH=/usr/bin:/bin"), "{text}");
    assert!(text.lines().any(|line| return line == format!("XDG_RUNTIME_DIR=/run/user/{uid}")), "{text}");
    for forbidden in ["HOME=", "CARGO", "NEXTEST", "USER=", "LD_"] {
        assert!(!text.lines().any(|line| return line.starts_with(forbidden)), "{forbidden} leaked into {text}");
    }
}

/// A client that outlives its timeout is killed and reaped,
/// and the failure says it timed out.
#[test]
fn a_client_past_its_timeout_is_killed() {
    let directory = scratch("timeout");
    let pid_file = directory.join("pid");
    let fake = script(&directory, &format!("echo $$ > '{}'\nexec /usr/bin/sleep 30", pid_file.display()));
    let client = UserClient { program: fake.to_string_lossy().into_owned(), timeout: Duration::from_millis(300) };
    let (uid, gid) = own_ids();
    let started = Instant::now();
    let error = client.run(uid, gid, &[]).expect_err("the client is too slow");
    assert!(started.elapsed() < Duration::from_secs(10), "the run waited for the client");
    assert!(matches!(error, ClientError::TimedOut(_)), "{error}");
    assert!(error.to_string().contains("was killed"));
    let pid = std::fs::read_to_string(pid_file).expect("the client recorded its pid");
    assert!(!Path::new("/proc").join(pid.trim()).exists(), "the client {} is still alive", pid.trim());
}

/// A failing client reports its status and its message,
/// and a missing client is a start failure.
#[test]
fn failures_name_their_cause() {
    let directory = scratch("failure");
    let fake = script(&directory, "echo 'boom' >&2\nexit 3");
    let client = UserClient { program: fake.to_string_lossy().into_owned(), timeout: Duration::from_secs(5) };
    let (uid, gid) = own_ids();
    let error = client.run(uid, gid, &[]).expect_err("the client fails");
    assert!(matches!(error, ClientError::Failed(_)));
    assert!(error.to_string().contains("boom"));
    assert!(error.to_string().contains("exit status: 3"));
    let missing = UserClient::new("/nonexistent/busctl");
    let start_error = missing.run(uid, gid, &[]).expect_err("no such client");
    assert!(matches!(start_error, ClientError::NotStarted(_)));
}
