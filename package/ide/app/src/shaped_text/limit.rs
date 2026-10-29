//! The raster limit of one tile:
//!  the rows a raster of a given width may have,
//!  the clip rules that keep a tile within them,
//!  and the guard that cuts any frame to fit before it is painted.

/// A clip is two distances below the first line's block,
/// and a viewport carries the one a frame is prepared for.
use super::frame::{Clip, Viewport};
/// A code row is the unit the clip edges are rounded to.
use crate::row_map::CODE_ROW;
/// The device's texture size limit is one process-wide number,
/// read by the window thread and the tile worker.
use std::sync::atomic::{AtomicU32, Ordering};

/// What:
///  Most bytes one texture of a source tile may hold:
///  64 MiB of RGBA.
/// Why:
///  A tile is drawn across as many GPU textures as its height needs,
///  and each one stays within this;
///  the window clips a tile that would pass the stack's bound,
///  so the check in the painter only guards a caller that did not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_RASTER_BYTES = 64 * 1024 * 1024;
/// ```
pub const MAX_RASTER_BYTES: u64 = 64 * 1024 * 1024;

/// What:
///  The texture size every GPU device offers at least:
///  wgpu's default limit for the width and the height of a 2D texture.
/// Why:
///  The application opens its device with wgpu's default limits raised to the adapter's,
///  so no device it works on offers less,
///  and a tile cut for this size fits any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_TEXTURE_DIMENSION = 8192;
/// ```
pub const DEFAULT_TEXTURE_DIMENSION: u32 = 8192;

/// The largest texture side the process's GPU device offers.
static TEXTURE_DIMENSION: AtomicU32 = AtomicU32::new(DEFAULT_TEXTURE_DIMENSION);

/// What:
///  Record the largest side of a 2D texture the process's GPU device offers
///  (`max_texture_dimension_2d`),
///  so that a tile is never cut taller than one texture can be.
/// Why:
///  The launcher and the test harness know the device before any window exists,
///  and the rendering notifier checks the value again when the device arrives.
///  The device is one per process,
/// so the limit is too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function setTextureDimension(limit: number): void;
/// ```
pub fn set_texture_dimension(limit: u32) {
    TEXTURE_DIMENSION.store(limit.max(1), Ordering::Relaxed);
}

/// The largest side of a 2D texture the device offers,
/// as last recorded by [`set_texture_dimension`].
pub fn texture_dimension() -> u32 {
    return TEXTURE_DIMENSION.load(Ordering::Relaxed);
}

/// What:
///  Logical pixels a raster extends past the wrap width.
/// Why:
///  The raster holds glyphs,
///  the terminator mark,
///  and underlines of the row ends;
///  everything else the window draws over it is drawn by the markup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const RASTER_MARGIN = 24;
/// ```
pub const RASTER_MARGIN: f32 = 24.0;

/// Raster width of a window that has not reported a width yet,
/// in logical pixels.
pub const UNWRAPPED_RASTER_WIDTH: f32 = 1300.0;

/// What:
///  The width in logical pixels of the raster of a frame whose lines are broken at `wrap`,
///  or of unbroken lines when there is no wrap width.
/// Why:
///  The window's tiles and the minimap's bands are painted at the same width,
///  so a band holds the glyphs of a tile pixel for pixel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rasterWidth(wrap?: number): number;
/// ```
pub fn raster_width(wrap: Option<f32>) -> f32 {
    return match wrap {
        Some(width) => (width + RASTER_MARGIN).ceil(),
        None => UNWRAPPED_RASTER_WIDTH,
    };
}

/// What:
///  Whether a raster `columns` pixels wide and `rows` pixels tall holds at most [`MAX_RASTER_BYTES`].
/// Why:
///  Every check against the byte rule asks this one question,
///  so no caller repeats the multiplication or its overflow rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rasterFits(columns: number, rows: number): boolean;
/// ```
pub fn raster_fits(columns: u64, rows: u64) -> bool {
    return columns.saturating_mul(rows).saturating_mul(4) <= MAX_RASTER_BYTES;
}

