//! Selections over a tall tile on the hardware GPU:
//!  the glyphs of a tile with dozens of selected rows are drawn,
//!  and a selection of a whole tile draws the same coverage as no selection.
//!
//! Vello gives every tile of the picture that a clip covers its own slot in a buffer of fixed size (2,097,152 slots),
//!  and when the buffer is too small the renderer draws nothing and still reports success.
//!  A selected row is a clip,
//!  so the number of rows a tile can hold selected depends on how tall the clip of one row is:
//!  as tall as the texture it cost about 36,700 slots per row on a tile 1113 by 8160 pixels,
//!  and from 58 selected rows on the tile came out blank (all texels zero).

/// The tile the GPU draws,
///  read back without a window,
///  the device the harness opens,
///  and how far two renders may differ.
use super::{
    gpu_snapshot::{install_gpu_platform, install_uncapped_gpu_platform},
    tile_reads::{RENDER_NOISE, rendered_tile, same_pixels},
};
/// The document with a selection,
///  the rows of a tile,
///  and the shaper that lays them out.
use ide_app::{
    code_colors::CodeColors,
    document::{Document, ReadingAffinity, ReadingPosition},
    row_map::{CODE_ROW, RowMap},
    shaped_text::{TextShaper, Viewport, texture_dimension, texture_fits},
};
/// The pixels the GPU drew.
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Black text,
/// red selected text,
/// on a transparent tile:
/// the two inks differ,
/// so every selected glyph run is clipped.
const RED_SELECTED: CodeColors = CodeColors {
    foreground: [0, 0, 0, 255],
    selected: [255, 0, 0, 255],
    dark: false,
};

/// Black text whose selected ink is the same black:
/// no run is clipped,
/// so it is the reference picture of the coverage.
const PLAIN: CodeColors = CodeColors {
    foreground: [0, 0, 0, 255],
    selected: [0, 0, 0, 255],
    dark: false,
};

/// The most inked texels a row may have and still count as holding no glyphs.
const BLANK_ROW: usize = 20;

/// `lines` lines of dense code,
/// each far narrower than the tile so that none wraps,
/// joined without a final terminator.
fn code(lines: usize) -> String {
    let mut text = String::new();
    for number in 0..lines {
        if number > 0 {
            text.push('\n');
        }
        text.push_str(&format!(
            "pub fn compute_{number}(first_item: u32, second_item: &str) -> Result<u32, String> {{ // {number}"
        ));
    }
    return text;
}

/// The pixels of a tile of `lines` rows `wrap + 24` pixels wide with its first `selected` rows selected,
/// read back from the GPU.
fn tile(
    lines: usize,
    selected: usize,
    wrap: f32,
    colors: CodeColors,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let mut document = Document::new(&code(lines));
    if selected > 0 {
        let head = if selected >= lines {
            document.text().len_chars()
        } else {
            document.text().line_to_char(selected)
        };
        document.select(ReadingPosition {
            anchor: 0,
            head,
            viewport: 0,
            affinity: ReadingAffinity::Downstream,
        });
    }
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(lines);
    let view = shaper.prepare_rows(
        &document,
        Viewport {
            first: 0,
            count: lines,
            width: wrap + 24.0,
            scale: 1.0,
            wrap: Some(wrap),
            clip: None,
        },
        &[],
        &plain,
    );
    assert_eq!(view.height as usize, lines * CODE_ROW as usize);
    assert!(texture_fits(u64::from(view.width), u64::from(view.height)));
    return rendered_tile(&view, colors, 0.0);
}

/// The first row (counting from 0) of `pixels` with `BLANK_ROW` or fewer inked texels,
/// or nothing when every row holds glyphs.
fn first_blank_row(pixels: &SharedPixelBuffer<Rgba8Pixel>) -> Option<usize> {
    let width = pixels.width() as usize;
    let rows = pixels.height() as usize / CODE_ROW as usize;
    for row in 0..rows {
        let start = row * CODE_ROW as usize * width;
        let band = &pixels.as_slice()[start..start + CODE_ROW as usize * width];
        let inked = band.iter().filter(|pixel| return pixel.a > 128).count();
        if inked <= BLANK_ROW {
            return Some(row);
        }
    }
    return None;
}

