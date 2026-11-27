//! The vertical row map:
//!  rows numbered from one and read back,
//!  a copy that changes without the original,
//!  wrapped lines that push later lines down,
//!  one line per pixel,
//!  a page step counted in visual rows,
//!  and a million lines that stay cheap.

use crate::fixtures::wrapped_map;
use ide_app::{
    line_rows::LineRows,
    row_map::{
        CODE_ROW,
        Place,
        RowMap,
    },
};

/// Every line counts one row until it is measured,
/// and a changed count reads back.
#[test]
fn rows_start_at_one_and_read_back_what_was_set() {
    let mut rows = LineRows::plain(1000);
    assert_eq!(rows.rows(0), 1);
    assert_eq!(rows.rows(999), 1);
    assert_eq!(rows.rows(5000), 1, "a line the table lacks counts as one");
    assert!(rows.apply([(300, 4)]));
    assert_eq!(rows.rows(300), 4);
    assert_eq!(rows.extra_before(300), 0);
    assert_eq!(rows.extra_before(301), 3);
    assert_eq!(rows.total_extra(), 3);
    assert!(!rows.apply([(300, 4)]), "the same count changes nothing");
    assert!(rows.apply([(300, 0)]), "a count below one is one");
    assert_eq!(rows.rows(300), 1);
    assert_eq!(rows.total_extra(), 0);
}

/// A copy shares its chunks,
/// and changing the copy leaves the original alone.
#[test]
fn a_copy_changes_without_touching_the_original() {
    let mut original = LineRows::plain(2000);
    original.apply([(10, 2), (1500, 3)]);
    let mut copy = original.clone();
    assert!(copy == original);
    copy.apply([(10, 5), (1900, 2)]);
    assert!(copy != original);
    assert_eq!(original.rows(10), 2);
    assert_eq!(original.rows(1900), 1);
    assert_eq!(copy.rows(10), 5);
    assert_eq!(copy.rows(1900), 2);
    assert_eq!(copy.rows(1500), 3, "an untouched chunk is shared");
    assert_eq!(original.total_extra(), 1 + 2);
    assert_eq!(copy.total_extra(), 4 + 2 + 1);
}

/// Wrapped lines push everything beneath them down by their extra rows,
/// and nothing above.
#[test]
fn wrapped_lines_push_later_lines_down() {
    let map = wrapped_map();
    assert_eq!(map.code_top(0), 0.0);
    assert_eq!(
        map.code_top(2),
        2.0 * CODE_ROW,
        "nothing above line 2 moved"
    );
    assert_eq!(map.code_bottom(2), 2.0 * CODE_ROW + 3.0 * CODE_ROW);
    assert_eq!(
        map.block_top(3),
        2.0 * CODE_ROW + 3.0 * CODE_ROW,
        "line 3 starts under line 2's three rows"
    );
    assert_eq!(map.code_top(3), map.block_top(3) + 26.0);
    assert_eq!(map.code_bottom(3), map.code_top(3) + 2.0 * CODE_ROW);
    assert_eq!(map.code_top(4), map.code_bottom(3));
    assert_eq!(
        map.code_top(8),
        map.code_top(4) + 3.0 * CODE_ROW + 5.0 * CODE_ROW
    );
    // Ten lines, nine of one row, plus 2 + 1 + 4 extra rows and one block.
    assert_eq!(map.height(), (10.0 + 7.0) * CODE_ROW + 26.0);
    assert_eq!(map.rows_of(7), 5);
    assert_eq!(map.rows_of(99), 1);
}

/// Every pixel belongs to exactly one line,
/// whatever the line's height.
#[test]
fn every_pixel_belongs_to_one_line_with_wrapped_lines() {
    let map = wrapped_map();
    let mut previous = 0;
    let mut y: f32 = 0.0;
    while y < map.height() {
        let place = map.locate(y);
        assert!(
            place.line == previous || place.line == previous + 1,
            "line {} after {previous} at {y}",
            place.line
        );
        assert!(y >= map.block_top(place.line) && y < map.code_bottom(place.line));
        assert_eq!(place.in_block, y < map.code_top(place.line));
        previous = place.line;
        y += 0.5;
    }
    assert_eq!(previous, 9);
    assert_eq!(
        map.locate(map.code_top(2) + 40.0),
        Place {
            line: 2,
            in_block: false
        }
    );
    assert_eq!(map.visual_row_at(2, map.code_top(2) + 40.0), 1);
    assert_eq!(map.visual_row_at(2, map.code_top(2) + 71.9), 2);
    assert_eq!(
        map.visual_row_at(2, map.code_top(2) + 500.0),
        2,
        "below the rows is the last row"
    );
    assert_eq!(map.visual_row_at(2, -5.0), 0);
}

/// A page step counts visual rows,
/// and a block counts with the first row of its line.
#[test]
fn a_page_step_counts_visual_rows_with_the_blocks_above_their_first_rows() {
    let map = wrapped_map();
    let (rows, pixels) = map.page(map.code_top(2), 150.0, true);
    assert_eq!(
        rows, 5,
        "line 2 takes three rows (72 px), line 3 takes 26 + 24 for its first row and 24 for its second"
    );
    assert_eq!(pixels, 3.0 * CODE_ROW + 26.0 + 2.0 * CODE_ROW);
    let (three, taken) = map.page(map.code_top(2), 100.0, true);
    assert_eq!(
        (three, taken),
        (3, 72.0),
        "the first row of line 3 carries its 26 px block, so it would end at 122 px and does not fit"
    );
}

/// A table for a million lines answers a position in a handful of lookups and copies in microseconds.
#[test]
fn a_million_lines_stay_cheap() {
    let mut rows = LineRows::plain(1_000_000);
    let mut updates: Vec<(usize, usize)> = Vec::new();
    for line in (0..1_000_000).step_by(1000) {
        updates.push((line, 3));
    }
    rows.apply(updates);
    let map = RowMap::plain(1_000_000).with_rows(&rows);
    let copy = map.clone();
    assert!(copy == map);
    assert_eq!(map.code_top(2000), 2000.0 * CODE_ROW + 2.0 * 2.0 * CODE_ROW);
    let place = map.locate(map.code_top(777_000) + 30.0);
    assert_eq!(place.line, 777_000);
    assert_eq!(map.rows_of(777_000), 3);
}
