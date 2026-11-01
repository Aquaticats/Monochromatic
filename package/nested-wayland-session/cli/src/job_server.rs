//! The fake Plasma job view server at `/JobViewServer`.
//!
//! Three interfaces live on that one path,
//!  as in Plasma:
//!  `org.kde.JobViewServer` (first version),
//!  `org.kde.JobViewServerV2`,
//!  and `org.kde.kuiserver`.
//! The first two grant job views;
//!  the third answers the way Plasma 6.7.5 does.

/// Imports `HashMap` from `std::collections`.
use std::collections::HashMap;

/// Imports `debug`,
///  `info`,
///  `warn` from `tracing`.
use tracing::{debug, info, warn};
/// Imports the items this module uses from `zbus`.
use zbus::{
    object_server::SignalEmitter,
    zvariant::{OwnedObjectPath, OwnedValue},
    ObjectServer,
};

/// Imports the items this module uses from `crate`.
use crate::{
    job_store::{JobRequest, JobStore},
    job_view::JobView,
    service_state::Shared,
    value_summary::{summarize_map, ValueSummary},
};

/// The well-known name that owns the second and later interface versions and `org.kde.kuiserver`.
pub const KUISERVER_NAME: &str = "org.kde.kuiserver";

/// The well-known name of the first interface version.
pub const JOB_VIEW_SERVER_NAME: &str = "org.kde.JobViewServer";

/// The object path of the job view server.
pub const JOB_VIEW_SERVER_PATH: &str = "/JobViewServer";

/// What `requiresJobTracker` answers.
///
/// What:
///      `false`.
/// Why:
///       Plasma answers `!jobsInNotifications`,
///  and jobs show inside notifications by default.
pub const REQUIRES_JOB_TRACKER: bool = false;

/// What `registerService` refuses with,
///  copied from Plasma's `jobsmodel_p.cpp`.
const REGISTER_SERVICE_REFUSAL: &str = "kuiserver proxying capabilities are deprecated and defunct.";

/// Registers a job object and returns its path,
///  the shared tail of both `requestView` versions.
async fn grant_view(
    store: &Shared<JobStore>,
    server: &ObjectServer,
    request: JobRequest,
) -> zbus::fdo::Result<OwnedObjectPath> {
    let record = store.lock().request_view(store.now_ms(), request);
    let object = JobView::new(store.clone(), record.id, record.path.clone());
    server.at(record.path.as_str(), object).await?;
    info!(id = record.id, path = %record.path, "granted a job view");
    return match OwnedObjectPath::try_from(record.path) {
        Ok(path) => Ok(path),
        Err(error) => Err(zbus::fdo::Error::Failed(error.to_string())),
    };
}

/// The first interface version:
///  an app name,
///  an icon name,
///  and capabilities.
pub struct JobViewServerV1 {
    /// The store every call is recorded in.
    store: Shared<JobStore>,
}

