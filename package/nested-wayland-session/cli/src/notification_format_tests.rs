//! The notification answers keep hostile app text on one line.

/// Brings the formatters into scope.
use super::*;

/// Value summary used to build a hint list.
use crate::value_summary::ValueSummary;

/// Builds a call whose every text field is hostile.
fn hostile_call() -> NotifyCall {
    return NotifyCall {
        app_name: "my app".to_string(),
        replaces_id: 0,
        app_icon: "icon;x".to_string(),
        summary: "Done\nerr injected".to_string(),
        body: "a=b; c,d:e".to_string(),
        actions: vec!["default".to_string(), "Open it".to_string(), "lonely".to_string()],
        hints: vec![ValueSummary {
            key: "urgency".to_string(),
            signature: "y".to_string(),
            text: "2".to_string(),
        }],
        expire_timeout: 5000,
    };
}

/// A visible notification prints every documented field and no raw separator inside values.
#[test]
fn visible_notification_line_has_the_documented_fields() {
    let shown = VisibleNotification { id: 4, call: hostile_call(), replaced_count: 2, updated_ms: 100 };
    let line = format_visible(&shown, 130);
    assert_eq!(
        line,
        "id=4 app=my%20app icon=icon%3Bx summary=Done%0Aerr%20injected body=a%3Db%3B%20c%2Cd%3Ae \
         actions=default=Open%20it,lonely= hints=urgency:y:2 expire=5000 replaced=2 age-ms=30",
    );
    assert!(!line.contains('\n'));
}

/// The list starts with its count and joins records with `; `.
#[test]
fn visible_list_starts_with_the_count() {
    assert_eq!(format_visible_list(&[], 0), "0");
    let shown = VisibleNotification { id: 1, call: hostile_call(), replaced_count: 0, updated_ms: 0 };
    let list = format_visible_list(&[shown.clone(), shown], 0);
    assert!(list.starts_with("2 id=1 "), "{list}");
    assert_eq!(list.matches("; id=1 ").count(), 1, "{list}");
}

/// Each event kind has its own record shape.
#[test]
fn event_records_cover_every_kind() {
    let notify = Timed { at_ms: 5, event: NotificationEvent::Notify { id: 3, call: hostile_call() } };
    assert!(format_event(&notify).starts_with("t=5 notify id=3 replaces=0 app=my%20app "));
    let close = Timed { at_ms: 6, event: NotificationEvent::CloseCall { id: 3, known: false } };
    assert_eq!(format_event(&close), "t=6 close-call id=3 known=false");
    let closed = Timed { at_ms: 7, event: NotificationEvent::Closed { id: 3, reason: 3 } };
    assert_eq!(format_event(&closed), "t=7 closed id=3 reason=3");
    let action = Timed { at_ms: 8, event: NotificationEvent::ActionInvoked { id: 3, key: "a b".to_string() } };
    assert_eq!(format_event(&action), "t=8 action-invoked id=3 key=a%20b");
    assert_eq!(format_event_list(&[&close, &closed]), "2 t=6 close-call id=3 known=false; t=7 closed id=3 reason=3");
}
