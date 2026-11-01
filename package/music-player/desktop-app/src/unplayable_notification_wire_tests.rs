//! Tests for the message builder and the `Notify` call in `unplayable_notification_wire.rs`, pulled in by
//! `#[cfg(test)] #[path = "unplayable_notification_wire_tests.rs"] mod tests;` at the bottom of that file.
//! The call is made against a private bus with a recording service; the user's session bus is never used.

use std::time::Duration;

use super::*;
use crate::unplayable_notification_bus_support::{Call, PrivateBus};
use crate::unplayable_notification_logic::{Ledger, SUMMARY};

fn identity() -> Identity {
    return Identity {
        desktop_entry: "monochromatic.music-player".to_string(),
        app_name: "Music Player".to_string(),
        app_icon: "multimedia-player".to_string(),
    };
}

fn notice(body: &str) -> Notice {
    return Notice { summary: SUMMARY, body: body.to_string() };
}

#[test]
fn the_hints_are_a_normal_urgency_byte_and_the_desktop_entry_and_nothing_else() {
    let who = identity();
    let hints = notice_hints(&who);
    assert_eq!(hints.len(), 2);
    assert_eq!(hints.get("urgency"), Some(&Value::U8(1)));
    assert_eq!(hints.get("desktop-entry"), Some(&Value::from("monochromatic.music-player")));
}

#[test]
fn the_urgency_is_normal_and_the_expiry_is_the_servers_default() {
    assert_eq!(URGENCY_NORMAL, 1, "0 is low and 2 is critical in the specification");
    assert_eq!(EXPIRY_SERVER_DEFAULT, -1, "0 would never expire and a positive value is a fixed time");
}

#[test]
fn one_notify_call_carries_the_identity_the_fixed_title_the_message_and_the_decided_hints() {
    let bus = PrivateBus::start("wire");
    let (_service, calls) = bus.serve_notifications(Duration::ZERO);
    let client = bus.connect(Duration::from_secs(3)).expect("the client connects");
    let report = [std::path::PathBuf::from("/invented/library/Example Track.flac")];
    let made = Ledger::new().admit(&report).expect("a new file makes a notice");
    let id = post(&client, &identity(), &made).expect("the recording service answers");
    assert_eq!(id, 1, "the id is the one the service returned");
    let seen = calls.lock().expect("not poisoned").clone();
    assert_eq!(
        seen,
        vec![Call {
            app_name: "Music Player".to_string(),
            replaces_id: 0,
            app_icon: "multimedia-player".to_string(),
            summary: "Cannot play a file".to_string(),
            body: "Example Track.flac can't be played.".to_string(),
            actions: Vec::new(),
            hint_names: vec!["desktop-entry".to_string(), "urgency".to_string()],
            urgency: Some(1),
            desktop_entry: Some("monochromatic.music-player".to_string()),
            expire_timeout: -1,
        }]
    );
}

#[test]
fn a_bus_with_no_notification_service_is_an_error_not_a_hang() {
    let bus = PrivateBus::start("wire_none");
    let client = bus.connect(Duration::from_secs(3)).expect("the client connects");
    let failed = post(&client, &identity(), &notice("x.wav can't be played."));
    assert!(failed.is_err(), "no service owns the name, so the call fails: {failed:?}");
}
