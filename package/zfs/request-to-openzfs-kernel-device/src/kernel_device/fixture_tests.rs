//! What:
//!  The device-free proof of the fixture's own rules:
//! which environment gets a
//!  fixture,
//! which is refused and how loudly,
//! and the source strings `zfs get` prints
//!  mapped to the ones the kernel writes.
//! Why:
//!  The gate that keeps a live request away from this host is the one thing
//!  standing between a test run and the host's own pool,
//! so it is proven without
//!  a device;
//! and a guest run that could not build its fixture used to return,
//!  which a test runner counts as a pass,
//! so the refusals are proven to panic instead.

use super::fixture::{Fixture, GUEST_VARIABLE, resolve};
use super::fixture_reply::expected_source;
use std::ffi::OsString;

/// The message a refused fixture panicked with.
fn refusal(result: Result<Option<Fixture>, Box<dyn std::any::Any + Send>>) -> String {
    let payload = match result {
        Err(payload) => payload,
        Ok(_) => panic!("the fixture is refused"),
    };
    return payload
        .downcast_ref::<String>()
        .cloned()
        .expect("a formatted refusal panics with a string");
}

/// An environment lookup that finds only the names it is given.
fn variables(names: &'static [&'static str]) -> impl Fn(&str) -> Option<OsString> {
    return move |name: &str| {
        return if names.contains(&name) {
            Some(OsString::from(format!("{name}-value")))
        } else {
            None
        };
    };
}

#[test]
fn a_host_run_gets_no_fixture() {
    assert!(
        resolve(
            "fixture_tests::a_host_run",
            None,
            "any-machine".to_string(),
            variables(&[]),
        )
        .is_none(),
        "an unset guest variable must refuse the fixture"
    );
}

#[test]
fn a_named_guest_that_is_another_machine_is_refused() {
    let message = refusal(std::panic::catch_unwind(|| {
        return resolve(
            "fixture_tests::another_machine",
            Some(OsString::from("the-named-guest")),
            "another-machine".to_string(),
            variables(&[]),
        );
    }));
    assert!(
        message.contains("another-machine") && message.contains("the-named-guest"),
        "the refusal names both machines: {message}"
    );
}

#[test]
fn a_named_guest_without_the_rest_of_the_environment_is_refused() {
    let message = refusal(std::panic::catch_unwind(|| {
        return resolve(
            "fixture_tests::an_incomplete_environment",
            Some(OsString::from("the-guest")),
            "the-guest".to_string(),
            variables(&[]),
        );
    }));
    assert!(
        message.contains("MONOCHROMATIC_ZFS_FIXTURE_SBIN")
            && message.contains("MONOCHROMATIC_ZFS_FIXTURE_ROOT")
            && message.contains("MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT")
            && message.contains("MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL"),
        "the refusal names every unset variable: {message}"
    );
    assert!(
        message.contains("the-guest") && message.contains(GUEST_VARIABLE),
        "the refusal names the guest it was meant for: {message}"
    );
}

#[test]
fn a_named_guest_on_its_own_machine_gets_the_fixture() {
    let fixture: Fixture = resolve(
        "fixture_tests::the_guest",
        Some(OsString::from("the-guest")),
        "the-guest".to_string(),
        variables(&[
            "MONOCHROMATIC_ZFS_FIXTURE_SBIN",
            "MONOCHROMATIC_ZFS_FIXTURE_ROOT",
            "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT",
            "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL",
        ]),
    )
    .expect("the named guest with its whole environment gets the fixture");
    assert_eq!(
        fixture.sbin,
        std::path::PathBuf::from("MONOCHROMATIC_ZFS_FIXTURE_SBIN-value")
    );
    assert_eq!(
        fixture.root,
        std::path::PathBuf::from("MONOCHROMATIC_ZFS_FIXTURE_ROOT-value")
    );
    assert_eq!(fixture.account, "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT-value");
    assert_eq!(
        fixture.account_pool,
        "MONOCHROMATIC_ZFS_FIXTURE_ACCOUNT_POOL-value"
    );
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
