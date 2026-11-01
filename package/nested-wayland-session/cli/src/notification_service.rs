//! The fake `org.freedesktop.Notifications` server on the private bus.
//!
//! It records every call in the notification store and draws nothing.
//! Closing and action signals leave only when a control command asks for them.

/// Imports `HashMap` from `std::collections`.
use std::collections::HashMap;

/// Imports `debug`,
///  `info` from `tracing`.
use tracing::{debug, info};
/// Imports `object_server::SignalEmitter`,
///  `zvariant::OwnedValue` from `zbus`.
use zbus::{object_server::SignalEmitter, zvariant::OwnedValue};

/// Imports the items this module uses from `crate`.
use crate::{
    notification_store::{NotificationStore, NotifyCall, REASON_CLOSED_BY_CALL},
    service_state::Shared,
    value_summary::summarize_map,
};

/// The well-known name of the notification service.
pub const NOTIFICATIONS_NAME: &str = "org.freedesktop.Notifications";

/// The object path of the notification service.
pub const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";

/// The interface of the notification service.
pub const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

/// What `GetCapabilities` reports.
///
/// What:
///      The capabilities of Plasma 6.7.5,
///  read from the live host with `GetCapabilities`,
///  minus `inline-reply`,
///  `sound`,
///  and `inhibitions`.
/// Why:
///       The fake records `Notify` calls and nothing else.
///  It plays no sound,
///  offers no reply field,
///  and has no `Inhibit` method,
///  so claiming those would make a client rely on behavior that does not exist.
pub const CAPABILITIES: [&str; 10] = [
    "body",
    "body-hyperlinks",
    "body-markup",
    "body-images",
    "icon-static",
    "actions",
    "persistence",
    "x-kde-urls",
    "x-kde-origin-name",
    "x-kde-display-appname",
];

/// The service object registered at `NOTIFICATIONS_PATH`.
pub struct Notifications {
    /// The store every call is recorded in.
    store: Shared<NotificationStore>,
}

/// Building the service object.
impl Notifications {
    /// Creates the service over a shared store.
    #[must_use]
    pub fn new(store: Shared<NotificationStore>) -> Self {
        return Self { store };
    }
}

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The methods and signals of `org.freedesktop.Notifications` that Plasma and the specification share.
    #[zbus::interface(name = "org.freedesktop.Notifications")]
    impl Notifications {
        /// Records a notification and returns its id.
        async fn notify(
            &self,
            app_name: String,
            replaces_id: u32,
            app_icon: String,
            summary: String,
            body: String,
            actions: Vec<String>,
            hints: HashMap<String, OwnedValue>,
            expire_timeout: i32,
        ) -> u32 {
            let call = NotifyCall {
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions,
                hints: summarize_map(&hints),
                expire_timeout,
            };
            let id = self.store.lock().notify(self.store.now_ms(), call);
            info!(id, replaces_id, "recorded a Notify call");
            return id;
        }

        /// Records a close request,
        ///  then announces the close the way Plasma does.
        ///
        /// Plasma emits `NotificationClosed` with reason 3 for every id,
        ///  known or not,
        ///  and never errors.
        async fn close_notification(
            &self,
            id: u32,
            #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        ) -> zbus::fdo::Result<()> {
            let known = self.store.lock().close_call(self.store.now_ms(), id);
            debug!(id, known, "recorded a CloseNotification call");
            Self::notification_closed(&emitter, id, REASON_CLOSED_BY_CALL).await?;
            self.store.lock().log_closed(self.store.now_ms(), id, REASON_CLOSED_BY_CALL);
            return Ok(());
        }

        /// Reports the capabilities listed in `CAPABILITIES`.
        fn get_capabilities(&self) -> Vec<String> {
            return CAPABILITIES.into_iter().map(String::from).collect();
        }

        /// Reports this fake as itself,
        ///  with the specification version Plasma 6.7.5 reports.
        fn get_server_information(&self) -> (String, String, String, String) {
            return (
                "monochromatic-nested-wayland-session".to_string(),
                "Monochromatic".to_string(),
                env!("CARGO_PKG_VERSION").to_string(),
                "1.2".to_string(),
            );
        }

        /// Announces that a notification was closed,
        ///  with the specification's reason code.
        #[zbus(signal)]
        pub async fn notification_closed(emitter: &SignalEmitter<'_>, id: u32, reason: u32) -> zbus::Result<()>;

        /// Announces that the user invoked an action of a notification.
        #[zbus(signal)]
        pub async fn action_invoked(emitter: &SignalEmitter<'_>, id: u32, action_key: &str) -> zbus::Result<()>;
    }
};
