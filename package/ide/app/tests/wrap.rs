//! Soft-wrap shaping and geometry:
//! break opportunities,
//! grapheme safety,
//! tiling,
//! indent,
//! and visual-row geometry.
//! No window is opened;
//! every test shapes lines with the same Parley setup the source view uses.

/// Rope and grapheme boundaries from the editor's own text model,
/// so grapheme checks match editing.
use helix_core::{Rope, graphemes::next_grapheme_boundary};
/// Canonical source of the lines under test.
use ide_app::document::{Document, ReadingAffinity};
/// The row map's code row height,
/// the step between visual rows.
use ide_app::row_map::CODE_ROW;
use ide_app::shaped_row::ShapedRow;
/// Shaper and shaped row under test.
use ide_app::shaped_text::TextShaper;
/// The break predicate under test,
/// and the continuation indent it is measured against.
use ide_app::wrap::{continuation_indent, intellij_break};

/// Text used for the long-line table:
/// words and punctuation with ordinary break opportunities.
const LONG_LINE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going, \
     and going, and going past every width in the table";

/// Shape row 0 of `text` at a logical wrap width and scale.
fn wrapped(text: &str, width: f32, scale: f32) -> ShapedRow {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    return shaper.wrapped_row(&document, 0, scale, width);
}

/// Advance of one space in logical pixels at `scale`,
/// measured on an unwrapped space.
fn space_advance(scale: f32) -> f32 {
    let document = Document::new(" ");
    let mut shaper = TextShaper::new();
    return shaper.row(&document, 0, scale).caret_x(1, scale);
}

/// Char indices where a grapheme cluster starts in `text`,
/// including the end of the text.
fn grapheme_starts(text: &str) -> Vec<usize> {
    let rope = Rope::from(text);
    let slice = rope.slice(..);
    let mut starts = vec![0];
    let mut at = 0;
    while at < rope.len_chars() {
        at = next_grapheme_boundary(slice, at);
        starts.push(at);
    }
    return starts;
}

/// Char index of byte `byte` in `text`.
fn char_index(text: &str, byte: usize) -> usize {
    return text[..byte].chars().count();
}

/// Whether every visual row begins on a grapheme cluster boundary.
fn rows_start_on_graphemes(row: &ShapedRow) -> bool {
    let text = row.projection.text.as_str();
    let starts = grapheme_starts(text);
    for index in 1..row.visual_rows() {
        let byte = row.visual_row_bytes(index).expect("row exists").start;
        if !starts.contains(&char_index(text, byte)) {
            return false;
        }
    }
    return true;
}

/// Whether the display byte ranges of all visual rows tile the whole display text,
/// in order.
fn rows_tile_the_text(row: &ShapedRow) -> bool {
    let mut next = 0;
    for index in 0..row.visual_rows() {
        let bytes = row.visual_row_bytes(index).expect("row exists");
        if bytes.start != next {
            return false;
        }
        next = bytes.end;
    }
    return next == row.projection.text.len();
}

/// Whether each visual row's visible advance fits the width,
/// which is the rule `OverflowWrap::Anywhere` keeps
///  only for a row that holds a single cluster.
fn rows_fit_width(row: &ShapedRow, width: f32, scale: f32) -> bool {
    for line in row.layout.lines() {
        let metrics = line.metrics();
        let visible = (metrics.advance - metrics.trailing_whitespace) / scale;
        if visible > width + 0.01 && line.text_range().len() > 1 {
            return false;
        }
    }
    return true;
}

/// Each row boundary of a wrapped line must be an IntelliJ break opportunity.
fn boundaries_are_break_opportunities(row: &ShapedRow) -> bool {
    let text = row.projection.text.as_str();
    for index in 1..row.visual_rows() {
        let byte = row.visual_row_bytes(index).expect("row exists").start;
        let before = text[..byte].chars().last().expect("text before a boundary");
        let after = text[byte..].chars().next().expect("text after a boundary");
        if !intellij_break(before, after) {
            return false;
        }
    }
    return true;
}

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

