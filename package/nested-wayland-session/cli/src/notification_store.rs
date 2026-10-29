//! The state of the fake `org.freedesktop.Notifications` server.
//!
//! The store holds plain values only,
//!  with no D-Bus types,
//!  so id allocation,
//!  replacement,
//!  and the ordered log are unit-tested directly.
//! The service thread calls it while handling client calls,
//!  and the control thread reads it.

/// Imports `event_ring::EventRing`,
///  `value_summary::ValueSummary` from `crate`.
use crate::{event_ring::EventRing, value_summary::ValueSummary};

/// The `NotificationClosed` reason for a notification closed by a `CloseNotification` call.
pub const REASON_CLOSED_BY_CALL: u32 = 3;

/// One `Notify` call as the client sent it.
///
/// What:
///      The eight arguments of `Notify`,
///  hints already summarized.
/// Why:
///       One value carries the call from the service to the store and into the log.
#[derive(Clone, Debug, PartialEq)]
pub struct NotifyCall {
    /// The `app_name` argument.
    pub app_name: String,
    /// The `replaces_id` argument;
    ///  `0` asks for a new notification.
    pub replaces_id: u32,
    /// The `app_icon` argument.
    pub app_icon: String,
    /// The `summary` argument.
    pub summary: String,
    /// The `body` argument.
    pub body: String,
    /// The `actions` argument:
    ///  alternating action key and label.
    pub actions: Vec<String>,
    /// The `hints` argument,
    ///  summarized.
    pub hints: Vec<ValueSummary>,
    /// The `expire_timeout` argument in milliseconds;
    ///  `-1` is the server default and `0` never expires.
    pub expire_timeout: i32,
}

/// A notification the fake server currently shows.
#[derive(Clone, Debug, PartialEq)]
pub struct VisibleNotification {
    /// The id the `Notify` call returned.
    pub id: u32,
    /// The call that created or last replaced this notification.
    pub call: NotifyCall,
    /// How many later calls replaced it.
    pub replaced_count: u32,
    /// Milliseconds since the compositor started,
    ///  when it was last created or replaced.
    pub updated_ms: u64,
}

/// Something the fake server observed or did,
///  in order.
#[derive(Clone, Debug, PartialEq)]
pub enum NotificationEvent {
    /// A client called `Notify`.
    Notify {
        /// The id returned to the client.
        id: u32,
        /// The call.
        call: NotifyCall,
    },
    /// A client called `CloseNotification`.
    CloseCall {
        /// The id the client asked to close.
        id: u32,
        /// Whether that notification was visible.
        known: bool,
    },
    /// The server emitted `NotificationClosed`.
    Closed {
        /// The closed id.
        id: u32,
        /// The reason code in the signal.
        reason: u32,
    },
    /// The server emitted `ActionInvoked`.
    ActionInvoked {
        /// The notification id.
        id: u32,
        /// The action key.
        key: String,
    },
}

/// Visible notifications,
///  the id counter,
///  and the ordered log.
#[derive(Debug, Default)]
pub struct NotificationStore {
    /// The most recently allocated id;
    ///  the next allocation starts after it.
    last_id: u32,
    /// Visible notifications,
    ///  oldest first.
    visible: Vec<VisibleNotification>,
    /// Every call and signal,
    ///  oldest first.
    events: EventRing<NotificationEvent>,
}

/// Recording calls and signals,
///  and reading the result.
impl NotificationStore {
    /// Picks the next id:
    ///  after the last one,
    ///  never `0`,
    ///  never one that is visible.
    ///
    /// What:
    ///      `fn allocate_id(&mut self) -> u32`.
    ///  The counter wraps past `u32::MAX` to `1`.
    /// Why:
    ///       The specification promises a nonzero id that is not reused while the notification lives.
    fn allocate_id(&mut self) -> u32 {
        loop {
            self.last_id = self.last_id.wrapping_add(1);
            if self.last_id == 0 {
                self.last_id = 1;
            }
            if self.index_of(self.last_id).is_none() {
                return self.last_id;
            }
        }
    }

