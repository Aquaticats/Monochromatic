//! One granted Plasma job view:
//!  the `org.kde.JobViewV3` object a client updates and terminates.
//!
//! Plasma's real job object accepts `update` and `terminate` without replying,
//!  changes its Pause button only after an `update` that carries `suspended`,
//!  and removes the object when the job ends.
//! This fake records the same calls,
//!  merges the same properties,
//!  and removes the object the same way.

/// Imports `HashMap` from `std::collections`.
use std::collections::HashMap;

/// Imports `debug`,
///  `info`,
///  `warn` from `tracing`.
use tracing::{debug, info, warn};
/// Imports the items this module uses from `zbus`.
use zbus::{
    message::{Flags, Header},
    object_server::SignalEmitter,
    zvariant::{OwnedValue, Value},
    ObjectServer,
};

/// Imports the items this module uses from `crate`.
use crate::{
    job_store::{JobChanges, JobStore, TerminateCall, UpdateCall},
    service_state::Shared,
    value_summary::summarize_map,
};

/// The interface of a job view object.
pub const JOB_VIEW_INTERFACE: &str = "org.kde.JobViewV3";

/// The job view object registered at each granted path.
pub struct JobView {
    /// The store every call is recorded in.
    store: Shared<JobStore>,
    /// The job number.
    id: u32,
    /// The path this object is registered at.
    path: String,
}

/// Building a job view object.
impl JobView {
    /// Creates the object for job `id` served at `path`.
    #[must_use]
    pub fn new(store: Shared<JobStore>, id: u32, path: String) -> Self {
        return Self { store, id, path };
    }
}

/// Looks up one property of an `update` call as a plain value.
fn property<'a>(properties: &'a HashMap<String, OwnedValue>, key: &str) -> Option<&'a Value<'static>> {
    let value = properties.get(key)?;
    return Some(value);
}

/// Reads a string property,
///  or `None` when the key is absent or not a string.
fn string_property(properties: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    return match property(properties, key) {
        Some(Value::Str(text)) => Some(text.to_string()),
        _ => None,
    };
}

/// Reads an unsigned integer property of any integer width,
///  or `None` when absent or not an integer.
///
/// What:
///      `fn percent_property(properties, key) -> Option<u32>`.
///  Plasma reads the value with `QVariant::toUInt`,
///  which converts between integer widths,
///  so a caller that sends `int32` instead of `uint32` still moves the bar.
///  The log shows the signature the caller used.
fn percent_property(properties: &HashMap<String, OwnedValue>, key: &str) -> Option<u32> {
    return match property(properties, key) {
        Some(Value::U8(number)) => Some(u32::from(*number)),
        Some(Value::U16(number)) => Some(u32::from(*number)),
        Some(Value::U32(number)) => Some(*number),
        Some(Value::U64(number)) => u32::try_from(*number).ok(),
        Some(Value::I16(number)) => u32::try_from(*number).ok(),
        Some(Value::I32(number)) => u32::try_from(*number).ok(),
        Some(Value::I64(number)) => u32::try_from(*number).ok(),
        _ => None,
    };
}

/// Reads a boolean property,
///  or `None` when the key is absent or not a boolean.
fn bool_property(properties: &HashMap<String, OwnedValue>, key: &str) -> Option<bool> {
    return match property(properties, key) {
        Some(Value::Bool(flag)) => Some(*flag),
        _ => None,
    };
}

/// Decodes the properties that change what a job shows.
fn job_changes(properties: &HashMap<String, OwnedValue>) -> JobChanges {
    return JobChanges {
        title: string_property(properties, "title"),
        info_message: string_property(properties, "infoMessage"),
        percent: percent_property(properties, "percent"),
        suspended: bool_property(properties, "suspended"),
    };
}

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The methods and signals of `org.kde.JobViewV3`.
    #[zbus::interface(name = "org.kde.JobViewV3")]
    impl JobView {
        /// Records an update and applies the properties it carries.
        ///
        /// Properties other than `title`,
        ///  `infoMessage`,
        ///  `percent`,
        ///  and `suspended` are logged and not applied,
        ///  and `descriptionField` is ignored,
        ///  as in Plasma's `job_p.cpp`.
        #[zbus(name = "update")]
        async fn update(&self, properties: HashMap<String, OwnedValue>, #[zbus(header)] header: Header<'_>) {
            let no_reply = header.primary().flags().contains(Flags::NoReplyExpected);
            let call = UpdateCall {
                id: self.id,
                no_reply,
                changes: job_changes(&properties),
                properties: summarize_map(&properties),
            };
            self.store.lock().update(self.store.now_ms(), call);
            debug!(id = self.id, no_reply, "recorded a job update");
        }

        /// Records the end of the job and removes the object,
        ///  as Plasma's `finish()` does.
        #[zbus(name = "terminate")]
        async fn terminate(
            &self,
            error_code: u32,
            error_message: String,
            hints: HashMap<String, OwnedValue>,
            #[zbus(header)] header: Header<'_>,
            #[zbus(object_server)] server: &ObjectServer,
        ) {
            let no_reply = header.primary().flags().contains(Flags::NoReplyExpected);
            let call = TerminateCall {
                id: self.id,
                no_reply,
                code: error_code,
                message: error_message,
                hints: summarize_map(&hints),
            };
            self.store.lock().terminate(self.store.now_ms(), call);
            info!(id = self.id, code = error_code, no_reply, "recorded the end of a job");
            match server.remove::<Self, _>(self.path.as_str()).await {
                Ok(removed) => debug!(id = self.id, removed, "unregistered the ended job object"),
                Err(error) => warn!(id = self.id, %error, "could not unregister the ended job object"),
            }
        }

        /// Asks the owner to send all properties again.
        #[zbus(signal, name = "updateRequested")]
        pub async fn update_requested(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

        /// The user pressed Pause.
        #[zbus(signal, name = "suspendRequested")]
        pub async fn suspend_requested(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

        /// The user pressed Resume.
        #[zbus(signal, name = "resumeRequested")]
        pub async fn resume_requested(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

        /// The user pressed Cancel.
        #[zbus(signal, name = "cancelRequested")]
        pub async fn cancel_requested(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
    }
};

/// Unit tests for the property decoding.
#[cfg(test)]
#[path = "job_view_tests.rs"]
mod tests;
