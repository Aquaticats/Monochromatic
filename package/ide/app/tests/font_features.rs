//! Font feature controls and ligature reading behavior use the production shaping path,
//!  and the paint items the GPU draws the code view from.

/// Canonical source and production geometry/rendering interfaces.
use ide_app::{
    code_colors::CodeColors,
    document::{Document, ReadingPosition},
    shaped_text::{ShapedView, TextShaper, Viewport},
    tile_scene::{read::drawn_runs, snapped, tile_items},
};
/// Typed OpenType settings and actual shaped glyph runs.
use parley::{FontFeature, PositionedLayoutItem, setting::Tag};

/// Source fixture geometry retains fractional display scaling in dedicated cases.
fn viewport() -> Viewport {
    return Viewport {
        first: 0,
        count: 1,
        width: 500.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
}

/// Compare glyph identities rather than assuming contextual ligatures reduce glyph count.
fn glyphs(view: &ShapedView) -> Vec<u32> {
    let mut result = Vec::new();
    for row in &view.rows {
        for line in row.layout.lines() {
            for item in line.items() {
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

/// The default source policy enables actual programming substitutions while retaining cell advances.
#[test]
fn jetbrains_programming_ligatures_have_a_real_off_control() {
    let document = Document::new("=== != => <= -> :: &&");
    let mut defaults = TextShaper::new();
    let mut disabled = TextShaper::with_features(vec![FontFeature::new(Tag::new(b"calt"), 0)]);
    let mut enabled = TextShaper::with_features(vec![FontFeature::new(Tag::new(b"calt"), 1)]);
    let normal = defaults.prepare(&document, viewport(), &[]);
    let off = disabled.prepare(&document, viewport(), &[]);
    let on = enabled.prepare(&document, viewport(), &[]);
    assert_ne!(
        glyphs(&normal),
        glyphs(&off),
        "the control must change actual glyph substitutions"
    );
    assert_eq!(glyphs(&normal), glyphs(&on));
    assert!((normal.rows[0].layout.width() - off.rows[0].layout.width()).abs() < 0.01);
}

/// Optional glyph choices work when explicitly requested;
///  they are not forced as new defaults.
#[test]
fn slashed_zero_changes_glyphs_not_source_or_advance() {
    let document = Document::new("0O");
    let mut defaults = TextShaper::new();
    let mut alternate = TextShaper::with_features(vec![FontFeature::new(Tag::new(b"zero"), 1)]);
    let normal = defaults.prepare(&document, viewport(), &[]);
    let changed = alternate.prepare(&document, viewport(), &[]);
    assert_ne!(glyphs(&normal), glyphs(&changed));
    assert!((normal.rows[0].layout.width() - changed.rows[0].layout.width()).abs() < 0.01);
    assert_eq!(document.text().to_string(), "0O");
}

/// Each source character inside an operator ligature remains a caret and copy boundary.
#[test]
fn ligature_interior_caret_hit_and_copy_remain_source_based() {
    let mut shaper = TextShaper::new();
    for source in ["===", "!=", "=>", "<=", "->"] {
        let mut document = Document::new(source);
        for position in 0..=source.len() {
            document.select(ReadingPosition {
                anchor: position,
                head: position,
                viewport: 0,
                affinity: ide_app::document::ReadingAffinity::Downstream,
            });
            let view = shaper.prepare(&document, viewport(), &[]);
            let caret = view.caret(&document);
            assert_eq!(
                view.hit(&document, 0, caret.x + 0.01),
                position,
                "{source} at {position}"
            );
        }
        document.select(ReadingPosition {
            anchor: 1,
            head: 2,
            viewport: 0,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
        assert_eq!(document.selected_text(), &source[1..2]);
    }
}

/// The ink of the ordinary glyphs and of the selected ones,
/// far apart so that a test tells them at a glance.
const COLORS: CodeColors = CodeColors {
    foreground: [255, 0, 0, 255],
    selected: [0, 255, 0, 255],
    dark: true,
};

/// Select the second character of `===` and shape it at `area`.
fn selected_ligature(shaper: &mut TextShaper, area: Viewport) -> (Document, ShapedView) {
    let mut document = Document::new("===");
    document.select(ReadingPosition {
        anchor: 1,
        head: 2,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let selected = shaper.prepare(&document, area, &[]);
    return (document, selected);
}

/// Partial selection recolors only selected glyph area without breaking the shaped operator:
/// the same glyphs are drawn twice,
/// in the selection ink inside the selection rectangle and in the ordinary ink outside.
#[test]
fn partial_ligature_selection_preserves_geometry_and_clips_ink() {
    let mut shaper = TextShaper::new();
    let unselected_document = Document::new("===");
    let unselected = shaper.prepare(&unselected_document, viewport(), &[]);
    let (_document, selected) = selected_ligature(&mut shaper, viewport());
    assert_eq!(
        glyphs(&unselected),
        glyphs(&selected),
        "selection must not change ligature shaping"
    );
    let rectangle = selected.selections[0];
    let items = tile_items(&selected, COLORS, 0.0).expect("selected ligature items");
    let runs = drawn_runs(&items);
    let mut inside = 0;
    let mut outside = 0;
    for run in &runs {
        let (clip_left, clip_right) = run.clip.expect("a selected ligature is drawn clipped");
        assert_eq!(
            run.ids,
            glyphs(&selected),
            "every clipped copy draws the whole shaped operator"
        );
        if run.ink == COLORS.selected {
            inside += 1;
            assert!(
                (clip_left - rectangle.x).abs() <= 0.5
                    && (clip_right - (rectangle.x + rectangle.width)).abs() <= 0.5,
                "selected ink is clipped to [{clip_left}, {clip_right}] instead of the selection [{}, {}]",
                rectangle.x,
                rectangle.x + rectangle.width
            );
        } else {
            assert_eq!(run.ink, COLORS.foreground);
            outside += 1;
            assert!(
                clip_right <= rectangle.x + 0.5 || clip_left >= rectangle.x + rectangle.width - 0.5,
                "ordinary ink is clipped to [{clip_left}, {clip_right}], which reaches into the selection"
            );
        }
    }
    assert!(
        inside > 0 && outside > 1,
        "the selected half and both ordinary halves must be drawn (inside {inside}, outside {outside})"
    );
}

/// Partial selection keeps its clip edges at fractional DPI and horizontal origins.
#[test]
fn fractional_ligature_selection_keeps_clip_edges() {
    let mut shaper = TextShaper::new();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        for horizontal in [0.0, 0.25, 5.75] {
            let mut area = viewport();
            area.scale = scale;
            let (_document, selected) = selected_ligature(&mut shaper, area);
            let rectangle = selected.selections[0];
            // The clip edges are moved to whole pixels, so that no pixel is covered by two copies of a glyph.
            let (left, right) = snapped(
                (rectangle.x - horizontal) * scale,
                (rectangle.x + rectangle.width - horizontal) * scale,
            );
            let items =
                tile_items(&selected, COLORS, horizontal).expect("fractional selected glyph");
            let mut edges = 0;
            for run in drawn_runs(&items) {
                let (clip_left, clip_right) = run.clip.expect("clipped");
                if run.ink == COLORS.selected {
                    edges += 1;
                    assert!(
                        (clip_left - left).abs() < 0.01 && (clip_right - right).abs() < 0.01,
                        "scale {scale}, origin {horizontal}: clip [{clip_left}, {clip_right}] is not [{left}, {right}]"
                    );
                } else {
                    assert!(
                        clip_right <= left + 0.01 || clip_left >= right - 0.01,
                        "scale {scale}, origin {horizontal}: ordinary ink reaches into the selection"
                    );
                }
            }
            assert_eq!(
                edges, 1,
                "one selected copy at scale {scale}, origin {horizontal}"
            );
        }
    }
}
