//! The limit of a tile that is drawn across stacked textures,
//! in the device's texture size,
//! Vello's bin count,
//! and the textures a stack may have:
//!  the rules that size a clip,
//!  and the guard that cuts a frame to the limit when the lines are taller than the map the caller chose its clip from.
//! No window is opened.

/// Canonical source of the lines under test.
use ide_app::document::Document;
/// The vertical mapping the frames are placed by,
/// and the height of one code row.
use ide_app::row_map::{CODE_ROW, RowMap};
/// The shaper,
/// the viewport,
/// and the rules under test.
use ide_app::shaped_text::{
    TextShaper, Viewport, band_clip, fit_clip, set_texture_dimension, stack_cap, stack_fits,
    stack_rows, texture_dimension, texture_fits, texture_heights, tile_cap, tile_texture_rows,
};
/// The palette of the tile,
/// and the items whose ink is read back.
use ide_app::{
    code_colors::CodeColors,
    tile_scene::{Item, paint_stack, tile_items},
};

/// Display scales of ordinary and fractional displays.
const SCALES: [f32; 7] = [0.5, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0];

/// Raster widths in physical pixels:
/// a very narrow window,
/// ordinary windows,
/// the widths of the owner's report,
/// and widths near the one where the cap is a single code row.
const COLUMNS: [u64; 12] = [
    1, 24, 200, 782, 1354, 1920, 3840, 7680, 100_000, 600_000, 699_000, 1_000_000,
];

/// Heights in logical pixels that a window asks a tile for:
/// nothing,
/// three views of an ordinary window,
/// and three views of a window far taller than one texture.
const WANTED: [f32; 3] = [0.0, 1980.0, 36_000.0];

/// A palette for painting.
fn colors() -> CodeColors {
    return CodeColors {
        foreground: [240, 240, 240, 255],
        selected: [255, 255, 255, 255],
        dark: true,
    };
}

/// The cap is never under one code row,
/// never over the stack's bound,
/// and a clip made for it fits the stack,
/// for every width,
/// scale,
/// and asked height,
/// including the widths where one code row is all the cap allows.
#[test]
fn a_clip_made_for_the_cap_fits_the_stack_at_every_width_and_scale() {
    for scale in SCALES {
        for columns in COLUMNS {
            for wanted in WANTED {
                let cap = tile_cap(columns, scale, wanted);
                assert!(
                    cap >= CODE_ROW,
                    "columns {columns} scale {scale} wanted {wanted}: cap {cap}"
                );
                assert!(
                    cap <= stack_cap(columns, scale),
                    "columns {columns} scale {scale} wanted {wanted}: cap {cap} is over the stack's"
                );
                // Sums stay exact in single precision at ten million pixels, which is above every cap but the narrowest.
                let Some(clip) = band_clip(0.0, 10_000_000.0, (5_000_000.0, 5_010_000.0), cap)
                else {
                    assert!(
                        stack_fits(columns, (10_000_000.0_f32 * scale).ceil() as u64),
                        "columns {columns} scale {scale}: ten million pixels were let through and do not fit"
                    );
                    continue;
                };
                let rows = ((clip.bottom - clip.top).max(CODE_ROW) * scale).ceil() as u64;
                // A width where even one code row is past the limit has nothing to cut to.
                let one_row = (CODE_ROW * scale).ceil() as u64;
                if stack_fits(columns, one_row) {
                    assert!(
                        stack_fits(columns, rows),
                        "columns {columns} scale {scale}: a clip of {rows} rows passes the limit"
                    );
                }
            }
        }
    }
}

/// A window the device can show always has its view held by the cap,
/// at every scale and width the device allows:
/// the tile never goes under the view and the room that the re-center keeps clear.
#[test]
fn the_cap_holds_the_view_of_the_tallest_window_the_device_can_show() {
    for side in [8192_u32, 16_384] {
        set_texture_dimension(side);
        for scale in [1.0_f32, 1.5, 2.0, 3.0] {
            for columns in [200_u64, 1358, 3840, 7259, 8192, 16_384] {
                if columns > u64::from(side) {
                    continue;
                }
                let view = side as f32 / scale;
                let cap = tile_cap(columns, scale, view * 3.0);
                assert!(
                    cap >= view + 4.0 * CODE_ROW,
                    "side {side} scale {scale} columns {columns}: cap {cap} does not hold a view of {view}"
                );
            }
        }
    }
    set_texture_dimension(8192);
}

