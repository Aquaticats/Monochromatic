//! Real variable coordinates and italic faces must survive source shaping,
//!  the GPU's glyph runs,
//!  and copying.

/// Production source font assets and typography interfaces.
use ide_app::{
    code_colors::CodeColors,
    document::{Document, ReadingPosition},
    font_asset::{CODE_ITALIC, CODE_ROMAN},
    shaped_text::{ShapedView, TextShaper, Viewport},
    source_typography::SourceTypography,
    tile_scene::{Item, tile_items},
};
/// Actual font runs expose selected font identity and normalized variation coordinates.
use parley::PositionedLayoutItem;

/// Shared source geometry for comparisons at a fractional scale.
fn viewport() -> Viewport {
    return Viewport {
        first: 0,
        count: 1,
        width: 400.0,
        scale: 1.25,
        wrap: None,
        clip: None,
    };
}

/// Read one primary run's cache identity and weight coordinates from an actual shaped result.
fn instance(view: &ShapedView) -> (u64, Vec<i16>) {
    for line in view.rows[0].layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                return (
                    run.run().font().data.id(),
                    run.run().normalized_coords().to_vec(),
                );
            }
        }
    }
    panic!("fixture must produce a real font run");
}

/// Continuous weights and italic requests select the exact corresponding variable font bytes.
#[test]
fn code_weights_and_italics_use_real_faces_without_synthesis() {
    let document = Document::new("office === != => 0");
    for italic in [false, true] {
        let expected = if italic { CODE_ITALIC } else { CODE_ROMAN };
        for weight in [100.0, 400.0, 527.5, 800.0] {
            let typography = SourceTypography {
                weight,
                italic,
                ..SourceTypography::default()
            };
            let view = TextShaper::with_typography(typography)
                .expect("supported variable instance")
                .prepare(&document, viewport(), &[]);
            let mut runs = 0;
            for line in view.rows[0].layout.lines() {
                for item in line.items() {
                    if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                        let run = glyph_run.run();
                        assert!(run.font().data.as_ref() == expected);
                        assert!(!run.synthesis().embolden());
                        assert!(run.synthesis().skew().is_none());
                        // Empty normalized coordinates are the upstream representation of default axes.
                        if weight == 400.0 {
                            for value in run.normalized_coords().iter() {
                                assert_eq!(*value, 0);
                            }
                        } else {
                            assert_eq!(run.normalized_coords().len(), 1);
                            assert_ne!(run.normalized_coords()[0], 0);
                        }
                        runs += 1;
                    }
                }
            }
            assert!(runs > 0);
        }
    }
}

/// Two weights share one font blob and glyph ids,
/// so only the variation coordinates of each glyph run tell the GPU which outlines to draw.
#[test]
fn variable_weight_reaches_the_gpu_as_variation_coordinates() {
    let document = Document::new("office === != => 0");
    let normal = TextShaper::new().prepare(&document, viewport(), &[]);
    let mut heavier = TextShaper::with_typography(SourceTypography {
        weight: 725.5,
        ..SourceTypography::default()
    })
    .expect("intermediate variable weight");
    let heavy = heavier.prepare(&document, viewport(), &[]);
    let normal_instance = instance(&normal);
    let heavy_instance = instance(&heavy);
    assert_eq!(
        normal_instance.0, heavy_instance.0,
        "cache control must reuse the same font identity"
    );
    assert_ne!(normal_instance.1, heavy_instance.1);
    let colors = CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
    // The weight reaches Vello as the variation coordinates of each glyph run.
    let normal_coords = drawn_coords(&normal, colors);
    let heavy_coords = drawn_coords(&heavy, colors);
    assert!(!normal_coords.is_empty() && !heavy_coords.is_empty());
    assert_ne!(
        normal_coords, heavy_coords,
        "the weight must change the variation coordinates the GPU draws with"
    );
}

/// The variation coordinates of every glyph run the tile draws,
/// in drawing order.
fn drawn_coords(view: &ShapedView, colors: CodeColors) -> Vec<Vec<i16>> {
    let mut found = Vec::new();
    for item in tile_items(view, colors, 0.0).expect("items") {
        if let Item::Glyphs(run) = item {
            found.push(run.coords.clone());
        }
    }
    return found;
}

/// Genuine italic ligatures retain per-character geometry and exact source copying.
#[test]
fn italic_ligatures_keep_source_positions() {
    let mut document = Document::new("===");
    let mut shaper = TextShaper::with_typography(SourceTypography {
        weight: 537.5,
        italic: true,
        ..SourceTypography::default()
    })
    .expect("real italic variable instance");
    for position in 0..=3 {
        document.select(ReadingPosition {
            anchor: position,
            head: position,
            viewport: 0,
            affinity: ide_app::document::ReadingAffinity::Downstream,
        });
        let view = shaper.prepare(&document, viewport(), &[]);
        let caret = view.caret(&document);
        assert_eq!(view.hit(&document, 0, caret.x + 0.01), position);
    }
    document.select(ReadingPosition {
        anchor: 1,
        head: 2,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    assert_eq!(document.selected_text(), "=");
}

/// Unsupported or non-finite weights never become layout coordinates or cache keys.
#[test]
fn invalid_source_weights_are_rejected() {
    for weight in [0.0, 99.0, 801.0, f32::NAN, f32::INFINITY] {
        assert!(
            TextShaper::with_typography(SourceTypography {
                weight,
                ..SourceTypography::default()
            })
            .is_err()
        );
    }
}
