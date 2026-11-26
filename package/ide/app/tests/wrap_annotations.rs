//! Inlay hints,
//! diagnostic messages and their blocks inside the text area:
//!  hints packed against the line as it is broken,
//!  message rows broken to the width,
//!  a block above the first row,
//!  and the work a new width redoes.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this crate's modules live under `tests/wrap_annotations/`.
//! Why:
//!  One test crate links the library once and shares the fixture stamped store,
//!       hints and viewport without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './wrap_annotations/fixtures';
//! ```

#[path = "wrap_annotations/fixtures.rs"]
mod fixtures;

/// Hints packed against the line and the text area's edge.
#[path = "wrap_annotations/hint_rows.rs"]
mod hint_rows;

/// Diagnostic message rows broken to the text area.
#[path = "wrap_annotations/message_rows.rs"]
mod message_rows;

/// Where a block and its hints stand over the rows.
#[path = "wrap_annotations/block_and_hint_placement.rs"]
mod block_and_hint_placement;

/// The work a new width redoes,
/// and in which order.
#[path = "wrap_annotations/new_width.rs"]
mod new_width;