/// Visual rows and geometry at both scales:
/// the row step is one code row in logical pixels.
#[test]
fn row_steps_are_one_code_row_at_both_scales() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        assert_eq!(row.visual_row_offset(0, scale), 0.0);
        for index in 1..row.visual_rows() {
            let offset = row.visual_row_offset(index, scale);
            assert!(
                (offset - CODE_ROW * index as f32).abs() < 0.01,
                "row {index} at scale {scale}: {offset}"
            );
        }
    }
}

/// A caret at a soft-wrap offset is drawn at the end of the upper row upstream,
/// and at the start of the lower row downstream.
#[test]
fn caret_at_a_wrap_offset_follows_its_affinity() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        let position = row.source_start + row.visual_row_sources(1).expect("second row").start;
        let upstream = row.caret_at(position, ReadingAffinity::Upstream, scale);
        let downstream = row.caret_at(position, ReadingAffinity::Downstream, scale);
        assert_eq!(
            upstream.row, 0,
            "upstream draws on the upper row at scale {scale}"
        );
        assert_eq!(
            downstream.row, 1,
            "downstream draws on the lower row at scale {scale}"
        );
        assert!(
            upstream.x > downstream.x,
            "upstream is at the end of the upper row at scale {scale}"
        );
        assert_eq!(
            row.caret_x(position, scale),
            downstream.x,
            "caret_x keeps its downstream meaning"
        );
    }
}

/// A point on the lower row hits a source character on that row,
/// and a point on the upper row hits one before the break.
#[test]
fn hit_probes_the_visual_row_under_the_point() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        assert!(row.visual_rows() > 1);
        let boundary = row.source_start + row.visual_row_sources(1).expect("second row").start;
        let lower = row.hit_at(0.0, CODE_ROW * 1.5, scale);
        assert!(
            lower >= boundary,
            "lower row hit {lower} at scale {scale} is on the lower row, from {boundary}"
        );
        let upper = row.hit_at(0.0, CODE_ROW * 0.5, scale);
        assert_eq!(
            upper, row.source_start,
            "upper row hit at its start at scale {scale}"
        );
    }
}

/// `hit` without a row offset probes the first visual row,
/// and an unwrapped row keeps its probe.
#[test]
fn hit_without_a_row_offset_probes_the_first_row() {
    let scale = 1.0;
    let row = wrapped(LONG_LINE, 160.0, scale);
    let x = 40.0;
    assert_eq!(row.hit(x, scale), row.hit_at(x, CODE_ROW * 0.5, scale));
}

/// A range that crosses rows yields one rectangle per visual row,
/// each on that row's y and one code row tall.
#[test]
fn range_yields_one_rectangle_per_visual_row() {
    for scale in [1.0, 2.0] {
        let row = wrapped(LONG_LINE, 160.0, scale);
        let rectangles = row.range(row.source_start, row.source_start + row.source_len(), scale);
        assert!(
            rectangles.len() >= row.visual_rows(),
            "one rectangle per row at scale {scale}"
        );
        for rectangle in &rectangles {
            assert!((rectangle.height - CODE_ROW).abs() < 0.001);
        }
        let mut ys: Vec<f32> = Vec::new();
        for rectangle in rectangles.iter() {
            ys.push(rectangle.y);
        }
        for index in 0..row.visual_rows() {
            let expected = row.visual_row_offset(index, scale);
            let mut found = false;
            for y in ys.iter() {
                if (*y - expected).abs() < 0.001 {
                    found = true;
                }
            }
            assert!(found, "row {index} has a rectangle at {expected}");
        }
    }
}

/// The first row's rectangle keeps the row's own top exactly,
/// as unwrapped rows always did.
#[test]
fn range_on_the_first_row_keeps_the_top_exactly() {
    let row = wrapped(LONG_LINE, 160.0, 1.0);
    let rectangles = row.range(row.source_start, row.source_start + 4, 1.0);
    assert!(!rectangles.is_empty());
    assert_eq!(rectangles[0].y, row.top);
}

/// Visual row that holds display byte `byte`,
/// found from the row byte ranges.
fn row_of_byte(row: &ShapedRow, byte: usize) -> usize {
    for index in 0..row.visual_rows() {
        let bytes = row.visual_row_bytes(index).expect("row exists");
        if bytes.start <= byte && byte < bytes.end {
            return index;
        }
    }
    return row.visual_rows().saturating_sub(1);
}

