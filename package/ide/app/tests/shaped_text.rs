//! Native-font geometry tests for mixed-script source rather than terminal cells.

/// Rasterization uses the same shaped rows and font faces.
use ide_app::{
    code_colors::CodeColors,
    tile_scene::{paint_tile, read::drawn_runs, tile_items},
};
/// Canonical document and shaped source-view interfaces.
use ide_app::{
    document::{Document, ReadingPosition},
    shaped_text::{TextShaper, Viewport},
};
/// Inspect actual glyph runs when checking baseline coherence.
use parley::PositionedLayoutItem;

/// Shared fixture geometry stays independent of a window server.
fn viewport() -> Viewport {
    return Viewport {
        first: 0,
        count: 10,
        width: 500.0,
        scale: 1.0,
        wrap: None,
        clip: None,
    };
}

/// Mixed CJK/Latin runs must share one baseline,
///  also matching other source lines.
#[test]
fn mixed_script_runs_share_the_source_baseline() {
    let document = Document::new("I am a big cat.\n猫 and e\u{301} are grapheme test cases.");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(), &[]);
    let baseline = view.rows[0].baseline;
    let mut mixed_runs = 0;
    for row in &view.rows {
        assert_eq!(row.baseline, baseline);
        for line in row.layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    assert!((run.baseline() + row.baseline_shift - baseline).abs() < 0.01);
                    if row.row == 1 {
                        mixed_runs += 1;
                    }
                }
            }
        }
    }
    assert!(
        mixed_runs >= 2,
        "test must exercise a real fallback-font run"
    );
}

/// Native caret geometry and pointer hit testing agree after a wide source character.
#[test]
fn caret_and_hit_test_share_mixed_script_advances() {
    let mut document = Document::new("猫 and a cat");
    document.select(ReadingPosition {
        anchor: 4,
        head: 4,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(), &[]);
    let caret = view.caret(&document);
    assert_eq!(view.hit(&document, 0, caret.x + 0.01), 4);
    assert!(caret.x > 0.0);
}

/// The line projection preserves source tabs rather than copying expanded spaces.
#[test]
fn selection_copies_source_not_projection() {
    let mut document = Document::new("a\t猫e\u{301}");
    document.select(ReadingPosition {
        anchor: 1,
        head: 5,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(), &[]);
    assert_eq!(document.selected_text(), "\t猫e\u{301}");
    assert!(!view.selection(&document).is_empty());
}

/// Source tile output contains glyph runs,
///  with bounded physical dimensions.
#[test]
fn tile_has_ink_and_requested_dimensions() {
    let document = Document::new("猫 and a cat");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(), &[]);
    let colors = CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
    let scene = paint_tile(&view, colors, 0.0)
        .expect("the test needs the shaped view to build a scene for a tile");
    assert_eq!(scene.width, 500);
    assert_eq!(scene.height, 24);
    let items = tile_items(&view, colors, 0.0).expect("items");
    let mut inked = false;
    for run in drawn_runs(&items) {
        if !run.ids.is_empty() && run.ink == colors.foreground {
            inked = true;
        }
    }
    assert!(inked);
}

/// Empty text still has a valid source caret and transparent tile.
#[test]
fn empty_shaped_source_is_valid() {
    let document = Document::new("");
    let mut shaper = TextShaper::new();
    let view = shaper.prepare(&document, viewport(), &[]);
    assert_eq!(view.hit(&document, 0, 0.0), 0);
    assert_eq!(view.caret(&document).x, 0.0);
}
