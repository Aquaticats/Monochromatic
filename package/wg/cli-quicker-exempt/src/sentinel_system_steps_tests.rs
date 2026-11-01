//! Verifies how the state of a stopped unit is read:
//! a unit that ended,
//! failed,
//! or was collected and is no longer loaded runs nothing,
//! and anything else,
//! or text that is no state,
//! may still run.

/// The steps under test.
use crate::sentinel_launch::SentinelSteps;
/// The reading under test.
use crate::sentinel_system_steps::{unit_state_means_gone, SystemSteps};

/// Only `inactive` and `failed` mean that no process of the unit is left.
/// A unit that was collected and is no longer loaded prints `inactive` (read on the host).
#[test]
fn only_inactive_and_failed_mean_the_unit_is_gone() {
    assert!(unit_state_means_gone("inactive\n"));
    assert!(unit_state_means_gone("failed\n"));
    assert!(!unit_state_means_gone("active\n"));
    assert!(!unit_state_means_gone("activating\n"));
    assert!(!unit_state_means_gone("deactivating\n"));
    assert!(!unit_state_means_gone("reloading\n"));
    assert!(!unit_state_means_gone(""));
    assert!(!unit_state_means_gone("inactive active"));
}

/// A unit that is not loaded
/// (an unaccepted unit,
/// or one that ended and was collected)
/// counts as gone,
/// although `systemctl stop` fails for it.
#[test]
#[ignore = "requires systemctl and a running system manager"]
fn a_unit_that_is_not_loaded_counts_as_gone() {
    assert_eq!(SystemSteps.stop_service("wgtest-no-such-unit.service"), Ok(()));
}

/// A unit that is still active after the stop was refused is an error that says so.
/// Run with the name of a running root service in `WGTEST_RUNNING_UNIT`
/// and as an unprivileged user,
/// who is not allowed to stop it.
#[test]
#[ignore = "requires a running root service named in WGTEST_RUNNING_UNIT and an unprivileged caller"]
fn a_unit_that_stays_active_is_an_error_that_says_so() {
    let unit = std::env::var("WGTEST_RUNNING_UNIT").expect("name a running root service in WGTEST_RUNNING_UNIT");
    let error = SystemSteps.stop_service(&unit).expect_err("an unprivileged caller cannot stop a root service");
    assert!(error.starts_with("the unit is active"), "{error}");
}
