//! Shaped tabs end on pixel tab stops for Latin,
//!  CJK,
//!  combining,
//!  astral,
//!  and ligature prefixes at any scale.

/// Canonical source,
///  the production shaper,
///  and the font-free stop arithmetic it must agree with.
use ide_app::{
    document::{Document, ReadingPosition},
    shaped_text::{ShapedView, TextShaper, Viewport},
    tab_stop::{TAB_SPACES, tab_advance},
};

/// What:
///  Shape `source` at `scale` into one materialized viewport;
///  `f32` is a 32-bit float (sibling `f64`).
/// Why:
///  Every assertion reads the same geometry the native window paints and hit-tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shaped(source: string, scale: number): [Document, ShapedView];
/// ```
fn shaped(source: &str, scale: f32) -> (Document, ShapedView) {
    let document = Document::new(source);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: 4,
        width: 600.0,
        scale,
        wrap: None,
        clip: None,
    };
    // What: `&document` lends the source to the shaper; the tuple hands both values to the caller.
    // Why: The view borrows nothing from the document after shaping, so both can be returned together.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [document, shaper.prepare(document, viewport, [])];
    // ```
    let view = shaper.prepare(&document, viewport, &[]);
    return (document, view);
}

/// Logical caret x before character `position` of a one-line source.
fn caret_x(source: &str, position: usize, scale: f32) -> f32 {
    let (_, view) = shaped(source, scale);
    return view.rows[0].caret_x(position, scale);
}

/// Measured advance of one space in the source font at scale one.
fn space() -> f32 {
    let advance = caret_x(" x", 1, 1.0);
    assert!(
        advance > 1.0,
        "the source font must have a usable space advance"
    );
    return advance;
}

/// Compare two pixel positions within a hundredth of a pixel.
fn close(actual: f32, expected: f32) -> bool {
    return (actual - expected).abs() < 0.01;
}

/// Leading tabs,
///  tabs after Latin text,
///  and several tabs per line end on multiples of two space advances.
#[test]
fn tabs_end_on_multiples_of_two_measured_space_advances() {
    let stop = space() * TAB_SPACES;
    // Each case: source, position after the tab under test, expected number of stops.
    let cases = [
        ("\tx", 1, 1.0),
        ("a\tx", 2, 1.0),
        ("ab\tx", 3, 2.0),
        ("abc\tx", 4, 2.0),
        ("\t\tx", 2, 2.0),
        ("a\t\tx", 3, 2.0),
        ("\ta\tb", 3, 2.0),
        ("x \tb", 3, 2.0),
    ];
    for (source, position, stops) in cases {
        let after = caret_x(source, position, 1.0);
        assert!(
            close(after, stop * stops),
            "{source:?}: the tab ends at {after}, expected {}",
            stop * stops
        );
    }
}

/// A tab after CJK,
///  combining,
///  astral,
///  or ligature text ends on the same pixel stops as after Latin text.
#[test]
fn tab_after_mixed_script_prefix_ends_on_a_pixel_stop() {
    let advance = space();
    let stop = advance * TAB_SPACES;
    let mut off_grid = 0;
    for prefix in [
        "猫",
        "猫a",
        "a猫",
        "猫猫",
        "猫猫猫",
        "e\u{301}",
        "𝒳",
        "===",
        "a => 猫",
    ] {
        let source = format!("{prefix}\tx");
        let count = prefix.chars().count();
        let before = caret_x(&source, count, 1.0);
        let after = caret_x(&source, count + 1, 1.0);
        let stops = after / stop;
        assert!(
            close(stops, stops.round()),
            "{source:?}: the tab ends at {after}, which is not a multiple of {stop}"
        );
        assert!(
            close(after - before, tab_advance(before, advance)),
            "{source:?}: tab width {} differs from the stop rule",
            after - before
        );
        assert!(
            after - before >= advance / 2.0 - 0.01,
            "{source:?}: the tab is narrower than half a space"
        );
        // A prefix whose width is not a whole number of space advances is what column counting gets wrong.
        if !close(before / advance, (before / advance).round()) {
            off_grid += 1;
        }
    }
    assert!(
        off_grid > 0,
        "no prefix was off the Latin cell grid, so the fixture no longer exercises mixed-script geometry"
    );
}

/// The logical tab geometry does not depend on the display scale.
#[test]
fn tab_geometry_is_the_same_at_every_scale() {
    for source in ["\tx", "a\tx", "猫\tx", "a猫\t\tx", "\ta\tb"] {
        let count = source.chars().count();
        for position in 0..=count {
            let reference = caret_x(source, position, 1.0);
            for scale in [1.25, 2.0, 3.0] {
                let scaled = caret_x(source, position, scale);
                assert!(
                    (scaled - reference).abs() < 0.05,
                    "{source:?} at {position}: {scaled} at scale {scale}, {reference} at scale 1"
                );
            }
        }
    }
}

/// A point inside a widened tab resolves to the nearer edge,
///  and the tab's selection covers it exactly.
#[test]
fn hit_selection_and_copy_treat_a_tab_as_one_source_character() {
    let advance = space();
    let stop = advance * TAB_SPACES;
    // a(0) b(1) tab(2) c(3): the tab spans one whole stop, from two to four space advances.
    let (mut document, view) = shaped("ab\tc", 1.0);
    let row = &view.rows[0];
    assert!(close(row.caret_x(2, 1.0), stop) && close(row.caret_x(3, 1.0), stop * 2.0));
    assert_eq!(
        row.hit(stop + stop * 0.25, 1.0),
        2,
        "the left part of a tab is before it"
    );
    assert_eq!(
        row.hit(stop + stop * 0.75, 1.0),
        3,
        "the right part of a tab is after it"
    );
    assert_eq!(row.hit(stop * 2.0 + advance * 0.25, 1.0), 3);
    assert_eq!(row.hit(stop * 2.0 + advance * 0.75, 1.0), 4);
    assert_eq!(
        row.hit(10_000.0, 1.0),
        4,
        "far right is the end of the line"
    );
    let rectangles = view.range(2, 3);
    assert_eq!(rectangles.len(), 1);
    assert!(
        close(rectangles[0].x, stop) && close(rectangles[0].width, stop),
        "the tab's selection must cover exactly its widened advance"
    );
    document.select(ReadingPosition {
        anchor: 2,
        head: 3,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    assert_eq!(
        document.selected_text(),
        "\t",
        "copying a tab must yield the source tab"
    );
    document.select(ReadingPosition {
        anchor: 0,
        head: 4,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    assert_eq!(document.selected_text(), "ab\tc");
}
