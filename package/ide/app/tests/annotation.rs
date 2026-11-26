//! Which hints and diagnostics a frame paints:
//!  stale snapshots paint nothing,
//!  hints are grouped by line and
//! accumulated per revision,
//!  underline windows are bounded to the materialized rows,
//!  multi-line ranges above a
//! window are found,
//!  problems at the caret are listed worst first,
//!  and space is held across a reload.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this crate's modules live under `tests/annotation/`.
//! Why:
//!  One test crate links the library once and shares the fixture documents,
//!       stamps and snapshots without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './annotation/fixtures';
//! ```

#[path = "annotation/fixtures.rs"]
mod fixtures;

/// Which snapshot a frame paints, recognizes or accepts.
#[path = "annotation/snapshot_identity.rs"]
mod snapshot_identity;

/// How a hint answer becomes rows.
#[path = "annotation/hint_rows.rs"]
mod hint_rows;

/// Space held for hints that left or have not arrived.
#[path = "annotation/held_space.rs"]
mod held_space;

/// Which diagnostic ranges a window shows, and what a problem spells out.
#[path = "annotation/diagnostic_windows.rs"]
mod diagnostic_windows;

/// The lines a stored hint answer covers.
#[path = "annotation/stored_answer_coverage.rs"]
mod stored_answer_coverage;
