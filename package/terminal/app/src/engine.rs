//! libghostty-vt terminal engine and render extraction.

/// What:
///      `use libghostty_vt::{...};` imports the safe Ghostty binding types.
///           `Terminal` owns VT state;
///  `RenderState` snapshots visible rows.
/// Why:
///       This module is the only place that talks to libghostty-vt directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Terminal, RenderState, TerminalOptions } from "libghostty-vt";
/// ```
use libghostty_vt::{RenderState, Terminal, TerminalOptions};

/// What:
///      `use libghostty_vt::render::{CellIterator, RowIterator};` imports
///           reusable iterator handles for snapshot rows and cells.
/// Why:
///       Render extraction walks rows and cells without using slow grid refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RowIterator, CellIterator } from "libghostty-vt/render";
/// ```
use libghostty_vt::render::{CellIterator, RowIterator};

/// What:
///      `use libghostty_vt::style::{RgbColor, Underline};` imports Ghostty
///           style helpers.
///  `RgbColor` is the upstream color record;
///  `Underline`
///           is the style enum.
/// Why:
///       The engine resolves inverse video and underline flags while copying cells.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { RgbColor, Underline } from "libghostty-vt/style";
/// ```
use libghostty_vt::style::{RgbColor, Underline};

/// What:
///      `use libghostty_vt::terminal::ScrollViewport;` imports the row-scroll
///           command enum.
///  Siblings are `Top`,
///  `Bottom`,
///  and `Delta` variants.
/// Why:
///       Absolute pixel scrolling is implemented by converting to row deltas.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ScrollViewport } from "libghostty-vt/terminal";
/// ```
use libghostty_vt::terminal::ScrollViewport;

/// What:
///      `use crate::...` imports sibling modules from this package.
/// Why:
///       Engine methods return crate models and crate errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { TerminalError, TerminalCell, TerminalSnapshot, mapPixelScroll } from "./deps";
/// ```
use crate::{
    error::TerminalError,
    render::{TerminalCell, TerminalSnapshot},
    scroll::map_pixel_scroll,
    scroll::ScrollMapping,
};

// What:     `#[derive(Clone, Copy, Debug, PartialEq)]` generates value behavior
//           for viewport geometry. `Copy` is fine because all fields are numbers.
// Why:      Resize code compares and stores this record by value.
//
// In TS you'd write (pseudocode):
// ```ts
// type ViewportGeometry = { cols: number; rows: number; cellWidthPx: number; cellHeightPx: number };
// ```
#[derive(Clone, Copy, Debug, PartialEq)]
/// What:
///      `pub struct ViewportGeometry` declares terminal dimensions in cells
///           plus the cell dimensions in logical pixels.
/// Why:
///       libghostty-vt resize needs both cell counts and pixel cell sizes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ViewportGeometry = { cols: number; rows: number; cellWidthPx: number; cellHeightPx: number };
/// ```
pub struct ViewportGeometry {
    /// What:
    ///      `pub cols: u16` stores terminal columns.
    ///  Sibling integers include
    ///           `usize` and `u32`;
    ///  libghostty-vt's API requires `u16`.
    /// Why:
    ///       Avoid casts at every resize call.
    pub cols: u16,
    /// What:
    ///      `pub rows: u16` stores terminal rows for libghostty-vt.
    /// Why:
    ///       Match Ghostty's API exactly.
    pub rows: u16,
    /// What:
    ///      `pub cell_width_px: f32` stores logical pixel cell width.
    /// Why:
    ///       Rust receives Slint lengths as `f32`.
    pub cell_width_px: f32,
    /// What:
    ///      `pub cell_height_px: f32` stores logical pixel cell height.
    /// Why:
    ///       This value drives both resize and scroll mapping.
    pub cell_height_px: f32,
}

