//! The measures of diagnostic underlines:
//!  each severity has its own line style,
//!  so severity is never shown by color alone.
//!
//! Errors get a wavy line,
//!  warnings a dashed line,
//!  information a dotted line,
//!  and hint-severity problems
//! sparse dots.
//!  The GPU code view and the minimap draw them as stroked paths (`tile_scene/underline.rs`)
//!  below the common baseline,
//!  from the measures here.

/// What:
///  Repeat length of the wave,
///  dash,
///  and sparse-dot patterns in logical pixels;
///  `f32` is a 32-bit float
///       (sibling `f64`).
/// Why:
///  Six pixels keep a wave visibly wavy and a dash visibly broken at 15 px source text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const PERIOD = 6;
/// ```
pub const PERIOD: f32 = 6.0;
/// Height of the wave above and below its center line,
///  in logical pixels.
pub const AMPLITUDE: f32 = 1.5;
/// Thickness of every underline style in logical pixels,
///  rounded to whole physical rows and at least one.
pub const THICKNESS: f32 = 1.25;
/// Distance of the underline's center line below the common source baseline,
///  in logical pixels.
pub const DROP: f32 = 3.0;
