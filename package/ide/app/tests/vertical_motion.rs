//! Up,
//!  Down,
//!  and page movement by shaped pixel position with a remembered column,
//!  without a window.

/// Canonical source,
///  the production shaper,
///  and the vertical movement under test.
use ide_app::{
    document::{Document, ReadingAffinity, ReadingPosition},
    shaped_text::TextShaper,
    vertical_motion::{PreferredColumn, vertical},
};

/// What:
///  A document with its caret at `head`;
///  `usize` is an address-sized character index (siblings `u32`,
///  `u64`).
/// Why:
///  Vertical movement starts from the document's own caret,
///  as in the native window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function at(source: string, head: number): Document;
/// ```
fn at(source: &str, head: usize) -> Document {
    let mut document = Document::new(source);
    document.select(ReadingPosition {
        anchor: head,
        head,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    return document;
}

/// What:
///  Apply a list of row steps like consecutive key presses and collect the caret after each;
/// `&[isize]` borrows signed steps (siblings:
///  owned `Vec<isize>`,
///  unsigned `usize`).
/// Why:
///  The remembered column only matters across several movements.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function travel(source: string, head: number, steps: number[]): number[];
/// ```
fn travel(source: &str, head: usize, steps: &[isize]) -> Vec<usize> {
    let mut document = at(source, head);
    let mut shaper = TextShaper::new();
    // What: `None` is the absent value of `Option` (TypeScript's `undefined`).
    // Why: The first movement has no remembered column yet.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let column: PreferredColumn | undefined;
    // ```
    let mut column: Option<PreferredColumn> = None;
    let mut visited = Vec::new();
    for step in steps {
        let landed = vertical(&mut shaper, &document, 1.0, 0.0, *step, column);
        column = Some(landed);
        document.select(ReadingPosition {
            anchor: landed.head,
            head: landed.head,
            viewport: 0,
            affinity: landed.affinity,
        });
        visited.push(landed.head);
    }
    return visited;
}

/// The caret returns to its column after passing through a short and an empty line.
#[test]
fn preferred_column_survives_short_and_empty_lines() {
    // Line starts: 0 "abcdefgh", 9 "ab", 12 "", 13 "abcdefgh", 22 "abcd" without terminator.
    let source = "abcdefgh\nab\n\nabcdefgh\nabcd";
    assert_eq!(
        travel(source, 6, &[1, 1, 1, 1]),
        [11, 12, 19, 26],
        "Down must clamp to short lines and return to column six"
    );
    assert_eq!(
        travel(source, 19, &[-1, -1, -1]),
        [12, 11, 6],
        "Up must clamp to short lines and return to column six"
    );
}

/// Up on the first line goes to the start of text,
///  Down on the last line to its end;
///  neither moves further.
#[test]
fn first_and_last_line_stop_at_the_ends_of_text() {
    let source = "abcdefgh\nab\n\nabcdefgh\nabcd";
    assert_eq!(travel(source, 6, &[-1, -1]), [0, 0]);
    assert_eq!(
        travel(source, 23, &[1, 1]),
        [26, 26],
        "a last line without terminator ends at the end of text"
    );
    // After a final terminator the last line is empty and can hold the caret.
    assert_eq!(travel("ab\ncd\n", 4, &[1, 1, -1]), [6, 6, 4]);
    assert_eq!(travel("", 0, &[1, -1]), [0, 0]);
}

/// A step larger than the remaining lines stops on the first or last line and keeps the column.
#[test]
fn page_steps_clamp_to_the_first_and_last_line() {
    let source = "abcdefgh\nab\n\nabcdefgh\nabcd";
    assert_eq!(travel(source, 6, &[3]), [19], "three lines down");
    assert_eq!(
        travel(source, 6, &[40]),
        [26],
        "a page beyond the end lands on the last line"
    );
    assert_eq!(
        travel(source, 26, &[-40]),
        [4],
        "a page beyond the start lands on the first line"
    );
    assert_eq!(travel(source, 26, &[-40, 40]), [4, 26]);
}

/// A remembered column that belongs to another caret position is ignored.
#[test]
fn stale_preferred_column_is_ignored() {
    let source = "abcdefgh\nabcdefgh";
    let document = at(source, 2);
    let mut shaper = TextShaper::new();
    let stale = PreferredColumn {
        head: 7,
        affinity: ReadingAffinity::Downstream,
        x: 10_000.0,
    };
    let landed = vertical(&mut shaper, &document, 1.0, 0.0, 1, Some(stale));
    assert_eq!(
        landed.head, 11,
        "a column remembered for another caret position moved the caret"
    );
    let current = PreferredColumn {
        head: 2,
        affinity: ReadingAffinity::Downstream,
        x: 10_000.0,
    };
    let kept = vertical(&mut shaper, &document, 1.0, 0.0, 1, Some(current));
    assert_eq!(
        kept.head, 17,
        "a column remembered for this caret position must be used"
    );
    assert_eq!(kept.x, 10_000.0);
}

/// Lines ending in CRLF are entered before their terminator.
#[test]
fn crlf_lines_are_entered_before_the_terminator() {
    // Line starts: 0 "abcd" CRLF, 6 "ab" CRLF, 10 "abcd".
    let source = "abcd\r\nab\r\nabcd";
    assert_eq!(travel(source, 3, &[1, 1]), [8, 13]);
    assert_eq!(travel(source, 4, &[1, 1, -1, -1]), [8, 14, 8, 4]);
}

/// The target is found by pixel position:
///  wide glyphs and tabs do not count as one column each.
#[test]
fn target_position_follows_pixels_across_cjk_and_tabs() {
    // Line starts: 0 "abcdefgh", 9 "猫猫猫猫", 14 tab "x", 17 "abcdefgh".
    let source = "abcdefgh\n猫猫猫猫\n\tx\nabcdefgh";
    let mut shaper = TextShaper::new();
    let document = at(source, 6);
    let aim = shaper.row(&document, 0, 1.0).caret_x(6, 1.0);
    let cjk = vertical(&mut shaper, &document, 1.0, 0.0, 1, None);
    assert_eq!(
        cjk.head,
        shaper.row(&document, 1, 1.0).hit(aim, 1.0),
        "the CJK line must be entered at the pixel position, not at column six"
    );
    let space = shaper.row(&document, 0, 1.0).caret_x(1, 1.0);
    let wide_row = shaper.row(&document, 1, 1.0);
    let wide = wide_row.caret_x(10, 1.0) - wide_row.caret_x(9, 1.0);
    assert!(
        wide > space + 1.0,
        "the fixture needs a glyph wider than a Latin cell, found {wide} against {space}"
    );
    let landed = wide_row.caret_x(cjk.head, 1.0);
    assert!(
        (landed - aim).abs() <= wide / 2.0 + 0.01 || cjk.head == 13,
        "the caret landed at {landed}, more than half a wide glyph from its aim {aim}"
    );
    // The tab line is entered at the nearer edge of its tab.
    let tab = shaper.row(&document, 2, 1.0);
    assert_eq!(
        tab.hit(space * 0.4, 1.0),
        14,
        "left of the tab's middle is before the tab"
    );
    assert_eq!(
        tab.hit(space * 1.6, 1.0),
        15,
        "right of the tab's middle is after the tab"
    );
    assert_eq!(
        travel(source, 6, &[1, 1, 1]).last().copied(),
        Some(23),
        "the column returns after the CJK and the tab line"
    );
}
