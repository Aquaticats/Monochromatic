//! The state of the fake Plasma job view server.
//!
//! The store holds plain values only,
//!  with no D-Bus types,
//!  so property merging,
//!  termination,
//!  and the ordered log are unit-tested directly.
//! The service thread calls it while handling client calls,
//!  and the control thread reads it.

/// Imports `event_ring::{EventRing, Timed}`,
///  `value_summary::ValueSummary` from `crate`.
use crate::{
    event_ring::{EventRing, Timed},
    value_summary::ValueSummary,
};

/// The object path of a job:
///  the prefix Plasma uses,
///  then the job number.
pub const JOB_PATH_PREFIX: &str = "/org/kde/notificationmanager/jobs/JobView_";

/// How a job ended,
///  as the client reported it to `terminate`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Termination {
    /// The `errorCode` argument;
    ///  `0` means no error.
    pub code: u32,
    /// The `errorMessage` argument.
    pub message: String,
}

/// The properties of an `update` call that change what the job shows.
///
/// What:
///      `pub struct JobChanges { title, info_message, percent, suspended }`,
///  each optional.
/// Why:
///       Plasma applies only the properties an update carries,
///  so a partial update keeps the others.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JobChanges {
    /// The `title` property.
    pub title: Option<String>,
    /// The `infoMessage` property.
    pub info_message: Option<String>,
    /// The `percent` property.
    pub percent: Option<u32>,
    /// The `suspended` property.
    pub suspended: Option<bool>,
}

/// A job view a client requested.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobRecord {
    /// The job number,
    ///  starting at one;
    ///  the object path ends in it.
    pub id: u32,
    /// The object path of the job's `org.kde.JobViewV3` object.
    pub path: String,
    /// The `desktopEntry` argument of `requestView` (or the app name of the first interface version).
    pub desktop_entry: String,
    /// The `capabilities` argument:
    ///  bit 1 cancel,
    ///  bit 2 suspend and resume.
    pub capabilities: i32,
    /// The latest `title`;
    ///  empty until an update sets it.
    pub title: String,
    /// The latest `infoMessage`;
    ///  empty until an update sets it.
    pub info_message: String,
    /// The latest `percent`;
    ///  zero until an update sets it.
    pub percent: u32,
    /// The latest `suspended`;
    ///  false until an update sets it.
    pub suspended: bool,
    /// How the job ended,
    ///  once `terminate` arrived.
    pub termination: Option<Termination>,
}

/// Asking a record whether its job is still running.
impl JobRecord {
    /// Tells whether `terminate` has not arrived yet.
    #[must_use]
    pub fn is_running(&self) -> bool {
        return self.termination.is_none();
    }
}

/// Something the fake server observed or did,
///  in order.
#[derive(Clone, Debug, PartialEq)]
pub enum JobEvent {
    /// A client called `requestView`.
    RequestView {
        /// The granted job number.
        id: u32,
        /// `1` for `org.kde.JobViewServer`,
        ///  `2` for `org.kde.JobViewServerV2`.
        api: u8,
        /// The desktop entry,
        ///  or the app name for the first interface version.
        desktop_entry: String,
        /// The capabilities bit field.
        capabilities: i32,
        /// The hints (or,
        ///  for the first version,
        ///  the app icon name),
        ///  summarized.
        hints: Vec<ValueSummary>,
    },
    /// A client called `update`.
    Update {
        /// The job number.
        id: u32,
        /// Whether the call carried the no-reply-expected flag,
        ///  as Qt sets for this method.
        no_reply: bool,
        /// Every property of the call,
        ///  summarized.
        properties: Vec<ValueSummary>,
    },
    /// A client called `terminate`.
    Terminate {
        /// The job number.
        id: u32,
        /// Whether the call carried the no-reply-expected flag.
        no_reply: bool,
        /// The `errorCode` argument.
        code: u32,
        /// The `errorMessage` argument.
        message: String,
        /// The `hints` argument,
        ///  summarized.
        hints: Vec<ValueSummary>,
    },
    /// The server emitted a `org.kde.JobViewV3` signal.
    Signal {
        /// The job number.
        id: u32,
        /// The signal name,
        ///  such as `cancelRequested`.
        name: String,
    },
    /// A client called a method of `org.kde.kuiserver`.
    KuiserverCall {
        /// The method name.
        method: String,
        /// The arguments,
        ///  or what the server answered when it refused.
        detail: String,
    },
}

/// Job records,
///  the number counter,
///  and the ordered log.
#[derive(Debug, Default)]
pub struct JobStore {
    /// The most recently granted job number.
    last_id: u32,
    /// Every job not yet cleared,
    ///  oldest first,
    ///  running and terminated.
    jobs: Vec<JobRecord>,
    /// Every call and signal,
    ///  oldest first.
    events: EventRing<JobEvent>,
}

