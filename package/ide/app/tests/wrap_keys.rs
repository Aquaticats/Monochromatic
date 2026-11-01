//! Home,
//! End,
//! Up,
//! Down,
//! and paging on the visual rows of wrapped lines,
//! without a window.
//! Every test runs at several wrap widths and at scales 1 and 2,
//! on lines the same shaper wraps as the source view.
//! Rows that start or end with a tab,
//! rows that fit exactly,
//! and a wrapped line at the end of the file are covered.

/// Canonical source and the reading position with its side of a wrap offset.
use ide_app::document::{Document, ReadingAffinity, ReadingPosition};
/// The pure Home and End rules,
/// and the shaped form the key handler calls.
use ide_app::line_boundary::{CaretPlace, Edge, end, home, moved_to_edge, row_containing};
/// Shaper and the rows it produces.
use ide_app::shaped_text::TextShaper;
/// The remembered column and the vertical step under test.
use ide_app::vertical_motion::{PreferredColumn, vertical};
/// Source character ranges of visual rows.
use std::ops::Range;

/// Wrap widths in logical pixels:
/// a narrow pane,
/// a middle one,
/// a wide one.
const WIDTHS: [f32; 3] = [240.0, 410.0, 700.0];
/// Display scales the movements are checked at.
const SCALES: [f32; 2] = [1.0, 2.0];

