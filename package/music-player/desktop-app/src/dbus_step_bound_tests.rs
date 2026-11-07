//! Tests for the bound in `dbus_step_bound.rs`, pulled in by
//! `#[cfg(all(test, target_os = "linux"))] #[path = "dbus_step_bound_tests.rs"] mod tests;` at the bottom of that file.
//! They run against a private bus with a recording notification service; the user's session bus is never used.

use std::io::ErrorKind;
use std::time::{Duration, Instant};

use super::*;
use crate::unplayable_notification_bus_support::PrivateBus;

/// A short bound for the tests that must see a step end, so each takes a fraction of a second.
const SHORT_BOUND: Duration = Duration::from_millis(300);

/// How many times a short bound the tests allow before calling an ended step late.
const LATE_FACTOR: u32 = 10;

/// The id the recording service gives its first notification.
const FIRST_ID: u32 = 1;

/// Calls `Notify` on `connection` with the smallest valid argument list and returns the id.
fn notify(connection: &Connection) -> zbus::Result<u32> {
    let proxy = step(zbus::Proxy::new(
        connection,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    ))?;
    let actions: Vec<&str> = Vec::new();
    let hints: std::collections::HashMap<&str, zbus::Variant<'_>> = std::collections::HashMap::new();
    return zbus::block_on(proxy.call("Notify", &("App", 0_u32, "icon", "Summary", "Body", actions, hints, -1_i32)));
}

#[test]
fn a_pending_step_ends_with_a_timed_out_error_at_the_bound() {
    let started = Instant::now();
    let outcome = zbus::block_on(within(SHORT_BOUND, std::future::pending::<zbus::Result<()>>()));
    let elapsed = started.elapsed();
    let Err(zbus::Error::InputOutput(error)) = outcome else {
        panic!("expected a timed-out I/O error, got {outcome:?}");
    };
    assert_eq!(error.kind(), ErrorKind::TimedOut);
    assert!(elapsed >= SHORT_BOUND && elapsed < SHORT_BOUND * LATE_FACTOR, "ended at {elapsed:?}");
}

#[test]
fn a_finished_step_returns_its_value_and_a_failed_step_its_own_error() {
    let value = zbus::block_on(within(SHORT_BOUND, async { return Ok::<u32, zbus::Error>(FIRST_ID) }));
    assert_eq!(value.expect("an immediate step succeeds"), FIRST_ID);
    let failure = zbus::block_on(within(SHORT_BOUND, async { return Err::<(), zbus::Error>(zbus::Error::Failure("boom".to_string())) }));
    assert!(matches!(failure, Err(zbus::Error::Failure(ref text)) if text == "boom"), "{failure:?}");
}

#[test]
fn a_call_to_a_wedged_service_ends_with_timed_out_at_the_connection_bound() {
    let bus = PrivateBus::start("step_wedged");
    let (_service, _calls) = bus.serve_notifications(SHORT_BOUND * LATE_FACTOR);
    let wedged_client = bus.connect(SHORT_BOUND).expect("the client connects");
    assert_eq!(wedged_client.method_timeout(), Some(SHORT_BOUND));
    let started = Instant::now();
    let outcome = notify(&wedged_client);
    let elapsed = started.elapsed();
    let Err(zbus::Error::InputOutput(error)) = outcome else {
        panic!("expected a timed-out I/O error, got {outcome:?}");
    };
    assert_eq!(error.kind(), ErrorKind::TimedOut);
    assert!(elapsed >= SHORT_BOUND && elapsed < SHORT_BOUND * LATE_FACTOR, "ended at {elapsed:?}");
}

#[test]
fn a_connect_and_a_call_on_a_private_bus_finish_far_inside_the_bound() {
    let bus = PrivateBus::start("step_times");
    let (_service, _calls) = bus.serve_notifications(Duration::ZERO);
    let connect_started = Instant::now();
    let client = connect(zbus::connection::Builder::address(bus.address.as_str()), DBUS_STEP_BOUND).expect("the client connects");
    let connect_time = connect_started.elapsed();
    assert_eq!(client.method_timeout(), Some(DBUS_STEP_BOUND));
    let call_started = Instant::now();
    let id = notify(&client).expect("the recording service answers");
    let call_time = call_started.elapsed();
    assert_eq!(id, FIRST_ID);
    println!("dbus step times: connect {connect_time:?}, one Notify call {call_time:?}");
    for (name, elapsed) in [("connect", connect_time), ("call", call_time)] {
        assert!(elapsed < DBUS_STEP_BOUND / LATE_FACTOR, "{name} took {elapsed:?} against a bound of {DBUS_STEP_BOUND:?}");
    }
}
