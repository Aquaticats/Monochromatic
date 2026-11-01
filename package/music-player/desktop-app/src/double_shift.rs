//! Double tap of Shift: the detector that opens Search (D106).
//!
//! What: `DoubleShift::step` takes one keyboard input and the time it happened, and returns the next state together
//! with whether a double tap just completed.
//! Why: D106 opens Search on two presses and releases of Shift within 400 ms with no other key between them. The rule
//! is pure logic with injected time, so every timing case is tested without a window or a clock.

/// What:
///      `DOUBLE_SHIFT_WINDOW_MS` is the longest time from the first press to the second release of a double tap.
/// Why:
///      D106 fixes it at 400 ms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DOUBLE_SHIFT_WINDOW_MS = 400;
/// ```
pub const DOUBLE_SHIFT_WINDOW_MS: u64 = 400;

/// What:
///      `pub enum ShiftInput` is one keyboard input the detector reads.
/// Why:
///      The detector needs to know only whether Shift went down or up, and whether any other key was pressed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShiftInput = { kind: "shift-down"; repeat: boolean } | { kind: "shift-up" } | { kind: "other-key" };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftInput {
    /// A Shift key went down; `repeat` is true for an auto-repeat of a Shift that is already held.
    ShiftDown {
        /// True when the platform reports the press as a repeat.
        repeat: bool,
    },
    /// A Shift key went up.
    ShiftUp,
    /// Any key other than Shift went down.
    OtherKey,
}

/// What:
///      `pub struct DoubleShift` is the detector state: the press in progress and the tap that completed before it.
/// Why:
///      Two taps make a double tap only when the first was clean, the second was clean, and both fit the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DoubleShift = { held: { at: number; clean: boolean } | null; firstTapAt: number | null };
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DoubleShift {
    /// The Shift press in progress: when it went down and whether no other key came during it.
    held: Option<(u64, bool)>,
    /// When the press of the first clean tap went down, until the second tap completes or something interrupts.
    first_tap_at: Option<u64>,
}

/// Methods of `DoubleShift`.
impl DoubleShift {
    /// What:
    ///      `step` applies one input at `now_ms` and returns the next state and whether a double tap completed.
    /// Why:
    ///      A repeat of a held Shift is ignored, another key resets the first tap and spoils a press in progress, and
    ///      the second clean release inside the window completes the double tap.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function step(state: DoubleShift, input: ShiftInput, nowMs: number): [DoubleShift, boolean];
    /// ```
    pub fn step(self, input: ShiftInput, now_ms: u64) -> (Self, bool) {
        return match input {
            ShiftInput::ShiftDown { repeat } => (self.press(repeat, now_ms), false),
            ShiftInput::OtherKey => (self.interrupt(), false),
            ShiftInput::ShiftUp => self.release(now_ms),
        };
    }

    /// What:
    ///      `press` records a Shift going down unless it repeats a Shift already held.
    /// Why:
    ///      Holding Shift sends repeats, which must not count as taps; a second Shift key pressed while one is held is
    ///      the same press.
    fn press(self, repeat: bool, now_ms: u64) -> Self {
        if repeat || self.held.is_some() {
            return self;
        }
        return Self { held: Some((now_ms, true)), first_tap_at: self.first_tap_at };
    }

    /// What:
    ///      `interrupt` handles any key other than Shift.
    /// Why:
    ///      A key between the taps cancels the double tap, and a key during a press makes it a chord such as Shift+A.
    fn interrupt(self) -> Self {
        return Self { held: self.held.map(|(at, _clean)| return (at, false)), first_tap_at: None };
    }

    /// What:
    ///      `release` handles a Shift going up and reports a completed double tap.
    /// Why:
    ///      A clean press is a tap: the first tap is remembered by its press time, and a second clean tap whose release
    ///      is within the window of the first press completes the double tap.
    fn release(self, now_ms: u64) -> (Self, bool) {
        let Some((pressed_at, clean)) = self.held else {
            return (self, false);
        };
        if !clean {
            return (Self::default(), false);
        }
        return match self.first_tap_at {
            Some(first_at) if now_ms.saturating_sub(first_at) <= DOUBLE_SHIFT_WINDOW_MS => (Self::default(), true),
            _ => (Self { held: None, first_tap_at: Some(pressed_at) }, false),
        };
    }
}

/// Unit tests in `double_shift_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "double_shift_tests.rs"]
mod tests;
