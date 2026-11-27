//! The rows hints are packed onto:
//!  each hint at the exact pixel x of its position,
//!  a new row for a hint that would come too close to the one before,
//!  and one row for hints that keep the gap.

use super::fixtures::{
    COLORS,
    close,
    label,
    with_block,
};
use ide_app::virtual_row::{
    BLOCK_GAP,
    HINT_GAP,
    ROW_HEIGHT,
};

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
