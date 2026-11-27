//! The D-Bus calls of the true-peak analysis indicator (D107),
//! Linux only.
//!
//! Two protocols carry the same indicator,
//! and this module holds nothing but their messages:
//!
//! - **Plasma job** (`org.kde.kuiserver`):
//!   `org.kde.JobViewServerV2.requestView` returns the path of an
//!   `org.kde.JobViewV3` object;
//!   `update` and `terminate` on it are `NoReply` calls,
//!   and the server answers a button
//!   with the `suspendRequested` and `resumeRequested` signals.
//!   Plasma shows the app's name and icon from the desktop
//!   entry,
//!   a determinate bar,
//!   and a Pause button when the job is suspendable.
//!   It never changes the button by
//!   itself:
//!   the app reports `suspended` in an `update`.
//!   Sources:
//!   `libnotificationmanager/dbus/org.kde.JobViewV3.xml`,
//!   `org.kde.JobViewServerV2.xml`,
//!   `jobsmodel_p.cpp` and `job_p.cpp` in `KDE/plasma-workspace`.
//! - **freedesktop notification** (`org.freedesktop.Notifications`):
//!   one `Notify` replaced in place by its returned
//!   id,
//!   with the de-facto `value` hint as the bar.
//!   Plasma's own server ignores that hint,
//!   which is why the job comes
//!   first on KDE.
//!
//! Every rule (what to show,
//! when to send) is in `analysis_notification_logic`;
//! the thread that calls these functions
//! is in `analysis_notification`.
//! The calls made on the two bus objects are in
//! `src/analysis_notification_wire_calls.rs`,
//! and the signal listener in `src/analysis_notification_wire_listener.rs`;
//! this file holds the names,
//! the message types and the payload builders.

/// Imports the map type that carries notification hints.
use std::collections::HashMap;

/// Imports the bus-daemon proxy through which the module asks the bus about a service.
use zbus::fdo::DBusProxy;
/// Imports the bus name type that addresses the services the module calls.
use zbus::names::BusName;
/// Imports the connection,
/// the proxy and the variant types of the D-Bus client.
use zbus::{
    Connection,
    Proxy,
    Variant,
};

/// Imports the identity that names the player to the notification service.
use crate::analysis_notification::Identity;
/// Imports the view and the outcome that the wire calls translate to and from bus messages.
use crate::analysis_notification_logic::{
    IndicatorView,
    Outcome,
};
/// Imports the time-bounded step that stops a bus call from blocking its thread for ever.
use crate::dbus_step_bound::step;
/// What:
/// `TITLE` is the indicator's title.
/// Why:
/// D107 names it;
/// American spelling is the human's choice for this label.
pub const TITLE: &str = "Analyzing true peak";

/// What:
/// `KUISERVER` is the bus name of Plasma's job server.
/// Why:
/// its presence on the bus selects the job indicator.
pub const KUISERVER: &str = "org.kde.kuiserver";

/// What:
/// `JOB_SERVER_PATH` is the object path of the job server.
/// Why:
/// `requestView` is called there.
pub const JOB_SERVER_PATH: &str = "/JobViewServer";

/// What:
/// `JOB_SERVER_INTERFACE` is the interface that hands out job views.
/// Why:
/// V2 takes a desktop entry and a hints dictionary,
/// which V1 does not.
pub const JOB_SERVER_INTERFACE: &str = "org.kde.JobViewServerV2";

/// What:
/// `JOB_VIEW_INTERFACE` is the interface of one job view.
/// Why:
/// V3 names its properties (`title`,
/// `infoMessage`,
/// `percent`,
/// `suspended`) instead of using positional setters.
pub const JOB_VIEW_INTERFACE: &str = "org.kde.JobViewV3";

/// What:
/// `CAPABILITY_SUSPENDABLE` is the `requestView` capability bit for a Pause button.
/// Why:
/// KJob::Suspendable is 2 and Plasma shows Pause and Resume for it;
/// Killable (1) is never offered because
/// D84 makes analysis non-optional.
pub const CAPABILITY_SUSPENDABLE: i32 = 2;

/// What:
/// `NOTIFICATIONS_NAME` is the bus name of the freedesktop notification service.
/// Why:
/// the fallback indicator is posted there.
pub const NOTIFICATIONS_NAME: &str = "org.freedesktop.Notifications";

/// What:
/// `NOTIFICATIONS_PATH` is the object path of the notification service.
/// Why:
/// `Notify` and `CloseNotification` are called there.
pub const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

/// What:
/// `NOTIFICATIONS_INTERFACE` is the notification interface.
/// Why:
/// its methods and signals are used.
pub const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

/// What:
/// `ACTION_PAUSE` is the action key of the fallback's Pause button.
/// Why:
/// `ActionInvoked` reports this key back.
pub const ACTION_PAUSE: &str = "pause";

