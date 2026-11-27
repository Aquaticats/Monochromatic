//! Lane-owned vertical scrolling for the pane strip.
//!
//! Full columns are static canvases.
//!  The outer scroller owns vertical wheel input for the whole app.
//! Each sibling-group lane,
//!  one parent pane plus its direct children,
//!  reacts to that global app
//! scroll and independently applies a bounded sticky offset inside its green-box range.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `geometry.rs`:
//!   lane grouping and lane rectangle arithmetic.
//! - `pane_row_positions.rs`:
//!   where one pane sits vertically,
//!   from its sticky target through rail bounds and neighbour spacing.
//! - `debug_child_lane_overlays.rs`:
//!   the debug-only green box and label drawn over one sibling group.
//! - `app_scroll_lane_offsets.rs`:
//!   the whole-app vertical scroll turned into each lane's bounded sticky offset,
//!   plus revealing one lane member.
//!
//! The `mod` items below read in rustfmt's alphabetical order;
//!  the list above is the order their contents may be read in.

/// What:
///  app-scroll sync and vertical reveal for the layout adapter.
/// Why:
///  the whole-app scroll decides each lane's sticky offset,
///  and a spawn asks for one lane
///      member to be brought into view.
mod app_scroll_lane_offsets;
/// What:
///  debug child-lane overlays for the layout adapter.
/// Why:
///  debug mode draws one green box per sibling group,
///  and that drawing is unrelated to the
///      offsets themselves.
mod debug_child_lane_overlays;
/// What:
///  lane geometry helpers.
/// Why:
///  grouping,
///  rectangles,
///  and lane dimensions are pure calculations split out to keep this file
///      under the max-lines budget.
mod geometry;
/// What:
///  pane row resolution for the layout adapter.
/// Why:
///  rendering,
///  reveal and rail bounds must all agree on where one pane sits vertically.
mod pane_row_positions;
