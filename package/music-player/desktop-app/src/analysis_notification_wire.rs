//! The D-Bus calls of the true-peak analysis indicator (D107), Linux only.
//!
//! Two protocols carry the same indicator, and this module holds nothing but their messages:
//!
//! - **Plasma job** (`org.kde.kuiserver`): `org.kde.JobViewServerV2.requestView` returns the path of an
//!   `org.kde.JobViewV3` object; `update` and `terminate` on it are `NoReply` calls, and the server answers a button
//!   with the `suspendRequested` and `resumeRequested` signals. Plasma shows the app's name and icon from the desktop
//!   entry, a determinate bar, and a Pause button when the job is suspendable. It never changes the button by
//!   itself: the app reports `suspended` in an `update`. Sources: `libnotificationmanager/dbus/org.kde.JobViewV3.xml`,
//!   `org.kde.JobViewServerV2.xml`, `jobsmodel_p.cpp` and `job_p.cpp` in `KDE/plasma-workspace`.
//! - **freedesktop notification** (`org.freedesktop.Notifications`): one `Notify` replaced in place by its returned
//!   id, with the de-facto `value` hint as the bar. Plasma's own server ignores that hint, which is why the job comes
//!   first on KDE.
//!
//! Every rule (what to show, when to send) is in `analysis_notification_logic`; the thread that calls these functions
//! is in `analysis_notification`.

use std::collections::HashMap;
use std::thread::JoinHandle;

use zbus::blocking::fdo::DBusProxy;
use zbus::blocking::{Connection, MessageIterator, Proxy};
use zbus::match_rule::MatchRule;
use zbus::message::Type;
use zbus::names::BusName;
use zbus::zvariant::{OwnedObjectPath, Value};

use crate::analysis_notification::Identity;
use crate::analysis_notification_logic::{IndicatorView, Outcome};

/// What: `TITLE` is the indicator's title.
/// Why: D107 names it; American spelling is the human's choice for this label.
pub const TITLE: &str = "Analyzing true peak";

/// What: `KUISERVER` is the bus name of Plasma's job server.
/// Why: its presence on the bus selects the job indicator.
pub const KUISERVER: &str = "org.kde.kuiserver";

/// What: `JOB_SERVER_PATH` is the object path of the job server.
/// Why: `requestView` is called there.
pub const JOB_SERVER_PATH: &str = "/JobViewServer";

/// What: `JOB_SERVER_INTERFACE` is the interface that hands out job views.
/// Why: V2 takes a desktop entry and a hints dictionary, which V1 does not.
pub const JOB_SERVER_INTERFACE: &str = "org.kde.JobViewServerV2";

/// What: `JOB_VIEW_INTERFACE` is the interface of one job view.
/// Why: V3 names its properties (`title`, `infoMessage`, `percent`, `suspended`) instead of using positional setters.
pub const JOB_VIEW_INTERFACE: &str = "org.kde.JobViewV3";

/// What: `CAPABILITY_SUSPENDABLE` is the `requestView` capability bit for a Pause button.
/// Why: KJob::Suspendable is 2 and Plasma shows Pause and Resume for it; Killable (1) is never offered because
/// D84 makes analysis non-optional.
pub const CAPABILITY_SUSPENDABLE: i32 = 2;

/// What: `NOTIFICATIONS_NAME` is the bus name of the freedesktop notification service.
/// Why: the fallback indicator is posted there.
pub const NOTIFICATIONS_NAME: &str = "org.freedesktop.Notifications";

/// What: `NOTIFICATIONS_PATH` is the object path of the notification service.
/// Why: `Notify` and `CloseNotification` are called there.
pub const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

/// What: `NOTIFICATIONS_INTERFACE` is the notification interface.
/// Why: its methods and signals are used.
pub const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

/// What: `ACTION_PAUSE` is the action key of the fallback's Pause button.
/// Why: `ActionInvoked` reports this key back.
pub const ACTION_PAUSE: &str = "pause";

/// What: `ACTION_RESUME` is the action key of the fallback's Resume button.
/// Why: the same button swaps its key and label while paused.
pub const ACTION_RESUME: &str = "resume";

/// What: `CLOSED_BY_EXPIRY` is the `NotificationClosed` reason for an expired notification.
/// Why: the spec's reason 1; with an expiry of zero a server should not send it, but it also means the user lost it.
pub const CLOSED_BY_EXPIRY: u32 = 1;

