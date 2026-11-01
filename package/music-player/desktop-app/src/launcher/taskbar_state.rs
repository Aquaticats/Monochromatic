//! The decision logic of the KDE taskbar progress bar, free of D-Bus and of the window.
//!
//! The taskbar bar has one meaning at a time: where the current track is, as a whole percent.
//!  It is hidden whenever
//! nothing is loaded.
//!  `TaskbarProgress` is a small state machine that is fed the engine's own messages and answers with
//! the next `LauncherEntry` value to send, or with nothing when the bar already shows the right thing.
//!
//! The rules, each of which fixes a jump seen in the capture of the old behavior:
//!
//! - The state is built from the messages alone, never from the window's `position` and `duration` properties.
//!    A new track resets the position and sets the new duration in one step, so an old position is never divided by
//!   a new duration.
//! - The value is a whole percent, sent as exactly `percent / 100`, because Plasma shows
//!   `qRound(progress * 100)` and a float such as `0.995` would sit on a rounding edge.
//! - While playing, a change of the percent is sent at most once per `MIN_UPDATE_GAP`, so a short track or a dragged
//!   seek bar cannot flood the bus; a held change is sent by the next playback tick.
//!    Pausing, resuming, the first seek after ordinary playback, a new track, going idle, and every change while paused
//!   are never held back, because no later tick would send them.
//! - Within a track the percent never moves backward except by a seek.

use std::time::Duration;

use crate::command::Update;

/// What: the shortest time between two sent percent changes that are caused by ordinary playback ticks.
/// Why: Plasma draws a coarse bar, so a faster rate adds bus traffic and no visible change.
pub const MIN_UPDATE_GAP: Duration = Duration::from_millis(250);

/// What: how far, in seconds, the position may move forward from the last known one and still be an ordinary tick.
/// Why: the engine reports every 0.1 s of audio and its output buffer can add about a second at a start or a resume,
/// so a larger forward step is a seek.
pub const SEEK_JUMP_SECS: f64 = 2.0;

/// What: how far, in seconds, the position may fall behind the last known one before it counts as a seek.
/// Why: the engine's positions only go backward through a seek (it counts decoded samples), so anything beyond float
/// noise is one; a 1 s step back on a short track must move the bar.
pub const BACKWARD_NOISE_SECS: f64 = 0.001;

/// What: how many seconds before the end of a track a stopped player counts as finished.
/// Why: at the end of the queue the engine only reports "not playing" and leaves the last track loaded; a bar stuck
/// at the end of a track nobody is playing would mislead.
pub const FINISHED_ZONE_SECS: f64 = 0.5;

/// What: how much of a percent (a millionth) a position may fall short of a whole percent and still count as reaching it.
/// Why: `1.2 / 120.0 * 100.0` is not exactly `1.0` in floating point; without the tolerance the bar would lag a whole
/// tick behind at every such edge.
const PERCENT_EDGE_TOLERANCE: f64 = 1e-6;

/// What: one input of the state machine.
/// Why: the machine knows nothing about the engine's `Update` type, so tests can drive it with plain values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TaskbarInput {
    /// What: a track was loaded, with its length in seconds; the position restarts at zero.
    /// Why: this is the only place the duration and the position change together.
    NowPlaying { duration_secs: f64 },
    /// What: an ordinary playback tick, in seconds from the start of the track.
    /// Why: a tick that would move backward is dropped as stale.
    Position { secs: f64 },
    /// What: the position jumped, in seconds from the start of the track.
    /// Why: a seek is the only way the percent may move backward; the first one after ordinary playback is sent at
    /// once, and the rest of a drag are held to the rate limit while playing.
    Seek { secs: f64 },
    /// What: playback started (`true`) or paused or stopped (`false`).
    /// Why: it decides whether a paused track at position zero shows a bar.
    Playing(bool),
    /// What: no track is loaded.
    /// Why: the bar is hidden and the position is forgotten.
    Idle,
}

