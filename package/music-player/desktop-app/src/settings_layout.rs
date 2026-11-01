//! Layout rules of the Settings page, written once as numbers (D109, D110).
//!
//! What: the window widths that choose the single or the two-pane form, the pane widths of the wide form, and the
//! row geometry measured on the stock Android Settings.
//! Why: `ui/settings.slint` and `ui/app_window.slint` carry the same numbers as Slint literals; the window tests build
//! the real window and compare what it lays out with these functions, so the markup cannot drift from the decision.

/// Window width in logical pixels at and above which the player, and so Settings, uses two panes (D110).
pub const WIDE_FROM_WIDTH: f32 = 600.0;

/// Share of the window width that the list pane takes in the wide form: the stock Settings
/// `config_activity_embed_split_ratio` (D110). A desktop has no crease, so no other split applies.
pub const LIST_SPLIT_RATIO: f32 = 0.3636;

/// Height of every row, which carries two lines (D109).
pub const ROW_HEIGHT: f32 = 72.0;

/// Gap between two rows of one group (D109).
pub const ROW_GAP: f32 = 2.0;

/// Gap between two groups, which is also the space above a group's heading (D109).
pub const GROUP_GAP: f32 = 16.0;

/// Margin between the window (or pane) edge and the rows' edge (D109).
pub const SIDE_MARGIN: f32 = 16.0;

/// Diameter of the circle behind each row's icon (D109).
pub const ICON_DIAMETER: f32 = 40.0;

/// Distance from the window (or pane) edge to the icon circle's left edge (D109).
pub const ICON_X_FROM_EDGE: f32 = 32.0;

/// Distance from the window (or pane) edge to the left edge of the title (D109).
pub const TITLE_X_FROM_EDGE: f32 = 84.0;

/// Height of the header that holds Back and the page title (D109).
pub const HEADER_HEIGHT: f32 = 64.0;

/// Corner radius of the selected row's pill in whole pixels: 21, inside the stock 21 to 22 dp (D110).
pub const PILL_RADIUS: f32 = 21.0;

/// What: `is_wide` tells whether a window of `width` logical pixels shows the two-pane Settings.
/// Why: Settings switches form at the same width as the player's own two-pane layout (D110).
pub fn is_wide(width: f32) -> bool {
    return width >= WIDE_FROM_WIDTH;
}

/// What: `list_pane_width` gives the width of the list pane for a window of `width` logical pixels.
/// Why: the narrow form is one pane of the whole width; the wide form gives the list the split ratio, rounded to a
/// whole pixel so no pane edge falls between two pixels.
pub fn list_pane_width(width: f32) -> f32 {
    if !is_wide(width) {
        return width;
    }
    return (width * LIST_SPLIT_RATIO).round();
}

/// What: `editor_pane_width` gives the width of the editor pane, which is what the list pane leaves.
/// Why: the panes together span the whole window with no band between them; the narrow form has no editor pane.
pub fn editor_pane_width(width: f32) -> f32 {
    if !is_wide(width) {
        return 0.0;
    }
    return width - list_pane_width(width);
}

/// What: `row_width` gives the width of one row inside a pane of `pane_width`.
/// Why: rows run from margin to margin with no maximum width and no centering (D109).
pub fn row_width(pane_width: f32) -> f32 {
    return pane_width - 2.0 * SIDE_MARGIN;
}

/// What: `row_top` gives the y of row `index` from the top of the group's first row.
/// Why: rows are `ROW_HEIGHT` tall with `ROW_GAP` between them, so each row starts one row plus one gap below the last.
pub fn row_top(index: usize) -> f32 {
    return index as f32 * (ROW_HEIGHT + ROW_GAP);
}

/// What: `icon_x_in_row` gives the icon circle's x relative to its row's left edge.
/// Why: the measured 32 from the window edge, less the rows' own margin.
pub fn icon_x_in_row() -> f32 {
    return ICON_X_FROM_EDGE - SIDE_MARGIN;
}

/// What: `title_x_in_row` gives the title's x relative to its row's left edge.
/// Why: the measured 84 from the window edge, less the rows' own margin.
pub fn title_x_in_row() -> f32 {
    return TITLE_X_FROM_EDGE - SIDE_MARGIN;
}

/// What: `icon_y_in_row` gives the icon circle's y relative to its row's top edge.
/// Why: the circle is centered in the row's height.
pub fn icon_y_in_row() -> f32 {
    return (ROW_HEIGHT - ICON_DIAMETER) / 2.0;
}

#[cfg(test)]
#[path = "settings_layout_tests.rs"]
mod tests;
