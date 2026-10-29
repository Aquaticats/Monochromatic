//! Caret,
//!  hit testing,
//!  selection,
//!  and copy agree at CJK,
//!  combining,
//!  astral,
//!  tab,
//!  ligature,
//!  CRLF,
//! and end-of-text boundaries,
//!  for every grapheme boundary of every fixture line.

/// What:
///  Helix's grapheme walk over the document rope.
/// Why:
///  The expected caret positions are the grapheme boundaries,
///  computed independently of the shaper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ensureGraphemeBoundaryNext, nextGraphemeBoundary } from 'helix-core/graphemes';
/// ```
use helix_core::graphemes::{ensure_grapheme_boundary_next, next_grapheme_boundary};
/// Canonical source,
///  line ends,
///  and the production shaped geometry.
use ide_app::{
    caret_motion::line_end,
    document::{Document, ReadingPosition},
    shaped_text::{ShapedView, TERMINATOR_MARK, TextShaper, Viewport},
};

/// What:
///  `const` names a compile-time list;
///  `[(&str, &str); 9]` is a fixed array of nine name and text pairs.
/// Why:
///  Every boundary class the reader must handle is checked by the same properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIXTURES: readonly [name: string, source: string][] = [...];
/// ```
const FIXTURES: [(&str, &str); 9] = [
    ("cjk", "a猫b猫猫c 猫"),
    ("combining", "cafe\u{301} e\u{301}\u{302}x a\u{308}"),
    ("astral", "x𝒳y𝔘z"),
    ("emoji", "a👩\u{200d}👩\u{200d}👧b🇯🇵c"),
    ("tab", "\ta\t猫\tb\t"),
    ("ligature", "a === b != c -> d => e"),
    ("crlf", "ab\r\n猫\tc\r\n\r\nd"),
    ("no-final-newline", "ab\n猫c"),
    ("final-newline", "ab\n\tc\n"),
];

/// Shape a whole fixture at scale one.
fn shaped(source: &str) -> (Document, ShapedView) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 16,
        width: 800.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
    // What: `&document` lends the source to the shaper; the tuple hands both values to the caller.
    // Why: Assertions need the document for copying and the view for geometry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [document, shaper.prepare(document, viewport, [])];
    // ```
    let view = shaper.prepare(&document, viewport, &[]);
    return (document, view);
}

/// What:
///  Every grapheme boundary of one row from its start to the end of its visible text;
/// `Vec<usize>` is a growable list of character positions (siblings:
///  `[usize; N]`,
///  `&[usize]`).
/// Why:
///  These are exactly the positions a caret may take on that row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function boundaries(document: Document, start: number): number[];
/// ```
fn boundaries(document: &Document, start: usize) -> Vec<usize> {
    let text = document.text().slice(..);
    let end = line_end(text, start);
    let mut result = vec![start];
    let mut position = start;
    while position < end {
        position = next_grapheme_boundary(text, position);
        result.push(position);
    }
    return result;
}

/// Caret positions grow along the row and each one is found again by a hit at its own x.
#[test]
fn caret_positions_are_ordered_and_hit_testing_returns_to_them() {
    for (name, source) in FIXTURES {
        let (document, view) = shaped(source);
        for row in &view.rows {
            let mut previous = -1.0;
            for position in boundaries(&document, row.source_start) {
                let x = row.caret_x(position, 1.0);
                assert!(
                    x >= previous,
                    "{name} row {}: caret x went back at {position}",
                    row.row
                );
                assert!(
                    (row.caret_x(row.hit(x, 1.0), 1.0) - x).abs() < 0.01,
                    "{name} row {}: a hit at the caret of {position} landed elsewhere",
                    row.row
                );
                previous = x;
            }
        }
    }
}