/// What: the value to send to the launcher entry: a whole percent and whether the bar is shown.
/// Why: comparing two of these decides whether anything needs to be sent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskbarUpdate {
    /// What: the whole percent, 0 to 100; always 0 while hidden.
    /// Why: a hidden bar has one spelling, so two hidden states never look different.
    pub percent: u8,
    /// What: whether the bar is shown.
    /// Why: Plasma hides the bar when this is false.
    pub visible: bool,
}

impl TaskbarUpdate {
    /// What: the bar hidden.
    /// Why: the one spelling of "nothing to show".
    pub const HIDDEN: TaskbarUpdate = TaskbarUpdate {
        percent: 0,
        visible: false,
    };

    /// What: the percent as the 0 to 1 fraction the `LauncherEntry` signal carries.
    /// Why: `percent / 100` rounds back to exactly `percent` under Plasma's `qRound(progress * 100)`.
    pub fn fraction(&self) -> f64 {
        return f64::from(self.percent) / 100.0;
    }
}

/// What: the last value sent and when it was sent.
/// Why: the next value is compared with it, and the gap is measured from it.
#[derive(Clone, Copy, Debug)]
struct Sent {
    value: TaskbarUpdate,
    at: Duration,
    /// What: whether a seek caused this send.
    /// Why: the next seek of a drag is held to the rate limit, the first one is not.
    by_seek: bool,
}

/// What: the state machine behind the taskbar bar.
/// Why: it is the single owner of what the bar means; the window's own properties are not consulted.
#[derive(Debug, Default)]
pub struct TaskbarProgress {
    /// What: the length of the loaded track in seconds; `None` when nothing valid is loaded.
    /// Why: a missing or zero length has no percent, so the bar is hidden.
    duration_secs: Option<f64>,
    /// What: where the loaded track is, in seconds.
    /// Why: the percent is computed from it and the duration only.
    position_secs: f64,
    /// What: whether playback is running.
    /// Why: a stopped track at its end hides the bar, and a track change while playing keeps it.
    playing: bool,
    /// What: whether the bar has been in use for the loaded track or the one before it: playback started, or the
    /// position moved.
    /// Why: pausing, or selecting another row (the engine reports "not playing" and then "playing" within a
    /// millisecond), must not hide and show the bar; only a fresh app with a loaded, untouched track shows nothing.
    armed: bool,
    /// What: the last value sent, or `None` before the first send.
    /// Why: a repeat of the same value is never sent.
    sent: Option<Sent>,
}

impl TaskbarProgress {
    /// What: a machine with nothing loaded and nothing sent.
    /// Why: the starting state of the app.
    pub const fn new() -> TaskbarProgress {
        return TaskbarProgress {
            duration_secs: None,
            position_secs: 0.0,
            playing: false,
            armed: false,
            sent: None,
        };
    }

    /// What: the machine's own reading of what the bar should show now.
    /// Why: one place decides the precedence: no track means hidden, a finished stopped track means hidden, a
    /// track whose bar is in use (playing, paused after playing, or moved) shows its percent.
    fn desired(&self) -> TaskbarUpdate {
        let Some(duration) = self.duration_secs else {
            return TaskbarUpdate::HIDDEN;
        };
        let finished = !self.playing && duration - self.position_secs <= FINISHED_ZONE_SECS;
        if finished {
            return TaskbarUpdate::HIDDEN;
        }
        if !self.playing && !self.armed {
            return TaskbarUpdate::HIDDEN;
        }
        let whole = (self.position_secs / duration * 100.0 + PERCENT_EDGE_TOLERANCE).floor().clamp(0.0, 100.0);
        // The clamp above keeps the value inside 0 to 100, so the cast cannot truncate.
        return TaskbarUpdate {
            percent: whole as u8,
            visible: true,
        };
    }