/// What:
///      `impl ViewportGeometry` starts methods attached to the geometry type.
/// Why:
///       Construction from pixels is reused by the binary and tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ViewportGeometry = { fromPixels(...) { ... } };
/// ```
impl ViewportGeometry {
    /// What:
    ///      `pub fn from_pixels(...) -> Self` builds cell counts from Slint
    ///           viewport pixels and fixed cell metrics.
    /// Why:
    ///       Resize support lives in Rust,
    ///  not Slint integer math.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return { cols: Math.max(1, Math.floor(width / cellWidth)), rows: Math.max(1, Math.floor(height / cellHeight)), cellWidthPx: cellWidth, cellHeightPx: cellHeight };
    /// ```
    pub fn from_pixels(
        width_px: f32,
        height_px: f32,
        cell_width_px: f32,
        cell_height_px: f32,
    ) -> Self {
        // What:     `let safe_cell_width_px = ...` creates an immutable local from an
        //           `if` expression. Sibling `cell_width_px` is the raw caller value.
        // Why:      A transient zero measured font width would otherwise divide by zero.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const safeCellWidthPx = cellWidthPx > 0 ? cellWidthPx : 1;
        // ```
        let safe_cell_width_px = if cell_width_px > 0.0 {
            cell_width_px
        } else {
            1.0
        };
        // What:     `let safe_cell_height_px = ...` creates an immutable local from an
        //           `if` expression. Sibling `cell_height_px` is the raw caller value.
        // Why:      A transient zero cell height would otherwise divide by zero.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const safeCellHeightPx = cellHeightPx > 0 ? cellHeightPx : 1;
        // ```
        let safe_cell_height_px = if cell_height_px > 0.0 {
            cell_height_px
        } else {
            1.0
        };
        // What:     `.max(1.0).floor() as u16` clamps, floors, and narrows the
        //           column count to Ghostty's `u16` input type.
        // Why:      A window can never have a zero-column terminal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const cols = Math.max(1, Math.floor(widthPx / safeCellWidthPx));
        // ```
        let cols = (width_px / safe_cell_width_px).max(1.0).floor() as u16;
        // What:     `.max(1.0).floor() as u16` computes rows the same way.
        // Why:      A window can never have a zero-row terminal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rows = Math.max(1, Math.floor(heightPx / safeCellHeightPx));
        // ```
        let rows = (height_px / safe_cell_height_px).max(1.0).floor() as u16;
        // What:     `Self { ... }` constructs the geometry record. Tail expression
        //           means this is returned.
        // Why:      Callers need both computed grid and sanitized cell metrics.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { cols, rows, cellWidthPx: safeCellWidthPx, cellHeightPx: safeCellHeightPx };
        // ```
        return Self {
            cols,
            rows,
            cell_width_px: safe_cell_width_px,
            cell_height_px: safe_cell_height_px,
        }
    }
}