/// Building the first-version server object.
impl JobViewServerV1 {
    /// Creates the object over a shared store.
    #[must_use]
    pub fn new(store: Shared<JobStore>) -> Self {
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
    /// The method of `org.kde.JobViewServer`.
    #[zbus::interface(name = "org.kde.JobViewServer")]
    impl JobViewServerV1 {
        /// Grants a job view;
        ///  the app icon name is kept in the log as a hint.
        #[zbus(name = "requestView")]
        async fn request_view(
            &self,
            app_name: String,
            app_icon_name: String,
            capabilities: i32,
            #[zbus(object_server)] server: &ObjectServer,
        ) -> zbus::fdo::Result<OwnedObjectPath> {
            let hints = vec![ValueSummary {
                key: "appIconName".to_string(),
                signature: "s".to_string(),
                text: app_icon_name,
            }];
            let request = JobRequest { api: 1, desktop_entry: app_name, capabilities, hints };
            return grant_view(&self.store, server, request).await;
        }
    }
};

/// The second interface version:
///  a desktop entry,
///  capabilities,
///  and hints.
pub struct JobViewServerV2 {
    /// The store every call is recorded in.
    store: Shared<JobStore>,
}

/// Building the second-version server object.
impl JobViewServerV2 {
    /// Creates the object over a shared store.
    #[must_use]
    pub fn new(store: Shared<JobStore>) -> Self {
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
    /// The method of `org.kde.JobViewServerV2`.
    #[zbus::interface(name = "org.kde.JobViewServerV2")]
    impl JobViewServerV2 {
        /// Grants a job view.
        ///
        /// `capabilities` is a signed 32-bit integer:
        ///  a caller that sends an unsigned one calls
        ///  `requestView(sua{sv})`,
        ///  which does not exist here,
        ///  as on Plasma.
        #[zbus(name = "requestView")]
        async fn request_view(
            &self,
            desktop_entry: String,
            capabilities: i32,
            hints: HashMap<String, OwnedValue>,
            #[zbus(object_server)] server: &ObjectServer,
        ) -> zbus::fdo::Result<OwnedObjectPath> {
            let request = JobRequest { api: 2, desktop_entry, capabilities, hints: summarize_map(&hints) };
            return grant_view(&self.store, server, request).await;
        }
    }
};

/// The `org.kde.kuiserver` interface.
pub struct Kuiserver {
    /// The store every call is recorded in.
    store: Shared<JobStore>,
}

/// Building the kuiserver object.
impl Kuiserver {
    /// Creates the object over a shared store.
    #[must_use]
    pub fn new(store: Shared<JobStore>) -> Self {
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
    /// The methods and signals of `org.kde.kuiserver`.
    #[zbus::interface(name = "org.kde.kuiserver")]
    impl Kuiserver {
        /// Refuses,
        ///  as Plasma 6.7.5 does:
        ///  proxying through kuiserver is defunct.
        #[zbus(name = "registerService")]
        async fn register_service(&self, service: String, object_path: String) -> zbus::fdo::Result<()> {
            let detail = format!("service={service} path={object_path} refused");
            self.store.lock().log_kuiserver_call(self.store.now_ms(), "registerService", &detail);
            warn!(%service, %object_path, "refused a kuiserver registerService call");
            return Err(zbus::fdo::Error::NotSupported(REGISTER_SERVICE_REFUSAL.to_string()));
        }

        /// Answers an empty list,
        ///  as Plasma 6.7.5 does.
        #[zbus(name = "registeredJobContacts")]
        async fn registered_job_contacts(&self) -> Vec<String> {
            self.store.lock().log_kuiserver_call(self.store.now_ms(), "registeredJobContacts", "empty");
            return Vec::new();
        }

        /// Answers `REQUIRES_JOB_TRACKER`.
        #[zbus(name = "requiresJobTracker")]
        async fn requires_job_tracker(&self) -> bool {
            self.store.lock().log_kuiserver_call(self.store.now_ms(), "requiresJobTracker", "false");
            return REQUIRES_JOB_TRACKER;
        }

        /// Emits `jobUrlsChanged` listing the destination URLs of unfinished jobs.
        ///
        /// The fake does not record `destUrl`,
        ///  so the list is always empty.
        #[zbus(name = "emitJobUrlsChanged")]
        async fn emit_job_urls_changed(
            &self,
            #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        ) -> zbus::fdo::Result<()> {
            self.store.lock().log_kuiserver_call(self.store.now_ms(), "emitJobUrlsChanged", "empty");
            debug!("emitting an empty jobUrlsChanged");
            Self::job_urls_changed(&emitter, Vec::new()).await?;
            return Ok(());
        }

        /// Announces the destination URLs of unfinished jobs.
        #[zbus(signal, name = "jobUrlsChanged")]
        pub async fn job_urls_changed(emitter: &SignalEmitter<'_>, urls: Vec<String>) -> zbus::Result<()>;

        /// Announces whether apps must keep tracking jobs themselves.
        #[zbus(signal, name = "requiresJobTrackerChanged")]
        pub async fn requires_job_tracker_changed(emitter: &SignalEmitter<'_>, required: bool) -> zbus::Result<()>;
    }
};
