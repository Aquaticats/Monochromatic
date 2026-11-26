//! Hits, carets and ink on continuation rows:
//!  a hit or a caret on a continuation row resolves in that row,
//!  the selected ink reaches it,
//!  and an underline across a wrap break stands on its own rows.

use crate::fixtures::{
    CODE,
    viewport,
};
use ide_app::{
    annotation::{
        Mark,
        Visible,
    },
    annotation_layout::{
        AnnotationColors,
        lay_out,
    },
    code_colors::CodeColors,
    document::{
        Document,
        ReadingAffinity,
        ReadingPosition,
    },
    language::diagnostics::Severity,
    row_map::{
        CODE_ROW,
        RowMap,
    },
    shaped_text::TextShaper,
    tile_scene::{
        read::{
            center_y,
            drawn_runs,
            drawn_strokes,
        },
        tile_items,
    },
};

/// A click inside a continuation row resolves in that row,
/// and the caret there is drawn on that row.
#[test]
fn hits_and_carets_on_continuation_rows_resolve_in_their_row() {
    let document = Document::new(CODE);
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let view = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    let row = &view.rows[0];
    assert!(row.visual_rows() >= 3);
    for index in 0..row.visual_rows() {
        let bytes = row.visual_row_bytes(index).expect("row exists");
        let y = row.visual_row_offset(index, 1.0) + CODE_ROW / 2.0;
        let hit = view.hit_at(&document, 0, 20.0, y);
        let local = hit - row.source_start;
        let byte = row.projection.source_to_byte[local];
        assert!(
            bytes.contains(&byte) || (index + 1 == row.visual_rows() && byte == bytes.end),
            "row {index}: a click at x 20 landed in byte {byte}, outside {bytes:?}"
        );
    }
    // The caret at the first character of the second row is on that row, one code row down.
    let second = row.visual_row_sources(1).expect("second row").start;
    let mut moved = Document::new(CODE);
    moved.select(ReadingPosition {
        anchor: second,
        head: second,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let lower = view.caret(&moved);
    moved.select(ReadingPosition {
        anchor: second,
        head: second,
        viewport: 0,
        affinity: ReadingAffinity::Upstream,
    });
    let upper = view.caret(&moved);
    assert_eq!(lower.y, CODE_ROW + 2.0, "downstream draws on the lower row");
    assert_eq!(upper.y, 2.0, "upstream draws on the upper row");
    assert!(upper.x > lower.x);
}

/// Selected ink reaches the glyphs of every visual row of a selected wrapped line.
#[test]
fn selected_ink_reaches_continuation_rows() {
    let mut document = Document::new(CODE);
    let end = document.text().len_chars();
    document.select(ReadingPosition {
        anchor: 0,
        head: end,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let view = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    assert!(view.rows[0].visual_rows() >= 3);
    assert_eq!(
        view.selections.len(),
        view.rows[0].visual_rows(),
        "one rectangle per visual row"
    );
    let colors = CodeColors {
        foreground: [0, 0, 0, 255],
        selected: [255, 0, 0, 255],
        dark: false,
    };
    let items = tile_items(&view, colors, 0.0).expect("items");
    let runs = drawn_runs(&items);
    // Every visual row has a run in the selected ink on its own baseline, and the selection clips it to the row's
    // rectangle, which spans the glyphs of the row.
    let mut baselines: Vec<f32> = Vec::new();
    for run in &runs {
        if run.ink == colors.selected {
            baselines.push(run.baseline);
        }
    }
    baselines.sort_by(f32::total_cmp);
    baselines.dedup_by(|left, right| return (*left - *right).abs() < 1.0);
    assert_eq!(
        baselines.len(),
        view.rows[0].visual_rows(),
        "every visual row has selected ink on its own baseline"
    );
    for rectangle in &view.selections {
        let mut reached = false;
        for run in &runs {
            if run.ink == colors.selected
                && run.clip.is_some_and(|(left, right)| {
                    return (left - rectangle.x).abs() <= 0.5
                        && (right - (rectangle.x + rectangle.width)).abs() <= 0.5;
                })
            {
                reached = true;
            }
        }
        assert!(
            reached,
            "no selected ink is clipped to the rectangle at y {}",
            rectangle.y
        );
    }
}

/// An underline across a wrap break is laid out once for each visual row it crosses,
/// each on its own row.
#[test]
fn underlines_across_a_wrap_break_stand_on_their_own_rows() {
    let document = Document::new(CODE);
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let view = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    let row = &view.rows[0];
    let second = row.visual_row_sources(1).expect("second row").start;
    let mark = Mark {
        start: second - 3,
        end: second + 3,
        severity: Severity::Error,
    };
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![mark],
    };
    let colors = AnnotationColors {
        hint: [1, 2, 3, 255],
        error: [200, 0, 0, 255],
        warning: [200, 100, 0, 255],
        information: [0, 0, 200, 255],
        suggestion: [90, 90, 90, 255],
    };
    let frame = lay_out(&view, &visible, &mut shaper, colors);
    let mut offsets: Vec<f32> = Vec::new();
    for run in frame.underlines.iter() {
        offsets.push(run.offset);
    }
    offsets.sort_by(f32::total_cmp);
    assert_eq!(
        offsets.len(),
        2,
        "one run for each visual row the mark crosses"
    );
    assert_eq!(
        offsets[0], 0.0,
        "the run on the upper row stays on the first row"
    );
    assert_eq!(
        offsets[1], CODE_ROW,
        "the run on the lower row is not drawn on its own visual row"
    );
    // Painted, the lower run's stroke lands in the lower row's band and none of it in the rows below.
    let mut framed = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    framed.annotations = Some(frame);
    let items = tile_items(
        &framed,
        CodeColors {
            foreground: [0, 0, 0, 255],
            selected: [255, 255, 255, 255],
            dark: false,
        },
        0.0,
    )
    .expect("items");
    let mut stroke_rows = Vec::new();
    for drawn in drawn_strokes(&items) {
        assert_eq!(
            drawn.stroke.ink, colors.error,
            "the stroke is in the error ink"
        );
        stroke_rows.push((center_y(&drawn.stroke) / CODE_ROW).floor() as usize);
    }
    stroke_rows.sort_unstable();
    stroke_rows.dedup();
    assert_eq!(
        stroke_rows,
        vec![0, 1],
        "the error ink stands on exactly the two rows the mark crosses"
    );
}
