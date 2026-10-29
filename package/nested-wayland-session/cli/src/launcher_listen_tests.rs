//! Merge rules and a real private-bus round trip for taskbar progress.

/// Brings the listener, its property record, and the error helpers into scope.
use super::*;

/// Value types a test puts into a `LauncherEntry.Update` body.
use zbus::zvariant::Value;

/// Bounds how long a test waits for an update that must already be on the private bus.
const UPDATE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Builds the previous state with progress shown, for merge checks.
fn shown_at(progress: f64) -> LauncherProperties {
    return LauncherProperties { progress, progress_visible: true, ..LauncherProperties::default() }
}

/// A key absent from an update keeps the previous value.
#[test]
fn absent_keys_keep_their_previous_values() {
    let changes: HashMap<String, OwnedValue> =
        HashMap::from([("count".to_owned(), OwnedValue::from(3_i64))]);
    let merged = merge_properties(shown_at(0.25), &changes);
    assert_eq!(merged.progress, 0.25);
    assert!(merged.progress_visible);
    assert_eq!(merged.count, 3);
    assert!(!merged.count_visible);
    assert!(!merged.urgent);
}

/// A key holding the wrong type is ignored and the previous value survives.
#[test]
fn wrong_typed_keys_keep_their_previous_values() {
    let changes: HashMap<String, OwnedValue> =
        HashMap::from([("progress".to_owned(), OwnedValue::from(true))]);
    let merged = merge_properties(shown_at(0.5), &changes);
    assert_eq!(merged.progress, 0.5);
}

/// An unknown key changes nothing.
#[test]
fn unknown_keys_change_nothing() {
    let changes: HashMap<String, OwnedValue> =
        HashMap::from([("badge-colour".to_owned(), OwnedValue::from(7_u32))]);
    assert_eq!(merge_properties(shown_at(0.75), &changes), shown_at(0.75));
}

/// A new app with no properties yet gets the defaults.
#[test]
fn empty_update_yields_defaults() {
    assert_eq!(merge_properties(LauncherProperties::default(), &HashMap::new()), LauncherProperties::default());
}

/// Polls the history until its newest update satisfies `ready`, or fails at the timeout.
fn wait_for_latest(log: &LauncherLog, ready: impl Fn(&LauncherUpdate) -> bool) -> anyhow::Result<LauncherUpdate> {
    let started = std::time::Instant::now();
    while started.elapsed() < UPDATE_TIMEOUT {
        let newest = log.lock().latest().cloned();
        if let Some(update) = newest.filter(|update| return ready(update)) {
            return Ok(update);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    anyhow::bail!("timed out waiting for a launcher update on the private bus")
}

/// Emitted updates reach the listener over the private bus, merge partial sends,
///  and ignore signals from other interfaces.
#[test]
fn private_bus_round_trip_records_and_merges_updates() -> anyhow::Result<()> {
    let bus = PrivateBus::start()?;
    let log = LauncherLog::start(&bus, std::time::Instant::now())?;
    let sender = zbus::blocking::connection::Builder::address(bus.address())?.build()?;
    let path = "/com/canonical/unity/launcherentry/7";
    let uri = "application://org.example.progress.desktop";

    // Negative control: a signal on another interface must not appear in the history.
    let unrelated: HashMap<&str, Value<'_>> = HashMap::from([("progress", Value::from(0.9_f64))]);
    sender.emit_signal(None::<&str>, path, "org.example.Other", "Update", &(uri, unrelated))?;

    let first: HashMap<&str, Value<'_>> = HashMap::from([
        ("progress", Value::from(0.25_f64)),
        ("progress-visible", Value::from(true)),
    ]);
    sender.emit_signal(None::<&str>, path, LAUNCHER_INTERFACE, LAUNCHER_MEMBER, &(uri, first))?;
    let second: HashMap<&str, Value<'_>> = HashMap::from([("count", Value::from(3_i64))]);
    sender.emit_signal(None::<&str>, path, LAUNCHER_INTERFACE, LAUNCHER_MEMBER, &(uri, second))?;

    let latest = wait_for_latest(&log, |update| return update.count == 3)?;
    assert_eq!(latest.app_uri, uri);
    assert_eq!(latest.progress, 0.25);
    assert!(latest.progress_visible);
    assert!(!latest.count_visible);

    let history = log.lock();
    let progress: Vec<f64> = history.last_n(64).iter().map(|record| return record.progress).collect();
    assert_eq!(progress, [0.25, 0.25]);
    return Ok(())
}
