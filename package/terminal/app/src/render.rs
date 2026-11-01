//! Renderer-neutral models extracted from libghostty-vt render state.

/// What:
///      `use libghostty_vt::style::RgbColor;` imports Ghostty's RGB type.
///           The sibling type in this file is `Rgb`,
///  our UI-neutral copy.
/// Why:
///       Conversion keeps libghostty-vt types at the engine boundary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { RgbColor } from "libghostty-vt/style";
/// ```
use libghostty_vt::style::RgbColor;

// What:     `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]` generates
//           common value-object behavior for an RGB record. `Copy` is valid
//           because all fields are bytes.
// Why:      Cells copy colors frequently while building snapshots.
//
// In TS you'd write (pseudocode):
// ```ts
// type Rgb = { red: number; green: number; blue: number };
// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// What:
///      `pub struct Rgb` declares a public fixed record.
///  Sibling shapes are
///           tuple structs and enums;
///  named fields are clearer for channels.
/// Why:
///       Slint conversion needs explicit channel names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Rgb = { red: number; green: number; blue: number };
/// ```
pub struct Rgb {
    /// What:
    ///      `pub red: u8` stores one 0 to 255 channel.
    ///  Sibling integer types
    ///           include `u16` and `usize`;
    ///  `u8` matches libghostty-vt and color APIs.
    /// Why:
    ///       Avoid widening colors until Slint conversion.
    pub red: u8,
    /// What:
    ///      `pub green: u8` stores one color channel.
    /// Why:
    ///       Keep the RGB record complete.
    pub green: u8,
    /// What:
    ///      `pub blue: u8` stores one color channel.
    /// Why:
    ///       Keep the RGB record complete.
    pub blue: u8,
}

/// What:
///      `impl From<RgbColor> for Rgb` defines a lossless conversion from
///           Ghostty's color type to this crate's color type.
/// Why:
///       Engine code can call `.into()` when copying resolved colors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fromGhosttyRgb(color: RgbColor): Rgb {
///   return { red: color.r, green: color.g, blue: color.b };
/// }
/// ```
impl From<RgbColor> for Rgb {
    /// What:
    ///      `fn from(color: RgbColor) -> Self` consumes Ghostty's RGB value and
    ///           returns this crate's RGB value.
    /// Why:
    ///       The channel bytes are identical,
    ///  only field names change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return { red: color.r, green: color.g, blue: color.b };
    /// ```
    fn from(color: RgbColor) -> Self {
        // What:     `Self { ... }` constructs an `Rgb`. No trailing semicolon makes
        //           it the implicit return.
        // Why:      Copy each libghostty-vt channel into stable field names.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { red: color.r, green: color.g, blue: color.b };
        // ```
        return Self {
            red: color.r,
            green: color.g,
            blue: color.b,
        }
    }
}

// What:     `#[derive(Clone, Debug, PartialEq, Eq)]` generates copying via clone,
//           debug output, and equality for the cell model. `Copy` is not derived
//           because `String` owns heap data.
// Why:      Slint conversion clones cells into a model, and tests compare cells.
//
// In TS you'd write (pseudocode):
// ```ts
// type TerminalCell = { row: number; col: number; text: string; ... };
// ```
#[derive(Clone, Debug, PartialEq, Eq)]
/// What:
///      `pub struct TerminalCell` declares one visible cell in viewport row
///           coordinates.
///  Sibling rows are implicit;
///  Slint receives a flat list.
/// Why:
///       A flat model is easy for Slint to draw with positioned rectangles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TerminalCell = {
///   row: number;
///   col: number;
///   text: string;
///   foreground: Rgb;
///   background: Rgb;
/// };
/// ```
pub struct TerminalCell {
    /// What:
    ///      `pub row: usize` stores a viewport-relative row index.
    ///  `usize`
    ///           matches Rust vector indexing;
    ///  `u32` would need casts.
    /// Why:
    ///       Slint positions the cell vertically from this row.
    pub row: usize,
    /// What:
    ///      `pub col: usize` stores a viewport-relative column index.
    /// Why:
    ///       Slint positions the cell horizontally from this column.
    pub col: usize,
    /// What:
    ///      `pub text: String` stores owned UTF-8.
    ///  Sibling `&str` borrows text;
    ///           owned `String` lets the snapshot outlive libghostty-vt iterators.
    /// Why:
    ///       Render iterators become invalid after update,
    ///  so text must be copied.
    pub text: String,
    /// What:
    ///      `pub foreground: Rgb` stores the resolved foreground color.
    /// Why:
    ///       Slint should not resolve palette indexes itself.
    pub foreground: Rgb,
    /// What:
    ///      `pub background: Rgb` stores the resolved background color.
    /// Why:
    ///       Slint can draw cell rectangles directly.
    pub background: Rgb,
    /// What:
    ///      `pub bold: bool` stores SGR bold state.
    /// Why:
    ///       The UI can draw a second shifted glyph for prototype bold.
    pub bold: bool,
    /// What:
    ///      `pub italic: bool` stores SGR italic state.
    /// Why:
    ///       The model exposes style even if the prototype UI does not use it yet.
    pub italic: bool,
    /// What:
    ///      `pub inverse: bool` stores whether inverse video was active.
    /// Why:
    ///       Tests and future renderers can see that the style existed.
    pub inverse: bool,
    /// What:
    ///      `pub underline: bool` stores whether any underline style was active.
    /// Why:
    ///       The UI draws a simple underline for all underline variants.
    pub underline: bool,
}

