//! The window thread's own step when the wrap width changes:
//!  the lines near the view are broken again and the tile is painted for the new width before the next frame,
//!  within a counted allowance,
//! and a change that does not fit the allowance goes to the tile worker as a scroll
//!  re-center does.

/// The parent state,
/// the tile bounds,
/// and the tile's raster size.
use super::{State, render::viewport_of, tile};
/// The height of one code row.
use ide_app::row_map::CODE_ROW;
/// The budget a measurement spends,
/// and what it found.
use ide_app::shaped_text::{Budget, Measured};
/// A wrap width must be a positive number of pixels.
use ide_app::wrap::has_wrap_width;

/// What:
///  Most lines the window thread brings in (shapes or breaks again) for one change of width.
/// Why:
///  A tile of three view heights holds about a hundred lines at ordinary window sizes,
///  and one of an ultra-tall window a few hundred;
///  a view with more lines than this goes to the tile worker.
///  The window thread's row cache keeps at least this many lines,
/// so the lines of one step are not evicted by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REWRAP_LINES = 384;
/// ```
pub(super) const REWRAP_LINES: usize = 384;

/// What:
///  Most work,
/// in the units of `ide_app::shaped_text::line_work` (characters shaped or broken,
/// with a charge for
///  each line and for each corrective pass of a tabbed line),
/// the window thread spends on one change of width.
/// Why:
///  The release shaping rate measured on 2026-10-11 was 14 to 18 microseconds for a line of about 45 characters,
///  so this many units is a few milliseconds of shaping at most;
///  an ordinary tile of a hundred lines costs about a third of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REWRAP_WORK = 24_000;
/// ```
pub(super) const REWRAP_WORK: usize = 24_000;

/// What:
///  The most bytes the raster of a tile built for a new width on the window thread may hold.
/// Why:
///  Painting a tile copies its pixels into an image and uploads them,
/// so the step's cost grows with the raster;
///  the raster limit of 64 MiB for a tile built on the worker is too large to pay in one frame.
///  A window large enough that its tile passes this limit leaves the change to the worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REWRAP_RASTER_BYTES = 32 * 1024 * 1024;
/// ```
pub(super) const REWRAP_RASTER_BYTES: u64 = 32 * 1024 * 1024;

/// What:
///  What one change of width may still spend on the window thread.
/// Why:
///  Every line the step shapes or breaks again is charged before it runs,
/// so the total is bounded by the
///  allowance whatever the lines are;
/// a line that does not fit leaves the whole change to the tile worker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Allowance = { budget: Budget };
/// ```
pub(super) struct Allowance {
    /// Lines and work still available.
    budget: Budget,
}

/// Charging.
impl Allowance {
    /// A full allowance with nothing charged.
    fn new() -> Self {
        return Self {
            budget: Budget {
                lines: REWRAP_LINES,
                work: REWRAP_WORK,
            },
        };
    }

    /// Lines brought in so far.
    pub(super) fn lines(&self) -> usize {
        return REWRAP_LINES.saturating_sub(self.budget.lines);
    }

    /// Work spent so far.
    pub(super) fn work(&self) -> usize {
        return REWRAP_WORK.saturating_sub(self.budget.work);
    }

    /// What:
    ///  Measure the lines of the requested tile at the wrap width within what is left of the allowance;
    ///  nothing when a line does not fit.
    /// Why:
    ///  Lines the row cache already holds at the width are free;
    ///  the others are charged before they are shaped or broken again.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// measure(current: State, scale: number, bounds: TileBounds): Measured | undefined;
    /// ```
    pub(super) fn measure(
        &mut self,
        current: &mut State,
        scale: f32,
        bounds: tile::TileBounds,
    ) -> Option<Measured> {
        let State {
            shaper,
            document,
            styles,
            ..
        } = current;
        return shaper.measure_lines_within(
            document,
            bounds.first..bounds.first + bounds.count,
            scale,
            Some(bounds.width),
            styles,
            &mut self.budget,
        );
    }
}

/// What:
///  Whether the raster of the tile at `bounds`,
/// at display scale `scale`,
/// is within [`REWRAP_RASTER_BYTES`].
/// Why:
///  The raster is as wide as the text area and as tall as the lines of the tile,
/// or the clip of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rasterFits(current: State, bounds: TileBounds, scale: number): boolean;
/// ```
pub(super) fn raster_fits(current: &State, bounds: tile::TileBounds, scale: f32) -> bool {
    let columns = (viewport_of(bounds, scale).width * scale).ceil() as u64;
    let map = &current.row_map;
    let extent = match bounds.clip {
        Some(clip) => clip.bottom - clip.top,
        None => {
            let end = (bounds.first + bounds.count).min(map.lines());
            map.block_top(end) - map.block_top(bounds.first)
        }
    };
    let rows = (extent.max(CODE_ROW) * scale).ceil() as u64;
    return columns * rows * 4 <= REWRAP_RASTER_BYTES;
}

/// What:
///  Start the immediate step for a width change the viewport just reported,
/// or hand the change to the worker.
///  The answer is the allowance when the window thread takes the step,
/// otherwise nothing.
/// Why:
///  A tile whose lines alone are more than the allowance covers is not tried at all;
///  its change is queued for the worker like a scroll re-center,
/// and the displayed tile stays on screen until it
///  arrives.
///  A flag left over from a report whose render never ran is dropped when the width no longer differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function begin(current: State): Allowance | undefined;
/// ```
pub(super) fn begin(current: &mut State) -> Option<Allowance> {
    if !std::mem::take(&mut current.rewrap) {
        return None;
    }
    let requested = tile::requested(current);
    if current.shown.width <= 0.0
        || !has_wrap_width(requested.width)
        || current.shown.width == requested.width
    {
        return None;
    }
    if requested.count > REWRAP_LINES {
        defer(
            current,
            "the tile holds more lines than the allowance covers",
        );
        return None;
    }
    if !raster_fits(current, requested, current.scale) {
        defer(
            current,
            "the raster of the tile is larger than the window thread paints in one frame",
        );
        return None;
    }
    return Some(Allowance::new());
}

/// What:
///  Leave the change of width to the tile worker:
/// the requested tile is queued as a scroll re-center is,
///  and the displayed tile stays on screen until the worker's tile arrives.
/// Why:
///  This is the path every change of width took before the window thread learned to take it,
/// so a change that does
///  not fit the allowance behaves as it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function defer(current: State, reason: string): void;
/// ```
pub(super) fn defer(current: &mut State, reason: &str) {
    let requested = tile::requested(current);
    current.scroll_wanted = Some(requested);
    #[cfg(debug_assertions)]
    {
        current.rewrap_deferred += 1;
    }
    // The target is the render module's, so the log filter that shows the tile timings shows this line too.
    tracing::debug!(
        target: "monochromatic_ide::native::render",
        reason,
        first = requested.first,
        count = requested.count,
        "left a new width to the tile worker"
    );
}