/// Every pixel of every row resolves to the nearest grapheme boundary,
///  never to the inside of a sequence.
#[test]
fn every_pixel_hits_the_nearest_grapheme_boundary() {
    for (name, source) in FIXTURES {
        let (document, view) = shaped(source);
        let text = document.text().slice(..);
        for row in &view.rows {
            let stops = boundaries(&document, row.source_start);
            // What: `last().copied()` reads the final list element by value; `expect` fails the test if absent.
            // Why: The list always holds at least the row start.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const end = stops.at(-1)!;
            // ```
            let end = stops.last().copied().expect("row start boundary");
            let width = row.caret_x(end, 1.0);
            let mut reached = row.source_start;
            let mut x = 0.0;
            while x <= width + 30.0 {
                let hit = row.hit(x, 1.0);
                assert_eq!(
                    ensure_grapheme_boundary_next(text, hit),
                    hit,
                    "{name} row {}: x {x} hit inside a grapheme at {hit}",
                    row.row
                );
                assert!(
                    hit >= reached && hit <= end,
                    "{name} row {}: x {x} hit {hit} outside or before {reached}",
                    row.row
                );
                let mut nearest = f32::MAX;
                for stop in &stops {
                    nearest = nearest.min((row.caret_x(*stop, 1.0) - x).abs());
                }
                assert!(
                    (row.caret_x(hit, 1.0) - x).abs() <= nearest + 0.01,
                    "{name} row {}: x {x} hit {hit}, which is not the nearest caret position",
                    row.row
                );
                reached = hit;
                x += 0.5;
            }
            assert_eq!(
                reached, end,
                "{name} row {}: the end of the line was unreachable",
                row.row
            );
        }
    }
}

/// Each grapheme's selection rectangle spans exactly between its two caret positions,
///  and copy yields its source.
#[test]
fn each_grapheme_selects_its_own_extent_and_copies_its_source_characters() {
    for (name, source) in FIXTURES {
        let (mut document, view) = shaped(source);
        // What: `chars().collect()` copies the text into a list of Unicode scalars.
        // Why: Expected copies are taken from the fixture by character position, not by byte.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const characters = [...source];
        // ```
        let characters: Vec<char> = source.chars().collect();
        for row in &view.rows {
            let stops = boundaries(&document, row.source_start);
            for pair in stops.windows(2) {
                let (start, end) = (pair[0], pair[1]);
                let left = row.caret_x(start, 1.0);
                let right = row.caret_x(end, 1.0);
                let rectangles = row.range(start, end, 1.0);
                if right - left > 0.01 {
                    assert_eq!(
                        rectangles.len(),
                        1,
                        "{name} row {}: {start}..{end}",
                        row.row
                    );
                    assert!(
                        (rectangles[0].x - left).abs() < 0.01
                            && (rectangles[0].width - (right - left)).abs() < 0.01,
                        "{name} row {}: selection of {start}..{end} does not span its caret positions",
                        row.row
                    );
                }
                document.select(ReadingPosition {
                    anchor: start,
                    head: end,
                    viewport: 0,
                    affinity: ide_app::document::ReadingAffinity::Downstream,
                });
                let expected: String = characters[start..end].iter().collect();
                assert_eq!(
                    document.selected_text(),
                    expected,
                    "{name} row {}: copy of {start}..{end}",
                    row.row
                );
            }
        }
    }
}

/// Selecting a whole line with its terminator copies the terminator and marks it after the line's text.
#[test]
fn selected_terminators_are_copied_and_marked() {
    for (name, source) in FIXTURES {
        let mut document = Document::new(source);
        let lines = document.text().len_lines();
        let characters: Vec<char> = source.chars().collect();
        for line in 0..lines {
            let start = document.text().line_to_char(line);
            let next = document.text().line_to_char((line + 1).min(lines));
            let visible = line_end(document.text().slice(..), start);
            document.select(ReadingPosition {
                anchor: start,
                head: next,
                viewport: 0,
                affinity: ide_app::document::ReadingAffinity::Downstream,
            });
            let expected: String = characters[start..next].iter().collect();
            assert_eq!(
                document.selected_text(),
                expected,
                "{name} line {line}: copy of the whole line"
            );
            let mut shaper = TextShaper::new();
            let viewport = Viewport {
                first: 0,
                count: 16,
                width: 800.0,
                scale: 1.0,
                wrap: None,
                clip: None,
            };
            let view = shaper.prepare(&document, viewport, &[]);
            let mut marks = 0;
            for rectangle in &view.selections {
                if rectangle.y == line as f32 * 24.0
                    && (rectangle.x - view.rows[line].caret_x(visible, 1.0)).abs() < 0.01
                    && rectangle.width == TERMINATOR_MARK
                {
                    marks += 1;
                }
            }
            // A line has a terminator exactly when its range ends after its visible text.
            let terminated = next > visible;
            assert_eq!(
                marks,
                usize::from(terminated),
                "{name} line {line}: terminator mark, selections {:?}",
                view.selections
            );
        }
    }
}
