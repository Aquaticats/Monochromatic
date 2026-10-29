//! Records Unity-style taskbar progress signals that hosted apps emit on the private bus.
//!
//! An app sends `com.canonical.Unity.LauncherEntry.Update` with its URI and the properties
//!  that changed.
//!  The listener merges each change into what it last knew for that URI and appends the
//!  result to the bounded history.
//!  It listens only on the private bus, never on the host session bus.

/// Imports the per-URI property memory and the lock-protected history.
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Instant,
};

/// Imports error context for connection and thread setup.
use anyhow::{Context, Result};

/// Imports diagnostic macros for the listener's branch decisions.
use tracing::{debug, warn};

/// Imports the bus types the listener needs.
///
/// What:
///      `Connection` is the blocking client connection,
///  `MessageIterator` yields every message the connection receives,
///  `MatchRule` names the signals a connection wants,
///  `OwnedValue` holds one dynamically typed property,
///  and `Message` is one received message.
/// Why:
///       The listener subscribes with a match rule, then decodes each signal body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BusConnection, MatchRule, Message, Variant } from "./dbus";
/// ```
use zbus::{
    blocking::{fdo::DBusProxy, MessageIterator},
    message::{Message, Type},
    zvariant::OwnedValue,
    MatchRule,
};

/// Imports the bounded history the listener appends to.
use crate::{
    launcher_history::{LauncherHistory, LauncherUpdate},
    private_bus::PrivateBus,
};

/// The interface a taskbar progress sender emits its update on.
const LAUNCHER_INTERFACE: &str = "com.canonical.Unity.LauncherEntry";

/// The signal name that carries one update.
const LAUNCHER_MEMBER: &str = "Update";

/// The properties one launcher entry holds, as last reported.
///
/// What:
///      `pub struct LauncherProperties { ... }`.
///  One field per property a sender may send.
/// Why:
///       A sender may send only the properties that changed,
///  so the listener needs the earlier values to fill the rest.
#[derive(Clone, Debug, PartialEq)]
pub struct LauncherProperties {
    /// Progress between 0 and 1; `0` until the app reports one.
    pub progress: f64,
    /// Whether the progress bar is shown; `false` until the app shows it.
    pub progress_visible: bool,
    /// Badge number; `0` until the app reports one.
    pub count: i64,
    /// Whether the badge is shown; `false` until the app shows it.
    pub count_visible: bool,
    /// Whether the app asks for attention; `false` until the app asks.
    pub urgent: bool,
}

/// Defaults used before an app has sent any property.
impl Default for LauncherProperties {
    /// The state of a launcher entry no update has touched.
    ///
    /// What:
    ///      `fn default() -> Self`.
    /// Why:
    ///       A fresh entry shows no progress, no badge, and no attention request.
    fn default() -> Self {
        return Self {
            progress: 0.0,
            progress_visible: false,
            count: 0,
            count_visible: false,
            urgent: false,
        }
    }
}

/// Handle to the running listener and the history it fills.
///
/// What:
///      `pub struct LauncherLog { history: Arc<Mutex<LauncherHistory>> }`.
///  The listener thread holds the other `Arc`.
/// Why:
///       The control thread reads the history on the compositor thread,
///  while the listener appends from its own thread.
pub struct LauncherLog {
    /// The bounded history, shared with the listener thread.
    history: Arc<Mutex<LauncherHistory>>,
}

