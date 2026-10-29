//! The pixels the GPU draws for wrapped lines,
//! selections,
//! clips and the wrap indicator,
//!  from scenes rendered on the hardware device and read back:
//!  selected ink on every visual row,
//!  a clipped tile that holds the pixels of its band,
//!  underlines on the rows they cross,
//!  ligature selection at fractional display scales,
//!  and the sign in its ink.
//! The paint items these scenes are made of are checked as data in `tests/`;
//!  this file shows that they land on the screen.

/// The tile the GPU draws,
/// read back without a window,
/// and how far two renders may differ.
use super::{
    gpu_snapshot::install_gpu_platform,
    gpu_tile::render_scene,
    tile_reads::{RENDER_NOISE, rendered_tile, same_buffers},
};
/// The marks that become underlines,
/// their layout,
/// and the document under test.
use ide_app::{
    annotation::{Mark, Visible},
    annotation_layout::{AnnotationColors, lay_out},
    code_colors::CodeColors,
    document::{Document, ReadingAffinity, ReadingPosition},
    language::diagnostics::Severity,
    row_map::{CODE_ROW, RowMap},
    shaped_text::{Clip, TextShaper, Viewport},
    wrap_sign::{SIGN_INK, Sign},
};
/// The pixels the GPU drew.
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// A long line of ordinary code that wraps at the widths used here.
const CODE: &str = "let value = compute_total(first_item, second_item); // a long comment that keeps going and going";

/// A viewport of `count` lines from `first`,
/// wrapped at `wrap` logical pixels.
fn viewport(first: usize, count: usize, wrap: f32) -> Viewport {
    return Viewport {
        first,
        count,
        width: wrap + 24.0,
        scale: 1.0,
        wrap: Some(wrap),
        clip: None,
    };
}

/// Words of the shape `word12 ` until the text holds `length` characters,
/// in one line without a terminator.
fn words(length: usize) -> String {
    let mut text = String::new();
    let mut number = 0;
    while text.len() < length {
        text.push_str(&format!("word{number} "));
        number += 1;
    }
    text.truncate(length);
    return text;
}

/// Black text on a transparent tile.
const PALETTE: CodeColors = CodeColors {
    foreground: [0, 0, 0, 255],
    selected: [255, 255, 255, 255],
    dark: false,
};

