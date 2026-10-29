//! Annotation geometry from the production shaped rows:
//!  underline runs over tabs,
//!  CJK,
//!  combining marks,
//! and ligatures,
//!  multi-line and point ranges,
//!  overlap order,
//!  hints packed onto rows at the exact pixel x of
//! their positions,
//!  message rows at the x of their diagnostics,
//!  and code rows whose inner geometry never changes.

/// The production layout and packing,
///  the records they consume,
///  and the vertical mapping blocks feed.
use ide_app::{
    annotation::{Label, Mark, Visible},
    annotation_layout::{AnnotationColors, AnnotationFrame, lay_out, pack},
    document::Document,
    language::diagnostics::Severity,
    row_map::{CODE_ROW, RowMap},
    shaped_text::{ShapedView, TERMINATOR_MARK, TextShaper, Viewport},
    virtual_row::{BLOCK_GAP, Block, CONTINUATION_INDENT, HINT_GAP, MessageRow, ROW_HEIGHT},
};
/// Blocks are shared between the window state and the frames that paint them.
use std::sync::Arc;

/// Inks are irrelevant to geometry;
///  any fixed set will do.
const COLORS: AnnotationColors = AnnotationColors {
    hint: [1, 2, 3, 255],
    error: [200, 0, 0, 255],
    warning: [200, 100, 0, 255],
    information: [0, 0, 200, 255],
    suggestion: [90, 90, 90, 255],
};

/// What:
///  Shape `source` and lay out `visible` against it;
///  the tuple returns the document,
///  the view,
///  and the frame.
/// Why:
///  Every assertion compares annotation positions with the geometry the window paints and hit-tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function laidOut(source: string, visible: Visible): [Document, ShapedView, AnnotationFrame];
/// ```
fn laid_out(source: &str, visible: &Visible) -> (Document, ShapedView, AnnotationFrame) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 8,
        width: 900.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let view = shaper.prepare(&document, viewport, &[]);
    let frame = lay_out(&view, visible, &mut shaper, COLORS);
    return (document, view, frame);
}

/// What:
///  One hint label at `position`;
///  `&str` lends the text,
///  which the label copies.
/// Why:
///  Packing takes labels exactly as the annotation store keeps them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function label(position: number, text: string): Label;
/// ```
fn label(position: usize, text: &str) -> Label {
    return Label {
        position,
        text: text.to_string(),
    };
}

/// One message row of `severity` aligned to source character `start`.
fn message(start: usize, continued: bool, severity: Severity, text: &str) -> MessageRow {
    return MessageRow {
        start,
        continued,
        severity,
        gap: None,
        text: text.to_string(),
    };
}

/// What:
///  Pack `labels` above line `line` of `source`,
///  add `messages`,
///  and shape and lay out the whole text with
///       that one block in the vertical mapping.
///  The tuple returns the view,
///  the frame,
///  and the block.
/// Why:
///  This is the production path:
///  packing against an unstyled row,
///  then painting against the frame's rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withBlock(source: string, line: number, labels: Label[], messages: MessageRow[]): [ShapedView, AnnotationFrame, Block];
/// ```
fn with_block(
    source: &str,
    line: usize,
    labels: &[Label],
    messages: Vec<MessageRow>,
) -> (ShapedView, AnnotationFrame, Arc<Block>) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let row = shaper.row(&document, line, 1.0);
    let (hint_rows, hints) = pack(&row, labels, &mut shaper, 1.0, None);
    let block = Arc::new(Block {
        line,
        hint_rows,
        hints,
        messages,
        held: (0.0, 0.0),
    });
    let map = RowMap::new(document.text().len_lines(), &[(line, block.height())]);
    let viewport = Viewport {
        first: 0,
        count: 8,
        width: 900.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    let view = shaper.prepare_rows(&document, viewport, &[], &map);
    let visible = Visible {
        blocks: vec![Arc::clone(&block)],
        marks: Vec::new(),
    };
    let frame = lay_out(&view, &visible, &mut shaper, COLORS);
    return (view, frame, block);
}

/// Compare two pixel positions within a thousandth of a pixel;
///  sums of floats may round differently.
fn close(actual: f32, expected: f32) -> bool {
    return (actual - expected).abs() < 0.001;
}

/// A mark of `severity` over `start..end`.
fn mark(start: usize, end: usize, severity: Severity) -> Mark {
    return Mark {
        start,
        end,
        severity,
    };
}

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

