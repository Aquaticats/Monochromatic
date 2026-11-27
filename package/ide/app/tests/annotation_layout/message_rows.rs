//! The message rows beneath the hint rows:
//!  where each starts,
//!  the indentation of a continued row,
//!  the ink of its severity,
//!  and that none of it moves the geometry inside the code row.

use super::fixtures::{
    COLORS,
    label,
    laid_out,
    message,
    with_block,
};
use ide_app::annotation::Visible;
use ide_app::language::diagnostics::Severity;
use ide_app::row_map::CODE_ROW;
use ide_app::virtual_row::{
    BLOCK_GAP,
    CONTINUATION_INDENT,
    ROW_HEIGHT,
};

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
