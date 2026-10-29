//! A tile's paint items flattened to what each draw command shows:
//!  the ink,
//! the glyphs,
//! the place,
//! and the clip it is drawn in.
//!
//! Tests read a tile through this instead of through pixels,
//!  so that "this text is drawn in the selection ink inside the selection and not outside it"
//!  is a statement about data.
//! The GPU tests then show that the data lands on the screen.

/// The items to flatten.
use super::{Item, StrokeItem};
/// The box of a path.
use vello::kurbo::Shape;

/// What:
///  One glyph run as it is drawn:
///  the ink,
/// the glyph ids,
/// the layout origin,
/// and the x range and y band it is clipped to.
/// Why:
///  A selected ligature is drawn twice,
/// once clipped to each side of the selection edge;
///  a test tells the two apart by their clip and ink,
///  and tells a clip as tall as a row from one as tall as the tile by its band.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DrawnRun = { clip?: [number, number]; band?: [number, number]; ink: Rgba; ids: number[]; origin: [number, number]; left: number; right: number; baseline: number };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DrawnRun {
    /// The x range this run is clipped to,
    ///  or nothing when it is drawn unclipped.
    pub clip: Option<(f32, f32)>,
    /// The y range this run is clipped to,
    ///  or nothing when it is drawn unclipped.
    pub band: Option<(f32, f32)>,
    /// Straight RGBA ink.
    pub ink: [u8; 4],
    /// Glyph ids in drawing order.
    pub ids: Vec<u32>,
    /// Physical position of the layout's origin.
    pub origin: (f32, f32),
    /// Physical x of the left edge of the first glyph.
    pub left: f32,
    /// Physical x of the right edge of the last glyph's origin.
    pub right: f32,
    /// Physical y of the baseline the run stands on.
    pub baseline: f32,
}

/// What:
///  One stroke as it is drawn:
///  the stroke and the x range and y band it is clipped to.
/// Why:
///  Underlines under a selection are drawn twice in different inks,
/// as glyphs are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DrawnStroke = { clip?: [number, number]; band?: [number, number]; stroke: StrokeItem };
/// ```
#[derive(Clone)]
pub struct DrawnStroke {
    /// The x range this stroke is clipped to,
    ///  or nothing when it is drawn unclipped.
    pub clip: Option<(f32, f32)>,
    /// The y range this stroke is clipped to,
    ///  or nothing when it is drawn unclipped.
    pub band: Option<(f32, f32)>,
    /// The stroked path.
    pub stroke: StrokeItem,
}

/// What:
///  The glyph runs of `items` in drawing order,
///  each with the clip that was open when it was drawn.
/// Why:
///  Clip items nest only one deep in a tile,
///  so the last clip opened is the clip of every run until the next `Unclip`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function drawnRuns(items: Item[]): DrawnRun[];
/// ```
pub fn drawn_runs(items: &[Item]) -> Vec<DrawnRun> {
    let mut runs: Vec<DrawnRun> = Vec::new();
    let mut clip: Option<(f32, f32)> = None;
    let mut band: Option<(f32, f32)> = None;
    for item in items {
        match item {
            Item::Clip {
                left,
                right,
                top,
                bottom,
            } => {
                clip = Some((*left, *right));
                band = Some((*top, *bottom));
            }
            Item::Unclip => {
                clip = None;
                band = None;
            }
            Item::Glyphs(run) => {
                let mut ids: Vec<u32> = Vec::new();
                let mut left = f32::MAX;
                let mut right = f32::MIN;
                let mut baseline = run.origin.1;
                for glyph in &run.glyphs {
                    baseline = run.origin.1 + glyph.y;
                    ids.push(glyph.id);
                    left = left.min(glyph.x + run.origin.0);
                    right = right.max(glyph.x + run.origin.0);
                }
                runs.push(DrawnRun {
                    clip,
                    band,
                    ink: run.ink,
                    ids,
                    origin: run.origin,
                    left,
                    right,
                    baseline,
                });
            }
            Item::Stroke(_) | Item::Fill { .. } => {}
        }
    }
    return runs;
}

/// What:
///  The strokes of `items` in drawing order,
///  each with the clip that was open when it was drawn.
/// Why:
///  Underlines and the wrap arrow are strokes,
///  and a test reads their ink,
///  dash pattern,
///  and path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function drawnStrokes(items: Item[]): DrawnStroke[];
/// ```
pub fn drawn_strokes(items: &[Item]) -> Vec<DrawnStroke> {
    let mut strokes: Vec<DrawnStroke> = Vec::new();
    let mut clip: Option<(f32, f32)> = None;
    let mut band: Option<(f32, f32)> = None;
    for item in items {
        match item {
            Item::Clip {
                left,
                right,
                top,
                bottom,
            } => {
                clip = Some((*left, *right));
                band = Some((*top, *bottom));
            }
            Item::Unclip => {
                clip = None;
                band = None;
            }
            Item::Stroke(stroke) => strokes.push(DrawnStroke {
                clip,
                band,
                stroke: stroke.clone(),
            }),
            Item::Glyphs(_) | Item::Fill { .. } => {}
        }
    }
    return strokes;
}

/// What:
///  One filled rectangle as it is drawn:
///  its edges and its ink.
/// Why:
///  The minimap draws its selection and match marks as rectangles,
///  and a test reads their place and ink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DrawnFill = { left: number; top: number; right: number; bottom: number; inset: number; ink: Rgba };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawnFill {
    /// Left edge in physical pixels.
    pub left: f32,
    /// Top edge in physical pixels.
    pub top: f32,
    /// Right edge in physical pixels.
    pub right: f32,
    /// Bottom edge in physical pixels.
    pub bottom: f32,
    /// Thickness of the frame drawn,
    /// or zero for the whole rectangle.
    pub inset: f32,
    /// Straight RGBA ink.
    pub ink: [u8; 4],
}

/// The filled rectangles of `items` in drawing order.
pub fn drawn_fills(items: &[Item]) -> Vec<DrawnFill> {
    let mut fills: Vec<DrawnFill> = Vec::new();
    for item in items {
        if let Item::Fill {
            left,
            top,
            right,
            bottom,
            inset,
            ink,
        } = item
        {
            fills.push(DrawnFill {
                left: *left,
                top: *top,
                right: *right,
                bottom: *bottom,
                inset: *inset,
                ink: *ink,
            });
        }
    }
    return fills;
}

/// What:
///  The physical y of the middle of `stroke`'s bounding box.
/// Why:
///  A test places an underline on a visual row by its middle,
///  which a wave and a flat dash share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function centerY(stroke: StrokeItem): number;
/// ```
pub fn center_y(stroke: &StrokeItem) -> f32 {
    let bounds = stroke.path.bounding_box();
    return ((bounds.y0 + bounds.y1) / 2.0) as f32;
}
