//! The four line shapes every row test wraps,
//!  and the helpers that shape them:
//!  a document whose caret sits at a place,
//!  the advance of one glyph,
//!  the glyphs that fill a row,
//!  a line's visual rows,
//!  and a line's rows checked to tile its text.
//! The fixture test below proves each shape wraps the way its builder's name says.

use ide_app::document::{
    Document,
    ReadingAffinity,
    ReadingPosition,
};
/// The pure row lookup,
///  used to say which visual row holds a caret.
use ide_app::line_boundary::{
    CaretPlace,
    row_containing,
};
/// Shaper and the rows it produces.
use ide_app::shaped_text::TextShaper;
/// Source character ranges of visual rows.
use std::ops::Range;

/// Wrap widths in logical pixels:
/// a narrow pane,
/// a middle one,
/// a wide one.
pub const WIDTHS: [f32; 3] = [240.0, 410.0, 700.0];
/// Display scales the movements are checked at.
pub const SCALES: [f32; 2] = [1.0, 2.0];

/// A document whose caret is at `head` on `affinity`,
/// with no selection.
pub fn at(
    text: &str,
    head: usize,
    affinity: ReadingAffinity,
) -> Document {
    let mut document = Document::new(text);
    document.select(ReadingPosition {
        anchor: head,
        head,
        viewport: 0,
        affinity,
    });
    return document;
}

/// Advance of one glyph of the source font in logical pixels at `scale`.
pub fn advance(scale: f32) -> f32 {
    let document = Document::new("mm");
    let mut shaper = TextShaper::new();
    return shaper.row(&document, 0, scale).caret_x(1, scale);
}

/// How many glyphs of the source font fill a row of `width` logical pixels.
pub fn cells(
    width: f32,
    scale: f32,
) -> usize {
    return (width / advance(scale)).floor() as usize;
}

/// Visual rows of line `line` of `text` as global character ranges,
/// shaped at `width` and `scale`.
pub fn rows_of(
    text: &str,
    line: usize,
    width: f32,
    scale: f32,
) -> Vec<Range<usize>> {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    return shaper
        .wrapped_row(&document, line, scale, width)
        .source_rows();
}

/// An indented line of words that wraps to several rows.
pub fn prose(
    width: f32,
    scale: f32,
) -> String {
    let words = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau";
    let mut text = String::from("    ");
    while text.chars().count() < cells(width, scale) * 5 {
        text.push_str(words);
        text.push(' ');
    }
    return text.trim_end().to_string();
}

/// A line whose second row starts with a tab:
/// a first row filled by an unbreakable token,
/// a blank that hangs on it
///  (an overflowing blank stays on the upper row,
/// so a tab right after it starts the next row),
/// then the tab.
pub fn tab_starts_a_row(
    width: f32,
    scale: f32,
) -> String {
    return format!("{} \ttail words", "m".repeat(cells(width, scale)));
}

/// A line whose first row ends with a tab that fills it exactly,
/// then a token that fills the next row.
pub fn tab_ends_a_row(
    width: f32,
    scale: f32,
) -> String {
    return format!(
        "{}\t{} more words",
        "m".repeat(cells(width, scale) - 1),
        "z".repeat(cells(width, scale))
    );
}

/// A line of an unbreakable token whose rows all fit the width exactly:
/// the first row holds `cells` glyphs and each
///  continuation row one less,
/// because of the indent that stands for the wrap mark.
pub fn exact_fit(
    width: f32,
    scale: f32,
) -> String {
    return "m".repeat(cells(width, scale) * 3 - 2);
}

/// Every row range of a line tiles its text:
/// each starts where the one before ended,
/// the last ends at the line end.
pub fn assert_tiles(
    rows: &[Range<usize>],
    line_end: usize,
    label: &str,
) {
    assert_eq!(rows[0].start, 0, "{label}: the first row starts the line");
    for pair in rows.windows(2) {
        assert_eq!(pair[0].end, pair[1].start, "{label}: rows tile");
    }
    assert_eq!(
        rows.last().expect("a row").end,
        line_end,
        "{label}: the last row ends the line"
    );
}

