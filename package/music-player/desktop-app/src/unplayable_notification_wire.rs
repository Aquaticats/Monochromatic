//! The D-Bus call of the "cannot be played" notification (D122), Linux only.
//!
//! One `org.freedesktop.Notifications.Notify` call posts the notification; it is never replaced, never given an action
//! button and never closed by the app.
//! The service name, path and interface are those the analysis indicator already uses.
//! What to say is in `unplayable_notification_logic`, and the thread that calls this is in `unplayable_notification`.

use std::collections::HashMap;

use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::Value;

use crate::analysis_notification::Identity;
use crate::analysis_notification_wire::{NOTIFICATIONS_INTERFACE, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH};
use crate::unplayable_notification_logic::Notice;

/// What: `URGENCY_NORMAL` is the `urgency` hint value of a normal notification.
/// Why: the spec's levels are 0 low, 1 normal and 2 critical; a file that cannot be played is not critical and, unlike
/// the progress indicator, not low.
pub const URGENCY_NORMAL: u8 = 1;

/// What: `EXPIRY_SERVER_DEFAULT` is the `expire_timeout` that lets the notification service choose.
/// Why: the spec's -1 means "the server's default"; 0 would keep the notification until dismissed, which the analysis
/// indicator wants and this one does not.
pub const EXPIRY_SERVER_DEFAULT: i32 = -1;

/// What: `notice_hints` builds the `Notify` hints.
/// Why: the urgency is a byte (a wider integer is a silent type error on the service), and `desktop-entry` ties the
/// notification to the app's `.desktop` file so the desktop shows its name and icon and groups it with the app.
pub fn notice_hints(identity: &Identity) -> HashMap<&'static str, Value<'_>> {
    return HashMap::from([
        ("urgency", Value::U8(URGENCY_NORMAL)),
        ("desktop-entry", Value::from(identity.desktop_entry.as_str())),
    ]);
}

/// What: `post` sends `notice` as one new notification and returns the id the service gave it.
/// Why: `replaces_id` is 0 because nothing is replaced, the action list is empty because there is nothing to press,
/// and the expiry is the service's default; the app name and icon are those of the app's identity.
pub fn post(connection: &Connection, identity: &Identity, notice: &Notice) -> zbus::Result<u32> {
    let proxy = Proxy::new(connection, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE)?;
    let actions: Vec<&str> = Vec::new();
    let id: u32 = proxy.call(
        "Notify",
        &(
            identity.app_name.as_str(),
            0_u32,
            identity.app_icon.as_str(),
            notice.summary,
            notice.body.as_str(),
            actions,
            notice_hints(identity),
            EXPIRY_SERVER_DEFAULT,
        ),
    )?;
    tracing::info!(id, "unplayable-file notification posted");
    return Ok(id);
}

/// Unit tests in `unplayable_notification_wire_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "unplayable_notification_wire_tests.rs"]
mod tests;
