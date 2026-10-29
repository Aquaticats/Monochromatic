//! Bounded-history rules for taskbar progress updates.

/// Brings the history and its update record into scope.
use super::*;

/// Builds an update whose progress encodes its arrival index, so order is checkable.
fn update(index: u64) -> LauncherUpdate {
    return LauncherUpdate {
        at_ms: index,
        app_uri: format!("application://app-{index}.desktop"),
        progress: index as f64 / 1000.0,
        progress_visible: true,
        count: 0,
        count_visible: false,
        urgent: false,
    }
}

/// An empty history has no latest update and returns nothing.
#[test]
fn empty_history_has_no_latest_and_no_records() {
    let history = LauncherHistory::default();
    assert_eq!(history.latest(), None);
    assert!(history.last_n(64).is_empty());
}

/// The latest entry is the last one pushed, and records come back oldest first.
#[test]
fn records_come_back_in_arrival_order() {
    let mut history = LauncherHistory::default();
    for index in 0..5 {
        history.push(update(index));
    }
    assert_eq!(history.latest(), Some(&update(4)));
    let recent: Vec<u64> = history.last_n(3).iter().map(|record| return record.at_ms).collect();
    assert_eq!(recent, [2, 3, 4]);
}

/// Asking for more records than exist returns every record without padding.
#[test]
fn last_n_larger_than_history_returns_everything() {
    let mut history = LauncherHistory::default();
    history.push(update(7));
    history.push(update(8));
    let all: Vec<u64> = history.last_n(64).iter().map(|record| return record.at_ms).collect();
    assert_eq!(all, [7, 8]);
}

/// The history keeps only the newest capacity entries and evicts the oldest first.
#[test]
fn history_is_bounded_and_evicts_oldest() {
    let mut history = LauncherHistory::default();
    let total = LAUNCHER_HISTORY_CAPACITY as u64 + 3;
    for index in 0..total {
        history.push(update(index));
    }
    let kept = history.last_n(usize::MAX);
    assert_eq!(kept.len(), LAUNCHER_HISTORY_CAPACITY);
    assert_eq!(kept.first().map(|record| return record.at_ms), Some(3));
    assert_eq!(kept.last().map(|record| return record.at_ms), Some(total - 1));
}

/// Clearing empties the history so the next latest query sees nothing.
#[test]
fn clear_removes_every_record() {
    let mut history = LauncherHistory::default();
    history.push(update(1));
    history.clear();
    assert_eq!(history.latest(), None);
    assert!(history.last_n(64).is_empty());
}
