//! Native scrolling retains fractional offsets between raster tile updates.

/// Parent state owns the document and the currently displayed tile;
///  `rows` holds the map-based scroll rules.
use super::{
    AppWindow, State, gpu_tile,
    minimap::forget_unless,
    render, rows,
    tile::{self, TileBounds},
    wheel,
};
/// The side of a soft-wrap offset a position is read on.
use ide_app::document::ReadingAffinity;
/// Height of one code row:
///  the least a line can take,
///  which bounds how many lines a view can show.
use ide_app::row_map::CODE_ROW;
/// The part of a tall tile's lines its raster holds.
use ide_app::shaped_text::Clip;
/// What:
///  `pub(super) use` brings the clip rule of the library in under its old name here,
///  for `clip_for` and the sibling tests.
/// Why:
///  The shaper guards every frame with the same rule the window chooses a clip with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { bandClip } from 'ide-app/shaped-text';
/// ```
pub(super) use ide_app::shaped_text::band_clip;
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;
/// ComponentHandle supplies weak references for callback lifetimes.
use slint::ComponentHandle;
/// Checked shared ownership remains confined to the UI thread,
/// and the system clock times the scroll steps.
use std::{cell::RefCell, rc::Rc, time::Instant};

/// What:
///  The materialized tile spans this many view heights:
/// one above the view,
/// the view,
/// and one below.
/// Why:
///  A view inside the tile's middle band needs no new pixels,
/// so scrolling inside it does no source work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TILE_VIEWS = 3;
/// ```
pub(super) const TILE_VIEWS: f32 = 3.0;

/// What:
///  How close,
/// in view heights,
/// the view's edge may come to the tile's edge before the tile is re-centered.
/// Why:
///  The view starts a re-center with one view height of room on each side,
/// so it may scroll
///  `1 - RECENTER_EDGE_VIEWS` = 0.75 view heights before the next re-center:
/// about one view height,
///  and the tile edge never reaches the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RECENTER_EDGE_VIEWS = 0.25;
/// ```
const RECENTER_EDGE_VIEWS: f32 = 0.25;

/// What:
///  Lines materialized past the last line of the tile's bottom edge,
/// as a margin against rounding.
/// Why:
///  A line's block can be taller than one code row when it carries virtual rows,
///  so the line count of the tile is not the tile's pixel height divided by one code row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TILE_SPARE_LINES = 2;
/// ```
const TILE_SPARE_LINES: usize = 2;

/// What:
///  Whether the materialized tile still holds the view:
///  the view's top edge is at least `edge` below the tile's top edge,
///  and its bottom edge is as far above the tile's bottom edge,
///  where `edge` is [`recenter_edge`].
///  A side that touches the first or last line of the text is always held.
/// Why:
///  Edges come from the live vertical mapping,
/// so rows that appear above the tile move them with the rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holds(current: State, bounds: TileBounds, view: number, height: number): boolean;
/// ```
pub(super) fn holds(current: &State, bounds: TileBounds, view: f32, height: f32) -> bool {
    let lines = current.document.text().len_lines();
    let edge = recenter_edge(bounds.width, current.scale, height);
    let end = (bounds.first + bounds.count).min(lines);
    let (top, bottom) = tile_edges(current, bounds, end);
    // The top of the text and its bottom are always held; with a clip they can lie inside a line.
    let top_held = top <= 0.0 || view >= top + edge;
    let bottom_held = bottom >= current.row_map.height() || view + height <= bottom - edge;
    return top_held && bottom_held;
}

/// What:
///  How close,
///  in logical pixels,
///  the view's edge may come to the edge of a tile built for a text area `width` wide at display `scale`
///  before the tile is re-centered:
///  `RECENTER_EDGE_VIEWS` view heights,
///  or what a raster of the largest height the limit allows leaves around a view `height` tall,
///  when that is less.
///  Two code rows are kept back for the rounding of the clip's edges to whole rows,
///  and the answer is negative for a view taller than the raster can be.
/// Why:
///  A tile is never taller than its raster limit.
///  Demanding more room than the limit leaves around the view
///  made every tile the worker built for a window of about 10,000 px or more fail the hold test,
///  so the window discarded it and asked for the same one again,
///  for as long as the window stayed that size,
///  and the measurement pass never finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recenterEdge(width: number, scale: number, height: number): number {
///   return Math.min(RECENTER_EDGE_VIEWS * height, (tileCap(width, scale) - height) / 2 - 2 * CODE_ROW);
/// }
/// ```
fn recenter_edge(width: f32, scale: f32, height: f32) -> f32 {
    let room = (render::tile_cap(width, scale, height) - height) / 2.0 - 2.0 * CODE_ROW;
    return (RECENTER_EDGE_VIEWS * height).min(room);
}

