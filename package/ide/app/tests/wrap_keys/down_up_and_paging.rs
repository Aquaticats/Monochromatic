//! Down and Up over the visual rows of wrapped lines,
//!  and a page step that crosses lines and clamps at the first and last row.

use crate::fixtures::{
    SCALES,
    WIDTHS,
    all_rows,
    at,
    caret_row,
    several_lines,
};
use ide_app::document::{
    ReadingAffinity,
    ReadingPosition,
};
/// Shaper and the rows it produces.
use ide_app::shaped_text::TextShaper;
/// The remembered column and the vertical step under test.
use ide_app::vertical_motion::{
    PreferredColumn,
    vertical,
};

/// Down and Up from the start of the text visit the start of every visual row in turn,
/// whatever the line is.
#[test]
fn down_and_up_visit_every_visual_row() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = several_lines(width, scale);
            let rows = all_rows(&text, width, scale);
            let mut shaper = TextShaper::new();
            let mut document = at(&text, 0, ReadingAffinity::Downstream);
            let mut column: Option<PreferredColumn> = None;
            for (index, expected) in rows.iter().enumerate().skip(1) {
                let landed = vertical(&mut shaper, &document, scale, width, 1, column);
                column = Some(landed);
                document.select(ReadingPosition {
                    anchor: landed.head,
                    head: landed.head,
                    viewport: 0,
                    affinity: landed.affinity,
                });
                assert_eq!(
                    caret_row(&document, width, scale),
                    index,
                    "{label}: Down {index}"
                );
                assert_eq!(
                    landed.head, expected.2.start,
                    "{label}: Down {index} lands at the row start"
                );
            }
            // Down from the last row goes to the end of the text.
            let end_of_text = vertical(&mut shaper, &document, scale, width, 1, column);
            assert_eq!(end_of_text.head, text.chars().count(), "{label}");
            for index in (0..rows.len() - 1).rev() {
                let landed = vertical(&mut shaper, &document, scale, width, -1, column);
                column = Some(landed);
                document.select(ReadingPosition {
                    anchor: landed.head,
                    head: landed.head,
                    viewport: 0,
                    affinity: landed.affinity,
                });
                assert_eq!(
                    caret_row(&document, width, scale),
                    index,
                    "{label}: Up to {index}"
                );
                assert_eq!(
                    landed.head, rows[index].2.start,
                    "{label}: Up to {index} lands at the row start"
                );
            }
            let to_start = vertical(&mut shaper, &document, scale, width, -1, column);
            assert_eq!(
                to_start.head, 0,
                "{label}: Up from the first row goes to the start"
            );
        }
    }
}

/// A step of many rows crosses lines of different heights and stops on the last row,
/// and on the first.
#[test]
fn a_page_of_rows_crosses_wrapped_lines_and_clamps() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = several_lines(width, scale);
            let rows = all_rows(&text, width, scale);
            let mut shaper = TextShaper::new();
            let document = at(&text, 0, ReadingAffinity::Downstream);
            for count in [1_usize, 3, 5, 7] {
                if count >= rows.len() {
                    continue;
                }
                let landed = vertical(&mut shaper, &document, scale, width, count as isize, None);
                assert_eq!(
                    landed.head, rows[count].2.start,
                    "{label}: {count} rows down"
                );
            }
            let beyond = vertical(&mut shaper, &document, scale, width, 1000, None);
            let last = &rows[rows.len() - 1];
            assert!(
                beyond.head >= last.2.start && beyond.head <= last.2.end,
                "{label}: a page beyond the end lands on the last row"
            );
            let bottom = at(&text, last.2.start, ReadingAffinity::Downstream);
            let up = vertical(&mut shaper, &bottom, scale, width, -1000, None);
            let landed = at(&text, up.head, up.affinity);
            assert_eq!(
                caret_row(&landed, width, scale),
                0,
                "{label}: a page beyond the start lands on the first row"
            );
        }
    }
}
