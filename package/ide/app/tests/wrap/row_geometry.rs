//! Where a caret,
//! a point and a range fall on the visual rows:
//!  a row step of one code row at both scales,
//!  a caret at a wrap offset following its affinity,
//!  a hit probe with and without a row offset,
//!  and the rectangle a range yields,
//! on its first row and the rest.

use crate::fixtures::{
    LONG_LINE,
    wrapped,
};
use ide_app::{
    document::ReadingAffinity,
    row_map::CODE_ROW,
};

/// Visual rows and geometry at both scales:
/// the row step is one code row in logical pixels.
#[test]
fn row_steps_are_one_code_row_at_both_scales() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        assert_eq!(row.visual_row_offset(0, scale), 0.0);
        for index in 1..row.visual_rows() {
            let offset = row.visual_row_offset(index, scale);
            assert!(
                (offset - CODE_ROW * index as f32).abs() < 0.01,
                "row {index} at scale {scale}: {offset}"
            );
        }
    }
}

/// A caret at a soft-wrap offset is drawn at the end of the upper row upstream,
/// and at the start of the lower row downstream.
#[test]
fn caret_at_a_wrap_offset_follows_its_affinity() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        let position = row.source_start + row.visual_row_sources(1).expect("second row").start;
        let upstream = row.caret_at(position, ReadingAffinity::Upstream, scale);
        let downstream = row.caret_at(position, ReadingAffinity::Downstream, scale);
        assert_eq!(
            upstream.row, 0,
            "upstream draws on the upper row at scale {scale}"
        );
        assert_eq!(
            downstream.row, 1,
            "downstream draws on the lower row at scale {scale}"
        );
        assert!(
            upstream.x > downstream.x,
            "upstream is at the end of the upper row at scale {scale}"
        );
        assert_eq!(
            row.caret_x(position, scale),
            downstream.x,
            "caret_x keeps its downstream meaning"
        );
    }
}

/// A point on the lower row hits a source character on that row,
/// and a point on the upper row hits one before the break.
#[test]
fn hit_probes_the_visual_row_under_the_point() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        let boundary = row.source_start + row.visual_row_sources(1).expect("second row").start;
        let lower = row.hit_at(0.0, CODE_ROW * 1.5, scale);
        assert!(
            lower >= boundary,
            "lower row hit {lower} at scale {scale} is on the lower row, from {boundary}"
        );
        let upper = row.hit_at(0.0, CODE_ROW * 0.5, scale);
        assert_eq!(
            upper, row.source_start,
            "upper row hit at its start at scale {scale}"
        );
    }
}

/// `hit` without a row offset probes the first visual row,
/// and an unwrapped row keeps its probe.
#[test]
fn hit_without_a_row_offset_probes_the_first_row() {
    let scale = 1.0;
    let row = wrapped(LONG_LINE, 160.0, scale);
    let x = 40.0;
    assert_eq!(row.hit(x, scale), row.hit_at(x, CODE_ROW * 0.5, scale));
}

/// A range that crosses rows yields one rectangle per visual row,
/// each on that row's y and one code row tall.
#[test]
fn range_yields_one_rectangle_per_visual_row() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        let rectangles = row.range(row.source_start, row.source_start + row.source_len(), scale);
        assert!(
            rectangles.len() >= row.visual_rows(),
            "one rectangle per row at scale {scale}"
        );
        for rectangle in &rectangles {
            assert!((rectangle.height - CODE_ROW).abs() < 0.001);
        }
        let mut ys: Vec<f32> = Vec::new();
        for rectangle in rectangles.iter() {
            ys.push(rectangle.y);
        }
        for index in 0..row.visual_rows() {
            let expected = row.visual_row_offset(index, scale);
            let mut found = false;
            for y in ys.iter() {
                if (*y - expected).abs() < 0.001 {
                    found = true;
                }
            }
            assert!(found, "row {index} has a rectangle at {expected}");
        }
    }
}

/// The first row's rectangle keeps the row's own top exactly,
/// as unwrapped rows always did.
#[test]
fn range_on_the_first_row_keeps_the_top_exactly() {
    let row = wrapped(LONG_LINE, 160.0, 1.0);
    let rectangles = row.range(row.source_start, row.source_start + 4, 1.0);
    assert!(!rectangles.is_empty());
    assert_eq!(rectangles[0].y, row.top);
}
