//! The underlines a frame draws for its marks:
//!  their runs over tabs,
//! CJK,
//! combining marks and ligatures,
//!  every row of a multi-line range,
//!  a point range,
//!  and the order overlapping ranges are drawn in.

use super::fixtures::{
    close,
    laid_out,
    mark,
};
use ide_app::annotation::Visible;
use ide_app::language::diagnostics::Severity;
use ide_app::shaped_text::TERMINATOR_MARK;

/// Underline runs equal the selection geometry of their range:
///  a widened tab,
///  a CJK glyph,
///  a letter with a
/// combining mark,
///  and half of a ligature.
#[test]
fn underline_runs_follow_range_geometry_through_tabs_cjk_combining_marks_and_ligatures() {
    // Characters: tab(0) b(1) space(2) 猫(3) 猫(4) space(5) e(6) U+0301(7) x(8) space(9) !(10) =(11).
    // A leading tab is two space advances wide.
    let source = "\tb 猫猫 e\u{301}x !=";
    let cases = [(0, 1), (3, 4), (6, 8), (10, 11), (0, 12)];
    for (start, end) in cases {
        let visible = Visible {
            blocks: Vec::new(),
            marks: vec![mark(start, end, Severity::Error)],
        };
        let (_, view, frame) = laid_out(source, &visible);
        let expected = view.range(start, end);
        assert_eq!(frame.underlines.len(), expected.len(), "{start}..{end}");
        for (run, rect) in frame.underlines.iter().zip(expected.iter()) {
            assert_eq!(run.x, rect.x, "{start}..{end}");
            assert_eq!(run.width, rect.width, "{start}..{end}");
            assert!(run.width > 0.0, "{start}..{end} has no width");
        }
    }
    // Half of the `!=` ligature is narrower than the whole ligature, and both have width.
    let (_, view, _) = laid_out(source, &Visible::default());
    let half = view.range(10, 11)[0].width;
    let whole = view.range(10, 12)[0].width;
    assert!(half > 0.0 && half < whole, "half {half}, whole {whole}");
    // The tab is wider than one space advance, because it reaches its stop.
    let tab = view.range(0, 1)[0].width;
    let space = view.range(2, 3)[0].width;
    assert!(tab > space, "tab {tab}, space {space}");
}

/// A range over several lines marks every row,
///  and an empty line inside it gets a terminator-wide run.
#[test]
fn multi_line_range_marks_every_row_including_an_empty_line() {
    // Line starts: 0, 4, 5; the mark runs from `b` on the first line to `e` on the third.
    let source = "abc\n\ndef";
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![mark(1, 7, Severity::Warning)],
    };
    let (_, view, frame) = laid_out(source, &visible);
    let mut rows: Vec<usize> = Vec::new();
    for run in frame.underlines.iter() {
        rows.push(run.row);
    }
    assert_eq!(rows, [0, 0, 1, 2]);
    let first_end = view.rows[0].caret_x(3, 1.0);
    assert_eq!(frame.underlines[1].x, first_end);
    assert_eq!(frame.underlines[1].width, TERMINATOR_MARK);
    assert_eq!(frame.underlines[2].x, 0.0);
    assert_eq!(frame.underlines[2].width, TERMINATOR_MARK);
    let last = view.range(5, 7);
    assert_eq!(frame.underlines[3].x, last[0].x);
    assert_eq!(frame.underlines[3].width, last[0].width);
}

/// A point at a line end is marked after the text;
///  a point inside a line is centered on its boundary;
/// a point at the start of a line does not reach left of the text.
#[test]
fn point_ranges_get_a_terminator_wide_run() {
    let source = "let x = 1\nnext";
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![
            mark(9, 9, Severity::Error),
            mark(4, 4, Severity::Error),
            mark(10, 10, Severity::Error),
        ],
    };
    let (_, view, frame) = laid_out(source, &visible);
    let at_end = view.rows[0].caret_x(9, 1.0);
    let inside = view.rows[0].caret_x(4, 1.0);
    assert_eq!(frame.underlines.len(), 3);
    assert_eq!(frame.underlines[0].x, at_end);
    assert!(close(frame.underlines[1].x, inside - TERMINATOR_MARK / 2.0));
    assert_eq!(frame.underlines[2].row, 1);
    assert_eq!(frame.underlines[2].x, 0.0);
    for run in &frame.underlines {
        assert_eq!(run.width, TERMINATOR_MARK);
    }
}

/// Where ranges overlap,
///  the mildest severity is drawn first,
///  so the worst ends on top.
#[test]
fn overlapping_ranges_draw_the_worst_severity_last() {
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![
            mark(0, 6, Severity::Error),
            mark(2, 8, Severity::Hint),
            mark(1, 3, Severity::Warning),
            mark(4, 5, Severity::Information),
        ],
    };
    let (_, _, frame) = laid_out("abcdefghij", &visible);
    let mut order: Vec<Severity> = Vec::new();
    for run in frame.underlines.iter() {
        order.push(run.severity);
    }
    assert_eq!(
        order,
        [
            Severity::Hint,
            Severity::Information,
            Severity::Warning,
            Severity::Error,
        ]
    );
}