/// A document whose caret is at `head` on `affinity`,
/// with no selection.
fn at(text: &str, head: usize, affinity: ReadingAffinity) -> Document {
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
fn advance(scale: f32) -> f32 {
    let document = Document::new("mm");
    let mut shaper = TextShaper::new();
    return shaper.row(&document, 0, scale).caret_x(1, scale);
}

/// How many glyphs of the source font fill a row of `width` logical pixels.
fn cells(width: f32, scale: f32) -> usize {
    return (width / advance(scale)).floor() as usize;
}

/// Visual rows of line `line` of `text` as global character ranges,
/// shaped at `width` and `scale`.
fn rows_of(text: &str, line: usize, width: f32, scale: f32) -> Vec<Range<usize>> {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    return shaper
        .wrapped_row(&document, line, scale, width)
        .source_rows();
}

/// An indented line of words that wraps to several rows.
fn prose(width: f32, scale: f32) -> String {
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
fn tab_starts_a_row(width: f32, scale: f32) -> String {
    return format!("{} \ttail words", "m".repeat(cells(width, scale)));
}

/// A line whose first row ends with a tab that fills it exactly,
/// then a token that fills the next row.
fn tab_ends_a_row(width: f32, scale: f32) -> String {
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
fn exact_fit(width: f32, scale: f32) -> String {
    return "m".repeat(cells(width, scale) * 3 - 2);
}

/// Every row range of a line tiles its text:
/// each starts where the one before ended,
/// the last ends at the line end.
fn assert_tiles(rows: &[Range<usize>], line_end: usize, label: &str) {
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

/// The caret at the end of an upper row belongs to that row,
/// and at the start of the lower row to the lower one.
#[test]
fn a_wrap_offset_belongs_to_the_row_the_side_names() {
    let rows = vec![0..5, 5..9, 9..12];
    assert_eq!(row_containing(&rows, 5, ReadingAffinity::Downstream), 1);
    assert_eq!(row_containing(&rows, 5, ReadingAffinity::Upstream), 0);
    assert_eq!(row_containing(&rows, 4, ReadingAffinity::Upstream), 0);
    assert_eq!(
        row_containing(&rows, 0, ReadingAffinity::Upstream),
        0,
        "no row above the first"
    );
    assert_eq!(
        row_containing(&rows, 12, ReadingAffinity::Downstream),
        2,
        "the end of the line is on the last row"
    );
    assert_eq!(row_containing(&rows, 12, ReadingAffinity::Upstream), 2);
}

/// A line "  first row  second row  third" in three rows,
/// to exercise the rules with exact characters.
fn sample() -> (ide_app::document::Document, Vec<Range<usize>>) {
    // Characters: 2 blanks, "first" 2..7, blank 7, "row" 8..11, 2 blanks 11..13 | "second" 13..19, blank 19,
    // "row" 20..23, 2 blanks 23..25 | "third" 25..30.
    let document = Document::new("  first row  second row  third");
    return (document, vec![0..13, 13..25, 25..30]);
}

/// A line of one visual row that holds `length` characters.
fn one_row(length: usize) -> Vec<Range<usize>> {
    return vec![Range {
        start: 0,
        end: length,
    }];
}

/// What:
///  One line's text and its row ranges,
///  so the tests name the line once and ask for a Home or End press at a caret.
/// Why:
///  Every assertion presses from a different caret in the same line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const from = { home: (head, affinity) => home(text, rows, head, affinity), end: ... };
/// ```
struct Navigator<'a> {
    /// The line's text.
    text: helix_core::RopeSlice<'a>,
    /// The visual rows of the line,
    ///  as character ranges.
    rows: &'a [Range<usize>],
}

impl Navigator<'_> {
    /// Press Home with the caret at `head` and `affinity`.
    fn home(&self, head: usize, affinity: ReadingAffinity) -> CaretPlace {
        return home(self.text, self.rows, head, affinity);
    }

    /// Press End with the caret at `head` and `affinity`.
    fn end(&self, head: usize, affinity: ReadingAffinity) -> CaretPlace {
        return end(self.text, self.rows, head, affinity);
    }
}

fn place(head: usize, affinity: ReadingAffinity) -> CaretPlace {
    return CaretPlace { head, affinity };
}

/// Home on a row goes to its start,
/// from the start of a continuation row to the first non-blank character,
///  and on the first row it alternates between the line start and the first non-blank character.
#[test]
fn home_goes_to_the_row_start_then_the_first_non_blank() {
    let (document, rows) = sample();
    let from = Navigator {
        text: document.text().slice(..),
        rows: &rows,
    };
    use ReadingAffinity::{Downstream, Upstream};
    assert_eq!(
        from.home(28, Downstream),
        place(25, Downstream),
        "inside the last row"
    );
    assert_eq!(
        from.home(25, Downstream),
        place(2, Downstream),
        "at a continuation row start: the first non-blank"
    );
    assert_eq!(
        from.home(13, Downstream),
        place(2, Downstream),
        "the same from the middle row start"
    );
    assert_eq!(
        from.home(13, Upstream),
        place(2, Downstream),
        "an end of the first row, past the first non-blank"
    );
    assert_eq!(from.home(24, Downstream), place(13, Downstream));
    assert_eq!(
        from.home(5, Downstream),
        place(2, Downstream),
        "past the indent on the first row"
    );
    assert_eq!(
        from.home(2, Downstream),
        place(0, Downstream),
        "at the first non-blank character"
    );
    assert_eq!(
        from.home(1, Downstream),
        place(0, Downstream),
        "inside the indent"
    );
    assert_eq!(
        from.home(0, Downstream),
        place(2, Downstream),
        "from the line start"
    );
    // A first row of blanks only has no first non-blank character.
    let blanks = Document::new("      then text");
    let blank_rows = vec![0..6, 6..15];
    let blank_text = blanks.text().slice(..);
    assert_eq!(
        home(blank_text, &blank_rows, 10, Downstream),
        place(6, Downstream)
    );
    assert_eq!(
        home(blank_text, &blank_rows, 6, Downstream),
        place(0, Downstream),
        "no first non-blank on row 0"
    );
    assert_eq!(
        home(blank_text, &blank_rows, 3, Downstream),
        place(0, Downstream)
    );
    assert_eq!(
        home(blank_text, &blank_rows, 0, Downstream),
        place(0, Downstream)
    );
}

/// End goes to the end of the row without its trailing blanks,
/// then to the row end,
///  then from the row end to the logical line end,
/// which again leaves its trailing blanks out first.
#[test]
fn end_goes_to_the_row_end_then_the_line_end() {
    let (document, rows) = sample();
    let from = Navigator {
        text: document.text().slice(..),
        rows: &rows,
    };
    use ReadingAffinity::{Downstream, Upstream};
    assert_eq!(
        from.end(0, Downstream),
        place(11, Downstream),
        "before the two trailing blanks of the first row"
    );
    assert_eq!(
        from.end(11, Downstream),
        place(13, Upstream),
        "the end of the upper row is drawn on it"
    );
    assert_eq!(
        from.end(13, Upstream),
        place(30, Downstream),
        "from the end of an upper row to the line end"
    );
    assert_eq!(
        from.end(13, Downstream),
        place(23, Downstream),
        "the middle row, before its trailing blanks"
    );
    assert_eq!(from.end(23, Downstream), place(25, Upstream));
    assert_eq!(from.end(25, Upstream), place(30, Downstream));
    assert_eq!(
        from.end(27, Downstream),
        place(30, Downstream),
        "the last row has no trailing blanks"
    );
    assert_eq!(
        from.end(30, Downstream),
        place(30, Downstream),
        "at the end of the text"
    );
    // A row of blanks only keeps its whole end.
    let gap = Document::new("aaaa    bb");
    let gap_rows = vec![0..4, 4..8, 8..10];
    let gap_text = gap.text().slice(..);
    assert_eq!(
        end(gap_text, &gap_rows, 5, Downstream),
        place(8, Upstream),
        "a row of blanks only keeps its whole end"
    );
    // Trailing blanks on the last row: End goes before them, then back after them.
    let trailing = Document::new("ab   ");
    let trailing_rows = one_row(5);
    let trailing_text = trailing.text().slice(..);
    assert_eq!(
        end(trailing_text, &trailing_rows, 0, Downstream),
        place(2, Downstream)
    );
    assert_eq!(
        end(trailing_text, &trailing_rows, 2, Downstream),
        place(5, Downstream)
    );
    assert_eq!(
        end(trailing_text, &trailing_rows, 5, Downstream),
        place(2, Downstream)
    );
    // A line of blanks only.
    let blank = Document::new("   ");
    let blank_rows = one_row(3);
    assert_eq!(
        end(blank.text().slice(..), &blank_rows, 0, Downstream),
        place(3, Downstream)
    );
    assert_eq!(
        end(blank.text().slice(..), &blank_rows, 3, Downstream),
        place(0, Downstream)
    );
}

/// Press Home or End from `head` until the place stops changing,
/// collecting each place.
fn press_until_still(
    shaper: &mut TextShaper,
    text: &str,
    start: CaretPlace,
    edge: Edge,
    width: f32,
    scale: f32,
) -> Vec<CaretPlace> {
    let mut places = vec![start];
    for _ in 0..12 {
        let last = *places.last().expect("a place");
        let document = at(text, last.head, last.affinity);
        let next = moved_to_edge(shaper, &document, scale, width, edge);
        if next == last {
            break;
        }
        places.push(next);
    }
    return places;
}

/// On shaped rows,
/// Home from the middle of every continuation row goes to the row start,
/// then to the line's first
///  non-blank character,
/// and the caret is drawn on the row it names.
#[test]
fn home_on_shaped_rows_at_every_width_and_scale() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = prose(width, scale);
            let rows = rows_of(&text, 0, width, scale);
            let mut shaper = TextShaper::new();
            for (index, row) in rows.iter().enumerate().skip(1) {
                let middle = (row.start + row.end) / 2;
                let start = place(middle, ReadingAffinity::Downstream);
                let places = press_until_still(&mut shaper, &text, start, Edge::Home, width, scale);
                assert_eq!(
                    places[1],
                    place(row.start, ReadingAffinity::Downstream),
                    "{label}: row {index}"
                );
                assert_eq!(
                    places[2],
                    place(4, ReadingAffinity::Downstream),
                    "{label}: row {index} second press"
                );
                assert_eq!(
                    places[3],
                    place(0, ReadingAffinity::Downstream),
                    "{label}: row {index} third press"
                );
                assert_eq!(
                    places[4],
                    place(4, ReadingAffinity::Downstream),
                    "{label}: row {index} fourth press"
                );
                let document = at(&text, row.start, ReadingAffinity::Downstream);
                let mut again = TextShaper::new();
                let caret = again.wrapped_row(&document, 0, scale, width).caret_at(
                    row.start,
                    ReadingAffinity::Downstream,
                    scale,
                );
                assert_eq!(
                    caret.row, index,
                    "{label}: the row start is drawn on its own row"
                );
            }
        }
    }
}

