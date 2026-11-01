//! Answers the notification control verbs from the fake notification service's records.
//!
//! Each verb is a plain function of the services handle,
//!  so the no-service refusal
//!  and every error text are testable without a display.

/// Imports `debug`,
///  `warn` from `tracing`.
use tracing::{debug, warn};

/// Imports the items this module uses from `crate`.
use crate::{
    notification_format::{format_event_list, format_visible_list},
    plasma_services::PlasmaServices,
    protocol::{NotificationCommand, Response},
};

/// The refusal every notification verb gives when the session serves no notification service.
const NO_SERVICE_MESSAGE: &str =
    "no notification service (start with --notifications freedesktop or --notifications kde)";

/// Runs one parsed notification verb.
///
/// What:
///      `pub fn run(services: Option<&PlasmaServices>, command: &NotificationCommand) -> Response`.
///  `None` means the session started without `--notifications`.
/// Why:
///       `ok 0` and the refusal are different answers:
///  the first means the service runs and is quiet,
///  the second means nothing can be observed at all.
pub fn run(services: Option<&PlasmaServices>, command: &NotificationCommand) -> Response {
    let Some(active) = services else {
        debug!(?command, "refused a notification verb: no notification service");
        return Response::Err(NO_SERVICE_MESSAGE.to_string());
    };
    let store = active.notifications();
    match command {
        NotificationCommand::List => {
            let guard = store.lock();
            return Response::OkWith(format_visible_list(guard.visible(), store.now_ms()));
        }
        NotificationCommand::Log { count } => {
            let guard = store.lock();
            return Response::OkWith(format_event_list(&guard.last_events(*count)));
        }
        NotificationCommand::Clear => {
            store.lock().clear();
            return Response::Ok;
        }
        NotificationCommand::Close { id, reason } => return close(active, *id, *reason),
        NotificationCommand::Action { id, key } => return invoke_action(active, *id, key),
    }
}

/// Closes a visible notification and emits `NotificationClosed`.
fn close(services: &PlasmaServices, id: u32, reason: u32) -> Response {
    let store = services.notifications();
    if !store.lock().close_visible(store.now_ms(), id, reason) {
        return Response::Err(format!("no visible notification with id {id}"));
    }
    match services.emit_closed(id, reason) {
        Ok(()) => return Response::Ok,
        Err(error) => {
            warn!(%error, id, "NotificationClosed was not delivered");
            return Response::Err(format!("{error:#}"));
        }
    }
}

/// Lists the action keys of a `Notify` call: the even elements of its alternating key and label list.
fn offered_keys(actions: &[String]) -> Vec<&str> {
    let mut keys = Vec::new();
    for key in actions.iter().step_by(2) {
        keys.push(key.as_str());
    }
    return keys;
}

/// Emits `ActionInvoked` for a visible notification that offered the action.
fn invoke_action(services: &PlasmaServices, id: u32, key: &str) -> Response {
    let store = services.notifications();
    {
        let mut guard = store.lock();
        let Some(shown) = guard.find(id) else {
            return Response::Err(format!("no visible notification with id {id}"));
        };
        let offered = offered_keys(&shown.call.actions);
        if !offered.contains(&key) {
            return Response::Err(format!(
                "notification {id} offers no action {key} (offered: {})",
                offered.join(", "),
            ));
        }
        guard.log_action(store.now_ms(), id, key);
    }
    match services.emit_action(id, key) {
        Ok(()) => return Response::Ok,
        Err(error) => {
            warn!(%error, id, "ActionInvoked was not delivered");
            return Response::Err(format!("{error:#}"));
        }
    }
}
