//! Markdown roles ask for a weight or the italic face as their second channel beside color,
//!  and neither may move a caret.

/// Production source font assets and shaping interfaces.
use ide_app::{
    code_colors::CodeColors,
    document::Document,
    font_asset::{CODE_ITALIC, CODE_ROMAN},
    selection_ink::contrast,
    shaped_text::{TextShaper, Viewport},
    source_style::{
        BOLD_ROLE, HEADING_ROLE, ITALIC_ROLE, LINK_ROLE, LINK_URL_ROLE, LIST_ROLE, QUOTE_ROLE,
        RAW_INLINE_ROLE, STRIKETHROUGH_ROLE, StyleSpan,
    },
    syntax::SCOPES,
    tile_scene::{read::drawn_runs, tile_items},
};
/// Actual font runs expose the selected font bytes and normalized variation coordinates.
use parley::PositionedLayoutItem;

/// One viewport line at scale one,
///  optionally wrapped to a width in logical pixels.
fn viewport(wrap: Option<f32>) -> Viewport {
    return Viewport {
        first: 0,
        count: 1,
        width: 800.0,
        scale: 1.0,
        wrap,
        clip: None,
    };
}

/// The heaviest weight among the faces,
///  or nothing without faces.
fn weight_of(faces: &[(bool, i16)]) -> Option<i16> {
    let mut heaviest: Option<i16> = None;
    for face in faces {
        let heavier = match heaviest {
            Some(held) => face.1 > held,
            None => true,
        };
        if heavier {
            heaviest = Some(face.1);
        }
    }
    return heaviest;
}

/// True when some channel of `color` is farther than four from the same channel of `hue`.
fn is_off_hue(color: &[u8; 3], hue: [u8; 3]) -> bool {
    for (drawn, wanted) in color.iter().zip(hue) {
        if drawn.abs_diff(wanted) > 4 {
            return true;
        }
    }
    return false;
}

/// The character range of the first occurrence of `word` in `source`.
fn span(source: &str, word: &str, style: u32) -> StyleSpan {
    let byte = source.find(word).expect("word is in the sample");
    let start = source[..byte].chars().count();
    return StyleSpan {
        start,
        end: start + word.chars().count(),
        style: style as usize,
    };
}

/// Whether every run keeps the roman face at the source weight.
fn all_plain(faces: &[(bool, i16)]) -> bool {
    for face in faces {
        if *face != (false, 0) {
            return false;
        }
    }
    return true;
}

/// Whether some run uses the real italic face.
fn any_italic(faces: &[(bool, i16)]) -> bool {
    for (italic, _) in faces {
        if *italic {
            return true;
        }
    }
    return false;
}

/// Whether some roman run is lighter than the source weight.
fn any_lighter_roman(faces: &[(bool, i16)]) -> bool {
    for (italic, weight) in faces {
        if !*italic && *weight < 0 {
            return true;
        }
    }
    return false;
}

/// What font a run of the first row used,
///  as the font bytes' first and last values and the normalized weight coordinate.
fn faces(source: &str, styles: &[StyleSpan]) -> Vec<(bool, i16)> {
    let document = Document::new(source);
    let view = TextShaper::new().prepare(&document, viewport(None), styles);
    let mut seen = Vec::new();
    for line in view.rows[0].layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let run = glyph_run.run();
                let italic = run.font().data.as_ref() == CODE_ITALIC;
                assert!(italic || run.font().data.as_ref() == CODE_ROMAN);
                let weight = run.normalized_coords().first().copied().unwrap_or(0);
                seen.push((italic, weight));
            }
        }
    }
    return seen;
}

/// The role numbers name the scopes the engine registers.
#[test]
fn role_numbers_name_the_registered_scopes() {
    let named = [
        (HEADING_ROLE, "markup.heading"),
        (BOLD_ROLE, "markup.bold"),
        (ITALIC_ROLE, "markup.italic"),
        (STRIKETHROUGH_ROLE, "markup.strikethrough"),
        (RAW_INLINE_ROLE, "markup.raw.inline"),
        (LINK_ROLE, "markup.link"),
        (LINK_URL_ROLE, "markup.link.url"),
        (LIST_ROLE, "markup.list"),
        (QUOTE_ROLE, "markup.quote"),
    ];
    for (role, scope) in named {
        assert_eq!(SCOPES[role as usize - 1], scope, "role {role}");
    }
}

/// Strong roles shape in a heavier instance of the roman face,
///  and the plain words around them in the regular instance.
#[test]
fn heavy_roles_shape_in_a_heavier_instance_of_the_roman_face() {
    let source = "plain strong plain";
    let plain = faces(source, &[]);
    assert!(all_plain(&plain));
    for role in [HEADING_ROLE, BOLD_ROLE, LIST_ROLE] {
        let styled = faces(source, &[span(source, "strong", role)]);
        let mut heavier = 0;
        for (italic, weight) in styled.iter() {
            if !*italic && *weight > 0 {
                heavier += 1;
            }
        }
        assert_eq!(heavier, 1, "role {role} makes one heavier run: {styled:?}");
    }
    let semibold = faces(source, &[span(source, "strong", LINK_ROLE)]);
    let bold = faces(source, &[span(source, "strong", BOLD_ROLE)]);
    assert!(weight_of(&semibold) > Some(0) && weight_of(&semibold) < weight_of(&bold));
}

/// Struck text is lighter than the source weight,
///  because a glyph-only painter cannot draw a line through it.
#[test]
fn struck_text_is_lighter_than_the_source_weight() {
    let source = "plain struck plain";
    let styled = faces(source, &[span(source, "struck", STRIKETHROUGH_ROLE)]);
    assert!(any_lighter_roman(&styled), "faces: {styled:?}");
}

