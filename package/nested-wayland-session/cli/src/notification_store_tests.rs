//! Id allocation,
//!  replacement,
//!  and closing of the fake notification store.

/// Brings the store and its types into scope.
use super::*;

/// Builds a `Notify` call with the given replacement id and summary.
fn call(replaces_id: u32, summary: &str) -> NotifyCall {
    return NotifyCall {
        app_name: "app".to_string(),
        replaces_id,
        app_icon: String::new(),
        summary: summary.to_string(),
        body: String::new(),
        actions: vec!["open".to_string(), "Open".to_string()],
        hints: Vec::new(),
        expire_timeout: -1,
    };
}

/// New notifications get increasing ids starting at one.
#[test]
fn ids_start_at_one_and_increase() {
    let mut store = NotificationStore::default();
    assert_eq!(store.notify(1, call(0, "a")), 1);
    assert_eq!(store.notify(2, call(0, "b")), 2);
    assert_eq!(store.visible().len(), 2);
}

/// A replacement keeps the id and position,
///  counts itself,
///  and takes the new content.
#[test]
fn replacement_keeps_the_id_and_counts() {
    let mut store = NotificationStore::default();
    let first = store.notify(1, call(0, "a"));
    store.notify(2, call(0, "other"));
    assert_eq!(store.notify(5, call(first, "a2")), first);
    assert_eq!(store.notify(9, call(first, "a3")), first);
    let shown = store.find(first).expect("the notification is still visible");
    assert_eq!((shown.call.summary.as_str(), shown.replaced_count, shown.updated_ms), ("a3", 2, 9));
    assert_eq!(store.visible().first().map(|entry| return entry.id), Some(first));
    assert_eq!(store.visible().len(), 2);
}

/// An unknown replacement id is returned unchanged and becomes visible,
///  as on Plasma.
#[test]
fn unknown_replacement_id_is_accepted() {
    let mut store = NotificationStore::default();
    assert_eq!(store.notify(1, call(77, "x")), 77);
    assert_eq!(store.find(77).map(|entry| return entry.replaced_count), Some(0));
    // The counter has not moved, and the occupied id is never handed out again.
    store.set_last_id(76);
    assert_eq!(store.notify(2, call(0, "y")), 78);
}

/// The counter wraps past the largest id to one and never yields zero.
#[test]
fn ids_wrap_without_zero() {
    let mut store = NotificationStore::default();
    store.set_last_id(u32::MAX - 1);
    assert_eq!(store.notify(1, call(0, "a")), u32::MAX);
    assert_eq!(store.notify(2, call(0, "b")), 1);
}

/// A close call removes a visible notification and reports unknown ids without failing.
#[test]
fn close_call_reports_whether_the_id_was_visible() {
    let mut store = NotificationStore::default();
    let id = store.notify(1, call(0, "a"));
    assert!(store.close_call(2, id));
    assert!(!store.close_call(3, id));
    assert!(store.visible().is_empty());
    let kinds: Vec<&NotificationEvent> = store.last_events(10).iter().map(|timed| return &timed.event).collect();
    assert!(matches!(kinds[1], NotificationEvent::CloseCall { known: true, .. }));
    assert!(matches!(kinds[2], NotificationEvent::CloseCall { known: false, .. }));
}

/// A control close logs the signal only for visible ids.
#[test]
fn control_close_logs_only_visible_ids() {
    let mut store = NotificationStore::default();
    let id = store.notify(1, call(0, "a"));
    assert!(!store.close_visible(2, id + 1, 2));
    assert!(store.close_visible(3, id, 2));
    assert!(!store.close_visible(4, id, 2));
    assert_eq!(store.last_events(10).len(), 2);
}

/// Clearing forgets notifications and the log but not the counter.
#[test]
fn clear_keeps_the_counter() {
    let mut store = NotificationStore::default();
    store.notify(1, call(0, "a"));
    store.clear();
    assert!(store.visible().is_empty());
    assert!(store.last_events(10).is_empty());
    assert_eq!(store.notify(2, call(0, "b")), 2);
}
