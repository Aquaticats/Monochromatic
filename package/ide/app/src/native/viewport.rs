//! Native scrolling retains fractional offsets between raster tile updates.

/// The source state,
///  the minimap's promise a placement keeps,
///  and the tile and wheel ease a step asks for.
use super::{
    State,
    minimap::forget_unless,
    tile,
    wheel,
};
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

/// The window's viewport callback and the frame that follows a step.
mod bindings;
/// The caret and a revealed range kept in view.
mod caret_reveal;
/// The lines the materialized tile holds and the edges the view stays clear of.
mod tile_geometry;

/// Install the window's viewport callback,
///  and with it the notifier that draws the code view.
pub(super) use bindings::bind_viewport;
/// Keep the caret and a revealed range in view with the smallest scroll.
pub(super) use caret_reveal::{
    follow,
    reveal,
};
/// Where the displayed tile's raster ends,
///  which lines cover a view,
///  and the lines a view asks the tile to hold.
pub(super) use tile_geometry::{
    TILE_VIEWS,
    holds,
    reach,
    request_window,
};

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
pub(super) fn note_view(
    current: &mut State,
    offset: f32,
    viewport_width: f32,
) -> (f32, f32) {
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
pub(super) fn place(
    current: &mut State,
    offset: f32,
    viewport_width: f32,
    height: f32,
) -> bool {
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
