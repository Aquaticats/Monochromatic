//! The break rules and the rows they make:
//!  a long ASCII line at several widths,
//!  a line that fits keeping one row,
//!  a token without break opportunities,
//!  CJK between ideographs and emoji that never split,
//!  the continuation indent,
//!  tabs that wrap and end on their stops,
//!  an empty line, and width zero.

use crate::fixtures::{
    LONG_LINE,
    boundaries_are_break_opportunities,
    row_of_byte,
    rows_fit_width,
    rows_start_on_graphemes,
    rows_tile_the_text,
    space_advance,
    wrapped,
};
use ide_app::{
    document::ReadingAffinity,
    wrap::continuation_indent,
};

/// Long ASCII line at several widths:
/// each row fits,
/// every break is a rule opportunity,
/// and no grapheme splits.
#[test]
fn long_ascii_line_wraps_at_several_widths() {
    // Widths below the longest token would break inside it, which the rule check does not describe.
    for width in [300.0, 160.0] {
        let row = wrapped(LONG_LINE, width, 1.0);
        assert!(row.visual_rows() > 1, "width {width} should wrap");
        assert!(rows_tile_the_text(&row), "rows tile at width {width}");
        assert!(
            rows_fit_width(&row, width, 1.0),
            "rows fit at width {width}"
        );
        assert!(
            boundaries_are_break_opportunities(&row),
            "breaks follow the rules at width {width}"
        );
        assert!(
            rows_start_on_graphemes(&row),
            "no grapheme split at width {width}"
        );
    }
    let narrow = wrapped(LONG_LINE, 90.0, 1.0);
    assert!(rows_tile_the_text(&narrow), "rows tile at width 90");
    assert!(
        rows_start_on_graphemes(&narrow),
        "no grapheme split at width 90"
    );
}

/// A line that fits the width keeps one visual row,
/// and a line exactly the width keeps one too.
#[test]
fn a_line_that_fits_keeps_one_row() {
    let text = "let value = 1;";
    let unwrapped = wrapped(text, 10_000.0, 1.0);
    assert_eq!(unwrapped.visual_rows(), 1);
    let exact = unwrapped.caret_x(text.chars().count(), 1.0);
    let row = wrapped(text, exact, 1.0);
    assert_eq!(
        row.visual_rows(),
        1,
        "a line exactly the width does not wrap"
    );
}

/// A 200 character token with no break opportunity breaks where the width is exceeded.
#[test]
fn a_token_without_break_opportunities_breaks_at_the_width() {
    let token = "a".repeat(200);
    let width = 100.0;
    let row = wrapped(&token, width, 1.0);
    assert!(row.visual_rows() >= 2, "a 200 character token wraps");
    assert!(rows_tile_the_text(&row));
    assert!(rows_fit_width(&row, width, 1.0), "each row fits the width");
    assert!(rows_start_on_graphemes(&row));
}

/// CJK ideographs break between each other,
/// and every break is an opportunity under the rules.
#[test]
fn cjk_text_breaks_between_ideographs() {
    let text = "中文字符测试".repeat(12);
    let row = wrapped(&text, 120.0, 1.0);
    assert!(row.visual_rows() > 1, "CJK text wraps");
    assert!(rows_tile_the_text(&row));
    assert!(
        boundaries_are_break_opportunities(&row),
        "CJK boundaries are opportunities"
    );
    assert!(rows_start_on_graphemes(&row));
}

/// Emoji with zero width joiners,
/// skin tones,
/// flags,
/// and combining marks never split.
#[test]
fn emoji_sequences_and_combining_marks_never_split() {
    let unit = "👩\u{200D}💻 e\u{301} 🇫🇷 👍🏽 x\u{FE0F}🙂 ";
    let text = unit.repeat(6);
    for width in [40.0, 90.0, 1.0] {
        let row = wrapped(&text, width, 1.0);
        assert!(rows_tile_the_text(&row), "rows tile at width {width}");
        assert!(rows_start_on_graphemes(&row), "no split at width {width}");
    }
}

