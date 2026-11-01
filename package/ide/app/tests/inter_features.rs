//! Audit the embedded Inter faces through the same Parley/fontique stack used by Slint.

/// Slint's shared-parley shaping path uses these same family,
///  size,
///  and weight properties.
use parley::{
    FontContext, FontFamily, FontFeature, FontFeatures, FontStyle, FontVariation, FontVariations,
    FontWeight, Layout, LayoutContext, PositionedLayoutItem, StyleProperty, setting::Tag,
};
/// Owned font bytes are shared with the font collection;
///  borrowed family text needs no allocation.
use std::{borrow::Cow, sync::Arc};

/// Both real variable faces must be chosen from the bundled bytes,
///  never synthesized or discovered on the host.
use ide_app::font_asset::{UI_ITALIC, UI_ROMAN};

/// Shape one UI label with explicit features and the registered real Inter faces.
fn label_with_style(
    text: &str,
    weight: f32,
    italic: bool,
    optical_size: Option<f32>,
    features: &[FontFeature],
) -> Layout<u32> {
    let mut fonts = FontContext::new();
    for bytes in [UI_ROMAN, UI_ITALIC] {
        let blob = parley::fontique::Blob::new(Arc::new(bytes));
        fonts.collection.register_fonts(blob, None);
    }
    let mut layouts = LayoutContext::new();
    // Slint's shared-parley UI path requests fractional layout rather than quantized metrics.
    let mut builder = layouts.ranged_builder(&mut fonts, text, 1.0, false);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed("Inter Variable"),
    )));
    builder.push_default(StyleProperty::FontSize(15.0));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(weight)));
    builder.push_default(StyleProperty::FontStyle(if italic {
        FontStyle::Italic
    } else {
        FontStyle::Normal
    }));
    let mut variations = Vec::new();
    if let Some(size) = optical_size {
        variations.push(FontVariation::new(Tag::new(b"opsz"), size));
    }
    builder.push_default(StyleProperty::FontVariations(FontVariations::List(
        Cow::Borrowed(&variations),
    )));
    builder.push_default(StyleProperty::FontFeatures(FontFeatures::List(
        Cow::Borrowed(features),
    )));
    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    return layout;
}

/// The native UI's ordinary request leaves the optical axis at its font default.
fn label(text: &str, weight: f32, features: &[FontFeature]) -> Layout<u32> {
    return label_with_style(text, weight, false, None, features);
}

/// Inspect substitutions without assuming that a ligature reduces the number of glyphs.
fn glyph_ids(layout: &Layout<u32>) -> Vec<u32> {
    let mut ids = Vec::new();
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(run) = item {
                for glyph in run.positioned_glyphs() {
                    ids.push(glyph.id);
                }
            }
        }
    }
    return ids;
}

/// Contextual alternates retain Inter's upstream default;
///  optional discretionary ligatures stay opt-in.
#[test]
fn inter_contextual_and_discretionary_features_keep_distinct_defaults() {
    let contextual = "3x9 12:34 3–8 +8+x -> --> => <->";
    let defaults = label(contextual, 400.0, &[]);
    let enabled = label(contextual, 400.0, &[FontFeature::new(Tag::new(b"calt"), 1)]);
    let disabled = label(contextual, 400.0, &[FontFeature::new(Tag::new(b"calt"), 0)]);
    assert_eq!(glyph_ids(&defaults), glyph_ids(&enabled));
    assert_ne!(glyph_ids(&defaults), glyph_ids(&disabled));
    let discretionary = "Difficult affine fjord interface";
    let normal = label(discretionary, 400.0, &[]);
    let off = label(
        discretionary,
        400.0,
        &[FontFeature::new(Tag::new(b"dlig"), 0)],
    );
    let on = label(
        discretionary,
        400.0,
        &[FontFeature::new(Tag::new(b"dlig"), 1)],
    );
    assert_eq!(glyph_ids(&normal), glyph_ids(&off));
    assert_ne!(glyph_ids(&normal), glyph_ids(&on));
}

/// Kerning uses a real off control,
///  rather than inferring shaping from a font-family label.
#[test]
fn inter_default_kerning_changes_real_advances() {
    let normal = label("AVATAR To WA", 400.0, &[]);
    let off = label(
        "AVATAR To WA",
        400.0,
        &[FontFeature::new(Tag::new(b"kern"), 0)],
    );
    assert!(
        (normal.width() - off.width()).abs() > 0.1,
        "kerning off must move the control"
    );
    eprintln!(
        "Inter kerning: default={} off={}",
        normal.width(),
        off.width()
    );
}

/// Intermediate weights and italics use real variable instances,
///  not synthetic emboldening or slant.
#[test]
fn inter_weights_and_italics_choose_real_variable_faces() {
    for italic in [false, true] {
        let expected = if italic { UI_ITALIC } else { UI_ROMAN };
        for weight in [100.0, 400.0, 537.5, 600.0, 900.0] {
            let shaped = label_with_style("Inter", weight, italic, None, &[]);
            let mut runs = 0;
            for line in shaped.lines() {
                for item in line.items() {
                    if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                        let run = glyph_run.run();
                        assert!(
                            run.font().data.as_ref() == expected,
                            "wrong Inter face for weight {weight}, italic={italic}"
                        );
                        assert!(!run.synthesis().embolden());
                        assert!(run.synthesis().skew().is_none());
                        // Harfrust represents an all-default instance with an empty coordinate slice.
                        if weight == 400.0 {
                            for value in run.normalized_coords().iter() {
                                assert_eq!(*value, 0);
                            }
                        } else {
                            assert_eq!(run.normalized_coords().len(), 2);
                            assert_ne!(run.normalized_coords()[1], 0);
                        }
                        runs += 1;
                    }
                }
            }
            assert!(runs > 0);
        }
    }
}

/// Optical size is a genuine axis;
///  the current toolkit default must not be mislabeled automatic sizing.
#[test]
fn inter_optical_size_changes_layout_with_an_explicit_control() {
    for italic in [false, true] {
        let default = label_with_style("Typography", 400.0, italic, None, &[]);
        let text = label_with_style("Typography", 400.0, italic, Some(14.0), &[]);
        let display = label_with_style("Typography", 400.0, italic, Some(32.0), &[]);
        assert!((default.width() - text.width()).abs() < 0.01);
        assert!((text.width() - display.width()).abs() > 0.1);
    }
}

/// Tabular figures work as an explicit feature,
///  while proportional figures remain distinguishable.
#[test]
fn inter_tabular_numbers_have_a_proportional_control() {
    let tabular = [FontFeature::new(Tag::new(b"tnum"), 1)];
    let proportional = [FontFeature::new(Tag::new(b"pnum"), 1)];
    let ones = label("1111", 400.0, &tabular);
    let eights = label("8888", 400.0, &tabular);
    assert!((ones.width() - eights.width()).abs() < 0.01);
    let narrow = label("1111", 400.0, &proportional);
    let wide = label("8888", 400.0, &proportional);
    assert!((narrow.width() - wide.width()).abs() > 0.1);
}