/// Lines of the crate's own Rust source:
/// real code with ordinary identifiers,
/// comments,
/// strings,
/// and long lines.
fn source_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .expect("the source folder is readable")
    {
        let path = entry.expect("a directory entry").path();
        if let Some(extension) = path.extension()
            && extension == "rs"
        {
            files.push(path);
        }
    }
    files.sort();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("a source file is readable");
        for line in text.lines() {
            if !line.trim().is_empty() {
                lines.push(line.to_string());
            }
        }
    }
    return lines;
}

/// Logical width of the unbroken token around `byte`:
/// the stretch between the nearest rule opportunities on each side.
fn token_width_around(text: &str, byte: usize, scale: f32) -> f32 {
    let characters: Vec<(usize, char)> = text.char_indices().collect();
    let mut start = 0;
    let mut end = text.len();
    for pair in characters.windows(2) {
        let boundary = pair[1].0;
        if intellij_break(pair[0].1, pair[1].1) {
            if boundary <= byte {
                start = boundary;
            } else {
                end = boundary;
                break;
            }
        }
    }
    let token = &text[start..end];
    let document = Document::new(token);
    let mut shaper = TextShaper::new();
    return shaper
        .row(&document, 0, scale)
        .caret_x(token.chars().count(), scale);
}

/// Row boundaries that are neither a rule opportunity nor inside a token wider than the width a row has.
///
/// A token with no opportunity breaks where the width is exceeded,
/// so a boundary inside one is expected.
///
/// `space` is the advance of one space at `scale`,
/// measured once by the caller:
/// measuring it here built a new shaper for every line of the crate.
fn unexcused_breaks(row: &ShapedRow, width: f32, scale: f32, space: f32) -> Vec<(char, char)> {
    let text = row.projection.text.as_str();
    let indent = continuation_indent(text, space);
    let mut unexcused = Vec::new();
    for index in 1..row.visual_rows() {
        let byte = row.visual_row_bytes(index).expect("row exists").start;
        let before = text[..byte].chars().last().expect("text before a boundary");
        let after = text[byte..].chars().next().expect("text after a boundary");
        if !intellij_break(before, after) && token_width_around(text, byte, scale) <= width - indent
        {
            unexcused.push((before, after));
        }
    }
    return unexcused;
}

/// What shaping every source line of the crate at one scale and width found.
struct SourceRun {
    /// Visual rows over all lines.
    rows: usize,
    /// Lines that wrapped onto more than one row.
    wrapped_lines: usize,
    /// Lines with a row boundary that is not a rule opportunity (a token wider than a row broke inside).
    inside_tokens: usize,
    /// One description for every line that broke a rule.
    failures: Vec<String>,
}

/// Shape every source line at `scale` and `width` and judge each row.
///
/// Every row tiles the text,
/// every row's visible advance fits the width,
/// every boundary is a rule opportunity or inside a token wider than a row,
/// and no row starts inside a grapheme.
fn shape_source(scale: f32, width: f32) -> SourceRun {
    let lines = source_lines();
    assert!(lines.len() > 1000, "the crate has source lines to shape");
    let mut run = SourceRun {
        rows: 0,
        wrapped_lines: 0,
        inside_tokens: 0,
        failures: Vec::new(),
    };
    let mut shaper = TextShaper::new();
    let space = space_advance(scale);
    for (index, text) in lines.iter().enumerate() {
        let document = Document::new(text);
        let row = shaper.wrapped_row(&document, 0, scale, width);
        run.rows += row.visual_rows();
        if row.visual_rows() > 1 {
            run.wrapped_lines += 1;
        }
        if !boundaries_are_break_opportunities(&row) {
            run.inside_tokens += 1;
        }
        let unexcused = unexcused_breaks(&row, width, scale, space);
        if !(rows_tile_the_text(&row)
            && rows_fit_width(&row, width, scale)
            && unexcused.is_empty()
            && rows_start_on_graphemes(&row))
        {
            run.failures.push(format!(
                "line {index} width {width} scale {scale} unexcused {unexcused:?}: {text}"
            ));
        }
    }
    return run;
}

