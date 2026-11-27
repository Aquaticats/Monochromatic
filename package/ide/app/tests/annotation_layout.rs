//! Annotation geometry from the production shaped rows:
//!  underline runs over tabs,
//!  CJK,
//!  combining marks,
//! and ligatures,
//!  multi-line and point ranges,
//!  overlap order,
//!  hints packed onto rows at the exact pixel x of
//! their positions,
//!  message rows at the x of their diagnostics,
//!  and code rows whose inner geometry never changes.

/// The shaper,
/// the records and the one block
///  every test of this file lays out.
#[path = "annotation_layout/fixtures.rs"]
mod fixtures;

/// The underline runs of a range:
///  its geometry over every script,
///  its rows,
///  and the order overlaps are drawn in.
#[path = "annotation_layout/underline_runs.rs"]
mod underline_runs;

/// Where a hint stands across the line,
///  and which row each hint is packed onto.
#[path = "annotation_layout/hint_rows.rs"]
mod hint_rows;

/// Message rows at the x of their diagnostic,
///  and the code row they leave unchanged.
#[path = "annotation_layout/message_rows.rs"]
mod message_rows;
