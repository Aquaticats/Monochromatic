//! The line style of each diagnostic severity as strokes:
//!  errors get a wavy line,
//!  warnings a dashed line,
//!  information a dotted line,
//!  and hint-severity problems sparse dots,
//!  so severity is never shown by color alone.

/// The stroke item and the item list it joins.
use super::{Item, StrokeItem};
/// Repeat length,
/// wave height,
/// and thickness of the line styles.
use crate::annotation_paint::{AMPLITUDE, PERIOD, THICKNESS};
/// Severities as the Language module names them.
use crate::language::diagnostics::Severity;
/// The path of a stroke.
use vello::kurbo::BezPath;

/// What:
///  How much thicker than a pixel row the wave is drawn,
/// as a factor.
/// Why:
///  A wave over six pixels climbs and falls two thirds of a row per column,
///  so anti-aliasing spreads each column's ink over two rows;
///  at a factor of one the crests measured 0.67 and 0.84 of the ink and the slopes less,
///  at 1.3 they measure 0.80 and 0.95.
const WAVE_WEIGHT: f32 = 1.3;

/// What:
///  Where one underline run lies and how it is drawn,
///  in physical pixels from the tile's top left corner.
/// Why:
///  The geometry is the same for every ink the run is drawn in,
///  so it is computed once per run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Geometry = { span: [number, number]; line: number; scale: number; severity: Severity };
/// ```
pub struct Geometry {
    /// Left and right edge of the run.
    pub span: (f32, f32),
    /// Center line of the run,
    ///  below the common baseline.
    pub line: f32,
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// Severity,
    ///  which selects the line style.
    pub severity: Severity,
}

/// What:
///  The on and off lengths of the dashes of `severity` in physical pixels,
///  or nothing for the continuous wave.
/// Why:
///  Four pixels drawn and two left out for warnings,
///  two-pixel dots as long as their gaps for information,
///  and one two-pixel dot per period for hint-severity problems.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dashes(severity: Severity, scale: number): number[] | undefined;
/// ```
fn dashes(severity: Severity, scale: f32) -> Option<Vec<f64>> {
    let unit = f64::from(scale);
    let period = f64::from(PERIOD);
    if severity == Severity::Error {
        return None;
    }
    if severity == Severity::Warning {
        return Some(vec![4.0 * unit, (period - 4.0) * unit]);
    }
    if severity == Severity::Information {
        return Some(vec![2.0 * unit, 2.0 * unit]);
    }
    return Some(vec![2.0 * unit, (period - 2.0) * unit]);
}

/// What:
///  The center line of a wave over `span`:
///  a triangle wave of the style's period and `swing`,
///  starting at the top of its swing at the run's left edge.
/// Why:
///  The wave is cut at the run's right edge,
///  so the last segment ends where the run does.
///  The swing is a whole number of pixels around the middle of a pixel row,
///  so the crests and troughs fill whole rows;
///  a swing of one and a half pixels around a row boundary measured at half the ink of the CPU wave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function wave(span: [number, number], middle: number, swingPixels: number, scale: number): Path;
/// ```
fn wave(span: (f32, f32), middle: f32, swing_pixels: f32, scale: f32) -> BezPath {
    let half = f64::from(PERIOD * scale) / 2.0;
    let swing = f64::from(swing_pixels);
    let (left, right) = (f64::from(span.0), f64::from(span.1));
    let center = f64::from(middle);
    let mut path = BezPath::new();
    path.move_to((left, center - swing));
    let mut index = 1_u32;
    loop {
        let x = left + f64::from(index) * half;
        let (previous, next) = if index % 2 == 0 {
            (center + swing, center - swing)
        } else {
            (center - swing, center + swing)
        };
        if x >= right {
            // The run ends inside this segment: cut it where the run ends.
            let share = (right - (x - half)) / half;
            path.line_to((right, previous + (next - previous) * share));
            break;
        }
        path.line_to((x, next));
        index += 1;
    }
    return path;
}

/// What:
///  Add the stroke of one underline run in `ink` to `items`.
/// Why:
///  Whole pixel rows keep a flat line in its full ink;
///  a line split over two rows reads as a paler color.
///  A wave is thinner across its slope,
/// so that its vertical thickness is the same as a flat line's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function underlineItems(items: Item[], geometry: Geometry, ink: Rgba): void;
/// ```
pub fn underline_items(items: &mut Vec<Item>, geometry: &Geometry, ink: [u8; 4]) {
    let (left, right) = geometry.span;
    if right <= left {
        return;
    }
    let scale = geometry.scale;
    let thickness = (THICKNESS * scale).round().max(1.0);
    // A flat line covers whole pixel rows.
    let top = (geometry.line - thickness / 2.0).round();
    let middle = top + thickness / 2.0;
    if geometry.severity == Severity::Error {
        let swing = (AMPLITUDE * scale).floor().max(1.0);
        // The slope of the wave's segments: a rise of two swings over half a period.
        let slope = 2.0 * swing / (PERIOD * scale / 2.0);
        // A little more than a row thick, because the anti-aliased edges of a thin slope are paler than a row.
        let vertical = thickness * WAVE_WEIGHT;
        items.push(Item::Stroke(StrokeItem {
            path: wave(geometry.span, middle, swing, scale),
            width: vertical / (1.0 + slope * slope).sqrt(),
            dash: Vec::new(),
            round: false,
            ink,
        }));
        return;
    }
    let center = f64::from(middle);
    let mut path = BezPath::new();
    path.move_to((f64::from(left), center));
    path.line_to((f64::from(right), center));
    items.push(Item::Stroke(StrokeItem {
        path,
        width: thickness,
        dash: dashes(geometry.severity, scale).unwrap_or_default(),
        round: false,
        ink,
    }));
}
