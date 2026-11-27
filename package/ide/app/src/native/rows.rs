//! The window's side of the vertical mapping:
//!  keep `State::row_map` equal to what the displayed text and its
//! annotations call for,
//!  and tell the window what follows from it.
//!
//! Every native function that places or finds a line vertically reads `State::row_map`;
//!  none multiplies a line
//! number by a row height.
//!  The markup only scrolls and draws at positions handed to it.

/// The source state,
///  the parts of the window that act on the map,
///  and the scrolling rule.
use super::{
    State,
    minimap::aim_again,
    tile::requested,
    viewport::{
        note_view,
        place,
    },
    wheel,
};
/// The rank that orders severities.
use ide_app::annotation::rank;
/// The mapping itself and the height of one code row.
use ide_app::row_map::{
    CODE_ROW,
    RowMap,
};
/// `VecModel` is the toolkit's growable list model that the placement names.
use slint::VecModel;
/// A shared pointer to the persistent models,
///  and the clock of the expiry check.
use std::{
    rc::Rc,
    time::Instant,
};

/// Space held open above the lines of a replaced text.
mod held;
/// The scrolling rule's clock.
mod quiet;
/// Deciding what the blocks and the map are made of,
/// rebuilding them,
///  and installing the result.
mod rebuild;
/// The window's models and setters for the vertical mapping.
mod window;

/// Reload and the hold that follows it carry space over to the new text.
pub(super) use held::{
    carried,
    hold,
};
/// The test clock the scrolling rule reads instead of the system clock.
#[cfg(test)]
pub(super) use quiet::hold_clock;
/// The moment the scrolling rule reads as now.
pub(super) use quiet::quiet_now;
/// Whether the reader scrolled so recently that a wheel animation may still be running.
use quiet::scrolling;
/// The map key and the measured rows a reload carries over.
pub(super) use rebuild::{
    RowKey,
    absorb,
};
/// Deciding what the map is made of,
///  rebuilding it,
///  and installing the result.
use rebuild::{
    install,
    rebuilt,
    wanted_key,
};
/// The persistent models,
///  and handing a placement to the window.
pub(super) use window::{
    mark_model,
    present,
    top_model,
};

/// What:
///  Bring `current.blocks` and `current.row_map` up to date at once and answer the previous map when
///       positions changed.
/// Why:
///  Callers that place the view themselves (a reload,
///  a file switch,
///  a revealed line) refresh before they
///      compute an offset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refresh(current: State, scale: number): RowMap | undefined;
/// ```
pub(super) fn refresh(
    current: &mut State,
    scale: f32,
) -> Option<RowMap> {
    // `?` leaves with nothing when the layout is already current.
    let layout = rebuilt(current, scale)?;
    return install(current, layout);
}

/// What:
///  Bring the vertical mapping up to date for a render and,
///  when it changed,
///  keep the view still:
///  the
///       answer is the scroll offset the window must take,
///  or nothing.
///  `view` holds the vertical offset,
///  the width of the text area,
///  and the height of the view in logical pixels.
///  `force` installs a change that moves the offset even while the reader scrolls,
///  for a finished tile whose rows only fit the new map.
/// Why:
///  Rows that appear,
///  change,
///  or vanish above the first visible code row must move nothing visible,
///  so
///      the offset follows them;
///  the materialized lines are chosen again for the offset that results.
///      While the reader is scrolling,
///  a change that would move the offset waits:
///  the rows it brings are
///      above the view anyway,
///  and assigning the offset would cut the toolkit's scroll animation short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settle(current: State, scale: number, view: [number, number, number], force: boolean): number | undefined;
/// ```
pub(super) fn settle(
    current: &mut State,
    scale: f32,
    view: (f32, f32, f32),
    force: bool,
) -> Option<f32> {
    let (offset, width, height) = view;
    let layout = rebuilt(current, scale)?;
    let still = anchored(&current.row_map, &layout.map, offset).min(limit(&layout.map, height));
    // A layout for the same text may wait; one for another text never does, its caller placed the view.
    let same_text = match current.row_key {
        Some(known) => known.0 == layout.key.0,
        None => false,
    };
    // What: `abs()` is the distance without its sign; the window's offset differs from the last one placed when
    //       a scroll step is under way whose change handler has not run yet.
    // Why: Under load a render can come between a wheel animation's last step and its change handler; that step
    //      is scrolling too, and the rows must not cut it short.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const moving = Math.abs(offset - current.offset) > 0.01;
    // ```
    let moving = (offset - current.offset).abs() > 0.01;
    if still != offset && same_text && !force && (moving || scrolling(current)) {
        tracing::debug!(
            offset,
            "rows above the view wait until scrolling has stopped"
        );
        return None;
    }
    // `?` leaves with nothing when no position changed.
    install(current, layout)?;
    // What: A press on the minimap that nothing has followed is mapped again through the layout just installed.
    // Why: The press landed through a layout that lacked rows still to arrive; the same press lands elsewhere in
    //      this one, and the reader has not moved the view since, so it goes where the press means now.
    let kept = aim_again(current, scale, (offset, width, height)).unwrap_or(still);
    // A re-center that waits for the worker and is still needed under new bounds (a measurement moved the rows,
    // or the width changed again) stays a worker re-center, never a build on the window thread.
    let waiting = current.scroll_wanted.is_some();
    if wheel::following(current, kept) {
        // The toolkit's scroll is still easing to the end the window asked a tile for: the new rows move that end,
        // so the same end is asked for again under the new map, and the ease stays followed.
        note_view(current, kept, width);
        wheel::refit(current, width, height);
    } else if place(current, kept, width, height) && waiting && current.shown.width > 0.0 {
        current.scroll_wanted = Some(requested(current));
    }
    tracing::debug!(
        before = offset,
        after = kept,
        "kept the view still across a change of virtual rows"
    );
    if kept == offset {
        return None;
    }
    return Some(kept);
}

