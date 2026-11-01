//! Wire text for notification answers.
//!
//! Every answer stays on one line and uses only the separators the protocol documents,
//!  so text a hosted app chose cannot break the response framing.

/// Imports the items this module uses from `crate`.
use crate::{
    event_ring::Timed,
    notification_store::{NotificationEvent, NotifyCall, VisibleNotification},
    value_summary::format_summaries,
    wire_text::{encode_text, format_records},
};

/// Writes the actions of a call as `key=label` items separated by `,`.
///
/// What:
///      `fn format_actions(actions: &[String]) -> String`.
///  The list alternates key and label.
///  A trailing key with no label is written with an empty label.
/// Why:
///       A reader wants to see which key to pass to `notification-action`.
fn format_actions(actions: &[String]) -> String {
    let mut items = Vec::new();
    for pair in actions.chunks(2) {
        let key = pair.first().map_or("", String::as_str);
        let label = pair.get(1).map_or("", String::as_str);
        items.push(format!("{}={}", encode_text(key), encode_text(label)));
    }
    return items.join(",");
}

/// Writes the fields every call record shares:
///  app,
///  icon,
///  summary,
///  body,
///  actions,
///  hints,
///  timeout.
fn format_call_fields(call: &NotifyCall) -> String {
    return format!(
        "app={} icon={} summary={} body={} actions={} hints={} expire={}",
        encode_text(&call.app_name),
        encode_text(&call.app_icon),
        encode_text(&call.summary),
        encode_text(&call.body),
        format_actions(&call.actions),
        format_summaries(&call.hints),
        call.expire_timeout,
    );
}

/// Formats one visible notification.
///
/// What:
///      `pub fn format_visible(shown: &VisibleNotification, now_ms: u64) -> String`.
///  Fields:
///  `id`,
///  `app`,
///  `icon`,
///  `summary`,
///  `body`,
///  `actions`,
///  `hints`,
///  `expire`,
///  `replaced`,
///  `age-ms`.
/// Why:
///       One line carries everything a test compares after a `Notify` or a replacement.
#[must_use]
pub fn format_visible(shown: &VisibleNotification, now_ms: u64) -> String {
    return format!(
        "id={} {} replaced={} age-ms={}",
        shown.id,
        format_call_fields(&shown.call),
        shown.replaced_count,
        now_ms.saturating_sub(shown.updated_ms),
    );
}

/// Formats the `notifications` answer payload:
///  a count,
///  then the visible notifications.
#[must_use]
pub fn format_visible_list(visible: &[VisibleNotification], now_ms: u64) -> String {
    let mut records = Vec::with_capacity(visible.len());
    for shown in visible {
        records.push(format_visible(shown, now_ms));
    }
    return format_records(&records);
}

/// Formats one log record:
///  the time,
///  the kind,
///  and the fields of that kind.
///
/// What:
///      `pub fn format_event(timed: &Timed<NotificationEvent>) -> String`.
///  Kinds:
///  `notify`,
///  `close-call`,
///  `closed` (the `NotificationClosed` signal),
///  and `action-invoked` (the `ActionInvoked` signal).
#[must_use]
pub fn format_event(timed: &Timed<NotificationEvent>) -> String {
    let detail = match &timed.event {
        NotificationEvent::Notify { id, call } => {
            format!("notify id={id} replaces={} {}", call.replaces_id, format_call_fields(call))
        }
        NotificationEvent::CloseCall { id, known } => format!("close-call id={id} known={known}"),
        NotificationEvent::Closed { id, reason } => format!("closed id={id} reason={reason}"),
        NotificationEvent::ActionInvoked { id, key } => {
            format!("action-invoked id={id} key={}", encode_text(key))
        }
    };
    return format!("t={} {detail}", timed.at_ms);
}

/// Formats the `notification-log` answer payload:
///  a count,
///  then the records.
#[must_use]
pub fn format_event_list(events: &[&Timed<NotificationEvent>]) -> String {
    let mut records = Vec::with_capacity(events.len());
    for timed in events {
        records.push(format_event(timed));
    }
    return format_records(&records);
}

/// Unit tests for the answer text.
#[cfg(test)]
#[path = "notification_format_tests.rs"]
mod tests;