/// On shaped rows,
/// End walks from the first row's start to every row end and to the line end,
///  the end of an upper row being drawn on that row.
#[test]
fn end_on_shaped_rows_at_every_width_and_scale() {
    let builders: [(&str, fn(f32, f32) -> String); 4] = [
        ("prose", prose),
        ("tab starts a row", tab_starts_a_row),
        ("tab ends a row", tab_ends_a_row),
        ("exact fit", exact_fit),
    ];
    for (name, build) in builders {
        for width in WIDTHS {
            for scale in SCALES {
                let label = format!("{name}, width {width} scale {scale}");
                let text = build(width, scale);
                let rows = rows_of(&text, 0, width, scale);
                let mut shaper = TextShaper::new();
                // Pressing End from each row start reaches the row end, drawn on the same row.
                for (index, row) in rows.iter().enumerate() {
                    let start = place(row.start, ReadingAffinity::Downstream);
                    let places =
                        press_until_still(&mut shaper, &text, start, Edge::End, width, scale);
                    let mut found_place = None;
                    for candidate in places.iter() {
                        if candidate.head == row.end {
                            found_place = Some(candidate);
                            break;
                        }
                    }
                    let Some(reached) = found_place else {
                        panic!("{label}: row {index} never reached its end, places {places:?}")
                    };
                    let expected_side = if index + 1 < rows.len() {
                        ReadingAffinity::Upstream
                    } else {
                        ReadingAffinity::Downstream
                    };
                    assert_eq!(
                        reached.affinity, expected_side,
                        "{label}: row {index} end side"
                    );
                    let document = at(&text, reached.head, reached.affinity);
                    let drawn = shaper.wrapped_row(&document, 0, scale, width).caret_at(
                        reached.head,
                        reached.affinity,
                        scale,
                    );
                    assert_eq!(
                        drawn.row, index,
                        "{label}: the end of row {index} is drawn on it"
                    );
                    // Everything skipped on the way is a blank that ends the row.
                    for step in places.iter().skip(1) {
                        if step.head < row.end && step.head >= row.start {
                            let mut only_blanks = true;
                            for character in text.chars().skip(step.head).take(row.end - step.head)
                            {
                                if character != ' ' && character != '\t' {
                                    only_blanks = false;
                                }
                            }
                            assert!(
                                only_blanks,
                                "{label}: row {index} stopped at {} before non-blanks",
                                step.head
                            );
                        }
                    }
                }
                // From the first row start the presses end at the line end, which is where a caret at the end of an
                // upper row goes.
                let start = place(0, ReadingAffinity::Downstream);
                let places = press_until_still(&mut shaper, &text, start, Edge::End, width, scale);
                assert_eq!(
                    places.last().expect("a place").head,
                    text.chars().count(),
                    "{label}: the presses end at the line end"
                );
            }
        }
    }
}

