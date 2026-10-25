//! The resting pointer:
//!  which source character it is over,
//!  and since when.
//!
//! editord asks for hover information when the pointer has not moved for 350 ms
//! (`HOVER_DEBOUNCE_MS` in `package-paused/desktop-daemon/editord/src/client/timing.ts`).
//! This view does the same per character:
//!  moving within one character keeps the timer.

/// The source state holds the shaped rows the pointer is resolved against.
use crate::native::State;
/// What:
///  `Duration` is a time span;
///  `Instant` is a point on a clock that never goes backwards.
/// Why:
///  Resting is measured from the last move to another character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const now = performance.now();
/// ```
use std::time::{Duration, Instant};

/// How long the pointer rests on one character before hover information is asked for.
pub(super) const REST: Duration = Duration::from_millis(350);

/// How far the drawn offset must change between two ticks to count as a move of the view,
///  in logical pixels.
///  Half a pixel is the tolerance `surface::stale` allows for rounding when the view is kept still.
const VIEW_MOVE: f32 = 0.5;

/// What:
///  The two numbers that say where the text stands under a pointer that does not move:
///  the offset the view is drawn at and the height of the whole text with its virtual rows.
/// Why:
///  A wheel turn or a jump changes the offset;
///  hints or messages arriving above the pointer change the height,
///  and may put other text under the pointer without changing the offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type View = { offset: number; extent: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct View {
    /// The offset that is drawn.
    pub(super) offset: f32,
    /// The height of the whole text with its virtual rows.
    pub(super) extent: f32,
}

/// What:
///  Where the pointer is,
///  as the numbers `character_at` takes for the view of that moment.
/// Why:
///  The pointer's own place in the window does not change while the view scrolls under it,
///  and the toolkit sends no event for it.
///  The distance from the top of the view is that place,
///  and adding the offset that is drawn at the time of a request gives the text point under the pointer then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Spot = { belowTop: number; x: number; offset: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Spot {
    /// Logical distance of the pointer below the top of the view.
    pub(super) below_top: f32,
    /// Logical distance from the start of a line's text,
    ///  negative over the line numbers.
    pub(super) x: f32,
    /// The drawn offset at the time of the move.
    pub(super) offset: f32,
    /// The height of the whole text at the time of the move.
    pub(super) extent: f32,
}

/// What:
///  Where the pointer rests.
///  `Option<(Option<usize>, Instant)>` is "nothing" while the
///       pointer is outside the source,
///  or the character it is over (nothing between lines'
///       ends and gutters) with the moment it arrived there.
/// Why:
///  A hover request is sent once per rest,
///  and resting over no character hides the popup.
///  The character is a test of the move,
///  not the answer:
///  what a request names is found again from `spot` when the rest is over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rest = { at?: [character: number | undefined, since: number]; asked: boolean; spot?: Spot; seen: number };
/// ```
#[derive(Debug, Default)]
pub(super) struct Rest {
    /// Character under the pointer and when the pointer arrived on it.
    pub(super) at: Option<(Option<usize>, Instant)>,
    /// The request for the current rest was already made.
    pub(super) asked: bool,
    /// The pointer's place at its last move.
    spot: Option<Spot>,
    /// The view at the last tick or move.
    seen: Option<View>,
    /// The view moved under this rest after the pointer arrived,
    ///  so a pointer that was over no character may be over one now.
    shifted: bool,
}

/// Movement and timing.
impl Rest {
    /// The view is `view` at this tick:
    ///  a view that moved puts other text under a pointer that did not,
    ///  so a rest that was not asked for yet starts again,
    ///  whether the pointer was over a character or over none,
    ///  and remembers that the text under it may have changed.
    pub(super) fn follow(&mut self, view: View) {
        let Some(seen) = self.seen else {
            self.seen = Some(view);
            return;
        };
        let offset_moved = (view.offset - seen.offset).abs() > VIEW_MOVE;
        let extent_moved = (view.extent - seen.extent).abs() > VIEW_MOVE;
        if !(offset_moved || extent_moved) {
            return;
        }
        self.seen = Some(view);
        if self.asked {
            return;
        }
        if let Some((character, _)) = self.at {
            tracing::debug!(
                offset = view.offset,
                extent = view.extent,
                "the view moved under a rest not yet asked for"
            );
            self.at = Some((character, Instant::now()));
            self.shifted = true;
        }
    }

