//! Soft wrapping of the lines the source view shows:
//!  the break rules and the rows they make,
//!  where a caret,
//! a point and a range fall on those rows,
//!  and the sweep that proves every row of this package's own source fits.
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this crate's modules live under `tests/wrap/`.
//! Why:
//!  One test crate links the library once and shares the shaped-row helpers
//!       without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './wrap/fixtures';
//! ```

#[path = "wrap/fixtures.rs"]
mod fixtures;

/// The break rules and the rows they make.
#[path = "wrap/break_rules.rs"]
mod break_rules;

/// A caret,
/// a point and a range on the visual rows.
#[path = "wrap/row_geometry.rs"]
mod row_geometry;

/// Every row of this package's own source,
/// at six width and scale pairs.
#[path = "wrap/real_source_sweep.rs"]
mod real_source_sweep;
