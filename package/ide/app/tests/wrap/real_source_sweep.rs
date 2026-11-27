//! Every row of this package's own source at six width and scale pairs:
//!  the lines are read from the crate's sources,
//!  shaped at each pair,
//!  and held to the coverage the sweep records,
//!  with the breaks that are allowed counted per run.

use crate::fixtures::{
    boundaries_are_break_opportunities,
    rows_fit_width,
    rows_start_on_graphemes,
    rows_tile_the_text,
    space_advance,
};
use ide_app::{
    document::Document,
    shaped_row::ShapedRow,
    shaped_text::TextShaper,
    wrap::{
        continuation_indent,
        intellij_break,
    },
};

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
fn token_width_around(
    text: &str,
    byte: usize,
    scale: f32,
) -> f32 {
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
fn unexcused_breaks(
    row: &ShapedRow,
    width: f32,
    scale: f32,
    space: f32,
) -> Vec<(char, char)> {
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
fn shape_source(
    scale: f32,
    width: f32,
) -> SourceRun {
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
fn assert_real_source_fits(
    scale: f32,
    width: f32,
    coverage: Coverage,
) {
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