    /// What:
    ///  The rest was resolved against the displayed view and found no character under the pointer,
    ///  so nothing was asked.
    /// Why:
    ///  The rest becomes the rest of a pointer that moved onto nothing:
    ///  it dismisses a popup after the rest time,
    ///  asks nothing while the view stands still,
    ///  and starts again when the view moves.
    pub(super) fn found_nothing(&mut self) {
        self.at = Some((None, Instant::now()));
        self.shifted = false;
    }

    /// What:
    ///  A request was made for `character`,
    ///  which the rest resolved to when it was over.
    /// Why:
    ///  The rest is now a rest over that character,
    ///  asked about once.
    ///  Left as a rest over no character it would close the popup of the answer as soon as it is shown.
    pub(super) fn asked_about(&mut self, character: usize) {
        if let Some((_, since)) = self.at {
            self.at = Some((Some(character), since));
        }
        self.asked = true;
    }

    /// The pointer moved to `character` at `spot`;
    ///  a different character restarts the rest.
    pub(super) fn moved(&mut self, character: Option<usize>, spot: Spot) {
        self.spot = Some(spot);
        self.seen = Some(View {
            offset: spot.offset,
            extent: spot.extent,
        });
        // `map` reads the character of the present rest, if any.
        let mut present = None;
        if let Some((current, _)) = self.at {
            present = Some(current);
        }
        if present == Some(character) {
            return;
        }
        // `Some((character, Instant::now()))` starts a new rest at this moment.
        self.at = Some((character, Instant::now()));
        self.asked = false;
        self.shifted = false;
    }

    /// The pointer left the source.
    pub(super) fn left(&mut self) {
        self.at = None;
        self.spot = None;
        self.asked = false;
    }

    /// The pointer is over the source text area.
    pub(super) fn over(&self) -> bool {
        return self.at.is_some();
    }

    /// What:
    ///  The place to ask hover information for,
    ///  once the pointer and the view rested long enough.
    /// Why:
    ///  Only one request per rest;
    ///  the caller marks it asked
    ///  and finds the character under the place in the view it draws then.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// due(): Spot | undefined
    /// ```
    pub(super) fn due(&self) -> Option<Spot> {
        if self.asked {
            return None;
        }
        // `?` leaves with nothing when the pointer is outside the source.
        let (character, since) = self.at?;
        // A pointer over no character in a view that stood still has nothing to ask about;
        //  one whose view moved under it may be over a character now.
        if character.is_none() && !self.shifted {
            return None;
        }
        if since.elapsed() < REST {
            return None;
        }
        return self.spot;
    }

    /// True once the pointer rested over no character for the rest time.
    pub(super) fn idle(&self) -> bool {
        return matches!(self.at, Some((None, since)) if since.elapsed() >= REST);
    }
}

/// What:
///  The source character under a point:
///  `y` is the logical distance from the top of the text
///       and `x` the logical distance from the start of a line's text,
///  negative over the line numbers.
/// Why:
///  Hover and Ctrl+click ask about the character the pointer is over,
///  not the nearest caret
///      boundary;
///  a point past the end of a line,
///  over the gutter,
///  below the text,
///  or on a line's
///      virtual rows (its hints and diagnostic messages) is over no character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function characterAt(state: State, y: number, x: number): number | undefined
/// ```
pub(super) fn character_at(state: &State, y: f32, x: f32) -> Option<usize> {
    if y < 0.0 || x < 0.0 || y >= state.row_map.height() {
        return None;
    }
    // The vertical mapping names the line and tells its code row from the virtual rows above it.
    let place = state.row_map.locate(y);
    if place.in_block {
        return None;
    }
    let line = place.line;
    // `?` returns nothing when no rows are shaped yet.
    let view = state.shaped.as_ref()?;
    let scale = view.viewport.scale;
    // `find` returns the first shaped row of that line, or nothing when it is not materialized.
    let mut found_row = None;
    for candidate in view.rows.iter() {
        if candidate.row == line {
            found_row = Some(candidate);
            break;
        }
    }
    let row = found_row?;
    let length = row.source_len();
    let end = row.source_start + length;
    // The pointer is on one visual row of a wrapped line; the end of that row, not of the line, is where text stops.
    let visual = state.row_map.visual_row_at(line, y);
    let within = (y - state.row_map.code_top(line)).max(0.0);
    let row_end = row.source_start + row.visual_row_sources(visual)?.end;
    if length == 0 || x >= row.x_on_row(row_end, visual, scale) {
        return None;
    }
    let boundary = row.hit_at(x, within, scale);
    // The nearest boundary right of the point means the character before it is under the point.
    let character = if boundary > row.source_start && x < row.x_on_row(boundary, visual, scale) {
        boundary - 1
    } else {
        boundary
    };
    if character >= end {
        return None;
    }
    return Some(character);
}