/// What:
///  Pixels on a side of one bin of Vello's coarse stage:
///  16 pixels in a tile and 16 tiles in a bin.
/// Why:
///  Vello 0.11 counts the bins a render covers (`vello-0.11.0/src/render.rs`,
/// the warning "Trying to paint too large
///  image") and a texture is cut into bins from its top left corner,
///  so a texture off this grid counts the partial bin at its right and bottom edges whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const BIN_SIDE = 256;
/// ```
pub const BIN_SIDE: u64 = 256;

/// What:
///  Most bins Vello 0.11 counts in one render before it logs that the image is too large:
///  256 bins of 256 by 256 pixels are 16,777,216 pixels,
///  which is exactly [`MAX_RASTER_BYTES`] of `Rgba8Unorm`.
/// Why:
///  The byte rule alone allowed a texture at the cap whose edges are off the bin grid to cover more bins than this,
///  and the renderer logged the warning at every render of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_BINS = 256;
/// ```
pub const MAX_BINS: u64 = 256;

/// What:
///  Most textures a tile of the code view is stacked from.
/// Why:
///  The tile is three view heights tall,
///  so a window past the limits of one texture needs several,
///  and the count bounds the memory a tile may hold at four times [`MAX_RASTER_BYTES`]:
///  a view that is taller than that is the one exception (see [`stack_rows`]).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MAX_STACK_TEXTURES = 4;
/// ```
pub const MAX_STACK_TEXTURES: u64 = 4;

/// What:
///  Physical rows a stack keeps beyond the largest window the device can show,
///  so a view of the tallest window still has the rows that the rounding of a clip to whole code rows needs
///  and the overscan that a re-center keeps clear.
/// Why:
///  A window is never taller than the device's texture side,
///  so a stack of at least that many rows plus this room always holds the whole view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const STACK_VIEW_ROOM = 1024;
/// ```
pub const STACK_VIEW_ROOM: u64 = 1024;

/// What:
///  The most rows one texture of a tile `columns` pixels wide may have:
///  the device's side,
///  and the rows where Vello counts at most [`MAX_BINS`] bins,
///  which also keeps the bytes within [`MAX_RASTER_BYTES`].
///  Nothing when the width itself is more than a texture of the device can be.
/// Why:
///  A texture the renderer draws in one pass must be one it can draw correctly:
///  past 128 MiB it renders wrong without an error
///  (`doc/troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md`),
///  and past 256 bins it logs a warning at every render.
///  Counting the bins from the texture's width makes the limit a multiple of the bin side,
///  so a texture at the limit is on the grid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileTextureRows(columns: number): number;
/// ```
pub fn tile_texture_rows(columns: u64) -> u64 {
    let side = u64::from(texture_dimension());
    if columns == 0 || columns > side {
        return 0;
    }
    let bins_across = columns.div_ceil(BIN_SIDE);
    let by_bins = (MAX_BINS / bins_across).max(1) * BIN_SIDE;
    let by_bytes = MAX_RASTER_BYTES / 4 / columns;
    return side.min(by_bins).min(by_bytes);
}

/// What:
///  Whether one texture `columns` pixels wide and `rows` pixels tall is within the device's texture size and within
///  Vello's bin count ([`tile_texture_rows`]).
/// Why:
///  Every texture of a stack is asked this before the GPU is,
///  since the GPU would reject a texture past its size with a validation error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textureFits(columns: number, rows: number): boolean;
/// ```
pub fn texture_fits(columns: u64, rows: u64) -> bool {
    return rows <= tile_texture_rows(columns);
}

/// What:
///  The most rows a tile `columns` pixels wide may have all together:
///  [`MAX_STACK_TEXTURES`] textures of [`tile_texture_rows`],
///  or the device's texture side and [`STACK_VIEW_ROOM`] rows when that is more.
///  Nothing when the width is more than a texture can be.
/// Why:
///  A tile past one texture is drawn across several,
///  so its height is bounded by the memory it takes and not by one texture.
///  The tallest window the device can show is as tall as its texture side,
///  and the view of it must be held whatever the width,
///  so the bound never goes under a view and its room.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackRows(columns: number): number;
/// ```
pub fn stack_rows(columns: u64) -> u64 {
    let side = u64::from(texture_dimension());
    if columns == 0 || columns > side {
        return 0;
    }
    let by_count = MAX_STACK_TEXTURES * tile_texture_rows(columns);
    return by_count.max(side + STACK_VIEW_ROOM);
}