/// The guard leaves a frame that fits as it is,
/// asked clip included,
/// and cuts one that does not around its middle,
/// to a clip that fits,
/// inside the lines,
/// on whole code rows.
#[test]
fn the_guard_keeps_a_fitting_frame_and_cuts_a_larger_one_around_its_middle() {
    for scale in SCALES {
        for columns in COLUMNS {
            if !stack_fits(columns, (CODE_ROW * scale).ceil() as u64 * 2) {
                continue;
            }
            let cap = stack_cap(columns, scale);
            let first_top = 480.0;
            for lines in [cap * 0.3, cap * 0.99, cap * 1.5, cap * 7.3] {
                let bottom = first_top + (lines / CODE_ROW).ceil() * CODE_ROW;
                let fitted = fit_clip(first_top, bottom, None, columns, scale);
                let whole = ((bottom - first_top) * scale).ceil() as u64;
                match fitted {
                    None => assert!(
                        stack_fits(columns, whole),
                        "columns {columns} scale {scale}: {lines} px passed as they are"
                    ),
                    Some(clip) => {
                        assert!(!stack_fits(columns, whole), "a fitting frame was cut");
                        let rows = ((clip.bottom - clip.top) * scale).ceil() as u64;
                        assert!(stack_fits(columns, rows), "the cut frame passes the limit");
                        assert!(clip.top >= 0.0 && clip.bottom <= bottom - first_top);
                        assert_eq!(clip.top % CODE_ROW, 0.0);
                        assert_eq!(clip.bottom % CODE_ROW, 0.0);
                        let middle = (bottom - first_top) / 2.0;
                        assert!(
                            clip.top <= middle && middle <= clip.bottom,
                            "columns {columns} scale {scale}: the cut [{}, {}] misses the middle {middle}",
                            clip.top,
                            clip.bottom
                        );
                    }
                }
            }
            // A clip that fits is kept, and one that does not is cut around its own middle.
            let asked = band_clip(
                first_top,
                first_top + cap * 6.0,
                (first_top, first_top + cap * 6.0),
                cap,
            );
            let kept = fit_clip(first_top, first_top + cap * 6.0, asked, columns, scale);
            assert_eq!(kept, asked, "a clip that fits was changed");
        }
    }
}

/// Lines that wrap to many rows,
/// shaped against a map that counts each of them as one row,
/// as the window's map does before its background pass reached them:
/// the frame is cut to the limit and paints,
/// where the clip the caller asked for (none) would have been refused.
#[test]
fn a_frame_taller_than_its_map_held_is_cut_to_the_limit_and_painted() {
    let line = "word ".repeat(60);
    let text = format!("{}\n", line.trim_end()).repeat(300);
    let document = Document::new(&text);
    let lines = document.text().len_lines();
    let map = RowMap::plain(lines);
    let mut shaper = TextShaper::new();
    let viewport = Viewport {
        first: 0,
        count: lines,
        width: 224.0,
        scale: 1.0,
        wrap: Some(200.0),
        clip: None,
    };
    let view = shaper.prepare_rows(&document, viewport, &[], &map);
    let whole = view.map.block_top(lines.min(300));
    assert!(
        !stack_fits(u64::from(view.width), whole as u64),
        "the fixture's lines ({whole} px at {} columns) are within the stack",
        view.width
    );
    assert!(
        stack_fits(u64::from(view.width), u64::from(view.height)),
        "the frame is {}x{}, past the limit",
        view.width,
        view.height
    );
    let clip = view.viewport.clip.expect("the frame was cut");
    assert_eq!(view.origin, clip.top);
    assert_eq!(view.height as f32, clip.bottom - clip.top);
    let items = tile_items(&view, colors(), 0.0).expect("a frame within the limit draws");
    let mut glyph_runs = 0;
    for item in &items {
        if let Item::Glyphs(run) = item
            && !run.glyphs.is_empty()
        {
            glyph_runs += 1;
        }
    }
    assert!(glyph_runs > 0, "the cut frame draws no glyph");
    let stack = paint_stack(&view, colors(), 0.0).expect("a frame within the limit builds a stack");
    assert_eq!((stack.width, stack.height), (view.width, view.height));
    let mut together = 0;
    for texture in &stack.textures {
        assert!(texture_fits(
            u64::from(texture.width),
            u64::from(texture.height)
        ));
        together += texture.height;
    }
    assert_eq!(together, view.height);
}

