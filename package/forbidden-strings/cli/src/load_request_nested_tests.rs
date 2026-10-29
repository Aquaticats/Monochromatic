//! What:
//!  Nested requests and partial-construction faults across the named load boundary.
//! Why:
//!  A compiler callback may re-enter loading;
//!  neither nesting nor panic may overwrite a pending owner's state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Consume outer arguments, execute an inner request, and recover independently from either failure.
//! ```

/// What:
///  Import the parent module's private request slot and guarded operation.
/// Why:
///  Test the real ownership boundary without publishing test-only panic controls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { protectedRequest, pending } from './load-request';
/// ```
use super::{LoadRequest, PENDING, protected_request};
/// Import the existing compiled-rule fixture and owned native filename.
use crate::frx_load::LoadedRules;
use std::path::PathBuf;

/// What:
///  Own request arguments,
///  using PathBuf rather than borrowed Path or UTF-8-only String.
/// Why:
///  The callback consumes native filename ownership independently of any caller lifetime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function request(name: string): LoadRequest { return { path: name, builtin: false, explicit: true }; }
/// ```
fn request(name: &str) -> LoadRequest {
    // Copy the borrowed name into an owned native path; return the request explicitly.
    return LoadRequest { path: PathBuf::from(name), builtin: false, explicit: true };
}

/// Complete the inner request after proving it retained its own arguments.
fn inner() -> anyhow::Result<LoadedRules> {
    // expect extracts the present request, failing the test if the boundary lost it.
    let current: LoadRequest = PENDING.take().expect("inner request");
    // Borrow both paths for equality without consuming their ownership.
    assert_eq!(current.path, PathBuf::from("inner"));
    // Ok wraps the compiled fixture as success, rather than returning an error.
    return Ok(crate::frx_load::test_rules("NESTED_FIXTURE_LONG\n"));
}

/// Consume outer arguments before entering an inner guarded request.
fn outer() -> anyhow::Result<LoadedRules> {
    // Taking the slot moves its value out and leaves it empty, so the inner load can enter.
    let current: LoadRequest = PENDING.take().expect("outer request");
    assert_eq!(current.path, PathBuf::from("outer"));
    // Propagate the nested ruleset, not a substitute or partial outer value.
    return protected_request(request("inner"), inner);
}

/// Construct a real ruleset,
///  then fault before accepting it.
fn panic_after_construction() -> anyhow::Result<LoadedRules> {
    // Consume arguments exactly as the production callback does before compilation.
    let _current: LoadRequest = PENDING.take().expect("partial request");
    // The local ruleset must be dropped by unwinding, never returned as success.
    let _partial: LoadedRules = crate::frx_load::test_rules("PARTIAL_FIXTURE_LONG\n");
    panic!("SYNTHETIC_PARTIAL_LOAD_PAYLOAD");
}

/// Nested success leaves no pending outer or inner state.
#[test]
fn nested_loads_consume_their_own_requests() {
    // Inspect the guarded result without extracting a partial ruleset on failure.
    assert!(protected_request(request("outer"), outer).is_ok());
    assert!(PENDING.take().is_none());
}

/// Partial construction is never accepted,
///  and the same thread can subsequently load again.
#[test]
fn constructed_rules_are_discarded_after_a_panic() {
    // match distinguishes failure from an accidentally returned partial compiled set.
    let error = match protected_request(request("partial"), panic_after_construction) {
        Ok(_) => panic!("partial rules must not be accepted"),
        Err(failure) => failure,
    };
    // to_string owns the redacted diagnostic independently of the caught payload.
    assert_eq!(error.to_string(), "Forbidden-strings rule loading panicked; no rule set was accepted.");
    assert!(protected_request(request("outer"), outer).is_ok());
    assert!(PENDING.take().is_none());
}

/// Rejecting a second request restores the first request's exact identity and flags.
#[test]
fn occupied_slot_restores_original_arguments() {
    // Some marks present arguments, rather than the empty None state.
    PENDING.set(Some(LoadRequest { path: PathBuf::from("original"), builtin: true, explicit: false }));
    assert!(protected_request(request("replacement"), inner).is_err());
    // Unwrap only test fixture state; this test must fail if restoration discarded it.
    let restored: LoadRequest = PENDING.take().expect("restored request");
    assert_eq!(restored.path, PathBuf::from("original"));
    assert!(restored.builtin);
    assert!(!restored.explicit);
}
