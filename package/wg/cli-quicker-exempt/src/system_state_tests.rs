//! Verifies when the system counts as stopping:
//! only by the state `systemctl` prints for `shutdown.target`,
//! never by the no-login file.

/// The state under test.
use crate::system_state::{shutdown_target_means_stopping, HostSystemState, SystemState};

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

/// The answer comes from the host's `shutdown.target` alone:
/// the machine that runs this test is not shutting down,
/// and a probe that cannot run counts as a system that keeps running,
/// so both give false.
/// The module reads no file whose presence could hide a death,
/// which `/run/nologin` would
/// (it exists before a scheduled shutdown,
/// and an administrator can create it).
#[test]
fn a_running_host_is_not_stopping() {
    assert!(!HostSystemState.is_stopping());
}
