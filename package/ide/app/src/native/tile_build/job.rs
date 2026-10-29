//! The inputs and result of one source tile,
//!  and the construction both the window thread and the tile worker run.

/// Requested and displayed tile bounds travel with each job.
use crate::native::tile::TileBounds;
/// What a measurement is valid for.
use crate::native::tile_pass::PassKey;
/// Build failures stay visible diagnostics.
use anyhow::Result;
/// Visible hints and diagnostic marks are paint inputs of the tile.
use ide_app::annotation::Visible;
/// Hint and diagnostic inks,
/// and the lay-out that places them beside the rows.
use ide_app::annotation_layout::{AnnotationColors, lay_out};
/// The palette of one tile.
use ide_app::code_colors::CodeColors;
/// Canonical source text,
/// cloned as a rope snapshot.
use ide_app::document::Document;
/// In-file find ranges and the active match,
/// painted from the same rows as selection.
use ide_app::find::{FindRange, FindRanges};
/// Match rectangles of the materialized rows.
use ide_app::find_paint::rectangles;
/// The one vertical mapping,
/// copied into each job.
use ide_app::row_map::RowMap;
/// Shared shaping,
/// the viewport description,
/// and the frame's rows.
use ide_app::shaped_text::{ShapedView, TextShaper, Viewport};
/// Paint identity of a tile,
/// compared again when its result arrives.
use ide_app::source_frame::FrameStamp;
/// Syntax classifications,
/// compared by shared identity.
use ide_app::source_style::SourceStyles;
/// Shaping choices a worker shaper must match.
use ide_app::source_typography::SourceTypography;
/// The scenes of one tile,
/// which cross to the window thread.
use ide_app::tile_scene::{TileStack, paint_stack};

/// What:
///  `struct` groups named fields into one owned value;
///  every field here is owned,
/// so the whole job moves to another thread.
/// Why:
///  The worker must not borrow native state,
/// which lives on the window thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TileJob = { bounds: Bounds; document: Document; styles: Styles; /* paint inputs */ };
/// ```
pub(in crate::native) struct TileJob {
    /// Tile bounds this job builds,
    /// echoed back in the result.
    pub(in crate::native) bounds: TileBounds,
    /// File-open generation the snapshot belongs to.
    pub(in crate::native) file: u64,
    /// Rope snapshot at the revision the job was taken from.
    pub(in crate::native) document: Document,
    /// Syntax classifications,
    /// shared by identity.
    pub(in crate::native) styles: SourceStyles,
    /// Shaping choices the worker shaper must match.
    pub(in crate::native) typography: SourceTypography,
    /// Materialized lines,
    /// width,
    /// and scale.
    pub(in crate::native) viewport: Viewport,
    /// Vertical mapping the rows are placed by.
    pub(in crate::native) map: RowMap,
    /// Palette of the source glyphs and selection.
    pub(in crate::native) colors: CodeColors,
    /// Hint and diagnostic inks.
    pub(in crate::native) inks: AnnotationColors,
    /// Visible annotations of the tile's lines.
    pub(in crate::native) shown: std::sync::Arc<Visible>,
    /// Find ranges to paint as matches.
    pub(in crate::native) ranges: FindRanges,
    /// The match the find bar selects.
    pub(in crate::native) active: Option<FindRange>,
    /// Paint identity the result must still match.
    pub(in crate::native) stamp: FrameStamp,
}