/// Every hint stands at the exact pixel x of the position it annotates,
///  also after a tab,
///  CJK,
///  a combining
/// mark,
///  and inside a ligature;
///  its row lies above the code row by whole virtual rows.
#[test]
fn hints_stand_at_the_exact_pixel_x_of_their_position() {
    // Characters: tab(0) l(1) e(2) t(3) space(4) 猫(5) space(6) =(7) space(8) e(9) U+0301(10) space(11) !(12) =(13).
    let source = "\tlet 猫 = e\u{301} != x;\nnext";
    let labels = [
        label(1, "a"),
        label(6, ": char"),
        label(11, "b"),
        label(13, "c"),
    ];
    let (view, frame, block) = with_block(source, 0, &labels, Vec::new());
    assert_eq!(frame.texts.len(), 4);
    for (text, wanted) in frame.texts.iter().zip(labels.iter()) {
        assert_eq!(
            text.x,
            view.rows[0].caret_x(wanted.position, 1.0),
            "the hint for position {} does not stand above it",
            wanted.position
        );
        assert_eq!(text.line, 0);
        assert_eq!(text.ink, COLORS.hint);
        assert_eq!(text.severity, None);
        assert!(text.width > 0.0);
    }
    // The stored placements agree with what the frame paints.
    for (text, place) in frame.texts.iter().zip(block.hints.iter()) {
        assert!(close(text.x, place.x));
        assert!(close(text.width, place.width));
        assert_eq!(text.rise, block.hint_rise(place.row));
    }
    // Half of the ligature is a real position of its own: the hint inside `!=` is right of the one before it.
    assert!(frame.texts[3].x > frame.texts[2].x);
}

/// Hints are packed as the reference editor packs them,
///  by pixels:
///  a hint that would start less than the
/// minimum gap after the previous hint's end starts a new row,
///  and only the current row is considered.
#[test]
fn overlapping_hints_take_a_new_row_and_only_the_current_row_is_considered() {
    // `area(2, 3, 4)` with a hint before each argument and one after the line's last character.
    // `format!` builds the line from a template; `repeat` writes fifty blanks.
    let source = format!("let total = area(2, 3, 4);{}end", " ".repeat(50));
    let labels = [
        label(17, "width:"),
        label(20, "height:"),
        label(23, "depth:"),
        label(source.chars().count(), "far"),
    ];
    let (_, frame, block) = with_block(&source, 0, &labels, Vec::new());
    let mut rows: Vec<usize> = Vec::new();
    for place in block.hints.iter() {
        rows.push(place.row);
    }
    assert_eq!(
        rows,
        [0, 1, 2, 2],
        "each overlapping hint starts a row; the far one shares the current row"
    );
    assert_eq!(block.hint_rows, 3);
    // The rule itself, on the measured geometry: same row exactly when the gap is kept.
    for index in 1..block.hints.len() {
        let previous = &block.hints[index - 1];
        let current = &block.hints[index];
        let fits = current.x >= previous.x + previous.width + HINT_GAP;
        assert_eq!(
            current.row == previous.row,
            fits,
            "hint {index}: x {} after an end at {}",
            current.x,
            previous.x + previous.width
        );
    }
    // The third hint would fit after the first on row 0, but the reference never goes back to an earlier row.
    assert!(block.hints[2].x >= block.hints[0].x + block.hints[0].width + HINT_GAP);
    // Rows are stacked from the code row upwards: the first hint is highest.
    assert_eq!(frame.texts[0].rise, 3.0 * ROW_HEIGHT);
    assert_eq!(frame.texts[1].rise, 2.0 * ROW_HEIGHT);
    assert_eq!(frame.texts[2].rise, ROW_HEIGHT);
    assert_eq!(frame.texts[3].rise, ROW_HEIGHT);
    assert_eq!(block.height(), BLOCK_GAP + 3.0 * ROW_HEIGHT);
    assert!(close(frame.extent, frame.texts[3].x + frame.texts[3].width));
}

/// Hints that keep the gap share one row.
#[test]
fn hints_that_keep_the_gap_share_one_row() {
    let source = "const label = make(1); const total = area(2);";
    let labels = [label(11, ": string"), label(34, ": number")];
    let (_, _, block) = with_block(source, 0, &labels, Vec::new());
    assert_eq!(block.hint_rows, 1);
    assert_eq!(block.hints[0].row, 0);
    assert_eq!(block.hints[1].row, 0);
    assert_eq!(block.height(), BLOCK_GAP + ROW_HEIGHT);
}

