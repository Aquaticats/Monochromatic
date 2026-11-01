//! Tests for the pure Settings layout rules in `settings_layout`.

use super::*;

#[test]
fn the_single_form_is_one_pane_of_the_whole_width() {
    for width in [360.0, 443.0, WIDE_FROM_WIDTH - 1.0] {
        assert!(!is_wide(width), "width {width}");
        assert_eq!(list_pane_width(width), width);
        assert_eq!(editor_pane_width(width), 0.0);
    }
}

#[test]
fn the_wide_form_starts_at_600_like_the_player() {
    assert!(!is_wide(WIDE_FROM_WIDTH - 0.1));
    assert!(is_wide(WIDE_FROM_WIDTH));
}

#[test]
fn the_wide_form_gives_the_list_the_default_split_ratio_in_whole_pixels() {
    // The stock panes measure 0.3632 of the window; the list gets LIST_SPLIT_RATIO of it, rounded to a pixel.
    for width in [WIDE_FROM_WIDTH, 850.0, 1200.0] {
        assert_eq!(list_pane_width(width), (width * LIST_SPLIT_RATIO).round(), "width {width}");
    }
}

#[test]
fn the_panes_span_the_window_with_no_band_between_them() {
    for width in [WIDE_FROM_WIDTH, 721.0, 850.0, 1200.0, 1920.0] {
        assert_eq!(list_pane_width(width) + editor_pane_width(width), width, "width {width}");
        assert_eq!(list_pane_width(width).fract(), 0.0, "width {width}");
    }
}

#[test]
fn rows_run_margin_to_margin_with_no_maximum_width() {
    for width in [443.0, 1200.0, 4000.0] {
        assert_eq!(row_width(width), width - 2.0 * SIDE_MARGIN, "width {width}");
    }
}

#[test]
fn rows_are_72_tall_with_2_between_them() {
    assert_eq!(row_top(0), 0.0);
    assert_eq!(row_top(1), ROW_HEIGHT + ROW_GAP);
    assert_eq!(row_top(2), 2.0 * (ROW_HEIGHT + ROW_GAP));
}

#[test]
fn the_icon_and_the_title_sit_where_the_stock_rows_put_them() {
    // ICON_X_FROM_EDGE and TITLE_X_FROM_EDGE are measured from the window edge; the row functions subtract the
    // rows' own SIDE_MARGIN, and the icon circle is centered in the row.
    assert_eq!(SIDE_MARGIN + icon_x_in_row(), ICON_X_FROM_EDGE);
    assert_eq!(SIDE_MARGIN + title_x_in_row(), TITLE_X_FROM_EDGE);
    assert_eq!(icon_x_in_row(), ICON_X_FROM_EDGE - SIDE_MARGIN);
    assert_eq!(title_x_in_row(), TITLE_X_FROM_EDGE - SIDE_MARGIN);
    assert_eq!(icon_y_in_row(), (ROW_HEIGHT - ICON_DIAMETER) / 2.0);
}

#[test]
fn the_title_clears_the_icon_circle() {
    assert!(title_x_in_row() > icon_x_in_row() + ICON_DIAMETER);
}
