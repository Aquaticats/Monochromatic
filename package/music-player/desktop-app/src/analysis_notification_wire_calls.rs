//! The calls the indicator makes on its two bus objects.
//!
//! What:
//! `impl JobView` opens,
//! updates and terminates a Plasma job,
//! and says whether a signal belongs to it;
//! `impl FdoNotification` posts,
//! replaces and closes a freedesktop notification,
//! and says whether a signal belongs to it.
//! Why:
//! the message shapes stay in `src/analysis_notification_wire.rs`,
//! and the signal listener in `src/analysis_notification_wire_listener.rs`,
//! so this file holds the calls themselves and nothing else.

/// What:
///      `use std::collections::HashMap` brings the dictionary type that the
///  `terminate` call sends as its hints.
/// Why:
///      `terminate` takes a hints dictionary,
///  and an empty one is still a dictionary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has Map built in; no import is needed.
/// ```
use std::collections::HashMap;

/// What:
///      `use zbus::{...}` brings the connection,
///  the returned object path,
///  the proxy and the variant a call is made of.
/// Why:
///      Each method builds a proxy for its own object,
///  and passes variants as arguments,
///  so it needs those four types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Connection, OwnedObjectPath, Proxy, Variant } from "zbus";
/// ```
use zbus::{
    Connection,
    OwnedObjectPath,
    Proxy,
    Variant,
};

/// What:
///      `use super::{...}` brings the two bus objects,
///  the message builders,
///  the names and the ownership lookup from the parent module.
/// Why:
///      A call is one builder's payload sent to one name,
///  so every piece of it belongs to the parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JobView, FdoNotification, KUISERVER, JOB_SERVER_PATH, JOB_SERVER_INTERFACE, JOB_VIEW_INTERFACE, CAPABILITY_SUSPENDABLE, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE, TITLE, jobRequestHints, jobUpdateProperties, terminateArguments, notificationHints, notificationActions, ownerOf } from "./analysis_notification_wire";
/// ```
use super::{
    CAPABILITY_SUSPENDABLE,
    FdoNotification,
    JOB_SERVER_INTERFACE,
    JOB_SERVER_PATH,
    JOB_VIEW_INTERFACE,
    JobView,
    KUISERVER,
    NOTIFICATIONS_INTERFACE,
    NOTIFICATIONS_NAME,
    NOTIFICATIONS_PATH,
    TITLE,
    job_request_hints,
    job_update_properties,
    notification_actions,
    notification_hints,
    owner_of,
    terminate_arguments,
};

/// What:
///      `use crate::analysis_notification::Identity` brings the app identity a job
///           or a notification is opened with.
/// Why:
///      Both protocols show the app's name and icon,
///  which only the identity carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { Identity } from "./analysis_notification";
/// ```
use crate::analysis_notification::Identity;

/// What:
///      `use crate::analysis_notification_logic::{...}` brings the indicator view a
///           call draws,
///  and the outcome a job ends with.
/// Why:
///      The payloads are built from those two values,
///  so the wire layer never decides what is shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { IndicatorView, Outcome } from "./analysis_notification_logic";
/// ```
use crate::analysis_notification_logic::{
    IndicatorView,
    Outcome,
};