/// Starts the listener and reads its history.
impl LauncherLog {
    /// Subscribes to launcher updates on the private bus and starts the listener thread.
    ///
    /// What:
    ///      `pub fn start(bus: &PrivateBus, start_time: Instant) -> Result<Self>`.
    ///  The match rule is registered before this returns,
    ///  so an update sent after startup is never missed.
    /// Why:
    ///       `dbus-daemon` delivers a broadcast signal only to connections with a matching rule,
    ///  and `start_time` is the compositor's own clock for every timestamp.
    pub fn start(bus: &PrivateBus, start_time: Instant) -> Result<Self> {
        // What:     Open a client connection to the private bus address only.
        // Why:      The compositor's inherited `DBUS_SESSION_BUS_ADDRESS` is the host bus,
        //           and it must never be read here.
        let connection = zbus::blocking::connection::Builder::address(bus.address())?
            .build()
            .context("connecting the launcher listener to the private bus")?;

        // What:     Match only the signal name and interface, not the object path, because
        //           senders choose their own path under `/com/canonical/unity/launcherentry/`.
        // Why:      A rule without a match would receive nothing at all.
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .interface(LAUNCHER_INTERFACE)?
            .member(LAUNCHER_MEMBER)?
            .build();
        DBusProxy::new(&connection)
            .context("creating the bus proxy for the launcher listener")?
            .add_match_rule(rule)
            .context("subscribing the launcher listener to LauncherEntry.Update")?;

        // What:     `let messages = MessageIterator::from(&connection);`. Opens the message
        //           stream here, before the thread starts, and the iterator keeps its own
        //           connection handle alive.
        // Why:      A stream opened later could miss an update that arrived in between.
        let messages = MessageIterator::from(&connection);

        // What:     `let history = Arc::new(Mutex::new(LauncherHistory::default()));` then a
        //           second owner for the thread.
        // Why:      The thread appends while the control thread reads.
        let history = Arc::new(Mutex::new(LauncherHistory::default()));
        let shared = Arc::clone(&history);
        std::thread::Builder::new()
            .name("nws-launcher".to_owned())
            .spawn(move || return listen(messages, shared, start_time))
            .context("starting the launcher listener thread")?;
        debug!("launcher listener subscribed to LauncherEntry.Update on the private bus");
        return Ok(Self { history })
    }