/// What:
///      `pub struct TerminalEngine` declares the stateful terminal wrapper.
///           The libghostty-vt handles are `!Send + !Sync`,
///  so this struct stays
///           on the Slint UI thread in the prototype.
/// Why:
///       One object owns VT state,
///  render iterators,
///  and viewport position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TerminalEngine { terminal; renderState; rowIterator; cellIterator; }
/// ```
pub struct TerminalEngine {
    /// What:
    ///      `terminal: Terminal<'static, 'static>` is the owned libghostty-vt
    ///           VT-state object:
    ///  the parser,
    ///  screen grid,
    ///  and scrollback store.
    ///  The
    ///           two `<'static, 'static>` are lifetime parameters;
    ///  `'static` means
    ///           "borrows nothing that lives shorter than the whole program",
    ///  i.e. it
    ///           owns or refers only to permanent data.
    ///  Siblings would be shorter,
    ///           named lifetimes like `<'a, 'b>` tied to some caller's stack frame.
    /// Why:
    ///       `'static` (not a shorter `<'a>`) lets this struct hold the terminal
    ///           for the whole UI session without a borrow expiring;
    ///  combined with the
    ///           type being `!Send + !Sync` (see the struct block above),
    ///  the engine
    ///           stays pinned to the Slint UI thread.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// terminal: Terminal;
    /// ```
    terminal: Terminal<'static, 'static>,
    /// What:
    ///      `render_state: RenderState<'static>` is the owned snapshot buffer that
    ///           libghostty-vt copies visible rows into each frame.
    ///  The one `<'static>`
    ///           is a lifetime parameter meaning "tied to nothing shorter-lived than the
    ///           program".
    ///  Sibling would be a shorter `<'a>` lifetime borrowing from a
    ///           local.
    /// Why:
    ///       `'static` (not a frame-bound `<'a>`) lets the engine keep and reuse one
    ///           render buffer across every frame instead of re-borrowing it per call.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// renderState: RenderState;
    /// ```
    render_state: RenderState<'static>,
    /// What:
    ///      `row_iterator: RowIterator<'static>` is an owned,
    ///  reusable cursor that
    ///           walks the rows of a render snapshot.
    ///  The `<'static>` lifetime parameter
    ///           means it is not bound to a shorter-lived borrow;
    ///  sibling would be a
    ///           frame-scoped `<'a>`.
    /// Why:
    ///       `'static` (not `<'a>`) lets the engine keep one row cursor alive for the
    ///           whole session and re-aim it at each new snapshot,
    ///  avoiding per-frame
    ///           allocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// rowIterator: RowIterator;
    /// ```
    row_iterator: RowIterator<'static>,
    /// What:
    ///      `cell_iterator: CellIterator<'static>` is an owned,
    ///  reusable cursor that
    ///           walks the cells within one row of a render snapshot.
    ///  The `<'static>`
    ///           lifetime parameter means it is not bound to a shorter-lived borrow;
    ///           sibling would be a frame-scoped `<'a>`.
    /// Why:
    ///       `'static` (not `<'a>`) lets the engine keep one cell cursor alive for the
    ///           whole session and re-aim it at each row,
    ///  avoiding per-frame allocation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// cellIterator: CellIterator;
    /// ```
    cell_iterator: CellIterator<'static>,
    /// What:
    ///      `viewport_top_row: usize` remembers Ghostty's current absolute top-row
    ///           offset into the scrollback.
    ///  `usize` is the unsigned integer wide enough
    ///           to address any element in memory (32 bits on a 32-bit OS,
    ///  64 on a
    ///           64-bit OS).
    ///  Siblings the reader might expect:
    ///  `u32`,
    ///  `u64`,
    ///  `i32`,
    ///  `i64`.
    /// Why:
    ///       `usize` (not `u32`/`u64`/`i32`/`i64`) because this value is used as a
    ///           row index and is compared against `scrollback_rows()`,
    ///  which returns
    ///           `usize`;
    ///  matching widths avoids casts everywhere.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// viewportTopRow: number;
    /// ```
    viewport_top_row: usize,
    /// What:
    ///      `geometry: ViewportGeometry` is the owned record of current grid size
    ///           (cols,
    ///  rows) plus cell pixel metrics,
    ///  declared earlier in this file.
    ///           It is a `Copy` value type,
    ///  not a reference;
    ///  sibling `&ViewportGeometry`
    ///           would only borrow one owned elsewhere.
    /// Why:
    ///       Owned (not `&ViewportGeometry`) because the engine outlives any caller
    ///           that produced the geometry,
    ///  and resize logic compares the stored value
    ///           against incoming ones to skip duplicate notifications.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// geometry: ViewportGeometry;
    /// ```
    geometry: ViewportGeometry,
}