/// What:
///  The top and bottom edge of the raster a tile at `bounds` holds,
/// in logical pixels from the top of the text:
///  the clip below the first line's block when there is one,
/// otherwise the blocks of its first and last lines.
///  `end` is the line after the tile's last.
/// Why:
///  The view must stay clear of these edges,
/// and they move with the live vertical mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileEdges(current: State, bounds: TileBounds, end: number): [top: number, bottom: number];
/// ```
pub(super) fn tile_edges(current: &State, bounds: TileBounds, end: usize) -> (f32, f32) {
    let first_top = current.row_map.block_top(bounds.first);
    return match bounds.clip {
        Some(clip) => (first_top + clip.top, first_top + clip.bottom),
        None => (first_top, current.row_map.block_top(end)),
    };
}

/// What:
///  The offsets between which the displayed tile's raster covers the whole view,
///  as the two edges of the raster in logical pixels from the top of the text:
///  a side that is the text's own first or last row is `HOLD_FREE` away,
///  because no offset passes it.
/// Why:
///  The window draws the view at the nearest offset inside this reach while the tile for the real offset has not
///  landed,
///  so a wheel ease that outruns the worker shows real rows of the tile it has and not a blank.
///  `holds` asks for more room (the re-center edge);
///  this is the raster's own edge,
///  which the picture cannot pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reach(current: State): [top: number, bottom: number];
/// ```
pub(super) fn reach(current: &State) -> (f32, f32) {
    let shown = current.shown;
    let lines = current.document.text().len_lines();
    let end = (shown.first + shown.count).min(lines);
    let (raster_top, raster_bottom) = tile_edges(current, shown, end);
    let top = if raster_top <= 0.0 {
        -HOLD_FREE
    } else {
        raster_top
    };
    let bottom = if raster_bottom >= current.row_map.height() {
        HOLD_FREE
    } else {
        raster_bottom
    };
    return (top, bottom);
}

/// What:
///  The distance,
///  in logical pixels,
///  that stands for "no limit" on a side of the reach.
/// Why:
///  The window holds plain numbers;
///  the text is never this tall.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HOLD_FREE = 1e7;
/// ```
pub(super) const HOLD_FREE: f32 = 10_000_000.0;

/// What:
///  Room kept at the right edge of the text area,
/// in logical pixels,
/// for a caret standing after the last glyph
///  of a full row.
/// Why:
///  There is no horizontal scrolling,
/// so the caret bar (2 px wide) must fit inside the view;
///  the view has no scrollbar element,
/// so this is the only term besides the gutter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CARET_ROOM = 2;
/// ```
pub(super) const CARET_ROOM: f32 = 2.0;

/// What:
///  The wrap width for a text area `viewport_width` logical pixels wide:
///  the width without the caret room,
///  or nothing when that is not positive.
/// Why:
///  A width of zero or less means the view is not laid out yet;
///  wrapping then waits and the last width is kept,
/// as IntelliJ's `SoftWrapApplianceManager` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wrapWidth(viewportWidth: number): number | undefined;
/// ```
pub(super) fn wrap_width(viewport_width: f32) -> Option<f32> {
    let width = viewport_width - CARET_ROOM;
    return has_wrap_width(width).then_some(width);
}

/// What:
///  Record that the view stands at `offset`:
///  the offset every later report is compared with,
///  the wrap width a report brings,
///  and the document position's viewport line.
///  Returns the wrap width in force and the offset clamped to the text.
/// Why:
///  A placement and a scroll step that asks for no tile both need exactly this and nothing of the tile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function noteView(current: State, offset: number, viewportWidth: number): [width: number, view: number];
/// ```
pub(super) fn note_view(current: &mut State, offset: f32, viewport_width: f32) -> (f32, f32) {
    // What: The map names the line at the view's top edge; `as usize` truncates a non-negative float to an
    //       address-sized index (sibling `u32`).
    // Why: Row indices address rope lines, whose interfaces use `usize`. Every line takes at least one code
    //      row, so this many lines always cover the view and its overscan, however many virtual rows there are.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const first = map.lineAt(Math.max(offset, 0)); const count = Math.ceil(height / CODE_ROW) + 3;
    // ```
    // Every caller that moves the view itself passes its new offset here first, so the offset the window
    // reports afterwards is recognized as not being the reader's own scrolling.
    current.offset = offset;
    // A press on the minimap that the view has been placed away from has been followed by scrolling.
    forget_unless(current, offset);
    // A width that cannot wrap keeps the last one.
    if let Some(wrap) = wrap_width(viewport_width) {
        current.width = wrap;
    }
    let width = current.width;
    let view = offset.max(0.0);
    // What: The document position's viewport line follows the view's top edge on every scroll step.
    // Why: Reload anchoring and navigation read that line, and it must not lag behind the view while the tile holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // position.viewport = text.lineToChar(map.lineAt(view));
    // ```
    let view_line = current.row_map.line_at(view);
    let text = current.document.text();
    let line = view_line.min(text.len_lines().saturating_sub(1));
    let viewport_char = text.line_to_char(line);
    if current.document.position().viewport != viewport_char {
        let mut position = current.document.position();
        position.viewport = viewport_char;
        current.document.select(position);
    }
    return (width, view);
}