/// The caret at the end of an exact-fit row is drawn on that row;
/// the same offset downstream is on the next.
#[test]
fn an_exact_fit_row_end_is_drawn_on_the_upper_row_only_when_the_side_says_so() {
    for width in WIDTHS {
        for scale in SCALES {
            let label = format!("width {width} scale {scale}");
            let text = exact_fit(width, scale);
            let rows = rows_of(&text, 0, width, scale);
            let document = Document::new(&text);
            let mut shaper = TextShaper::new();
            let shaped = shaper.wrapped_row(&document, 0, scale, width);
            let wrap = rows[0].end;
            let upper = shaped.caret_at(wrap, ReadingAffinity::Upstream, scale);
            let lower = shaped.caret_at(wrap, ReadingAffinity::Downstream, scale);
            assert_eq!((upper.row, lower.row), (0, 1), "{label}");
            assert!(
                upper.x > lower.x + width / 2.0,
                "{label}: the upper caret is at the right edge, the lower at the left"
            );
        }
    }
}

/// Global visual rows of `text`:
/// every line's rows in order,
/// as (line,
/// row,
/// range).
fn all_rows(text: &str, width: f32, scale: f32) -> Vec<(usize, usize, Range<usize>)> {
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
fn several_lines(width: f32, scale: f32) -> String {
    return format!(
        "{}\nshort\n\n{}\n{}",
        prose(width, scale),
        tab_starts_a_row(width, scale),
        exact_fit(width, scale)
    );
}

/// The global index of the visual row that holds the caret of `document`.
fn caret_row(document: &Document, width: f32, scale: f32) -> usize {
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
                let landed = shaped.hit_at(10_000.0, 24.0 * rows.len() as f32 + 100.0, scale);
                assert_eq!(landed, end, "{label}: a point below the rows is at the end");
            }
        }
    }
    assert!(
        tails > 0,
        "no fixture produced an empty tail, so the test proves nothing about it"
    );
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
