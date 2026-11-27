//! Home and End on rows the shaper produced,
//!  at every wrap width and at both scales:
//!  pressing from each row start until the place stops changing,
//!  the caret drawn on the row it names,
//!  and the end of an exactly filled row drawn on the upper row only when the side says so.

use crate::fixtures::{
    SCALES,
    WIDTHS,
    at,
    exact_fit,
    place,
    prose,
    rows_of,
    tab_ends_a_row,
    tab_starts_a_row,
};
use ide_app::document::{
    Document,
    ReadingAffinity,
};
/// The pure Home and End rules,
///  and the shaped form the key handler calls.
use ide_app::line_boundary::{
    CaretPlace,
    Edge,
    moved_to_edge,
};
/// Shaper and the rows it produces.
use ide_app::shaped_text::TextShaper;

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