/// Selected ink reaches the glyphs of every visual row of a selected wrapped line,
/// and no unselected ink is left on any of them.
#[test]
fn selected_ink_reaches_the_pixels_of_continuation_rows() {
    install_gpu_platform();
    let mut document = Document::new(CODE);
    let end = document.text().len_chars();
    document.select(ReadingPosition {
        anchor: 0,
        head: end,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let view = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    assert!(view.rows[0].visual_rows() >= 3);
    let colors = CodeColors {
        foreground: [0, 0, 0, 255],
        selected: [255, 0, 0, 255],
        dark: false,
    };
    let pixels = rendered_tile(&view, colors, 0.0);
    let width = pixels.width() as usize;
    for index in 0..view.rows[0].visual_rows() {
        let top = index * CODE_ROW as usize;
        let mut red = 0;
        let mut black = 0;
        for y in top..top + CODE_ROW as usize {
            for x in 0..width {
                let pixel = pixels.as_slice()[y * width + x];
                if pixel.a > 128 {
                    if pixel.r > 128 && pixel.g < 64 {
                        red += 1;
                    } else if pixel.r < 64 {
                        black += 1;
                    }
                }
            }
        }
        assert!(red > 20, "row {index} has selected ink");
        assert_eq!(black, 0, "row {index} has no unselected ink left");
    }
}

/// The sign image carries ink in the sign color,
/// for a symbol and for the drawn arrow,
/// at both scales.
#[test]
fn the_sign_image_is_drawn_in_the_sign_color() {
    install_gpu_platform();
    let mut shaper = TextShaper::new();
    let chosen = shaper.choose_sign(1.0);
    for sign in [chosen, Sign::Arrow] {
        for scale in [1.0_f32, 2.0] {
            let scene = shaper.sign_scene(sign, scale, 12.0).expect("sign");
            assert_eq!(scene.height as f32, CODE_ROW * scale);
            let pixels = render_scene(&scene);
            let mut inked = 0;
            for pixel in pixels.as_slice() {
                if pixel.a > 0 {
                    inked += 1;
                    // The texture holds straight alpha: each covered texel carries the sign ink itself.
                    for (channel, expected) in [pixel.r, pixel.g, pixel.b].iter().zip(SIGN_INK) {
                        assert!(
                            channel.abs_diff(expected) <= 2,
                            "{sign:?} at {scale}: a channel is {channel} for alpha {}",
                            pixel.a
                        );
                    }
                }
            }
            assert!(inked > 4, "{sign:?} at scale {scale} has ink");
        }
    }
}

/// An underline across a wrap break is drawn on exactly the two visual rows it crosses.
#[test]
fn an_underline_across_a_wrap_break_is_drawn_on_its_two_rows() {
    install_gpu_platform();
    let document = Document::new(CODE);
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    let view = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    let second = view.rows[0]
        .visual_row_sources(1)
        .expect("second row")
        .start;
    let visible = Visible {
        blocks: Vec::new(),
        marks: vec![Mark {
            start: second - 3,
            end: second + 3,
            severity: Severity::Error,
        }],
    };
    let colors = AnnotationColors {
        hint: [1, 2, 3, 255],
        error: [200, 0, 0, 255],
        warning: [200, 100, 0, 255],
        information: [0, 0, 200, 255],
        suggestion: [90, 90, 90, 255],
    };
    let frame = lay_out(&view, &visible, &mut shaper, colors);
    let mut framed = shaper.prepare_rows(&document, viewport(0, 1, 200.0), &[], &plain);
    framed.annotations = Some(frame);
    let pixels = rendered_tile(&framed, PALETTE, 0.0);
    let width = pixels.width() as usize;
    let mut red_rows = Vec::new();
    for y in 0..pixels.height() as usize {
        let mut red = 0;
        for x in 0..width {
            let pixel = pixels.as_slice()[y * width + x];
            if pixel.r > 150 && pixel.g < 60 && pixel.a > 90 {
                red += 1;
            }
        }
        if red > 0 {
            red_rows.push(y / CODE_ROW as usize);
        }
    }
    red_rows.dedup();
    assert_eq!(
        red_rows,
        vec![0, 1],
        "the error ink stands on exactly the two rows the mark crosses"
    );
}

/// A clipped tile draws the pixels its band has in the unclipped tile of the same lines.
#[test]
fn a_clipped_tile_draws_the_pixels_of_its_band() {
    install_gpu_platform();
    let document = Document::new(&format!("short\n{}", words(9000)));
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(2);
    let whole = shaper.prepare_rows(&document, viewport(1, 1, 300.0), &[], &plain);
    let band = Clip {
        top: 10.0 * CODE_ROW,
        bottom: 30.0 * CODE_ROW,
    };
    let clipped = shaper.prepare_rows(
        &document,
        Viewport {
            clip: Some(band),
            ..viewport(1, 1, 300.0)
        },
        &[],
        &plain,
    );
    let full = rendered_tile(&whole, PALETTE, 0.0);
    let part = rendered_tile(&clipped, PALETTE, 0.0);
    let row = full.width() as usize;
    let from = band.top as usize * row;
    let to = band.bottom as usize * row;
    assert_eq!(part.as_slice().len(), to - from);
    // The band is cut out of the whole tile as a buffer of its own, to compare it as a picture.
    let mut cut = SharedPixelBuffer::<Rgba8Pixel>::new(part.width(), part.height());
    cut.make_mut_slice()
        .copy_from_slice(&full.as_slice()[from..to]);
    assert!(
        same_buffers(&part, &cut),
        "the clipped tile differs from its band of the whole tile"
    );
    let mut inked = false;
    for pixel in part.as_slice() {
        if pixel.a > 0 {
            inked = true;
        }
    }
    assert!(inked, "the band has ink");
}

/// At the real limit a line that no texture can hold is cut by the shaper and drawn in a clip,
/// at both display scales:
/// every code row of the band has ink.
#[test]
fn a_line_past_the_real_limit_is_drawn_inside_a_clip() {
    install_gpu_platform();
    let document = Document::new(&words(100_000));
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(1);
    for scale in [1.0_f32, 2.0] {
        let base = Viewport {
            scale,
            ..viewport(0, 1, 782.0)
        };
        let whole = shaper.prepare_rows(&document, base, &[], &plain);
        let rows = whole.rows[0].visual_rows();
        let top = (rows as f32 / 2.0).floor() * CODE_ROW;
        let band = Clip {
            top,
            bottom: top + 150.0 * CODE_ROW,
        };
        let clipped = shaper.prepare_rows(
            &document,
            Viewport {
                clip: Some(band),
                ..base
            },
            &[],
            &plain,
        );
        let pixels = rendered_tile(&clipped, PALETTE, 0.0);
        assert_eq!(pixels.height() as f32, 150.0 * CODE_ROW * scale);
        let row_pixels = (CODE_ROW * scale) as usize;
        let width = pixels.width() as usize;
        for code_row in 0..150 {
            let mut inked = false;
            for y in code_row * row_pixels..(code_row + 1) * row_pixels {
                for pixel in &pixels.as_slice()[y * width..(y + 1) * width] {
                    if pixel.a > 0 {
                        inked = true;
                    }
                }
            }
            assert!(
                inked,
                "scale {scale}: code row {code_row} of the band has no ink"
            );
        }
    }
}

/// Partial ligature selection keeps the glyph's own coverage at fractional display scales and horizontal origins,
/// and puts each ink on its own side of the selection edge,
/// which lies on a whole pixel.
#[test]
fn a_selected_ligature_keeps_its_coverage_and_puts_each_ink_on_its_side() {
    install_gpu_platform();
    let mut shaper = TextShaper::new();
    let colors = CodeColors {
        foreground: [255, 0, 0, 255],
        selected: [0, 255, 0, 255],
        dark: true,
    };
    let mut decided = 0;
    for scale in [1.0_f32, 1.25, 1.5, 2.0] {
        for horizontal in [0.0_f32, 0.25, 5.75] {
            let area = Viewport {
                first: 0,
                count: 1,
                width: 500.0,
                scale,
                wrap: None,
                clip: None,
            };
            let mut document = Document::new("===");
            let ordinary = shaper.prepare(&document, area, &[]);
            let baseline = rendered_tile(&ordinary, colors, horizontal);
            document.select(ReadingPosition {
                anchor: 1,
                head: 2,
                viewport: 0,
                affinity: ReadingAffinity::Downstream,
            });
            let selected = shaper.prepare(&document, area, &[]);
            let rectangle = selected.selections[0];
            let left = ((rectangle.x - horizontal) * scale).round();
            let right = ((rectangle.x + rectangle.width - horizontal) * scale).round();
            let image = rendered_tile(&selected, colors, horizontal);
            let width = image.width() as usize;
            for (index, (pixel, before)) in
                image.as_slice().iter().zip(baseline.as_slice()).enumerate()
            {
                assert!(
                    pixel.a.abs_diff(before.a) <= RENDER_NOISE,
                    "selection changed the coverage at scale {scale}, origin {horizontal}"
                );
                if pixel.a < 64 {
                    continue;
                }
                let x = (index % width) as f32;
                if x >= left && x + 1.0 <= right {
                    decided += 1;
                    assert!(
                        pixel.r < 64,
                        "red inside the selection at scale {scale}, origin {horizontal}"
                    );
                } else if x + 1.0 <= left || x >= right {
                    decided += 1;
                    assert!(
                        pixel.g < 64,
                        "green outside the selection at scale {scale}, origin {horizontal}"
                    );
                }
            }
        }
    }
    assert!(
        decided > 100,
        "the control must exercise real pixels of both inks ({decided})"
    );
}
