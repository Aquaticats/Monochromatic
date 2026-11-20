//! What:
//!  The device-free proof of the fixture's own rules:
//! a host run is refused with a
//!  line naming the reason,
//! and the source strings `zfs get` prints map to the
//!  ones the kernel writes.
//! Why:
//!  The gate that keeps a live request away from this host is the one thing
//!  standing between a test run and the host's own pool,
//! so it is proven without
//!  a device,
//! and the source mapping is what the property test compares
//!  against.

use super::fixture::{Fixture, GUEST_VARIABLE, hostname};
use super::fixture_reply::expected_source;

#[test]
fn the_fixture_runs_only_in_the_guest_the_environment_names() {
    let named: Option<String> = std::env::var(GUEST_VARIABLE).ok();
    let fixture: Option<Fixture> = Fixture::from_environment("fixture_tests::the_gate");
    match named {
        None => assert!(
            fixture.is_none(),
            "an unset guest variable must refuse the fixture"
        ),
        Some(guest) => assert_eq!(
            fixture.is_some(),
            hostname() == guest,
            "the fixture runs exactly when this host is the one the variable names"
        ),
    };
}

#[test]
fn a_local_source_names_the_dataset_the_property_was_read_from() {
    assert_eq!(
        expected_source("local", "pool/home/checkout"),
        "pool/home/checkout",
        "a local value's source is the dataset it was read from"
    );
}

#[test]
fn an_inherited_source_names_the_dataset_the_value_was_set_on() {
    assert_eq!(expected_source("inherited from pool", "pool/home"), "pool");
}

#[test]
fn a_received_source_stays_the_word_the_kernel_keeps() {
    assert_eq!(expected_source("received", "pool/home"), "received");
}

#[test]
fn a_default_source_stays_the_word_the_kernel_keeps() {
    assert_eq!(expected_source("default", "pool/home"), "default");
}