/// What: `CLOSED_BY_USER` is the `NotificationClosed` reason for a dismissal.
/// Why: the spec's reason 2.
pub const CLOSED_BY_USER: u32 = 2;

/// What: `JOB_ERROR_NONE` is the `terminate` code for success.
/// Why: Plasma shows a finished job; with the `transient` hint it removes it without notice.
const JOB_ERROR_NONE: u32 = 0;

/// What: `JOB_ERROR_KILLED` is the `terminate` code for a job that was stopped.
/// Why: KJob::KilledJobError (KIO::ERR_USER_CANCELED); Plasma removes it without notice.
const JOB_ERROR_KILLED: u32 = 1;

/// What: `JOB_ERROR_USER` is the `terminate` code for a failure with a message.
/// Why: KJob::UserDefinedError; Plasma shows the message as a failed job.
const JOB_ERROR_USER: u32 = 2;

/// What: `BusSignal` is one signal the indicator listens for, with who sent it.
/// Why: the thread matches the sender and the path or id against its own job or notification before acting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BusSignal {
    /// The job's Pause button was pressed.
    JobSuspend { sender: String, path: String },
    /// The job's Resume button was pressed.
    JobResume { sender: String, path: String },
    /// A notification action button was pressed.
    ActionInvoked { sender: String, id: u32, key: String },
    /// A notification closed for `reason`.
    NotificationClosed { sender: String, id: u32, reason: u32 },
}

/// What: `Backend` names the two protocols.
/// Why: the thread keeps the one that opened, and the listener is started once per protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// A Plasma job view.
    Job,
    /// A freedesktop notification.
    Notification,
}

/// What: `JobView` is an open Plasma job.
/// Why: it keeps the proxy of the view object, its path, and the unique name of the server that owns it.
#[derive(Debug)]
pub struct JobView {
    proxy: Proxy<'static>,
    path: String,
    owner: String,
}

/// What: `FdoNotification` is an open freedesktop notification.
/// Why: it keeps the proxy of the service, the id every replacement reuses, and the service's unique name.
#[derive(Debug)]
pub struct FdoNotification {
    proxy: Proxy<'static>,
    id: u32,
    owner: String,
}

/// What: `job_request_hints` builds the `requestView` hints.
/// Why: `application-display-name` and `application-icon-name` give the header its name and icon even when the
/// desktop file is not installed (a dev run), and `transient` makes Plasma remove a finished job without notice.
pub fn job_request_hints(identity: &Identity) -> HashMap<&'static str, Value<'_>> {
    return HashMap::from([
        ("application-display-name", Value::from(identity.app_name.as_str())),
        ("application-icon-name", Value::from(identity.app_icon.as_str())),
        ("transient", Value::from(true)),
    ]);
}

/// What: `job_update_properties` builds an `update` dictionary for `view`.
/// Why: `percent` is the bar, `infoMessage` the count, and `suspended` flips Plasma's Pause and Resume button, which
/// Plasma never does itself. The title is sent only at the open.
pub fn job_update_properties(view: &IndicatorView, with_title: bool) -> HashMap<&'static str, Value<'_>> {
    let mut properties = HashMap::from([
        ("infoMessage", Value::from(view.text.as_str())),
        ("percent", Value::from(view.percent)),
        ("suspended", Value::from(view.paused)),
    ]);
    if with_title {
        properties.insert("title", Value::from(TITLE));
    }
    return properties;
}

/// What: `terminate_arguments` maps an outcome to the `terminate` error code and message.
/// Why: success and a stop show nothing; only a failure carries text.
pub fn terminate_arguments(outcome: &Outcome) -> (u32, String) {
    return match outcome {
        Outcome::Finished => (JOB_ERROR_NONE, String::new()),
        Outcome::Stopped => (JOB_ERROR_KILLED, String::new()),
        Outcome::Failed(message) => (JOB_ERROR_USER, message.clone()),
    };
}

