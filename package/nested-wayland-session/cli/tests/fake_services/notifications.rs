//! The fake `org.freedesktop.Notifications` server,
//!  driven by a real client on a real private bus.

use std::collections::HashMap;

use super::support::{
    Fixture,
    proxy,
};

use nested_wayland_session::bus_test_client::Proxy;

use nested_wayland_session::{
    control_notification::run,
    notification_service::{
        NOTIFICATIONS_INTERFACE,
        NOTIFICATIONS_NAME,
        NOTIFICATIONS_PATH,
    },
    protocol::NotificationCommand,
};

use zbus::Variant;

/// What the list and log answers carry.
mod what_the_list_shows;

/// What a client asks the server to do,
///  and the signals those requests raise.
mod client_requests_and_signals;

/// The mode that serves notifications alone.
mod the_freedesktop_mode;

/// Opens the client's proxy to the fake service by its well-known name.
fn notifications_proxy(fixture: &Fixture) -> anyhow::Result<Proxy<'static>> {
    return proxy(
        &fixture.client,
        NOTIFICATIONS_NAME,
        NOTIFICATIONS_PATH,
        NOTIFICATIONS_INTERFACE,
    );
}

/// Calls `Notify` with every argument spelled out.
fn notify(
    client: &Proxy<'static>,
    replaces_id: u32,
    summary: &str,
    actions: &[&str],
    hints: &HashMap<&str, Variant<'_>>,
) -> anyhow::Result<u32> {
    return Ok(client.call(
        "Notify",
        &(
            "Example App",
            replaces_id,
            "dialog-information",
            summary,
            "Body text",
            actions,
            hints,
            5000_i32,
        ),
    )?);
}

/// Runs a notification verb against the fixture's services.
fn verb(
    fixture: &Fixture,
    command: &NotificationCommand,
) -> nested_wayland_session::protocol::Response {
    return run(Some(&fixture.services), command);
}

/// Splits a `<count> <record>; <record>` payload into its records.
fn records(payload: &str) -> Vec<String> {
    let rest = payload.split_once(' ').map_or("", |(_, tail)| return tail);
    if rest.is_empty() {
        return Vec::new();
    }
    return rest.split("; ").map(str::to_owned).collect();
}
