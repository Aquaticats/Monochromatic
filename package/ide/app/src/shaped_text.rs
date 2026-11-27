//! Shared shaped rows replace independently centered fallback-font glyph items.

/// Shaped rows kept by line for one text revision and scale.
use crate::shaped_row_cache::RowCache;
/// Immutable font requests validate continuous weights before shaping.
use crate::source_typography::SourceTypography;
/// Font and paragraph layout are supplied by the same Parley stack Slint uses.
use parley::{
    FontContext,
    LayoutContext,
};

/// What:
///  `pub use` re-exports the row types under this module's name.
/// Why:
///  Callers written against `shaped_text` keep their imports while row geometry lives in its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { ReadingRect, ShapedRow } from './shaped-row';
/// ```
pub use crate::shaped_row::{
    ReadingRect,
    ShapedRow,
    VisualCaret,
};

/// Reading geometry of a prepared frame:
///  hits,
///  caret,
///  selection and range rectangles,
///  and rebasing.
mod view;

/// Whole-file passes:
/// bringing rows into the cache for a wrap width,
///  and breaking kept rows again at a new width.
mod pass;
/// What:
///  `pub use` re-exports the pass results and the work measure under this module's name.
/// Why:
///  Callers written against `shaped_text` keep their imports while the passes live in their own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { Budget, Ensured, LINE_WORK, Measured, lineWork } from './pass';
/// ```
pub use pass::{
    Budget,
    Ensured,
    LINE_WORK,
    Measured,
    line_work,
};

/// Types of a prepared frame:
///  the clip of a raster,
///  the viewport,
///  and the shaped view.
mod frame;
/// What:
///  `pub use` re-exports the frame types under this module's name.
/// Why:
///  Callers written against `shaped_text` keep their imports while the types live in their own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { Clip, ShapedView, Viewport } from './frame';
/// ```
pub use frame::{
    Clip,
    ShapedView,
    Viewport,
};

/// The raster limit of a tile and the clip rules that keep a frame within it.
mod limit;
/// What:
///  `pub use` re-exports the limit rules under this module's name.
/// Why:
///  The window chooses a tile's clip with the same rules the shaper guards its frames with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { RASTER_MARGIN, UNWRAPPED_RASTER_WIDTH, bandClip, fitClip, fitViewport, rasterFits, rasterWidth, tileCap } from './limit';
/// ```
pub use limit::{
    BIN_SIDE,
    CAP_SLACK,
    DEFAULT_TEXTURE_DIMENSION,
    MAX_BINS,
    MAX_RASTER_BYTES,
    MAX_STACK_TEXTURES,
    RASTER_MARGIN,
    STACK_VIEW_ROOM,
    UNWRAPPED_RASTER_WIDTH,
    band_clip,
    fit_clip,
    fit_viewport,
    raster_fits,
    raster_width,
    set_texture_dimension,
    stack_cap,
    stack_fits,
    stack_rows,
    texture_dimension,
    texture_fits,
    texture_heights,
    tile_cap,
    tile_texture_rows,
};

/// Shaping of single lines and virtual rows with the shared font setup.
mod layout;
/// Shaping one line without the row cache.
mod line;
/// The inputs that shape one source line,
///  which the tab passes build too.
pub(crate) use layout::LineRequest;

/// The shaper's construction,
///  its row and view entry points,
///  and its counters, fork and forget.
mod shaper;

/// What:
///  `pub const` exports a compile-time value;
///  `f32` is a 32-bit float of logical pixels (sibling `f64`).
/// Why:
///  A selected line terminator has no glyph;
///  a mark about one space wide shows that it is selected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TERMINATOR_MARK = 9;
/// ```
pub const TERMINATOR_MARK: f32 = 9.0;

/// What:
///  Paint role of virtual-row glyphs,
///  a number outside the syntax roles;
///  `u32` is the brush type of layouts.
/// Why:
///  The raster paints hints and messages with one given ink each,
///  never with a syntax or selection color.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const ROW_ROLE = 65;
/// ```
pub const ROW_ROLE: u32 = 65;

/// Own font discovery and shaping scratch space,
///  rather than recreate per glyph.
pub struct TextShaper {
    /// Primary and fallback fonts,
    ///  including the embedded source face.
    fonts: FontContext,
    /// Reusable paragraph-building allocations.
    layouts: LayoutContext<u32>,
    /// Immutable variable-weight,
    ///  italic,
    ///  and feature policy shared by rows and baseline probes.
    typography: SourceTypography,
    /// Measured space advance for one scale;
    ///  typography is immutable,
    ///  so only the scale can change it.
    space: Option<(f32, f32)>,
    /// Shaped rows by line for the last synced text revision and scale;
    ///  `prepare_rows` reads and fills it.
    rows: RowCache,
    /// Source rows shaped by this shaper,
    /// never reset.
    shaped_rows: u64,
    /// Kept rows broken again at a new wrap width instead of being shaped again,
    /// never reset.
    rebroken_rows: u64,
    /// Corrective shaping passes of tab-bearing wrapped lines since this shaper was made.
    tab_passes: u64,
}

/// What:
///  Which line to bring into the row cache,
///  at what scale and wrap width,
///  against which common baseline.
/// Why:
///  Ensuring a row and shaping one take the same four inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RowRequest = { row: number; scale: number; wrap?: number; baseline: number };
/// ```
#[derive(Clone, Copy)]
struct RowRequest {
    /// Line to bring in.
    row: usize,
    /// Physical pixels per logical pixel.
    scale: f32,
    /// Wrap width in logical pixels.
    wrap: Option<f32>,
    /// Common baseline of every row in physical pixels.
    baseline: f32,
}