/// What: `notification_hints` builds the `Notify` hints for `view`.
/// Why: `value` draws the bar on servers that support it; urgency 0 (a byte) is low; `suppress-sound` keeps it silent;
/// `category` is the spec's generic `transfer` because it defines no progress category; `desktop-entry` names the app;
/// `resident` keeps the notification when its action button is pressed, which the spec says removes it otherwise.
pub fn notification_hints<'a>(identity: &'a Identity, view: &IndicatorView) -> HashMap<&'static str, Value<'a>> {
    let percent = i32::try_from(view.percent).unwrap_or(i32::MAX);
    return HashMap::from([
        ("value", Value::I32(percent)),
        ("urgency", Value::U8(0)),
        ("suppress-sound", Value::Bool(true)),
        ("category", Value::from("transfer")),
        ("desktop-entry", Value::from(identity.desktop_entry.as_str())),
        ("resident", Value::Bool(true)),
    ]);
}

/// What: `notification_actions` lists the action key and label pairs for `view`.
/// Why: a suspendable indicator offers Pause, or Resume while paused; otherwise there is no button.
pub fn notification_actions(suspendable: bool, view: &IndicatorView) -> Vec<&'static str> {
    if !suspendable {
        return Vec::new();
    }
    if view.paused {
        return vec![ACTION_RESUME, "Resume"];
    }
    return vec![ACTION_PAUSE, "Pause"];
}

/// What: `has_owner` asks the bus whether `name` is owned, without starting its service.
/// Why: a plain method call to an activatable name could start a stray process; this only reads the bus.
pub fn has_owner(connection: &Connection, name: &str) -> zbus::Result<bool> {
    let bus = DBusProxy::new(connection)?;
    return Ok(bus.name_has_owner(BusName::try_from(name)?)?);
}

/// What: `owner_of` returns the unique name that owns `name`.
/// Why: signals are trusted only from the process that answered the request.
fn owner_of(connection: &Connection, name: &str) -> zbus::Result<String> {
    let bus = DBusProxy::new(connection)?;
    return Ok(bus.get_name_owner(BusName::try_from(name)?)?.to_string());
}

