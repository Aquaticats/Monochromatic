//! What:
//!  Named load-boundary failures,
//!  recovery and native-thread isolation.
//! Why:
//!  A caught panic must not leave a pending request or mix another thread's rule source.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Fail before/after request consumption, then submit another request through the same boundary.
//! ```

/// Import the exact protected boundary and test-only cache-free matcher constructor.
use super::{LoadRequest, PENDING, protected_request};
use crate::frx_load::LoadedRules;
use std::path::PathBuf;

/// Owned fixture arguments avoid filesystem/cache side effects.
fn request() -> LoadRequest {
    return LoadRequest { path: PathBuf::from("fixture-rules"), builtin: false, explicit: true };
}

/// Failure before consuming the request proves cleanup is owned by the boundary.
fn panic_before_take() -> anyhow::Result<LoadedRules> {
    panic!("SYNTHETIC_PRIVATE_RULE_94a1");
}

/// A successful callback consumes its exact request and returns a real compiled fixture.
fn complete_request() -> anyhow::Result<LoadedRules> {
    let current: LoadRequest = PENDING.take().expect("pending request");
    assert_eq!(current.path, PathBuf::from("fixture-rules"));
    return Ok(crate::frx_load::test_rules("FIXTURE_LITERAL_LONG\n"));
}

/// Failure after consumption must also permit a later load.
fn panic_after_take() -> anyhow::Result<LoadedRules> {
    let _current: LoadRequest = PENDING.take().expect("pending request");
    panic!("SYNTHETIC_PRIVATE_RULE_94a1");
}

/// Both fault timings return payload-free errors and leave the next call usable.
#[test]
fn panics_do_not_poison_the_next_request() {
    for operation in [panic_before_take as fn() -> anyhow::Result<LoadedRules>, panic_after_take] {
        let error = match protected_request(request(), operation) {
            Ok(_) => panic!("fault must not return a ruleset"),
            Err(failure) => failure,
        };
        assert_eq!(error.to_string(), "Forbidden-strings rule loading panicked; no rule set was accepted.");
        assert!(!error.to_string().contains("SYNTHETIC_PRIVATE_RULE_94a1"));
        assert!(PENDING.take().is_none());
        assert!(protected_request(request(), complete_request).is_ok());
    }
}

/// An existing request is never silently overwritten.
#[test]
fn occupied_request_is_restored_to_its_owner() {
    PENDING.set(Some(request()));
    assert!(protected_request(request(), complete_request).is_err());
    assert!(PENDING.take().is_some());
}

/// Named native-thread worker verifies that another thread's pending slot is inaccessible.
fn thread_worker() {
    assert!(PENDING.take().is_none());
    assert!(protected_request(request(), complete_request).is_ok());
}

/// Concurrent loads use separate slots without a process-global lock or hook swap.
#[test]
fn requests_remain_local_to_each_native_thread() {
    PENDING.set(Some(request()));
    let first = std::thread::spawn(thread_worker);
    let second = std::thread::spawn(thread_worker);
    first.join().expect("first load thread");
    second.join().expect("second load thread");
    assert!(PENDING.take().is_some());
}
