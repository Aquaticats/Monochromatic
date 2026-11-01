//! Pixel-to-row scroll mapping for the Slint/libghostty-vt bridge.

/// What:
///      `pub const DEFAULT_CELL_WIDTH_PX: f32 = 9.0;` declares a public
///           32-bit floating-point constant.
///  Sibling numeric types include `f64`,
///           `u32`,
///  and `usize`;
///  `f32` matches Slint's `float`/`length` mapping.
/// Why:
///       Rust tests and non-UI callers need a fallback cell width;
///  the Slint
///           binary replaces this with renderer-measured font metrics at runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_CELL_WIDTH_PX = 9;
/// ```
pub const DEFAULT_CELL_WIDTH_PX: f32 = 9.0;

/// What:
///      `pub const DEFAULT_CELL_HEIGHT_PX: f32 = 18.0;` is the vertical cell
///           height in logical pixels.
///  `f32` is chosen for Slint interop;
///  `usize`
///           would be awkward because Slint passes fractional lengths.
/// Why:
///       This is the denominator for pixel-to-row scrolling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const DEFAULT_CELL_HEIGHT_PX = 18;
/// ```
pub const DEFAULT_CELL_HEIGHT_PX: f32 = 18.0;

// What:     `#[derive(Clone, Copy, Debug, PartialEq)]` asks Rust to generate
//           copying, debug printing, and equality for the struct below. `Copy`
//           is valid because every field is a plain number.
// Why:      Tests compare mappings, and callers pass them around by value.
//
// In TS you'd write (pseudocode):
// ```ts
// type ScrollMapping = { pixelScroll: number; wholeRowOffset: number; fractionalPx: number };
// ```
#[derive(Clone, Copy, Debug, PartialEq)]
/// What:
///      `pub struct ScrollMapping` declares a public record.
///  Siblings include
///           tuple structs for unnamed fields and enums for tagged unions.
/// Why:
///       The UI needs every part of the pixel-to-row split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScrollMapping = {
///   pixelScroll: number;
///   wholeRowOffset: number;
///   fractionalPx: number;
/// };
/// ```
pub struct ScrollMapping {
    /// What:
    ///      `pub pixel_scroll: f32` stores the clamped pixel offset.
    ///  `f32`
    ///           matches Slint;
    ///  `f64` would add casts without useful precision.
    /// Why:
    ///       The status text and tests can report the exact pixel position used.
    pub pixel_scroll: f32,
    /// What:
    ///      `pub whole_row_offset: usize` stores a non-negative row count.
    ///           Sibling integers include `u32`/`u64`/`i32`;
    ///  `usize` matches Rust
    ///           indexing and libghostty-vt scrollback row counts.
    /// Why:
    ///       This is the absolute row offset sent to libghostty-vt.
    pub whole_row_offset: usize,
    /// What:
    ///      `pub fractional_px: f32` stores the leftover pixels after taking
    ///           the whole-row floor.
    /// Why:
    ///       Slint keeps this fractional motion smooth between row updates.
    pub fractional_px: f32,
}

/// What:
///      `pub fn map_pixel_scroll(...) -> ScrollMapping` declares a public
///           pure function.
///  Parameters use `f32` for Slint pixels and `usize` for
///           row counts because those are the native caller types.
/// Why:
///       This is the required bridge:
///  floor(pixel / cell_height) plus modulo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function mapPixelScroll(pixelScroll, cellHeight, maxRowOffset) {
///   const clampedPixel = Math.min(Math.max(pixelScroll, 0), maxRowOffset * cellHeight);
///   const wholeRowOffset = Math.floor(clampedPixel / cellHeight);
///   return { pixelScroll: clampedPixel, wholeRowOffset, fractionalPx: clampedPixel % cellHeight };
/// }
/// ```
pub fn map_pixel_scroll(
    pixel_scroll: f32,
    cell_height_px: f32,
    max_row_offset: usize,
) -> ScrollMapping {
    // What:     `let safe_cell_height_px = ...` creates an immutable local. The
    //           `if` expression returns one of two `f32` values.
    // Why:      A zero or negative height would divide by zero, so one pixel is
    //           the safe fallback for bad resize input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const safeCellHeight = cellHeightPx > 0 ? cellHeightPx : 1;
    // ```
    let safe_cell_height_px = if cell_height_px > 0.0 {
        cell_height_px
    } else {
        1.0
    };

    // What:     `let max_pixel_scroll = max_row_offset as f32 * ...` casts the
    //           row count to `f32`. `usize` cannot multiply by `f32` directly.
    // Why:      Slint scrolls in pixels, but libghostty-vt clamps in rows.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const maxPixelScroll = maxRowOffset * safeCellHeight;
    // ```
    let max_pixel_scroll = max_row_offset as f32 * safe_cell_height_px;

    // What:     `.clamp(0.0, max_pixel_scroll)` bounds the pixel scroll between
    //           top and bottom. This method returns a new `f32`.
    // Why:      The UI can overshoot during resize or fling; the engine must not.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const clampedPixel = Math.min(Math.max(pixelScroll, 0), maxPixelScroll);
    // ```
    let clamped_pixel_scroll = pixel_scroll.clamp(0.0, max_pixel_scroll);

    // What:     `(clamped_pixel_scroll / safe_cell_height_px).floor() as usize`
    //           divides pixels by row height, floors to a whole row, and casts to
    //           Rust's index type.
    // Why:      libghostty-vt scrolls in rows, not fractional pixels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const wholeRowOffset = Math.floor(clampedPixel / safeCellHeight);
    // ```
    let whole_row_offset = (clamped_pixel_scroll / safe_cell_height_px).floor() as usize;

    // What:     `clamped_pixel_scroll - whole_row_offset as f32 * ...` computes
    //           the modulo without `%`, because `%` on floats is easy to misread.
    // Why:      This is the smooth sub-row translation Slint preserves.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fractionalPx = clampedPixel - wholeRowOffset * safeCellHeight;
    // ```
    let fractional_px = clamped_pixel_scroll - whole_row_offset as f32 * safe_cell_height_px;

    // What:     `ScrollMapping { ... }` constructs the record. No trailing `;`
    //           makes it the implicit return value.
    // Why:      Hand all mapping pieces to the engine and UI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { pixelScroll: clampedPixel, wholeRowOffset, fractionalPx };
    // ```
    return ScrollMapping {
        pixel_scroll: clamped_pixel_scroll,
        whole_row_offset,
        fractional_px,
    }
}

/// What:
///      `#[cfg(test)] #[path = "scroll_tests.rs"] mod tests;`
///           declares a test-only submodule whose code lives in the sibling
///           file `scroll_tests.rs`.
///  `#[cfg(test)]` gates it to test
///           builds only;
///  `#[path = "..."]` aims the module at a flat sibling
///           file instead of the default `scroll/tests.rs`
///           subdirectory lookup.
///  The file stays the `tests` CHILD of
///           scroll,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `scroll.rs` to production code;
///  the tests live
///           beside it without inflating this file or its max-lines budget
///           (sibling `*_tests.rs` files are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // scroll.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "scroll_tests.rs"]
mod tests;