/// Leading indentation and tabs:
/// the continuation row starts at the first non-blank column plus one mark.
#[test]
fn continuation_row_starts_at_the_indent() {
    let text = "    let total = first_value + second_value + third_value + fourth_value + fifth;";
    for scale in [1.0, 2.0] {
        let row = wrapped(text, 220.0, scale);
        assert!(row.visual_rows() > 1, "the line wraps");
        let space = space_advance(scale);
        let expected = continuation_indent(text, space);
        let start = row.visual_row_sources(1).expect("second row").start;
        let caret = row.caret_at(start, ReadingAffinity::Downstream, scale);
        assert_eq!(caret.row, 1);
        assert!(
            (caret.x - expected).abs() < 0.01,
            "scale {scale}: the continuation row starts at {} where the indent is {}",
            caret.x,
            expected
        );
    }
}

/// Tab-bearing lines wrap,
/// and the tiling holds with tabs in the rows.
#[test]
fn tab_bearing_lines_wrap_and_tile() {
    let text = "\tfn compute(\tvalue: u32) -> u32 {\t\tlet total = value * 2 + value * 3 + value * 5;\t\treturn total;";
    for width in [260.0, 140.0] {
        let row = wrapped(text, width, 1.0);
        assert!(row.visual_rows() > 1, "width {width} wraps");
        assert!(rows_tile_the_text(&row), "tiling holds at width {width}");
        assert!(rows_start_on_graphemes(&row));
    }
}

/// Tab stops on a wrapped row end on stops of the layout's own grid,
/// measured from its origin.
#[test]
fn tabs_on_a_wrapped_row_end_on_their_stops() {
    let text = "a\tb\tc\td\te\tf\tg\th\ti\tj\tk\tl\tm\tn\to\tp\tq\tr\ts\tt\tu\tv\tw\tx\ty\tz";
    let scale = 1.0;
    let row = wrapped(text, 150.0, scale);
    assert!(row.visual_rows() > 1, "the tabbed line wraps");
    let stop = space_advance(scale) * 2.0;
    for byte in row.projection.tabs.clone() {
        let source_after = row.projection.byte_to_source[byte + 1];
        let end = row.caret_at(
            row.source_start + source_after,
            ReadingAffinity::Downstream,
            scale,
        );
        // A tab that ends a visual row leaves the next row at its indent, so only tabs ending on their own row have a stop to check.
        let tab_row = row_of_byte(&row, byte);
        if end.row != tab_row {
            continue;
        }
        let remainder = (end.x / stop).fract();
        assert!(
            remainder.abs() < 0.02 || (1.0 - remainder).abs() < 0.02,
            "tab ends off the stop: {}",
            end.x
        );
    }
}

/// An empty line and a line with only a newline both have one visual row and no bytes.
#[test]
fn an_empty_line_has_one_row() {
    let empty = wrapped("", 100.0, 1.0);
    assert_eq!(empty.visual_rows(), 1);
    assert_eq!(empty.visual_row_bytes(0), Some(0..0));
    let terminated = wrapped("abc\n", 100.0, 1.0);
    assert_eq!(
        terminated.visual_rows(),
        1,
        "the terminator is not a visual row"
    );
    assert_eq!(terminated.projection.text, "abc");
}

/// Width zero,
/// a negative width,
/// and a non-finite width mean no wrapping,
/// and a tiny width wraps per cluster.
#[test]
fn width_zero_means_no_wrapping_and_tiny_width_wraps_per_cluster() {
    for width in [0.0, -4.0, f32::NAN, f32::INFINITY] {
        let row = wrapped(LONG_LINE, width, 1.0);
        assert_eq!(
            row.visual_rows(),
            1,
            "width {width} keeps the unbroken line"
        );
    }
    let tiny = wrapped("abcd", 0.5, 1.0);
    assert_eq!(
        tiny.visual_rows(),
        4,
        "a tiny width puts each character on its own row"
    );
    assert!(rows_tile_the_text(&tiny));
}
