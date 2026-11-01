//! Match rectangles come from the shaped rows used by selection and never change shaping.

/// Canonical source,
///  the production matcher,
///  and the production geometry path.
use ide_app::{
    document::{Document, ReadingPosition},
    find::{FindRange, MAX_FIND_MATCHES, find_matches},
    find_paint::rectangles,
    shaped_text::{ShapedView, TextShaper, Viewport},
};
/// Actual shaped glyph runs prove that marking matches does not reshape a ligature.
use parley::PositionedLayoutItem;

/// One logical row at scale one unless a test materializes more.
fn viewport(first: usize, count: usize) -> Viewport {
    return Viewport {
        first,
        count,
        width: 500.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
}

/// Glyph identities of every materialized row,
///  in visual order.
fn glyphs(view: &ShapedView) -> Vec<u32> {
    // What: `Vec::new()` creates an empty growable list; `u32` is a fixed 32-bit unsigned glyph index.
    // Why: Comparing identities detects any substitution change, even with equal glyph counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const result: number[] = [];
    // ```
    let mut result = Vec::new();
    for row in &view.rows {
        for line in row.layout.lines() {
            for item in line.items() {
                // What: `if let` runs only for glyph runs, the only item kind this layout produces.
                // Why: Inline boxes carry no glyph identities.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (item.kind === 'glyphRun') for (const glyph of item.glyphs) result.push(glyph.id);
                // ```
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    for glyph in run.positioned_glyphs() {
                        result.push(glyph.id);
                    }
                }
            }
        }
    }
    return result;
}

/// Production matches of `query` in the document,
///  as a plain list.
fn ranges(document: &Document, query: &str) -> Vec<FindRange> {
    // What: `to_string` copies the rope into one owned string; `expect` fails the test on a matcher error.
    // Why: The matcher takes contiguous text, exactly as the worker supplies it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return findMatches(document.text.toString(), query, MAX_FIND_MATCHES).ranges;
    // ```
    let text = document.text().to_string();
    let matches = find_matches(&text, query, MAX_FIND_MATCHES).expect("fixture query");
    return matches.ranges.to_vec();
}

/// Each character inside the `===` ligature is its own match rectangle;
///  shaping is untouched.
#[test]
fn matches_inside_a_ligature_use_selection_geometry_without_reshaping() {
    let mut document = Document::new("a === b");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(0, 1), &[]);
    let found = ranges(&document, "=");
    assert_eq!(found.len(), 3);
    let marks = rectangles(&view, &found, None);
    assert_eq!(marks.len(), 3);
    for (index, mark) in marks.iter().enumerate() {
        document.select(ReadingPosition {
            anchor: found[index].start,
            head: found[index].end,
            viewport: 0,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
        let selected = shaper.prepare(&document, viewport(0, 1), &[]);
        let selection = selected.selections[0];
        assert_eq!(
            (mark.x, mark.y, mark.width, mark.height),
            (selection.x, selection.y, selection.width, selection.height),
            "match {index} must use the same rectangle as selecting those characters"
        );
        assert_eq!(
            glyphs(&view),
            glyphs(&selected),
            "selection must not reshape the ligature"
        );
    }
    assert!(marks[0].x + marks[0].width <= marks[1].x + 0.01);
    assert!(marks[1].x + marks[1].width <= marks[2].x + 0.01);
    assert!(
        marks[1].width > 1.0,
        "the middle ligature character must have a visible rectangle"
    );
    let plain = shaper.prepare(&Document::new("a === b"), viewport(0, 1), &[]);
    assert_eq!(
        glyphs(&view),
        glyphs(&plain),
        "marking matches must not change glyphs"
    );
}

/// The active match is drawn as the selection,
///  so it is left out of the other-match rectangles.
#[test]
fn active_match_is_excluded_from_other_match_rectangles() {
    let document = Document::new("needle and needle and needle");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(0, 1), &[]);
    let found = ranges(&document, "needle");
    let every = rectangles(&view, &found, None);
    // What: `Some(found[1])` wraps a copy of the second match as the active one.
    // Why: Only the exactly selected match is excluded; its neighbours stay marked.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const others = rectangles(view, found, found[1], 0, 500);
    // ```
    let others = rectangles(&view, &found, Some(found[1]));
    assert_eq!(every.len(), 3);
    assert_eq!(
        others.len(),
        2,
        "the active match was also drawn as an ordinary match"
    );
    assert_eq!(others[0].x, every[0].x);
    assert_eq!(others[1].x, every[2].x);
}

/// Only materialized rows produce rectangles.
#[test]
fn rectangles_are_limited_to_materialized_rows() {
    let document = Document::new(&"needle and a far needle\n".repeat(100));
    let mut shaper = TextShaper::new();
    let found = ranges(&document, "needle");
    assert_eq!(found.len(), 200);
    let view = shaper.prepare(&document, viewport(10, 5), &[]);
    let marks = rectangles(&view, &found, None);
    assert_eq!(
        marks.len(),
        10,
        "rectangles were built for rows outside the materialized viewport"
    );
    for mark in &marks {
        assert!(mark.y >= 10.0 * 24.0 && mark.y < 15.0 * 24.0);
    }
    let empty = shaper.prepare(&document, viewport(500, 5), &[]);
    assert!(rectangles(&empty, &found, None).is_empty());
    assert!(rectangles(&view, &[], None).is_empty());
}

/// Tabs,
///  CJK,
///  and combining marks keep match rectangles aligned with their source characters.
#[test]
fn rectangles_follow_tabs_wide_glyphs_and_combining_marks() {
    let document = Document::new("\t猫 cafe\u{301} 猫");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(0, 1), &[]);
    let cats = ranges(&document, "猫");
    assert_eq!(
        cats,
        [
            FindRange { start: 1, end: 2 },
            FindRange { start: 9, end: 10 }
        ]
    );
    let marks = rectangles(&view, &cats, None);
    assert_eq!(marks.len(), 2);
    let tab = view.range(0, 1)[0];
    assert!(
        (marks[0].x - (tab.x + tab.width)).abs() < 0.01,
        "the first match must start where the expanded tab ends"
    );
    assert!(marks[0].width > 1.0 && (marks[0].width - marks[1].width).abs() < 0.01);
    let accent = ranges(&document, "e\u{301}");
    assert_eq!(accent, [FindRange { start: 6, end: 8 }]);
    let accented = rectangles(&view, &accent, None);
    let before = view.range(5, 6)[0];
    assert_eq!(accented.len(), 1);
    assert!(
        (accented[0].x - (before.x + before.width)).abs() < 0.01 && accented[0].width > 1.0,
        "a base letter and its combining mark must share one rectangle after the preceding letter"
    );
}