/// What:
///      `impl TerminalEngine` starts methods for the stateful wrapper.
/// Why:
///       Keep libghostty-vt operations behind a small testable API.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TerminalEngine { static create(...) {} }
/// ```
impl TerminalEngine {
    /// What:
    ///      `pub fn new(...) -> Result<Self, TerminalError>` constructs a
    ///           terminal and render iterator handles.
    ///  `Result` is Rust's
    ///           success-or-error wrapper;
    ///  sibling `Option` has no error payload.
    /// Why:
    ///       libghostty-vt allocation can fail,
    ///  so construction is fallible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const terminal = new Terminal({ cols, rows, maxScrollback });
    /// return new TerminalEngine(terminal);
    /// ```
    pub fn new(
        geometry: ViewportGeometry,
        max_scrollback: usize,
    ) -> Result<Self, TerminalError> {
        // What:     `Terminal::new(TerminalOptions { ... })?` creates Ghostty's
        //           terminal. `?` returns early after converting errors.
        // Why:      This is the actual VT parser and scrollback store.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const terminal = new Terminal({ cols: geometry.cols, rows: geometry.rows, maxScrollback });
        // ```
        let mut terminal = Terminal::new(TerminalOptions {
            cols: geometry.cols,
            rows: geometry.rows,
            max_scrollback,
        })?;
        // What:     `terminal.resize(...)?` tells Ghostty the pixel cell size.
        // Why:      Size reports and image protocols depend on pixel dimensions.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // terminal.resize(geometry.cols, geometry.rows, geometry.cellWidthPx, geometry.cellHeightPx);
        // ```
        terminal.resize(
            geometry.cols,
            geometry.rows,
            geometry.cell_width_px as u32,
            geometry.cell_height_px as u32,
        )?;
        // What:     `Self { ... }` constructs the engine with reusable render handles.
        //           `RenderState::new()?` and iterator constructors are fallible.
        // Why:      Reusing handles avoids allocation churn on every frame.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { terminal, renderState: new RenderState(), rowIterator: new RowIterator(), cellIterator: new CellIterator() };
        // ```
        return Ok(Self {
            terminal,
            render_state: RenderState::new()?,
            row_iterator: RowIterator::new()?,
            cell_iterator: CellIterator::new()?,
            viewport_top_row: 0,
            geometry,
        })
    }