/// Methods of `JobView`.
impl JobView {
    /// What: `open` asks Plasma for a job view and sends its first properties.
    /// Why: the title, count, percent and `suspended: false` appear together, so Plasma shows a complete job at once.
    pub fn open(
        connection: &Connection,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<JobView> {
        let owner = owner_of(connection, KUISERVER)?;
        let server = Proxy::new(connection, KUISERVER, JOB_SERVER_PATH, JOB_SERVER_INTERFACE)?;
        let capabilities = if suspendable { CAPABILITY_SUSPENDABLE } else { 0 };
        let hints = job_request_hints(identity);
        let reply: OwnedObjectPath = server.call("requestView", &(identity.desktop_entry.as_str(), capabilities, hints))?;
        let path = reply.to_string();
        tracing::info!(path = %path, owner = %owner, suspendable, "analysis job view granted by Plasma");
        let proxy = Proxy::new(connection, KUISERVER, path.clone(), JOB_VIEW_INTERFACE)?;
        let job = JobView { proxy, path, owner };
        job.update(view, true)?;
        return Ok(job);
    }

    /// What: `update` sends the properties of `view`; `NoReply`, so it never waits for Plasma.
    /// Why: updates are fire-and-forget in the protocol, and waiting would stall the thread for no information.
    pub fn update(&self, view: &IndicatorView, with_title: bool) -> zbus::Result<()> {
        return self.proxy.call_noreply("update", &(job_update_properties(view, with_title),));
    }

    /// What: `terminate` ends the job.
    /// Why: it removes the view; a failure shows its message.
    pub fn terminate(&self, outcome: &Outcome) -> zbus::Result<()> {
        let (code, message) = terminate_arguments(outcome);
        let hints: HashMap<&str, Value<'_>> = HashMap::new();
        tracing::info!(path = %self.path, code, "analysis job view terminates");
        return self.proxy.call_noreply("terminate", &(code, message, hints));
    }

    /// What: `owns` is true when a signal came from this job's server about this job's path.
    /// Why: another app's job signals, or a different path, must not pause the analysis.
    pub fn owns(&self, sender: &str, path: &str) -> bool {
        return self.owner == sender && self.path == path;
    }
}

/// Methods of `FdoNotification`.
impl FdoNotification {
    /// What: `notify` posts or replaces the notification and keeps the id the service returns.
    /// Why: the first call passes id 0 and gets a new id; every later call passes it back and the service replaces
    /// the notification in place.
    fn notify(
        proxy: &Proxy<'static>,
        replaces_id: u32,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<u32> {
        let actions = notification_actions(suspendable, view);
        let hints = notification_hints(identity, view);
        return proxy.call(
            "Notify",
            &(
                identity.app_name.as_str(),
                replaces_id,
                identity.app_icon.as_str(),
                TITLE,
                view.text.as_str(),
                actions,
                hints,
                0_i32,
            ),
        );
    }

    /// What: `open` posts the first notification.
    /// Why: it records the id and the owner for the later replacements and the signal checks.
    pub fn open(
        connection: &Connection,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<FdoNotification> {
        let proxy = Proxy::new(connection, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE)?;
        let id = FdoNotification::notify(&proxy, 0, identity, suspendable, view)?;
        let owner = owner_of(connection, NOTIFICATIONS_NAME)?;
        tracing::info!(id, owner = %owner, "analysis notification posted");
        return Ok(FdoNotification { proxy, id, owner });
    }

    /// What: `update` replaces the notification with `view`.
    /// Why: the returned id is kept in case a server answers with a different one.
    pub fn update(&mut self, identity: &Identity, suspendable: bool, view: &IndicatorView) -> zbus::Result<()> {
        self.id = FdoNotification::notify(&self.proxy, self.id, identity, suspendable, view)?;
        return Ok(());
    }

    /// What: `close` removes the notification.
    /// Why: D107 removes the indicator when analysis ends.
    pub fn close(&self) -> zbus::Result<()> {
        tracing::info!(id = self.id, "analysis notification closes");
        return self.proxy.call("CloseNotification", &(self.id,));
    }

    /// What: `owns` is true when a signal came from this service about this notification.
    /// Why: another app's notification ids must not be taken for ours.
    pub fn owns(&self, sender: &str, id: u32) -> bool {
        return self.owner == sender && self.id == id;
    }
}

/// What: `decode_signal` turns a bus message into a `BusSignal`, or `None` for anything else.
/// Why: the listener forwards only the four signals the indicator acts on.
fn decode_signal(message: &zbus::Message) -> Option<BusSignal> {
    let header = message.header();
    let sender = header.sender().map(ToString::to_string).unwrap_or_default();
    let member = header.member().map(ToString::to_string)?;
    let path = header.path().map(ToString::to_string).unwrap_or_default();
    return match member.as_str() {
        "suspendRequested" => Some(BusSignal::JobSuspend { sender, path }),
        "resumeRequested" => Some(BusSignal::JobResume { sender, path }),
        "ActionInvoked" => {
            let (id, key): (u32, String) = message.body().deserialize().ok()?;
            Some(BusSignal::ActionInvoked { sender, id, key })
        }
        "NotificationClosed" => {
            let (id, reason): (u32, u32) = message.body().deserialize().ok()?;
            Some(BusSignal::NotificationClosed { sender, id, reason })
        }
        _ => None,
    };
}

/// What: `spawn_listener` starts a thread that forwards the signals of `backend`'s interface to `forward`.
/// Why: a blocking connection cannot be waited on together with a channel, so signals get their own thread. The
/// thread ends when `forward` returns false (the indicator's thread is gone), when the stream ends, or when the
/// connection closes. It subscribes before the first request, so an early button press is not lost.
pub fn spawn_listener<F>(connection: &Connection, backend: Backend, forward: F) -> zbus::Result<JoinHandle<()>>
where
    F: Fn(BusSignal) -> bool + Send + 'static,
{
    let interface = match backend {
        Backend::Job => JOB_VIEW_INTERFACE,
        Backend::Notification => NOTIFICATIONS_INTERFACE,
    };
    let rule = MatchRule::builder().msg_type(Type::Signal).interface(interface)?.build();
    let stream = MessageIterator::for_match_rule(rule, connection, Some(64))?;
    tracing::debug!(interface, "analysis indicator listening for signals");
    let handle = std::thread::Builder::new()
        .name(format!("analysis-signals-{interface}"))
        .spawn(move || {
            for received in stream {
                let message = match received {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::info!(error = %error, interface, "analysis signal stream ended with an error");
                        break;
                    }
                };
                if let Some(signal) = decode_signal(&message)
                    && !forward(signal)
                {
                    break;
                }
            }
            tracing::debug!(interface, "analysis signal listener stopped");
        })?;
    return Ok(handle);
}

/// Unit tests in `analysis_notification_wire_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "analysis_notification_wire_tests.rs"]
mod tests;