    /// What: classifies one engine message as a state machine input, or `None` when it does not concern the bar.
    /// Why: callers also need the class, because a seek must pass the seek bar's own rate limit; it reads the
    /// machine's position to tell a seek from a tick.
    pub fn input_for(&self, update: &Update) -> Option<TaskbarInput> {
        match update {
            Update::NowPlaying { index, duration, .. } => {
                if index.is_none() {
                    return Some(TaskbarInput::Idle);
                }
                return Some(TaskbarInput::NowPlaying {
                    duration_secs: *duration,
                });
            }
            Update::Reconciled { index, .. } => {
                // The same track keeps playing across a rescan, so only a vanished track changes the bar.
                if index.is_none() {
                    return Some(TaskbarInput::Idle);
                }
                return None;
            }
            Update::Position(secs) => {
                let step = *secs - self.position_secs;
                if !(-BACKWARD_NOISE_SECS..=SEEK_JUMP_SECS).contains(&step) {
                    return Some(TaskbarInput::Seek { secs: *secs });
                }
                return Some(TaskbarInput::Position { secs: *secs });
            }
            Update::Playing(on) => return Some(TaskbarInput::Playing(*on)),
            _ => return None,
        }
    }

    /// What: feeds one engine message and returns the value to send, if any.
    /// Why: the whole bridge from engine messages to the launcher entry is this call.
    pub fn apply_update(&mut self, update: &Update, now: Duration) -> Option<TaskbarUpdate> {
        let input = self.input_for(update)?;
        return self.apply(input, now);
    }

    /// What: feeds one input and returns the value to send, if any.
    /// Why: it updates the state first, then asks `desired` what the bar should show and whether it may be sent now.
    pub fn apply(&mut self, input: TaskbarInput, now: Duration) -> Option<TaskbarUpdate> {
        tracing::trace!(?input, "taskbar progress input");
        match input {
            TaskbarInput::NowPlaying { duration_secs } => {
                if duration_secs.is_finite() && duration_secs > 0.0 {
                    self.duration_secs = Some(duration_secs);
                    self.armed = self.armed || self.playing;
                } else {
                    tracing::debug!(duration_secs, "taskbar progress: track without a usable length, bar hidden");
                    self.duration_secs = None;
                    self.armed = false;
                }
                self.position_secs = 0.0;
            }
            TaskbarInput::Position { secs } => {
                if self.duration_secs.is_none() || !secs.is_finite() || secs < self.position_secs {
                    tracing::trace!(secs, "taskbar progress: tick dropped (no track, not a number, or behind)");
                    return None;
                }
                self.position_secs = secs;
                self.armed = self.armed || secs > 0.0;
            }
            TaskbarInput::Seek { secs } => {
                if self.duration_secs.is_none() || !secs.is_finite() || secs < 0.0 {
                    tracing::debug!(secs, "taskbar progress: seek dropped (no track or not a valid position)");
                    return None;
                }
                self.position_secs = secs;
                self.armed = self.armed || secs > 0.0;
            }
            TaskbarInput::Playing(on) => {
                self.playing = on;
                self.armed = self.armed || (on && self.duration_secs.is_some());
            }
            TaskbarInput::Idle => {
                self.duration_secs = None;
                self.position_secs = 0.0;
                self.armed = false;
            }
        }
        let desired = self.desired();
        if let Some(sent) = self.sent {
            if sent.value == desired {
                return None;
            }
            let gap = now.saturating_sub(sent.at);
            // Only while playing: the next playback tick then sends whatever is held. A paused bar has no next tick.
            let may_hold = self.playing
                && match input {
                    TaskbarInput::Position { .. } => true,
                    TaskbarInput::Seek { .. } => sent.by_seek,
                    _ => false,
                };
            if may_hold && gap < MIN_UPDATE_GAP {
                tracing::trace!(percent = desired.percent, "taskbar progress: change held for the rate limit");
                return None;
            }
        }
        tracing::debug!(percent = desired.percent, visible = desired.visible, ?input, "taskbar progress: send");
        self.sent = Some(Sent {
            value: desired,
            at: now,
            by_seek: matches!(input, TaskbarInput::Seek { .. }),
        });
        return Some(desired);
    }
}

#[cfg(test)]
#[path = "taskbar_state_tests.rs"]
mod tests;