/// What:
///  `&mut State` lends the source state mutably;
///  `f32` is a 32-bit float of logical pixels;
/// the `bool` answer says whether the materialized tile changed.
/// Why:
///  Native scrolling and programmatic reveal must agree on which rows are materialized;
///  the wrap width joins the tile's identity,
/// so a resize asks for a new tile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function place(current: State, offset: number, width: number, height: number): boolean;
/// ```
pub(super) fn place(current: &mut State, offset: f32, viewport_width: f32, height: f32) -> bool {
    // A placement ends any wheel ease the window was following: the view is set, not eased.
    wheel::release(current);
    // Any placement supersedes a scroll re-center still waiting for the worker; `place_scrolled` sets it again.
    current.scroll_wanted = None;
    let (width, view) = note_view(current, offset, viewport_width);
    // What: A view inside the tile's middle band keeps the tile, so the frame stamp and the raster do not change.
    // Why: Scrolling inside the tile must not cost a shaping pass, a raster, or an image upload.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (current.width === width && holds(current, view, height)) return false;
    // ```
    let shown = current.shown;
    if shown.width == width && holds(current, shown, view, height) {
        // The displayed tile still holds the view, so nothing is wanted; the requested window is the displayed one.
        tile::cancel(current);
        current.first = shown.first;
        current.count = shown.count;
        current.width = shown.width;
        current.clip = shown.clip;
        return false;
    }
    request_window(current, view, width, height);
    return true;
}

/// What:
///  Choose the window of lines the tile for a view at `view` holds:
///  one view height of overscan above the view and one below,
///  at wrap width `width`,
///  and set it as the requested tile.
/// Why:
///  Re-centering then happens only after the view has traveled about one view height inside the tile.
///  A wheel turn asks for the tile of the place its ease ends at with the same rule,
///  so that tile holds the view when the ease arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function requestWindow(current: State, view: number, width: number, height: number): void;
/// ```
pub(super) fn request_window(current: &mut State, view: f32, width: f32, height: f32) {
    let tile_start = view + height * 0.5 - height * TILE_VIEWS * 0.5;
    let tile_end = view + height * 0.5 + height * TILE_VIEWS * 0.5;
    let first = current.row_map.line_at(tile_start.max(0.0));
    let last = current.row_map.line_at(tile_end);
    current.first = first;
    current.count = last.saturating_sub(first) + 1 + TILE_SPARE_LINES;
    current.width = width;
    current.clip = clip_for(current, (tile_start, tile_end), height);
}

/// What:
///  The part of the lines just chosen for the tile that its raster holds:
///  nothing while the lines fit one raster,
///  otherwise the rows of `band`,
/// the three view heights around the view,
/// cut down to the raster limit.
///  The edges are whole code rows below the first line's block.
/// Why:
///  A tile holds whole lines,
/// and a line wrapped to thousands of rows would pass the raster limit.
///  The band is where the view can go before the next re-center,
/// so the rows outside it are never needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clipFor(current: State, band: [top: number, bottom: number], view: number): Clip | undefined;
/// ```
fn clip_for(current: &State, band: (f32, f32), view: f32) -> Option<Clip> {
    let lines = current.document.text().len_lines();
    let end = (current.first + current.count).min(lines);
    let first_top = current.row_map.block_top(current.first);
    let bottom = current.row_map.block_top(end);
    let cap = render::tile_cap(current.width, current.scale, view);
    let clip = band_clip(first_top, bottom, band, cap);
    if let Some(chosen) = clip {
        tracing::debug!(
            first = current.first,
            top = chosen.top,
            bottom = chosen.bottom,
            cap,
            "clipped a tile of lines taller than one raster"
        );
    }
    return clip;
}