/// What the corpus must show at a width,
/// so that the rule checks cannot pass without wrapping anything.
struct Coverage {
    /// Fewer lines than this wrapping is a failure.
    wrapped_lines_over: usize,
    /// Whether some line must break inside a token wider than a row,
    /// which exercises the excuse for such tokens.
    token_excuse_exercised: bool,
}

/// Coverage at width 600:
/// more than 100 lines wrap and a token wider than a row exists.
///
/// These are the thresholds of the former single test,
/// which asserted them over six scale and width pairs together;
/// each width 600 test asserts them alone,
/// which implies the totals.
const COVERAGE_AT_600: Coverage = Coverage {
    wrapped_lines_over: 100,
    token_excuse_exercised: true,
};

/// Coverage at width 900:
/// more than 100 lines wrap (347 do on 2026-10-18),
/// and no line breaks inside a token.
const COVERAGE_AT_900: Coverage = Coverage {
    wrapped_lines_over: 100,
    token_excuse_exercised: false,
};

/// Coverage at width 1400:
/// some lines wrap (14 do on 2026-10-18),
/// and no line breaks inside a token.
const COVERAGE_AT_1400: Coverage = Coverage {
    wrapped_lines_over: 0,
    token_excuse_exercised: false,
};

/// The checks of one scale and width over the crate's own source:
/// the coverage holds,
/// and no row breaks a rule.
fn assert_real_source_fits(scale: f32, width: f32, coverage: Coverage) {
    let run = shape_source(scale, width);
    assert!(
        run.wrapped_lines > coverage.wrapped_lines_over,
        "some lines wrap at this width: {}",
        run.wrapped_lines
    );
    assert!(
        !coverage.token_excuse_exercised || run.inside_tokens > 0,
        "the corpus has a token wider than a row, so the excuse is exercised"
    );
    assert!(
        run.failures.is_empty(),
        "{} of {} rows broke a rule, first: {:?}",
        run.failures.len(),
        run.rows,
        &run.failures[..run.failures.len().min(3)]
    );
}

/// Real source at the widths and scales the window uses:
/// every row's visible advance fits the width,
/// every row tiles
/// the text,
/// every boundary is a rule opportunity or inside a token wider than a row,
/// and no row starts inside a
/// grapheme.
/// An earlier measurement counted rows whose raw advance passed the width (966 of 57,938 at width 600);
/// the advance includes a hanging space,
/// so this test measures the visible advance.
///
/// One test per scale and width,
/// so the pairs run in parallel and a failure names its case.
#[test]
fn every_row_of_real_source_fits_width_600_at_scale_1() {
    assert_real_source_fits(1.0, 600.0, COVERAGE_AT_600);
}

/// Real source at width 900 and scale 1:
/// see `every_row_of_real_source_fits_width_600_at_scale_1`.
#[test]
fn every_row_of_real_source_fits_width_900_at_scale_1() {
    assert_real_source_fits(1.0, 900.0, COVERAGE_AT_900);
}

/// Real source at width 1400 and scale 1:
/// see `every_row_of_real_source_fits_width_600_at_scale_1`.
#[test]
fn every_row_of_real_source_fits_width_1400_at_scale_1() {
    assert_real_source_fits(1.0, 1400.0, COVERAGE_AT_1400);
}

/// Real source at width 600 and scale 2:
/// see `every_row_of_real_source_fits_width_600_at_scale_1`.
#[test]
fn every_row_of_real_source_fits_width_600_at_scale_2() {
    assert_real_source_fits(2.0, 600.0, COVERAGE_AT_600);
}

/// Real source at width 900 and scale 2:
/// see `every_row_of_real_source_fits_width_600_at_scale_1`.
#[test]
fn every_row_of_real_source_fits_width_900_at_scale_2() {
    assert_real_source_fits(2.0, 900.0, COVERAGE_AT_900);
}

/// Real source at width 1400 and scale 2:
/// see `every_row_of_real_source_fits_width_600_at_scale_1`.
#[test]
fn every_row_of_real_source_fits_width_1400_at_scale_2() {
    assert_real_source_fits(2.0, 1400.0, COVERAGE_AT_1400);
}