/// What:
/// `ACTION_RESUME` is the action key of the fallback's Resume button.
/// Why:
/// the same button swaps its key and label while paused.
pub const ACTION_RESUME: &str = "resume";

/// What:
/// `CLOSED_BY_EXPIRY` is the `NotificationClosed` reason for an expired notification.
/// Why:
/// the spec's reason 1;
/// with an expiry of zero a server should not send it,
/// but it also means the user lost it.
pub const CLOSED_BY_EXPIRY: u32 = 1;

/// What:
/// `CLOSED_BY_USER` is the `NotificationClosed` reason for a dismissal.
/// Why:
/// the spec's reason 2.
pub const CLOSED_BY_USER: u32 = 2;

/// What:
/// `JOB_ERROR_NONE` is the `terminate` code for success.
/// Why:
/// Plasma shows a finished job;
/// with the `transient` hint it removes it without notice.
const JOB_ERROR_NONE: u32 = 0;

/// What:
/// `JOB_ERROR_KILLED` is the `terminate` code for a job that was stopped.
/// Why:
/// KJob::KilledJobError (KIO::ERR_USER_CANCELED);
/// Plasma removes it without notice.
const JOB_ERROR_KILLED: u32 = 1;

/// What:
/// `JOB_ERROR_USER` is the `terminate` code for a failure with a message.
/// Why:
/// KJob::UserDefinedError;
/// Plasma shows the message as a failed job.
const JOB_ERROR_USER: u32 = 2;

/// What:
/// `BusSignal` is one signal the indicator listens for,
/// with who sent it.
/// Why:
/// the thread matches the sender and the path or id against its own job or notification before acting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BusSignal {
    /// The job's Pause button was pressed.
    JobSuspend {
        /// Unique name of the bus peer that sent the signal.
        sender: String,
        /// Object path of the job that was suspended.
        path: String,
    },
    /// The job's Resume button was pressed.
    JobResume {
        /// Unique name of the bus peer that sent the signal.
        sender: String,
        /// Object path of the job that was resumed.
        path: String,
    },
    /// A notification action button was pressed.
    ActionInvoked {
        /// Unique name of the bus peer that sent the signal.
        sender: String,
        /// Identifier of the notification whose button was pressed.
        id: u32,
        /// Key of the action that was pressed.
        key: String,
    },
    /// A notification closed for `reason`.
    NotificationClosed {
        /// Unique name of the bus peer that sent the signal.
        sender: String,
        /// Identifier of the notification that closed.
        id: u32,
        /// Reason code that the service gave.
        reason: u32,
    },
}

/// What:
/// `Backend` names the two protocols.
/// Why:
/// the thread keeps the one that opened,
/// and the listener is started once per protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// A Plasma job view.
    Job,
    /// A freedesktop notification.
    Notification,
}

/// What:
/// `JobView` is an open Plasma job.
/// Why:
/// it keeps the proxy of the view object,
/// its path,
/// and the unique name of the server that owns it.
#[derive(Debug)]
pub struct JobView {
    /// Proxy of the job object on the bus.
    proxy: Proxy<'static>,
    /// Object path of the job on the bus.
    path: String,
    /// Unique name of the server that owns the job,
    /// which signals are checked against.
    owner: String,
}

/// What:
/// `FdoNotification` is an open freedesktop notification.
/// Why:
/// it keeps the proxy of the service,
/// the id every replacement reuses,
/// and the service's unique name.
#[derive(Debug)]
pub struct FdoNotification {
    /// Proxy of the notification service.
    proxy: Proxy<'static>,
    /// Identifier that the service gave the notification,
    /// which later calls and signals use.
    id: u32,
    /// Unique name of the service,
    /// which signals are checked against.
    owner: String,
}

/// What:
/// `job_request_hints` builds the `requestView` hints.
/// Why:
/// `application-display-name` and `application-icon-name` give the header its name and icon even when the
/// desktop file is not installed (a dev run),
/// and `transient` makes Plasma remove a finished job without notice.
pub fn job_request_hints(identity: &Identity) -> HashMap<&'static str, Variant<'_>> {
    return HashMap::from([
        (
            "application-display-name",
            Variant::from(identity.app_name.as_str()),
        ),
        (
            "application-icon-name",
            Variant::from(identity.app_icon.as_str()),
        ),
        ("transient", Variant::from(true)),
    ]);
}

/// What:
/// `job_update_properties` builds an `update` dictionary for `view`.
/// Why:
/// `percent` is the bar,
/// `infoMessage` the count,
/// and `suspended` flips Plasma's Pause and Resume button,
/// which
/// Plasma never does itself.
/// The title is sent only at the open.
pub fn job_update_properties(
    view: &IndicatorView,
    with_title: bool,
) -> HashMap<&'static str, Variant<'_>> {
    let mut properties = HashMap::from([
        ("infoMessage", Variant::from(view.text.as_str())),
        ("percent", Variant::from(view.percent)),
        ("suspended", Variant::from(view.paused)),
    ]);
    if with_title {
        properties.insert("title", Variant::from(TITLE));
    }
    return properties;
}