/// What:
///  Whether a tile `columns` pixels wide and `rows` pixels tall can be drawn across stacked textures:
///  within [`stack_rows`].
/// Why:
///  The painter refuses a view past it,
///  so a caller that did not clip the frame to the limit gets a message and not a GPU validation error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackFits(columns: number, rows: number): boolean;
/// ```
pub fn stack_fits(columns: u64, rows: u64) -> bool {
    return rows <= stack_rows(columns);
}

/// What:
///  The heights of the textures a tile `columns` pixels wide and `rows` tall is stacked from,
///  top first:
///  as many full [`tile_texture_rows`] as fit,
///  and the rest in the last.
///  Nothing for a tile with no rows or no width a texture can have.
/// Why:
///  The cuts between the textures stand at whole rows from the tile's top,
///  and every texture is as tall as the limit allows,
///  so a tile that one texture holds is one texture and the window tells nothing else from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textureHeights(columns: number, rows: number): number[];
/// ```
pub fn texture_heights(columns: u64, rows: u64) -> Vec<u32> {
    let most = tile_texture_rows(columns);
    let mut heights = Vec::new();
    if most == 0 {
        return heights;
    }
    let mut left = rows;
    while left > 0 {
        let height = left.min(most);
        heights.push(height as u32);
        left -= height;
    }
    return heights;
}

/// What:
///  Logical pixels a tile may be taller than the height the window asked it for,
///  for the rounding of its edges to whole code rows and the sizes of a tile's partial lines.
/// Why:
///  A cap exactly as tall as the asked height would cut the overscan the re-center keeps clear
///  every time a rounding added a row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const CAP_SLACK = 6 * CODE_ROW;
/// ```
pub const CAP_SLACK: f32 = 6.0 * CODE_ROW;

/// What:
///  The tallest tile in logical pixels whose stack of textures,
///  `columns` physical pixels wide at display `scale`,
///  stays within [`stack_rows`].
///  One code row is kept free for the rounding of the texture's size up to whole pixels,
///  and the answer is never under one code row.
/// Why:
///  This is the bound that no frame may pass whatever the window asked for;
///  the shaper guards every frame with it,
///  and the window asks for less ([`tile_cap`]).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stackCap(columns: number, scale: number): number;
/// ```
pub fn stack_cap(columns: u64, scale: f32) -> f32 {
    let rows = stack_rows(columns.max(1));
    return ((rows as f32 / scale).floor() - CODE_ROW).max(CODE_ROW);
}

/// What:
///  The tallest tile in logical pixels that the window lets a raster `columns` physical pixels wide at display
///  `scale` be when it wants a tile `wanted` logical pixels tall:
///  what one texture holds,
/// or `wanted` and [`CAP_SLACK`] when that is more,
///  and never more than [`stack_cap`].
/// Why:
///  A tile is a stack of textures of the text area's width,
/// so a window larger than one texture allows gets
///  several,
/// but a tile that the view and its overscan do not need is not made:
///  a line wrapped to thousands of rows is cut to the rows near the view exactly as it was
///  when a tile was one texture,
/// and only a window whose three views are taller than one texture
///  stacks more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tileCap(columns: number, scale: number, wanted: number): number;
/// ```
pub fn tile_cap(columns: u64, scale: f32, wanted: f32) -> f32 {
    let single =
        ((tile_texture_rows(columns.max(1)) as f32 / scale).floor() - CODE_ROW).max(CODE_ROW);
    return stack_cap(columns, scale).min(single.max(wanted + CAP_SLACK));
}

