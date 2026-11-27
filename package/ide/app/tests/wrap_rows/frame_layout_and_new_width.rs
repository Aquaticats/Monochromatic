//! The frame's layout of wrapped rows:
//!  each row placed by its own height,
//!  a second measure that shapes once,
//!  and a new width that breaks kept rows again at both scales.

use crate::fixtures::{
    CODE,
    viewport,
};
use ide_app::{
    document::{
        Document,
        ReadingAffinity,
    },
    row_map::{
        CODE_ROW,
        RowMap,
    },
    shaped_text::TextShaper,
};

/// A frame that wraps places its rows by what it shaped,
/// even when the map counts every line as one row.
#[test]
fn a_frame_places_wrapped_rows_by_their_own_heights() {
    let text = format!("short\n{CODE}\nshort\n{CODE}\nend\n");
    let document = Document::new(&text);
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(document.text().len_lines());
    let view = shaper.prepare_rows(&document, viewport(0, 6, 300.0), &[], &plain);
    let mut expected_top = 0.0;
    for row in &view.rows {
        assert_eq!(
            row.top, expected_top,
            "row {} sits right under the row before it",
            row.row
        );
        assert_eq!(view.map.rows_of(row.row), row.visual_rows());
        expected_top += row.visual_rows() as f32 * CODE_ROW;
    }
    assert!(view.rows[1].visual_rows() > 1, "the long line wraps");
    assert_eq!(view.origin, 0.0);
    assert_eq!(view.height as f32, view.map.block_top(6).max(CODE_ROW));
    assert!(
        view.map != plain,
        "the frame's map carries the measured rows"
    );
}

/// Bringing lines in shapes them once,
/// and asking again at the same width shapes nothing.
#[test]
fn measuring_twice_shapes_once() {
    let text = format!("{CODE}\n").repeat(40);
    let document = Document::new(&text);
    let mut shaper = TextShaper::new();
    let first = shaper.measure_lines(&document, 0..40, 1.0, Some(300.0), &[]);
    assert_eq!(first.shaped, 40);
    assert_eq!(first.rebroken, 0);
    assert_eq!(first.lines.len(), 40);
    for (_, rows) in first.lines.iter() {
        assert!(*rows > 1);
    }
    let again = shaper.measure_lines(&document, 0..40, 1.0, Some(300.0), &[]);
    assert_eq!((again.shaped, again.rebroken), (0, 0));
    assert_eq!(again.lines, first.lines);
}

/// A new width breaks the kept rows again,
/// shapes none,
/// and ends with the rows a fresh shape gives.
#[test]
fn a_new_width_breaks_kept_rows_again_without_shaping() {
    let spaced = "        let value = compute_total(first_item, second_item); // trailing words keep it going on";
    let blanks = "                                                                      x";
    let mut lines = Vec::new();
    for index in 0..30 {
        lines.push(format!("{}{}", " ".repeat(index % 7), CODE));
    }
    lines.push(spaced.to_string());
    lines.push(blanks.to_string());
    lines.push("\tlet tabbed = 1; // a leading tab and then enough words to wrap around at least once more".to_string());
    let text = lines.join("\n");
    let document = Document::new(&text);
    let count = document.text().len_lines();
    let mut kept = TextShaper::new();
    let mut fresh = TextShaper::new();
    kept.measure_lines(&document, 0..count, 1.0, Some(420.0), &[]);
    let shaped_before = kept.shaped_rows();
    for width in [300.0, 180.0, 640.0, 90.0] {
        let moved = kept.measure_lines(&document, 0..count, 1.0, Some(width), &[]);
        assert_eq!(
            moved.shaped, 0,
            "width {width}: no line may be shaped again"
        );
        assert_eq!(
            moved.rebroken as usize, count,
            "width {width}: every kept row is broken again"
        );
        fresh.forget_rows();
        let wanted = fresh.measure_lines(&document, 0..count, 1.0, Some(width), &[]);
        assert_eq!(
            moved.lines, wanted.lines,
            "width {width}: row counts equal a fresh shaping"
        );
        // The rows the cache now holds are the re-broken ones; a fresh shaping must agree with each of them.
        let plain = RowMap::plain(count);
        let view = kept.prepare_rows(&document, viewport(0, count, width), &[], &plain);
        for kept_row in &view.rows {
            let fresh_row = fresh.wrapped_row(&document, kept_row.row, 1.0, width);
            assert_eq!(kept_row.visual_rows(), fresh_row.visual_rows());
            for index in 0..kept_row.visual_rows() {
                assert_eq!(
                    kept_row.visual_row_bytes(index),
                    fresh_row.visual_row_bytes(index),
                    "width {width}, line {}, row {index}: same break",
                    kept_row.row
                );
                let start =
                    kept_row.visual_row_sources(index).expect("row").start + kept_row.source_start;
                let (left, right) = (
                    kept_row.caret_at(start, ReadingAffinity::Downstream, 1.0),
                    fresh_row.caret_at(start, ReadingAffinity::Downstream, 1.0),
                );
                assert!(
                    (left.x - right.x).abs() < 0.01,
                    "width {width}: row start x {} vs {}",
                    left.x,
                    right.x
                );
            }
        }
    }
    assert_eq!(kept.shaped_rows(), shaped_before, "no resize shaped a line");
    assert!(kept.rebroken_rows() >= 4 * count as u64);
}

/// Rows broken again at a new width at scale 2 count as many rows as a fresh shaping gives:
///  the continuation indent is the same on both paths.
#[test]
fn a_new_width_breaks_kept_rows_like_a_fresh_shaping_at_scale_two() {
    let mut lines = Vec::new();
    for index in 0..30 {
        lines.push(format!("{}{}", " ".repeat(2 + index % 9), CODE));
    }
    let text = lines.join("\n");
    let document = Document::new(&text);
    let count = document.text().len_lines();
    let mut kept = TextShaper::new();
    let mut fresh = TextShaper::new();
    kept.measure_lines(&document, 0..count, 2.0, Some(420.0), &[]);
    for width in [300.0, 180.0, 640.0] {
        let moved = kept.measure_lines(&document, 0..count, 2.0, Some(width), &[]);
        assert_eq!(moved.shaped, 0, "width {width}: no line is shaped again");
        fresh.forget_rows();
        let wanted = fresh.measure_lines(&document, 0..count, 2.0, Some(width), &[]);
        assert_eq!(
            moved.lines, wanted.lines,
            "scale 2 width {width}: broken again like a fresh shaping"
        );
    }
}