/// Emphasis,
///  quotes,
///  and link destinations use the real italic face rather than a skew.
#[test]
fn italic_roles_use_the_real_italic_face() {
    let source = "plain words plain";
    for role in [ITALIC_ROLE, QUOTE_ROLE, LINK_URL_ROLE] {
        let styled = faces(source, &[span(source, "words", role)]);
        assert!(any_italic(&styled), "role {role}: {styled:?}");
    }
}

/// Roles that only choose a color leave the face alone.
#[test]
fn color_only_roles_keep_the_source_face() {
    let source = "plain `code` plain";
    let styled = faces(source, &[span(source, "code", RAW_INLINE_ROLE)]);
    assert!(all_plain(&styled), "faces: {styled:?}");
}

/// Carets sit where the unstyled line puts them,
///  on tabbed and wrapped lines with heavy,
///  italic,
///  and fallback-font text,
///  because the geometry paths shape without roles.
#[test]
fn faces_leave_every_caret_position_and_break_where_they_were() {
    let source = "\tstrong text\tafter a tab 猫🐈 and emphasis with a long tail of words that wrap around the width twice over";
    let styles = [
        span(source, "strong", BOLD_ROLE),
        span(source, "猫🐈", HEADING_ROLE),
        span(source, "emphasis", ITALIC_ROLE),
        span(source, "text", LINK_ROLE),
        span(source, "words", STRIKETHROUGH_ROLE),
        span(source, "tail", QUOTE_ROLE),
    ];
    let document = Document::new(source);
    for wrap in [None, Some(220.0), Some(400.0)] {
        let mut shaper = TextShaper::new();
        let view = shaper.prepare(&document, viewport(wrap), &styles);
        let styled = &view.rows[0];
        let plain = match wrap {
            Some(width) => shaper.wrapped_row(&document, 0, 1.0, width),
            None => shaper.row(&document, 0, 1.0),
        };
        assert_eq!(styled.visual_rows(), plain.visual_rows(), "wrap {wrap:?}");
        assert_eq!(styled.source_rows(), plain.source_rows(), "wrap {wrap:?}");
        for position in 0..=styled.source_len() {
            assert_eq!(
                styled.caret_x(position, 1.0),
                plain.caret_x(position, 1.0),
                "caret x at {position}, wrap {wrap:?}"
            );
        }
    }
}

/// The hue each Markdown role is painted in,
///  as the dark and the light scheme's ink.
const HUES: [(u32, [u8; 3], [u8; 3]); 9] = [
    (HEADING_ROLE, [130, 190, 235], [20, 86, 127]),
    (BOLD_ROLE, [225, 212, 157], [113, 83, 16]),
    (ITALIC_ROLE, [199, 146, 234], [118, 54, 164]),
    (STRIKETHROUGH_ROLE, [160, 168, 176], [85, 92, 101]),
    (RAW_INLINE_ROLE, [168, 204, 140], [50, 101, 32]),
    (LINK_ROLE, [130, 190, 235], [20, 86, 127]),
    (LINK_URL_ROLE, [240, 188, 122], [139, 77, 4]),
    (LIST_ROLE, [240, 188, 122], [139, 77, 4]),
    (QUOTE_ROLE, [160, 168, 176], [85, 92, 101]),
];

/// The ink of every glyph run a styled word draws,
///  as its straight color,
///  in the given scheme.
fn ink_of(role: u32, dark: bool) -> Vec<[u8; 3]> {
    let source = "Markdown";
    let document = Document::new(source);
    let styles = [span(source, source, role)];
    let view = TextShaper::new().prepare(&document, viewport(None), &styles);
    let colors = CodeColors {
        foreground: if dark {
            [240, 240, 240, 255]
        } else {
            [20, 20, 20, 255]
        },
        selected: [255, 255, 255, 255],
        dark,
    };
    let items = tile_items(&view, colors, 0.0).expect("items of a styled word");
    let mut found = Vec::new();
    for run in drawn_runs(&items) {
        // A run is drawn in one straight ink; the GPU's coverage only fades its edges.
        if !run.ids.is_empty() {
            found.push([run.ink[0], run.ink[1], run.ink[2]]);
        }
    }
    return found;
}

/// Each role draws its word in its own hue in both schemes,
///  and the hues stay legible on the scheme's background.
/// The backgrounds are assumed to be near black (32) in the dark scheme and white in the light scheme;
///  the window's palette decides the real ones.
#[test]
fn markdown_roles_are_painted_in_their_hue_in_both_schemes() {
    for (role, dark_hue, light_hue) in HUES {
        for (dark, hue, background) in [
            (true, dark_hue, [32, 32, 32, 255]),
            (false, light_hue, [255, 255, 255, 255]),
        ] {
            let scheme = if dark { "dark" } else { "light" };
            let found = ink_of(role, dark);
            assert!(
                !found.is_empty(),
                "role {role}, {scheme}: nothing was drawn"
            );
            let mut off = 0;
            for color in found.iter() {
                if is_off_hue(color, hue) {
                    off += 1;
                }
            }
            assert_eq!(
                off,
                0,
                "role {role}, {scheme}: {off} of {} runs are not the hue {hue:?}",
                found.len()
            );
            let ratio = contrast([hue[0], hue[1], hue[2], 255], background);
            eprintln!(
                "role {role}, {scheme}: hue {hue:?} has contrast {ratio:.2} on {background:?}"
            );
            assert!(
                ratio >= 4.5,
                "role {role}, {scheme}: contrast {ratio:.2} is below 4.5"
            );
        }
    }
}