/// What:
///  Whether the window must render now for the vertical mapping's sake:
///  held space ran out of time,
///  or
///       rows that waited for scrolling to stop can be shown.
/// Why:
///  Neither comes with an event of its own;
///  the source refresh timer asks every tick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function due(current: State, scale: number): boolean;
/// ```
pub(super) fn due(
    current: &mut State,
    scale: f32,
) -> bool {
    let expired = current.annotations.expire(Instant::now());
    let waiting = current.row_key.is_some()
        && current.row_key != Some(wanted_key(current, scale))
        && !scrolling(current);
    return expired || waiting;
}

/// What:
///  The scroll offset in `next` that shows what `offset` showed in `previous`:
///  the code row of the line
///       at the view's top edge keeps its distance from that edge.
/// Why:
///  Virtual rows that appear,
///  change,
///  or vanish above the first visible code row must move nothing that
///      is visible.
///  A view at the very top of the text stays there,
///  so rows arriving for the first lines push
///      the text down instead of hiding above the top edge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anchored(previous: RowMap, next: RowMap, offset: number): number;
/// ```
pub(super) fn anchored(
    previous: &RowMap,
    next: &RowMap,
    offset: f32,
) -> f32 {
    if offset <= 0.0 {
        return 0.0;
    }
    let line = previous.line_at(offset).min(next.lines() - 1);
    // A line that is now shorter cannot hold the view's top edge below its end.
    let within = (offset - previous.code_top(line)).min(next.code_height(line));
    return (next.code_top(line) + within).max(0.0);
}

/// What:
///  The scroll offset that shows the visual row `rows_down` rows below the top of `row`'s code rows,
///  in a view of `height` starting at `offset`,
///  changed by the smallest amount;
///  the answer is unrounded and not clamped to the text.
/// Why:
///  Caret keys and drags keep the caret in view,
/// on the visual row it is drawn on.
///  A caret above the view on a line's first row makes that line the top line
///      together with its virtual rows,
///  so what is said about the caret's line is in view with it;
///  a caret below the view makes its row the bottom row.
///  The code row wins when both do not fit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function showing(map: RowMap, row: number, rowsDown: number, offset: number, height: number): number;
/// ```
pub(super) fn showing(
    map: &RowMap,
    row: usize,
    rows_down: usize,
    offset: f32,
    height: f32,
) -> f32 {
    let top = map.code_top(row) + rows_down as f32 * CODE_ROW;
    let mut wanted = offset;
    if top < offset {
        wanted = if rows_down == 0 {
            map.block_top(row)
        } else {
            top
        };
    }
    if top + CODE_ROW > wanted + height {
        wanted = top + CODE_ROW - height;
    }
    return wanted;
}

/// Largest scroll offset of a view of `height` over the mapped text.
pub(super) fn limit(
    map: &RowMap,
    height: f32,
) -> f32 {
    return (map.height() - height).max(0.0);
}

