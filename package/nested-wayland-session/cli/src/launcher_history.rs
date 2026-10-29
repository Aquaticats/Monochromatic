//! Bounded record of taskbar progress updates observed on the private bus.
//!
//! The history holds plain values only, with no D-Bus types,
//!  so the ordering and eviction rules are unit-tested directly.
//!  The listener thread appends to it and the control thread reads it.

/// Imports the owned growable ring buffer that holds the updates in arrival order.
use std::collections::VecDeque;

/// Most updates kept before the oldest is dropped.
///
/// What:
///      `pub const LAUNCHER_HISTORY_CAPACITY: usize = 4096;`.
/// Why:
///       A chatty app can emit many updates; a fixed bound keeps memory flat
///  for a long session while still covering any test that reads recent history.
pub const LAUNCHER_HISTORY_CAPACITY: usize = 4096;

/// One `LauncherEntry.Update` after its partial properties were merged.
///
/// What:
///      `pub struct LauncherUpdate { ... }`.
///  Each field is the value the app's entry holds after this update.
/// Why:
///       A test reads the full state at each point in time, not only the fields that changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LauncherUpdate = { atMs: number; appUri: string; progress: number; progressVisible: boolean; count: bigint; countVisible: boolean; urgent: boolean };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct LauncherUpdate {
    /// Milliseconds since the compositor started, when the update arrived.
    pub at_ms: u64,
    /// The `app_uri` argument of the signal, such as `application://example.desktop`.
    pub app_uri: String,
    /// Progress between 0 and 1 as the app reports it.
    pub progress: f64,
    /// Whether the taskbar shows the progress bar.
    pub progress_visible: bool,
    /// Badge number the app reports.
    pub count: i64,
    /// Whether the taskbar shows the badge.
    pub count_visible: bool,
    /// Whether the app asks for attention.
    pub urgent: bool,
}

/// Oldest-first ring of the most recent updates.
///
/// What:
///      `pub struct LauncherHistory { updates: VecDeque<LauncherUpdate> }`.
///  A growable double-ended queue whose front is the oldest entry.
/// Why:
///       Appending at the back and evicting at the front keeps the order of arrival
///  without reallocating on every update.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class LauncherHistory { private updates: LauncherUpdate[] = []; }
/// ```
#[derive(Debug, Default)]
pub struct LauncherHistory {
    /// Updates in arrival order, oldest at the front.
    updates: VecDeque<LauncherUpdate>,
}

/// Appending, reading the newest entries, and clearing the history.
impl LauncherHistory {
    /// Appends an update and evicts the oldest once the capacity is reached.
    ///
    /// What:
    ///      `pub fn push(&mut self, update: LauncherUpdate)`.
    /// Why:
    ///       The bound applies per push, so the history never exceeds the capacity.
    pub fn push(&mut self, update: LauncherUpdate) {
        // What:     `if self.updates.len() == LAUNCHER_HISTORY_CAPACITY { pop_front(); }`.
        //           Drop the oldest entry before the append would exceed the bound.
        // Why:      The history is a window over the most recent updates.
        if self.updates.len() == LAUNCHER_HISTORY_CAPACITY {
            self.updates.pop_front();
        }
        self.updates.push_back(update);
    }

    /// Returns the newest update, if any has arrived since the last clear.
    ///
    /// What:
    ///      `pub fn latest(&self) -> Option<&LauncherUpdate>`.
    /// Why:
    ///       The `launcher` command reports the current state of the most recent update.
    #[must_use]
    pub fn latest(&self) -> Option<&LauncherUpdate> {
        return self.updates.back()
    }

    /// Returns up to `count` of the newest updates, oldest first.
    ///
    /// What:
    ///      `pub fn last_n(&self, count: usize) -> Vec<&LauncherUpdate>`.
    ///  Fewer than `count` come back when the history is shorter.
    /// Why:
    ///       `launcher-log` prints the recent window in arrival order so a test can compare
    ///  it with the sequence it emitted.
    #[must_use]
    pub fn last_n(&self, count: usize) -> Vec<&LauncherUpdate> {
        // What:     `let skip = self.updates.len().saturating_sub(count);`. How many of the
        //           oldest entries fall outside the window.
        // Why:      `saturating_sub` keeps a short history from underflowing.
        let skip = self.updates.len().saturating_sub(count);
        return self.updates.iter().skip(skip).collect()
    }

    /// Removes every stored update.
    ///
    /// What:
    ///      `pub fn clear(&mut self)`.
    /// Why:
    ///       A test resets the history between steps without restarting the compositor.
    pub fn clear(&mut self) {
        self.updates.clear();
    }
}

/// Unit tests for the bounded history.
#[cfg(test)]
#[path = "launcher_history_tests.rs"]
mod tests;