/// The count of texels whose coverage differs by more than the render noise between two reads of the same size.
fn coverage_differences(
    left: &SharedPixelBuffer<Rgba8Pixel>,
    right: &SharedPixelBuffer<Rgba8Pixel>,
) -> usize {
    assert_eq!(left.as_slice().len(), right.as_slice().len());
    let mut differing = 0;
    for (one, other) in left.as_slice().iter().zip(right.as_slice()) {
        if one.a.abs_diff(other.a) > RENDER_NOISE {
            differing += 1;
        }
    }
    return differing;
}

/// A selection of dozens of rows in a tile as tall as the test device allows keeps the glyphs of every row,
/// and the rows it selects look as they do in a selection of a few rows.
#[test]
fn many_selected_rows_in_a_tall_tile_keep_their_glyphs_and_selected_ink() {
    install_gpu_platform();
    // 340 rows of 24 pixels is the tallest tile the device's 8192 pixels hold, 1113 pixels wide.
    let lines = 340;
    let wrap = 1089.0;
    // Past the 58 rows that used the whole buffer, with rows left unselected below them.
    let many = tile(lines, 100, wrap, RED_SELECTED);
    let few = tile(lines, 8, wrap, RED_SELECTED);
    assert_eq!(many.width(), 1113);
    assert_eq!(many.height(), 8160);
    assert_eq!(
        first_blank_row(&many),
        None,
        "every row of the tile with 100 selected rows holds glyphs"
    );
    assert_eq!(first_blank_row(&few), None);
    // Rows 0 to 6 are selected in both tiles (the last selected row of the small one ends in the line terminator).
    let width = many.width() as usize;
    let shared = 7 * CODE_ROW as usize * width;
    assert!(
        same_pixels(&many.as_slice()[..shared], &few.as_slice()[..shared]),
        "the selected rows look the same in a selection of 100 rows and of 8"
    );
    // Positive control: row 20 is selected (red) in one tile and not (black) in the other.
    let from = 20 * CODE_ROW as usize * width;
    let to = 21 * CODE_ROW as usize * width;
    assert!(
        !same_pixels(&many.as_slice()[from..to], &few.as_slice()[from..to]),
        "a row selected in one tile only differs from the same row of the other"
    );
}

/// A selection of every row of a tile as tall as the test device allows draws the same coverage as no selection,
/// with the selected ink on every glyph.
#[test]
fn select_all_over_the_tallest_tile_of_the_test_device_draws_every_glyph() {
    install_gpu_platform();
    let lines = 340;
    let wrap = 1089.0;
    let everything = tile(lines, lines, wrap, RED_SELECTED);
    let reference = tile(lines, lines, wrap, PLAIN);
    assert_eq!(everything.height(), 8160);
    assert_eq!(first_blank_row(&everything), None);
    assert_eq!(
        coverage_differences(&everything, &reference),
        0,
        "the glyph coverage of the selected tile is the coverage of the unselected one"
    );
    // The selected ink is red on every row: some glyph texel of the last row is red and none is black.
    let width = everything.width() as usize;
    let last = (lines - 2) * CODE_ROW as usize * width;
    let band = &everything.as_slice()[last..last + CODE_ROW as usize * width];
    assert!(
        band.iter()
            .any(|pixel| return pixel.a > 200 && pixel.r > 200)
    );
    assert!(
        !band
            .iter()
            .any(|pixel| return pixel.a > 200 && pixel.r < 64)
    );
}

/// A selection of every row of the tallest tile the device's own limit allows draws every glyph:
/// 682 rows of 24 pixels,
/// 1024 pixels wide,
/// 64 MiB.
#[test]
fn select_all_over_the_tallest_tile_of_the_real_device_draws_every_glyph() {
    install_uncapped_gpu_platform();
    let limit = texture_dimension();
    if limit < 16384 {
        eprintln!(
            "this host's GPU offers textures of {limit} pixels on a side, not 16384: nothing to prove"
        );
        return;
    }
    let lines = 16384 / CODE_ROW as usize;
    let wrap = 1000.0;
    let everything = tile(lines, lines, wrap, RED_SELECTED);
    let reference = tile(lines, lines, wrap, PLAIN);
    assert_eq!(everything.width(), 1024);
    assert_eq!(everything.height() as usize, lines * CODE_ROW as usize);
    assert_eq!(first_blank_row(&everything), None);
    assert_eq!(coverage_differences(&everything, &reference), 0);
}
