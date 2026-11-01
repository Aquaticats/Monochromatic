//! Pointer selection by character,
//!  word,
//!  or line:
//!  counting clicks and extending a drag by whole units.

/// Word and line ranges come from the same rope functions as keyboard movement.
use crate::caret_motion::{line_range, word_range};
/// What:
///  Helix's borrowed view of the document rope.
/// Why:
///  Units are looked up in the text without copying it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { RopeSlice } from 'helix-core';
/// ```
use helix_core::RopeSlice;
/// What:
///  `Instant` is a point on a clock that never goes backwards;
///  `Duration` is a span of time.
/// Why:
///  A multi-click is defined by the time between presses,
///  which wall-clock adjustments must not change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::time::{Duration, Instant};

/// Longest pause between two presses of one multi-click;
///  the toolkit's own double-click default.
pub const CLICK_INTERVAL: Duration = Duration::from_millis(500);

/// What:
///  `f32` is a 32-bit float (sibling `f64`),
///  the unit of logical pixels.
/// Why:
///  A hand does not hold the pointer perfectly still between the presses of a double click.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLICK_SLOP = 4;
/// ```
pub const CLICK_SLOP: f32 = 4.0;

/// What:
///  `enum` lists the selection units as named variants;
///  `derive` generates copying and comparison.
/// Why:
///  The unit chosen by the press also governs how the following drag extends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Granularity = 'character' | 'word' | 'line';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Granularity {
    /// A single press:
    ///  caret placement and character-wise drag.
    Character,
    /// A double press:
    ///  whole words.
    Word,
    /// A triple press:
    ///  whole lines including their terminator.
    Line,
}

/// What:
///  The previous press;
///  `usize` is an address-sized row index (siblings `u32`,
///  `u64`).
/// Why:
///  The next press continues a multi-click only when it is soon enough and close enough.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Press = { at: number; row: number; x: number; count: number };
/// ```
#[derive(Clone, Copy)]
struct Press {
    /// When the button went down.
    at: Instant,
    /// Visual row under the pointer,
    /// counted from the first row of the text.
    row: usize,
    /// Logical x inside the source text.
    x: f32,
    /// Position of this press in its multi-click,
    ///  starting at one.
    count: u8,
}

/// What:
///  `Option<Press>` holds either the previous press or nothing (TypeScript's `Press | undefined`).
/// Why:
///  The first press of a session has no predecessor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ClickCounter { private last?: Press }
/// ```
#[derive(Default)]
pub struct ClickCounter {
    /// The most recent press,
    ///  if any.
    last: Option<Press>,
}

/// Count presses into single,
///  double,
///  and triple clicks.
impl ClickCounter {
    /// Record a press and return the unit it selects.
    ///
    /// A press continues the multi-click when it follows within [`CLICK_INTERVAL`] on the same visual row
    /// and within [`CLICK_SLOP`] pixels.
    ///  A fourth press starts over with a single click.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// press(at: number, row: number, x: number): Granularity;
    /// ```
    pub fn press(&mut self, at: Instant, row: usize, x: f32) -> Granularity {
        let mut count = 1;
        // What: `if let Some(previous)` runs only when an earlier press exists.
        // Why: Only then is there a pause and a distance to compare.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (this.last && soonEnough && closeEnough && this.last.count < 3) count = this.last.count + 1;
        // ```
        if let Some(previous) = self.last
            && at.saturating_duration_since(previous.at) <= CLICK_INTERVAL
            && previous.row == row
            && (previous.x - x).abs() <= CLICK_SLOP
            && previous.count < 3
        {
            count = previous.count + 1;
        }
        self.last = Some(Press { at, row, x, count });
        if count == 2 {
            return Granularity::Word;
        }
        if count == 3 {
            return Granularity::Line;
        }
        return Granularity::Character;
    }

    /// Forget the previous press,
    ///  so the next one is a single click.
    /// A press that extends the selection with Shift is never part of a multi-click.
    pub fn reset(&mut self) {
        self.last = None;
    }
}

/// The unit of `granularity` at caret boundary `position`,
///  as a start and an end position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unit(text: RopeSlice, position: number, granularity: Granularity): [number, number];
/// ```
pub fn unit(text: RopeSlice, position: usize, granularity: Granularity) -> (usize, usize) {
    if granularity == Granularity::Word {
        return word_range(text, position);
    }
    if granularity == Granularity::Line {
        return line_range(text, position);
    }
    let bounded = position.min(text.len_chars());
    return (bounded, bounded);
}

/// Anchor and head of a drag from the pressed unit `origin` to the pointer position `hit`.
///
/// The pressed unit always stays selected.
///  Dragging before it selects back to the start of the unit under
/// the pointer;
///  dragging after it selects forward to the end of that unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function extended(text: RopeSlice, origin: [number, number], granularity: Granularity, hit: number):
///   [anchor: number, head: number];
/// ```
pub fn extended(
    text: RopeSlice,
    origin: (usize, usize),
    granularity: Granularity,
    hit: usize,
) -> (usize, usize) {
    let (start, end) = unit(text, hit, granularity);
    if start < origin.0 {
        return (origin.1, start);
    }
    // Units never overlap partially, so a unit that does not start before the pressed one ends at or after it.
    return (origin.0, end);
}
