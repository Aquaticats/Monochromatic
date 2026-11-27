//! Home,
//! End,
//! Up,
//! Down,
//! and paging on the visual rows of wrapped lines,
//! without a window.
//! Every test runs at several wrap widths and at scales 1 and 2,
//! on lines the same shaper wraps as the source view.
//! Rows that start or end with a tab,
//! rows that fit exactly,
//! and a wrapped line at the end of the file are covered.

/// What:
///  `#[path = "..."]` names the file a module is read from;
///  this crate's modules live under `tests/wrap_keys/`.
/// Why:
///  One test crate links the library once and shares the fixture line shapes
///       and their helpers without unused-code warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fixtures from './wrap_keys/fixtures';
/// ```
#[path = "wrap_keys/fixtures.rs"]
mod fixtures;

/// The caret's side of a wrap offset,
///  and Home and End on rows the test hands over.
#[path = "wrap_keys/home_and_end_on_given_rows.rs"]
mod home_and_end_on_given_rows;

/// Home and End on rows the shaper produced,
///  at every width and scale.
#[path = "wrap_keys/home_and_end_on_shaped_rows.rs"]
mod home_and_end_on_shaped_rows;

/// Down and Up over every visual row,
///  and a page step across lines.
#[path = "wrap_keys/down_up_and_paging.rs"]
mod down_up_and_paging;

/// The aim a vertical step remembers across short rows.
#[path = "wrap_keys/remembered_column.rs"]
mod remembered_column;

/// The rows at and after the end of the text.
#[path = "wrap_keys/end_of_the_text_rows.rs"]
mod end_of_the_text_rows;
