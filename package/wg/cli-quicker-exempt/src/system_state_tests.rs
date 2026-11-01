//! Verifies when the system counts as stopping:
//! the state `systemctl` prints for `shutdown.target`,
//! and the no-login file.

/// The state under test.
use crate::system_state::{shutdown_target_means_stopping, HostSystemState, SystemState};
/// Fixture paths.
use std::path::Path;

/// Only an active or activating `shutdown.target` means a shutdown is under way.
#[test]
fn only_an_active_or_activating_shutdown_target_means_stopping() {
    assert!(shutdown_target_means_stopping("active\n"));
    assert!(shutdown_target_means_stopping("activating\n"));
    assert!(!shutdown_target_means_stopping("inactive\n"));
    assert!(!shutdown_target_means_stopping("deactivating\n"));
    assert!(!shutdown_target_means_stopping("failed\n"));
    assert!(!shutdown_target_means_stopping(""));
    assert!(!shutdown_target_means_stopping("inactive active"));
    assert!(!shutdown_target_means_stopping("reactivating"));
}

/// A no-login file means the system is stopping,
/// whatever `shutdown.target` says.
#[test]
fn a_no_login_file_means_stopping() {
    let directory = crate::test_scratch::scratch_root().join(format!("wg-quicker-nologin-{}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("make the directory");
    let nologin = directory.join("nologin");
    std::fs::write(&nologin, "System is going down.\n").expect("write the file");
    assert!(HostSystemState { nologin: &nologin }.is_stopping());
}

/// With no no-login file the answer comes from the host:
/// the machine that runs this test is not shutting down,
/// and a probe that cannot run counts as a system that keeps running,
/// so both give false.
#[test]
fn without_a_no_login_file_a_running_host_is_not_stopping() {
    assert!(!HostSystemState { nologin: Path::new("/nonexistent/no-such-nologin") }.is_stopping());
}