/// What:
///  `place` for the view's own reports:
/// a tile it changes for the same width is a scroll re-center,
///  which the worker may build while the displayed tile stays on screen,
///  and a tile it changes for a new width is built by the window thread before the next frame,
/// within an allowance.
/// Why:
///  Keyboard,
/// pointer,
/// and reveal moves call `place` directly and keep the synchronous path.
///  A frame of the old width's text in a window of the new width overflows its edge or stops short of it,
///  so the render that follows a new width paints the tile for it (`rewrap::begin`) and leaves only the lines
///  far from the view to the worker;
/// a tile that does not fit the allowance is queued for the worker as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeScrolled(state: State, offset: number, width: number, height: number): boolean;
/// ```
pub(super) fn place_scrolled(
    current: &mut State,
    offset: f32,
    viewport_width: f32,
    height: f32,
) -> bool {
    // What: A step of a wheel ease whose end already has a tile requested asks for no tile of its own.
    // Why: The tile of the end supersedes the tile of every step on the way, which the worker would build late.
    if wheel::following(current, offset) {
        note_view(current, offset, viewport_width);
        return wheel::unsent(current);
    }
    let changed = place(current, offset, viewport_width, height);
    // The first width the view reports replaces a tile that was never wrapped; it is built at once, so the
    // window never shows long lines unwrapped.
    if changed && current.shown.width > 0.0 {
        if current.shown.width == current.width {
            current.scroll_wanted = Some(tile::requested(current));
        } else {
            current.rewrap = true;
        }
    }
    return changed;
}

/// Retain pixel scrolling;
///  update the raster only at viewport tile boundaries and when the wrap width changes.
pub(super) fn bind_viewport(owner: &AppWindow, shared: &Rc<RefCell<State>>) {
    // The code view is drawn on the GPU, by the window's one rendering notifier.
    gpu_tile::bind(owner, &shared.borrow().gpu);
    // What: `Rc::clone` copies the pointer to the shared state; `as_weak` does not keep the window alive.
    // Why: The stored callback outlives this binding function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const state = shared; const weak = new WeakRef(owner);
    // ```
    let flick_state = Rc::clone(shared);
    let flick_weak = owner.as_weak();
    // The source scroll reports every wheel turn or pan it takes; the turn the window noted names the end of the ease.
    owner.on_source_flicked(move || {
        if let Some(window) = flick_weak.upgrade() {
            wheel::flicked(&window, &flick_state);
        }
    });
    let state = Rc::clone(shared);
    let weak = owner.as_weak();
    // What: `move |...|` transfers the captured handles into the stored callback.
    // Why: Borrowed locals would not outlive this function.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // owner.onViewportChanged((offset, width, height) => { ... });
    // ```
    owner.on_viewport_changed(move |offset, viewport_width, height| {
        // What: `borrow_mut` lends the state mutably; `&mut current` passes that loan to `place`.
        // Why: The loan must end before rendering borrows the state again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const changed = place(state.current, offset, width, height);
        // ```
        let mut reported = None;
        if let Some(window) = weak.upgrade() {
            reported = Some(window.window().scale_factor());
        }
        tracing::debug!(
            target: "monochromatic_ide::native::frame",
            viewport_width,
            height,
            "the view reported its size"
        );
        let mut current = state.borrow_mut();
        if let Some(factor) = reported {
            // The size limit of the tile depends on the display scale.
            current.scale = factor;
        }
        // An offset nobody announced is the reader scrolling: by wheel, by touch, or by a drag past an edge.
        let stepped = offset != current.offset;
        if stepped {
            // `Some(...)` records this moment as the latest scroll.
            current.scrolled_at = Some(rows::quiet_now());
            current.stepped_at = Some(Instant::now());
        }
        let changed = place_scrolled(&mut current, offset, viewport_width, height);
        drop(current);
        // What: `upgrade` returns `Some(window)` only while the window still exists.
        // Why: Closing the app must not keep a hidden window alive through callbacks.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const window = weak.deref(); if (window) render(window, state);
        // ```
        let Some(window) = weak.upgrade() else {
            return;
        };
        if changed {
            render(&window, &state);
        } else {
            // The tile stays, but the minimap's picture slides with the scroll and may need other bands.
            super::minimap::note(&window, &state);
        }
        if stepped {
            // What: A tile that finished since the last step is swapped in here, after the step's own work.
            // Why: The poll of the tile worker leaves it for this step, so the setters that swap it in run in
            //      the frame this step draws, not in a frame of their own. State and window agree here, as
            //      they do when the poll's timer fires.
            tile::poll(&window, &state);
        }
    });
}

