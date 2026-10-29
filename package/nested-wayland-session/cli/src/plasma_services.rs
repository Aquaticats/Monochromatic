//! The fake desktop notification and Plasma job services on the private bus.
//!
//! One bus connection owns every name,
//!  as Plasma's one process does:
//!  `org.freedesktop.Notifications`,
//!  and in `kde` mode also `org.kde.JobViewServer` and `org.kde.kuiserver`.
//! The host session bus is never opened.
//! The services record and draw nothing,
//!  so notifications and jobs exist only as records here.

/// Imports `sync::Arc`,
///  `time::Instant` from `std`.
use std::{sync::Arc, time::Instant};

/// Imports `Context`,
///  `Result` from `anyhow`.
use anyhow::{Context, Result};
/// Imports `info` from `tracing`.
use tracing::info;
/// Imports `connection::Builder`,
///  `Connection` from `zbus::blocking`.
use zbus::blocking::{connection::Builder, Connection};

/// Imports the items this module uses from `crate`.
use crate::{
    job_server::{
        JobViewServerV1, JobViewServerV2, Kuiserver, JOB_VIEW_SERVER_NAME, JOB_VIEW_SERVER_PATH,
        KUISERVER_NAME,
    },
    job_store::JobStore,
    job_view::JOB_VIEW_INTERFACE,
    notification_mode::NotificationMode,
    notification_service::{
        Notifications, NOTIFICATIONS_INTERFACE, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH,
    },
    notification_store::NotificationStore,
    private_bus::PrivateBus,
    service_state::Shared,
};

/// The running fake services and the records they fill.
pub struct PlasmaServices {
    /// The connection that owns every service name;
    ///  dropped before the bus.
    connection: Connection,
    /// Which services this session serves.
    mode: NotificationMode,
    /// Visible notifications and the notification log.
    notifications: Shared<NotificationStore>,
    /// Jobs and the job log;
    ///  present in `kde` mode only.
    jobs: Option<Shared<JobStore>>,
    /// The private bus,
    ///  kept alive while the services are served.
    _bus: Arc<PrivateBus>,
}

/// Starting the services and emitting their signals.
impl PlasmaServices {
    /// Serves the services `mode` names on `bus`;
    ///  `None` for the mode that serves nothing.
    ///
    /// What:
    ///      `pub fn start(mode, bus, start_time) -> Result<Option<Self>>`.
    ///  The names are owned before this returns,
    ///  so a hosted client started afterwards finds them.
    pub fn start(mode: NotificationMode, bus: Arc<PrivateBus>, start_time: Instant) -> Result<Option<Self>> {
        if !mode.serves_notifications() {
            info!("no notification service requested; calls to it get ServiceUnknown");
            return Ok(None);
        }
        let notifications = Shared::new(NotificationStore::default(), start_time);
        let jobs = if mode.serves_jobs() { Some(Shared::new(JobStore::default(), start_time)) } else { None };
        let mut builder = Builder::address(bus.address())?
            .name(NOTIFICATIONS_NAME)?
            .serve_at(NOTIFICATIONS_PATH, Notifications::new(notifications.clone()))?;
        if let Some(store) = &jobs {
            builder = builder
                .name(JOB_VIEW_SERVER_NAME)?
                .name(KUISERVER_NAME)?
                .serve_at(JOB_VIEW_SERVER_PATH, JobViewServerV1::new(store.clone()))?
                .serve_at(JOB_VIEW_SERVER_PATH, JobViewServerV2::new(store.clone()))?
                .serve_at(JOB_VIEW_SERVER_PATH, Kuiserver::new(store.clone()))?;
        }
        let connection = builder.build().context("starting the private notification services")?;
        info!(?mode, "started the private notification services");
        return Ok(Some(Self { connection, mode, notifications, jobs, _bus: bus }));
    }

    /// Returns which services are served.
    #[must_use]
    pub fn mode(&self) -> NotificationMode {
        return self.mode;
    }

    /// Returns the notification records.
    #[must_use]
    pub fn notifications(&self) -> &Shared<NotificationStore> {
        return &self.notifications;
    }

    /// Returns the job records,
    ///  in `kde` mode.
    #[must_use]
    pub fn jobs(&self) -> Option<&Shared<JobStore>> {
        return self.jobs.as_ref();
    }

    /// Emits `NotificationClosed(id, reason)` as a broadcast from the service path.
    pub fn emit_closed(&self, id: u32, reason: u32) -> Result<()> {
        self.connection
            .emit_signal(None::<&str>, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE, "NotificationClosed", &(id, reason))
            .context("emitting NotificationClosed on the private bus")?;
        return Ok(());
    }

    /// Emits `ActionInvoked(id, key)` as a broadcast from the service path.
    pub fn emit_action(&self, id: u32, key: &str) -> Result<()> {
        self.connection
            .emit_signal(None::<&str>, NOTIFICATIONS_PATH, NOTIFICATIONS_INTERFACE, "ActionInvoked", &(id, key))
            .context("emitting ActionInvoked on the private bus")?;
        return Ok(());
    }

    /// Emits one `org.kde.JobViewV3` signal as a broadcast from the job's path.
    ///
    /// What:
    ///      `pub fn emit_job_signal(&self, path: &str, name: &str) -> Result<()>`.
    ///  Plasma's Qt adaptor broadcasts these signals;
    ///  the app's match rule selects them by path.
    pub fn emit_job_signal(&self, path: &str, name: &str) -> Result<()> {
        self.connection
            .emit_signal(None::<&str>, path, JOB_VIEW_INTERFACE, name, &())
            .context("emitting a job view signal on the private bus")?;
        return Ok(());
    }
}
