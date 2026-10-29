//! The minimap:
//! the scroll control of the source view and the picture of the text it shows,
//!  sized as CodeGlance Pro sizes it.
//!
//! The geometry (`geometry`) is pure:
//! the picture's fixed scale,
//! the track,
//! how far a long text's picture has slid past it,
//! the viewport box,
//! and the maps between a
//!  position on the minimap,
//! a scroll offset,
//! and a visual row of the vertical mapping.
//!
//! The picture is drawn by the GPU from the window's own shaped lines,
//!  zoomed out,
//!  and it holds only the bands near the visible part of it.
//!  The bands are planned (`plan`),
//!  the lines of the window's bands are shaped once and kept by what shaped them (`line_cache`),
//!  and `scene` turns them,
//!  with the selection,
//!  the matches,
//!  and the annotations the window draws (`fills`),
//!  into Vello scenes scaled to the minimap's pixels,
//!  one scene for each chunk of at most 512 rows (`chunks`),
//!  on a worker thread.
//!  The window renders the scenes one after another into the textures of the picture,
//!  each of whole chunks and no taller than the device's texture limit (`texture_runs`),
//!  and shows the textures as images,
//!  one below the other.
//!  A window taller than the limit therefore has a picture that covers its whole track.

/// The paint items cut into the chunks a slice is rendered in.
mod chunks;
/// The markup's selection and match fills behind the glyphs.
mod fills;
/// The minimap's sizes,
/// the viewport box,
/// and the maps between pixels and visual rows.
mod geometry;
/// The horizontal map of the picture.
mod horizontal;
/// The shaped lines the picture is drawn from,
/// kept by what shaped them.
mod line_cache;
/// The bands the picture is cut into,
/// and the rows a run of bands makes up.
mod plan;
/// The picture's scene,
/// its inputs,
/// and what a build reports.
mod scene;
/// Shaping the lines of a slice and placing them in a view.
mod shape;

/// The cut of the paint items into chunks.
pub use chunks::cut;
/// The fill inks and the items that draw the marks.
pub use fills::{Fills, fill_items};
/// The geometry's sizes,
///  the target of a position,
///  and the maps between pixels and visual rows.
pub use geometry::{
    Geometry, MIN_BOX, NARROWEST, PIXELS_PER_ROW, Target, VIEW_SHARE, WIDTH, width_for,
};
/// A position across the minimap and the text's width.
pub use horizontal::{text_span, text_x};
/// The shaped lines the builds keep.
pub use line_cache::{LineCache, LineKey, line_key};
/// The bands,
/// and the chunks a slice is rendered in.
pub use plan::{
    BAND, CHUNK_ROWS, Heading, LEAD, MIN_SCALE, band_count, chunk_rows, most_bands, picture_rows,
    slice_rows, texture_rows, texture_runs,
};
/// The picture's build,
///  its inputs,
///  its work budget,
///  and what it reports.
pub use scene::{BUILD_WORK, Built, Cancel, Input, Source, Stats, build, slice_items};
/// The blocks and marks that belong to a run of lines.
pub use shape::{blocks_between, marks_between};
