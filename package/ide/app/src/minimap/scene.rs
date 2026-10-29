//! The minimap's picture as GPU scenes:
//!  the lines near the visible part of the picture,
//!  drawn as the window draws them (the same shaped rows,
//!  inks,
//!  selection and match marks,
//!  hints,
//!  messages,
//!  and underlines),
//!  scaled down to the minimap's pixels.
//!
//! Nothing here fills a pixel.
//! The scene holds glyph outlines,
//!  rectangles,
//!  and strokes,
//!  and the GPU turns them into the minimap's pixels:
//!  a minimap pixel is the coverage of the outlines it stands for,
//!  so the picture is the average the window's own pixels give,
//!  without a reduction pass.
//! A slice is cut into chunks of at most 512 rows,
//! a scene for each,
//!  because the renderer's scratch buffers hold a fixed amount of work.

/// The paint items cut into the chunks.
use super::chunks::cut;
/// The marks drawn under the glyphs.
use super::fills::{Fills, fill_items};
/// The geometry of the picture:
/// its fixed scale.
use super::geometry::Geometry;
/// The width of the text the picture covers.
use super::horizontal::text_span;
/// Shaped lines kept by what shaped them.
use super::line_cache::LineCache;
/// The rows of the picture a slice holds.
use super::plan::{MIN_SCALE, chunk_rows, slice_rows};
/// The lines of a slice and the view that carries their marks.
use super::shape::{lines_of, shape_lines, view_of};
/// Diagnostic marks.
use crate::annotation::Mark;
/// The positioning of hints and messages beside the rows.
use crate::annotation_layout::AnnotationColors;
/// The palette of the glyphs.
use crate::code_colors::CodeColors;
/// Canonical text,
/// whose reading position is the selection.
use crate::document::Document;
/// Matches of the in-file find.
use crate::find::FindRange;
/// The vertical mapping the lines are placed by.
use crate::row_map::{CODE_ROW, RowMap};
/// Shaping,
/// and the frame it prepares.
use crate::shaped_text::TextShaper;
/// Syntax classes by character offset.
use crate::source_style::StyleSpan;
/// The paint items of a shaped view,
/// and the scene they encode to.
use crate::tile_scene::{Item, TileScene, encode, tile_items};
/// Blocks of virtual rows.
use crate::virtual_row::Block;
/// Blocks are shared between the window and the worker;
/// a window of bands is a range of numbers.
use std::{ops::Range, sync::Arc};
/// Vello's scene and transforms.
use vello::{Scene, kurbo::Affine};

/// What:
///  Work,
///  in the units of `line_work`,
///  a picture may spend shaping lines.
/// Why:
///  A picture shapes only the lines near the visible part of the picture,
///  so ordinary text costs a small part of this;
///  a line the rows kept do not hold,
///  whose cost is more than what is left,
///  is drawn blank.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const BUILD_WORK = 5_000_000;
/// ```
pub const BUILD_WORK: usize = 5_000_000;

/// What:
///  The question a build asks between lines:
///  whether its result is still wanted.
/// Why:
///  A newer request or a closed window makes the rest of the build wasted work,
///  and the build ends early with nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Cancel { isCancelled(): boolean }
/// ```
pub trait Cancel {
    /// Whether the build should stop.
    fn is_cancelled(&self) -> bool;
}

/// What:
///  Everything a picture is drawn from.
/// Why:
///  One value borrowed from the worker's snapshot,
///  so a build touches no window state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Source = { document: Document; styles: StyleSpan[]; map: RowMap; colors: CodeColors; inks: AnnotationColors; ... };
/// ```
#[derive(Clone, Copy)]
pub struct Source<'a> {
    /// The text and its reading position.
    pub document: &'a Document,
    /// Syntax classes by character offset,
    /// ascending.
    pub styles: &'a [StyleSpan],
    /// The window's vertical mapping.
    pub map: &'a RowMap,
    /// The palette of the glyphs.
    pub colors: CodeColors,
    /// The inks of hints and diagnostics.
    pub inks: AnnotationColors,
    /// The inks of the selection and match fills.
    pub fills: Fills,
    /// Every block of virtual rows,
    /// in line order.
    pub blocks: &'a [Arc<Block>],
    /// Every diagnostic mark,
    /// in start order.
    pub marks: &'a [Mark],
    /// The matches of the in-file find,
    /// in order.
    pub ranges: &'a [FindRange],
    /// The match the selection covers.
    pub active: Option<FindRange>,
    /// The width lines are broken at,
    /// in logical pixels.
    pub wrap: Option<f32>,
    /// Physical pixels per logical pixel.
    pub scale: f32,
}

/// What:
///  What a build reads:
///  the source,
///  the sizes,
///  and the bands to draw.
/// Why:
///  Every field is borrowed from a snapshot the worker owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Input = { source: Source; view: number; width: number; window: [number, number] };
/// ```
#[derive(Clone)]
pub struct Input<'a> {
    /// What the picture is drawn from.
    pub source: Source<'a>,
    /// Height of the view,
    /// in logical pixels,
    /// which is the most the picture can be.
    pub view: f32,
    /// Width of the picture,
    /// in logical pixels.
    pub width: f32,
    /// The numbers of the bands to draw,
    /// counted from the top of the text.
    pub window: Range<usize>,
}

