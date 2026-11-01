//! Tests for the pure Settings layout rules in `settings_layout`.

use super::*;

#[test]
fn the_single_form_is_one_pane_of_the_whole_width() {
    for width in [360.0, 443.0, 599.0] {
        assert!(!is_wide(width), "width {width}");
        assert_eq!(list_pane_width(width), width);
        assert_eq!(editor_pane_width(width), 0.0);
    }
}

#[test]
fn the_wide_form_starts_at_600_like_the_player() {
    assert!(!is_wide(599.9));
    assert!(is_wide(600.0));
}

#[test]
fn the_wide_form_gives_the_list_the_default_split_ratio_in_whole_pixels() {
    // 0.3636 of 600, 850 and 1200, rounded to a pixel (the stock panes measure 0.3632 of the window).
    assert_eq!(list_pane_width(600.0), 218.0);
    assert_eq!(list_pane_width(850.0), 309.0);
    assert_eq!(list_pane_width(1200.0), 436.0);
}

#[test]
fn the_panes_span_the_window_with_no_band_between_them() {
    for width in [600.0, 721.0, 850.0, 1200.0, 1920.0] {
        assert_eq!(list_pane_width(width) + editor_pane_width(width), width, "width {width}");
        assert_eq!(list_pane_width(width).fract(), 0.0, "width {width}");
    }
}

#[test]
fn rows_run_margin_to_margin_with_no_maximum_width() {
    assert_eq!(row_width(443.0), 411.0);
    assert_eq!(row_width(1200.0), 1168.0);
    assert_eq!(row_width(4000.0), 3968.0);
}

#[test]
fn rows_are_72_tall_with_2_between_them() {
    assert_eq!(row_top(0), 0.0);
    assert_eq!(row_top(1), 74.0);
    assert_eq!(row_top(2), 148.0);
}

#[test]
fn the_icon_and_the_title_sit_where_the_stock_rows_put_them() {
    // 32 and 84 from the window edge, which are 16 and 68 inside a row that starts at the 16 margin.
    assert_eq!(SIDE_MARGIN + icon_x_in_row(), 32.0);
    assert_eq!(SIDE_MARGIN + title_x_in_row(), 84.0);
    assert_eq!(icon_x_in_row(), 16.0);
    assert_eq!(title_x_in_row(), 68.0);
    assert_eq!(icon_y_in_row(), 16.0);
    assert_eq!(ICON_DIAMETER, 40.0);
}

#[test]
fn the_title_clears_the_icon_circle() {
    assert!(title_x_in_row() > icon_x_in_row() + ICON_DIAMETER);
}