/// A caller's clip that is itself too tall is cut too,
/// and a clip that fits is left alone.
#[test]
fn a_clip_asked_for_that_passes_the_limit_is_cut_and_one_that_fits_is_kept() {
    let line = "word ".repeat(60);
    let text = format!("{}\n", line.trim_end()).repeat(300);
    let document = Document::new(&text);
    let lines = document.text().len_lines();
    let map = RowMap::plain(lines);
    let mut shaper = TextShaper::new();
    let mut viewport = Viewport {
        first: 0,
        count: lines,
        width: 224.0,
        scale: 1.0,
        wrap: Some(200.0),
        clip: Some(ide_app::shaped_text::Clip {
            top: 0.0,
            bottom: 90_000.0,
        }),
    };
    let tall = shaper.prepare_rows(&document, viewport, &[], &map);
    let cut = tall.viewport.clip.expect("a clip");
    assert!(cut.bottom - cut.top < 90_000.0);
    assert!(stack_fits(u64::from(tall.width), u64::from(tall.height)));
    let small = ide_app::shaped_text::Clip {
        top: 240.0,
        bottom: 1_200.0,
    };
    viewport.clip = Some(small);
    let kept = shaper.prepare_rows(&document, viewport, &[], &map);
    assert_eq!(kept.viewport.clip, Some(small));
}

/// A 2000 px tall window at display scale 2 on a device whose textures are at most 8192 px on a side:
/// the tile of three views (12,000 physical rows) is not cut,
/// it is two textures that together hold the whole view with overscan.
#[test]
fn a_tile_at_scale_two_in_a_2000_px_window_is_two_textures_on_a_device_limited_to_8192() {
    assert_eq!(
        texture_dimension(),
        8192,
        "the default is the smallest limit wgpu guarantees"
    );
    let scale = 2.0_f32;
    let view_height = 2000.0_f32;
    // The text area of a 1147 px wide window, in physical pixels.
    let columns = (1147.0_f32 * scale).ceil() as u64;
    let cap = tile_cap(columns, scale, view_height * 3.0);
    assert!(
        cap >= view_height * 3.0,
        "the cap {cap} px does not hold the three views that the window asks for"
    );
    // A whole-document tile of three views is not cut: it fits the stack.
    let first_top = 0.0;
    let bottom = view_height * 3.0;
    assert_eq!(
        fit_clip(first_top, bottom, None, columns, scale),
        None,
        "three views were cut"
    );
    let rows = (bottom * scale).ceil() as u64;
    assert!(!texture_fits(columns, rows), "one texture holds the tile");
    let heights = texture_heights(columns, rows);
    assert_eq!(heights.len(), 2, "heights {heights:?}");
    assert_eq!(u64::from(heights[0] + heights[1]), rows);
    assert_eq!(u64::from(heights[0]), tile_texture_rows(columns));
}

/// The device's texture size bounds a texture of the stack on its own when the bin count would allow more,
/// and the stack is as tall as its textures allow.
#[test]
fn the_texture_size_of_the_device_bounds_each_texture_of_the_stack() {
    set_texture_dimension(4096);
    // 200 columns are one bin wide: 256 bins would allow 65,536 rows; the device allows 4096.
    assert_eq!(tile_texture_rows(200), 4096);
    assert!(texture_fits(200, 4096) && !texture_fits(200, 4097));
    assert!(
        !texture_fits(4097, 1),
        "a tile wider than the texture size fits"
    );
    assert_eq!(texture_heights(200, 10_000), vec![4096, 4096, 1808]);
    assert!(stack_fits(200, 4 * 4096) && !stack_fits(200, 4 * 4096 + 1));
    set_texture_dimension(8192);
    assert!(tile_texture_rows(200) > 4096);
    assert_eq!(stack_rows(200), 4 * 8192);
}

/// The stack never holds fewer rows than the tallest window the device can show and the room for its re-center,
/// whatever the width:
/// at 16,384 columns the bin count allows 1024 rows to a texture and four textures are not enough.
#[test]
fn a_stack_is_never_under_the_tallest_window_and_its_room() {
    set_texture_dimension(16_384);
    assert_eq!(tile_texture_rows(16_384), 1024);
    assert_eq!(stack_rows(16_384), 16_384 + 1024);
    assert!(stack_fits(16_384, 16_384 + 1024) && !stack_fits(16_384, 16_384 + 1025));
    assert!(!stack_fits(16_385, 1));
    set_texture_dimension(8192);
}