/// Recording calls and signals,
///  and reading the result.
impl JobStore {
    /// Grants the next job number and records the job.
    ///
    /// What:
    ///      `pub fn request_view(&mut self, at_ms: u64, request: JobRequest) -> JobRecord`.
    ///  Numbers start at one,
    ///  as Plasma's counter does.
    pub fn request_view(&mut self, at_ms: u64, request: JobRequest) -> JobRecord {
        self.last_id += 1;
        let record = JobRecord {
            id: self.last_id,
            path: format!("{JOB_PATH_PREFIX}{}", self.last_id),
            desktop_entry: request.desktop_entry.clone(),
            capabilities: request.capabilities,
            title: String::new(),
            info_message: String::new(),
            percent: 0,
            suspended: false,
            termination: None,
        };
        self.events.push(
            at_ms,
            JobEvent::RequestView {
                id: record.id,
                api: request.api,
                desktop_entry: request.desktop_entry,
                capabilities: request.capabilities,
                hints: request.hints,
            },
        );
        self.jobs.push(record.clone());
        return record;
    }

    /// Records an `update` call and applies the properties it carries.
    ///
    /// What:
    ///      `pub fn update(&mut self, at_ms: u64, call: UpdateCall)`.
    ///  The job keeps every property the call leaves out.
    pub fn update(&mut self, at_ms: u64, call: UpdateCall) {
        if let Some(job) = self.find_mut(call.id) {
            if let Some(title) = call.changes.title {
                job.title = title;
            }
            if let Some(info) = call.changes.info_message {
                job.info_message = info;
            }
            if let Some(percent) = call.changes.percent {
                job.percent = percent;
            }
            if let Some(suspended) = call.changes.suspended {
                job.suspended = suspended;
            }
        }
        self.events.push(
            at_ms,
            JobEvent::Update { id: call.id, no_reply: call.no_reply, properties: call.properties },
        );
    }

    /// Records a `terminate` call and marks the job ended.
    pub fn terminate(&mut self, at_ms: u64, call: TerminateCall) {
        if let Some(job) = self.find_mut(call.id) {
            job.termination = Some(Termination { code: call.code, message: call.message.clone() });
        }
        self.events.push(
            at_ms,
            JobEvent::Terminate {
                id: call.id,
                no_reply: call.no_reply,
                code: call.code,
                message: call.message,
                hints: call.hints,
            },
        );
    }

    /// Logs a V3 signal about to be emitted.
    pub fn log_signal(&mut self, at_ms: u64, id: u32, name: &str) {
        self.events.push(at_ms, JobEvent::Signal { id, name: name.to_string() });
    }

    /// Logs a call to `org.kde.kuiserver`.
    pub fn log_kuiserver_call(&mut self, at_ms: u64, method: &str, detail: &str) {
        self.events.push(
            at_ms,
            JobEvent::KuiserverCall { method: method.to_string(), detail: detail.to_string() },
        );
    }

    /// Returns a job by number.
    #[must_use]
    pub fn find(&self, id: u32) -> Option<&JobRecord> {
        for job in &self.jobs {
            if job.id == id {
                return Some(job);
            }
        }
        return None;
    }

    /// Returns a job by number for changing.
    fn find_mut(&mut self, id: u32) -> Option<&mut JobRecord> {
        for job in &mut self.jobs {
            if job.id == id {
                return Some(job);
            }
        }
        return None;
    }

    /// Returns every job,
    ///  oldest first.
    #[must_use]
    pub fn jobs(&self) -> &[JobRecord] {
        return &self.jobs;
    }

    /// Returns up to `count` of the newest log entries,
    ///  oldest first.
    #[must_use]
    pub fn last_events(&self, count: usize) -> Vec<&Timed<JobEvent>> {
        return self.events.last_n(count);
    }

    /// Forgets the log and every terminated job;
    ///  running jobs stay.
    ///
    /// What:
    ///      `pub fn clear(&mut self)`.
    /// Why:
    ///       A running job still owns an object on the bus,
    ///  so dropping its record would orphan it.
    pub fn clear(&mut self) {
        self.events.clear();
        self.jobs.retain(JobRecord::is_running);
    }
}

/// One `requestView` call.
#[derive(Clone, Debug, PartialEq)]
pub struct JobRequest {
    /// `1` for `org.kde.JobViewServer`,
    ///  `2` for `org.kde.JobViewServerV2`.
    pub api: u8,
    /// The desktop entry (or app name).
    pub desktop_entry: String,
    /// The capabilities bit field.
    pub capabilities: i32,
    /// The hints,
    ///  summarized.
    pub hints: Vec<ValueSummary>,
}

/// One `update` call,
///  decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateCall {
    /// The job number.
    pub id: u32,
    /// Whether the no-reply-expected flag was set.
    pub no_reply: bool,
    /// The properties that change the job's state.
    pub changes: JobChanges,
    /// Every property,
    ///  summarized,
    ///  for the log.
    pub properties: Vec<ValueSummary>,
}

/// One `terminate` call,
///  decoded.
#[derive(Clone, Debug, PartialEq)]
pub struct TerminateCall {
    /// The job number.
    pub id: u32,
    /// Whether the no-reply-expected flag was set.
    pub no_reply: bool,
    /// The error code.
    pub code: u32,
    /// The error message.
    pub message: String,
    /// The hints,
    ///  summarized.
    pub hints: Vec<ValueSummary>,
}

/// Unit tests for numbering,
///  merging,
///  and termination.
#[cfg(test)]
#[path = "job_store_tests.rs"]
mod tests;