/// What:
///  Everything the window draws from the map and the displayed frame,
///  read while the state is borrowed:
///  the scroll extent,
///  one gutter entry for each visual row of the frame (its top,
///  the line number on the first row of a line and none on a continuation row,
///  and the severity mark),
///  the line count,
///  and the place of the line a language surface is anchored to.
/// Why:
///  Window setters run only after the state borrow ended,
///  so the values are collected first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placement = { extent: number; tops: number[]; numbers: number[]; marks: number[]; lines: number;
///   anchor: [top: number, height: number] };
/// ```
pub(super) struct Placement {
    /// Height of the whole text.
    extent: f32,
    /// Top of each visual row of the frame.
    tops: Vec<f32>,
    /// The persistent row-top model.
    top_model: Rc<VecModel<f32>>,
    /// Line number (one-based) on the first visual row of a line,
    /// zero on a continuation row.
    numbers: Vec<i32>,
    /// The persistent line-number model.
    number_model: Rc<VecModel<i32>>,
    /// Gutter severity mark of each visual row;
    ///  see `State::row_marks`.
    marks: Vec<i32>,
    /// The persistent severity-mark model.
    mark_model: Rc<VecModel<i32>>,
    /// Number of lines,
    ///  whose last number sets the width of the gutter's number column.
    lines: usize,
    /// Top and height of everything the anchoring line owns.
    anchor: (f32, f32),
}

/// What:
///  Collect the [`Placement`] of the displayed frame and the current map;
///  `anchor_line` is the line the window says a language
///       surface belongs to.
/// Why:
///  The markup holds no row arithmetic;
///  every position it uses comes from here.
///  The frame's own rows decide how many gutter entries a line has,
///  so the gutter always matches the picture beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function measure(current: State, anchorLine: number): Placement;
/// ```
pub(super) fn measure(
    current: &State,
    anchor_line: i32,
) -> Placement {
    let map = &current.row_map;
    let mut tops = Vec::new();
    let mut numbers = Vec::new();
    let mut marks = Vec::new();
    // What: Blocks and frame rows both ascend by line, so one moving index finds each row's block.
    // Why: A line's message rows are listed worst first, so the first row names the worst severity starting on
    //      the line. Blocks with held space only have no rows, so a stale text shows no letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const row of frame.rows) { const block = blocks.find((b) => b.line === row.row); ... }
    // ```
    let mut next_block = 0;
    if let Some(view) = &current.shaped {
        // The raster's vertical extent, which only a clipped tile limits the gutter to.
        let band = (
            view.origin,
            view.origin + view.height as f32 / view.viewport.scale,
        );
        for row in &view.rows {
            while next_block < current.blocks.len() && current.blocks[next_block].line < row.row {
                next_block += 1;
            }
            let mut mark = 0;
            if let Some(block) = current.blocks.get(next_block)
                && block.line == row.row
                && let Some(worst) = block.messages.first()
            {
                mark = i32::from(rank(worst.severity)) + 1;
            }
            for visual in 0..row.visual_rows() {
                let top = row.top + row.visual_row_offset(visual, view.viewport.scale);
                // A clipped tile paints only the rows inside its raster, and the gutter has an entry for just those.
                if view.viewport.clip.is_some() && (top + CODE_ROW <= band.0 || top >= band.1) {
                    continue;
                }
                tops.push(top);
                // The number belongs to the first visual row; a continuation row shows the wrap indicator instead.
                numbers.push(if visual == 0 {
                    i32::try_from(row.row + 1).unwrap_or(i32::MAX)
                } else {
                    0
                });
                marks.push(if visual == 0 { mark } else { 0 });
            }
        }
    }
    return Placement {
        extent: map.height(),
        tops,
        // `Rc::clone` copies the pointer to the shared model, not its rows.
        top_model: Rc::clone(&current.row_tops),
        numbers,
        number_model: Rc::clone(&current.row_numbers),
        marks,
        mark_model: Rc::clone(&current.row_marks),
        lines: map.lines(),
        anchor: anchor_place(map, anchor_line),
    };
}

/// What:
///  Top and height of everything `anchor_line` owns,
///  its virtual rows and its code row;
///  a pair (tuple).
/// Why:
///  A language popup or list is placed beside that span,
///  so it covers neither the code row nor the
///      line's own virtual rows,
///  also when rows appear above the line while the surface is shown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function anchorPlace(map: RowMap, anchorLine: number): [top: number, height: number];
/// ```
pub(super) fn anchor_place(
    map: &RowMap,
    anchor_line: i32,
) -> (f32, f32) {
    // `max(0) as usize` turns the toolkit's integer into a line index.
    let line = (anchor_line.max(0) as usize).min(map.lines() - 1);
    let top = map.block_top(line);
    return (top, map.code_bottom(line) - top);
}
