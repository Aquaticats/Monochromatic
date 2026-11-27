//! What a slice draws:
//!  the glyphs of its lines in the palette inks,
//!  the selection and the matches as marks under them,
//!  and a diagnostic as a stroke in its ink.

use crate::fixtures::{
    DARK,
    FILLS,
    INKS,
    fixture,
    whole,
};
use ide_app::{
    annotation::Mark,
    code_colors::ink,
    find::FindRange,
    language::diagnostics::Severity,
    tile_scene::{
        Item,
        read::{
            drawn_fills,
            drawn_runs,
            drawn_strokes,
        },
    },
};

/// The glyphs of a slice are drawn in the palette's inks for their syntax classes,
/// and the slice holds the lines of its bands and little else.
#[test]
fn a_slice_draws_the_glyphs_of_its_lines_in_the_palette_inks() {
    let fixture = fixture(300, 730.0, 1.0);
    let (items, rows, stats) = whole(&fixture);
    let runs = drawn_runs(&items);
    assert!(!rows.is_empty());
    assert_eq!(stats.shaped, stats.lines, "a cold build shapes every line");
    assert_eq!(stats.skipped, 0);
    assert!(runs.len() >= stats.lines, "a run for every line at least");
    let mut classes = std::collections::BTreeSet::new();
    for run in &runs {
        classes.insert(run.ink);
    }
    assert!(
        classes.contains(&DARK.foreground),
        "the plain text is drawn in the foreground"
    );
    for role in 1..=5_u32 {
        let wanted = ink(role, DARK);
        assert!(
            classes.contains(&wanted),
            "no run is drawn in the ink of class {role}: {wanted:?}"
        );
    }
}

/// The selection,
/// the matches,
/// and the active match's boundary are drawn as marks under the glyphs,
/// in the markup's inks and order.
#[test]
fn the_selection_and_the_matches_are_marks_under_the_glyphs() {
    let mut fixture = fixture(40, 730.0, 1.0);
    let plain = whole(&fixture).0;
    assert!(
        drawn_fills(&plain).is_empty(),
        "a text with no selection and no match has no marks"
    );
    // A selection over the middle of the second and third lines.
    let start = fixture.document.text().line_to_char(1) + 4;
    let end = fixture.document.text().line_to_char(2) + 12;
    fixture.select(start, end);
    let selected = whole(&fixture).0;
    let fills = drawn_fills(&selected);
    // The first line's mark and the cell its line break takes touch, and are drawn as one so no seam shows.
    assert_eq!(
        fills.len(),
        2,
        "one mark for each of the two lines the selection crosses: {fills:?}"
    );
    for mark in &fills {
        assert_eq!(mark.ink, FILLS.selection);
        assert!(mark.right > mark.left && mark.bottom > mark.top);
    }
    let first_glyph = selected
        .iter()
        .position(|item| return matches!(item, Item::Glyphs(_)))
        .expect("glyphs");
    let last_fill = selected
        .iter()
        .rposition(|item| return matches!(item, Item::Fill { .. }))
        .expect("marks");
    assert!(last_fill < first_glyph, "the marks stand under the glyphs");
    // The same selection as an active find match is outlined too.
    fixture.fills.boundary = true;
    let outlined = drawn_fills(&whole(&fixture).0);
    assert!(
        outlined.len() > fills.len(),
        "the active match has a boundary"
    );
    assert!(
        outlined
            .iter()
            .any(|mark| return mark.ink == FILLS.foreground),
        "the boundary is drawn in the foreground"
    );
    // A match that is not the selection: a translucent fill and a thin boundary.
    fixture.fills.boundary = false;
    fixture.select(0, 0);
    let from = fixture.document.text().line_to_char(5) + 4;
    fixture.ranges = vec![FindRange {
        start: from,
        end: from + 5,
    }];
    let found = drawn_fills(&whole(&fixture).0);
    assert_eq!(found.len(), 2, "a fill and one frame: {found:?}");
    let fill = found
        .iter()
        .find(|mark| return mark.inset == 0.0)
        .expect("the translucent fill");
    let frame = found
        .iter()
        .find(|mark| return mark.inset > 0.0)
        .expect("the boundary drawn as one frame");
    assert!(
        fill.ink[3] > 0 && fill.ink[3] < 60,
        "the fill is faint: {fill:?}"
    );
    assert!(
        frame.ink[3] > 100,
        "the boundary is 0.7 of the foreground: {frame:?}"
    );
    assert!(
        frame.left == fill.left && frame.right == fill.right,
        "the frame is the outline of the fill"
    );
}

/// A diagnostic's underline is a stroke,
/// and an error's differs from a warning's.
#[test]
fn a_diagnostic_is_drawn_as_a_stroke_in_its_ink() {
    let mut fixture = fixture(40, 730.0, 1.0);
    let line = fixture.document.text().line_to_char(4);
    fixture.marks = vec![
        Mark {
            start: line + 4,
            end: line + 9,
            severity: Severity::Error,
        },
        Mark {
            start: line + 12,
            end: line + 17,
            severity: Severity::Warning,
        },
    ];
    let strokes = drawn_strokes(&whole(&fixture).0);
    assert!(
        strokes
            .iter()
            .any(|drawn| return drawn.stroke.ink == INKS.error && drawn.stroke.dash.is_empty()),
        "a solid wave in the error ink"
    );
    assert!(
        strokes
            .iter()
            .any(|drawn| return drawn.stroke.ink == INKS.warning && !drawn.stroke.dash.is_empty()),
        "dashes in the warning ink"
    );
}