/// What:
///  The clip of lines that span `first_top` to `bottom` (logical pixels from the top of the text) for a view whose
///  three-height `band` is given,
/// when a raster may be `cap` pixels tall at most;
/// nothing while the lines fit.
///  The band may reach past the lines at either end;
/// its middle is where a raster taller than `cap` is centered.
///  The edges are whole code rows below `first_top`,
/// and the clip is never taller than `cap`.
/// Why:
///  The rounding to whole code rows adds up to two rows,
/// and the cap leaves only one,
/// so the height is cut back to the
///  rows the cap allows;
/// a view taller than the cap can never be held by such a tile,
/// so it is placed again at every
///  scroll step,
/// as a window that tall has no better answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bandClip(firstTop: number, bottom: number, band: [number, number], cap: number): Clip | undefined;
/// ```
pub fn band_clip(first_top: f32, bottom: f32, band: (f32, f32), cap: f32) -> Option<Clip> {
    if bottom - first_top <= cap {
        return None;
    }
    let mut from = band.0.max(first_top);
    let mut to = band.1.min(bottom);
    if to - from > cap {
        // What: The raster is as tall as the cap, centered on the middle of the band as it was asked for (the view's
        //       middle, when the band is three view heights around the view) and moved inside the lines.
        // Why: The ends of the text cut the band short on one side, which would move its middle away from the view.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // from = clamp(middle - cap / 2, firstTop, Math.max(firstTop, bottom - cap)); to = from + cap;
        // ```
        let middle = (band.0 + band.1) / 2.0;
        from = (middle - cap / 2.0).clamp(first_top, (bottom - cap).max(first_top));
        to = from + cap;
    }
    // Whole code rows below the first line's block keep the raster on whole pixels at the usual scales.
    let mut top = ((from - first_top) / CODE_ROW).floor().max(0.0) * CODE_ROW;
    let mut cut = ((to - first_top) / CODE_ROW).ceil() * CODE_ROW;
    cut = cut.min(bottom - first_top).max(top + CODE_ROW);
    // The rounding must not take the raster past what the cap allows.
    let allowed = (cap / CODE_ROW).floor().max(1.0) * CODE_ROW;
    if cut - top > allowed {
        // What: When the window reaches the last line, the top edge gives way, so the last row of the text stays in
        //       the raster; otherwise the bottom edge does.
        // Why: Rounding the top edge down must not push a raster that ends with the text short of its last row.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (to >= bottom - 0.5) top = Math.ceil((cut - allowed) / CODE_ROW) * CODE_ROW; else cut = top + allowed;
        // ```
        if to >= bottom - 0.5 {
            top = ((cut - allowed) / CODE_ROW).ceil() * CODE_ROW;
        } else {
            cut = top + allowed;
        }
    }
    return Some(Clip { top, bottom: cut });
}

/// What:
///  The clip a frame must carry so that its raster,
///  `columns` physical pixels wide at display `scale`,
///  is within the limit:
///  `clip` unchanged when its raster (all of the lines from `first_top` to `bottom` when it is `None`) fits,
///  otherwise the part of it [`band_clip`] keeps around its middle.
/// Why:
///  The window chooses a clip from the heights its map holds,
///  and a line it never measured counts as one row there;
///  the shaper sizes the raster from the heights it measured itself.
///  This is the one place where the frame's own final heights meet the limit,
///  so a frame the window judged too small can never reach the painter too large.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fitClip(firstTop: number, bottom: number, clip: Clip | undefined, columns: number, scale: number):
///   Clip | undefined;
/// ```
pub fn fit_clip(
    first_top: f32,
    bottom: f32,
    clip: Option<Clip>,
    columns: u64,
    scale: f32,
) -> Option<Clip> {
    let (from, to) = match clip {
        Some(held) => (first_top + held.top, first_top + held.bottom),
        None => (first_top, bottom),
    };
    let rows = ((to - from).max(CODE_ROW) * scale).ceil() as u64;
    if stack_fits(columns, rows) {
        return clip;
    }
    return band_clip(first_top, bottom, (from, to), stack_cap(columns, scale));
}

/// What:
///  `viewport` with its clip replaced by the one [`fit_clip`] gives for a raster `columns` pixels wide,
///  for the lines from `first_top` to `bottom`;
///  the cut is logged at warn level.
/// Why:
///  The shaper prepares every frame through this,
///  so the viewport a frame keeps names the clip its raster really has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fitViewport(viewport: Viewport, firstTop: number, bottom: number, columns: number): Viewport;
/// ```
pub fn fit_viewport(viewport: Viewport, first_top: f32, bottom: f32, columns: u64) -> Viewport {
    let fitted = fit_clip(first_top, bottom, viewport.clip, columns, viewport.scale);
    if fitted == viewport.clip {
        return viewport;
    }
    tracing::warn!(
        first = viewport.first,
        count = viewport.count,
        asked = ?viewport.clip,
        fitted = ?fitted,
        tile_top = first_top,
        tile_bottom = bottom,
        "cut a frame to the raster limit: the lines are taller than the window's map held"
    );
    return Viewport {
        clip: fitted,
        ..viewport
    };
}