/// Message rows follow the hint rows,
///  start at the pixel x of their diagnostic,
///  take their severity's ink,
///  and
/// the last one sits directly on the code row;
///  a continued row is indented.
#[test]
fn message_rows_start_at_their_diagnostic_and_sit_tight_on_the_code_row() {
    let source = "first line\nlet total = area(2, 3);\nlast";
    let labels = [label(20, ": u32")];
    let messages = vec![
        message(
            23,
            false,
            Severity::Error,
            "Error E0308 (rustc): mismatched types",
        ),
        message(23, true, Severity::Error, "expected `u32`, found `&str`"),
        message(15, false, Severity::Hint, "Hint: unused"),
    ];
    let (view, frame, block) = with_block(source, 1, &labels, messages);
    assert_eq!(frame.texts.len(), 4);
    assert_eq!(block.height(), BLOCK_GAP + 4.0 * ROW_HEIGHT);
    // The code row of the annotated line starts below its block; the line above did not move.
    assert_eq!(view.rows[0].top, 0.0);
    assert_eq!(view.rows[1].top, CODE_ROW + block.height());
    assert_eq!(view.rows[2].top, 2.0 * CODE_ROW + block.height());
    let row = &view.rows[1];
    // Hint row first, then the messages in their given order; rises count rows up from the code row.
    assert_eq!(frame.texts[0].rise, 4.0 * ROW_HEIGHT);
    assert_eq!(frame.texts[1].rise, 3.0 * ROW_HEIGHT);
    assert_eq!(frame.texts[2].rise, 2.0 * ROW_HEIGHT);
    assert_eq!(
        frame.texts[3].rise, ROW_HEIGHT,
        "the last row must sit directly on the code row"
    );
    assert_eq!(frame.texts[1].x, row.caret_x(23, 1.0));
    assert_eq!(
        frame.texts[2].x,
        row.caret_x(23, 1.0) + CONTINUATION_INDENT,
        "a continued row is indented"
    );
    assert_eq!(frame.texts[3].x, row.caret_x(15, 1.0));
    assert_eq!(frame.texts[0].ink, COLORS.hint);
    assert_eq!(frame.texts[1].ink, COLORS.error);
    assert_eq!(frame.texts[2].ink, COLORS.error);
    assert_eq!(frame.texts[3].ink, COLORS.suggestion);
    assert_eq!(frame.texts[1].severity, Some(Severity::Error));
    assert_eq!(frame.texts[3].severity, Some(Severity::Hint));
    // Every text of the block lies inside the block: below its gap and above the code row.
    for text in &frame.texts {
        assert!(text.rise >= ROW_HEIGHT && text.rise <= block.height() - BLOCK_GAP);
        assert_eq!(text.line, 1);
    }
}

/// Virtual rows never change the geometry inside a code row:
///  every caret x,
///  hit,
///  and range width is the same
/// with and without a block above the line;
///  only the row's top differs,
///  by the block's height.
#[test]
fn geometry_inside_a_code_row_is_unchanged_by_its_block() {
    let source = "\tlet 猫 = e\u{301} != x;\nnext";
    let labels = [label(2, ": i32"), label(8, "value:")];
    let messages = vec![message(5, false, Severity::Error, "Error: a problem")];
    let (document, plain, _) = laid_out(source, &Visible::default());
    let (annotated, frame, block) = with_block(source, 0, &labels, messages);
    let length = document.text().line(0).len_chars() - 1;
    assert_eq!(plain.rows[0].top, 0.0);
    assert_eq!(
        annotated.rows[0].top,
        block.height(),
        "a block above the first line starts the text"
    );
    assert_eq!(annotated.origin, 0.0);
    for position in 0..=length {
        let x = plain.rows[0].caret_x(position, 1.0);
        assert_eq!(x, annotated.rows[0].caret_x(position, 1.0));
        assert_eq!(
            plain.rows[0].hit(x + 0.01, 1.0),
            annotated.rows[0].hit(x + 0.01, 1.0)
        );
        for end in position..=length {
            let before = plain.range(position, end);
            let after = annotated.range(position, end);
            assert_eq!(before.len(), after.len());
            for (expected, actual) in before.iter().zip(after.iter()) {
                assert_eq!(expected.x, actual.x);
                assert_eq!(expected.width, actual.width);
                assert_eq!(actual.y, expected.y + block.height());
                assert_eq!(actual.height, CODE_ROW);
            }
        }
    }
    assert!(!frame.texts.is_empty());
    // The raster is taller by exactly the block.
    assert_eq!(annotated.height, plain.height + block.height() as u32);
}
