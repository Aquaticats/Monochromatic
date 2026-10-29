//! Answers the Plasma job control verbs from the fake job view server's records.
//!
//! Each verb is a plain function of the services handle,
//!  so the refusals are testable without a display.

/// Imports `debug`,
///  `warn` from `tracing`.
use tracing::{debug, warn};

/// Imports the items this module uses from `crate`.
use crate::{
    job_format::{format_job_event_list, format_job_list},
    plasma_services::PlasmaServices,
    protocol::{JobCommand, Response},
};

/// The refusal every job verb gives when the session serves no job view server.
const NO_SERVICE_MESSAGE: &str = "no Plasma job service (start with --notifications kde)";

/// Runs one parsed job verb.
///
/// What:
///      `pub fn run(services: Option<&PlasmaServices>, command: &JobCommand) -> Response`.
///  `None` means the session started without `--notifications`.
///  `--notifications freedesktop` serves notifications and no job view server,
///  so it is refused the same way.
pub fn run(services: Option<&PlasmaServices>, command: &JobCommand) -> Response {
    let Some(store) = services.and_then(PlasmaServices::jobs) else {
        debug!(?command, "refused a job verb: no job view server");
        return Response::Err(NO_SERVICE_MESSAGE.to_string());
    };
    match command {
        JobCommand::List => return Response::OkWith(format_job_list(store.lock().jobs())),
        JobCommand::Log { count } => {
            let guard = store.lock();
            return Response::OkWith(format_job_event_list(&guard.last_events(*count)));
        }
        JobCommand::Clear => {
            store.lock().clear();
            return Response::Ok;
        }
        JobCommand::Suspend { id } => return signal(services, *id, "suspendRequested"),
        JobCommand::Resume { id } => return signal(services, *id, "resumeRequested"),
        JobCommand::Cancel { id } => return signal(services, *id, "cancelRequested"),
        JobCommand::Refresh { id } => return signal(services, *id, "updateRequested"),
    }
}

/// Emits one V3 signal on a running job's path.
///
/// What:
///      `fn signal(services, id, name) -> Response`.
///  An unknown or ended job is refused:
///  its object no longer exists on the bus.
///  The capabilities are not checked,
///  so a test can send a signal the real Plasma UI could not.
fn signal(services: Option<&PlasmaServices>, id: u32, name: &str) -> Response {
    let Some(active) = services else {
        return Response::Err(NO_SERVICE_MESSAGE.to_string());
    };
    let Some(store) = active.jobs() else {
        return Response::Err(NO_SERVICE_MESSAGE.to_string());
    };
    let path = {
        let mut guard = store.lock();
        let Some(job) = guard.find(id) else {
            return Response::Err(format!("no job with id {id}"));
        };
        if job.termination.is_some() {
            return Response::Err(format!("job {id} has ended; its object is gone from the bus"));
        }
        let path = job.path.clone();
        guard.log_signal(store.now_ms(), id, name);
        path
    };
    match active.emit_job_signal(&path, name) {
        Ok(()) => return Response::Ok,
        Err(error) => {
            warn!(%error, id, name, "job signal was not delivered");
            return Response::Err(format!("{error:#}"));
        }
    }
}