/// What:
///  What one build cost and what it left out.
/// Why:
///  The worker logs it and the tests assert the bounds from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stats = { units: number; lines: number; shaped: number; reused: number; skipped: number; work: number; items: number };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Bands of the window.
    pub units: usize,
    /// Lines the picture draws.
    pub lines: usize,
    /// Lines shaped in this build.
    pub shaped: usize,
    /// Lines taken from the row cache.
    pub reused: usize,
    /// Lines the work budget could not pay for,
    /// which are drawn blank.
    pub skipped: usize,
    /// Work charged for the lines shaped.
    pub work: usize,
    /// Paint items of the scene.
    pub items: usize,
}

/// What:
///  A finished picture:
///  the scene of each chunk of the slice,
///  top first,
///  and the row of the whole picture the slice's first row is.
///  The chunks are as wide as the slice and stack without a gap;
///  the slice is as tall as they are together,
///  and the GPU draws it into one texture or,
///  past the device's texture limit,
///  into several that stack.
///  No chunk when the slice holds no rows.
/// Why:
///  The window thread hands the scenes to the GPU and places the textures by the top row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Built = { chunks: TileScene[]; top: number; stats: Stats };
/// ```
pub struct Built {
    /// The scene of each chunk and the size of the texture region it fills.
    pub chunks: Vec<TileScene>,
    /// The row of the whole picture that the first row of the texture is.
    pub top: u32,
    /// What the build cost.
    pub stats: Stats,
}

/// What:
///  The paint items of the picture of `input`,
///  in physical pixels of the text at the display scale,
///  with the rows of the whole picture they fill.
///  Nothing when the window holds no rows or the build was cancelled.
/// Why:
///  The items are the picture before it is scaled down:
///  the marks first,
///  then the glyphs,
///  then the underlines,
///  and the tests read them as they read a tile's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sliceItems(shaper: TextShaper, cache: LineCache, input: Input, cancelled: Cancel): { items: Item[]; rows: [number, number]; stats: Stats } | undefined;
/// ```
pub fn slice_items(
    shaper: &mut TextShaper,
    cache: &mut LineCache,
    input: &Input,
    cancelled: &dyn Cancel,
) -> Option<(Vec<Item>, Range<usize>, Stats)> {
    let source = &input.source;
    let scale = source.scale.max(MIN_SCALE);
    let ratio = f64::from(Geometry::new(source.map.height(), input.view).scale());
    let rows = slice_rows(&input.window, source.map.height(), scale, ratio);
    let mut stats = Stats {
        units: input.window.len(),
        ..Stats::default()
    };
    if rows.is_empty() {
        return Some((Vec::new(), rows, stats));
    }
    let lines = lines_of(source.map, &rows, scale, ratio);
    stats.lines = lines.len();
    cache.begin();
    let first = lines.start;
    let shaped = shape_lines(shaper, cache, source, lines, &mut stats, cancelled)?;
    cache.finish();
    let view = view_of(shaper, source, shaped, first);
    let mut items = fill_items(&view, source.fills);
    match tile_items(&view, source.colors, 0.0) {
        Ok(drawn) => items.extend(drawn),
        Err(error) => {
            tracing::error!(%error, "the minimap's lines could not be turned into paint items")
        }
    }
    stats.items = items.len();
    return Some((items, rows, stats));
}

/// What:
///  Draw the picture of `input`:
///  the items of its slice,
///  cut into chunks of at most [`CHUNK_ROWS`](super::plan::CHUNK_ROWS) rows,
///  each scaled to the minimap's width across and the fixed scale down,
///  and placed so the first row of its texture region is the chunk's first row.
/// Why:
///  The picture shows the text's whole width in the minimap's width,
///  so the horizontal factor is the minimap's pixels over the text's;
///  the vertical factor is the picture's fixed scale.
///  A chunk's scene holds only the items whose ink can reach its rows,
///  so the renderer's work for a slice is what the slice costs,
///  however many chunks it has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function build(shaper: TextShaper, cache: LineCache, input: Input, cancelled: Cancel): Built | undefined;
/// ```
pub fn build(
    shaper: &mut TextShaper,
    cache: &mut LineCache,
    input: &Input,
    cancelled: &dyn Cancel,
) -> Option<Built> {
    let (items, rows, stats) = slice_items(shaper, cache, input, cancelled)?;
    let source = &input.source;
    let scale = source.scale.max(MIN_SCALE);
    let geometry = Geometry::new(source.map.height(), input.view);
    let ratio = f64::from(geometry.scale());
    let width = ((input.width * scale).round() as usize).max(1);
    let across = f64::from(text_span(source.wrap)) * f64::from(scale);
    let spans = chunk_rows(&rows);
    // The text rows each chunk reaches: a glyph's ink reaches a code row or two past its layout box.
    let mut reaches: Vec<(f32, f32)> = Vec::with_capacity(spans.len());
    for span in &spans {
        reaches.push((
            (span.start as f64 / ratio) as f32,
            (span.end as f64 / ratio) as f32,
        ));
    }
    let cuts = cut(&items, &reaches, 2.0 * CODE_ROW * scale);
    // A clip item clips to the text's whole height, so a tall text never cuts a glyph run short.
    let extent = (source.map.height().max(CODE_ROW) * scale).ceil() + 1.0;
    let mut chunks = Vec::with_capacity(spans.len());
    for (span, kept) in spans.iter().zip(&cuts) {
        let mut scene = Scene::new();
        if !kept.is_empty() {
            let transform = Affine::translate((0.0, -(span.start as f64)))
                * Affine::scale_non_uniform(width as f64 / across, ratio);
            scene.append(&encode(kept, extent), Some(transform));
        }
        chunks.push(TileScene {
            scene,
            width: width as u32,
            height: span.len() as u32,
        });
    }
    return Some(Built {
        chunks,
        top: rows.start as u32,
        stats,
    });
}