/// The fixtures wrap the way the tests below need,
/// at every width and scale.
#[test]
fn the_fixtures_wrap_as_described() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = prose(width, scale);
            let rows = rows_of(&text, 0, width, scale);
            assert!(
                rows.len() >= 4,
                "{label}: prose wraps to {} rows",
                rows.len()
            );
            assert_tiles(&rows, text.chars().count(), &label);
            let tabbed = tab_starts_a_row(width, scale);
            let tab_rows = rows_of(&tabbed, 0, width, scale);
            let mut continuation_starts_with_tab = false;
            for row in tab_rows.iter().skip(1) {
                if tabbed.chars().nth(row.start) == Some('\t') {
                    continuation_starts_with_tab = true;
                }
            }
            assert!(
                continuation_starts_with_tab,
                "{label}: some continuation row starts with a tab, rows {tab_rows:?}"
            );
            let ending = tab_ends_a_row(width, scale);
            let ending_rows = rows_of(&ending, 0, width, scale);
            let mut upper_row_ends_with_tab = false;
            for row in ending_rows.iter().take(ending_rows.len() - 1) {
                if ending.chars().nth(row.end - 1) == Some('\t') {
                    upper_row_ends_with_tab = true;
                }
            }
            assert!(
                upper_row_ends_with_tab,
                "{label}: some upper row ends with a tab, rows {ending_rows:?}"
            );
            let exact = exact_fit(width, scale);
            let exact_rows = rows_of(&exact, 0, width, scale);
            assert_eq!(
                exact_rows.len(),
                3,
                "{label}: an exact-fit token takes three rows, cells {}, rows {exact_rows:?}, advance {}",
                cells(width, scale),
                advance(scale)
            );
            let mut lengths: Vec<usize> = Vec::new();
            for row in exact_rows.iter() {
                lengths.push(row.len());
            }
            assert_eq!(
                lengths,
                vec![
                    cells(width, scale),
                    cells(width, scale) - 1,
                    cells(width, scale) - 1
                ],
                "{label}: every row fits exactly"
            );
        }
    }
}

pub fn place(
    head: usize,
    affinity: ReadingAffinity,
) -> CaretPlace {
    return CaretPlace {
        head,
        affinity,
    };
}

/// Global visual rows of `text`:
/// every line's rows in order,
/// as (line,
/// row,
/// range).
pub fn all_rows(
    text: &str,
    width: f32,
    scale: f32,
) -> Vec<(usize, usize, Range<usize>)> {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    let mut result = Vec::new();
    for line in 0..document.text().len_lines() {
        let shaped = shaper.wrapped_row(&document, line, scale, width);
        let line_start = document.text().line_to_char(line);
        let mut rows = shaped.source_rows();
        // A line without text still has one empty row at its start.
        if rows.is_empty() {
            rows.push(line_start..line_start);
        }
        for (index, row) in rows.into_iter().enumerate() {
            result.push((line, index, row));
        }
    }
    return result;
}

/// A document of wrapped lines,
/// a short line,
/// an empty line,
/// and a wrapped line at the end of the file.
pub fn several_lines(
    width: f32,
    scale: f32,
) -> String {
    return format!(
        "{}\nshort\n\n{}\n{}",
        prose(width, scale),
        tab_starts_a_row(width, scale),
        exact_fit(width, scale)
    );
}

/// The global index of the visual row that holds the caret of `document`.
pub fn caret_row(
    document: &Document,
    width: f32,
    scale: f32,
) -> usize {
    let position = document.position();
    let rows = all_rows(&document.text().to_string(), width, scale);
    let line = document.text().char_to_line(position.head);
    let mut in_line: Vec<usize> = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        if row.0 == line {
            in_line.push(index);
        }
    }
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for index in in_line.iter() {
        ranges.push(rows[*index].2.clone());
    }
    return in_line[row_containing(&ranges, position.head, position.affinity)];
}
