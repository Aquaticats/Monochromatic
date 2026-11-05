//! Controls for the guarded call: a return is a value, a panic is an error carrying its payload.

use super::*;

/// A call that returns gives back what it returned, computed from its state.
#[test]
fn a_call_that_returns_gives_back_its_value() {
    fn double(state: u32) -> u32 {
        return state * 2;
    }
    let value: u32 = call_catching_panics(21, double).expect("the call returned");
    assert_eq!(value, 42);
}

/// A call that panics gives back the payload of its panic instead of unwinding past the caller.
#[test]
fn a_call_that_panics_gives_back_the_payload() {
    fn refuse(state: &str) -> u32 {
        panic!("{state} failed");
    }
    let payload = call_catching_panics("the call", refuse).expect_err("the call panicked");
    assert_eq!(payload.downcast_ref::<String>().map(String::as_str), Some("the call failed"));
}