    /// What:
    ///      `pub fn feed(&mut self, bytes: &[u8]) -> Result<(), TerminalError>`
    ///           takes a mutable engine and a borrowed byte slice,
    ///  then returns a
    ///           fallible result.
    ///  `&[u8]` is like a read-only Uint8Array view.
    /// Why:
    ///       PTY I/O or demo data can push raw VT bytes through Ghostty,
    ///  and the
    ///           engine must refresh its remembered bottom-row offset afterward.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.terminal.vtWrite(bytes);
    /// this.viewportTopRow = this.terminal.scrollbackRows();
    /// ```
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), TerminalError> {
        // What:     `self.terminal.vt_write(bytes)` parses untrusted VT bytes.
        // Why:      This updates screen, cursor, style, and scrollback state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.terminal.vtWrite(bytes);
        // ```
        self.terminal.vt_write(bytes);
        // What:     `self.terminal.scroll_viewport(ScrollViewport::Bottom)` asks
        //           Ghostty to show the active area after new content.
        // Why:      Demo startup should land at the terminal bottom like a real app.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.terminal.scrollViewport("bottom");
        // ```
        self.terminal.scroll_viewport(ScrollViewport::Bottom);
        // What:     `self.viewport_top_row = self.terminal.scrollback_rows()?` reads
        //           Ghostty's current bottom offset and stores it. `?` propagates
        //           libghostty-vt errors through `TerminalError`.
        // Why:      Later absolute pixel-scroll requests need a correct relative
        //           starting point for `ScrollViewport::Delta`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.viewportTopRow = this.terminal.scrollbackRows();
        // ```
        self.viewport_top_row = self.terminal.scrollback_rows()?;
        // What:     `Ok(())` constructs a successful `Result` carrying unit.
        // Why:      Feeding and viewport bookkeeping completed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return;
        // ```
        return Ok(())
    }

    /// What:
    ///      `pub fn resize(&mut self, geometry: ViewportGeometry) -> Result...`
    ///           mutates Ghostty's grid dimensions.
    /// Why:
    ///       The Slint window can be resized after startup.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// this.terminal.resize(geometry.cols, geometry.rows, geometry.cellWidthPx, geometry.cellHeightPx);
    /// ```
    pub fn resize(&mut self, geometry: ViewportGeometry) -> Result<(), TerminalError> {
        // What:     `if self.geometry == geometry { return Ok(()); }` exits early
        //           when no cell count or cell metric changed.
        // Why:      Slint may emit duplicate resize notifications.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (sameGeometry(this.geometry, geometry)) return;
        // ```
        if self.geometry == geometry {
            // What:     `Ok(())` constructs a successful `Result` carrying unit.
            // Why:      Nothing changed, but the operation succeeded.
            return Ok(());
        }
        // What:     `self.terminal.resize(...)?` forwards the new grid and pixel
        //           metrics to Ghostty.
        // Why:      Ghostty reflows primary-screen content on resize.
        self.terminal.resize(
            geometry.cols,
            geometry.rows,
            geometry.cell_width_px as u32,
            geometry.cell_height_px as u32,
        )?;
        // What:     `self.geometry = geometry` stores the new record.
        // Why:      Future duplicate notifications can be skipped.
        self.geometry = geometry;
        // What:     `let max_top_row = self.terminal.scrollback_rows()?` queries the
        //           new maximum viewport top after reflow.
        // Why:      Resize can reduce available scrollback offset.
        let max_top_row = self.terminal.scrollback_rows()?;
        // What:     `if self.viewport_top_row > max_top_row { ... }` clamps the
        //           remembered absolute top row.
        // Why:      The next scroll delta must be relative to Ghostty's real viewport.
        if self.viewport_top_row > max_top_row {
            self.set_viewport_top_row(max_top_row)?;
        }
        // What:     `Ok(())` returns success after the resize.
        // Why:      Match the method's `Result` return type.
        return Ok(())
    }

    /// What:
    ///      `pub fn scrollback_rows(&self) -> Result<usize, TerminalError>`
    ///           borrows the engine immutably and returns the scrollback count.
    /// Why:
    ///       The binary uses this to initialize Flickable at the bottom.
    pub fn scrollback_rows(&self) -> Result<usize, TerminalError> {
        // What:     `self.terminal.scrollback_rows()?` reads Ghostty's count and
        //           propagates errors.
        // Why:      Ghostty is the source of truth for scrollback size.
        let rows = self.terminal.scrollback_rows()?;
        // What:     `Ok(rows)` wraps the count in a successful `Result`.
        // Why:      The method promised `Result` for error propagation.
        return Ok(rows)
    }

    /// What:
    ///      `pub fn set_pixel_scroll(...) -> Result<ScrollMapping, ...>` maps
    ///           Slint pixels to Ghostty rows and moves the Ghostty viewport.
    /// Why:
    ///       This is the central smooth-scroll bridge.
    pub fn set_pixel_scroll(
        &mut self,
        pixel_scroll: f32,
    ) -> Result<ScrollMapping, TerminalError> {
        // What:     `let max_top_row = ...` reads the current bottom row offset.
        // Why:      Pixel scrolling must clamp to actual scrollback length.
        let max_top_row = self.terminal.scrollback_rows()?;
        // What:     `map_pixel_scroll(...)` performs the pure floor/modulo split.
        // Why:      Keep math tested separately from Ghostty side effects.
        let mapping = map_pixel_scroll(pixel_scroll, self.geometry.cell_height_px, max_top_row);
        // What:     `self.set_viewport_top_row(...)?` moves Ghostty to the whole-row
        //           part of the Slint pixel offset.
        // Why:      RenderState only exposes row-aligned viewports.
        self.set_viewport_top_row(mapping.whole_row_offset)?;
        // What:     `Ok(mapping)` returns the same mapping to the caller.
        // Why:      Slint status can show fractional pixels and row offset.
        return Ok(mapping)
    }

    /// What:
    ///      `pub fn snapshot(...) -> Result<TerminalSnapshot, ...>` extracts a
    ///           renderer-neutral frame from Ghostty's current row viewport.
    /// Why:
    ///       The binary should not know libghostty-vt iterator APIs.
    pub fn snapshot(
        &mut self,
        mapping: ScrollMapping,
    ) -> Result<TerminalSnapshot, TerminalError> {
        // What:     `let snapshot = self.render_state.update(&self.terminal)?` asks
        //           Ghostty to copy renderable state into `RenderState`.
        // Why:      Rows and cells are read from the render snapshot, not raw screen refs.
        let snapshot = self.render_state.update(&self.terminal)?;
        // What:     `let colors = snapshot.colors()?` reads resolved default colors.
        // Why:      Empty foreground/background values fall back to these.
        let colors = snapshot.colors()?;
        // What:     `let viewport_cols = usize::from(snapshot.cols()?)` converts
        //           Ghostty's `u16` width to Rust's index-sized integer.
        // Why:      Snapshot models and status text use `usize`.
        let viewport_cols = usize::from(snapshot.cols()?);
        // What:     `let viewport_rows = usize::from(snapshot.rows()?)` does the
        //           same conversion for height.
        // Why:      Row indexes are `usize` in this crate.
        let viewport_rows = usize::from(snapshot.rows()?);
        // What:     `let cells = { ... }` starts a scoped block that returns the
        //           owned cell vector at the end. The block is not a closure; it
        //           just limits how long row-iterator borrows can live.
        // Why:      Clippy rejects manual `drop` for non-`Drop` iterator guards, so
        //           lexical scope is the correct way to end their borrows.
        let cells = {
            // What:     `let mut cells = Vec::new()` creates an owned growable array.
            // Why:      We push cells while walking the render iterators.
            let mut cells = Vec::new();
            // What:     `let mut row_iteration = self.row_iterator.update(&snapshot)?`
            //           attaches the reusable row iterator to this snapshot.
            // Why:      Rows are valid only while this snapshot borrow is alive.
            let mut row_iteration = self.row_iterator.update(&snapshot)?;
            // What:     `let mut row_index = 0usize` creates a mutable row counter.
            //           `usize` matches vector indexing; `u32` would need casts.
            // Why:      The iterator yields row data but not its numeric index.
            let mut row_index = 0usize;
            // What:     `while let Some(row) = row_iteration.next()` loops over rows
            //           from the lending iterator until it returns `None`.
            // Why:      libghostty-vt exposes rows through this stateful cursor API.
            while let Some(row) = row_iteration.next() {
                // What:     `let mut cell_iteration = self.cell_iterator.update(row)?`
                //           attaches the cell iterator to the current row.
                // Why:      Cell data is row-scoped in the render-state API.
                let mut cell_iteration = self.cell_iterator.update(row)?;
                // What:     `let mut col_index = 0usize` tracks the current column.
                // Why:      The cell iterator yields cells in left-to-right order.
                let mut col_index = 0usize;
                // What:     `while let Some(cell) = cell_iteration.next()` loops over
                //           cells until the row cursor is exhausted.
                // Why:      This copies every visible/styled cell into an owned model.
                while let Some(cell) = cell_iteration.next() {
                    // What:     `let text: String = cell.graphemes()?.into_iter().collect()`
                    //           copies Ghostty grapheme codepoints into owned UTF-8 text.
                    // Why:      The Slint model cannot borrow from Ghostty iterators.
                    let text: String = cell.graphemes()?.into_iter().collect();
                    // What:     `let style = cell.style()?` copies SGR flags for the cell.
                    // Why:      Bold, italic, inverse, and underline are UI-visible styles.
                    let style = cell.style()?;
                    // What:     `let raw_foreground = cell.fg_color()?.unwrap_or(...)`
                    //           uses Ghostty's resolved color or the default foreground.
                    // Why:      Slint receives concrete RGB, never palette indexes.
                    let raw_foreground = cell.fg_color()?.unwrap_or(colors.foreground);
                    // What:     `let raw_background_option = cell.bg_color()?` reads the
                    //           optional resolved background color.
                    // Why:      Empty cells with no custom background can be skipped.
                    let raw_background_option = cell.bg_color()?;
                    // What:     `let (foreground, background) = resolve_inverse(...)`
                    //           returns possibly swapped colors for inverse video.
                    // Why:      The UI can draw inverse cells without extra style logic.
                    let (foreground, background) = resolve_inverse(
                        raw_foreground,
                        raw_background_option.unwrap_or(colors.background),
                        style.inverse,
                    );
                    // What:     `if should_copy_cell(...) { ... }` skips plain empty cells.
                    // Why:      The Slint model stays smaller while preserving styled blanks.
                    if should_copy_cell(text.as_str(), raw_background_option, style.inverse) {
                        // What:     `cells.push(TerminalCell { ... })` appends one owned
                        //           cell model to the growable vector.
                        // Why:      This cell has visible text or visible styling.
                        cells.push(TerminalCell {
                            row: row_index,
                            col: col_index,
                            text,
                            foreground: foreground.into(),
                            background: background.into(),
                            bold: style.bold,
                            italic: style.italic,
                            inverse: style.inverse,
                            underline: style.underline != Underline::None,
                        });
                    }
                    // What:     `col_index += 1` advances the mutable column counter.
                    // Why:      The next cell is one grid column to the right.
                    col_index += 1;
                }
                // What:     `row.set_dirty(false)?` clears Ghostty's per-row dirty flag.
                // Why:      A real renderer would use dirty tracking; this prototype keeps
                //           the state tidy after copying the row.
                row.set_dirty(false)?;
                // What:     `row_index += 1` advances the row counter.
                // Why:      The next render row is one grid row lower.
                row_index += 1;
            }
            // What:     `cells` without a trailing semicolon is the block's return
            //           value. This moves the owned vector out of the scoped block.
            // Why:      Returning here ends the row and cell iterator borrows before
            //           the terminal metadata reads below.
            cells
        };
        // What:     `let total_rows = self.terminal.total_rows()?` reads total active
        //           screen rows including scrollback from Ghostty.
        // Why:      Slint sizes the Flickable content from this count.
        let total_rows = self.terminal.total_rows()?;
        // What:     `let scrollback_rows = self.terminal.scrollback_rows()?` reads rows
        //           above the active viewport.
        // Why:      Slint uses this as the maximum whole-row scroll.
        let scrollback_rows = self.terminal.scrollback_rows()?;
        // What:     `let title = self.terminal.title()?.to_string()` copies the
        //           borrowed Ghostty title into an owned `String`.
        // Why:      The borrowed title expires on the next VT write or reset.
        let title = self.terminal.title()?.to_string();
        // What:     `Ok(TerminalSnapshot { ... })` constructs and returns the frame.
        // Why:      The binary receives one owned value with all UI data.
        return Ok(TerminalSnapshot {
            cells,
            viewport_rows,
            viewport_cols,
            total_rows,
            scrollback_rows,
            whole_row_offset: mapping.whole_row_offset,
            fractional_px: mapping.fractional_px,
            cell_width_px: self.geometry.cell_width_px,
            cell_height_px: self.geometry.cell_height_px,
            title,
        })
    }

    /// What:
    ///      `fn set_viewport_top_row(...) -> Result<(), TerminalError>` is a
    ///           private helper that converts absolute row offsets to Ghostty calls.
    /// Why:
    ///       Ghostty supports Top,
    ///  Bottom,
    ///  or Delta,
    ///  not direct absolute set.
    fn set_viewport_top_row(&mut self, target_top_row: usize) -> Result<(), TerminalError> {
        // What:     `let max_top_row = ...` fetches the bottom offset.
        // Why:      Absolute row requests must clamp to current scrollback size.
        let max_top_row = self.terminal.scrollback_rows()?;
        // What:     `.min(max_top_row)` clamps the target top row.
        // Why:      The UI may hold a stale pixel scroll after resize.
        let target = target_top_row.min(max_top_row);
        // What:     `if target == self.viewport_top_row { return Ok(()); }` skips
        //           redundant Ghostty scroll calls.
        // Why:      Fractional-only Slint movement should not touch row state.
        if target == self.viewport_top_row {
            return Ok(());
        }
        // What:     `if target == 0 { ... } else if target == max_top_row { ... }`
        //           uses Ghostty's exact top/bottom commands when possible.
        // Why:      Top and bottom remain correct even if our remembered row drifted.
        if target == 0 {
            self.terminal.scroll_viewport(ScrollViewport::Top);
        } else if target == max_top_row {
            self.terminal.scroll_viewport(ScrollViewport::Bottom);
        } else {
            // What:     `target as isize - self.viewport_top_row as isize` computes a
            //           signed row delta. Positive means down toward active area.
            // Why:      Ghostty's Delta variant is relative to current viewport.
            let delta = target as isize - self.viewport_top_row as isize;
            self.terminal.scroll_viewport(ScrollViewport::Delta(delta));
        }
        // What:     `self.viewport_top_row = target` updates remembered state.
        // Why:      The next absolute request can become a relative delta.
        self.viewport_top_row = target;
        // What:     `Ok(())` returns success.
        // Why:      The helper has completed the row scroll.
        return Ok(())
    }
}