// What:     `#[derive(Clone, Debug)]` generates clone and debug output for the
//           snapshot. `PartialEq` is omitted because large cell lists are usually
//           inspected through targeted test assertions.
// Why:      The binary owns snapshots briefly while converting them to Slint.
//
// In TS you'd write (pseudocode):
// ```ts
// type TerminalSnapshot = { cells: TerminalCell[]; viewportRows: number; ... };
// ```
#[derive(Clone, Debug)]
/// What:
///      `pub struct TerminalSnapshot` declares all UI data for one frame.
/// Why:
///       One value crosses from engine extraction into Slint conversion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TerminalSnapshot = { cells: TerminalCell[]; title: string };
/// ```
pub struct TerminalSnapshot {
    /// What:
    ///      `pub cells: Vec<TerminalCell>` stores owned cells.
    ///  `Vec<T>` is a
    ///           growable array;
    ///  siblings are fixed arrays and borrowed slices.
    /// Why:
    ///       The number of non-empty/styled cells changes per frame.
    pub cells: Vec<TerminalCell>,
    /// What:
    ///      `pub viewport_rows: usize` stores visible terminal rows.
    /// Why:
    ///       Status text and tests need the render-state viewport height.
    pub viewport_rows: usize,
    /// What:
    ///      `pub viewport_cols: usize` stores visible terminal columns.
    /// Why:
    ///       Status text and resize verification need the width.
    pub viewport_cols: usize,
    /// What:
    ///      `pub total_rows: usize` stores active screen rows plus scrollback.
    /// Why:
    ///       Slint uses it to size the Flickable content.
    pub total_rows: usize,
    /// What:
    ///      `pub scrollback_rows: usize` stores rows above the active viewport.
    /// Why:
    ///       Slint clamps bottom scroll to this many whole rows.
    pub scrollback_rows: usize,
    /// What:
    ///      `pub whole_row_offset: usize` stores the viewport's absolute top row.
    /// Why:
    ///       Slint positions visible cells at absolute content rows.
    pub whole_row_offset: usize,
    /// What:
    ///      `pub fractional_px: f32` stores sub-row scroll remainder.
    /// Why:
    ///       Status text shows the smooth scrolling bridge explicitly.
    pub fractional_px: f32,
    /// What:
    ///      `pub cell_width_px: f32` stores logical pixel cell width.
    /// Why:
    ///       Slint content width and Rust resize math stay in sync.
    pub cell_width_px: f32,
    /// What:
    ///      `pub cell_height_px: f32` stores logical pixel cell height.
    /// Why:
    ///       Slint content height and Rust scroll math stay in sync.
    pub cell_height_px: f32,
    /// What:
    ///      `pub title: String` stores the terminal title copied from Ghostty.
    ///           `String` owns its bytes;
    ///  `&str` would borrow from the terminal.
    /// Why:
    ///       The UI can show OSC title state without lifetime coupling.
    pub title: String,
}