/// What:
///      `use crate::dbus_step_bound::step` brings the bounded wait that runs one bus
///           call.
/// Why:
///      A bus call may never return,
///  and no caller of these methods may be blocked for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { step } from "./dbus_step_bound";
/// ```
use crate::dbus_step_bound::step;
/// Methods of `JobView`.
impl JobView {
    /// What:
    /// `open` asks Plasma for a job view and sends its first properties.
    /// Why:
    /// the title,
    /// count,
    /// percent and `suspended: false` appear together,
    /// so Plasma shows a complete job at once.
    pub fn open(
        connection: &Connection,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<JobView> {
        let owner = owner_of(connection, KUISERVER)?;
        let server = step(Proxy::new(
            connection,
            KUISERVER,
            JOB_SERVER_PATH,
            JOB_SERVER_INTERFACE,
        ))?;
        let capabilities = if suspendable {
            CAPABILITY_SUSPENDABLE
        } else {
            0
        };
        let hints = job_request_hints(identity);
        let reply: OwnedObjectPath = step(server.call(
            "requestView",
            &(identity.desktop_entry.as_str(), capabilities, hints),
        ))?;
        let path = reply.to_string();
        tracing::info!(path = %path, owner = %owner, suspendable, "analysis job view granted by Plasma");
        let proxy = step(Proxy::new(
            connection,
            KUISERVER,
            path.clone(),
            JOB_VIEW_INTERFACE,
        ))?;
        let job = JobView {
            proxy,
            path,
            owner,
        };
        job.update(view, true)?;
        return Ok(job);
    }

    /// What:
    /// `update` sends the properties of `view`;
    /// `NoReply`,
    /// so it never waits for Plasma.
    /// Why:
    /// updates are fire-and-forget in the protocol,
    /// and waiting would stall the thread for no information.
    pub fn update(
        &self,
        view: &IndicatorView,
        with_title: bool,
    ) -> zbus::Result<()> {
        return step(
            self.proxy
                .call_noreply("update", &(job_update_properties(view, with_title),)),
        );
    }

    /// What:
    /// `terminate` ends the job.
    /// Why:
    /// it removes the view;
    /// a failure shows its message.
    pub fn terminate(
        &self,
        outcome: &Outcome,
    ) -> zbus::Result<()> {
        let (code, message) = terminate_arguments(outcome);
        let hints: HashMap<&str, Variant<'_>> = HashMap::new();
        tracing::info!(path = %self.path, code, "analysis job view terminates");
        return step(
            self.proxy
                .call_noreply("terminate", &(code, message, hints)),
        );
    }

    /// What:
    /// `owns` is true when a signal came from this job's server about this job's path.
    /// Why:
    /// another app's job signals,
    /// or a different path,
    /// must not pause the analysis.
    pub fn owns(
        &self,
        sender: &str,
        path: &str,
    ) -> bool {
        return self.owner == sender && self.path == path;
    }
}

/// Methods of `FdoNotification`.
impl FdoNotification {
    /// What:
    /// `notify` posts or replaces the notification and keeps the id the service returns.
    /// Why:
    /// the first call passes id 0 and gets a new id;
    /// every later call passes it back and the service replaces
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
        return step(proxy.call(
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
        ));
    }

    /// What:
    /// `open` posts the first notification.
    /// Why:
    /// it records the id and the owner for the later replacements and the signal checks.
    pub fn open(
        connection: &Connection,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<FdoNotification> {
        let proxy = step(Proxy::new(
            connection,
            NOTIFICATIONS_NAME,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
        ))?;
        let id = FdoNotification::notify(&proxy, 0, identity, suspendable, view)?;
        let owner = owner_of(connection, NOTIFICATIONS_NAME)?;
        tracing::info!(id, owner = %owner, "analysis notification posted");
        return Ok(FdoNotification {
            proxy,
            id,
            owner,
        });
    }

    /// What:
    /// `update` replaces the notification with `view`.
    /// Why:
    /// the returned id is kept in case a server answers with a different one.
    pub fn update(
        &mut self,
        identity: &Identity,
        suspendable: bool,
        view: &IndicatorView,
    ) -> zbus::Result<()> {
        self.id = FdoNotification::notify(&self.proxy, self.id, identity, suspendable, view)?;
        return Ok(());
    }

    /// What:
    /// `close` removes the notification.
    /// Why:
    /// D107 removes the indicator when analysis ends.
    pub fn close(&self) -> zbus::Result<()> {
        tracing::info!(id = self.id, "analysis notification closes");
        return step(self.proxy.call("CloseNotification", &(self.id,)));
    }

    /// What:
    /// `owns` is true when a signal came from this service about this notification.
    /// Why:
    /// another app's notification ids must not be taken for ours.
    pub fn owns(
        &self,
        sender: &str,
        id: u32,
    ) -> bool {
        return self.owner == sender && self.id == id;
    }
}