/// What:
/// `terminate_arguments` maps an outcome to the `terminate` error code and message.
/// Why:
/// success and a stop show nothing;
/// only a failure carries text.
pub fn terminate_arguments(outcome: &Outcome) -> (u32, String) {
    return match outcome {
        Outcome::Finished => (JOB_ERROR_NONE, String::new()),
        Outcome::Stopped => (JOB_ERROR_KILLED, String::new()),
        Outcome::Failed(message) => (JOB_ERROR_USER, message.clone()),
    };
}

/// What:
/// `notification_hints` builds the `Notify` hints for `view`.
/// Why:
/// `value` draws the bar on servers that support it;
/// urgency 0 (a byte) is low;
/// `suppress-sound` keeps it silent;
/// `category` is the spec's generic `transfer` because it defines no progress category;
/// `desktop-entry` names the app;
/// `resident` keeps the notification when its action button is pressed,
/// which the spec says removes it otherwise.
pub fn notification_hints<'a>(
    identity: &'a Identity,
    view: &IndicatorView,
) -> HashMap<&'static str, Variant<'a>> {
    let percent = i32::try_from(view.percent).unwrap_or(i32::MAX);
    return HashMap::from([
        ("value", Variant::I32(percent)),
        ("urgency", Variant::U8(0)),
        ("suppress-sound", Variant::Bool(true)),
        ("category", Variant::from("transfer")),
        (
            "desktop-entry",
            Variant::from(identity.desktop_entry.as_str()),
        ),
        ("resident", Variant::Bool(true)),
    ]);
}

/// What:
/// `notification_actions` lists the action key and label pairs for `view`.
/// Why:
/// a suspendable indicator offers Pause,
/// or Resume while paused;
/// otherwise there is no button.
pub fn notification_actions(
    suspendable: bool,
    view: &IndicatorView,
) -> Vec<&'static str> {
    if !suspendable {
        return Vec::new();
    }
    if view.paused {
        return vec![ACTION_RESUME, "Resume"];
    }
    return vec![ACTION_PAUSE, "Pause"];
}

/// What:
/// `has_owner` asks the bus whether `name` is owned,
/// without starting its service.
/// Why:
/// a plain method call to an activatable name could start a stray process;
/// this only reads the bus.
pub fn has_owner(
    connection: &Connection,
    name: &str,
) -> zbus::Result<bool> {
    let bus = step(DBusProxy::new(connection))?;
    let bus_name = BusName::try_from(name)?;
    return Ok(step(async {
        return Ok(bus.name_has_owner(bus_name).await?);
    })?);
}

/// What:
/// `owner_of` returns the unique name that owns `name`.
/// Why:
/// signals are trusted only from the process that answered the request.
fn owner_of(
    connection: &Connection,
    name: &str,
) -> zbus::Result<String> {
    let bus = step(DBusProxy::new(connection))?;
    let bus_name = BusName::try_from(name)?;
    return Ok(step(async { return Ok(bus.get_name_owner(bus_name).await?) })?.to_string());
}
/// What:
///      `mod analysis_notification_wire_calls;` loads the sibling
///           `src/analysis_notification_wire_calls.rs`,
///  the calls made on the job view and on the notification.
/// Why:
///       The calls are one proxy per object and belong to no message shape,
///           so they keep a file of their own while this module stays within the line
///           budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as analysisNotificationWireCalls from "./analysis_notification_wire_calls";
/// ```
#[path = "analysis_notification_wire_calls.rs"]
mod analysis_notification_wire_calls;

/// What:
///      `mod analysis_notification_wire_listener;` loads the sibling
///           `src/analysis_notification_wire_listener.rs`,
///  the signal decoding and the subscription thread.
/// Why:
///       Listening has a thread and a queue of its own,
///  and nothing in it decides what the indicator shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as analysisNotificationWireListener from "./analysis_notification_wire_listener";
/// ```
#[path = "analysis_notification_wire_listener.rs"]
mod analysis_notification_wire_listener;

/// What:
///      `pub use analysis_notification_wire_listener::spawn_listener;` re-exports
///           the listener's entry point at this module's path.
/// Why:
///       Callers name `analysis_notification_wire::spawn_listener`,
///  and only where it is declared moved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { spawn_listener } from "./analysis_notification_wire_listener";
/// ```
pub use analysis_notification_wire_listener::spawn_listener;

/// Unit tests in `analysis_notification_wire_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "analysis_notification_wire_tests.rs"]
mod tests;
