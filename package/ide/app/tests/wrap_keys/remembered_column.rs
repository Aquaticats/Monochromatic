//! The remembered aim of a vertical step:
//!  it survives rows that are shorter or indented,
//!  it returns the caret to its place,
//!  it lands at a short upper row's end,
//!  and it belongs to one side of a wrap offset.

use crate::fixtures::{
    SCALES,
    WIDTHS,
    advance,
    all_rows,
    at,
    caret_row,
    cells,
    exact_fit,
    rows_of,
    several_lines,
};
use ide_app::document::{
    Document,
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

/// The remembered x survives rows that are shorter or have an indent,
/// and the caret returns to its place.
#[test]
fn the_remembered_x_survives_short_rows_and_returns() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = several_lines(width, scale);
            let rows = all_rows(&text, width, scale);
            let start_head = rows[0].2.start + cells(width, scale) * 2 / 3;
            let mut shaper = TextShaper::new();
            let mut document = at(&text, start_head, ReadingAffinity::Downstream);
            let origin = vertical(&mut shaper, &document, scale, width, 0, None);
            let mut column: Option<PreferredColumn> = None;
            let mut visited = Vec::new();
            for step in 0..rows.len() - 1 {
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
                    step + 1,
                    "{label}: Down {step}"
                );
                visited.push((landed.head, landed.affinity));
            }
            let aim_kept = match column {
                Some(remembered) => (remembered.x - origin.x).abs() < 0.01,
                None => false,
            };
            assert!(aim_kept, "{label}: the aim never changed");
            for _ in 0..rows.len() - 1 {
                let landed = vertical(&mut shaper, &document, scale, width, -1, column);
                column = Some(landed);
                document.select(ReadingPosition {
                    anchor: landed.head,
                    head: landed.head,
                    viewport: 0,
                    affinity: landed.affinity,
                });
            }
            assert_eq!(
                (document.position().head, document.position().affinity),
                (start_head, ReadingAffinity::Downstream),
                "{label}: Up as often as Down returned to the start"
            );
            assert!(visited.len() >= 4, "{label}: the fixture has several rows");
        }
    }
}

/// A caret aiming past the end of a shorter upper row lands at that row's end,
/// drawn on the upper row.
#[test]
fn a_target_row_shorter_than_the_aim_ends_on_that_row() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let cells = cells(width, scale);
            // Row 0 is half full because the next word is too long for what is left; row 1 holds that word and
            // more, ending near the right edge.
            let text = format!(
                "{} {} {}",
                "x".repeat(cells / 2 - 1),
                "w".repeat(cells * 7 / 10),
                "y z".repeat(cells / 10)
            );
            let rows = rows_of(&text, 0, width, scale);
            assert!(rows.len() >= 2, "{label}: {rows:?}");
            let mut shaper = TextShaper::new();
            let document_all = Document::new(&text);
            let shaped = shaper.wrapped_row(&document_all, 0, scale, width);
            let far = shaped
                .caret_at(rows[1].end, ReadingAffinity::Upstream, scale)
                .x;
            let upper_end = shaped
                .caret_at(rows[0].end, ReadingAffinity::Upstream, scale)
                .x;
            assert!(
                far > upper_end + advance(scale),
                "{label}: the lower row reaches farther right: {far} against {upper_end}"
            );
            let from_lower = at(&text, rows[1].start + 1, ReadingAffinity::Downstream);
            let aim = PreferredColumn {
                head: rows[1].start + 1,
                affinity: ReadingAffinity::Downstream,
                x: far,
            };
            let landed = vertical(&mut shaper, &from_lower, scale, width, -1, Some(aim));
            assert_eq!(
                landed.head, rows[0].end,
                "{label}: lands at the end of the upper row"
            );
            assert_eq!(
                landed.affinity,
                ReadingAffinity::Upstream,
                "{label}: and stays on it"
            );
            let again = vertical(
                &mut shaper,
                &at(&text, landed.head, landed.affinity),
                scale,
                width,
                1,
                Some(landed),
            );
            assert_eq!(again.x, far, "{label}: the aim is kept");
            assert!(
                again.head > rows[1].start,
                "{label}: Down returns to the lower row at the aim"
            );
        }
    }
}

/// A remembered column belongs to one side of a wrap offset:
///  the end of an upper row and the start of the next row are the same position,
/// but not the same place.
#[test]
fn a_remembered_column_belongs_to_one_side_of_a_wrap_offset() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = exact_fit(width, scale);
            let rows = rows_of(&text, 0, width, scale);
            let wrap = rows[0].end;
            let mut shaper = TextShaper::new();
            let at_row_end = at(&text, wrap, ReadingAffinity::Upstream);
            let other_side = PreferredColumn {
                head: wrap,
                affinity: ReadingAffinity::Downstream,
                x: 0.0,
            };
            let ignored = vertical(&mut shaper, &at_row_end, scale, width, 1, Some(other_side));
            assert!(
                ignored.head > rows[1].start + 3,
                "{label}: a column remembered for the start of the lower row moved a caret at the end of the upper row"
            );
            let same_side = PreferredColumn {
                head: wrap,
                affinity: ReadingAffinity::Upstream,
                x: 0.0,
            };
            let used = vertical(&mut shaper, &at_row_end, scale, width, 1, Some(same_side));
            assert!(
                used.head <= rows[1].start + 1,
                "{label}: a column remembered for this side must be used, landed at {}",
                used.head
            );
        }
    }
}