/// What:
///  How many visual rows below the first row of its line the character `position` is drawn,
///  on the side of a soft wrap `affinity` names.
///  A line outside the materialized rows is shaped on the spot.
/// Why:
///  Keeping the caret in view means keeping its visual row in view,
/// not only its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rowsDown(current: State, position: number, affinity: ReadingAffinity, scale: number): number;
/// ```
fn rows_down(current: &mut State, position: usize, affinity: ReadingAffinity, scale: f32) -> usize {
    let line = current
        .document
        .text()
        .char_to_line(position.min(current.document.text().len_chars()));
    if let Some(view) = &current.shaped {
        for row in &view.rows {
            if row.row == line {
                return row.caret_at(position, affinity, view.viewport.scale).row;
            }
        }
    }
    if !has_wrap_width(current.width) {
        return 0;
    }
    let State {
        document,
        shaper,
        width,
        ..
    } = current;
    return shaper
        .wrapped_row(document, line, scale, *width)
        .caret_at(position, affinity, scale)
        .row;
}

/// Render after a keyboard caret movement and keep the caret visible with the smallest possible scroll.
///
/// `paged` first moves the view by that many logical pixels,
///  so a page key keeps the caret's place on screen.
/// A caret above the view becomes the top row,
///  one below the bottom row;
///  a visible caret keeps the offset.
/// Offsets are assigned directly,
///  like other programmatic reveals;
///  native wheel easing is unaffected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function follow(window: AppWindow, state: Shared<State>, paged: number): void;
/// ```
pub(super) fn follow(window: &AppWindow, state: &Rc<RefCell<State>>, paged: f32) {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let before = -window.get_scroll_y();
    let scale = window.window().scale_factor();
    let mut current = state.borrow_mut();
    let caret = current.document.position();
    let head = caret.head;
    let row = current.document.text().char_to_line(head);
    let down = rows_down(&mut current, head, caret.affinity, scale);
    let limit = rows::limit(&current.row_map, height);
    let paged_offset = (before + paged).clamp(0.0, limit);
    let mut offset =
        rows::showing(&current.row_map, row, down, paged_offset, height).clamp(0.0, limit);
    // A fractional offset left by smooth scrolling is kept while the caret stays in view.
    if offset != before {
        offset = offset.round();
    }
    place(&mut current, offset, width, height);
    drop(current);
    if offset != before {
        window.set_scroll_y(-offset);
    }
    render(window, state);
    tracing::debug!(row, vertical = offset, "followed source caret");
}

/// Scroll a source character range into view and render it.
/// A visible range keeps its offset;
///  a hidden row is centered.
/// Offsets are assigned directly,
///  like other programmatic reveals;
///  native wheel easing is unaffected.
pub(super) fn reveal(window: &AppWindow, state: &Rc<RefCell<State>>, start: usize, end: usize) {
    let height = window.get_viewport_height();
    let width = window.get_viewport_width();
    let before = -window.get_scroll_y();
    let scale = window.window().scale_factor();
    let mut offset = before;
    let mut current = state.borrow_mut();
    let text = current.document.text();
    let bounded = start.min(text.len_chars());
    let row = text.char_to_line(bounded);
    // A range to reveal is not the caret, so its start stands on the default side of a wrap offset.
    let down = rows_down(&mut current, bounded, ReadingAffinity::Downstream, scale);
    // What: The target line's own visual rows go into the map before the offset is limited by its extent.
    // Why: A line the background measurement has not reached counts as one row, so a match on a late row of the
    //      last line would be clamped out of the view, and nothing moves the view again once the rows are known.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // absorb(state, measureLines(row, row + 1)); settle(state);
    // ```
    if down > 0 && has_wrap_width(current.width) {
        let wrap = current.width;
        let measured = {
            let State {
                shaper,
                document,
                styles,
                ..
            } = &mut *current;
            shaper.measure_lines(document, row..row + 1, scale, Some(wrap), styles)
        };
        if rows::absorb(&mut current, measured.lines) {
            tracing::debug!(
                row,
                down,
                "measured the revealed line before limiting the offset"
            );
            rows::settle(&mut current, scale, (before, width, height), true);
        }
    }
    let top = current.row_map.code_top(row) + down as f32 * CODE_ROW;
    if top < offset || top + CODE_ROW > offset + height {
        let limit = rows::limit(&current.row_map, height);
        offset = (top - ((height - CODE_ROW) / 2.0).max(0.0))
            .clamp(0.0, limit)
            .round();
    }
    place(&mut current, offset, width, height);
    drop(current);
    if offset != before {
        window.set_scroll_y(-offset);
    }
    render(window, state);
    tracing::debug!(row, vertical = offset, end, "revealed source range");
}
