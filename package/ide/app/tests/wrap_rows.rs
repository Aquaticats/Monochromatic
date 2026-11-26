//! The vertical row map of wrapped lines,
//!  the frame that places those rows,
//!  the hits and ink on continuation rows,
//!  the gutter's wrap indicator,
//!  and the tiles painted from them.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this crate's modules live under `tests/wrap_rows/`.
//! Why:
//!  One test crate links the library once and shares the fixture row map,
//!       viewport and palette without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './wrap_rows/fixtures';
//! ```

#[path = "wrap_rows/fixtures.rs"]
mod fixtures;

/// The visual row map, its pixels and its pages.
#[path = "wrap_rows/row_map_and_pages.rs"]
mod row_map_and_pages;

/// How a frame places wrapped rows, and what a new width redoes.
#[path = "wrap_rows/frame_layout_and_new_width.rs"]
mod frame_layout_and_new_width;

/// Hits, carets, selection ink and underlines on continuation rows.
#[path = "wrap_rows/hits_and_ink.rs"]
mod hits_and_ink;

/// The gutter's wrap indicator and its glyph coverage.
#[path = "wrap_rows/wrap_signs.rs"]
mod wrap_signs;

/// The tiles painted from wrapped rows, and their clips.
#[path = "wrap_rows/tile_painting.rs"]
mod tile_painting;
