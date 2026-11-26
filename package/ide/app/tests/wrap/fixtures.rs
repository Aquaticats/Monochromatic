//! The text, the shaped row and the row checks every test builds on:
//!  a long line of ordinary code,
//!  a wrapped row at a width and a scale,
//!  the advance of a space,
//!  the grapheme starts of a text,
//!  and the checks that rows start on graphemes, tile the text, fit the width
//!  and break only where a break is allowed.

use helix_core::{
    Rope,
    graphemes::next_grapheme_boundary,
};
use ide_app::{
    document::Document,
    shaped_row::ShapedRow,
    shaped_text::TextShaper,
    wrap::intellij_break,
};

/// Text used for the long-line table:
/// words and punctuation with ordinary break opportunities.
pub const LONG_LINE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going, \
     and going, and going past every width in the table";

/// Shape row 0 of `text` at a logical wrap width and scale.
pub fn wrapped(
    text: &str,
    width: f32,
    scale: f32,
) -> ShapedRow {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    return shaper.wrapped_row(&document, 0, scale, width);
}

/// Advance of one space in logical pixels at `scale`,
/// measured on an unwrapped space.
pub fn space_advance(scale: f32) -> f32 {
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
fn char_index(
    text: &str,
    byte: usize,
) -> usize {
    return text[..byte].chars().count();
}

/// Whether every visual row begins on a grapheme cluster boundary.
pub fn rows_start_on_graphemes(row: &ShapedRow) -> bool {
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
pub fn rows_tile_the_text(row: &ShapedRow) -> bool {
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
pub fn rows_fit_width(
    row: &ShapedRow,
    width: f32,
    scale: f32,
) -> bool {
    for line in row.layout.lines() {
        let metrics = line.metrics();
        let visible = (metrics.advance - metrics.hanging_advance) / scale;
        if visible > width + 0.01 && line.text_range().len() > 1 {
            return false;
        }
    }
    return true;
}

/// Each row boundary of a wrapped line must be an IntelliJ break opportunity.
pub fn boundaries_are_break_opportunities(row: &ShapedRow) -> bool {
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

/// Visual row that holds display byte `byte`,
/// found from the row byte ranges.
pub fn row_of_byte(
    row: &ShapedRow,
    byte: usize,
) -> usize {
    for index in 0..row.visual_rows() {
        let bytes = row.visual_row_bytes(index).expect("row exists");
        if bytes.start <= byte && byte < bytes.end {
            return index;
        }
    }
    return row.visual_rows().saturating_sub(1);
}