/// What:
///  A finished tile:
/// the shaped rows and rectangles,
/// and the scene to draw.
/// Why:
///  The window thread swaps all of them in together,
/// so the image and its offset always belong to one tile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TileBuilt = { bounds: Bounds; view: ShapedView; scene: Scene[]; stamp: Stamp };
/// ```
pub(in crate::native) struct TileBuilt {
    /// Tile bounds that were built:
    /// the requested ones,
    /// with the clip the frame was really cut to when the shaper had to cut it to the raster limit.
    pub(in crate::native) bounds: TileBounds,
    /// Tile bounds that were asked for,
    /// which the window matches against the job it is waiting for.
    pub(in crate::native) requested: TileBounds,
    /// File-open generation the tile was built for.
    pub(in crate::native) file: u64,
    /// Shaped rows,
    /// rectangles,
    /// and annotation positions.
    pub(in crate::native) view: ShapedView,
    /// The Vello scenes of the tile and the sizes of the textures they are drawn into,
    /// top first;
    /// the GPU turns them into pixels.
    pub(in crate::native) scene: TileStack,
    /// Paint identity the tile was built for.
    pub(in crate::native) stamp: FrameStamp,
    /// Active match used for the match rectangles.
    pub(in crate::native) active: Option<FindRange>,
    /// What the visual rows measured while shaping the tile's lines are valid for.
    pub(in crate::native) key: PassKey,
    /// Visual rows of each line of the tile,
    /// exact for the wrap width of its key.
    pub(in crate::native) measured: Vec<(usize, usize)>,
    /// Source rows shaped while building this tile,
    /// counted in debug builds for the structural tests.
    #[cfg(debug_assertions)]
    pub(in crate::native) rows_shaped: u64,
}

/// Test-only switch:
/// the next build refuses to paint,
/// as a raster past the limit would,
/// so a test can see what the window does when a tile fails.
#[cfg(test)]
pub(in crate::native) static FAIL_NEXT_BUILD: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What:
///  `fn` declares a function;
/// `&mut` lends the shaper for this call only;
///  `Result<TileBuilt>` is the built tile or the reason it could not be built.
/// Why:
///  The same code builds a tile on the window thread and on the worker,
/// so both produce the same scene.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildTile(shaper: TextShaper, job: TileJob): TileBuilt;
/// ```
pub(in crate::native) fn build_tile(shaper: &mut TextShaper, job: TileJob) -> Result<TileBuilt> {
    #[cfg(test)]
    if FAIL_NEXT_BUILD.swap(false, std::sync::atomic::Ordering::SeqCst) {
        anyhow::bail!("Source viewport refused by the test switch");
    }
    #[cfg(debug_assertions)]
    let before = shaper.shaped_rows();
    let mut view = shaper.prepare_rows(&job.document, job.viewport, &job.styles, &job.map);
    // What: The shaper cuts a frame to the raster limit when its measured lines are taller than the window's map held;
    //       the bounds and the stamp then name the clip that was really painted.
    // Why: The window shows a tile by its bounds, and compares the stamp to the inputs it would paint now.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const bounds = { ...job.bounds, clip: view.viewport.clip };
    // ```
    let bounds = TileBounds {
        clip: view.viewport.clip,
        ..job.bounds
    };
    let stamp = if bounds == job.bounds {
        job.stamp
    } else {
        job.stamp.with_clip(bounds.clip)
    };
    // The lines this tile shaped know their own heights; the window merges them into its table.
    let mut measured = Vec::new();
    if job.viewport.wrap.is_some() {
        for row in &view.rows {
            measured.push((row.row, row.visual_rows()));
        }
    }
    view.matches = rectangles(&view, &job.ranges, job.active);
    // Annotations are positioned after shaping and never change the geometry inside a code row.
    let frame = lay_out(&view, &job.shown, shaper, job.inks);
    view.annotations = Some(frame);
    // The scene is drawn from the very layouts that measured the rows, so glyphs and geometry cannot disagree.
    let scene = paint_stack(&view, job.colors, 0.0)?;
    return Ok(TileBuilt {
        bounds,
        requested: job.bounds,
        file: job.file,
        view,
        scene,
        stamp,
        active: job.active,
        key: PassKey {
            file: job.file,
            revision: job.document.revision(),
            scale: job.viewport.scale.to_bits(),
            wrap: job.viewport.wrap.map_or(0, f32::to_bits),
        },
        measured,
        #[cfg(debug_assertions)]
        rows_shaped: shaper.shaped_rows() - before,
    });
}

/// Whether two typographies make the same glyphs.
pub(in crate::native) fn same_typography(
    left: &SourceTypography,
    right: &SourceTypography,
) -> bool {
    return left.weight == right.weight
        && left.italic == right.italic
        && left.features == right.features;
}
