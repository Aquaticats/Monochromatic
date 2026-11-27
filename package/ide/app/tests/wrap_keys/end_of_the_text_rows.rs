//! The rows at the end of the text:
//!  a blank that overflows the last row hangs on it,
//!  the empty line the engine adds after it is no row,
//!  and the caret at the end of the text is on the last real row at its right edge.

use crate::fixtures::{
    SCALES,
    WIDTHS,
    assert_tiles,
    cells,
};
use ide_app::document::{
    Document,
    ReadingAffinity,
};
/// The height of one code row,
///  which places a point below the rows.
use ide_app::row_map::CODE_ROW;
/// Shaper and the rows it produces.
use ide_app::shaped_text::TextShaper;

/// A blank that overflows the last row hangs on it,
/// and the empty line the engine adds after it is no row:
///  the rows tile the text,
/// the caret at the end of the text is on the last real row at its right edge.
#[test]
fn a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row() {
    let mut tails = 0;
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let fit = cells(width, scale);
            for text in [
                format!("{} ", "m".repeat(fit)),
                format!("{} ", "m".repeat(fit * 2 - 1)),
                format!("{}   ", "m".repeat(fit * 3 - 2)),
            ] {
                let document = Document::new(&text);
                let mut shaper = TextShaper::new();
                let shaped = shaper.wrapped_row(&document, 0, scale, width);
                tails += usize::from(shaped.has_empty_tail());
                let rows = shaped.source_rows();
                assert_eq!(
                    rows.len(),
                    shaped.visual_rows(),
                    "{label}: every counted row has a range"
                );
                assert_tiles(&rows, text.chars().count(), &label);
                let mut none_empty = true;
                for row in rows.iter() {
                    if row.is_empty() {
                        none_empty = false;
                    }
                }
                assert!(
                    none_empty,
                    "{label}: no counted row is empty, rows {rows:?}"
                );
                let end = text.chars().count();
                for affinity in [ReadingAffinity::Downstream, ReadingAffinity::Upstream] {
                    let caret = shaped.caret_at(end, affinity, scale);
                    assert_eq!(
                        caret.row,
                        rows.len() - 1,
                        "{label}: the end of the text is on the last counted row ({affinity:?})"
                    );
                    // With an empty tail the end of the text is the hanging blank at the right edge of the last row.
                    assert!(
                        !shaped.has_empty_tail() || caret.x > width / 2.0,
                        "{label}: and at the right of its text, found x {} ({affinity:?})",
                        caret.x
                    );
                }
                let landed = shaped.hit_at(10_000.0, CODE_ROW * rows.len() as f32 + 100.0, scale);
                assert_eq!(landed, end, "{label}: a point below the rows is at the end");
            }
        }
    }
    assert!(
        tails > 0,
        "no fixture produced an empty tail, so the test proves nothing about it"
    );
}