    /// Records a `Notify` call and returns the id for the reply.
    ///
    /// What:
    ///      `pub fn notify(&mut self, at_ms: u64, call: NotifyCall) -> u32`.
    ///  A `replaces_id` above zero is returned unchanged,
    ///  as Plasma does.
    ///  A visible notification with that id is replaced in place and counts the replacement;
    ///  an unknown id becomes a new visible notification with that id.
    /// Why:
    ///       Plasma's server never validates `replaces_id`,
    ///  and a recorder that did would hide app bugs.
    pub fn notify(&mut self, at_ms: u64, call: NotifyCall) -> u32 {
        let id = if call.replaces_id == 0 { self.allocate_id() } else { call.replaces_id };
        match self.index_of(id) {
            Some(index) => {
                if let Some(shown) = self.visible.get_mut(index) {
                    shown.replaced_count += 1;
                    shown.call = call.clone();
                    shown.updated_ms = at_ms;
                }
            }
            None => self.visible.push(VisibleNotification {
                id,
                call: call.clone(),
                replaced_count: 0,
                updated_ms: at_ms,
            }),
        }
        self.events.push(at_ms, NotificationEvent::Notify { id, call });
        return id;
    }

    /// Records a `CloseNotification` call and removes the notification when it is visible.
    ///
    /// What:
    ///      `pub fn close_call(&mut self, at_ms: u64, id: u32) -> bool`.
    ///  Returns whether the id was visible.
    ///  The caller emits `NotificationClosed` either way,
    ///  as Plasma does.
    pub fn close_call(&mut self, at_ms: u64, id: u32) -> bool {
        let known = self.remove(id).is_some();
        self.events.push(at_ms, NotificationEvent::CloseCall { id, known });
        return known;
    }

    /// Finds the position of a visible notification by id.
    fn index_of(&self, id: u32) -> Option<usize> {
        for (index, shown) in self.visible.iter().enumerate() {
            if shown.id == id {
                return Some(index);
            }
        }
        return None;
    }

    /// Removes a visible notification without logging.
    fn remove(&mut self, id: u32) -> Option<VisibleNotification> {
        let index = self.index_of(id)?;
        return Some(self.visible.remove(index));
    }

    /// Closes a visible notification for a control command and logs the signal about to be emitted.
    ///
    /// What:
    ///      `pub fn close_visible(&mut self, at_ms: u64, id: u32, reason: u32) -> bool`.
    ///  Returns `false` and logs nothing when the id is not visible.
    pub fn close_visible(&mut self, at_ms: u64, id: u32, reason: u32) -> bool {
        if self.remove(id).is_none() {
            return false;
        }
        self.events.push(at_ms, NotificationEvent::Closed { id, reason });
        return true;
    }

    /// Logs a `NotificationClosed` signal that answers a `CloseNotification` call.
    pub fn log_closed(&mut self, at_ms: u64, id: u32, reason: u32) {
        self.events.push(at_ms, NotificationEvent::Closed { id, reason });
    }

    /// Logs an `ActionInvoked` signal about to be emitted.
    pub fn log_action(&mut self, at_ms: u64, id: u32, key: &str) {
        self.events.push(at_ms, NotificationEvent::ActionInvoked { id, key: key.to_string() });
    }

    /// Returns a visible notification by id.
    #[must_use]
    pub fn find(&self, id: u32) -> Option<&VisibleNotification> {
        let index = self.index_of(id)?;
        return self.visible.get(index);
    }

    /// Returns the visible notifications,
    ///  oldest first.
    #[must_use]
    pub fn visible(&self) -> &[VisibleNotification] {
        return &self.visible;
    }

    /// Returns up to `count` of the newest log entries,
    ///  oldest first.
    #[must_use]
    pub fn last_events(&self, count: usize) -> Vec<&crate::event_ring::Timed<NotificationEvent>> {
        return self.events.last_n(count);
    }

    /// Forgets every visible notification and every log entry;
    ///  the id counter is kept.
    ///
    /// What:
    ///      `pub fn clear(&mut self)`.
    ///  No signal is emitted.
    /// Why:
    ///       A test resets between steps,
    ///  and an id handed out earlier must not come back.
    pub fn clear(&mut self) {
        self.visible.clear();
        self.events.clear();
    }

    /// Sets the id counter,
    ///  so tests can reach the wrap point without four billion calls.
    #[cfg(test)]
    pub fn set_last_id(&mut self, last_id: u32) {
        self.last_id = last_id;
    }
}

/// Unit tests for id allocation,
///  replacement,
///  and closing.
#[cfg(test)]
#[path = "notification_store_tests.rs"]
mod tests;