    /// Locks the history for reading or clearing.
    ///
    /// What:
    ///      `pub fn lock(&self) -> MutexGuard<'_, LauncherHistory>`.
    /// Why:
    ///       A poisoned lock still holds consistent records,
    ///  because each push completes before the lock is released,
    ///  so the guard is recovered rather than panicking the compositor.
    pub fn lock(&self) -> MutexGuard<'_, LauncherHistory> {
        return self.history.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Reads signals until the private bus closes.
///
/// What:
///      `fn listen(messages: MessageIterator, history: Arc<Mutex<LauncherHistory>>, start_time: Instant)`.
///  Owns the subscribed message stream and the per-URI memory for the thread's lifetime.
/// Why:
///       The compositor exits when the bus goes away,
///  and the listener needs no separate shutdown signal.
fn listen(messages: MessageIterator, history: Arc<Mutex<LauncherHistory>>, start_time: Instant) {
    // What:     `let mut entries: HashMap<String, LauncherProperties> = HashMap::new();`.
    //           The last known properties for each app URI.
    // Why:      Partial updates merge into this memory.
    let mut entries: HashMap<String, LauncherProperties> = HashMap::new();
    for received in messages {
        match received {
            Ok(message) => record_message(&message, &mut entries, &history, start_time),
            Err(error) => {
                debug!(%error, "launcher listener stopped reading the private bus");
                return;
            }
        }
    }
    debug!("launcher listener ended because the private bus closed");
}

/// Decodes one update, merges it, and appends the result to the history.
///
/// What:
///      `fn record_message(message: &Message, entries: &mut HashMap<String, LauncherProperties>,
///           history: &Mutex<LauncherHistory>, start_time: Instant)`.
///  A body that does not match `(s, a{sv})` is logged and skipped.
/// Why:
///       One malformed signal from one app must not end the listener for every app.
fn record_message(
    message: &Message,
    entries: &mut HashMap<String, LauncherProperties>,
    history: &Mutex<LauncherHistory>,
    start_time: Instant,
) {
    // What:     `if message.message_type() != Type::Signal { return; }`. Drops replies to this
    //           listener's own bus calls, such as the `AddMatch` acknowledgement.
    // Why:      The stream also yields method returns, whose empty bodies are not updates,
    //           and only a signal can carry a launcher update.
    if message.message_type() != Type::Signal {
        debug!("launcher listener skipped a non-signal bus message");
        return;
    }

    // What:     `message.body().deserialize::<(String, HashMap<String, OwnedValue>)>()`.
    //           Decodes the URI and the changed-property map in one step.
    // Why:      The signature `(sa{sv})` is the documented shape of the update.
    let (app_uri, changes) = match message.body().deserialize::<(String, HashMap<String, OwnedValue>)>() {
        Ok(body) => body,
        Err(error) => {
            warn!(%error, "ignored a LauncherEntry.Update whose body does not match (sa{{sv}})");
            return;
        }
    };

    // What:     `let previous = entries.get(&app_uri).cloned().unwrap_or_default();`. The
    //           last known state of this app, or the defaults for a new app.
    // Why:      Keys absent from the update keep their previous value.
    let previous = entries.get(&app_uri).cloned().unwrap_or_default();
    let merged = merge_properties(previous, &changes);
    entries.insert(app_uri.clone(), merged.clone());

    // What:     `let update = LauncherUpdate { ... };`. The full state of this entry at the
    //           moment it arrived, stamped with the compositor's clock.
    // Why:      A test reads the whole state at each point, not only the changed keys.
    let update = LauncherUpdate {
        at_ms: start_time.elapsed().as_millis() as u64,
        app_uri,
        progress: merged.progress,
        progress_visible: merged.progress_visible,
        count: merged.count,
        count_visible: merged.count_visible,
        urgent: merged.urgent,
    };
    debug!(app_uri = %update.app_uri, progress = update.progress, "recorded a taskbar progress update");
    let mut guard = history.lock().unwrap_or_else(PoisonError::into_inner);
    guard.push(update);
}

/// Applies the keys present in one update over the previous properties.
///
/// What:
///      `pub fn merge_properties(previous: LauncherProperties, changes: &HashMap<String, OwnedValue>)
///           -> LauncherProperties`.
///  Each known key that holds a value of the expected type replaces the previous value.
///  An unknown key is ignored, and a key of the wrong type is logged and keeps the previous value.
/// Why:
///       Senders send only what changed,
///  and the Unity convention is that absent properties keep their state.
pub fn merge_properties(
    previous: LauncherProperties,
    changes: &HashMap<String, OwnedValue>,
) -> LauncherProperties {
    return LauncherProperties {
        progress: read_property(changes, "progress", previous.progress),
        progress_visible: read_property(changes, "progress-visible", previous.progress_visible),
        count: read_property(changes, "count", previous.count),
        count_visible: read_property(changes, "count-visible", previous.count_visible),
        urgent: read_property(changes, "urgent", previous.urgent),
    }
}

/// Reads one typed property, keeping `previous` when it is absent or the wrong type.
///
/// What:
///      `fn read_property<'a, T>(changes: &'a HashMap<String, OwnedValue>, key: &str,
///           previous: T) -> T`.
///  `T` is the Rust type the D-Bus variant must convert to.
/// Why:
///       One helper keeps the absent and wrong-type rules identical for every property.
fn read_property<'a, T>(changes: &'a HashMap<String, OwnedValue>, key: &str, previous: T) -> T
where
    T: TryFrom<&'a OwnedValue>,
    <T as TryFrom<&'a OwnedValue>>::Error: std::fmt::Display,
{
    // What:     `match changes.get(key) { None => previous, Some(value) => ... }`. An absent
    //           key keeps the previous value without any log.
    // Why:      Absence is the common partial-update case, not a problem.
    let Some(value) = changes.get(key) else {
        return previous;
    };
    match T::try_from(value) {
        Ok(typed) => return typed,
        Err(error) => {
            warn!(key, %error, "launcher property has an unexpected type; kept the previous value");
            return previous
        }
    }
}

/// Unit and bus tests for the listener.
#[cfg(test)]
#[path = "launcher_listen_tests.rs"]
mod tests;
