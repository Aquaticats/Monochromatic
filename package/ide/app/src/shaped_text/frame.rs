//! Types of a prepared frame:
//!  the part of the lines a raster holds,
//!  the viewport it is prepared for,
//!  and the shaped view that reading geometry and painting share.

/// Positioned hints and diagnostic marks travel with the frame they were laid out against.
use crate::annotation_layout::AnnotationFrame;
/// The one vertical mapping:
///  where each line's code row starts and how tall rows and the caret are.
use crate::row_map::RowMap;
/// Shaped rows and the rectangles that reading geometry derives from them.
use crate::shaped_row::{ReadingRect, ShapedRow};

/// What:
///  The part of a tile's lines that its raster holds,
///  as two distances in logical pixels from the top of the first materialized line's block.
/// Why:
///  A tile holds whole lines,
/// and one line wrapped to thousands of rows would pass the raster limit.
///  The clip keeps the rows near the view;
/// rows outside it are not painted,
///  and the gutter and the other row lists leave them out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Clip = { top: number; bottom: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    /// Where the raster starts,
    /// below the top of the first line's block.
    pub top: f32,
    /// Where the raster ends,
    /// below the top of the first line's block.
    pub bottom: f32,
}

/// Logical dimensions and scale of the source viewport.
#[derive(Clone, Copy, PartialEq)]
pub struct Viewport {
    /// First logical line materialized,
    ///  including overscan.
    pub first: usize,
    /// Maximum number of source lines materialized.
    pub count: usize,
    /// Width of the raster in logical pixels.
    pub width: f32,
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// Wrap width in logical pixels that every line is broken at,
    ///  or nothing to leave lines unbroken.
    pub wrap: Option<f32>,
    /// The part of the materialized lines the raster holds,
    ///  or nothing for all of them.
    pub clip: Option<Clip>,
}

/// Viewport text and selection geometry share these exact shaped rows.
pub struct ShapedView {
    /// Visible source rows.
    pub rows: Vec<ShapedRow>,
    /// Viewport request retained for coordinate conversion.
    pub viewport: Viewport,
    /// Physical raster width.
    pub width: u32,
    /// Physical raster height.
    pub height: u32,
    /// Top of the raster in logical pixels from the top of the text:
    ///  the top of the first materialized
    /// line's block,
    /// or the clip's top below it.
    pub origin: f32,
    /// The vertical mapping this frame was shaped against;
    ///  rows outside the frame are placed by it.
    pub map: RowMap,
    /// Shared logical selection rectangles drive both native backgrounds and glyph clipping.
    pub selections: Vec<ReadingRect>,
    /// In-file find rectangles from the same row geometry;
    ///  filled by the native renderer.
    pub matches: Vec<ReadingRect>,
    /// What:
    ///  `Option<AnnotationFrame>` is the frame's positioned hints and diagnostic marks,
    ///  or nothing.
    /// Why:
    ///  The renderer fills it after shaping;
    ///  rows are never changed by it,
    ///  so reading geometry ignores it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// annotations?: AnnotationFrame;
    /// ```
    pub annotations: Option<AnnotationFrame>,
}