/// What:
///      `fn resolve_inverse(...) -> (RgbColor, RgbColor)` returns a tuple of
///           Ghostty colors.
///  Tuples are fixed-size ordered records;
///  sibling named
///           structs would be more verbose here.
/// Why:
///       Inverse video swaps foreground and background before UI conversion.
fn resolve_inverse(
    foreground: RgbColor,
    background: RgbColor,
    inverse: bool,
) -> (RgbColor, RgbColor) {
    // What:     `if inverse { ... } else { ... }` returns one tuple or the other.
    // Why:      SGR inverse means foreground and background trade places.
    if inverse {
        return (background, foreground)
    } else {
        return (foreground, background)
    }
}

/// What:
///      `fn should_copy_cell(...) -> bool` is a private visibility filter.
/// Why:
///       Plain empty cells are represented by the content background,
///  not models.
fn should_copy_cell(
    text: &str,
    background: Option<RgbColor>,
    inverse: bool,
) -> bool {
    // What:     `!text.is_empty() || background.is_some() || inverse` combines three
    //           visibility reasons. `Option::is_some` checks for a present value.
    // Why:      Text, explicit backgrounds, and inverse blanks all need drawing.
    return !text.is_empty() || background.is_some() || inverse
}

/// What:
///      `#[cfg(test)] #[path = "engine_tests.rs"] mod tests;`
///           declares a test-only submodule whose code lives in the sibling
///           file `engine_tests.rs`.
///  `#[cfg(test)]` gates it to test
///           builds only;
///  `#[path = "..."]` aims the module at a flat sibling
///           file instead of the default `engine/tests.rs`
///           subdirectory lookup.
///  The file stays the `tests` CHILD of
///           engine,
///  so its `use super::*` reaches the module items
///           (including private ones) unchanged.
/// Why:
///       Keep `engine.rs` to production code;
///  the tests live
///           beside it without inflating this file or its max-lines budget
///           (sibling `*_tests.rs` files are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // engine.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
