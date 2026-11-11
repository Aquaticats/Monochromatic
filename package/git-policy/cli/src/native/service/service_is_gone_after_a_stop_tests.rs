//! Controls for whether a stopped commit service's processes are gone.
//! The manager is a recorder,
//! which answers a unit no control names as one the manager does not hold.

use super::*;
use crate::service_unit_manager_recorder_test_support::Recorder;

/// An instance name a control asks about.
const INSTANCE: &str = "0000000000000000000000000000000a";

/// What:
///  A unit the manager does not hold has no process of ours,
///  so nothing can still write its journal.
/// Why:
///   This is the answer a stop of a unit the manager has already unloaded leaves,
/// and it must be `true` there or every such journal would stay for ever.
#[test]
fn a_unit_the_manager_does_not_hold_is_gone() {
    let mut manager: Recorder = Recorder::default();
    assert!(service_is_gone(&mut manager, INSTANCE), "{:?}", manager.calls());
}

/// What:
///  A unit that reports it is deactivating and then gone is waited for,
///  and the answer says the processes are gone.
/// Why:
///   A stop answers when the job is queued,
/// so the first state a stopped unit reports is its own ending,
/// and the answer must follow it to the end.
#[test]
fn a_unit_that_goes_from_deactivating_to_inactive_is_gone() {
    let mut manager: Recorder = Recorder::default();
    manager.report_states_of(service_unit_instance(INSTANCE).as_str(), &[Some("deactivating"), Some("inactive")]);
    assert!(service_is_gone(&mut manager, INSTANCE), "{:?}", manager.calls());
    assert!(manager.calls().len() > 1, "the unit's ending is asked about more than once: {:?}", manager.calls());
}

/// What:
///  A unit that still reports running is not gone,
///  and the answer is bounded by `SERVICE_GONE_AFTER_A_STOP`.
/// Why:
///   A service that is finishing a landing must not have its journal removed,
/// and a manager that never says it ended must not hold the command either.
#[test]
fn a_unit_that_keeps_running_is_not_gone() {
    let mut manager: Recorder = Recorder::default();
    manager.report_state_of(service_unit_instance(INSTANCE).as_str(), Some("active"));
    let began: std::time::Instant = std::time::Instant::now();
    assert!(!service_is_gone(&mut manager, INSTANCE));
    let waited: std::time::Duration = began.elapsed();
    assert!(waited >= SERVICE_GONE_AFTER_A_STOP, "the wait ended before its bound ({waited:?})");
    assert!(waited < SERVICE_GONE_AFTER_A_STOP + SERVICE_GONE_POLL * 20, "the wait outlasted its bound by more than a poll each ({waited:?})");
}

/// A manager that cannot answer proves nothing,
/// so the answer is that the service is not known to be gone.
#[test]
fn a_manager_that_cannot_answer_leaves_the_service_unproven() {
    let mut manager: Recorder = Recorder::default();
    manager.refuse_state_of(service_unit_instance(INSTANCE).as_str(), crate::systemd_dbus::CallError::TimedOut);
    assert!(!service_is_gone(&mut manager, INSTANCE));
}
